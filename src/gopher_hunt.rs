
use accelerometer::Accelerometer;
use embedded_graphics::{
    pixelcolor::{Rgb555, Rgb565, Rgb888},
    prelude::{DrawTarget, RgbColor, WebColors},
};
use embedded_hal::digital::InputPin;
use lis3dh::{Lis3dh, Lis3dhCore};
use micromath::F32Ext;
use rp2040_hal::gpio::{
    FunctionSio, Pin, PullDown, SioInput,
    bank0::Gpio11,
};
use tinybmp::Bmp;

use crate::{
    RUST_CRAB,
    bmp::BmpExt,
    log::{log_accel, log_angle},
    sprite::{Sprite, SpriteBuilder},
};

fn accel_to_angle_deg(x: f32, y: f32) -> f32 {
    let radians = y.atan2(x);
    radians.to_degrees() - 90.0
}

fn is_colliding<'a, C>(a: &Sprite<'a, C>, b: &Sprite<'a, C>) -> bool {
    a.pos.x.abs_diff(b.pos.x) < 10 && a.pos.y.abs_diff(b.pos.y) < 10
}

pub fn gopher_hunt<D, C, L>(
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
    let player_bmp = Bmp::from_slice(RUST_CRAB).unwrap();
    let mut player_position = player_bmp.screen_center();
    let mut player = SpriteBuilder::builder(player_bmp)
        .with_position(player_position)
        .with_screen_boundaries()
        .build();

    loop {
        let accel = lis3dh.accel_norm().unwrap();

        log_accel(display, &accel);

        player_position.x -= (accel.x * 10.0) as i32;
        player_position.y -= (accel.y * 10.0) as i32;

        let angle = accel_to_angle_deg(accel.x, accel.y);

        log_angle(display, angle);

        player.move_to(display, &mut player_position, C::BLACK, angle);

        if b_btn_pin.is_low().unwrap() {
            break;
        }
    }
}
