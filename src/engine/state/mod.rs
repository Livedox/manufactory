use std::{collections::HashMap, iter, path::Path, sync::Arc, time::Instant};


use itertools::Itertools;
use wgpu::{util::DeviceExt, TextureFormat, TextureFormatFeatureFlags, Adapter};
use winit::window::Window;
use crate::{engine::{bind_group, bind_group_layout::{Layouts}, pipeline::Pipelines, shaders::Shaders, texture::Texture}, graphic::complex_object::{load_complex_object, ComplexObject}, meshes::Mesh, models::{animated_model::AnimatedModel, load_animated_model::load_animated_model, load_model::load_model, model::Model}, my_time::Time, rev_qumark};
use crate::engine::texture::TextureAtlas;
use super::{bind_group_buffer::BindGroupsBuffers, egui::Egui, setting::GraphicSetting, texture::{self}};

pub mod draw;

pub struct Indices {
    pub block: HashMap<String, u32>,
    pub models: HashMap<String, u32>,
    pub animated_models: HashMap<String, u32>,
}

pub fn load_complex_objects(
    complex_objects_path: impl AsRef<Path>,
    tmp_indices: &Indices
) -> (HashMap::<String, u32>, Box<[ComplexObject]>) {
    let files = walkdir::WalkDir::new(complex_objects_path)
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| f.file_type().is_file())
        .enumerate();
    let mut indices = HashMap::<String, u32>::new();
    let complex_objects: Box<[ComplexObject]> = files.map(|(index, file)| {
        let file_name = file.file_name().to_str().unwrap();
        let dot_index = file_name.rfind('.').unwrap();
        let name = file_name[..dot_index].to_string();
        let model = load_complex_object(file.path(), tmp_indices);
        indices.insert(name, index as u32);
        model
    }).collect();

    (indices, complex_objects)
}

pub fn load_animated_models(
    models_path: impl AsRef<Path>,
    textures_path: impl AsRef<Path>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_layout: &wgpu::BindGroupLayout
) -> (HashMap::<String, u32>, Box<[AnimatedModel]>) {
    let files = walkdir::WalkDir::new(models_path)
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| f.file_type().is_file())
        .enumerate();
    let textures_path = textures_path.as_ref();

    let mut indices = HashMap::<String, u32>::new();
    let models: Box<[AnimatedModel]> = files.map(|(index, file)| {
        let file_name = file.file_name().to_str().unwrap();
        let dot_index = file_name.rfind('.').unwrap();
        let name = file_name[..dot_index].to_string();
        let texture_name = name.clone() + ".png";
        let src_texture = textures_path.join(texture_name);
        let model = load_animated_model(device, queue, texture_layout, file.path(), src_texture, &name);
        indices.insert(name, index as u32);
        model
    }).collect();

    println!("{:?}", indices);

    (indices, models)
  }

pub fn load_models(
    models_path: impl AsRef<Path>,
    textures_path: impl AsRef<Path>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_layout: &wgpu::BindGroupLayout
) -> (HashMap::<String, u32>, Box<[Model]>) {
    let files = walkdir::WalkDir::new(models_path)
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| f.file_type().is_file())
        .enumerate();
    let textures_path = textures_path.as_ref();

    let mut indices = HashMap::<String, u32>::new();
    let models: Box<[Model]> = files.map(|(index, file)| {
        let file_name = file.file_name().to_str().unwrap();
        let dot_index = file_name.rfind('.').unwrap();
        let name = file_name[..dot_index].to_string();
        let texture_name = name.clone() + ".png";
        let src_texture = textures_path.join(texture_name);
        let model = load_model(device, queue, texture_layout, file.path(), src_texture, &name);
        indices.insert(name, index as u32);
        model
    }).collect();

    (indices, models)
}

pub fn load_textures(device: &wgpu::Device, queue: &wgpu::Queue) -> (HashMap::<String, u32>, Texture) {
    let files = walkdir::WalkDir::new("./res/assets/blocks/")
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| f.file_type().is_file())
        .enumerate();

    let mut indices = HashMap::<String, u32>::new();
    let images = files.map(|(index, file)| {
        let name = file.file_name().to_str().unwrap();
        let dot_index = name.rfind('.').unwrap();
        indices.insert(name[..dot_index].to_string(), index as u32);

        image::open(file.path()).unwrap_or_else(|_| panic!("Failed to open image on path: {:?}", file.path()))
    }).collect_vec();

    (indices, texture::Texture::image_array_n(device, queue, &images))
}

pub trait Priority {
    fn to_priority(&self) -> u8;
}

