use embedded_graphics::{
    image::Image,
    mono_font::{ascii, MonoFont, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::{Dimensions, DrawTarget, Point, RgbColor},
    primitives::Rectangle,
    text::{renderer::CharacterStyle, Text},
    Drawable,
};
use heapless::Vec;
use tinybmp::Bmp;

pub trait Michelangelo<D> {
    fn text(
        &mut self,
        display: &mut D,
        text: &str,
        x: i32,
        y: i32,
        font: Option<MonoFont>,
        text_color: Option<Rgb565>,
        bg_color: Option<Rgb565>,
    );

    fn bmp(&mut self, display: &mut D, bytes: &[u8], x: i32, y: i32);

    fn clear(&mut self, display: &mut D);
}

pub struct Draw {
    cleanup: Vec<Rectangle, 32>,
}

impl Draw {
    pub fn new() -> Self {
        Draw {
            cleanup: Vec::new(),
        }
    }
}

impl<D> Michelangelo<D> for Draw
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    fn text(
        &mut self,
        display: &mut D,
        text: &str,
        x: i32,
        y: i32,
        font: Option<MonoFont>,
        text_color: Option<Rgb565>,
        bg_color: Option<Rgb565>,
    ) {
        let font = font.unwrap_or(ascii::FONT_6X13);
        let text_color = text_color.unwrap_or(Rgb565::WHITE);

        let mut style = MonoTextStyle::new(&font, text_color);
        style.set_background_color(bg_color);

        let text = Text::new(text, Point::new(x, y), style);
        text.draw(display).unwrap();

        self.cleanup.push(text.bounding_box()).unwrap();
    }

    fn bmp(&mut self, display: &mut D, bytes: &[u8], x: i32, y: i32) {
        let img: Bmp<Rgb565> = Bmp::from_slice(bytes).unwrap();
        let img = Image::new(&img, Point::new(x, y));
        img.draw(display).unwrap();

        self.cleanup.push(img.bounding_box()).unwrap();
    }

    fn clear(&mut self, display: &mut D) {
        display.clear(Rgb565::BLACK).unwrap();
    }
}
