use accelerometer::Accelerometer;
use embedded_graphics::{
    pixelcolor::{Rgb555, Rgb565, Rgb888},
    prelude::{DrawTarget, RgbColor, WebColors},
};
use embedded_hal::digital::InputPin;
use lis3dh::{Lis3dh, Lis3dhCore};
use rp2040_hal::gpio::{bank0::Gpio11, FunctionSio, Pin, PullDown, SioInput};
use tinybmp::Bmp;

use crate::{bmp::BmpExt, log::log_accel, sprite::SpriteBuilder, RUST_PRIDE};

pub fn accel<D, C, L>(
    display: &mut D,
    lis3dh: &mut Lis3dh<L>,
    b_btn_pin: &mut Pin<Gpio11, FunctionSio<SioInput>, PullDown>,
) where
    C: RgbColor + WebColors + From<Rgb555> + From<Rgb565> + From<Rgb888>,
    D: DrawTarget<Color = C>,
    D::Error: core::fmt::Debug,
    L: Lis3dhCore,
    L::PinError: core::fmt::Debug,
    L::BusError: core::fmt::Debug,
{
    display.clear(C::BLACK).unwrap();

    let rust_logo: Bmp<C> = Bmp::from_slice(RUST_PRIDE).unwrap();
    let mut rust_logo_position = rust_logo.screen_center();
    let mut rust_logo = SpriteBuilder::builder(rust_logo)
        .with_position(rust_logo_position)
        .with_screen_boundaries()
        .build();

    loop {
        let accel = lis3dh.accel_norm().unwrap();
        log_accel(display, &accel);
        rust_logo_position.x -= (accel.x * 10.0) as i32;
        rust_logo_position.y -= (accel.y * 10.0) as i32;

        rust_logo.move_to(display, &mut rust_logo_position, C::BLACK);

        if b_btn_pin.is_low().unwrap() {
            break;
        }
    }
}
