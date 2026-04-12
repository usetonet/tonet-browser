//! Windows-only: subtle rounded window corners via DWM (issue #22 follow-up).

use raw_window_handle::{HasWindowHandle, RawWindowHandle};

const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
/// DWMWCP_ROUNDSMALL — slight rounding, not a full pill.
const DWMWCP_ROUNDSMALL: u32 = 3;

#[link(name = "dwmapi")]
unsafe extern "system" {
    fn DwmSetWindowAttribute(
        hwnd: isize,
        dwattribute: u32,
        pvattribute: *const std::ffi::c_void,
        cbattribute: u32,
    ) -> i32;
}

/// Returns `true` if the attribute was applied (best effort; older Windows may ignore it).
pub fn try_apply_round_corners(frame: &eframe::Frame) -> bool {
    let Ok(handle) = frame.window_handle() else {
        return false;
    };
    let raw = handle.as_raw();
    let hwnd = match raw {
        RawWindowHandle::Win32(w) => w.hwnd.get(),
        _ => return false,
    };
    let pref = DWMWCP_ROUNDSMALL;
    // SAFETY: HWND is valid for the lifetime of the frame; DWM accepts a DWORD attribute.
    let st = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            std::ptr::addr_of!(pref).cast::<std::ffi::c_void>(),
            std::mem::size_of_val(&pref) as u32,
        )
    };
    st == 0
}
