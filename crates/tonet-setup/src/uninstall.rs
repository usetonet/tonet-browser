//! Per-user uninstall for the web stub install (`%LOCALAPPDATA%\Programs\Tonet`).

#[cfg(windows)]
use std::path::{Path, PathBuf};

#[cfg(windows)]
use crate::install::InstallError;
#[cfg(windows)]
use crate::windows_util::{hidden_powershell, reg_add_dword, reg_add_sz};

#[cfg(windows)]
const UNINSTALL_REG_KEY: &str =
    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Tonet";

#[cfg(windows)]
pub fn default_install_dir() -> PathBuf {
    let local = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into());
    PathBuf::from(local).join("Programs").join("Tonet")
}

#[cfg(windows)]
pub fn read_registry_display_version() -> Option<String> {
    use crate::windows_util::hidden_command;

    let out = hidden_command(
        "reg.exe",
        &[
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Tonet",
            "/v",
            "DisplayVersion",
        ],
    )
    .ok()?;
    if !out.status.success() {
        return None;
    }
    parse_reg_query_value(&String::from_utf8_lossy(&out.stdout), "DisplayVersion")
}

#[cfg(windows)]
fn parse_reg_query_value(stdout: &str, key: &str) -> Option<String> {
    for line in stdout.lines() {
        if line.contains(key) {
            return line.split_whitespace().last().map(|s| s.to_string());
        }
    }
    None
}

#[cfg(windows)]
pub fn run_uninstall(install_dir: &Path) -> Result<(), InstallError> {
    remove_shortcuts()?;
    remove_uninstall_registry()?;
    schedule_delete_dir(install_dir)?;
    Ok(())
}

#[cfg(windows)]
fn remove_shortcuts() -> Result<(), InstallError> {
    let ps = r#"
$s = New-Object -ComObject WScript.Shell
$desktop = [Environment]::GetFolderPath('Desktop')
$dl = Join-Path $desktop 'Tonet.lnk'
if (Test-Path -LiteralPath $dl) { Remove-Item -LiteralPath $dl -Force }
$programs = [Environment]::GetFolderPath('Programs')
$sl = Join-Path $programs 'Tonet.lnk'
if (Test-Path -LiteralPath $sl) { Remove-Item -LiteralPath $sl -Force }
"#;
    let status = hidden_powershell(ps)?;
    if !status.success() {
        return Err(InstallError::Msg("Could not remove shortcuts.".into()));
    }
    Ok(())
}

#[cfg(windows)]
pub fn register_uninstall_registry(
    install_dir: &Path,
    uninstall_exe: &Path,
    version: &str,
    icon_ico: &Path,
) -> Result<(), InstallError> {
    let uninstall_string = format!("\"{}\" --uninstall", uninstall_exe.display());
    let checks = [
        reg_add_sz(UNINSTALL_REG_KEY, "DisplayName", "Tonet"),
        reg_add_sz(UNINSTALL_REG_KEY, "Publisher", "usetonet.com"),
        reg_add_sz(UNINSTALL_REG_KEY, "DisplayVersion", version),
        reg_add_sz(
            UNINSTALL_REG_KEY,
            "InstallLocation",
            &install_dir.to_string_lossy(),
        ),
        reg_add_sz(
            UNINSTALL_REG_KEY,
            "DisplayIcon",
            &icon_ico.to_string_lossy(),
        ),
        reg_add_sz(UNINSTALL_REG_KEY, "UninstallString", &uninstall_string),
        reg_add_dword(UNINSTALL_REG_KEY, "NoModify", 1),
        reg_add_dword(UNINSTALL_REG_KEY, "NoRepair", 1),
    ];
    if checks.iter().any(|r| r.as_ref().is_err() || !r.as_ref().unwrap_or(&false)) {
        return Err(InstallError::Msg(
            "Could not register uninstall entry.".into(),
        ));
    }
    Ok(())
}

#[cfg(windows)]
fn remove_uninstall_registry() -> Result<(), InstallError> {
    let ps = r#"
$k = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Tonet'
if (Test-Path -LiteralPath $k) { Remove-Item -LiteralPath $k -Recurse -Force }
"#;
    let _ = hidden_powershell(ps);
    Ok(())
}

#[cfg(windows)]
fn schedule_delete_dir(dir: &Path) -> Result<(), InstallError> {
    let dir_s = dir.to_string_lossy().replace('"', "\"\"");
    let ps = format!(
        r#"Start-Process -FilePath 'cmd.exe' -WindowStyle Hidden -ArgumentList '/c','ping 127.0.0.1 -n 2 > nul & rd /s /q "{dir_s}"' | Out-Null"#
    );
    let status = hidden_powershell(&ps)?;
    if !status.success() {
        return Err(InstallError::Msg("Could not schedule folder removal.".into()));
    }
    Ok(())
}

#[cfg(windows)]
pub fn show_uninstall_error_message(detail: &str) {
    let detail = detail.replace('\'', "''");
    let ps = format!(
        r#"
Add-Type -AssemblyName System.Windows.Forms
[System.Windows.Forms.MessageBox]::Show(
  'No se pudo desinstalar Tonet: {detail}',
  'Tonet',
  [System.Windows.Forms.MessageBoxButtons]::OK,
  [System.Windows.Forms.MessageBoxIcon]::Error
) | Out-Null
"#
    );
    let _ = hidden_powershell(&ps);
}

#[cfg(windows)]
pub fn show_uninstall_finished_message() {
    let ps = r#"
Add-Type -AssemblyName System.Windows.Forms
[System.Windows.Forms.MessageBox]::Show(
  'Tonet se ha desinstalado.',
  'Tonet',
  [System.Windows.Forms.MessageBoxButtons]::OK,
  [System.Windows.Forms.MessageBoxIcon]::Information
) | Out-Null
"#;
    let _ = hidden_powershell(ps);
}
