use accelerometer::vector::F32x3;
use core::{fmt::Write, write};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
    pixelcolor::{Rgb555, Rgb565, Rgb888},
    prelude::{DrawTarget, Point, RgbColor},
    text::Text,
    Drawable,
};
use heapless::String;
use smart_leds::RGB8;

pub fn log<D, C>(display: &mut D, text: &str, x: i32, y: i32)
where
    D: DrawTarget<Color = C>,
    D::Error: core::fmt::Debug,
    C: RgbColor + From<Rgb555> + From<Rgb565> + From<Rgb888>,
{
    Text::new(
        text,
        Point::new(x, y),
        MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(C::WHITE)
            .background_color(C::BLACK)
            .build(),
    )
    .draw(display)
    .unwrap();
}

pub fn log_accel<D, C>(display: &mut D, accel: &F32x3)
where
    D: DrawTarget<Color = C>,
    D::Error: core::fmt::Debug,
    C: RgbColor + From<Rgb555> + From<Rgb565> + From<Rgb888>,
{
    let mut buf: String<32> = String::new();
    let _ = write!(
        &mut buf,
        "X: {:.2} Y: {:.2} Z: {:.2}",
        accel.x, accel.y, accel.z
    );
    log(display, &buf, 10, 10);
}

pub fn log_dpad<D, C>(display: &mut D, buttons: (bool, bool, bool, bool))
where
    D: DrawTarget<Color = C>,
    D::Error: core::fmt::Debug,
    C: RgbColor + From<Rgb555> + From<Rgb565> + From<Rgb888>,
{
    let mut buf: String<64> = String::new();
    let _ = write!(
        &mut buf,
        "left: {} right: {} up: {} down: {}",
        if buttons.0 { 1 } else { 0 },
        if buttons.1 { 1 } else { 0 },
        if buttons.2 { 1 } else { 0 },
        if buttons.3 { 1 } else { 0 }
    );
    log(display, &buf, 10, 10);
}

pub fn log_color<D, C>(display: &mut D, eye_color: &C, led_color: &RGB8)
where
    D: DrawTarget<Color = C>,
    D::Error: core::fmt::Debug,
    C: RgbColor + From<Rgb555> + From<Rgb565> + From<Rgb888>,
{
    let mut buf: String<32> = String::new();
    let _ = write!(
        &mut buf,
        "eye_color: r:{} g:{} b:{}",
        eye_color.r(),
        eye_color.g(),
        eye_color.b()
    );
    log(display, &buf, 10, 10);
    let mut buf: String<32> = String::new();
    let _ = write!(
        &mut buf,
        "led_color: r:{} g:{} b:{}",
        led_color.r, led_color.g, led_color.b,
    );
    log(display, &buf, 10, 20);
}
