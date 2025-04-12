// main.rs
use std::error::Error;
use std::thread;
use std::time::Duration;
use witmotion_rs::*;

fn main() -> Result<(), Box<dyn Error>> {
    let mut port = open_serial("/dev/ttyACM0", 115200)?;
    println!("Serial port opened");

    send_config_sequence(&mut *port)?;
    println!("Handshake complete, streaming enabled");

    let mut buffer: Vec<u8> = Vec::new();

    loop {
        let mut temp = read_bytes(&mut *port, 64);
        buffer.append(&mut temp);

        while buffer.len() >= 11 {
            if buffer[0] != 0x55 {
                buffer.remove(0);
                continue;
            }

            if let Some(frame) = parse_frame(&buffer[..11]) {
                println!(
                    "{:?}: x={} y={} z={} temp={}",
                    frame.frame_type, frame.x, frame.y, frame.z, frame.temperature
                );
                buffer.drain(..11);
            } else {
                buffer.remove(0);
            }
        }

        thread::sleep(Duration::from_millis(10));
    }
}

