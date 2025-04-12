// main.rs
use serialport::{SerialPort, available_ports};
use std::error::Error;
use std::thread;
use std::time::Duration;
use witmotion_rs::*;

fn try_all_bauds(path: &str) -> Option<(Box<dyn SerialPort>, u32)> {
    let bauds = [
        2400, 4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600,
    ];

    for &baud in bauds.iter() {
        println!("Trying baud: {}", baud);
        if let Ok(mut port) = serialport::new(path, baud)
            .timeout(Duration::from_millis(200))
            .open()
        {
            let _ = send_config_sequence(&mut *port); // send stream enable
            let bytes = read_bytes(&mut *port, 32);
            if !bytes.is_empty() {
                println!("🎯 Success at baud: {}", baud);
                return Some((port, baud));
            }
        }
    }

    None
}

fn main() -> Result<(), Box<dyn Error>> {
    let (mut imu_port, baud) =
        try_all_bauds("/dev/ttyS0").expect("Failed to auto-detect baud rate");
    println!("Using baud {} for IMU", baud);
    println!("Serial port opened");

    send_config_sequence(&mut *imu_port)?;
    println!("Handshake complete, streaming enabled");

    loop {
        let bytes = read_bytes(&mut *imu_port, 64);

        if bytes.is_empty() {
            println!("No data received");
        } else {
            print!("Raw: ");
            for b in &bytes {
                if *b >= 0x20 && *b <= 0x7E {
                    print!("{}", *b as char);
                } else {
                    print!("{:02X} ", b);
                }
            }
            println!();
        }

        thread::sleep(Duration::from_millis(100));
    }
}
