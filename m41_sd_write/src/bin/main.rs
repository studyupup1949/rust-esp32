#![no_std]
#![no_main]

use chrono::{Datelike, NaiveDateTime, Timelike, TimeZone, NaiveDate, Utc};
use defmt::{info, println};
use embassy_executor::Spawner;
use embassy_time::{Delay, Duration, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::rtc_cntl::Rtc;
use esp_hal::spi;
use esp_hal::spi::master::Spi;
use esp_hal::time::{Rate, }; 
use esp_hal::timer::timg::TimerGroup;
use esp_println::{self as _};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
struct SdTimeSource {
    timer: Rtc<'static>,
}
impl SdTimeSource {
    fn new(timer: Rtc<'static>) -> Self {
        Self { timer }
    }

    fn current_time(&self) -> chrono::NaiveDateTime {
        //self.timer.current_time()
        
        // 1. 从RTC获取微秒时间戳并转换为毫秒
        let ms = (self.timer.current_time_us() / 1000) as i64;
        
        // 2. 处理LocalResult枚举
        match Utc.timestamp_millis_opt(ms) {
            chrono::LocalResult::Single(dt) => dt.naive_utc(),  // 获取DateTime并转换
            chrono::LocalResult::None => panic!("无效的时间戳，超出范围"),  // 处理无效时间戳
            chrono::LocalResult::Ambiguous(dt1, _dt2) => {
                dt1.naive_utc()
            }
        }
        
    }
}

impl TimeSource for SdTimeSource {
    fn get_timestamp(&self) -> Timestamp {
        let now = self.current_time();
        Timestamp {
            year_since_1970: (now.year() - 1970).unsigned_abs() as u8,
            zero_indexed_month: now.month().wrapping_sub(1) as u8,
            zero_indexed_day: now.day().wrapping_sub(1) as u8,
            hours: now.hour() as u8,
            minutes: now.minute() as u8,
            seconds: now.second() as u8,
        }
    }
}


esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // generator version: 0.3.1

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timer0 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    info!("Embassy initialized!");

    // Configure SPI
    let spi_bus = Spi::new(
        peripherals.SPI2,
        spi::master::Config::default()
            .with_frequency(Rate::from_khz(400))
            .with_mode(spi::Mode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO18)
    .with_mosi(peripherals.GPIO23)
    .with_miso(peripherals.GPIO19)
    .into_async();
    let sd_cs = Output::new(peripherals.GPIO5, Level::High, OutputConfig::default());
    let spi_dev = ExclusiveDevice::new(spi_bus, sd_cs, Delay).unwrap();

    // Timer for sdcard
    let rtc = Rtc::new(peripherals.LPWR);
    const CURRENT_TIME: &str = env!("CURRENT_DATETIME");

    let current_time = NaiveDateTime
    ::parse_from_str(CURRENT_TIME, "%Y-%m-%d %H:%M:%S").unwrap();

    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1)
    .and_then(|date| date.and_hms_opt(0, 0, 0))
    .expect("无效的纪元时间");
    // 计算与纪元时间的差值（微秒）并转换为u64
    let timestamp_us = (current_time - epoch).num_microseconds().unwrap() as u64;

    rtc.set_current_time_us(timestamp_us);

    let sd_timer = SdTimeSource::new(rtc);

    let sdcard = SdCard::new(spi_dev, Delay);
    let mut volume_mgr = VolumeManager::new(sdcard, sd_timer);

    println!("Init SD card controller and retrieve card size...");
    let sd_size = volume_mgr.device().num_bytes().unwrap();
    println!("card size is {} bytes\r\n", sd_size);

    let mut volume0 = volume_mgr.open_volume(VolumeIdx(0)).unwrap();
    let mut root_dir = volume0.open_root_dir().unwrap();

    let mut my_file = root_dir
        .open_file_in_dir(
            "WRITE.TXT",
            embedded_sdmmc::Mode::ReadWriteCreateOrTruncate,
        )
        .unwrap();

    let line = "Rust love me, too!";
    if let Ok(()) = my_file.write(line.as_bytes()) {
        my_file.flush().unwrap();
        println!("Written Data");
    } else {
        println!("Not wrote");
    }

    loop {
        Timer::after(Duration::from_secs(30)).await;
    }
}




