use embedded_graphics::{
    image::{GetPixel, Image},
    pixelcolor::{Rgb555, Rgb565, Rgb888},
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use tinybmp::Bmp;

use crate::{TFT_DISPLAY_HEIGHT, TFT_DISPLAY_WIDTH};

pub struct MovableSpriteBuilder<'a, C> {
    bmp: Bmp<'a, C>,
    pos: Option<Point>,
    transparent_color: Option<C>,
    screen_boundaries: bool,
}

impl<'a, C> MovableSpriteBuilder<'a, C>
where
    C: RgbColor + From<Rgb555> + From<Rgb565> + From<Rgb888>,
{
    pub fn builder(bmp: Bmp<'a, C>) -> Self {
        Self {
            bmp,
            pos: None,
            transparent_color: None,
            screen_boundaries: false,
        }
    }

    pub fn build(self) -> MovableSprite<'a, C> {
        MovableSprite::new(
            self.bmp,
            self.pos.unwrap_or(Point::new(0, 0)),
            self.screen_boundaries,
            self.transparent_color,
        )
    }

    pub fn with_position(mut self, pos: Point) -> Self {
        self.pos = Some(pos);
        self
    }

    #[allow(dead_code)]
    pub fn with_transparency(mut self, transparent_color: C) -> Self {
        self.transparent_color = Some(transparent_color);
        self
    }

    pub fn with_screen_boundaries(mut self) -> Self {
        self.screen_boundaries = true;
        self
    }
}

pub struct MovableSprite<'a, C> {
    bmp: Bmp<'a, C>,
    pos: Point,
    size: Size,
    screen_boundaries: bool,
    transparent_color: Option<C>,
}

impl<'a, C> MovableSprite<'a, C>
where
    C: RgbColor + From<Rgb555> + From<Rgb565> + From<Rgb888>,
{
    pub fn new(
        bmp: Bmp<'a, C>,
        pos: Point,
        screen_boundaries: bool,
        transparent_color: Option<C>,
    ) -> Self {
        let size = bmp.size();
        Self {
            bmp,
            pos,
            size,
            screen_boundaries,
            transparent_color,
        }
    }

    pub fn draw<D>(&self, display: &mut D)
    where
        D: DrawTarget<Color = C>,
        D::Error: core::fmt::Debug,
    {
        Image::new(&self.bmp, self.pos).draw(display).unwrap();
    }

    pub fn draw_with_transparency<D>(&self, display: &mut D)
    where
        D: DrawTarget<Color = C>,
        D::Error: core::fmt::Debug,
    {
        for y in 0..self.bmp.size().height {
            for x in 0..self.bmp.size().width {
                let pixel = self.bmp.pixel(Point::new(x as i32, y as i32)).unwrap();

                if pixel != self.transparent_color.unwrap() {
                    display
                        .draw_iter(core::iter::once(Pixel(
                            Point::new(x as i32, y as i32) + self.pos,
                            pixel,
                        )))
                        .unwrap();
                }
            }
        }
    }

    pub fn clear_diff<D>(&self, display: &mut D, old_pos: Point, new_pos: &mut Point, bg: C)
    where
        D: DrawTarget<Color = C>,
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
            let width = dx.unsigned_abs();
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
            let height = dy.unsigned_abs();
            let rect = Rectangle::new(
                Point::new(new_pos.x, y),
                Size::new(self.size.width, height.min(self.size.height)),
            );
            rect.into_styled(PrimitiveStyle::with_fill(bg))
                .draw(display)
                .unwrap();
        }
    }

    pub fn move_to<D>(&mut self, display: &mut D, new_pos: &mut Point, bg: C)
    where
        D: DrawTarget<Color = C>,
        D::Error: core::fmt::Debug,
    {
        if self.screen_boundaries {
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
        }

        let old_pos = self.pos;
        self.clear_diff(display, old_pos, new_pos, bg);
        self.pos = *new_pos;
        if self.transparent_color.is_none() {
            self.draw(display);
        } else {
            self.draw_with_transparency(display);
        }
    }
}
