use egui::{Context, vec2, RichText};
use crate::{setting::Setting, save_load::SettingSave};


pub fn draw_setting(ctx: &Context, open: &mut bool, setting: &mut Setting, save: &SettingSave) {
    let setting_save = unsafe { &*(setting as *mut Setting) };
    let device_type: &mut Option<wgpu::DeviceType> = &mut setting.graphic.device_type;
    let backends: &mut Option<wgpu::Backends> = &mut setting.graphic.backends;
    let sample_count: &mut u32 = &mut setting.graphic.sample_count;
    egui::Window::new("Setting")
        .open(open)
        .movable(true)
        .resizable(false)
        .collapsible(false)
        .default_pos([0.0, 0.0])
        .show(ctx, |ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Heading);
            ui.spacing_mut().item_spacing = vec2(4.0, 2.0);
            ui.horizontal(|ui| {
                ui.label("Render radius:");
                ui.spacing_mut().slider_width = 180.0;
                ui.add(
                    egui::Slider::new(&mut setting.render_radius, 3..=100)
                        .show_value(false)
                );
                ui.label(&format!(" {}", setting.render_radius));
            });
            ui.horizontal(|ui| {
                ui.label("Greedy meshing:");
                ui.checkbox(&mut setting.is_greedy_meshing, "");
            });

            ui.horizontal(|ui| {
                ui.label("Fullscreen:");
                ui.checkbox(&mut true, "");
            });

            ui.add_space(10.0);
            ui.heading(RichText::new("Graphics Settings (Restart required)").size(20.0));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Vsync:");
                ui.checkbox(&mut setting.graphic.vsync, "");
            });
            ui.horizontal(|ui| {
                ui.label("Backend:");
                ui.selectable_value(backends, None, "Auto");
                ui.selectable_value(backends, Some(wgpu::Backends::VULKAN), "Vulkan");
                ui.selectable_value(backends, Some(wgpu::Backends::DX12), "Dx12");
                ui.selectable_value(backends, Some(wgpu::Backends::METAL), "Metal");
            });
            ui.horizontal(|ui| {
                ui.label("Device:");
                ui.selectable_value(device_type, None, "Auto");
                ui.selectable_value(device_type, Some(wgpu::DeviceType::DiscreteGpu), "DiscreteGpu");
                ui.selectable_value(device_type, Some(wgpu::DeviceType::IntegratedGpu), "IntegratedGpu");
            });
            ui.horizontal(|ui| {
                ui.label("Sample count:");
                ui.selectable_value(sample_count, 1, "X1");
                ui.selectable_value(sample_count, 2, "X2");
                ui.selectable_value(sample_count, 4, "X4");
                ui.selectable_value(sample_count, 8, "X8");
                ui.selectable_value(sample_count, 16, "X16");
            });

            if ui.button("Save setting").clicked() {
                save.save(setting_save);
            };
        });
}