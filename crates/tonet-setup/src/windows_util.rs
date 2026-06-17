//! Run subprocesses without flashing a console window on Windows.

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use std::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[cfg(windows)]
pub fn hidden_command(program: &str, args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new(program)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
}

#[cfg(windows)]
pub fn hidden_powershell(script: &str) -> std::io::Result<std::process::ExitStatus> {
    Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-Command",
            script,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
}

#[cfg(windows)]
pub fn reg_add_sz(key: &str, name: &str, value: &str) -> std::io::Result<bool> {
    let out = hidden_command(
        "reg.exe",
        &["add", key, "/v", name, "/t", "REG_SZ", "/d", value, "/f"],
    )?;
    Ok(out.status.success())
}

#[cfg(windows)]
pub fn reg_add_dword(key: &str, name: &str, value: u32) -> std::io::Result<bool> {
    let v = value.to_string();
    let out = hidden_command(
        "reg.exe",
        &["add", key, "/v", name, "/t", "REG_DWORD", "/d", &v, "/f"],
    )?;
    Ok(out.status.success())
}
