use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::delay::*;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::ledc::config::TimerConfig;
use esp_idf_svc::hal::ledc::*;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::sntp::*;
use esp_idf_svc::sys::EspError;
use esp_idf_svc::wifi::*;

use std::time::Duration;

use log::*;

mod every;

use ephemeris::{shadow_angle_from_unix_timestamp, MOON_EPHEMERIS};

fn main() -> Result<(), EspError> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // take system peripherals handle
    let p = Peripherals::take()?;
    // take system event loop handle
    let sysloop = EspSystemEventLoop::take()?;

    // -- NVS --
    let nvs = EspNvs::new(EspNvsPartition::<NvsDefault>::take().unwrap(), "moon", true).unwrap();

    // -- WIFI --
    let ssid = "Livebox-DE90";
    let passphrase = Some("ZPLPztn6rPt4N6dP6V");

    let espwifi = EspWifi::new(p.modem, sysloop.clone(), None)?;
    let mut wifi = BlockingWifi::wrap(espwifi, sysloop)?;
    // if a passphrase is set use WPA2 if not use no authentication
    let auth_method = if passphrase.is_some() {
        AuthMethod::WPA2Personal
    } else {
        AuthMethod::None
    };

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid.parse().unwrap(),
        password: passphrase.unwrap_or("").parse().unwrap(),
        channel: None,
        auth_method,
        ..Default::default()
    }))?;

    wifi.start()?;
    wifi.connect()?;
    wifi.wait_netif_up()?;

    // -- SNTP --
    let sntp = EspSntp::new(&SntpConf {
        sync_mode: SyncMode::Smooth,
        ..Default::default()
    })?;

    // wait for NTP sync
    while !matches!(sntp.get_sync_status(), SyncStatus::Completed) {
        info!("syncing SNTP");
        std::thread::sleep(Duration::from_secs(1));
    }

    // -- MOON BACKLIGHT --
    let timer_driver = LedcTimerDriver::new(
        p.ledc.timer0,
        &TimerConfig::default()
            .frequency(1000.Hz().into())
            .resolution(Resolution::Bits14),
    )
    .unwrap();

    let mut dr = LedcDriver::new(p.ledc.channel0, &timer_driver, p.pins.gpio7).unwrap();
    let mut dg = LedcDriver::new(p.ledc.channel1, &timer_driver, p.pins.gpio8).unwrap();
    let mut db = LedcDriver::new(p.ledc.channel2, &timer_driver, p.pins.gpio9).unwrap();

    // build a helper function to set backlight
    let mut set_backlight = move |r: u8, g: u8, b: u8| {
        let duty = |duty: u8, max: u32| -> u32 {
            // scale u8 duty cycle to proper one
            (duty as u32) * 255 / max
        };

        dr.set_duty(duty(r, dr.get_max_duty())).ok();
        dg.set_duty(duty(g, dg.get_max_duty())).ok();
        db.set_duty(duty(b, db.get_max_duty())).ok();
    };

    // quick backlight check
    for (r, g, b) in [(255, 0, 0), (0, 255, 0), (0, 0, 255)] {
        set_backlight(r, g, b);
        std::thread::sleep(Duration::from_millis(100));
    }
    // turn backlight off
    set_backlight(0, 0, 0);

    // -- GLOBE INDEX --
    // turn optical fork on
    let mut index_led = PinDriver::output(p.pins.gpio5)?;
    index_led.set_high()?;
    // setup index as input pulled low
    let mut index = PinDriver::input(p.pins.gpio6)?;
    index.set_pull(Pull::Down)?;

    // build a helper function to accomodate sensor polarity
    let index_detected = move || {
        //
        match index.is_high() {
            true => true,
            false => false,
        }
    };

    // -- STEPPER MOTOR --
    let m1 = PinDriver::output(p.pins.gpio1)?;
    let m2 = PinDriver::output(p.pins.gpio2)?;
    let m3 = PinDriver::output(p.pins.gpio3)?;
    let m4 = PinDriver::output(p.pins.gpio4)?;

    // configure and energize motor
    const STEPS_PER_REV: u32 = 4096;
    let mut motor = embedded_stepper::create_stepper_4pin(m1, m2, m3, m4, Ets, STEPS_PER_REV);
    // set speed in RPMs
    motor.set_speed(60 / 2);

    // indexing steps
    const INDEXING_STEPS_FAST: i32 = 10;
    const INDEXING_STEPS_SLOW: i32 = 1;
    // index motor by turning positive until INDEX is not seen by sensor
    while index_detected() {
        motor.step(INDEXING_STEPS_FAST).ok();
    }
    // continue indexing by turning positive until INDEX is seen
    while !index_detected() {
        motor.step(INDEXING_STEPS_FAST).ok();
    }
    // finalize indexing by turning positive slowly until INDEX is not seen
    while index_detected() {
        motor.step(INDEXING_STEPS_SLOW).ok();
        std::thread::sleep(Duration::from_millis(1));
    }
    // at this point motor should be at located at mechanical reference
    const ZERO_INDEX_OFFSET: u32 = 0;
    // winding motor position in steps
    let mut motor_position = ZERO_INDEX_OFFSET;

    // -- MAIN LOOP --
    loop {
        // get current unix timestamp
        let now = chrono::Utc::now();
        info!("DATE: {now:?} {}", now.timestamp());

        // compute moon shadow angle from unix timestamp
        let angle =
            ephemeris::shadow_angle_from_unix_timestamp(&MOON_EPHEMERIS, now.timestamp()).unwrap();
        info!("ANGLE = {angle}");

        std::thread::sleep(Duration::from_millis(10));
    }
}
