pub mod mock;
pub mod whisplay_panel;

#[cfg(all(target_os = "linux", feature = "whisplay-hardware"))]
pub mod whisplay;

use anyhow::Result;

use crate::render::Framebuffer;

pub trait DisplayDevice {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn flush_full_frame(&mut self, framebuffer: &Framebuffer) -> Result<()>;
    fn set_backlight(&mut self, brightness: f32) -> Result<()>;
}

pub trait ButtonDevice {
    fn pressed(&mut self) -> Result<bool>;
}
