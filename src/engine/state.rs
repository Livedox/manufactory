use std::{iter, collections::HashMap, rc::Rc, sync::Arc};

use egui::{FontDefinitions, vec2};

use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use wgpu::{util::DeviceExt, TextureFormat, TextureFormatFeatureFlags, Adapter};
use winit::window::Window;

use crate::{
vertices::{
    block_vertex::BlockVertex,
    model_vertex::ModelVertex,
    model_instance::ModelInstance,
    animated_model_instance::AnimatedModelInstance,
    animated_model_vertex::AnimatedModelVertex,
    selection_vertex::SelectionVertex
},
meshes::Meshes,
pipelines::{
    bind_group_layout::{
        texture::get_texture_bind_group_layout,
        camera::get_camera_bind_group_layout,
    },
new_pipeline
},
my_time::Time,
models::{
    load_model::load_models,
    model::Model,
    load_animated_model::load_animated_models,
    animated_model::AnimatedModel
},
world::sun::Sun, rev_qumark, engine::{bind_group, bind_group_buffer::BindGroupBuffer, shaders::{Shaders, self}}, camera
};


use crate::engine::texture::TextureAtlas;
use super::texture::{self};
use super::bind_group_layout;

pub const IS_LINE: bool = false;
const PRIMITIVE_TOPOLOGY: wgpu::PrimitiveTopology = match IS_LINE {
    true => wgpu::PrimitiveTopology::LineList,
    false => wgpu::PrimitiveTopology::TriangleList,
};

const MAX_SAMPLE_COUNT: u32 = 4;

fn get_supported_multisample_count(surface_format: &TextureFormat, sample_flags: &TextureFormatFeatureFlags) -> Vec<u32> {
    let sample: [u32; 5] = [1, 2, 4, 8, 16];
    let surface_flags = surface_format.guaranteed_format_features(wgpu::Features::empty()).flags;

    sample.into_iter().filter(|a| {
        surface_flags.sample_count_supported(*a)
        && sample_flags.sample_count_supported(*a)
    }).collect()
}


async fn request_adapter(instance: &wgpu::Instance, surface: &wgpu::Surface, power: wgpu::PowerPreference)
    -> Option<wgpu::Adapter>
{
    rev_qumark!(instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: power,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await);
    // Why is a second search needed?
    // https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#state-new
    instance.enumerate_adapters(wgpu::Backends::all())
        .find(|adapter| {
            adapter.is_surface_supported(&surface)
        })
}

/// REMEMBER
/// When the number of block textures exceeds 256, change the "max_texture_array_layers" parameter in the Limits.
async fn request_device(adapter: &Adapter) -> Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError> {
    adapter.request_device(
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
        None,
    ).await
}

pub struct Info {
    name: String,
    vendor: String,

}

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    model_pipeline: wgpu::RenderPipeline,
    block_texture: texture::Texture,
    block_texutre_bg: wgpu::BindGroup,
    window: Arc<Window>,
    camera: BindGroupBuffer,

    depth_texture: texture::Texture,
    multisampled_framebuffer: wgpu::TextureView,
    sample_count: u32,

    pub egui_platform: Platform,
    egui_rpass: RenderPass,

    models: HashMap<String, Model>,
    pub animated_models: HashMap<String, AnimatedModel>,

    animated_model_pipeline: wgpu::RenderPipeline,
    pub animated_model_buffer: wgpu::Buffer,
    pub animated_model_layout: wgpu::BindGroupLayout,

    transport_belt_pipeline: wgpu::RenderPipeline,
    time: BindGroupBuffer,

    crosshair_render_pipeline: wgpu::RenderPipeline,
    crosshair_u_ar_scale: BindGroupBuffer,

    sun: BindGroupBuffer,

    pub texture_atlas: Arc<TextureAtlas>,

    pub selection_vertex_buffer: Option<wgpu::Buffer>,
    selection_pipeline: wgpu::RenderPipeline,

    is_ui_interaction: bool,
    is_crosshair: bool,
}

impl State {
    pub async fn new(window: Arc<Window>, proj_view: &[[f32; 4]; 4]) -> Self {
        let size = window.inner_size();
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        for i in instance.enumerate_adapters(wgpu::Backends::all()) {
            println!("{:?} {:?}", i.get_info().device_type, i.get_info().name);
        }
        
        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(window.as_ref()) }.unwrap();

