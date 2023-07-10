use std::iter;

use wgpu::{util::DeviceExt, TextureFormat};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use async_std::task::block_on;

use crate::{texture::{self, Texture}, vertices::block_vertex::BlockVertex, meshes::Meshes, pipelines::{bind_group_layout::{texture::get_texture_bind_group_layout, camera::get_camera_bind_group_layout}, new_pipeline}};

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4, /* padding */ 0];
pub const IS_LINE: bool = false;
const PRIMITIVE_TOPOLOGY: wgpu::PrimitiveTopology = match IS_LINE {
    true => wgpu::PrimitiveTopology::LineList,
    false => wgpu::PrimitiveTopology::TriangleList,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CompareFunction {
    Undefined = 0,
    Never = 1,
    Less = 2,
    Equal = 3,
    LessEqual = 4,
    Greater = 5,
    NotEqual = 6,
    GreaterEqual = 7,
    Always = 8,
}


pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    diffuse_texture: texture::Texture,
    diffuse_bind_group: wgpu::BindGroup,
    window: Window,
    camera_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
    depth_texture: Texture,
    multisampled_framebuffer: wgpu::TextureView,
    sample_count: u32,
}

impl State {
    pub fn new(window: Window, proj_view: &[[f32; 4]; 4]) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            dx12_shader_compiler: Default::default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let power_preference = wgpu::PowerPreference::HighPerformance;
        let adapter = block_on(instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })).unwrap();
        println!("{:?}", adapter.get_info());
        let (device, queue) = block_on(adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None, // Trace path
            )).unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![surface_format],
        };
        surface.configure(&device, &config);


        let sample_flags = adapter
            .get_texture_format_features(config.view_formats[0])
            .flags;

        let max_sample_count = {
            if sample_flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X16) {
                4
            } else if sample_flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X8) {
                4
            } else if sample_flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X4) {
                4
            } else if sample_flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X2) {
                2
            } else {
                1
            }
        };
        let sample_count = max_sample_count;

        let diffuse_texture = texture::Texture::image_array(&device, &queue, &[
            "./assets/blocks/0_no_texture.png",
            "./assets/blocks/1_block.png",
            "./assets/blocks/2_block.png",
            "./assets/blocks/marble.png",
            "./assets/blocks/iron_ore.png",
            "./assets/blocks/top.png",
            "./assets/blocks/green.png",
            "./assets/blocks/conveyor.png"], None, sample_count).unwrap();
        // let diffuse_texture = texture::Texture::image_array_arr(&device, &queue, &[
        //     "./assets/blocks/1t_block.png","./assets/blocks/1t_block.png"], None).unwrap();

        let texture_bind_group_layout = get_texture_bind_group_layout(&device, wgpu::TextureViewDimension::D2Array);

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(proj_view),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let camera_bind_group_layout = get_camera_bind_group_layout(&device);
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        let multisampled_framebuffer =
            texture::Texture::create_multisampled_framebuffer(&device, &config, sample_count);
        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture", sample_count);

        let render_pipeline = new_pipeline(
            &device, 
            &[&texture_bind_group_layout,
             &camera_bind_group_layout],
            &[BlockVertex::desc()],
            &shader,
            config.format,
            PRIMITIVE_TOPOLOGY,
            sample_count);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            diffuse_texture,
            diffuse_bind_group,
            window,

            camera_bind_group,
            camera_buffer,

            depth_texture,
            multisampled_framebuffer,
            sample_count
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture", self.sample_count);
            self.multisampled_framebuffer =
                texture::Texture::create_multisampled_framebuffer(&self.device, &self.config, self.sample_count);
        }
    }
    pub fn update(&mut self, proj_view: &[[f32; 4]; 4]) {
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(proj_view));
    }

    pub fn render(&mut self, meshes: &Meshes) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let clear_color = wgpu::Color {r: 0.18, g: 0.525, b: 0.87, a: 1.0};
            let rpass_color_attachment = if self.sample_count == 1 {
                wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: true,
                    },
                }
            } else {
                wgpu::RenderPassColorAttachment {
                    view: &self.multisampled_framebuffer,
                    resolve_target: Some(&view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        // Storing pre-resolve MSAA data is unnecessary if it isn't used later.
                        // On tile-based GPU, avoid store can reduce your app's memory footprint.
                        store: false,
                    },
                }
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(rpass_color_attachment)],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            // render_pass.set_vertex_buffer(0, meshes.block()[0].as_ref().unwrap().0.slice(..));
            // println!("{:?}", meshes.block()[0].as_ref().unwrap().0);
            // render_pass.set_index_buffer(meshes.block()[0].as_ref().unwrap().1.slice(..), wgpu::IndexFormat::Uint16);
            // render_pass.draw_indexed(0..meshes.block()[0].as_ref().unwrap().3 as u32, 0, 0..1);
            // render_pass.draw(0..meshes.block()[0].as_ref().unwrap().2 as u32, 0..1);
            meshes.block().iter().for_each(move |mesh| {
                if let Some(mesh) = mesh {
                    render_pass.set_vertex_buffer(0, mesh.0.slice(..));
                    render_pass.set_index_buffer(mesh.1.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..mesh.3 as u32, 0, 0..1);
                }
            });
            // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // // render_pass.draw(0..vertex_len as u32, 0..1);
            // render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            // render_pass.draw_indexed(0..index_len as u32, 0, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }


    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
}