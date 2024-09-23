pub mod joint;
pub mod animation;
pub mod bone_key_frames;
pub mod key_frame;
pub mod joint_transform;

use std::collections::HashMap;
use animation::Animation;
use joint::Joint;
use nalgebra_glm as glm;

#[derive(Debug, Clone)]
pub struct Animator {
    length: f32,
    joint_count: usize,
    joint: Joint,
    animations: HashMap<String, Animation>,
    correction: glm::Mat4,
}


impl Animator {
    pub fn new(
        length: f32,
        joint_count: usize,
        joint: Joint,
        animations: HashMap<String, Animation>,
        correction: glm::Mat4,
    ) -> Self {
        Self { length, joint_count, joint, animations, correction }
    }

    pub fn joint_count(&self) -> usize {
        self.joint_count
    }

    pub fn calculate_transforms(&self, animation_name: Option<&str>, progress: f32) -> Vec<glm::Mat4> {
        let mut transforms: Vec<glm::Mat4> = Vec::with_capacity(self.joint_count);
        self.animations
            .get(animation_name.unwrap_or("default"))
            .unwrap()
            .calculate_transforms(&mut transforms, &self.joint, &self.correction, progress*self.length);

        transforms
    }
}