        let power_preference = wgpu::PowerPreference::HighPerformance;
        let adapter = request_adapter(&instance, &surface, power_preference)
            .await.expect("Failed to request adapter!");
        println!("{:?}", adapter.get_info());
        let (device, queue) = request_device(&adapter)
            .await.expect("Failed to request device");

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
        let surface_present_mode = surface_caps.present_modes[0];
        let surface_alpha_mode = surface_caps.alpha_modes[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_present_mode,
            alpha_mode: surface_alpha_mode,
            view_formats: vec![surface_format],
        };
        println!("format: {:?}, present_mode: {:?}, alpha_mode: {:?}", surface_format, surface_present_mode, surface_alpha_mode);
        surface.configure(&device, &config);


        let sample_flags = adapter
            .get_texture_format_features(config.view_formats[0])
            .flags;
        let samples_count = get_supported_multisample_count(&surface_format, &sample_flags);
        let sample_count = if samples_count.contains(&MAX_SAMPLE_COUNT) {
            MAX_SAMPLE_COUNT.min(samples_count.into_iter().max().unwrap())
        } else {
            samples_count.into_iter().max().unwrap()
        };
        println!("Sample count X{}", sample_count);

        let egui_platform = Platform::new(PlatformDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        let mut egui_rpass = RenderPass::new(&device, surface_format, 1);
        

        let texture_atlas = TextureAtlas::new(&mut egui_rpass, &device, &queue, "./assets/items/items.png", 4);
        let style = egui::Style {
            spacing: egui::style::Spacing { item_spacing: vec2(0.0, 0.0), ..Default::default() },
            ..Default::default()
        };
        egui_platform.context().set_style(style);
       

        let block_texture = texture::Texture::image_array(&device, &queue, &[
            "./assets/blocks/0_no_texture.png",
            "./assets/blocks/1_block.png",
            "./assets/blocks/2_block.png",
            "./assets/blocks/marble.png",
            "./assets/blocks/iron_ore.png",
            "./assets/blocks/top.png",
            "./assets/blocks/green.png",
            "./assets/blocks/conveyor.png",
            "./assets/blocks/box.png",
            "./assets/blocks/rock.png",
            "./assets/debug/0.png",
            "./assets/debug/1.png",
            "./assets/debug/2.png",
            "./assets/debug/3.png",
            "./assets/debug/4.png",
            "./assets/debug/5.png",
            "./assets/debug/6.png",
            "./assets/debug/7.png",
            "./assets/debug/8.png",
            "./assets/debug/9.png",
            "./assets/debug/10.png",
            "./assets/debug/11.png",
            "./assets/debug/12.png",
            "./assets/debug/13.png",
            "./assets/debug/14.png",
            "./assets/debug/15.png",], None).unwrap();

        let block_texture_bgl = bind_group_layout::texture::get(&device, wgpu::TextureViewDimension::D2Array, "1");
        let block_texutre_bg = bind_group::block_texture::get(&device, &block_texture_bgl, &block_texture);
        
        let models_bind_group_layout = bind_group_layout::texture::get(&device, wgpu::TextureViewDimension::D2, "2");
        let models = load_models(&device, &queue, &models_bind_group_layout, &[
            ("./models/monkey.obj", "./assets/models/monkey.png", "monkey"),
            ("./models/astronaut.obj", "./assets/models/astronaut.png", "astronaut"),
            ("./models/furnace.obj", "./assets/models/furnace.png", "furnace"),
            ("./models/drill.obj", "./assets/models/drill.png", "drill"),
            ("./models/assembling_machine.obj", "./assets/models/assembling_machine.png", "assembler"),
        ]);
        let animated_models = load_animated_models(&device, &queue, &models_bind_group_layout, &[
            ("./models/manipulator.dae", "./assets/models/manipulator.png", "manipulator"),
            ("./models/cowboy.dae", "./assets/models/cowboy.png", "cowboy"),
        ]);

        let shaders = Shaders::new(&device);

        let sun_bgl = bind_group_layout::vertex_uniform::get(&device, "sun_bgl");
        let sun = BindGroupBuffer::new(&device, bytemuck::cast_slice(&[1.0, 1.0, 1.0]), &sun_bgl, "sun");

        let crosshair_u_ar_scale_bgl = bind_group_layout::vertex_uniform::get(&device, "crosshair_u_scale_bgl");
        let crosshair_u_ar_scale = BindGroupBuffer::new(
            &device,
            bytemuck::cast_slice(&[0.0f32, 0.0]),
            &crosshair_u_ar_scale_bgl,
            "crosshair_u_ar_scale");

        let crosshair_render_pipeline = new_pipeline(
            &device, 
            &[&crosshair_u_ar_scale_bgl],
            &[],
            &shaders.crosshair,
            config.format,
            wgpu::PrimitiveTopology::TriangleList,
            sample_count,
            "crosshair");

        let camera_bgl = bind_group_layout::vertex_uniform::get(&device, "camera_bgl");
        let camera = BindGroupBuffer::new(&device, bytemuck::cast_slice(proj_view), &camera_bgl, "camera");

        let multisampled_framebuffer =
            texture::Texture::create_multisampled_framebuffer(&device, &config, sample_count);
        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture", sample_count);

        let render_pipeline = new_pipeline(
            &device, 
            &[&sun_bgl, &block_texture_bgl, &camera_bgl],
            &[BlockVertex::desc()],
            &shaders.block,
            config.format,
            PRIMITIVE_TOPOLOGY,
            sample_count,
            "block");
        
        let time_bgl = bind_group_layout::vertex_uniform::get(&device, "transport_belt_bgl");
        let time = BindGroupBuffer::new(&device, &0.0f32.to_be_bytes(), &time_bgl, "time");

        let transport_belt_pipeline = new_pipeline(
            &device, 
            &[&sun_bgl, &block_texture_bgl, &camera_bgl, &time_bgl],
            &[BlockVertex::desc()],
            &shaders.transport_belt,
            config.format,
            PRIMITIVE_TOPOLOGY,
            sample_count,
            "transport belt");


        let model_pipeline = new_pipeline(
            &device, 
            &[&sun_bgl, &models_bind_group_layout, &camera_bgl],
            &[ModelVertex::desc(), ModelInstance::desc()],
            &shaders.model,
            config.format,
            PRIMITIVE_TOPOLOGY,
            sample_count,
            "model");
        
        let animated_model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Animated model storage buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let animated_model_layout = bind_group_layout::vertex_storage::get(&device, true, "animated_model_bgl");

        let animated_model_pipeline = new_pipeline(
            &device, 
            &[&sun_bgl, &models_bind_group_layout, &camera_bgl, &animated_model_layout],
            &[AnimatedModelVertex::desc(), AnimatedModelInstance::desc()],
            &shaders.animated_model,
            config.format,
            PRIMITIVE_TOPOLOGY,
            sample_count,
            "animated_model");

        let selection_pipeline = new_pipeline(
            &device, 
            &[&camera_bgl],
            &[SelectionVertex::desc()],
            &shaders.selection,
            config.format,
            wgpu::PrimitiveTopology::LineList,
            sample_count,
            "selection");

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            block_texture,
            block_texutre_bg,
            window,

            camera,

            depth_texture,
            multisampled_framebuffer,
            sample_count,

            egui_platform,
            egui_rpass,

            model_pipeline,
            models,
            animated_models,

            animated_model_pipeline,
            animated_model_buffer,
            animated_model_layout,

            texture_atlas: Arc::new(texture_atlas),

            transport_belt_pipeline,
            time,

            crosshair_render_pipeline,
            crosshair_u_ar_scale,

            sun,

            selection_pipeline,
            selection_vertex_buffer: None,

            is_crosshair: true,
            is_ui_interaction: true
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
            
            self.queue.write_buffer(&self.crosshair_u_ar_scale.buffer, 0, 
                bytemuck::cast_slice(&[new_size.height as f32/new_size.width as f32, 600.0/new_size.height as f32]));
        }
    }


    pub fn update(&mut self, proj_view: &[[f32; 4]; 4], time: &Time) {
        self.queue.write_buffer(&self.camera.buffer, 0, bytemuck::cast_slice(proj_view));
        self.queue.write_buffer(&self.time.buffer, 0, &time.current().to_le_bytes());
    }

    pub fn render<const N: usize>(&mut self, indices: &[usize], sun: &Sun<N>, meshes: &Meshes, mut ui: impl FnMut(&egui::Context)) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        //Egui
        self.egui_platform.begin_frame();

        let ctx = &self.egui_platform.context();
        ui(ctx);
        let window = if self.is_ui_interaction {Some(self.window.as_ref())} else {None};
        let full_output = self.egui_platform.end_frame(window);
        let paint_jobs = self.egui_platform.context().tessellate(full_output.shapes);


        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let (sun, sky) = sun.sun_sky();
        self.queue.write_buffer(&self.sun.buffer, 0, bytemuck::cast_slice(&(<[f32; 3]>::from(sun))));
        let clear_color = wgpu::Color {r: sky.0 as f64, g: sky.1 as f64, b: sky.2 as f64, a: 1.0};
        let rpass_color_attachment = if self.sample_count == 1 {
            wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
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
                    store: wgpu::StoreOp::Discard,
                },
            }
        };
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(rpass_color_attachment)],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });


            // Set sun
            render_pass.set_bind_group(0, &self.sun.bind_group, &[]);

            // Set camera
            render_pass.set_bind_group(2, &self.camera.bind_group, &[]);

            //Render blocks
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(1, &self.block_texutre_bg, &[]);
            indices.iter().for_each(|i| {
                if let Some(Some(mesh)) = &meshes.meshes().get(*i) {
                    render_pass.set_vertex_buffer(0, mesh.block_vertex_buffer.slice(..));
                    render_pass.set_index_buffer(mesh.block_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..mesh.block_index_count, 0, 0..1);
                }
            });

            // meshes.meshes().iter().for_each(|m| {
            //     if let Some(mesh) = &m {
            //         render_pass.set_vertex_buffer(0, mesh.block_vertex_buffer.slice(..));
            //         render_pass.set_index_buffer(mesh.block_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            //         render_pass.draw_indexed(0..mesh.block_index_count, 0, 0..1);
            //     }
            // });

            //Render transport belt
            render_pass.set_pipeline(&self.transport_belt_pipeline);
            render_pass.set_bind_group(3, &self.time.bind_group, &[]);
            indices.iter().for_each(|i| {
                if let Some(Some(mesh)) = meshes.meshes().get(*i) {
                    render_pass.set_vertex_buffer(0, mesh.transport_belt_vertex_buffer.slice(..));
                    render_pass.set_index_buffer(mesh.transport_belt_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..mesh.transport_belt_index_count, 0, 0..1);
                }
            });


            render_pass.set_pipeline(&self.animated_model_pipeline);
            // render_pass.set_bind_group(2, &self.camera_bind_group, &[]);
            // render_pass.set_bind_group(2, &self.animated_model_bind_group, &[]);
            indices.iter().for_each(|i| {
                let Some(Some(mesh)) = meshes.meshes().get(*i) else {return};
                let Some(bind_group) = &mesh.transformation_matrices_bind_group else {return};

                render_pass.set_bind_group(3, bind_group, &[]);
                mesh.animated_models.iter().for_each(|(name, (instance, len))| {
                    let Some(animated_model) = self.animated_models.get(name) else { return; };
                    if !mesh.animated_models.is_empty() {
                        render_pass.set_bind_group(1, &animated_model.texture, &[]);
                        render_pass.set_vertex_buffer(0, animated_model.vertex_buffer.slice(..));
    
                        render_pass.set_vertex_buffer(1, instance.slice(..));
                        render_pass.draw(0..animated_model.vertex_count as u32, 0..*len as u32);
                    }
                });
            });  

            // Render Models
            render_pass.set_pipeline(&self.model_pipeline);
            // render_pass.set_bind_group(2, &self.camera_bind_group, &[]);
            indices.iter().for_each(|i| {
                let Some(Some(mesh)) = &meshes.meshes().get(*i) else {return};

                mesh.models.iter().for_each(|(name, (instance, len))| {
                    let Some(model) = self.models.get(name) else {return};

                    render_pass.set_bind_group(1, &model.texture, &[]);

                    render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
                    render_pass.set_vertex_buffer(1, instance.slice(..));

                    render_pass.draw(0..model.vertex_count as u32, 0..*len as u32);
                });
            });


            if let Some(selection_vertex_buffer) = &self.selection_vertex_buffer {
                render_pass.set_pipeline(&self.selection_pipeline);
                render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
                render_pass.set_vertex_buffer(0, selection_vertex_buffer.slice(..));
                render_pass.draw(0..24, 0..1);
            }


            //Draw crosshair
            if self.is_crosshair {
                render_pass.set_pipeline(&self.crosshair_render_pipeline);
                render_pass.set_bind_group(0, &self.crosshair_u_ar_scale.bind_group, &[]);
                render_pass.draw(0..12, 0..1);
            }
        }
        {
            //EGUI

            let screen_descriptor = ScreenDescriptor {
                physical_width: self.config.width,
                physical_height: self.config.height,
                scale_factor: self.window.scale_factor() as f32,
            };
            let tdelta: egui::TexturesDelta = full_output.textures_delta;
            self.egui_rpass
                .add_textures(&self.device, &self.queue, &tdelta)
                .expect("add texture ok");
            self.egui_rpass.update_buffers(&self.device, &self.queue, &paint_jobs, &screen_descriptor);

            self.egui_rpass
                .execute(
                    &mut encoder,
                    &view,
                    &paint_jobs,
                    &screen_descriptor,
                    None,
                )
                .unwrap();

            self.egui_rpass
                .remove_textures(tdelta)
                .expect("remove texture ok");
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }


    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn set_crosshair(&mut self, value: bool) {
        self.is_crosshair = value;
    }

    pub fn set_ui_interaction(&mut self, value: bool) {
        self.is_ui_interaction = value;
    }
}