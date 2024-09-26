use std::{collections::HashMap, error::Error, path::{Path, PathBuf}};

use graphics_engine::{animator::{animation::Animation, bone_key_frames::BoneKeyFrames, joint::Joint, joint_transform::JointTransform, key_frame::KeyFrame, Animator}, constants::{BLOCK_MIPMAP_COUNT, BLOCK_TEXTURE_SIZE}, models::{model::Model, raw_animated_model::RawAnimatedModel, raw_model::RawModel}, raw_texture::RawTexture, vertices::{animated_model_vertex::AnimatedModelVertex, model_vertex::ModelVertex}};
use image::{imageops::FilterType, DynamicImage, ImageError};
use itertools::Itertools;
use russimp::{bone::Bone, node::Node, scene::{PostProcess, Scene}};
use nalgebra_glm as glm;

#[derive(Debug, Clone)]
pub struct GamePath<T: AsRef<Path>> {
    pub path: T,
    pub prefix: Option<String>,
}

pub fn load_texture(src: impl AsRef<Path>) -> Result<RawTexture, ImageError> {
    let img = image::open(src)?;

    Ok(RawTexture {
        width: img.width(),
        height: img.height(),
        data: img.into_bytes()
    })
}

#[derive(Debug, Clone)]
pub struct Indices {
    pub block: HashMap<String, u32>,
    pub models: HashMap<String, u32>,
    pub animated_models: HashMap<String, u32>,
}

pub fn load_blocks_textures(paths: &[GamePath<PathBuf>]) -> (HashMap::<String, u32>, Vec<Vec<u8>>, u32) {
    let (names, images): (Vec<String>, Vec<DynamicImage>) = paths.iter().flat_map(|path| {
            let prefix = path.prefix.as_ref().map(|s| s.as_str()).unwrap_or("");
            let files = walkdir::WalkDir::new(&path.path)
                .into_iter()
                .filter_map(|f| f.ok())
                .filter(|f| f.file_type().is_file());

            files.map(move |file| {
                let name = file.file_name().to_str().unwrap();
                let dot_index = name.rfind('.').unwrap();
                (format!("{}{}", &prefix, &name[..dot_index]), image::open(file.path())
                    .unwrap_or_else(|_| panic!("Failed to open image on path: {:?}", file.path())))
            })
        }).unzip();

    let data = (0..BLOCK_MIPMAP_COUNT).map(|mipmap| {
        let size = BLOCK_TEXTURE_SIZE / 2u32.pow(mipmap as u32);
        images.iter().flat_map(|image| {
            if mipmap == 0 {return image.to_rgba8().to_vec()};
            image.resize(size, size, FilterType::Triangle).to_rgba8().to_vec()
        }).collect_vec()
    }).collect_vec();

    let indices: HashMap<String, u32> = names.into_iter().enumerate()
        .map(|(i, n)| (n, i as u32)).collect();

    (indices, data, images.len() as u32)
}

pub fn load_animated_models(
    model_paths: &[impl AsRef<Path>],
    texture_paths: &[impl AsRef<Path>],
) -> (HashMap::<String, u32>, Vec<RawAnimatedModel>) {
    let load = |p: &Path, m: RawTexture| {load_animated_model(p, m)};
    load_with_texture(model_paths, texture_paths, &load)
}

pub fn load_models(
    model_paths: &[impl AsRef<Path>],
    texture_paths: &[impl AsRef<Path>], 
) -> (HashMap::<String, u32>, Vec<RawModel>) {
    let load = |p: &Path, m: RawTexture| {load_model(p, m)};
    let (indices, models) = load_with_texture(model_paths, texture_paths, &load);
    (indices, models.into_iter().map(|m| m.unwrap()).collect())
}

pub fn load_model(
    src_model: impl AsRef<Path>,
    texture: RawTexture,
) -> Result<RawModel, Box<dyn Error>> {
    let scene = Scene::from_file(src_model.as_ref().to_str().unwrap(),
        vec![PostProcess::FlipUVs, PostProcess::MakeLeftHanded])?;

    let mut model_vertices: Vec<ModelVertex> = vec![];
    let mesh = &scene.meshes[0];

    mesh.vertices.iter().for_each(|vertex| {
        model_vertices.push(ModelVertex {
            position: [vertex.x + 0.5, vertex.y, vertex.z + 0.5],
            uv: [0.0, 0.0]});
    });

    mesh.texture_coords.iter().for_each(|coords| {
        if let Some(coords) = coords.as_ref() {
            coords.iter().enumerate().for_each(|(index, coords)| {
                model_vertices[index].uv = [coords.x, coords.y]})}
    });

    Ok(RawModel {
        vertices: model_vertices,
        texture,
    })
}

pub fn load_with_texture<T>(
    paths: &[impl AsRef<Path>],
    texture_paths: &[impl AsRef<Path>],
    load: &dyn for<'a> Fn(&'a Path, RawTexture) -> T
) -> (HashMap::<String, u32>, Vec<T>) {
    let textures_files = texture_paths.iter().flat_map(|path| walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| f.file_type().is_file()));

    let mut textures: HashMap<String, RawTexture> = textures_files.map(|file| {
        let file_name = file.file_name().to_str().unwrap();
        let dot_index = file_name.rfind('.').unwrap();
        let name = file_name[..dot_index].to_string();
        load_texture(file.path()).ok().map(|t| (name, t))
    }).flatten().collect();

    let files = paths.iter().flat_map(|path| walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| f.file_type().is_file())
        .enumerate());

    let mut indices = HashMap::<String, u32>::new();
    let datas: Vec<T> = files.map(|(index, file)| {
        let file_name = file.file_name().to_str().unwrap();
        let dot_index = file_name.rfind('.').unwrap();
        let name = file_name[..dot_index].to_string();
        let data = load(file.path(), textures.remove(&name).unwrap());
        indices.insert(name, index as u32);
        data
    }).collect();

    (indices, datas)
}


pub fn load_animated_model(src: impl AsRef<Path>, texture: RawTexture) -> RawAnimatedModel {
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

    RawAnimatedModel {
        animator,
        vertices,
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