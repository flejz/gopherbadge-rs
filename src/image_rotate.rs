use core::f64::consts::PI;

use embedded_graphics::pixelcolor::{Rgb555, Rgb888};
use embedded_graphics::{Drawable, Pixel, image::GetPixel, pixelcolor::Rgb565, prelude::*};
use fixed::types::I16F16;
use micromath::F32Ext;
use tinybmp::Bmp;

pub struct ImageRotate<'a, C> {
    bmp: &'a Bmp<'a, C>,
    pos: Point,
    angle_deg: f32,
}

impl<'a, C> ImageRotate<'a, C>
where
    C: RgbColor + From<Rgb555> + From<Rgb565> + From<Rgb888>,
{
    pub fn new(bmp: &'a Bmp<'a, C>, pos: Point, angle_deg: f32) -> Self {
        Self {
            bmp,
            pos,
            angle_deg,
        }
    }
}

impl<'a, C> Drawable for ImageRotate<'a, C>
where
    C: RgbColor + From<Rgb555> + From<Rgb565> + From<Rgb888>,
{
    type Color = C;
    type Output = ();

    fn draw<D: DrawTarget<Color = C>>(&self, display: &mut D) -> Result<(), D::Error> {
        let size = self.bmp.size();
        let w = size.width as i32;
        let h = size.height as i32;

        let cx = w / 2;
        let cy = h / 2;

        // Constants
        let pi: I16F16 = I16F16::from_num(PI);
        let deg_to_rad = pi / I16F16::from_num(180);

        // Calculate sin/cos using fixed
        let theta = I16F16::from_num(self.angle_deg) * deg_to_rad;
        let sin = I16F16::from_num((theta.to_num::<f32>() as f32).sin());
        let cos = I16F16::from_num((theta.to_num::<f32>() as f32).cos());

        for y_out in 0..h {
            for x_out in 0..w {
                let dx = I16F16::from_num(x_out - cx);
                let dy = I16F16::from_num(y_out - cy);

                // Reverse rotate
                let x_src = cos * dx + sin * dy + I16F16::from_num(cx);
                let y_src = -sin * dx + cos * dy + I16F16::from_num(cy);

                let x_src_i = x_src.round().to_num::<i32>();
                let y_src_i = y_src.round().to_num::<i32>();

                if x_src_i >= 0 && x_src_i < w && y_src_i >= 0 && y_src_i < h {
                    if let Some(color) = self.bmp.pixel(Point::new(x_src_i, y_src_i)) {
                        display.draw_iter([Pixel(self.pos + Point::new(x_out, y_out), color)])?;
                    }
                }
            }
        }

        Ok(())
    }
}
