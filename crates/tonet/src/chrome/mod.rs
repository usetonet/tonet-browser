//! Browser chrome components: tab strip, toolbar, and window caption controls.

mod caption;
pub mod ids;
mod tab_strip;
mod toolbar;

pub use tab_strip::show_tab_bar;
pub use toolbar::show_chrome_toolbar;
