use std::{collections::{HashMap, HashSet}, fs::DirEntry, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};

use crate::content_loader::indices::{load_blocks_textures, GamePath};

use self::indices::Indices;

pub mod indices;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContentInfo {
    name: String,
    version: String,

    #[serde(default)]
    authors: Option<Vec<String>>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    category: Option<String>,
}

impl ContentInfo {
    pub fn name(&self) -> &str { &self.name }
    pub fn version(&self) -> &str { &self.name }
}

#[derive(Debug, Clone)]
pub struct ContentDetails {
    active: bool,
    path: PathBuf,
    info: ContentInfo,
}

impl ContentDetails {
    pub fn new(active: bool, path: PathBuf, info: ContentInfo) -> Self {
        Self {
            active,
            info,
            path
        }
    }

    pub fn active(&self) -> bool {self.active}
    pub fn path(&self) -> &Path {&self.path}
}

pub fn load_info(entry: &DirEntry) -> Option<ContentInfo> {
    let mut path = entry.path();
    path.push("info.toml");
    let file = std::fs::read(path).ok()?;
    let s = &String::from_utf8(file).ok()?;
    toml::from_str(s).ok()
}

pub struct ContentLoader {
    details: HashMap::<String, ContentDetails>,
}

impl ContentLoader {
    pub fn new(content: impl AsRef<Path>) -> Self {
        let active_content_packs: HashSet<String> = std::fs::read("./data/content_packs.json")
            .ok().and_then(|data| serde_json::from_slice(&data).ok())
            .unwrap_or_default();
        let mut details = HashMap::<String, ContentDetails>::new();
        for folder in std::fs::read_dir(content).unwrap().flatten() {
            if !folder.file_type().unwrap().is_dir() {continue};
            let Some(info) = load_info(&folder) else {continue};
            if let Some(old_detail) = details.get(info.name()) {
                let old_info = &old_detail.info;
                eprintln!("This content pack \"{}:{}\" in the directory \"{}\" will be ignored!",
                    info.name(), info.version(), folder.path().to_str().unwrap_or("unknown"));
                if old_info.version() == info.version() {
                    eprintln!("Reason: duplicate");
                } else {
                    eprintln!("Reason: content pack found with a different version \"{}\"", 
                        old_info.version());
                }
            } else {
                let detail = ContentDetails::new(active_content_packs.contains(info.name()),
                    folder.path(), info);
                details.insert(detail.info.name().to_string(), detail);
            }
        }


        Self { details }
    }

    pub fn load_indices(&self, game: impl AsRef<Path>) -> Indices {
        println!("{:?}", game.as_ref());
        let block_paths: Vec<GamePath<PathBuf>> = self.details().values().filter(|d| d.active)
            .map(|d| {
                GamePath {
                    path: d.path.join("assets/blocks/"),
                    prefix: Some(format!("@{}:", d.info.name)),
                }
            })
            .chain(std::iter::once(GamePath {
                path: game.as_ref().join("assets/blocks/"),
                prefix: None,
            }))
            .collect();

        let (indices, blocks, count) = load_blocks_textures(&block_paths);
        println!("{:?} {:?}", indices, block_paths);
        todo!();
    }

    pub fn details(&self) -> &HashMap<String, ContentDetails> {
        &self.details
    }
}