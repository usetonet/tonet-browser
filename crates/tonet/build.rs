//! When `servo-engine` is enabled on Windows, surfman loads `libEGL.dll` from the directory of
//! the main executable. Mozangle builds those DLLs under `target/*/build/mozangle-*/out/`; copy
//! them next to `tonet.exe` so `cargo run` works without manual PATH setup.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let servo_engine = env::var_os("CARGO_FEATURE_SERVO_ENGINE").is_some();
    if target_os != "windows" || !servo_engine {
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let Some(target_dir) = out_dir.ancestors().nth(3).map(Path::to_path_buf) else {
        println!("cargo:warning=tonet: could not resolve target dir from OUT_DIR");
        return;
    };

    let build_dir = target_dir.join("build");
    let Some(moz_out) = find_mozangle_out(&build_dir) else {
        println!(
            "cargo:warning=tonet: mozangle build output not found under {}; EGL run may panic until you rebuild deps",
            build_dir.display()
        );
        return;
    };

    for dll in ["libEGL.dll", "libGLESv2.dll"] {
        let src = moz_out.join(dll);
        if !src.is_file() {
            println!("cargo:warning=tonet: expected {}", src.display());
            continue;
        }
        println!("cargo:rerun-if-changed={}", src.display());
        let dst = target_dir.join(dll);
        let _ = fs::copy(&src, &dst);
    }
}

fn find_mozangle_out(build_dir: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(build_dir).ok()?;
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("mozangle-") {
            continue;
        }
        let egl = entry.path().join("out").join("libEGL.dll");
        if !egl.is_file() {
            continue;
        }
        let Ok(mtime) = fs::metadata(&egl).and_then(|m| m.modified()) else {
            continue;
        };
        let Some(out_dir) = egl.parent().map(Path::to_path_buf) else {
            continue;
        };
        match &best {
            None => best = Some((mtime, out_dir)),
            Some((t, _)) if mtime > *t => best = Some((mtime, out_dir)),
            _ => {}
        }
    }
    best.map(|(_, p)| p)
}
