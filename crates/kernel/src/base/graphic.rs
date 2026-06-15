/// Color representation and pixel format implementations
pub mod color;
mod framebuffer;
/// Coordinate system and position management
pub mod position;

pub use framebuffer::{DisplayDraw, FRAME_BUFFER, FrameBuffer};
