//! Platform-specific install steps after a release asset is downloaded to disk.

use std::fs;
#[cfg(windows)]
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(windows)]
use std::thread;
#[cfg(windows)]
use std::time::Duration;

use thiserror::Error;

#[cfg(windows)]
use crate::windows_util::hidden_powershell;
#[cfg(windows)]
use zip::ZipArchive;

// This enum is shared across targets; some variants/helpers only apply on Linux/macOS.
#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum InstallError {
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP: {0}")]
    Zip(String),
    #[error("MSI install failed with status {0:?}")]
    MsiFailed(std::process::ExitStatus),
    #[error("No tonet.exe found inside portable archive")]
    MissingExeInZip,
    #[error("No tonet binary found inside macOS archive")]
    MissingMacBinary,
    #[error("{0}")]
    Msg(String),
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn temp_dir() -> PathBuf {
    std::env::temp_dir().join("tonet-setup")
}

/// Stop a running Tonet session so files in `dest_dir` can be replaced.
#[cfg(windows)]
pub fn stop_running_tonet() {
    let _ = hidden_powershell(
        "Get-Process -Name tonet -ErrorAction SilentlyContinue | Stop-Process -Force",
    );
    thread::sleep(Duration::from_millis(400));
}

#[cfg(windows)]
fn copy_with_retry(from: &Path, to: &Path, attempts: u32) -> Result<(), InstallError> {
    if from == to {
        return Ok(());
    }
    if to.exists() {
        let _ = fs::remove_file(to);
    }
    let mut last_err = None;
    for _ in 0..attempts {
        match fs::copy(from, to) {
            Ok(_) => return Ok(()),
            Err(e) => {
                last_err = Some(e);
                thread::sleep(Duration::from_millis(350));
            }
        }
    }
    Err(last_err.unwrap().into())
}

/// Extract the portable Windows zip (`tonet.exe` + ANGLE DLLs) into `dest_dir`.
#[cfg(windows)]
pub fn install_windows_portable_zip(
    zip_path: &Path,
    dest_dir: &Path,
    mut on_extract_progress: impl FnMut(f32),
) -> Result<PathBuf, InstallError> {
    stop_running_tonet();
    fs::create_dir_all(dest_dir)?;
    let f = fs::File::open(zip_path)?;
    let mut archive = ZipArchive::new(f).map_err(|e| InstallError::Zip(e.to_string()))?;
    let total = archive.len().max(1);
    let mut exe_path: Option<PathBuf> = None;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| InstallError::Zip(e.to_string()))?;
        let raw_name = file.name().replace('\\', "/");
        if file.is_dir() {
            on_extract_progress((i + 1) as f32 / total as f32);
            continue;
        }
        let base = Path::new(&raw_name)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if base.is_empty() {
            on_extract_progress((i + 1) as f32 / total as f32);
            continue;
        }
        let out = dest_dir.join(base);
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)?;
        }
        if out.exists() {
            let _ = fs::remove_file(&out);
        }
        let mut out_file = fs::File::create(&out)?;
        out_file.write_all(&buf)?;
        if base.eq_ignore_ascii_case("tonet.exe") {
            exe_path = Some(out);
        }
        on_extract_progress((i + 1) as f32 / total as f32);
    }

    exe_path.ok_or(InstallError::MissingExeInZip)
}

/// Run MSI silently (used by enterprise/offline flows; web stub uses the portable zip).
#[cfg(windows)]
#[allow(dead_code)]
pub fn install_windows_msi(msi_path: &Path) -> Result<(), InstallError> {
    let status = Command::new("msiexec.exe")
        .args([
            "/i",
            msi_path
                .to_str()
                .ok_or_else(|| InstallError::Msg("invalid MSI path".into()))?,
            "/qn",
            "/norestart",
        ])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(InstallError::MsiFailed(status))
    }
}

/// Write branding icon, uninstaller copy, registry entry, and desktop shortcut.
#[cfg(windows)]
pub fn finalize_windows_install(
    dest_dir: &Path,
    exe_path: &Path,
    version: &str,
) -> Result<(), InstallError> {
    let ico_path = dest_dir.join("app.ico");
    fs::write(&ico_path, crate::branding::APP_ICO)?;

    let uninstall_exe = dest_dir.join("Uninstall Tonet.exe");
    let self_exe = std::env::current_exe()?;
    copy_with_retry(&self_exe, &uninstall_exe, 5)?;

    crate::uninstall::register_uninstall_registry(dest_dir, &uninstall_exe, version, &ico_path)?;
    crate::install_state::write_version_file(dest_dir, version)?;
    create_windows_shortcuts(exe_path, &ico_path)?;
    Ok(())
}

