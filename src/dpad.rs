use embedded_graphics::{
    pixelcolor::{Rgb555, Rgb565, Rgb888},
    prelude::{DrawTarget, RgbColor, WebColors},
};
use embedded_hal::digital::InputPin;
use rp2040_hal::gpio::{
    bank0::{Gpio11, Gpio22, Gpio23, Gpio24, Gpio25},
    FunctionSio, Pin, PullDown, SioInput,
};
use tinybmp::Bmp;

use crate::{
    bmp::BmpExt,
    log::log_dpad,
    movable_sprite::MovableSpriteBuilder,
    RUST_PRIDE,
};

pub fn dpad<D, C>(
    display: &mut D,
    b_btn_pin: &mut Pin<Gpio11, FunctionSio<SioInput>, PullDown>,
    down_btn_pin: &mut Pin<Gpio23, FunctionSio<SioInput>, PullDown>,
    up_btn_pin: &mut Pin<Gpio24, FunctionSio<SioInput>, PullDown>,
    left_btn_pin: &mut Pin<Gpio25, FunctionSio<SioInput>, PullDown>,
    right_btn_pin: &mut Pin<Gpio22, FunctionSio<SioInput>, PullDown>,
) where
    C: RgbColor + WebColors + From<Rgb555> + From<Rgb565> + From<Rgb888>,
    D: DrawTarget<Color = C>,
    D::Error: core::fmt::Debug,
{
    display.clear(C::BLACK).unwrap();

    let rust_logo: Bmp<C> = Bmp::from_slice(RUST_PRIDE).unwrap();
    let mut rust_logo_position = rust_logo.screen_center();
    let mut rust_logo = MovableSpriteBuilder::builder(rust_logo)
        .with_position(rust_logo_position)
        .with_screen_boundaries()
        .build();

    let mut draw = true;

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

        if draw {
            rust_logo.move_to(display, &mut rust_logo_position, C::BLACK);
            draw = false;
        }

        if b_btn_pin.is_low().unwrap() {
            break;
        }
    }
}
