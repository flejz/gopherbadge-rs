use embedded_graphics::{
    image::Image,
    mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, Primitive, RgbColor, Size},
    primitives::{Circle, PrimitiveStyle, Rectangle, Styled},
    text::Text,
    Drawable,
};
use embedded_hal::digital::{InputPin, OutputPin};
use rp2040_hal::gpio::{FunctionSio, Pin, PinId, PullDown, SioInput, SioOutput};
use tinybmp::Bmp;

use crate::{TFT_DISPLAY_HEIGHT, TFT_DISPLAY_WIDTH};

const RUST_LOGO: &'static [u8] = include_bytes!("./assets/rust-pride.bmp");

pub fn draw<D>(
    display: &mut D,
    offset_x: &i32,
    offset_y: &i32,
) -> impl Fn() -> Styled<Rectangle, PrimitiveStyle<Rgb565>>
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    display.clear(Rgb565::BLACK).unwrap();

    let rust_logo: Bmp<Rgb565> = Bmp::from_slice(RUST_LOGO).unwrap();

    let size = rust_logo.as_raw().header().image_size;

    let x = (TFT_DISPLAY_WIDTH as u32 / 2) - (size.width / 2);
    let y = (TFT_DISPLAY_HEIGHT as u32 / 2) - (size.height / 2);

    let pos_x = offset_x + x as i32;
    let pos_y = offset_y + y as i32;

    let rust_logo = Image::new(&rust_logo, Point::new(pos_x, pos_y));

    rust_logo.draw(display).unwrap();

    move || {
        let clear_area = Rectangle::new(Point::new(pos_x, pos_y), size);
        clear_area.into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
    }
}

pub fn draw_sample<D>(display: &mut D)
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    display.clear(Rgb565::BLACK).unwrap();

    Circle::new(Point::new(0, 0), 41)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
        .draw(display)
        .unwrap();

    Rectangle::new(Point::new(20, 20), Size::new(80, 60))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
        .draw(display)
        .unwrap();

    let no_background = MonoTextStyleBuilder::new()
        .font(&FONT_6X9)
        .text_color(Rgb565::WHITE)
        .build();

    let filled_background = MonoTextStyleBuilder::new()
        .font(&FONT_6X9)
        .text_color(Rgb565::YELLOW)
        .background_color(Rgb565::BLUE)
        .build();

    let inverse_background = MonoTextStyleBuilder::new()
        .font(&FONT_6X9)
        .text_color(Rgb565::BLUE)
        .background_color(Rgb565::YELLOW)
        .build();

    Text::new(
        "Hello world! - no background",
        Point::new(15, 15),
        no_background,
    )
    .draw(display)
    .unwrap();

    Text::new(
        "Hello world! - filled background",
        Point::new(15, 30),
        filled_background,
    )
    .draw(display)
    .unwrap();

    Text::new(
        "Hello world! - inverse background",
        Point::new(15, 45),
        inverse_background,
    )
    .draw(display)
    .unwrap();
}

pub fn led_on_button_press(
    mut button_pin: Pin<impl PinId, FunctionSio<SioInput>, PullDown>,
    mut led_pin: Pin<impl PinId, FunctionSio<SioOutput>, PullDown>,
) -> impl FnMut() {
    return move || {
        if button_pin.is_high().unwrap() {
            led_pin.set_high().unwrap();
        } else {
            led_pin.set_low().unwrap();
        }
    };
}
