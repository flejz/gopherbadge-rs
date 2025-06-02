use embedded_graphics::{
    pixelcolor::{Rgb555, Rgb565, Rgb888},
    prelude::{OriginDimensions, PixelColor, Point},
};
use tinybmp::Bmp;

use crate::{TFT_DISPLAY_HEIGHT, TFT_DISPLAY_WIDTH};

pub trait BmpExt {
    fn screen_center(&self) -> Point;
}

impl<'a, C> BmpExt for Bmp<'a, C>
where
    C: PixelColor + From<Rgb555> + From<Rgb565> + From<Rgb888>,
{
    fn screen_center(&self) -> Point {
        let size = self.size();
        Point::new(
            (TFT_DISPLAY_WIDTH as i32 / 2) - (size.width as i32 / 2),
            (TFT_DISPLAY_HEIGHT as i32 / 2) - (size.height as i32 / 2),
        )
    }
}
