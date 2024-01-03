use serde::{Deserialize, Serialize};
use crate::engine::setting::GraphicSetting;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Setting {
    pub render_radius: u32,
    pub graphic: GraphicSetting,
}


impl Setting {
    pub fn new() -> Self{Self::default()}
}


impl Default for Setting {
    fn default() -> Self {
        Self {
            render_radius: 3,
            graphic: Default::default(),
        }
    }
}