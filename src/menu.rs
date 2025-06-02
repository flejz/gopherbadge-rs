use cortex_m::delay::Delay;
use embedded_graphics::{
    Drawable,
    mono_font::{MonoTextStyleBuilder, ascii::FONT_9X15_BOLD},
    pixelcolor::{Rgb555, Rgb565, Rgb888},
    prelude::{DrawTarget, Point, RgbColor, WebColors},
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};
use embedded_hal::{delay::DelayNs, digital::InputPin};
use embedded_hal_compat::Forward;
use rp2040_hal::gpio::{
    FunctionSio, Pin, PullDown, SioInput,
    bank0::{Gpio10, Gpio23, Gpio24},
};
use tinybmp::Bmp;

use crate::{RUST_PRIDE, bmp::BmpExt, sprite::SpriteBuilder};

#[derive(PartialEq)]
pub enum MenuOption {
    Badge,
    AccelerometerDPad,
    Neopixel,
    HuntTheGopher,
    GopherbadgeRust,
}

impl MenuOption {
    pub fn options() -> [Self; 5] {
        [
            Self::Badge,
            Self::AccelerometerDPad,
            Self::Neopixel,
            Self::HuntTheGopher,
            Self::GopherbadgeRust,
        ]
    }
}

impl Iterator for MenuOption {
    type Item = MenuOption;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            MenuOption::Badge => Some(MenuOption::AccelerometerDPad),
            MenuOption::AccelerometerDPad => Some(MenuOption::Neopixel),
            MenuOption::Neopixel => Some(MenuOption::HuntTheGopher),
            MenuOption::HuntTheGopher => Some(MenuOption::GopherbadgeRust),
            MenuOption::GopherbadgeRust => Some(MenuOption::Badge),
        }
    }
}

impl DoubleEndedIterator for MenuOption {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            MenuOption::Badge => Some(MenuOption::GopherbadgeRust),
            MenuOption::AccelerometerDPad => Some(MenuOption::Badge),
            MenuOption::Neopixel => Some(MenuOption::AccelerometerDPad),
            MenuOption::HuntTheGopher => Some(MenuOption::Neopixel),
            MenuOption::GopherbadgeRust => Some(MenuOption::HuntTheGopher),
        }
    }
}

impl From<&MenuOption> for Point {
    fn from(option: &MenuOption) -> Self {
        let y = match *option {
            MenuOption::Badge => 20,
            MenuOption::AccelerometerDPad => 40,
            MenuOption::Neopixel => 60,
            MenuOption::HuntTheGopher => 80,
            MenuOption::GopherbadgeRust => 100,
        };

        Point::new(20, y)
    }
}

impl From<&MenuOption> for &'static str {
    fn from(option: &MenuOption) -> Self {
        match *option {
            MenuOption::Badge => "Conference Badge",
            MenuOption::AccelerometerDPad => "Accelerometer + DPad",
            MenuOption::Neopixel => "Neopixel - Sight beyond sight",
            MenuOption::HuntTheGopher => "Hunt the Gopher",
            MenuOption::GopherbadgeRust => "gopherbadge-rs",
        }
    }
}

pub fn menu<D, C>(
    display: &mut D,
    delay: &mut Forward<Delay>,
    a_btn_pin: &mut Pin<Gpio10, FunctionSio<SioInput>, PullDown>,
    down_btn_pin: &mut Pin<Gpio23, FunctionSio<SioInput>, PullDown>,
    up_btn_pin: &mut Pin<Gpio24, FunctionSio<SioInput>, PullDown>,
) -> MenuOption
where
    C: RgbColor + WebColors + From<Rgb555> + From<Rgb565> + From<Rgb888>,
    D: DrawTarget<Color = C>,
    D::Error: core::fmt::Debug,
{
    display.clear(C::CSS_ORANGE_RED).unwrap();
    let mut selected_option = MenuOption::Badge;
    let mut redraw = true;

    let char_style = MonoTextStyleBuilder::new()
        .font(&FONT_9X15_BOLD)
        .text_color(C::CSS_WHITE)
        .background_color(C::CSS_ORANGE_RED)
        .build();

    let selected_char_style = MonoTextStyleBuilder::new()
        .font(&FONT_9X15_BOLD)
        .text_color(C::CSS_ORANGE_RED)
        .background_color(C::CSS_WHITE)
        .build();

    let text_style = TextStyleBuilder::new()
        .alignment(Alignment::Left)
        .baseline(Baseline::Middle)
        .build();

    let rust_logo_bmp = Bmp::from_slice(RUST_PRIDE).unwrap();
    let mut rust_logo_position = rust_logo_bmp.screen_bottom_right();
    rust_logo_position.x -= 10;
    rust_logo_position.y -= 10;
    SpriteBuilder::<C>::builder(rust_logo_bmp)
        .with_position(rust_logo_position)
        .with_transparency(C::BLACK)
        .build()
        .draw_with_transparency(display);

    let text = |option: &MenuOption, selected_option: &MenuOption| {
        Text::with_text_style(
            option.into(),
            option.into(),
            if selected_option == option {
                selected_char_style
            } else {
                char_style
            },
            text_style,
        )
    };

    loop {
        if down_btn_pin.is_low().unwrap() {
            selected_option = selected_option.next().unwrap();
            redraw = true;
        }
        if up_btn_pin.is_low().unwrap() {
            selected_option = selected_option.next_back().unwrap();
            redraw = true;
        }
        if a_btn_pin.is_low().unwrap() {
            break;
        }

        if redraw {
            MenuOption::options().iter().for_each(|option| {
                text(option, &selected_option).draw(display).unwrap();
            });

            delay.delay_ms(200);
            redraw = false;
        }

        delay.delay_ms(10);
    }

    selected_option
}
