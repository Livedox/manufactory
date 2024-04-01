use std::{fs::DirEntry, path::Path};

use serde::{Deserialize, Serialize};

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

pub fn load_info(entry: &DirEntry) -> Option<ContentInfo> {
    let mut path = entry.path();
    path.push("info.toml");
    let file = std::fs::read(path).ok()?;
    let s = &String::from_utf8(file).ok()?;
    toml::from_str(s).ok()
}

pub enum Content {
    Pack(Vec<String>),
    Content(String)
}

pub struct ContentLoader {
    contents: Vec<Content>,
}

impl ContentLoader {
    pub fn new(root: impl AsRef<Path>) -> Self {
        let mut names = vec![];
        let mut infos: Vec<ContentInfo> = vec![];
        for folder in std::fs::read_dir(root).unwrap().flatten() {
            if !folder.file_type().unwrap().is_dir() {continue};
            let file_name = folder.file_name();
            let name = file_name.to_str().unwrap();
            if name.ends_with("_pack") {
                for folder in std::fs::read_dir(folder.path()).unwrap().flatten() {
                    if !folder.file_type().unwrap().is_dir() {continue};
                    let file_name = folder.file_name();
                    let name = file_name.to_str().unwrap();
                    names.push(name.to_string());
                    infos.push(load_info(&folder).unwrap());
                }
            } else {
                names.push(name.to_string());
                infos.push(load_info(&folder).unwrap());
            }
        }
        println!("{:?} {:?}", names, infos);

        Self { contents: vec![] }
    }
}