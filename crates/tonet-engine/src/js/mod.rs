//! JavaScript execution and host bindings.
//!
//! **Phase 1:** scripts are not executed; this module documents intent only (`TONET_VISION.md` §2).

/// Whether the engine runs page scripts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptExecutionMode {
    /// Default until an embedded runtime exists.
    Disabled,
}

/// Tonet currently ships as a **no-JS** static renderer.
#[inline]
pub const fn script_execution_mode() -> ScriptExecutionMode {
    ScriptExecutionMode::Disabled
}
