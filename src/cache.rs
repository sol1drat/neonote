use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Cache {
    vault: PathBuf,
}

pub fn cache_exists() -> bool {
    cache_dir()
        .map(|dir| dir.join("index.json").is_file())
        .unwrap_or(false)
}

pub fn get_vault() -> Option<PathBuf> {
    let dir = cache_dir()?;

    let json_path = dir.join("index.json");
    let json = fs::read_to_string(&json_path).ok()?;

    let cache: Cache = serde_json::from_str(&json).ok()?;

    Some(cache.vault)
}

pub fn cache_dir() -> Option<PathBuf> {
    let proj = ProjectDirs::from("", "", "neonote")?;
    Some(proj.cache_dir().to_path_buf())
}

pub fn cache_vault(vault: PathBuf) {
    let dir = cache_dir().expect("no usable cache dir");
    let _ = fs::create_dir_all(&dir);

    let cache = Cache { vault };

    let json_path = dir.join("index.json");
    let json = serde_json::to_string_pretty(&cache).unwrap();
    let _ = fs::write(&json_path, json);
}
