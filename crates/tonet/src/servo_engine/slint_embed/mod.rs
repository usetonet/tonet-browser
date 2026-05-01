//! Surfman + swap-chain Servo surface adapted from Slint `examples/servo` (MIT / MPL headers in sources).
//! Tonet uses GPU ANGLE/surfman rendering and samples the framebuffer into egui (no D3D11↔wgpu interop yet).

mod gpu_rendering_context;
mod surfman_context;

pub use gpu_rendering_context::GPURenderingContext;
