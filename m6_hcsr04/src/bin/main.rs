#![no_std]
#![no_main]

use esp_hal::clock::CpuClock;
use esp_hal::gpio::{InputConfig, OutputConfig};
use esp_hal::ledc::{LSGlobalClkSource, LowSpeed};
use esp_hal::main;
use esp_hal::time::Rate;

use esp_hal::{
    delay::Delay,
    gpio::{Input, Level, Output, Pull},
    ledc::{
        channel::{self, ChannelIFace},
        timer::{self, TimerIFace},
        Ledc,
    },
    rtc_cntl::Rtc,
};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // generator version: 0.3.1

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let led = peripherals.GPIO2; // uses onboard LED
    //let led = peripherals.GPIO33;

    // Configure LEDC
    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty5Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_khz(24),
        })
        .unwrap();
    let mut channel0 = ledc.channel(channel::Number::Channel0, led);
    channel0
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 10,
            pin_config: channel::config::PinConfig::PushPull,
        })
        .unwrap();

    // For HC-SR04 Ultrasonic
    let mut trig = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default());
    let echo = Input::new(
        peripherals.GPIO18,
        InputConfig::default().with_pull(Pull::Down),
    );

    let delay = Delay::new(); // We can use this since we are using unstable features

    let rtc = Rtc::new(peripherals.LPWR);  //初始化实时时钟，记录时间戳

    loop {
        delay.delay_millis(5);

        // Trigger ultrasonic waves
        trig.set_low();
        delay.delay_micros(2);  //微妙
        trig.set_high();
        delay.delay_micros(10);
        trig.set_low();

       
        // 测量信号保持高电平的持续时间
        while echo.is_low() {}
        let time1 = rtc.current_time_us();  // 高电平开始时间
        while echo.is_high() {}
        let time2 = rtc.current_time_us();  // 高电平结束时间

        // 直接计算差值（单位：微秒），注意处理时间戳溢出的情况
        let pulse_width = if time2 >= time1 {
            (time2 - time1) as f64  // 正常情况：直接相减得到微秒数
        } else {
            // 处理时间戳溢出（如果 RTC 计数器归零）
            (u64::MAX - time1 + time2) as f64
        };

        // Derive distance from the pulse width
        let distance = (pulse_width * 0.0343) / 2.0;
        // esp_println::println!("Pulse Width: {}", pulse_width);
        // esp_println::println!("Distance: {}", distance);

        // Our own logic to calculate duty cycle percentage for the distance
        let duty_pct: u8 = if distance < 30.0 {
            let ratio = (30.0 - distance) / 30.0;
            let p = (ratio * 100.0) as u8;
            p.min(100)
        } else {
            0
        };

        if let Err(e) = channel0.set_duty(duty_pct) {
            // esp_println::println!("Failed to set duty cycle: {:?}", e);
            panic!("Failed to set duty cycle: {:?}", e);
        }

        delay.delay_millis(60);
    }
}
