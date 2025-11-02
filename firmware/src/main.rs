use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::sntp::*;
use esp_idf_svc::sys::EspError;
use esp_idf_svc::wifi::*;

use std::time::Duration;

use log::*;

mod every;

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
    let sntp = EspSntp::new_default()?;

    // -- LOOP --
    loop {
        info!("SNTP: {:?}", sntp.get_sync_status());

        let date = chrono::Utc::now();
        info!("DATE: {date:?}");

        std::thread::sleep(Duration::from_secs(1));
    }
}