impl Priority for wgpu::DeviceType {
    fn to_priority(&self) -> u8 {
        match self {
            wgpu::DeviceType::DiscreteGpu => 3,
            wgpu::DeviceType::IntegratedGpu => 2,
            _ => 0
        }
    }
}


fn get_supported_multisample_count(surface_format: &TextureFormat, sample_flags: &TextureFormatFeatureFlags) -> Vec<u32> {
    let sample: [u32; 5] = [1, 2, 4, 8, 16];
    let surface_flags = surface_format.guaranteed_format_features(wgpu::Features::empty()).flags;

    sample.into_iter().filter(|a| {
        surface_flags.sample_count_supported(*a)
        && sample_flags.sample_count_supported(*a)
    }).collect()
}


async fn request_adapter(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface<'_>,
    power: wgpu::PowerPreference,
    setting: &GraphicSetting,
) -> Option<wgpu::Adapter> {
    if setting.backends.is_some() || setting.device_type.is_some() {
        let backends = setting.backends.unwrap_or(wgpu::Backends::all());
        let device = setting.device_type;
        rev_qumark!(instance.enumerate_adapters(backends)
            .into_iter()
            .filter(|a| a.is_surface_supported(surface))
            .max_by(|a1, a2| {
                let d1 = a1.get_info().device_type;
                let d2 = a2.get_info().device_type;
                let n1 = if device.map_or(false, |d| d == d1) {u8::MAX} else {d1.to_priority()};
                let n2 = if device.map_or(false, |d| d == d2) {u8::MAX} else {d2.to_priority()};
                n1.cmp(&n2)
            }));
    }

    rev_qumark!(instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: power,
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        }).await);
    // Why is a second search needed?
    // https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#state-new
    instance.enumerate_adapters(wgpu::Backends::all())
        .into_iter()
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
            required_features: wgpu::Features::empty().union(wgpu::Features::DUAL_SOURCE_BLENDING),
            // WebGL doesn't support all of wgpu's features, so if
            // we're building for the web we'll have to disable some.
            required_limits: if cfg!(target_arch = "wasm32") {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            },
        },
        None,
    ).await
}

pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    pub indices: Indices,
    block_texutre_bg: wgpu::BindGroup,
    window: Arc<Window>,

    pipelines: Pipelines,

    depth_texture: texture::Texture,
    accum_texture: texture::Texture,
    reveal_texture: texture::Texture,
    multisampled_framebuffer: wgpu::TextureView,
    sample_count: u32,

    pub egui: Egui,

    models: Box<[Model]>,
    pub animated_models: Box<[AnimatedModel]>,

    pub animated_model_buffer: wgpu::Buffer,

    bind_groups_buffers: BindGroupsBuffers,
    pub layouts: Layouts,

    pub texture_atlas: Arc<TextureAtlas>,

    pub selection_vertex_buffer: Option<wgpu::Buffer>,

    is_ui_interaction: bool,
    is_crosshair: bool,
    clear_color: wgpu::Color,
}

