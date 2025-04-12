/*
   Native Rust driver for Witmotion WT901 (proprietary protocol)

   https://fcc.report/FCC-ID/2AZAR-BWT901CL/5176673.pdf

   MIT License

   Copyright (c) 2025 Udi Shamir

   Permission is hereby granted, free of charge, to any person obtaining a copy
   of this software and associated documentation files (the "Software"), to deal
   in the Software without restriction, including without limitation the rights
   to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
   copies of the Software, and to permit persons to whom the Software is
   furnished to do so, subject to the following conditions:

   The above copyright notice and this permission notice shall be included in
   all copies or substantial portions of the Software.

   THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
   IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
   FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
   AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
   LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
   OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
   THE SOFTWARE.
*/

use serialport::SerialPort;
use std::io;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    Acceleration,
    Gyroscope,
    Angle,
    Magnetic,
    Unknown(u8),
}

#[derive(Debug, Clone, Copy)]
pub struct WitmotionFrame {
    pub frame_type: FrameType,
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub temperature: i16,
}

pub fn verify_device_responding(port: &mut dyn SerialPort) -> bool {
    let mut buf = [0u8; 32];

    for _ in 0..3 {
        match port.read(&mut buf) {
            Ok(n) => {
                if n > 0 && buf[..n].contains(&0x55) {
                    return true;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                continue; // retry
            }
            Err(_) => return false, // any other error: fail
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    false
}

pub fn open_serial(
    path: &str,
    baud_rate: u32,
) -> Result<Box<dyn SerialPort>, Box<dyn std::error::Error>> {
    let ports = serialport::available_ports()?;
    if !ports.iter().any(|p| p.port_name == path) {
        return Err(format!("Serial device not found: {}", path).into());
    }

    let port = serialport::new(path, baud_rate)
        .timeout(Duration::from_millis(100))
        .open()?;

    Ok(port)
}

pub fn send_config_sequence(port: &mut dyn SerialPort) -> io::Result<()> {
    let cmds = [
        [0xFF, 0xAA, 0x03, 0x00], // Set protocol to normal
        [0xFF, 0xAA, 0x27, 0x00], // Start streaming
        [0xFF, 0xAA, 0x02, 0x00], // Disable all outputs
        [0xFF, 0xAA, 0x02, 0x01], // Enable Acc
        [0xFF, 0xAA, 0x02, 0x04], // Enable Gyro
        [0xFF, 0xAA, 0x02, 0x08], // Enable Angle
        [0xFF, 0xAA, 0x03, 0x08], // Set output rate to 50Hz
    ];

    for cmd in cmds.iter() {
        port.write_all(cmd)?;
        port.flush()?;
        sleep(Duration::from_millis(100));
    }

    Ok(())
}

pub fn read_bytes(port: &mut dyn SerialPort, max_len: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; max_len];
    match port.read(&mut buffer) {
        Ok(n) => {
            buffer.truncate(n);
            buffer
        }
        Err(_) => vec![],
    }
}

pub const WIT_ACC: u8 = 0x51;
pub const WIT_GYRO: u8 = 0x52;
pub const WIT_ANGLE: u8 = 0x53;

pub fn checksum_valid(data: &[u8]) -> bool {
    if data.len() < 11 {
        return false;
    }

    let sum: u16 = data[..10].iter().map(|&b| b as u16).sum();
    data[10] == (sum & 0xFF) as u8
}

pub fn parse_frame(data: &[u8]) -> Option<WitmotionFrame> {
    if data.len() != 11 || data[0] != 0x55 || !checksum_valid(data) {
        return None;
    }

    let id = data[1];
    let x = i16::from_le_bytes([data[2], data[3]]);
    let y = i16::from_le_bytes([data[4], data[5]]);
    let z = i16::from_le_bytes([data[6], data[7]]);
    let temp = i16::from_le_bytes([data[8], data[9]]);
    let frame_type = match id {
        0x51 => FrameType::Acceleration,
        0x52 => FrameType::Gyroscope,
        0x53 => FrameType::Angle,
        0x54 => FrameType::Magnetic,
        other => FrameType::Unknown(other),
    };

    Some(WitmotionFrame {
        frame_type,
        x,
        y,
        z,
        temperature: temp,
    })
}

pub fn extract_frames(buffer: &[u8]) -> Vec<WitmotionFrame> {
    let mut frames = Vec::new();
    let mut i = 0;
    while i + 11 <= buffer.len() {
        if buffer[i] == 0x55 {
            if let Some(frame) = parse_frame(&buffer[i..i + 11]) {
                frames.push(frame);
                i += 11;
                continue;
            }
        }
        i += 1;
    }
    frames
}
