// jkcoxson

use std::sync::Arc;

use tokio::{io::AsyncReadExt, sync::Mutex};

use crate::handle::{cope, instruction};

mod central_data;
mod handle;
mod heartbeat;
mod mdns;
mod raw_packet;

#[tokio::main]
async fn main() {
    println!("Starting netmuxd");

    let mut port = 27015;
    let mut host = "127.0.0.1".to_string();
    let mut plist_storage = None;

    // Loop through args
    let mut i = 0;
    while i < std::env::args().len() {
        match std::env::args().nth(i).unwrap().as_str() {
            "-p" | "--port" => {
                port = std::env::args().nth(i + 1).unwrap().parse::<i32>().unwrap();
                i += 2;
            }
            "-h" | "--host" => {
                host = std::env::args().nth(i + 1).unwrap().to_string();
                i += 2;
            }
            "--plist-storage" => {
                plist_storage = Some(std::env::args().nth(i + 1).unwrap());
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    let data = Arc::new(Mutex::new(central_data::CentralData::new(plist_storage)));
    let data_clone = data.clone();
    tokio::spawn(async move {
        mdns::discover(data_clone).await;
        println!("mDNS discovery stopped, how the heck did you break this");
    });

    // Create TcpListener
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();

    println!("Listening on {}:{}", host, port);

    loop {
        let (mut socket, _) = match listener.accept().await {
            Ok(s) => s,
            Err(_) => {
                continue;
            }
        };
        let cloned_data = data.clone();
        // Wait for a message from the client
        let mut buf = [0; 1024];
        let size = match socket.read(&mut buf).await {
            Ok(s) => s,
            Err(e) => {
                break;
            }
        };
        if size == 0 {
            break;
        }
        let buffer = &buf[0..size];

        let parsed: raw_packet::RawPacket = buffer.into();

        if parsed.message == 69 && parsed.tag == 69 {
            instruction(parsed, socket, cloned_data.clone())
                .await
                .unwrap();
        } else {
            cope(parsed, socket, cloned_data).await.unwrap();
        }
    }
}