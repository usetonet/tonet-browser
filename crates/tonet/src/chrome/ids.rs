//! Stable [`egui::Id`] builders for the main chrome (toolbar, tab strip).
//!
//! Keeps string literals in one place for focus routing, tests, and future accessibility hooks.
//! See `docs/SERVO_INTEGRATION_CHECKLIST.md` B2.

use egui::Id;

#[inline]
pub fn chrome_back() -> Id {
    Id::new("tonet_chrome_back")
}

#[inline]
pub fn chrome_forward() -> Id {
    Id::new("tonet_chrome_forward")
}

#[inline]
pub fn chrome_stop() -> Id {
    Id::new("tonet_chrome_stop")
}

#[inline]
pub fn chrome_reload() -> Id {
    Id::new("tonet_chrome_reload")
}

#[inline]
pub fn chrome_tool_1() -> Id {
    Id::new("tonet_chrome_tool_1")
}

#[inline]
pub fn chrome_tool_2() -> Id {
    Id::new("tonet_chrome_tool_2")
}

#[inline]
pub fn chrome_tool_3() -> Id {
    Id::new("tonet_chrome_tool_3")
}

#[inline]
pub fn chrome_tool_4() -> Id {
    Id::new("tonet_chrome_tool_4")
}

#[inline]
pub fn chrome_tool_5() -> Id {
    Id::new("tonet_chrome_tool_5")
}

#[inline]
pub fn chrome_tool_6() -> Id {
    Id::new("tonet_chrome_tool_6")
}

#[inline]
pub fn chrome_tool_7() -> Id {
    Id::new("tonet_chrome_tool_7")
}

#[inline]
pub fn chrome_menu() -> Id {
    Id::new("tonet_chrome_menu")
}

/// Tab strip cell for tab at `index` (0-based).
#[inline]
pub fn tab_strip_tab(index: usize) -> Id {
    Id::new("tonet_chrome_tab").with(index)
}

#[inline]
pub fn new_tab_button() -> Id {
    Id::new("tonet_chrome_new_tab")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_match_legacy_string_names() {
        assert_eq!(chrome_back(), Id::new("tonet_chrome_back"));
        assert_eq!(chrome_forward(), Id::new("tonet_chrome_forward"));
        assert_eq!(chrome_stop(), Id::new("tonet_chrome_stop"));
        assert_eq!(chrome_reload(), Id::new("tonet_chrome_reload"));
        assert_eq!(chrome_tool_3(), Id::new("tonet_chrome_tool_3"));
        assert_eq!(chrome_menu(), Id::new("tonet_chrome_menu"));
        assert_eq!(tab_strip_tab(2), Id::new("tonet_chrome_tab").with(2));
        assert_eq!(new_tab_button(), Id::new("tonet_chrome_new_tab"));
    }
}
