use std::collections::HashMap;

use nalgebra_glm as glm;
use wgpu::util::DeviceExt;

use crate::engine::vertices::animated_model_vertex::AnimatedModelVertex;

#[derive(Debug)]
pub struct AnimatedModel {
    pub vertex_count: usize,
    pub vertex_buffer: wgpu::Buffer,
    pub texture: wgpu::BindGroup,

    animator: Animator,
    joint_count: usize,
}


impl AnimatedModel {
    pub fn new(
      device: &wgpu::Device,
      model: &[AnimatedModelVertex],
      texture: wgpu::BindGroup,
      animator: Animator,
      name: &str,
      joint_count: usize
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Model vertex buffer ({})", name)),
            contents: bytemuck::cast_slice(model),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {vertex_count: model.len(), vertex_buffer, texture, animator, joint_count}
    }


    pub fn joint_count(&self) -> usize {self.joint_count}


    pub fn calculate_transforms(&self, animation_name: Option<&str>, progress: f32) -> Vec<glm::Mat4> {
        self.animator.calculate_transforms(animation_name, progress)
    }

    pub fn calculate_bytes_transforms(&self, animation_name: Option<&str>, progress: f32) -> Vec<u8> {
        let mut result = vec![];
        self.animator
            .calculate_transforms(animation_name, progress)
            .iter()
            .for_each(|transform| {
                result.extend(bytemuck::cast_slice(transform.as_slice()));
            });

        result
    }
}


#[derive(Debug)]
pub struct Joint {
    pub index: usize,
    pub name: String,
    pub children: Vec<Joint>,

    inverse_bind_transform: glm::Mat4,
}

impl Joint {
    pub fn new(index: usize, name: String, inverse_bind_transform: glm::Mat4) -> Self {
        Self {
            index,
            name,
            children: vec![],

            inverse_bind_transform,
        }
    }


    pub fn add_child(&mut self, joint: Joint) {
        self.children.push(joint);
    }

    pub fn inverse_bind_transform(&self) -> &glm::Mat4 {
        &self.inverse_bind_transform
    }
}


#[derive(Debug)]
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

    fn calculate_transforms(&self, animation_name: Option<&str>, progress: f32) -> Vec<glm::Mat4> {
        let mut transforms: Vec<glm::Mat4> = Vec::with_capacity(self.joint_count);
        self.animations
            .get(animation_name.unwrap_or("default"))
            .unwrap()
            .calculate_transforms(&mut transforms, &self.joint, &self.correction, progress*self.length);

        transforms
    }
}


#[derive(Debug)]
pub struct Animation {
    bones: Vec<BoneKeyFrames>,
}


impl Animation {
    pub fn new(bones: Vec<BoneKeyFrames>) -> Self {Self { bones }}

    fn calculate_transforms(&self, transforms: &mut Vec<glm::Mat4>, joint: &Joint, parent_transform: &glm::Mat4, time: f32) {
        let current_local_transform = self.bones[joint.index].interpolate_pose(time);
        let current_transform = parent_transform * current_local_transform;
        transforms.push(current_transform * joint.inverse_bind_transform());

        joint.children.iter().for_each(|child| {
            self.calculate_transforms(transforms, child, &current_transform, time);
        });
    }
}


#[derive(Debug)]
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

#[derive(Debug)]
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



#[derive(Debug)]
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