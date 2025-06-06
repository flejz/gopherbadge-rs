use accelerometer::Accelerometer;
use cortex_m::delay::Delay;
use embedded_graphics::{
    Drawable,
    mono_font::{MonoTextStyleBuilder, ascii::FONT_6X9},
    pixelcolor::{Rgb555, Rgb565, Rgb888},
    prelude::{DrawTarget, Point, RgbColor, WebColors},
    text::{Alignment, Text},
};
use embedded_hal::{delay::DelayNs, digital::InputPin};
use embedded_hal_compat::Forward;
use lis3dh::{Lis3dh, Lis3dhCore};
use rp2040_hal::gpio::{
    FunctionSio, Pin, PullDown, SioInput,
    bank0::{Gpio10, Gpio11, Gpio22, Gpio23, Gpio24, Gpio25},
};
use tinybmp::Bmp;

use crate::{
    RUST_PRIDE, TFT_DISPLAY_HEIGHT,
    bmp::BmpExt,
    log::{log_accel, log_dpad},
    sprite::SpriteBuilder,
};

#[allow(clippy::too_many_arguments)]
pub fn accel_dpad<D, C, L>(
    display: &mut D,
    delay: &mut Forward<Delay>,
    lis3dh: &mut Lis3dh<L>,
    a_btn_pin: &mut Pin<Gpio10, FunctionSio<SioInput>, PullDown>,
    b_btn_pin: &mut Pin<Gpio11, FunctionSio<SioInput>, PullDown>,
    down_btn_pin: &mut Pin<Gpio23, FunctionSio<SioInput>, PullDown>,
    up_btn_pin: &mut Pin<Gpio24, FunctionSio<SioInput>, PullDown>,
    left_btn_pin: &mut Pin<Gpio25, FunctionSio<SioInput>, PullDown>,
    right_btn_pin: &mut Pin<Gpio22, FunctionSio<SioInput>, PullDown>,
) where
    C: RgbColor + WebColors + From<Rgb555> + From<Rgb565> + From<Rgb888>,
    D: DrawTarget<Color = C>,
    D::Error: core::fmt::Debug,
    L: Lis3dhCore,
    L::PinError: core::fmt::Debug,
    L::BusError: core::fmt::Debug,
{
    display.clear(C::BLACK).unwrap();

    let rust_logo_bmp: Bmp<C> = Bmp::from_slice(RUST_PRIDE).unwrap();
    let mut rust_logo_position = rust_logo_bmp.screen_center();
    let mut rust_logo = SpriteBuilder::builder(&rust_logo_bmp)
        .with_position(rust_logo_position)
        .with_screen_boundaries()
        .build();

    let mut draw = true;
    let mut dpad = true;

    loop {
        Text::with_alignment(
            if dpad {
                "mode: D-PAD - press A to toggle"
            } else {
                "mode: ACCEL - press A to toggle"
            },
            Point::new(
                display.bounding_box().center().x,
                (TFT_DISPLAY_HEIGHT - 10) as i32,
            ),
            MonoTextStyleBuilder::new()
                .font(&FONT_6X9)
                .text_color(C::WHITE)
                .background_color(C::BLACK)
                .build(),
            Alignment::Center,
        )
        .draw(display)
        .unwrap();

        if !dpad {
            let accel = lis3dh.accel_norm().unwrap();

            log_accel(display, &accel);

            rust_logo_position.x -= (accel.x * 10.0) as i32;
            rust_logo_position.y -= (accel.y * 10.0) as i32;
            draw = true;
        } else {
            let left_is_low = left_btn_pin.is_low().unwrap();
            let right_is_low = right_btn_pin.is_low().unwrap();
            let up_is_low = up_btn_pin.is_low().unwrap();
            let down_is_low = down_btn_pin.is_low().unwrap();

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

            log_dpad(display, (left_is_low, right_is_low, up_is_low, down_is_low));
        }

        if draw {
            rust_logo.move_to(display, &mut rust_logo_position, C::BLACK, 0.0);
            draw = false;
            delay.delay_ms(1);
        }

        if a_btn_pin.is_low().unwrap() {
            dpad = !dpad;
            display.clear(C::BLACK).unwrap();
            draw = true;
            delay.delay_ms(200);
        }

        if b_btn_pin.is_low().unwrap() {
            break;
        }
    }
}
