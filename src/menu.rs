use cortex_m::delay::Delay;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyleBuilder},
    pixelcolor::{Rgb555, Rgb565, Rgb888},
    prelude::{DrawTarget, Point, RgbColor, WebColors},
    text::{Alignment, Baseline, Text, TextStyleBuilder},
    Drawable,
};
use embedded_hal::{delay::DelayNs, digital::InputPin};
use embedded_hal_compat::Forward;
use rp2040_hal::gpio::{
    bank0::{Gpio10, Gpio23, Gpio24},
    FunctionSio, Pin, PullDown, SioInput,
};

#[derive(PartialEq)]
pub enum MenuOption {
    Badge,
    Accelerometer,
    DPad,
    HuntTheGopher,
    GopherbadgeRust,
}

impl MenuOption {
    pub fn options() -> [Self; 5] {
        [
            Self::Badge,
            Self::Accelerometer,
            Self::DPad,
            Self::HuntTheGopher,
            Self::GopherbadgeRust,
        ]
    }
}

impl Iterator for MenuOption {
    type Item = MenuOption;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            MenuOption::Badge => Some(MenuOption::Accelerometer),
            MenuOption::Accelerometer => Some(MenuOption::DPad),
            MenuOption::DPad => Some(MenuOption::HuntTheGopher),
            MenuOption::HuntTheGopher => Some(MenuOption::GopherbadgeRust),
            MenuOption::GopherbadgeRust => Some(MenuOption::Badge),
        }
    }
}

impl DoubleEndedIterator for MenuOption {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            MenuOption::Badge => Some(MenuOption::GopherbadgeRust),
            MenuOption::Accelerometer => Some(MenuOption::Badge),
            MenuOption::DPad => Some(MenuOption::Accelerometer),
            MenuOption::HuntTheGopher => Some(MenuOption::DPad),
            MenuOption::GopherbadgeRust => Some(MenuOption::HuntTheGopher),
        }
    }
}

impl From<&MenuOption> for Point {
    fn from(option: &MenuOption) -> Self {
        let y = match *option {
            MenuOption::Badge => 20,
            MenuOption::Accelerometer => 50,
            MenuOption::DPad => 80,
            MenuOption::HuntTheGopher => 110,
            MenuOption::GopherbadgeRust => 140,
        };

        Point::new(20, y)
    }
}

impl From<&MenuOption> for &'static str {
    fn from(option: &MenuOption) -> Self {
        match *option {
            MenuOption::Badge => "Conference Badge",
            MenuOption::Accelerometer => "Accelerometer",
            MenuOption::DPad => "DPad",
            MenuOption::HuntTheGopher => "Hunt the Gopher",
            MenuOption::GopherbadgeRust => "gopherbadge-rs",
        }
    }
}

pub fn menu<D, C>(
    display: &mut D,
    a_btn_pin: &mut Pin<Gpio10, FunctionSio<SioInput>, PullDown>,
    down_btn_pin: &mut Pin<Gpio23, FunctionSio<SioInput>, PullDown>,
    up_btn_pin: &mut Pin<Gpio24, FunctionSio<SioInput>, PullDown>,
    delay: &mut Forward<Delay>,
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
        .font(&FONT_10X20)
        .text_color(C::CSS_WHITE)
        .background_color(C::CSS_ORANGE_RED)
        .build();

    let selected_char_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(C::CSS_ORANGE_RED)
        .background_color(C::CSS_WHITE)
        .build();

    let text_style = TextStyleBuilder::new()
        .alignment(Alignment::Left)
        .baseline(Baseline::Middle)
        .build();

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
