use nalgebra_glm as glm;

use super::{bone_key_frames::BoneKeyFrames, joint::Joint};

#[derive(Debug, Clone)]
pub struct Animation {
    bones: Vec<BoneKeyFrames>,
}

impl Animation {
    pub fn new(bones: Vec<BoneKeyFrames>) -> Self {Self { bones }}

    pub fn calculate_transforms(&self, transforms: &mut Vec<glm::Mat4>, joint: &Joint, parent_transform: &glm::Mat4, time: f32) {
        let current_local_transform = self.bones[joint.index].interpolate_pose(time);
        let current_transform = parent_transform * current_local_transform;
        transforms.push(current_transform * joint.inverse_bind_transform());

        joint.children.iter().for_each(|child| {
            self.calculate_transforms(transforms, child, &current_transform, time);
        });
    }
}