/// Desktop + Start menu shortcuts (Start menu = listed in the Windows Start screen / app list).
#[cfg(windows)]
pub fn create_windows_shortcuts(target_exe: &Path, icon_path: &Path) -> Result<(), InstallError> {
    create_shortcut_at_folder("Desktop", target_exe, icon_path)?;
    create_shortcut_at_folder("Programs", target_exe, icon_path)?;
    Ok(())
}

#[cfg(windows)]
fn create_shortcut_at_folder(
    shell_folder: &str,
    target_exe: &Path,
    icon_path: &Path,
) -> Result<(), InstallError> {
    let target = ps_escape_single(target_exe);
    let icon = ps_escape_single(icon_path);
    let folder = ps_escape_single_str(shell_folder);
    let ps = format!(
        r#"$s = New-Object -ComObject WScript.Shell; $p = [Environment]::GetFolderPath('{folder}'); $l = Join-Path $p 'Tonet.lnk'; $k = $s.CreateShortcut($l); $k.TargetPath = '{target}'; $k.WorkingDirectory = Split-Path -Parent '{target}'; $k.IconLocation = '{icon},0'; $k.Description = 'Tonet'; $k.Save()"#
    );
    let status = hidden_powershell(&ps)?;
    if !status.success() {
        return Err(InstallError::Msg(format!(
            "Could not create shortcut in {shell_folder}."
        )));
    }
    Ok(())
}

#[cfg(windows)]
fn ps_escape_single(path: &Path) -> String {
    path.to_string_lossy().replace('\'', "''")
}

#[cfg(windows)]
fn ps_escape_single_str(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(not(windows))]
pub fn create_windows_shortcuts(_target_exe: &Path, _icon_path: &Path) -> Result<(), InstallError> {
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn find_file_named(root: &Path, name: &str) -> Option<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let read = fs::read_dir(&dir).ok()?;
        for e in read.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.is_file() && p.file_name().and_then(|n| n.to_str()) == Some(name) {
                return Some(p);
            }
        }
    }
    None
}

/// Linux: try `pkexec dpkg -i`. If that fails, extract `.deb` manually into `~/.local`.
#[cfg(target_os = "linux")]
pub fn install_linux_deb(deb_path: &Path) -> Result<PathBuf, InstallError> {
    let status = Command::new("pkexec")
        .args(["dpkg", "-i"])
        .arg(deb_path)
        .status();

    if let Ok(st) = status {
        if st.success() {
            return Ok(PathBuf::from("/usr/bin/tonet"));
        }
    }

    // Fallback: manual unpack to ~/.local/share/tonet and symlink ~/.local/bin/tonet
    let home = dirs::home_dir().ok_or_else(|| InstallError::Msg("no home directory".into()))?;
    let base = home.join(".local/share/tonet");
    fs::create_dir_all(&base)?;
    let stage = temp_dir().join("deb-stage");
    let _ = fs::remove_dir_all(&stage);
    fs::create_dir_all(&stage)?;

    let out = Command::new("dpkg-deb")
        .args(["-x", deb_path.to_str().unwrap(), stage.to_str().unwrap()])
        .status()?;
    if !out.success() {
        return Err(InstallError::Msg(
            "dpkg-deb failed — install dpkg-deb or run with pkexec/sudo".into(),
        ));
    }

    let src = find_file_named(&stage, "tonet")
        .ok_or_else(|| InstallError::Msg("tonet binary not found in .deb".into()))?;
    let dest_bin = base.join("tonet");
    fs::copy(&src, &dest_bin)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dest_bin)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dest_bin, perms)?;
    }

    let local_bin = home.join(".local/bin");
    fs::create_dir_all(&local_bin)?;
    let link = local_bin.join("tonet");
    let _ = fs::remove_file(&link);
    #[cfg(unix)]
    std::os::unix::fs::symlink(&dest_bin, &link)?;

    Ok(link)
}

/// macOS: extract `tonet` from `.tar.gz` into `~/Applications/Tonet` (per-user).
#[cfg(target_os = "macos")]
pub fn install_macos_tgz(tgz_path: &Path) -> Result<PathBuf, InstallError> {
    let out_dir = temp_dir().join("mac-extract");
    let _ = fs::remove_dir_all(&out_dir);
    fs::create_dir_all(&out_dir)?;
    let st = Command::new("tar")
        .args(["-xzf", tgz_path.to_str().unwrap(), "-C", out_dir.to_str().unwrap()])
        .status()?;
    if !st.success() {
        return Err(InstallError::Msg("tar extraction failed".into()));
    }

    let src = find_file_named(&out_dir, "tonet").ok_or(InstallError::MissingMacBinary)?;

    let home = dirs::home_dir().ok_or_else(|| InstallError::Msg("no home directory".into()))?;
    let dest_dir = home.join("Applications/Tonet");
    fs::create_dir_all(&dest_dir)?;
    let dest = dest_dir.join("tonet");
    fs::copy(&src, &dest)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dest)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dest, perms)?;
    }

    Ok(dest)
}
