use std::{collections::HashMap, path::Path};
use itertools::Itertools;
use nalgebra_glm as glm;
use russimp::{bone::Bone, node::Node, scene::{PostProcess, Scene}};

use crate::texture::{self, load_texture, ModelTexture};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct AnimatedModelVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub bone_id: [i32; 3],
    pub bone_weight: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct AnimatedModel {
    pub vertices: Vec<AnimatedModelVertex>,
    pub texture: ModelTexture,
    pub animator: Animator,
    pub joint_count: usize,
}


impl AnimatedModel {
    pub fn new(
        vertices: Vec<AnimatedModelVertex>,
        texture: ModelTexture,
        animator: Animator,
        joint_count: usize,
    ) -> Self {
        Self { vertices, animator, joint_count, texture }
    }

    pub fn joint_count(&self) -> usize {self.joint_count}


    pub fn calculate_transforms(&self, animation_name: Option<&str>, progress: f32) -> Vec<glm::Mat4> {
        self.animator.calculate_transforms(animation_name, progress)
    }

    pub fn calculate_bytes_transforms(&self, animation_name: Option<&str>, progress: f32) -> Vec<u8> {
        self.calculate_transforms(animation_name, progress)
            .iter().map(|mat| bytemuck::bytes_of(mat)).flatten().cloned().collect()
    }
}


#[derive(Debug, Clone)]
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


#[derive(Debug, Clone)]
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

pub fn load_animated_model(src: impl AsRef<Path>, src_texture: impl AsRef<Path>) -> AnimatedModel {
    let texture = load_texture(src_texture).unwrap();
    let scene = Scene::from_file(src.as_ref().to_str().unwrap(),
        vec![PostProcess::FlipUVs, PostProcess::MakeLeftHanded]).unwrap();
    let root = scene.root.unwrap();
    let mesh = &scene.meshes[0];
    let bones = &mesh.bones;
    let texture_coords = &mesh.texture_coords;
    let armature_name = get_armature_name(&scene.animations[0].name);
    let channels = &scene.animations[0].channels;

    let correction_vec = &glm::vec3(0.5, 0.0, 0.5);
    let correction = glm::translate(&glm::identity(), correction_vec);

    let mut vertices: Vec<AnimatedModelVertex> = Vec::with_capacity(mesh.vertices.len());
    mesh.vertices.iter().for_each(|vertex| {
        vertices.push(AnimatedModelVertex {
            position: [vertex.x, vertex.y, vertex.z],
            uv: [0.0, 0.0],
            bone_id: [-1, -1, -1],
            bone_weight: [0.0, 0.0, 0.0]});
    });


    texture_coords.iter().for_each(|texture_coord| {
        let Some(texture_coord) = texture_coord else {return};
        texture_coord.iter().zip(vertices.iter_mut())
            .for_each(|(coord, vertex)| { vertex.uv = [coord.x, coord.y]; });
    });


    let mut weights: HashMap<usize, Vec<(usize, f32)>> = HashMap::new();
    bones.iter().enumerate().for_each(|(bone_id, bone)| {
        bone.weights.iter().for_each(|vertex_weight| {
            weights.entry(vertex_weight.vertex_id as usize)
                .and_modify(|weight| weight.push((bone_id, vertex_weight.weight)))
                .or_insert(vec![(bone_id, vertex_weight.weight)]);
        });
    });


    weights.iter().for_each(|(vertex_id, vertex_weight)| { 
        let sorted = vertex_weight
            .iter()
            .sorted_by(|a, b| b.1.partial_cmp(&a.1).unwrap())
            .take(3);
        let sum = sorted.clone().map(|(_, weight)| {weight}).sum::<f32>();
        sorted
            .enumerate()
            .for_each(|(index, (bone_id, weight))| {
                vertices[*vertex_id].bone_id[index] = *bone_id as i32;
                vertices[*vertex_id].bone_weight[index] = weight/sum;
            });
    });
    vertices.iter_mut().for_each(|vertex| {
        if vertex.bone_id != [-1, -1, -1] {return};
        vertex.position[0] += correction_vec.x;
        vertex.position[1] += correction_vec.y;
        vertex.position[2] += correction_vec.z;
    });

    let joint = create_joint(&root, bones, &armature_name)
        .unwrap(); 
    
    
    let mut bones_with_keyframes: Vec<BoneKeyFrames> = vec![];
    bones.iter().for_each(|bone| {
        let channel = channels.iter().find(|channel| check_bone_name(&channel.name, &bone.name));
        let Some(channel) = channel else {return};

        let mut key_frames: Vec<KeyFrame> = vec![];
        channel.position_keys.iter().zip(channel.rotation_keys.iter()).for_each(|item| { 
            let joint_transform = JointTransform::new(
                glm::vec3(item.0.value.x, item.0.value.y, item.0.value.z),
                glm::quat(item.1.value.x, item.1.value.y, item.1.value.z, item.1.value.w));
            key_frames.push(KeyFrame::new(item.0.time as f32, joint_transform));
        });
        bones_with_keyframes.push(BoneKeyFrames::new(key_frames));
    });
    let animation = Animation::new(bones_with_keyframes);
    let mut animations = HashMap::new();
    animations.insert("default".to_string(), animation);
    let animator = Animator::new(
        scene.animations[0].duration as f32,
        bones.len(),
        joint,
        animations,
        correction);

    AnimatedModel {
        animator,
        vertices,
        joint_count: bones.len(),
        texture,
    }
}

fn get_armature_name(name: &str) -> String {
    let mut armature_name = name.replace('.', "_");
    if let Some(offset) = armature_name.find('|') {
        armature_name.drain(offset..);
    };
    armature_name
}

fn check_bone_name(name: &str, bone_name: &str) -> bool {
    //This check is needed because the bones are called "BONE_NAME" or Armature_"BONE_NAME"
    //while the nodes or channels are called only Armature_"BONE_NAME"
    if bone_name.len() > name.len() { return false };
    &name[name.len() - bone_name.len()..] == bone_name
}

fn parse_node_children(node: &Node, bones: &Vec<Bone>, index: &mut usize) -> Option<Joint> {  
    if *index >= bones.len() || !check_bone_name(&node.name[..], &bones[*index].name[..]) {
        *index -= 1;
        return None;
    }

    let bm = bones[*index].offset_matrix;
    let inverse_bind_transform = glm::mat4(
        bm.a1, bm.a2, bm.a3, bm.a4,
        bm.b1, bm.b2, bm.b3, bm.b4,
        bm.c1, bm.c2, bm.c3, bm.c4,
        bm.d1, bm.d2, bm.d3, bm.d4);
    let mut joint = Joint::new(*index, bones[*index].name.clone(), inverse_bind_transform);
    node.children.borrow().iter().for_each(|child| {
        *index += 1;
        let child_joint = parse_node_children(child.as_ref(), bones, index);
        if let Some(child_joint) = child_joint {
            joint.add_child(child_joint);
        }
    });
    Some(joint)
}

fn create_joint(root_node: &Node, bones: &Vec<Bone>, armature_name: &str) -> Result<Joint, String> {
    let binding = root_node.children
        .borrow();
    let armature_node = binding
        .iter()
        .find(|child| {child.name == armature_name})
        .ok_or(format!("No armature with the name {:?} were found", armature_name))?;

    let node = &armature_node.children.borrow()[0];
    parse_node_children(node.as_ref(), bones, &mut 0).ok_or("Failed to create joint".to_string())
}