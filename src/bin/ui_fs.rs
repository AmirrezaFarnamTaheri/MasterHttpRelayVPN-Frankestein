use std::path::{Path, PathBuf};

use mhrv_jni::data_dir;

/// Where we drop downloaded release assets. Prefer the OS user Downloads
/// dir (via the directories crate that's already in our tree), fall back
/// to the user-data dir for platforms that don't expose one (edge case).
pub(crate) fn downloads_dir() -> PathBuf {
    directories::UserDirs::new()
        .and_then(|u| u.download_dir().map(|p| p.to_path_buf()))
        .unwrap_or_else(data_dir::data_dir)
}

/// Open the OS file manager with the given file highlighted/selected.
/// Best-effort: fires the platform-specific command and swallows errors.
pub(crate) fn reveal_in_file_manager(p: &Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg("-R").arg(p).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let arg = format!("/select,\"{}\"", p.display());
        let _ = std::process::Command::new("explorer").arg(arg).spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        // No universal "select this file" primitive on Linux; just open
        // the containing folder.
        if let Some(parent) = p.parent() {
            let _ = std::process::Command::new("xdg-open").arg(parent).spawn();
        }
    }
}

pub(crate) fn open_local_resource(relative_path: &str) {
    if let Some(path) = resolve_local_resource(relative_path) {
        if path.is_dir() {
            open_directory(&path);
        } else {
            reveal_in_file_manager(&path);
        }
    }
}

fn resolve_local_resource(relative_path: &str) -> Option<PathBuf> {
    let rel = PathBuf::from(relative_path);
    if rel.is_absolute() && rel.exists() {
        return Some(rel);
    }

    let mut roots = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.to_path_buf());
            if let Some(parent) = dir.parent() {
                roots.push(parent.to_path_buf());
                if let Some(grandparent) = parent.parent() {
                    roots.push(grandparent.to_path_buf());
                }
            }
        }
    }

    roots
        .into_iter()
        .map(|root| root.join(&rel))
        .find(|candidate| candidate.exists())
}

fn open_directory(p: &Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(p).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(p).spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(p).spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_relative_resource_returns_none() {
        assert!(resolve_local_resource("__mhrv_missing_resource__").is_none());
    }

    #[test]
    fn absolute_existing_resource_resolves() {
        let exe = std::env::current_exe().expect("current exe");
        assert_eq!(resolve_local_resource(&exe.to_string_lossy()), Some(exe));
    }
}
