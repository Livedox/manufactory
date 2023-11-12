use std::{iter, time::Duration, collections::HashMap, rc::Rc, cell::RefCell};

use egui::{FontDefinitions, vec2};
use egui_demo_lib::DemoWindows;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use wgpu::{util::DeviceExt, TextureFormat, TextureFormatFeatureFlags};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use async_std::task::block_on;
use nalgebra_glm as glm;

use crate::{texture::{self, Texture, TextureAtlas}, vertices::{block_vertex::BlockVertex, model_vertex::ModelVertex, model_instance::ModelInstance, animated_model_instance::AnimatedModelInstance, animated_model_vertex::AnimatedModelVertex, selection_vertex::SelectionVertex}, meshes::Meshes, pipelines::{bind_group_layout::{texture::get_texture_bind_group_layout, camera::get_camera_bind_group_layout}, new_pipeline}, gui::gui_controller::{GuiController, self}, player::{inventory::{PlayerInventory}, player::Player}, my_time::Time, recipes::{recipe::{Recipe, RecipeCategory, Recipes}, item::Item, storage::Storage}, model::{load_model::load_models, model::Model, load_animated_model::load_animated_models, animated_model::AnimatedModel}, voxels::{chunks::Chunks, voxel_data::PlayerUnlockableStorage}, world::sun::Sun};

pub const IS_LINE: bool = false;
const PRIMITIVE_TOPOLOGY: wgpu::PrimitiveTopology = match IS_LINE {
    true => wgpu::PrimitiveTopology::LineList,
    false => wgpu::PrimitiveTopology::TriangleList,
};

const MAX_SAMPLE_COUNT: u32 = 4;

pub fn get_supported_multisample_count(surface_format: &TextureFormat, sample_flags: &TextureFormatFeatureFlags) -> Vec<u32> {
    let sample: [u32; 5] = [1, 2, 4, 8, 16];
    let surface_flags = surface_format.guaranteed_format_features(wgpu::Features::empty()).flags;

    sample.into_iter().filter(|a| {
        surface_flags.sample_count_supported(*a)
        && sample_flags.sample_count_supported(*a)
    }).collect()
}

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    model_pipeline: wgpu::RenderPipeline,
    diffuse_texture: texture::Texture,
    diffuse_bind_group: wgpu::BindGroup,
    window: Rc<Window>,
    camera_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,

    depth_texture: Texture,
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
    transport_belt_bind_group: wgpu::BindGroup,
    transport_belt_buffer: wgpu::Buffer,

    crosshair_render_pipeline: wgpu::RenderPipeline,
    crosshair_u_ar_bind_group: wgpu::BindGroup,
    crosshair_u_ar_buffer: wgpu::Buffer,
    crosshair_u_scale_bind_group: wgpu::BindGroup,
    crosshair_u_scale_buffer: wgpu::Buffer,

    sun_bind_group: wgpu::BindGroup,
    sun_buffer: wgpu::Buffer,

    pub texture_atlas: Rc<TextureAtlas>,

    pub selection_vertex_buffer: Option<wgpu::Buffer>,
    selection_pipeline: wgpu::RenderPipeline,
}

impl State {
    pub fn new(window: Rc<Window>, proj_view: &[[f32; 4]; 4]) -> Self {
        let size = window.inner_size();
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        for i in instance.enumerate_adapters(wgpu::Backends::all()) {
            println!("{:?}", i.get_info().backend);
        }
        
        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(window.as_ref()) }.unwrap();

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
            physical_width: size.width as u32,
            physical_height: size.height as u32,
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
       

        let diffuse_texture = texture::Texture::image_array(&device, &queue, &[
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
            "./assets/debug/15.png",], None, sample_count).unwrap();

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

        
        let models_bind_group_layout = get_texture_bind_group_layout(&device, wgpu::TextureViewDimension::D2);
        let models = load_models(&device, &queue, &models_bind_group_layout, &[
            ("./models/monkey.obj", "./assets/models/monkey.png", "monkey"),
            ("./models/astronaut.obj", "./assets/models/astronaut.png", "astronaut"),
            ("./models/furnace.obj", "./assets/models/furnace.png", "furnace"),
            ("./models/drill.obj", "./assets/models/drill.png", "drill"),
            ("./models/assembling_machine.obj", "./assets/models/drill.png", "assembler"),
        ]);
        let animated_models = load_animated_models(&device, &queue, &models_bind_group_layout, &[
            ("./models/manipulator.dae", "./assets/models/manipulator.png", "manipulator"),
            ("./models/cowboy.dae", "./assets/models/cowboy.png", "cowboy"),
        ]);

