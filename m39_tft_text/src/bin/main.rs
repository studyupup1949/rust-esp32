#![no_std]
#![no_main]

// Usual imports
use defmt::info;
use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp_println as _;

// Embedded Grpahics related
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};

// Larger font
use profont::{PROFONT_18_POINT, PROFONT_24_POINT};

// ESP32 SPI + Display Driver bridge
use display_interface_spi::SPIInterface;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::delay::Delay;
use esp_hal::spi::master::Config as SpiConfig;
use esp_hal::spi::master::Spi;
use esp_hal::spi::Mode as SpiMode;
use esp_hal::time::Rate; // For specifying SPI frequency
use ili9341::{DisplaySize240x320, Ili9341, Orientation};

// For managing GPIO state
use esp_hal::gpio::{Level, Output, OutputConfig};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Initialize SPI
    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(1))
            .with_mode(SpiMode::_0),
    )
    .unwrap()   
    //CLK
    .with_sck(peripherals.GPIO18)
    //DIN
    .with_mosi(peripherals.GPIO23);
    let cs = Output::new(peripherals.GPIO15, Level::Low, OutputConfig::default());
    let dc = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
    let reset = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());
    
    let spi_dev = ExclusiveDevice::new_no_delay(spi, cs);
    let interface = SPIInterface::new(spi_dev, dc);

    let mut display = Ili9341::new(
        interface,
        reset,
        &mut Delay::new(),
        Orientation::Landscape,
        DisplaySize240x320,
    )
    .unwrap();

    display.clear(Rgb565::WHITE).unwrap();

    let text_style = MonoTextStyle::new(&PROFONT_24_POINT, Rgb565::RED);

    Text::with_baseline("impl Rust", Point::new(50, 150), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    let text_style = MonoTextStyle::new(&PROFONT_18_POINT, Rgb565::CSS_DIM_GRAY);

    Text::with_baseline("for ESP32", Point::new(60, 180), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    loop {
        info!("Hello world!");
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(5000) {}
    }
}

