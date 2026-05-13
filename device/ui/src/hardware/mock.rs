use anyhow::Result;

use crate::hardware::{ButtonDevice, DisplayDevice};
use crate::renderer::Framebuffer;
use crate::router::DirtyRegion;

#[derive(Debug)]
pub struct MockDisplay {
    width: usize,
    height: usize,
    pub frames: usize,
    pub regional_frames: usize,
}

impl MockDisplay {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            frames: 0,
            regional_frames: 0,
        }
    }
}

impl DisplayDevice for MockDisplay {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn flush_full_frame(&mut self, _framebuffer: &Framebuffer) -> Result<()> {
        self.frames += 1;
        Ok(())
    }

    fn flush_region(&mut self, _framebuffer: &Framebuffer, _region: DirtyRegion) -> Result<()> {
        self.regional_frames += 1;
        Ok(())
    }

    fn set_backlight(&mut self, _brightness: f32) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct MockButton {
    pressed: bool,
}

impl MockButton {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ButtonDevice for MockButton {
    fn pressed(&mut self) -> Result<bool> {
        Ok(self.pressed)
    }
}