        let sun_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Sun Buffer"),
                contents: bytemuck::cast_slice(&[1.0, 1.0, 1.0]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let sun_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("sun_bind_group_layout"),
        });
        let sun_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &sun_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: sun_buffer.as_entire_binding(),
                }
            ],
            label: Some("sun_group"),
        });

        let shader_crosshair = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader crosshair"),
            source: wgpu::ShaderSource::Wgsl(include_str!("crosshair.wgsl").into()),
        });

        let crosshair_u_ar_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Crosshair ar Buffer"),
                contents: &0.0f32.to_be_bytes(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let crosshair_u_ar_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("crosshair_u_ar_bind_group_layout"),
        });
        let crosshair_u_ar_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &crosshair_u_ar_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: crosshair_u_ar_buffer.as_entire_binding(),
                }
            ],
            label: Some("crosshair_u_scale_group"),
        });

        let crosshair_u_scale_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Crosshair scale Buffer"),
                contents: &0.0f32.to_be_bytes(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let crosshair_u_scale_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("crosshair_u_scale_bind_group_layout"),
        });
        let crosshair_u_scale_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &crosshair_u_scale_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: crosshair_u_scale_buffer.as_entire_binding(),
                }
            ],
            label: Some("crosshair_u_scale_group"),
        });

        let crosshair_render_pipeline = new_pipeline(
            &device, 
            &[&crosshair_u_ar_bind_group_layout, &crosshair_u_scale_bind_group_layout],
            &[],
            &shader_crosshair,
            config.format,
            wgpu::PrimitiveTopology::TriangleList,
            sample_count,
            "crosshair");

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
            &[&sun_bind_group_layout, &texture_bind_group_layout, &camera_bind_group_layout],
            &[BlockVertex::desc()],
            &shader,
            config.format,
            PRIMITIVE_TOPOLOGY,
            sample_count,
            "block");
        
        let transport_belt_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Transport Belt Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("transport_belt.wgsl").into()),
        });
        let transport_belt_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Transport Belt Buffer"),
                contents: &0.0f32.to_be_bytes(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let transport_belt_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("transport_belt_bind_group_layout"),
        });
        let transport_belt_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &transport_belt_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: transport_belt_buffer.as_entire_binding(),
                }
            ],
            label: Some("transport_belt_group"),
        });
        let transport_belt_pipeline = new_pipeline(
            &device, 
            &[&sun_bind_group_layout, &texture_bind_group_layout, &camera_bind_group_layout, &transport_belt_bind_group_layout],
            &[BlockVertex::desc()],
            &transport_belt_shader,
            config.format,
            PRIMITIVE_TOPOLOGY,
            sample_count,
            "transport belt");


        let model_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader model"),
            source: wgpu::ShaderSource::Wgsl(include_str!("model.wgsl").into()),
        });
        let model_pipeline = new_pipeline(
            &device, 
            &[&sun_bind_group_layout, &models_bind_group_layout, &camera_bind_group_layout],
            &[ModelVertex::desc(), ModelInstance::desc()],
            &model_shader,
            config.format,
            PRIMITIVE_TOPOLOGY,
            sample_count,
            "model");
        
        let transforms = animated_models.get("manipulator").unwrap().calculate_transforms(None, 0.0);
        let mut bytemuck_transforms: Vec<u8> = vec![];
        transforms.iter().for_each(|transform| {
            bytemuck_transforms.extend(bytemuck::cast_slice(transform.as_slice()));
        });
        let animated_model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Animated model storage buffer"),
            contents: &bytemuck_transforms,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let animated_model_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("animated_model_bind_group_layout"),
        });

        let animated_model_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader animated_model"),
            source: wgpu::ShaderSource::Wgsl(include_str!("animated_model.wgsl").into()),
        });
        let animated_model_pipeline = new_pipeline(
            &device, 
            &[&sun_bind_group_layout, &models_bind_group_layout, &camera_bind_group_layout, &animated_model_layout],
            &[AnimatedModelVertex::desc(), AnimatedModelInstance::desc()],
            &animated_model_shader,
            config.format,
            PRIMITIVE_TOPOLOGY,
            sample_count,
            "animated_model");

        let selection_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader animated_model"),
            source: wgpu::ShaderSource::Wgsl(include_str!("selection.wgsl").into()),
        });
        let selection_pipeline = new_pipeline(
            &device, 
            &[&camera_bind_group_layout],
            &[SelectionVertex::desc()],
            &selection_shader,
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
            diffuse_texture,
            diffuse_bind_group,
            window,

            camera_bind_group,
            camera_buffer,

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

            texture_atlas: Rc::new(texture_atlas),

            transport_belt_pipeline,
            transport_belt_bind_group,
            transport_belt_buffer,

            crosshair_render_pipeline,
            crosshair_u_ar_bind_group,
            crosshair_u_ar_buffer,
            crosshair_u_scale_bind_group,
            crosshair_u_scale_buffer,

            sun_bind_group,
            sun_buffer,

            selection_pipeline,
            selection_vertex_buffer: None,
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
            
            self.queue.write_buffer(&self.crosshair_u_ar_buffer, 0, &((new_size.height as f32/new_size.width as f32)).to_le_bytes());
            self.queue.write_buffer(&self.crosshair_u_scale_buffer, 0, &(600.0/new_size.height as f32).to_le_bytes());
        }
    }
    pub fn update(&mut self, proj_view: &[[f32; 4]; 4], time: &Time) {
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(proj_view));
        self.queue.write_buffer(&self.transport_belt_buffer, 0, &time.current().to_le_bytes());
    }

    pub fn render<const N: usize>(&mut self, sun: &Sun<N>, player: &mut Player, gui_controller: &GuiController, meshes: &Meshes, time: &Time, block_id: &mut u32, debug_data: &str) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        //Egui
        self.egui_platform.begin_frame();

        player.inventory().borrow_mut().update_recipe();
        let ctx = &self.egui_platform.context();
        gui_controller
            .draw_inventory(ctx, player, 2)
            // .draw_hotbar(ctx, player, 2)
            .draw_debug(ctx, debug_data, block_id)
            .draw_active_recieps(ctx, player, time);

        let window = if gui_controller.is_cursor() {Some(self.window.as_ref())} else {None};
        let full_output = self.egui_platform.end_frame(window);
        let paint_jobs = self.egui_platform.context().tessellate(full_output.shapes);


        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let (sun, sky) = sun.sun_sky();
        self.queue.write_buffer(&self.sun_buffer, 0, bytemuck::cast_slice(&(<[f32; 3]>::from(sun))));
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
            render_pass.set_bind_group(0, &self.sun_bind_group, &[]);

            // Set camera
            render_pass.set_bind_group(2, &self.camera_bind_group, &[]);

            //Render blocks
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(1, &self.diffuse_bind_group, &[]);
            meshes.meshes().iter().for_each(|mesh| {
                if let Some(mesh) = mesh {
                    render_pass.set_vertex_buffer(0, mesh.block_vertex_buffer.slice(..));
                    render_pass.set_index_buffer(mesh.block_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..mesh.block_index_count, 0, 0..1);
                }
            });
            //Render transport belt
            render_pass.set_pipeline(&self.transport_belt_pipeline);
            render_pass.set_bind_group(3, &self.transport_belt_bind_group, &[]);
            meshes.meshes().iter().for_each(|mesh| {
                if let Some(mesh) = mesh {
                    render_pass.set_vertex_buffer(0, mesh.transport_belt_vertex_buffer.slice(..));
                    render_pass.set_index_buffer(mesh.transport_belt_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..mesh.transport_belt_index_count, 0, 0..1);
                }
            });


            render_pass.set_pipeline(&self.animated_model_pipeline);
            // render_pass.set_bind_group(2, &self.camera_bind_group, &[]);
            // render_pass.set_bind_group(2, &self.animated_model_bind_group, &[]);
            meshes.meshes().iter().for_each(|mesh| {
                let Some(mesh) = mesh else { return; };
                let Some(bind_group) = &mesh.transformation_matrices_bind_group else {return};

                render_pass.set_bind_group(3, bind_group, &[]);
                mesh.animated_models.iter().for_each(|(name, (instance, len))| {
                    let Some(animated_model) = self.animated_models.get(name) else { return; };
                    if mesh.animated_models.len() > 0 {
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
            meshes.meshes().iter().for_each(|mesh| {
                let Some(mesh) = mesh else {return};

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
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_vertex_buffer(0, selection_vertex_buffer.slice(..));
                render_pass.draw(0..24, 0..1);
            }


            //Draw crosshair
            if gui_controller.is_ui() {
                render_pass.set_pipeline(&self.crosshair_render_pipeline);
                render_pass.set_bind_group(0, &self.crosshair_u_ar_bind_group, &[]);
                render_pass.set_bind_group(1, &self.crosshair_u_scale_bind_group, &[]);
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

            // Record all render passes.
            self.egui_rpass
                .execute(
                    &mut encoder,
                    &view,
                    &paint_jobs,
                    &screen_descriptor,
                    None,
                )
                .unwrap();
            // Submit the commands.
            // self.queue.submit(iter::once(encoder.finish()));

            // Redraw egui
            // output.present();

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
}