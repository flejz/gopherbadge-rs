use cortex_m::delay::Delay;
use embedded_graphics::{
    pixelcolor::{Rgb555, Rgb565, Rgb888},
    prelude::{DrawTarget, RgbColor, WebColors},
};
use embedded_hal::{delay::DelayNs, digital::InputPin};
use embedded_hal_compat::Forward;
use rp2040_hal::{
    gpio::{
        bank0::{Gpio11, Gpio15, Gpio22, Gpio23, Gpio24, Gpio25},
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
use tinybmp::Bmp;
use ws2812_pio::Ws2812;

use crate::{bmp::BmpExt, log::log_dpad, sprite::SpriteBuilder, RUST_PRIDE};

#[allow(clippy::too_many_arguments)]
pub fn neopixel<D, C>(
    display: &mut D,
    delay: &mut Forward<Delay>,
    b_btn_pin: &mut Pin<Gpio11, FunctionSio<SioInput>, PullDown>,
    down_btn_pin: &mut Pin<Gpio23, FunctionSio<SioInput>, PullDown>,
    up_btn_pin: &mut Pin<Gpio24, FunctionSio<SioInput>, PullDown>,
    left_btn_pin: &mut Pin<Gpio25, FunctionSio<SioInput>, PullDown>,
    right_btn_pin: &mut Pin<Gpio22, FunctionSio<SioInput>, PullDown>,
    ws: &mut Ws2812<PIO0, SM0, CountDown, Pin<Gpio15, FunctionPio0, PullDown>>,
) where
    C: RgbColor + WebColors + From<Rgb555> + From<Rgb565> + From<Rgb888>,
    D: DrawTarget<Color = C>,
    D::Error: core::fmt::Debug,
{
    display.clear(C::BLACK).unwrap();

    let rust_logo: Bmp<C> = Bmp::from_slice(RUST_PRIDE).unwrap();
    let mut rust_logo_position = rust_logo.screen_center();
    let mut rust_logo = SpriteBuilder::builder(rust_logo)
        .with_position(rust_logo_position)
        .with_screen_boundaries()
        .build();

    let mut draw = true;
    let mut hue: u8 = 0;

    loop {
        let left_is_low = left_btn_pin.is_low().unwrap();
        let right_is_low = right_btn_pin.is_low().unwrap();
        let up_is_low = up_btn_pin.is_low().unwrap();
        let down_is_low = down_btn_pin.is_low().unwrap();

        log_dpad(display, (left_is_low, right_is_low, up_is_low, down_is_low));

        if right_is_low {
            rust_logo_position.x += 1;
            draw = true;
        }
        if left_is_low {
            rust_logo_position.x -= 1;
            draw = true;
        }
        if up_is_low {
            rust_logo_position.y -= 1;
            draw = true;
        }
        if down_is_low {
            rust_logo_position.y += 1;
            draw = true;
        }

        let led1 = hsv2rgb(Hsv {
            hue,
            sat: 255,
            val: 32,
        });
        let led2 = hsv2rgb(Hsv {
            hue: 255 - hue,
            sat: 255,
            val: 32,
        });

        let data = [led1, led2];
        ws.write(data.iter().cloned()).unwrap();
        hue = hue.wrapping_add(1);

        if draw {
            rust_logo.move_to(display, &mut rust_logo_position, C::BLACK);
            draw = false;
        } else {
            delay.delay_ms(20);
        }

        if b_btn_pin.is_low().unwrap() {
            break;
        }
    }

    let data = [RGB8::new(0, 0, 0), RGB8::new(0, 0, 0)];
    ws.write(data.iter().cloned()).unwrap();
}
