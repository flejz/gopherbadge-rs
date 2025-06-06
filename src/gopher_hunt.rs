use accelerometer::Accelerometer;
use cortex_m::delay::Delay;
use embedded_graphics::{
    Drawable,
    image::Image,
    pixelcolor::{Rgb555, Rgb565, Rgb888},
    prelude::{DrawTarget, Point, RgbColor, WebColors},
};
use embedded_hal::{delay::DelayNs, digital::InputPin};
use embedded_hal_compat::Forward;
use lis3dh::{Lis3dh, Lis3dhCore};
use micromath::F32Ext;
use rand_chacha::ChaCha8Rng;
use rand_core::{RngCore, SeedableRng};
use rp2040_hal::gpio::{
    FunctionSio, Pin, PullDown, SioInput,
    bank0::{Gpio10, Gpio11},
};
use tinybmp::Bmp;

use crate::{
    GOPHER_DEAD, GOPHER_HEAD, RUST_CRAB, TFT_DISPLAY_HEIGHT, TFT_DISPLAY_WIDTH,
    bmp::BmpExt,
    log::{log_accel, log_angle},
    sprite::{Sprite, SpriteBuilder},
};

fn accel_to_angle_deg(x: f32, y: f32) -> f32 {
    let radians = y.atan2(x);
    radians.to_degrees() - 90.0
}

fn is_colliding<'a, C>(a: &Sprite<'a, C>, b: &Sprite<'a, C>) -> bool
where
    C: RgbColor + WebColors + From<Rgb555> + From<Rgb565> + From<Rgb888>,
{
    let a_center = a.center();
    let b_center = b.center();
    a_center.x.abs_diff(b_center.x) < 15 && a_center.y.abs_diff(b_center.y) < 15
}

pub fn run_away_from<'a, C>(
    runner: &'a Sprite<'a, C>,
    chaser: &'a Sprite<'a, C>,
    rng: &mut impl RngCore,
    max_speed: i32,
    panic_distance: i32,
) -> Point
where
    C: RgbColor + WebColors + From<Rgb555> + From<Rgb565> + From<Rgb888>,
{
    let to_runner = runner.center() - chaser.center();
    let dist = ((to_runner.x.pow(2) + to_runner.y.pow(2)) as f32).sqrt();
    let panic_mode = dist < panic_distance as f32;

    let (dx, dy) = if panic_mode {
        (to_runner.x.clamp(-1, 1), to_runner.y.clamp(-1, 1))
    } else {
        (
            (rng.next_u32() % 3).wrapping_sub(1) as i32,
            (rng.next_u32() % 3).wrapping_sub(1) as i32,
        )
    };

    let jitter_x = (rng.next_u32() % 3).wrapping_sub(1) as i32;
    let jitter_y = (rng.next_u32() % 3).wrapping_sub(1) as i32;

    let mut move_vec = Point::new(dx + jitter_x, dy + jitter_y);

    if move_vec == Point::new(0, 0) {
        move_vec = if panic_mode {
            Point::new(dx, dy)
        } else {
            Point::new(jitter_x.clamp(-1, 1), jitter_y.clamp(-1, 1))
        };
    }

    let speed = if panic_mode {
        ((panic_distance as f32 / (dist + 1.0)) * max_speed as f32)
            .min(max_speed as f32)
            .max(1.0)
            .round() as i32
    } else {
        (rng.next_u32() % (max_speed as u32 + 1)) as i32
    };

    let center = runner.center() + move_vec * speed;

    let half_width = (runner.size().width / 2) as i32;
    let half_height = (runner.size().height / 2) as i32;

    let new_top_left = Point::new(center.x - half_width, center.y - half_height);

    let wrapped_x = new_top_left.x.rem_euclid(TFT_DISPLAY_WIDTH as i32);
    let wrapped_y = new_top_left.y.rem_euclid(TFT_DISPLAY_HEIGHT as i32);

    Point::new(wrapped_x, wrapped_y)
}

pub fn gopher_hunt<D, C, L>(
    display: &mut D,
    delay: &mut Forward<Delay>,
    lis3dh: &mut Lis3dh<L>,
    a_btn_pin: &mut Pin<Gpio10, FunctionSio<SioInput>, PullDown>,
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
    let mut player = SpriteBuilder::builder(&player_bmp)
        .with_position(player_position)
        .with_screen_boundaries()
        .build();

    let gopher_bmp = Bmp::from_slice(GOPHER_HEAD).unwrap();
    let mut gopher = SpriteBuilder::builder(&gopher_bmp)
        .with_position(Point::new(100, 100))
        .with_screen_boundaries()
        .build();

    gopher.draw(display, 0.0);

    let mut rng = ChaCha8Rng::seed_from_u64(0x12345678);
    let mut draw = false;
    let mut dead = false;

    loop {
        if dead {
            if draw {
                display.clear(C::WHITE).unwrap();
                let gopher_dead = Bmp::from_slice(GOPHER_DEAD).unwrap();
                Image::new(&gopher_dead, gopher_dead.screen_center())
                    .draw(display)
                    .unwrap();
                draw = false;
            }

            delay.delay_ms(100);

            if a_btn_pin.is_low().unwrap() {
                dead = false;
                gopher.move_to(display, &mut gopher_bmp.screen_center(), C::BLACK, 0.0);
                display.clear(C::BLACK).unwrap();
            }
            continue;
        }

        let accel = lis3dh.accel_norm().unwrap();

        log_accel(display, &accel);

        player_position.x -= (accel.x * 10.0) as i32;
        player_position.y -= (accel.y * 10.0) as i32;

        let angle = accel_to_angle_deg(accel.x, accel.y);

        log_angle(display, angle);

        player.move_to(display, &mut player_position, C::BLACK, angle);
        delay.delay_ms(1);

        let mut new_pos = run_away_from(&gopher, &player, &mut rng, 6, 70);
        gopher.move_to(display, &mut new_pos, C::BLACK, 0.0);

        if is_colliding(&gopher, &player) {
            draw = true;
            dead = true;
        }

        if b_btn_pin.is_low().unwrap() {
            break;
        }
    }
}
