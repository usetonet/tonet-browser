//! Native window integration (issue #22): borderless root window + in-app caption on Windows.

/// When true, Tonet hides the OS title bar and draws minimize / maximize / close in the tab row.
///
/// Non-Windows builds keep the system frame for predictable behavior across compositors.
///
/// On Windows, set environment variable `TONET_SYSTEM_DECORATIONS=1` (or `true`) to restore the
/// classic framed window.
pub fn integrated_title_chrome() -> bool {
    if !cfg!(target_os = "windows") {
        return false;
    }
    !matches!(
        std::env::var("TONET_SYSTEM_DECORATIONS").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}
