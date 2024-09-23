use nalgebra_glm as glm;

use super::key_frame::KeyFrame;

#[derive(Debug, Clone)]
pub struct BoneKeyFrames {
    key_frames: Vec<KeyFrame>,
}

impl BoneKeyFrames {
    pub fn new(key_frames: Vec<KeyFrame>) -> Self {Self { key_frames }}

    fn prev_and_next_frame(&self, time: f32) -> (&KeyFrame, &KeyFrame) {
        let mut prev = &self.key_frames[0];
        let mut next = &self.key_frames[0];
        for key_frame in self.key_frames.iter().skip(1) {
            next = key_frame;
            if next.time_stamp() >= time {break;}
            prev = key_frame;
        }
        (prev, next)
    }

    
    fn calculate_progression(prev: &KeyFrame, next: &KeyFrame, time: f32) -> f32 {
        let total = next.time_stamp() - prev.time_stamp();
        let current = time - prev.time_stamp();
        current / total
    }


    pub fn interpolate_pose(&self, time: f32) -> glm::Mat4 {
        let (prev, next) = self.prev_and_next_frame(time);
        let progress = Self::calculate_progression(prev, next, time);

        prev.pose()
            .interpolate_joint(next.pose(), progress)
            .local_transform()
    }
}