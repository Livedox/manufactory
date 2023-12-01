use egui::{FontDefinitions, vec2};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::window::Window;

pub struct Egui {
    pub rpass: RenderPass,
    platform: Platform,
    screen_descriptor: ScreenDescriptor,
    textures_delta: Option<egui::TexturesDelta>,
    paint_jobs: Option<Vec<egui::ClippedPrimitive>>
}

impl Egui {
    #[inline]
    pub fn new(
      device: &wgpu::Device,
      format: wgpu::TextureFormat,
      width: u32,
      height: u32,
      scale_factor: f64
    ) -> Self {Self {
        platform: Platform::new(PlatformDescriptor {
            physical_width: width,
            physical_height: height,
            scale_factor,
            font_definitions: FontDefinitions::default(),
            style: egui::Style {
                spacing: egui::style::Spacing { item_spacing: vec2(0.0, 0.0), ..Default::default() },
                ..Default::default()
            },
        }),
        rpass: RenderPass::new(&device, format, 1),
        screen_descriptor: ScreenDescriptor {
            physical_width: width, physical_height: height, scale_factor: scale_factor as f32 },
        textures_delta: None,
        paint_jobs: None
    }}

    #[inline]
    pub fn handle_event(&mut self, event: &winit::event::Event<'_, ()>) {
        self.platform.handle_event(event);
    }

    #[inline]
    pub fn resize(&mut self, width: u32, height: u32, scale_factor: f32) {
        self.screen_descriptor = ScreenDescriptor {
            physical_width: width,
            physical_height: height,
            scale_factor,
        };
    }

    #[inline]
    pub fn start(&mut self, window: &Window, is_ui_interaction: bool, mut ui: impl FnMut(&egui::Context)) {
        self.platform.begin_frame();
        let ctx = &self.platform.context();
        ui(ctx);
        let window = if is_ui_interaction {Some(window)} else {None};
        let full_output = self.platform.end_frame(window);
        let paint_jobs = self.platform.context().tessellate(full_output.shapes);
        self.textures_delta = Some(full_output.textures_delta);
        self.paint_jobs = Some(paint_jobs);
    }

    #[inline]
    pub fn end(
      &mut self,
      encoder: &mut wgpu::CommandEncoder,
      device: &wgpu::Device,
      queue: &wgpu::Queue,
      view: &wgpu::TextureView
    ) {
        let tdelta = self.textures_delta.take().expect("Need to start");
        let paint_jobs = self.paint_jobs.take().expect("Need to start");
        self.rpass
            .add_textures(device, queue, &tdelta)
            .expect("add texture ok");
        self.rpass.update_buffers(device, queue, &paint_jobs, &self.screen_descriptor);

        self.rpass
            .execute(
                encoder,
                view,
                &paint_jobs,
                &self.screen_descriptor,
                None,
            )
            .unwrap();

        self.rpass
            .remove_textures(tdelta)
            .expect("remove texture ok");
    }
}