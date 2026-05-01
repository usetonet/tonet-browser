//! Persisted Servo embedder permission decisions. Keys are `origin + '\\t' + feature_token`
//! (same strings as the in-memory map in `runtime_win`).

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("tonet").join("servo_permissions.json"))
}

pub(crate) fn load() -> HashMap<String, bool> {
    let Some(path) = file_path() else {
        return HashMap::new();
    };
    let Ok(data) = fs::read(&path) else {
        return HashMap::new();
    };
    serde_json::from_slice::<HashMap<String, bool>>(&data).unwrap_or_default()
}

pub(crate) fn save(map: &HashMap<String, bool>) {
    let Some(path) = file_path() else {
        return;
    };
    let Some(parent) = path.parent() else {
        return;
    };
    let _ = fs::create_dir_all(parent);
    let Ok(bytes) = serde_json::to_vec_pretty(map) else {
        return;
    };
    let _ = fs::write(path, bytes);
}

/// Remove persisted decisions (e.g. when the user clears browsing data / history).
pub(crate) fn remove_file() {
    if let Some(path) = file_path() {
        let _ = fs::remove_file(path);
    }
}
