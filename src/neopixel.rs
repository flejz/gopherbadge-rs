use cortex_m::delay::Delay;
use embedded_graphics::{
    pixelcolor::{Rgb555, Rgb565, Rgb888},
    prelude::{DrawTarget, Point, Primitive, RgbColor, WebColors},
    primitives::{Circle, PrimitiveStyle},
    Drawable,
};
use embedded_hal::{delay::DelayNs, digital::InputPin};
use embedded_hal_compat::Forward;
use rp2040_hal::{
    gpio::{
        bank0::{Gpio11, Gpio15},
        FunctionPio0, FunctionSio, Pin, PullDown, SioInput,
    },
    pac::PIO0,
    pio::SM0,
    timer::CountDown,
};
use smart_leds::{
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, RGB8,
};
use ws2812_pio::Ws2812;

use crate::log::log_color;

fn rgb8_to_rgb565(rgb: &RGB8) -> Rgb565 {
    let max_rgb = rgb.r.max(rgb.g).max(rgb.b).max(1); // avoid divide-by-zero

    let r_scaled = (rgb.r as u16 * 255 / max_rgb as u16).min(255);
    let g_scaled = (rgb.g as u16 * 255 / max_rgb as u16).min(255);
    let b_scaled = (rgb.b as u16 * 255 / max_rgb as u16).min(255);

    let r5 = (r_scaled * 31 + 127) / 255;
    let g6 = (g_scaled * 63 + 127) / 255;
    let b5 = (b_scaled * 31 + 127) / 255;

    Rgb565::new(r5 as u8, g6 as u8, b5 as u8)
}

#[allow(clippy::too_many_arguments)]
pub fn neopixel<D, C>(
    display: &mut D,
    delay: &mut Forward<Delay>,
    b_btn_pin: &mut Pin<Gpio11, FunctionSio<SioInput>, PullDown>,
    ws: &mut Ws2812<PIO0, SM0, CountDown, Pin<Gpio15, FunctionPio0, PullDown>>,
) where
    C: RgbColor + WebColors + From<Rgb555> + From<Rgb565> + From<Rgb888>,
    D: DrawTarget<Color = C>,
    D::Error: core::fmt::Debug,
{
    display.clear(C::CSS_PURPLE).unwrap();
    let center = display.bounding_box().center();

    Circle::new(Point::new(center.x - 150, center.y - 70), 140)
        .into_styled(PrimitiveStyle::with_fill(C::WHITE))
        .draw(display)
        .unwrap();

    Circle::new(Point::new(center.x + 10, center.y - 70), 140)
        .into_styled(PrimitiveStyle::with_fill(C::WHITE))
        .draw(display)
        .unwrap();

    let mut hue: u8 = 0;

    loop {
        let led1_color = hsv2rgb(Hsv {
            hue,
            sat: 255,
            val: 32,
        });
        let led2_color = hsv2rgb(Hsv {
            hue: 255 - hue,
            sat: 255,
            val: 32,
        });

        let eye1_color: C = rgb8_to_rgb565(&led1_color).into();
        let eye2_color: C = rgb8_to_rgb565(&led2_color).into();

        log_color(display, &eye1_color, &led1_color);

        Circle::new(Point::new(center.x - 140, center.y - 25), 50)
            .into_styled(PrimitiveStyle::with_fill(eye1_color))
            .draw(display)
            .unwrap();

        Circle::new(Point::new(center.x + 20, center.y - 25), 50)
            .into_styled(PrimitiveStyle::with_fill(eye2_color))
            .draw(display)
            .unwrap();

        ws.write([led1_color, led2_color].iter().cloned()).unwrap();
        hue = hue.wrapping_add(1);

        if b_btn_pin.is_low().unwrap() {
            delay.delay_ms(10);
            break;
        }
    }

    ws.write([RGB8::new(0, 0, 0), RGB8::new(0, 0, 0)].iter().cloned())
        .unwrap();
}
