use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GraphicSetting {
    pub sample_count: u32,
    pub backends: Option<wgpu::Backends>,
    pub device_type: Option<wgpu::DeviceType>,
    pub vsync: bool,
}

impl Default for GraphicSetting {
    fn default() -> Self {
        Self {
            sample_count: 16,
            backends: None,
            device_type: None,
            vsync: true,
        }
    }
}