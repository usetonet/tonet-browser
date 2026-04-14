//! Platform-specific install steps after a release asset is downloaded to disk.

use std::fs;
#[cfg(windows)]
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use thiserror::Error;

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

/// Extract `tonet.exe` from our portable zip into `dest_dir` (created if missing).
#[cfg(windows)]
pub fn install_windows_portable_zip(zip_path: &Path, dest_dir: &Path) -> Result<PathBuf, InstallError> {
    fs::create_dir_all(dest_dir)?;
    let f = fs::File::open(zip_path)?;
    let mut archive = ZipArchive::new(f).map_err(|e| InstallError::Zip(e.to_string()))?;
    let mut found: Option<Vec<u8>> = None;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| InstallError::Zip(e.to_string()))?;
        let name = file.name().replace('\\', "/");
        let base = Path::new(&name)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if base.eq_ignore_ascii_case("tonet.exe") {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            found = Some(buf);
            break;
        }
    }
    let bytes = found.ok_or(InstallError::MissingExeInZip)?;
    let exe = dest_dir.join("tonet.exe");
    let mut out = fs::File::create(&exe)?;
    out.write_all(&bytes)?;
    drop(out);
    Ok(exe)
}

/// Run MSI silently. Requires sufficient privileges for a per-machine package.
#[cfg(windows)]
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

/// Create a `.lnk` on the current user's desktop pointing to `target_exe`.
#[cfg(windows)]
pub fn create_desktop_shortcut(target_exe: &Path) -> Result<(), InstallError> {
    let target = target_exe.to_string_lossy().replace('\'', "''");
    let ps = format!(
        r#"$s = New-Object -ComObject WScript.Shell; $p = [Environment]::GetFolderPath('Desktop'); $l = Join-Path $p 'Tonet.lnk'; $k = $s.CreateShortcut($l); $k.TargetPath = '{target}'; $k.WorkingDirectory = Split-Path -Parent '{target}'; $k.Save()"#
    );
    let status = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &ps])
        .status()?;
    if !status.success() {
        return Err(InstallError::Msg(
            "Could not create desktop shortcut (PowerShell)".into(),
        ));
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn create_desktop_shortcut(_target_exe: &Path) -> Result<(), InstallError> {
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
