use cortex_m::delay::Delay;
use embedded_graphics::{
    image::Image,
    mono_font::{ascii::FONT_9X18_BOLD, MonoTextStyleBuilder},
    pixelcolor::{Rgb555, Rgb565, Rgb888},
    prelude::{DrawTarget, Point, RgbColor, WebColors},
    text::{Alignment, Baseline, Text, TextStyleBuilder},
    Drawable,
};
use embedded_hal::{delay::DelayNs, digital::OutputPin};
use embedded_hal_compat::Forward;
use rp2040_hal::gpio::{FunctionSio, Pin, PinId, PullDown, SioOutput};
use tinybmp::Bmp;

use crate::bmp::BmpExt;

pub fn splash_screen<D, C, P>(
    display: &mut D,
    delay: &mut Forward<Delay>,
    display_backlight_pin: &mut Pin<P, FunctionSio<SioOutput>, PullDown>,
    splash_logo: &[u8],
) where
    C: RgbColor + WebColors + From<Rgb555> + From<Rgb565> + From<Rgb888>,
    D: DrawTarget<Color = C>,
    D::Error: core::fmt::Debug,
    P: PinId,
{
    display_backlight_pin.set_high().unwrap();
    display.clear(C::WHITE).unwrap();

    let gopher_panic: Bmp<C> = Bmp::from_slice(splash_logo).unwrap();
    Image::new(&gopher_panic, gopher_panic.screen_center())
        .draw(display)
        .unwrap();

    Text::with_text_style(
        "Rustified Gopherbadge ",
        Point::new(display.bounding_box().center().x, 20),
        MonoTextStyleBuilder::new()
            .font(&FONT_9X18_BOLD)
            .text_color(C::CSS_TOMATO)
            .build(),
        TextStyleBuilder::new()
            .alignment(Alignment::Center)
            .baseline(Baseline::Middle)
            .build(),
    )
    .draw(display)
    .unwrap();

    // delay.delay_ms(2000);
    delay.delay_ms(200);
}
