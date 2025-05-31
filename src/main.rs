#![no_std]
#![no_main]

mod draw;
mod movable_sprite;
mod sample;

use defmt::*;
use defmt_rtt as _;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, RgbColor},
    Drawable,
};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use embedded_hal_compat::ForwardCompat;
use mipidsi::{
    interface::SpiInterface,
    models::ST7789,
    options::{ColorInversion, Orientation, Rotation},
    Builder,
};
use movable_sprite::MovableSprite;
use panic_probe as _;

use rp2040_hal::{
    self as hal,
    fugit::RateExtU32,
    gpio::{FunctionSpi, Pins},
    Spi,
};

use hal::{
    clocks::{init_clocks_and_plls, Clock},
    entry, pac,
    sio::Sio,
    usb::UsbBus,
    watchdog::Watchdog,
};

use tinybmp::Bmp;
use usb_device::{
    bus::UsbBusAllocator,
    device::{StringDescriptors, UsbDeviceBuilder, UsbVidPid},
};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

// the linker will place this boot block at the start of our program image. we
// need this to help the rom bootloader get our code up and running.
// TODO: create a BSP for gopherbadge
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;
pub const TFT_DISPLAY_HEIGHT: u16 = 240;
pub const TFT_DISPLAY_WIDTH: u16 = 320;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    let clocks = init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let mut delay = delay.forward();

    // usb
    let usb_bus = UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        false,
        &mut pac.RESETS,
    );

    let usb_bus = UsbBusAllocator::new(usb_bus);
    let mut _serial = SerialPort::new(&usb_bus);

    let mut _usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::default()
            .manufacturer("Evil Corp")
            .product("go-desecrator")
            .serial_number("N#-of-the-BEST")]) // not beast :)
        .unwrap()
        .device_class(USB_CLASS_CDC) // from: https://www.usb.org/defined-class-codes
        .build();

    // display initialization
    // spi & control pins
    let sck = pins.gpio18.into_function::<FunctionSpi>();
    let mosi = pins.gpio19.into_function::<FunctionSpi>();
    let dc = pins.gpio20.into_push_pull_output();
    let cs = pins.gpio17.into_push_pull_output();

    // spi device
    let spi = Spi::<_, _, _>::new(pac.SPI0, (mosi, sck)).init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        16_000_000u32.Hz(),
        embedded_hal::spi::MODE_3,
    );

    let spi_device = ExclusiveDevice::new(spi, cs, NoDelay).unwrap();

    // display interface
    let mut buffer = [0_u8; 512];
    let di = SpiInterface::new(spi_device, dc, &mut buffer);

    // display initialization
    let orientation = Orientation::new();
    let orientation = orientation.rotate(Rotation::Deg270);
    let mut display = Builder::new(ST7789, di)
        .display_size(TFT_DISPLAY_HEIGHT, TFT_DISPLAY_WIDTH)
        .invert_colors(ColorInversion::Inverted)
        .orientation(orientation)
        .init(&mut delay)
        .unwrap();

    // light up display backlight and clear display
    pins.gpio12.into_push_pull_output().set_high().unwrap();
    display.clear(Rgb565::BLACK).unwrap();

    // led
    let backside_led_pin = pins.gpio2.into_push_pull_output();
    let a_button_pin = pins.gpio10.into_pull_down_input();

    let mut moved = true;
    let mut r_button_pin = pins.gpio22.into_pull_down_input();
    let mut d_button_pin = pins.gpio23.into_pull_down_input();
    let mut u_button_pin = pins.gpio24.into_pull_down_input();
    let mut l_button_pin = pins.gpio25.into_pull_down_input();

    // draw
    const RUST_LOGO: &'static [u8] = include_bytes!("./assets/rust-pride.bmp");
    let mut rust_logo_position = Point::new(100, 100);
    let mut rust_logo = MovableSprite::new(
        Bmp::from_slice(RUST_LOGO).unwrap(),
        rust_logo_position.clone(),
    );
    rust_logo.draw(&mut display);

    loop {
        if r_button_pin.is_low().unwrap() {
            rust_logo_position.x += 1;
            moved = true;
        }
        if l_button_pin.is_low().unwrap() {
            rust_logo_position.x -= 1;
            moved = true;
        }
        if u_button_pin.is_low().unwrap() {
            rust_logo_position.y -= 1;
            moved = true;
        }
        if d_button_pin.is_low().unwrap() {
            rust_logo_position.y += 1;
            moved = true;
        }

        if moved {
            rust_logo.move_to(&mut display, &mut rust_logo_position, Rgb565::BLACK);
            moved = false;
        }
        // delay.delay_ms(10);
    }
}
