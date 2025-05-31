use embedded_graphics::{
    image::Image,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use tinybmp::Bmp;

use crate::{TFT_DISPLAY_HEIGHT, TFT_DISPLAY_WIDTH};

pub struct MovableSprite<'a> {
    bmp: Bmp<'a, Rgb565>,
    pos: Point,
    size: Size,
}

impl<'a> MovableSprite<'a> {
    pub fn new(bmp: Bmp<'a, Rgb565>, pos: Point) -> Self {
        let size = bmp.size();
        Self { bmp, pos, size }
    }

    pub fn draw<D>(&self, display: &mut D)
    where
        D: DrawTarget<Color = Rgb565>,
        D::Error: core::fmt::Debug,
    {
        Image::new(&self.bmp, self.pos).draw(display).unwrap();
    }

    pub fn clear_diff<D>(&self, display: &mut D, old_pos: Point, new_pos: &mut Point, bg: Rgb565)
    where
        D: DrawTarget<Color = Rgb565>,
        D::Error: core::fmt::Debug,
    {
        let dx = new_pos.x - old_pos.x;
        let dy = new_pos.y - old_pos.y;

        // horizontal move
        if dx != 0 {
            let x = if dx > 0 {
                old_pos.x
            } else {
                new_pos.x + self.size.width as i32
            };
            let width = dx.abs() as u32;
            let rect = Rectangle::new(
                Point::new(x, new_pos.y),
                Size::new(width.min(self.size.width), self.size.height),
            );
            rect.into_styled(PrimitiveStyle::with_fill(bg))
                .draw(display)
                .unwrap();
        }

        // vertical move
        if dy != 0 {
            let y = if dy > 0 {
                old_pos.y
            } else {
                new_pos.y + self.size.height as i32
            };
            let height = dy.abs() as u32;
            let rect = Rectangle::new(
                Point::new(new_pos.x, y),
                Size::new(self.size.width, height.min(self.size.height)),
            );
            rect.into_styled(PrimitiveStyle::with_fill(bg))
                .draw(display)
                .unwrap();
        }
    }

    pub fn move_to<D>(&mut self, display: &mut D, new_pos: &mut Point, bg: Rgb565)
    where
        D: DrawTarget<Color = Rgb565>,
        D::Error: core::fmt::Debug,
    {
        // keep within screen bounds
        if new_pos.x + self.size.width as i32 >= TFT_DISPLAY_WIDTH as i32 {
            new_pos.x = TFT_DISPLAY_WIDTH as i32 - self.size.width as i32;
        } else if new_pos.x < 0 {
            new_pos.x = 0;
        }
        if new_pos.y + self.size.height as i32 >= TFT_DISPLAY_HEIGHT as i32 {
            new_pos.y = TFT_DISPLAY_HEIGHT as i32 - self.size.height as i32;
        } else if new_pos.y < 0 {
            new_pos.y = 0;
        }

        let old_pos = self.pos;
        self.clear_diff(display, old_pos, new_pos, bg);
        self.pos = new_pos.clone();
        self.draw(display);
    }
}
