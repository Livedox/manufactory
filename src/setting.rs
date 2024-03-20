use serde::{Deserialize, Serialize};
use graphics_engine::setting::GraphicSetting;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Setting {
    pub is_greedy_meshing: bool,
    pub render_radius: u32,
    pub graphic: GraphicSetting,
}


impl Default for Setting {
    fn default() -> Self {
        Self {
            is_greedy_meshing: true,
            render_radius: 3,
            graphic: Default::default(),
        }
    }
}