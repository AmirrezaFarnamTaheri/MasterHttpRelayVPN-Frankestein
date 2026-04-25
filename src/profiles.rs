use crate::config::Config;
use crate::data_dir;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid profile name")]
    InvalidName,
}

fn profiles_dir() -> PathBuf {
    data_dir::data_dir().join("profiles")
}

pub fn sanitize_profile_name(name: &str) -> Result<String, ProfileError> {
    let n = name.trim();
    if n.is_empty() {
        return Err(ProfileError::InvalidName);
    }
    // Conservative: only allow ascii alnum + _-.
    if !n
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ProfileError::InvalidName);
    }
    Ok(n.to_string())
}

pub fn profile_path(name: &str) -> Result<PathBuf, ProfileError> {
    let n = sanitize_profile_name(name)?;
    Ok(profiles_dir().join(format!("{n}.json")))
}

pub fn list_profiles() -> Result<Vec<String>, ProfileError> {
    let dir = profiles_dir();
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    for ent in std::fs::read_dir(dir)? {
        let ent = ent?;
        let p = ent.path();
        if p.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
            out.push(stem.to_string());
        }
    }
    out.sort();
    Ok(out)
}

pub fn save_profile(name: &str, cfg: &Config) -> Result<PathBuf, ProfileError> {
    let path = profile_path(name)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(cfg)?;
    std::fs::write(&path, json)?;
    Ok(path)
}

/// Save a config snapshot under a fixed name (used for last-known-good backup).
pub fn save_snapshot(name: &str, cfg: &Config) -> Result<PathBuf, ProfileError> {
    save_profile(name, cfg)
}

pub fn load_profile(name: &str) -> Result<Config, ProfileError> {
    let path = profile_path(name)?;
    load_profile_from_path(&path)
}

pub fn load_profile_from_path(path: &Path) -> Result<Config, ProfileError> {
    let data = std::fs::read_to_string(path)?;
    let cfg: Config = serde_json::from_str(&data)?;
    Ok(cfg)
}
