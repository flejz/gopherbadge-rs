use cortex_m::delay::Delay;
use embedded_graphics::{
    pixelcolor::{Rgb555, Rgb565, Rgb888},
    prelude::{DrawTarget, Point, RgbColor, WebColors},
};
use embedded_hal::{delay::DelayNs, digital::InputPin};
use embedded_hal_compat::Forward;
use rp2040_hal::gpio::{FunctionSio, Pin, PullDown, SioInput, bank0::Gpio11};
use tinybmp::Bmp;

use crate::{GOPHERBADGE_RS, sprite::SpriteBuilder};

pub fn gopherbadge_rs<D, C>(
    display: &mut D,
    delay: &mut Forward<Delay>,
    b_btn_pin: &mut Pin<Gpio11, FunctionSio<SioInput>, PullDown>,
) where
    C: RgbColor + WebColors + From<Rgb555> + From<Rgb565> + From<Rgb888>,
    D: DrawTarget<Color = C>,
    D::Error: core::fmt::Debug,
{
    SpriteBuilder::builder(Bmp::from_slice(GOPHERBADGE_RS).unwrap())
        .with_position(Point::new(0, 0))
        .build()
        .draw(display);

    loop {
        if b_btn_pin.is_low().unwrap() {
            break;
        }
        delay.delay_ms(100);
    }
}
