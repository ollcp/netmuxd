// jkcoxson

use crate::devices::SharedDevices;
use log::info;
use std::net::IpAddr;
use std::sync::Arc;

use tokio::sync::Mutex;

#[cfg(not(feature = "zeroconf"))]
use {
    futures_util::{pin_mut, stream::StreamExt},
    mdns::{Record, RecordKind},
    std::time::Duration,
};

#[cfg(feature = "zeroconf")]
use {
    zeroconf::prelude::*,
    zeroconf::{MdnsBrowser, ServiceType},
};

const SERVICE_NAME: &str = "apple-mobdev2";
const SERVICE_PROTOCOL: &str = "tcp";

#[cfg(feature = "zeroconf")]
pub async fn discover(data: Arc<Mutex<SharedDevices>>) {
    let service_name = format!("_{}._{}.local", SERVICE_NAME, SERVICE_PROTOCOL);
    println!("Starting mDNS discovery for {} with zeroconf", service_name);

    let mut lock = data.lock().await;
    let uuid = "00008110-000A4842149B801E";
    let addr = "192.168.199.174";

    lock.add_network_device(
        udid,
        addr,
        service_name.clone(),
        "Network".to_string(),
        data.clone(),
        );
    println!("Starting mDNS discovery for {} with zeroconf XXXXX", service_name);
    return;

    let mut browser = MdnsBrowser::new(ServiceType::new(SERVICE_NAME, SERVICE_PROTOCOL).unwrap());
    loop {
        let result = browser.browse_async().await;

        if let Ok(service) = result {
            info!("Service discovered: {:?}", service);
            let name = service.name();
            if !name.contains("@") {
                continue;
            }
            let addr = match service.address() {
                addr if addr.contains(":") => IpAddr::V6(addr.parse().unwrap()),
                addr => IpAddr::V4(addr.parse().unwrap()),
            };

            let mac_addr = name.split("@").collect::<Vec<&str>>()[0];
            let mut lock = data.lock().await;
            if let Ok(udid) = lock.get_udid_from_mac(mac_addr.to_string()) {
                if lock.devices.contains_key(&udid) {
                    info!("Device has already been added to muxer, skipping");
                    continue;
                }
                println!("Adding device {}", udid);

                lock.add_network_device(
                    udid,
                    addr,
                    service_name.clone(),
                    "Network".to_string(),
                    data.clone(),
                )
            }
        }
    }
}

#[cfg(not(feature = "zeroconf"))]
pub async fn discover(data: Arc<Mutex<SharedDevices>>) {
    let service_name = format!("_{}._{}.local", SERVICE_NAME, SERVICE_PROTOCOL);
    println!("Starting mDNS discovery for {} with mdns", service_name);

    let mut uuid = std::env::var("UUID").unwrap_or("00008110-000A4842149B801E".to_string());
    let mut xaddr = std::env::var("ADDR").unwrap_or("127.0.0.1".to_string());
    let mut mac_addr = std::env::var("MAC").unwrap_or("30:d5:3e:4f:a7:dc".to_string());
    loop {
        std::thread::sleep(Duration::from_secs(3));
        let mut lock = data.lock().await;
        let addr = std::net::IpAddr::V4(xaddr.parse().unwrap());
        lock.get_udid_from_mac(mac_addr.to_string());

        if let Ok(udid) = lock.get_udid_from_mac(mac_addr.to_string()) {
            if lock.devices.contains_key(&udid) {
                info!("Device has already been added to muxer, skipping");
                continue;
            }
        }


        lock.add_network_device(
            uuid.clone(),
            addr,
            service_name.clone(),
            "Network".to_string(),
            data.clone(),
            );
        println!("Starting mDNS discovery for {} with mdns XXXX", service_name);
    }

}

#[cfg(not(feature = "zeroconf"))]
fn to_ip_addr(record: &Record) -> Option<IpAddr> {
    match record.kind {
        RecordKind::A(addr) => Some(addr.into()),
        RecordKind::AAAA(addr) => Some(addr.into()),
        _ => None,
    }
}
