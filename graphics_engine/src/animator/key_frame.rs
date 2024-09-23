use super::joint_transform::JointTransform;

#[derive(Debug, Clone)]
pub struct KeyFrame {
    time_stamp: f32,
    pose: JointTransform,
}

impl KeyFrame {
    pub fn new(time_stamp: f32, pose: JointTransform) -> Self {
        Self { time_stamp, pose }
    }

    pub fn time_stamp(&self) -> f32 {self.time_stamp}
    pub fn pose(&self) -> &JointTransform {&self.pose}
}