use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GraphicSetting {
    pub sample_count: Option<u8>,
    pub backend: Backend,
    pub device_type: DeviceType,
    pub vsync: bool,
}

impl Default for GraphicSetting {
    fn default() -> Self {
        Self {
            sample_count: None,
            backend: Default::default(),
            device_type: Default::default(),
            vsync: false,
        }
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub enum Backend {
    #[default]
    Auto,
    Vulkan,
    Metal,
    Dx12,
    Dx11
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "Auto"),
            Self::Vulkan => write!(f, "Vulkan"),
            Self::Metal => write!(f, "Metal"),
            Self::Dx12 => write!(f, "Dx12"),
            Self::Dx11 => write!(f, "Dx11"),
        }
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub enum DeviceType {
    #[default]
    Auto,
    DiscreteGpu,
    IntegratedGpu,
    Cpu
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "Auto"),
            Self::DiscreteGpu => write!(f, "DiscreteGpu"),
            Self::IntegratedGpu => write!(f, "IntegratedGpu"),
            Self::Cpu => write!(f, "Cpu"),
        }
    }
}