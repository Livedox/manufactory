use std::{iter, collections::HashMap, sync::Arc};
use wgpu::{util::DeviceExt, TextureFormat, TextureFormatFeatureFlags, Adapter};
use winit::window::Window;
use crate::{meshes::Mesh, my_time::Time, models::{load_model::load_models, model::Model, load_animated_model::load_animated_models, animated_model::AnimatedModel}, rev_qumark, engine::{bind_group, shaders::Shaders, bind_group_layout::Layouts, pipeline::Pipelines, egui::Egui}};
use crate::engine::texture::TextureAtlas;
use super::{texture::{self}, bind_group_buffer::BindGroupsBuffers};


pub mod draw;

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
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        }).await);
    // Why is a second search needed?
    // https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#state-new
    instance.enumerate_adapters(wgpu::Backends::all())
        .find(|adapter| {
            adapter.is_surface_supported(surface)
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

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    block_texutre_bg: wgpu::BindGroup,
    window: Arc<Window>,

    pipelines: Pipelines,

    depth_texture: texture::Texture,
    multisampled_framebuffer: wgpu::TextureView,
    sample_count: u32,

    pub egui: Egui,

    models: HashMap<String, Model>,
    pub animated_models: HashMap<String, AnimatedModel>,

    pub animated_model_buffer: wgpu::Buffer,
    pub animated_model_layout: wgpu::BindGroupLayout,

    bind_groups_buffers: BindGroupsBuffers,

    pub texture_atlas: Arc<TextureAtlas>,

    pub selection_vertex_buffer: Option<wgpu::Buffer>,

    is_ui_interaction: bool,
    is_crosshair: bool,
    clear_color: wgpu::Color,
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

        let mut egui = Egui::new(&device, surface_format, size.width, size.height, window.scale_factor());
        
        // TODO: Make texture atlas from files not one file.
        let texture_atlas = TextureAtlas::new(&mut egui.rpass, &device, &queue, "./assets/items/items.png", 4);
       
        let shaders = Shaders::new(&device);
        let layouts = Layouts::new(&device);
        let bind_groups_buffers = BindGroupsBuffers::new(&device, &layouts, proj_view);
        let pipelines = Pipelines::new(&device, &layouts, &shaders, config.format, sample_count);

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

        let block_texutre_bg = bind_group::block_texture::get(&device, &layouts.block_texture, &block_texture);
        
        let models = load_models(&device, &queue, &layouts.model_texture, &[
            ("./models/monkey.obj", "./assets/models/monkey.png", "monkey"),
            ("./models/astronaut.obj", "./assets/models/astronaut.png", "astronaut"),
            ("./models/furnace.obj", "./assets/models/furnace.png", "furnace"),
            ("./models/drill.obj", "./assets/models/drill.png", "drill"),
            ("./models/assembling_machine.obj", "./assets/models/assembling_machine.png", "assembler"),
        ]);
        let animated_models = load_animated_models(&device, &queue, &layouts.model_texture, &[
            ("./models/manipulator.dae", "./assets/models/manipulator.png", "manipulator"),
            ("./models/cowboy.dae", "./assets/models/cowboy.png", "cowboy"),
        ]);

        let multisampled_framebuffer =
            texture::Texture::create_multisampled_framebuffer(&device, &config, sample_count);
        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture", sample_count);
        
        let transforms_storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Animated model storage buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            block_texutre_bg,
            window,

            depth_texture,
            multisampled_framebuffer,
            sample_count,

            egui,

            pipelines,

            models,
            animated_models,

            animated_model_buffer: transforms_storage_buffer,
            animated_model_layout: layouts.transforms_storage,

            texture_atlas: Arc::new(texture_atlas),
            selection_vertex_buffer: None,

            bind_groups_buffers,

            is_crosshair: true,
            is_ui_interaction: true,
            clear_color: wgpu::Color {r: 1.0, g: 1.0, b: 1.0, a: 1.0}
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
            self.depth_texture = texture::Texture::create_depth_texture(
                &self.device, &self.config, "depth_texture", self.sample_count);
            
            self.multisampled_framebuffer =
                texture::Texture::create_multisampled_framebuffer(&self.device, &self.config, self.sample_count);
            
            self.queue.write_buffer(&self.bind_groups_buffers.crosshair_aspect_scale.buffer, 0, 
                bytemuck::cast_slice(&[new_size.height as f32/new_size.width as f32, 600.0/new_size.height as f32]));

            self.egui.resize(new_size.width, new_size.height, self.window.scale_factor() as f32);
        }
    }


    pub fn update(&mut self, proj_view: &[[f32; 4]; 4], time: &Time) {
        self.queue.write_buffer(&self.bind_groups_buffers.camera.buffer, 0, bytemuck::cast_slice(proj_view));
        self.queue.write_buffer(&self.bind_groups_buffers.time.buffer, 0, &time.current().to_le_bytes());
    }

    pub fn render(&mut self, meshes: &[&Mesh], ui: impl FnMut(&egui::Context)) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.egui.start(&self.window, self.is_ui_interaction, ui);

        let mut encoder = self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let rpass_color_attachment = self.get_rpass_color_attachment(&view);
        self.draw_all(&mut encoder, rpass_color_attachment, meshes);

        self.egui.end(&mut encoder, &self.device, &self.queue, &view);

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

    pub fn set_clear_color(&mut self, color: [f64; 3]) {
        self.clear_color = wgpu::Color {r: color[0], g: color[1], b: color[2], a: 1.0};
    }

    pub fn set_sun_color(&mut self, color: [f32; 3]) {
        self.queue.write_buffer(&self.bind_groups_buffers.sun.buffer, 0, bytemuck::cast_slice(&color));
    }

    fn get_rpass_color_attachment<'a>(&'a self, view: &'a wgpu::TextureView) -> wgpu::RenderPassColorAttachment {
        if self.sample_count == 1 {
            wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: wgpu::StoreOp::Store,
                },
            }
        } else {
            wgpu::RenderPassColorAttachment {
                view: &self.multisampled_framebuffer,
                resolve_target: Some(view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    // Storing pre-resolve MSAA data is unnecessary if it isn't used later.
                    // On tile-based GPU, avoid store can reduce your app's memory footprint.
                    store: wgpu::StoreOp::Discard,
                },
            }
        }
    }

    pub fn handle_event(&mut self, event: &winit::event::Event<'_, ()>) {
        self.egui.handle_event(event);
        match event {
            winit::event::Event::WindowEvent {
                ref event, window_id
            } if *window_id == self.window.id() => {
                match event {
                    winit::event::WindowEvent::Resized(physical_size) => {
                        self.resize(*physical_size);
                    }
                    winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        self.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}