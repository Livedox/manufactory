use egui::{style::Spacing, vec2, ClippedPrimitive, Style, Visuals};
use winit::window::Window;

pub struct EguiRenderer<'a> {
    primitives: Vec<ClippedPrimitive>,
    screen_descriptor: egui_wgpu::ScreenDescriptor,
    renderer: &'a mut egui_wgpu::Renderer,
    free: Vec<egui::TextureId>
}


impl<'a> EguiRenderer<'a> {
    pub fn render<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>) {
        self.renderer.render(
            render_pass,
            &self.primitives,
            &self.screen_descriptor
        );
    }
}

impl Drop for EguiRenderer<'_> {
    fn drop(&mut self) {
        for id in &self.free {
            self.renderer.free_texture(id);
        }
    }
}
pub struct Egui {
    renderer: egui_wgpu::Renderer,
    state: egui_winit::State,
}

impl Egui {
    pub fn new(device: &wgpu::Device, window: &Window, format: wgpu::TextureFormat, sample: u32) -> Self {
        let renderer = egui_wgpu::Renderer::new(&device, format, None, sample);
        let egui_ctx = egui::Context::default();
        egui_ctx.set_style(Style {
            visuals: Visuals {
                window_highlight_topmost: false,
                ..Default::default()
            },
            spacing: Spacing {
                item_spacing: vec2(0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        });
        let viewport_id = egui_ctx.viewport_id();
        let state = egui_winit::State::new(egui_ctx, viewport_id, &window, None, None);

        Self { renderer, state }
    }


    pub fn prepare<'a>(
        &'a mut self,
        encoder: &mut wgpu::CommandEncoder,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size_in_pixels: [u32; 2],
        ui: impl FnMut(&egui::Context)
    ) -> EguiRenderer<'a> {
        let input = self.state.take_egui_input(window);
        let output = self.state.egui_ctx().run(input, ui);
        self.state.handle_platform_output(window, output.platform_output);

        let primitives = self.state.egui_ctx().tessellate(output.shapes, output.pixels_per_point);
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            pixels_per_point: output.pixels_per_point,
            size_in_pixels,
        };
        for (id, image_delta) in output.textures_delta.set {
            self.renderer.update_texture(device, queue, id, &image_delta);
        }
        self.renderer.update_buffers(device, queue, encoder, &primitives, &screen_descriptor);

        EguiRenderer {
            free: output.textures_delta.free,
            primitives,
            renderer: &mut self.renderer,
            screen_descriptor: screen_descriptor,
        }
    }

    pub fn renderer(&mut self) -> &mut egui_wgpu::Renderer {
        &mut self.renderer
    }

    pub fn state(&mut self) -> &mut egui_winit::State {
        &mut self.state
    }
}