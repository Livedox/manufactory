use std::{collections::HashMap, path::Path};

use crate::engine::vertices::animated_model_vertex::AnimatedModelVertex;

use super::animated_model::{BoneKeyFrames, KeyFrame, JointTransform, Animation, Animator, AnimatedModel, Joint};

use itertools::Itertools;
use nalgebra_glm as glm;
use russimp::{node::Node, bone::Bone, scene::{Scene, PostProcess}};

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

pub fn load_animated_model(
  device: &wgpu::Device,
  queue: &wgpu::Queue,
  texture_layout: &wgpu::BindGroupLayout,
  src: impl AsRef<Path>,
  src_texture: impl AsRef<Path>,
  name: &str
) -> AnimatedModel {
    let texture = crate::models::load_texture::load_texture(device, queue, texture_layout, src_texture, name);
    let scene = Scene::from_file(src.as_ref().to_str().unwrap(),
        vec![PostProcess::FlipUVs, PostProcess::MakeLeftHanded]).unwrap();
    let root = scene.root.unwrap();
    let bones = &scene.meshes[0].bones;
    let vertices = &scene.meshes[0].vertices;
    let texture_coords = &scene.meshes[0].texture_coords;
    let armature_name = get_armature_name(&scene.animations[0].name);
    let channels = &scene.animations[0].channels;

    let correction_vec = &glm::vec3(0.5, 0.0, 0.5);
    let correction = glm::translate(&glm::identity(), correction_vec);

    let mut animated_model_vertices: Vec<AnimatedModelVertex> = Vec::with_capacity(vertices.len());
    vertices.iter().for_each(|vertex| {
        animated_model_vertices.push(AnimatedModelVertex {
            position: [vertex.x, vertex.y, vertex.z],
            uv: [0.0, 0.0],
            bone_id: [-1, -1, -1],
            bone_weight: [0.0, 0.0, 0.0]});
    });


    texture_coords.iter().for_each(|texture_coord| {
        let Some(texture_coord) = texture_coord else {return};
        texture_coord
            .iter()
            .zip(animated_model_vertices.iter_mut())
            .for_each(|(coord, vertex)| {
                vertex.uv = [coord.x, coord.y];
            });
    });


    let mut weights: HashMap<usize, Vec<(usize, f32)>> = HashMap::new();
    bones.iter().enumerate().for_each(|(bone_id, bone)| {
        bone.weights.iter().for_each(|vertex_weight| {
            let vertex_id = vertex_weight.vertex_id as usize;
            if let Some(weight) = weights.get_mut(&vertex_id) {
                weight.push((bone_id, vertex_weight.weight));
            } else {
                weights.insert(vertex_id, vec![(bone_id, vertex_weight.weight)]);
            };
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
                animated_model_vertices[*vertex_id].bone_id[index] = *bone_id as i32;
                animated_model_vertices[*vertex_id].bone_weight[index] = weight/sum;
            });
    });
    animated_model_vertices.iter_mut().for_each(|vertex| {
        if vertex.bone_id == [-1, -1, -1] {
            vertex.position[0] += correction_vec.x;
            vertex.position[1] += correction_vec.y;
            vertex.position[2] += correction_vec.z;
        }
    });

    let joint = create_joint(&root, bones, &armature_name)
        .unwrap_or_else(|_| panic!("Model loading error (unknown)")); 
    
    
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

    AnimatedModel::new(device, &animated_model_vertices, texture, animator, name, bones.len())
}


pub fn load_animated_models(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_layout: &wgpu::BindGroupLayout,
    models_textures_names: &[(&str, &str, &str)],
  ) -> HashMap<String, AnimatedModel> {
    let mut models: HashMap<String, AnimatedModel> = HashMap::new();
    models_textures_names.iter().for_each(|mtn| {
        models.insert(
            mtn.2.to_string(), 
            load_animated_model(device, queue, texture_layout, mtn.0, mtn.1, mtn.2)
        );
    });

    models
}