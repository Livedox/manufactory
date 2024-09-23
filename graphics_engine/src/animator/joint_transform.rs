use nalgebra_glm as glm;

#[derive(Debug, Clone)]
pub struct JointTransform {
    pub position: glm::Vec3,
    pub rotation: glm::Quat,
}


impl JointTransform {
    pub fn new(position: glm::Vec3, rotation: glm::Quat) -> Self {    
        Self {
            position,
            rotation: rotation.normalize()
        }
    }

    pub fn local_transform(&self) -> glm::Mat4 {
        glm::translate(&glm::Mat4::identity(), &self.position) * glm::quat_cast::<f32>(&self.rotation)
    }

    pub(super) fn interpolate_joint(&self, joint: &Self, progression: f32) -> Self {
        let position = glm::mix(&self.position, &joint.position, progression);
        let rotation = glm::quat_slerp(&self.rotation, &joint.rotation, progression);

        Self::new(position, rotation)
    }
}