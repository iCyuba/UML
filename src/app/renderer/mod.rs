use std::error::Error;

pub mod canvas;
#[cfg(not(target_arch = "wasm32"))]
pub mod png;
pub mod window;

pub use canvas::Canvas;
#[cfg(not(target_arch = "wasm32"))]
pub use png::PngRenderer;
pub use window::WindowRenderer;

pub trait Renderer {
    type RenderOutput;

    /// Get a reference to the canvas.
    fn canvas(&mut self) -> &mut Canvas;

    /// Render the canvas.
    fn render(&mut self) -> Result<Self::RenderOutput, Box<dyn Error>>;
}
