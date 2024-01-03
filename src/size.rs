static mut VH: f32 = 0.0;
static mut VW: f32 = 0.0;

pub const FONT_SIZE: f32 = 16.0;

pub fn vh() -> f32 {unsafe {VH}}
pub fn vw() -> f32 {unsafe {VW}}

pub fn change_vh(sreen_height: f32) {unsafe {VH = sreen_height / 100.0}}
pub fn change_vw(sreen_width: f32) {unsafe {VW = sreen_width / 100.0}}