//! Stable [`egui::Id`] values for Servo **embedder** UI on Windows (`runtime_win`):
//! script dialogs, HTTP auth, site permissions, context menu, pickers, notification toast,
//! and the bottom **page console** strip in [`crate::app`] (same cfg gate).
//!
//! See `chrome/ids` for the main browser chrome. Checklist: `docs/SERVO_INTEGRATION_CHECKLIST.md` B2.

use egui::Id;

#[inline]
pub fn simple_dialog() -> Id {
    Id::new("tonet_servo_simple_dialog")
}

#[inline]
pub fn web_notification() -> Id {
    Id::new("tonet_servo_web_notification")
}

#[inline]
pub fn http_auth() -> Id {
    Id::new("tonet_servo_http_auth")
}

#[inline]
pub fn site_permission() -> Id {
    Id::new("tonet_servo_permission")
}

#[inline]
pub fn context_menu() -> Id {
    Id::new("tonet_servo_context_menu")
}

#[inline]
pub fn select_element() -> Id {
    Id::new("tonet_servo_select_element")
}

#[inline]
pub fn color_picker() -> Id {
    Id::new("tonet_servo_color_picker")
}

/// Bottom monospace console strip when the Servo viewport is active (`show_console_message` drain).
#[inline]
pub fn page_console_strip() -> Id {
    Id::new("tonet_servo_page_console")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_match_legacy_string_names() {
        assert_eq!(simple_dialog(), Id::new("tonet_servo_simple_dialog"));
        assert_eq!(web_notification(), Id::new("tonet_servo_web_notification"));
        assert_eq!(http_auth(), Id::new("tonet_servo_http_auth"));
        assert_eq!(site_permission(), Id::new("tonet_servo_permission"));
        assert_eq!(context_menu(), Id::new("tonet_servo_context_menu"));
        assert_eq!(select_element(), Id::new("tonet_servo_select_element"));
        assert_eq!(color_picker(), Id::new("tonet_servo_color_picker"));
        assert_eq!(page_console_strip(), Id::new("tonet_servo_page_console"));
    }
}
