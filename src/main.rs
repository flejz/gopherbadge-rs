#![no_std]
#![no_main]

mod accel_dpad;
mod bmp;
mod gopherbadge_rs;
mod log;
mod menu;
mod neopixel;
mod splash;
mod sprite;

use accel_dpad::accel_dpad;
use defmt_rtt as _;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use embedded_hal_compat::ForwardCompat;
use lis3dh::{DataRate, Lis3dh, Range, SlaveAddr};
use menu::{MenuOption, menu};
use mipidsi::{
    Builder,
    interface::SpiInterface,
    models::ST7789,
    options::{ColorInversion, Orientation, Rotation},
};
use neopixel::neopixel;
use panic_probe as _;

use rp2040_hal::{
    self as hal, I2C, Spi,
    fugit::RateExtU32,
    gpio::{FunctionSpi, Pins},
    pio::PIOExt,
};

use hal::{
    clocks::{Clock, init_clocks_and_plls},
    entry, pac,
    sio::Sio,
    usb::UsbBus,
    watchdog::Watchdog,
};

use splash::splash_screen;
use usb_device::{
    bus::UsbBusAllocator,
    device::{StringDescriptors, UsbDeviceBuilder, UsbVidPid},
};
use usbd_serial::{SerialPort, USB_CLASS_CDC};
use ws2812_pio::Ws2812;

use crate::gopherbadge_rs::gopherbadge_rs;

// the linker will place this boot block at the start of our program image. we
// need this to help the rom bootloader get our code up and running.
// TODO: create a BSP for gopherbadge
#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;
pub const TFT_DISPLAY_HEIGHT: u16 = 240;
pub const TFT_DISPLAY_WIDTH: u16 = 320;

pub static GOPHER_PANIC: &[u8] = include_bytes!("./assets/gopher-panic.bmp");
pub static GOPHERBADGE_RS: &[u8] = include_bytes!("./assets/gopherbadge-rs.bmp");
pub static RUST_PRIDE: &[u8] = include_bytes!("./assets/rust-pride.bmp");
pub static RUST_CRAB: &[u8] = include_bytes!("./assets/crab.bmp");

#[entry]
fn main() -> ! {
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

    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    // -- usb serial
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

    // -- i2c - accelerometer
    let sda = pins.gpio0.reconfigure();
    let scl = pins.gpio1.reconfigure();

    let i2c = I2C::i2c0(
        pac.I2C0,
        sda,
        scl,
        400.kHz(),
        &mut pac.RESETS,
        clocks.system_clock.freq(),
    );

    // sensor driver
    let mut lis3dh = Lis3dh::new_i2c(i2c, SlaveAddr::Default).unwrap();
    lis3dh.set_range(Range::G2).unwrap();
    lis3dh.set_datarate(DataRate::Hz_100).unwrap();

    // -- spi - display
    // control pins
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

    // neopixel led
    let neopixel_pin = pins.gpio15.into_function();
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);

    let mut ws = Ws2812::new(
        neopixel_pin,
        &mut pio,
        sm0,
        clocks.peripheral_clock.freq(),
        timer.count_down(),
    );

    // -- io pins
    // display backlight
    let display_backlight_pin = &mut pins.gpio12.into_push_pull_output();

    // led
    let _backside_led_pin = pins.gpio2.into_push_pull_output();

    // buttons
    let mut a_btn_pin = pins.gpio10.into_pull_down_input();
    let mut b_btn_pin = pins.gpio11.into_pull_down_input();
    let mut down_btn_pin = pins.gpio23.into_pull_down_input();
    let mut up_btn_pin = pins.gpio24.into_pull_down_input();
    let mut left_btn_pin = pins.gpio25.into_pull_down_input();
    let mut right_btn_pin = pins.gpio22.into_pull_down_input();

    splash_screen(
        &mut display,
        &mut delay,
        display_backlight_pin,
        GOPHER_PANIC,
    );

    loop {
        match menu(
            &mut display,
            &mut delay,
            &mut a_btn_pin,
            &mut down_btn_pin,
            &mut up_btn_pin,
        ) {
            MenuOption::Badge => {
                // badge(&mut display, &mut delay);
            }
            MenuOption::AccelerometerDPad => {
                accel_dpad(
                    &mut display,
                    &mut delay,
                    &mut lis3dh,
                    &mut a_btn_pin,
                    &mut b_btn_pin,
                    &mut down_btn_pin,
                    &mut up_btn_pin,
                    &mut left_btn_pin,
                    &mut right_btn_pin,
                );
            }
            MenuOption::Neopixel => {
                neopixel(&mut display, &mut delay, &mut b_btn_pin, &mut ws);
            }
            MenuOption::HuntTheGopher => {
                // hunt_the_gopher(&mut display, &mut delay);
            }
            MenuOption::GopherbadgeRust => {
                gopherbadge_rs(&mut display, &mut delay, &mut b_btn_pin);
            }
        }
    }
}
