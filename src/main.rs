#![no_std]
#![no_main]

mod accel;
mod bmp;
mod dpad;
mod log;
mod menu;
mod movable_sprite;
mod splash;
// mod draw;
// mod sample;

use accel::accel;
use defmt::*;
use defmt_rtt as _;
use dpad::dpad;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use embedded_hal_compat::ForwardCompat;
use lis3dh::{DataRate, Lis3dh, Range, SlaveAddr};
use menu::{menu, MenuOption};
use mipidsi::{
    interface::SpiInterface,
    models::ST7789,
    options::{ColorInversion, Orientation, Rotation},
    Builder,
};
use panic_probe as _;

use rp2040_hal::{
    self as hal,
    fugit::RateExtU32,
    gpio::{FunctionSpi, Pins},
    Spi, I2C,
};

use hal::{
    clocks::{init_clocks_and_plls, Clock},
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

// the linker will place this boot block at the start of our program image. we
// need this to help the rom bootloader get our code up and running.
// TODO: create a BSP for gopherbadge
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;
pub const TFT_DISPLAY_HEIGHT: u16 = 240;
pub const TFT_DISPLAY_WIDTH: u16 = 320;

pub static GOPHER_PANIC: &[u8] = include_bytes!("./assets/gopher-panic.bmp");
pub static RUST_PRIDE: &[u8] = include_bytes!("./assets/rust-pride.bmp");
pub static RUST_CRAB: &[u8] = include_bytes!("./assets/crab.bmp");

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

    // I2C - accelerometer
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

    // Create sensor driver
    let mut lis3dh = Lis3dh::new_i2c(i2c, SlaveAddr::Default).unwrap();
    lis3dh.set_range(Range::G2).unwrap();
    lis3dh.set_datarate(DataRate::Hz_100).unwrap();

    // SPI - display initialization
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

    // state machine
    // let mut state_machine = StateMachine::new(
    //     pins.gpio10.into_pull_down_input(),
    //     pins.gpio11.into_pull_down_input(),
    //     pins.gpio24.into_pull_down_input(),
    //     pins.gpio23.into_pull_down_input(),
    //     pins.gpio25.into_pull_down_input(),
    //     pins.gpio22.into_pull_down_input(),
    //     pins.gpio12.into_push_pull_output(),
    //     pins.gpio2.into_push_pull_output(),
    // );

    // led

    // let _backside_led_pin = pins.gpio2.into_push_pull_output();

    // let mut paint = true;
    let mut a_btn_pin = pins.gpio10.into_pull_down_input();
    let mut b_btn_pin = pins.gpio11.into_pull_down_input();
    let mut down_btn_pin = pins.gpio23.into_pull_down_input();
    let mut up_btn_pin = pins.gpio24.into_pull_down_input();
    let mut left_btn_pin = pins.gpio25.into_pull_down_input();
    let mut right_btn_pin = pins.gpio22.into_pull_down_input();

    // draw
    // let mut rust_logo_position = Point::new(100, 100);
    // let rust_logo: Bmp<Rgb565> = Bmp::from_slice(RUST_PRIDE).unwrap();
    // let mut rust_logo = MovableSpriteBuilder::builder(rust_logo)
    //     .with_position(rust_logo_position)
    //     .with_screen_boundaries()
    //     .build();

    // rust_logo.draw(&mut display);

    splash_screen(
        &mut display,
        &mut pins.gpio12.into_push_pull_output(),
        &mut delay,
        GOPHER_PANIC,
    );

    loop {
        match menu(
            &mut display,
            &mut a_btn_pin,
            &mut down_btn_pin,
            &mut up_btn_pin,
            &mut delay,
        ) {
            MenuOption::Badge => {
                // badge(&mut display, &mut delay);
            }
            MenuOption::Accelerometer => {
                // let accel_f32x3 = lis3dh.accel_norm().unwrap();
                accel(&mut display, &mut lis3dh, &mut b_btn_pin);
            }
            MenuOption::DPad => {
                dpad(
                    &mut display,
                    &mut b_btn_pin,
                    &mut down_btn_pin,
                    &mut up_btn_pin,
                    &mut left_btn_pin,
                    &mut right_btn_pin,
                );
            }
            MenuOption::HuntTheGopher => {
                // hunt_the_gopher(&mut display, &mut delay);
            }
            MenuOption::GopherbadgeRust => {
                // gopherbadge_rust(&mut display, &mut delay);
            }
        }

        // rust_logo_position.x -= (accel.x * 10.0) as i32;
        // rust_logo_position.y -= (accel.y * 10.0) as i32;

        // if r_button_pin.is_low().unwrap() {
        //     rust_logo_position.x += 1;
        //     paint = true;
        // }
        // if l_button_pin.is_low().unwrap() {
        //     rust_logo_position.x -= 1;
        //     paint = true;
        // }
        // if u_button_pin.is_low().unwrap() {
        //     rust_logo_position.y -= 1;
        //     paint = true;
        // }
        // if d_button_pin.is_low().unwrap() {
        //     rust_logo_position.y += 1;
        //     paint = true;
        // }

        // // rust_logo.move_to(&mut display, &mut rust_logo_position, Rgb565::BLACK);
        // if paint {
        //     rust_logo.move_to(&mut display, &mut rust_logo_position, Rgb565::BLACK);
        //     // paint = false;
        // }
    }
}
