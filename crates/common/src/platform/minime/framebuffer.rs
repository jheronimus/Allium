use anyhow::{Context, Result, anyhow};
use framebuffer::Framebuffer;
use tiny_skia::{Pixmap, PixmapMut, PixmapRef};

use crate::display::Display;
use crate::display::color::Color;
use crate::geom::Rect;
use crate::platform::minime::traits::Traits;

pub struct MinimeDisplay {
    pixmap: Pixmap,
    framebuffer: Framebuffer,
    rotation: u32,
    saved: Vec<Pixmap>,
}

impl MinimeDisplay {
    pub fn new(traits: &Traits) -> Result<Self> {
        let framebuffer = Framebuffer::new(&traits.gpu_device)?;
        if framebuffer.var_screen_info.bits_per_pixel != 32 {
            return Err(anyhow!(
                "unsupported framebuffer depth: {}",
                framebuffer.var_screen_info.bits_per_pixel
            ));
        }
        let pixmap = Pixmap::new(traits.screen_width, traits.screen_height)
            .ok_or_else(|| anyhow!("failed to create display pixmap"))?;
        Ok(Self {
            pixmap,
            framebuffer,
            rotation: traits.screen_rotation,
            saved: Vec::new(),
        })
    }
}

impl Display for MinimeDisplay {
    fn width(&self) -> u32 {
        self.pixmap.width()
    }

    fn height(&self) -> u32 {
        self.pixmap.height()
    }

    fn pixmap(&self) -> PixmapRef<'_> {
        self.pixmap.as_ref()
    }

    fn pixmap_mut(&mut self) -> PixmapMut<'_> {
        self.pixmap.as_mut()
    }

    fn map_pixels<F>(&mut self, mut f: F) -> Result<()>
    where
        F: FnMut(Color) -> Color,
    {
        for pixel in self.pixmap.pixels_mut() {
            *pixel = f((*pixel).into()).into();
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        let physical_width = self.framebuffer.var_screen_info.xres as usize;
        let physical_height = self.framebuffer.var_screen_info.yres as usize;
        for y in 0..self.height() as usize {
            for x in 0..self.width() as usize {
                let (frame_x, frame_y) =
                    rotate(x, y, physical_width, physical_height, self.rotation);
                let frame_index = (frame_y * physical_width + frame_x) * 4;
                let pixel = self.pixmap.pixels()[y * self.width() as usize + x];
                self.framebuffer.frame[frame_index..frame_index + 4].copy_from_slice(&[
                    pixel.blue(),
                    pixel.green(),
                    pixel.red(),
                    pixel.alpha(),
                ]);
            }
        }
        Ok(())
    }

    fn save(&mut self) -> Result<()> {
        self.saved.push(self.pixmap.clone());
        Ok(())
    }

    fn load(&mut self, rect: Rect) -> Result<()> {
        let saved = self.saved.last().context("no saved image")?;
        for y in rect.y.max(0) as usize..(rect.y.max(0) as u32 + rect.h).min(self.height()) as usize
        {
            for x in
                rect.x.max(0) as usize..(rect.x.max(0) as u32 + rect.w).min(self.width()) as usize
            {
                let index = y * self.width() as usize + x;
                self.pixmap.pixels_mut()[index] = saved.pixels()[index];
            }
        }
        Ok(())
    }

    fn pop(&mut self) -> bool {
        self.saved.pop();
        !self.saved.is_empty()
    }
}

fn rotate(x: usize, y: usize, width: usize, height: usize, rotation: u32) -> (usize, usize) {
    match rotation {
        90 => (width - y - 1, x),
        180 => (width - x - 1, height - y - 1),
        270 => (y, height - x - 1),
        _ => (x, y),
    }
}
