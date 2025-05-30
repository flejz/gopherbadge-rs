#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, Primitive, RgbColor, Size},
    primitives::{Circle, PrimitiveStyle, Rectangle},
    text::Text,
    Drawable,
};
use embedded_hal::{delay::DelayNs, digital::OutputPin};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use embedded_hal_compat::ForwardCompat;
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
    Spi,
};

use hal::{
    clocks::{init_clocks_and_plls, Clock},
    entry, pac,
    sio::Sio,
    usb::UsbBus,
    watchdog::Watchdog,
};

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

// External high-speed crystal on the Gopherbadge board is 12 MHz
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

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
            .manufacturer("Fake company")
            .product("Serial port")
            .serial_number("TEST")])
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
        .display_size(240, 320)
        .invert_colors(ColorInversion::Inverted)
        .orientation(orientation)
        .init(&mut delay)
        .unwrap();

    // draw
    draw(&mut display);

    // led
    let mut backside_led_pin = pins.gpio2.into_push_pull_output();
    let mut led_backlight_pin = pins.gpio12.into_push_pull_output();
    led_backlight_pin.set_high().unwrap();

    loop {
        backside_led_pin.set_high().unwrap();
        delay.delay_ms(500);
        backside_led_pin.set_low().unwrap();
        delay.delay_ms(2000);
    }
}

fn draw<D>(display: &mut D)
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    display.clear(Rgb565::BLACK).unwrap();

    Circle::new(Point::new(0, 0), 41)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
        .draw(display)
        .unwrap();

    Rectangle::new(Point::new(20, 20), Size::new(80, 60))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
        .draw(display)
        .unwrap();

    let no_background = MonoTextStyleBuilder::new()
        .font(&FONT_6X9)
        .text_color(Rgb565::WHITE)
        .build();

    let filled_background = MonoTextStyleBuilder::new()
        .font(&FONT_6X9)
        .text_color(Rgb565::YELLOW)
        .background_color(Rgb565::BLUE)
        .build();

    let inverse_background = MonoTextStyleBuilder::new()
        .font(&FONT_6X9)
        .text_color(Rgb565::BLUE)
        .background_color(Rgb565::YELLOW)
        .build();

    Text::new(
        "Hello world! - no background",
        Point::new(15, 15),
        no_background,
    )
    .draw(display)
    .unwrap();

    Text::new(
        "Hello world! - filled background",
        Point::new(15, 30),
        filled_background,
    )
    .draw(display)
    .unwrap();

    Text::new(
        "Hello world! - inverse background",
        Point::new(15, 45),
        inverse_background,
    )
    .draw(display)
    .unwrap();
}