impl<'a> State<'a> {
    pub async fn new(window: Arc<Window>, proj_view: &[[f32; 4]; 4], setting: &GraphicSetting) -> Self {
        let size = window.inner_size();
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        for i in instance.enumerate_adapters(wgpu::Backends::all()) {
            println!("{:?} {:?} {:?}", i.get_info().device_type, i.get_info().name, i.get_info().backend);
        }
        
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        let power_preference = wgpu::PowerPreference::HighPerformance;
        let adapter = request_adapter(&instance, &surface, power_preference, setting)
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
        let surface_present_mode = 
            if setting.vsync {wgpu::PresentMode::AutoVsync} else {wgpu::PresentMode::AutoNoVsync};
        
        println!("format: {:?}, present_mode: {:?}, alpha_mode: {:?}", surface_caps.formats, surface_caps.present_modes, surface_caps.alpha_modes);
        
        let surface_alpha_mode = surface_caps.alpha_modes[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_present_mode,
            alpha_mode: surface_alpha_mode,
            view_formats: vec![surface_format],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let sample_flags = adapter
            .get_texture_format_features(config.view_formats[0])
            .flags;
        let samples_count = get_supported_multisample_count(&surface_format, &sample_flags);
        let sample_count = if samples_count.contains(&setting.sample_count) {
            setting.sample_count
        } else {
            samples_count.into_iter().max().unwrap()
        };
        println!("Sample count X{}", sample_count);

        // let mut egui = Egui::new(&device, surface_format, size.width, size.height, window.scale_factor());
        let mut egui = Egui::new(&device, &window, surface_format, sample_count);
        // TODO: Make texture atlas from files not one file.
        let texture_atlas = TextureAtlas::new(egui.renderer(), &device, &queue, "./res/assets/items/items.png", 4);
       
        let shaders = Shaders::new(&device);
        let layouts = Layouts::new(&device);
        let bind_groups_buffers = BindGroupsBuffers::new(&device, &layouts, proj_view);
        let pipelines = Pipelines::new(&device, &layouts, &shaders, config.format, sample_count);

        let start = Instant::now();
        let (block_texture_id, block_texture) = load_textures(&device, &queue);
        println!("Load time: {:?}", start.elapsed().as_secs_f32());

        let block_texutre_bg = bind_group::block_texture::get(&device, &layouts.block_texture, &block_texture);

        let (models_indices, models) = load_models("./res/models", "./res/assets/models", &device, &queue, &layouts.model_texture);
        let (animated_models_indices, animated_models) = load_animated_models(
            "./res/animated_models", "./res/assets/models", &device, &queue, &layouts.model_texture);
        let indices = Indices {
            block: block_texture_id,
            models: models_indices,
            animated_models: animated_models_indices,
        };


        let multisampled_framebuffer =
            texture::Texture::create_multisampled_framebuffer(&device, &config, sample_count);

        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture", sample_count);
        let accum_texture = texture::Texture::create_accum_texture(&device, &config, "accum_texture", sample_count);
        let reveal_texture = texture::Texture::create_reveal_texture(&device, &config, "reveal_texture");
        
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

            indices,

            block_texutre_bg,
            window,

            depth_texture,
            accum_texture,
            reveal_texture,
            multisampled_framebuffer,
            sample_count,

            egui,

            pipelines,

            models,
            animated_models,

            animated_model_buffer: transforms_storage_buffer,

            texture_atlas: Arc::new(texture_atlas),
            selection_vertex_buffer: None,

            bind_groups_buffers,
            layouts,

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
            self.accum_texture = texture::Texture::create_accum_texture(
                &self.device, &self.config, "accum_texture", self.sample_count);
            self.reveal_texture = texture::Texture::create_reveal_texture(
                &self.device, &self.config, "reveal_texture");
            
            self.multisampled_framebuffer =
                texture::Texture::create_multisampled_framebuffer(&self.device, &self.config, self.sample_count);
            
            self.queue.write_buffer(&self.bind_groups_buffers.crosshair_aspect_scale.buffer, 0, 
                bytemuck::cast_slice(&[new_size.height as f32/new_size.width as f32, 600.0/new_size.height as f32]));

            self.window.request_redraw();
            // self.egui.resize(new_size.width, new_size.height, self.window.scale_factor() as f32);
        }
    }

    pub fn update_time(&mut self, time: &Time) {
        self.queue.write_buffer(&self.bind_groups_buffers.time.buffer, 0, &time.current().to_le_bytes());
    }

    pub fn update_camera(&mut self, proj_view: &[[f32; 4]; 4]) {
        self.queue.write_buffer(&self.bind_groups_buffers.camera.buffer, 0, bytemuck::cast_slice(proj_view));
    }

    pub fn render(&mut self, meshes: &[Arc<Mesh>], ui: impl FnMut(&egui::Context)) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let output_texture = &output.texture;
        let view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        self.draw_all(&mut encoder, output_texture, &view, meshes);
        
        let egui_renderer = self.egui.prepare(&mut encoder, &self.window, &self.device,
            &self.queue, [self.config.width, self.config.height], ui);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass Egui"),
            color_attachments: &[Some(
                if self.sample_count == 1 {
                    wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    }
                } else {
                    wgpu::RenderPassColorAttachment {
                        view: &self.multisampled_framebuffer,
                        resolve_target: Some(&view),
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            // Storing pre-resolve MSAA data is unnecessary if it isn't used later.
                            // On tile-based GPU, avoid store can reduce your app's memory footprint.
                            store: wgpu::StoreOp::Discard,
                        },
                    }
                }
            )],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        egui_renderer.render(&mut render_pass);
        drop(render_pass);
        // self.egui.end(&mut encoder, &self.device, &self.queue, &view);
        
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

    fn get_rpass_color_attachment<'b>(&'b self, view: &'b wgpu::TextureView, store: wgpu::StoreOp) -> wgpu::RenderPassColorAttachment {
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
                    store,
                },
            }
        }
    }

    pub fn handle_event(&mut self, event: &winit::event::Event<()>) {
        match event {
            winit::event::Event::WindowEvent {
                ref event, window_id
            } if *window_id == self.window.id() => {
                let _ = self.egui.state().on_window_event(self.window.as_ref(), event);
                if let winit::event::WindowEvent::Resized(physical_size) = event {
                    self.resize(*physical_size);
                }
            }
            _ => {}
        }
    }
}