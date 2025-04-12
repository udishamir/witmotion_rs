/*
   Interacting with Witmotion BWT901CL Accelerometer

   By Udi Shamir

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

use std::env;
use witmotion_rs::{WitmotionFrame, extract_frames, open_serial, send_config_sequence};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let use_scaled = args.iter().any(|arg| arg == "--scaled");

    let port_name = "/dev/ttyUSB0";
    let baud = 115200;
    let mut port = open_serial(port_name, baud)?;
    println!("Serial port opened");

    send_config_sequence(&mut *port)?;
    println!("Handshake complete, streaming enabled");

    loop {
        let buffer = witmotion_rs::read_bytes(&mut *port, 128);
        let frames = extract_frames(&buffer);
        if frames.is_empty() {
            println!("No data received");
        }

        for frame in frames {
            if use_scaled {
                print_scaled(&frame);
            } else {
                println!("{:?}", frame);
            }
        }
    }
}

fn print_scaled(frame: &WitmotionFrame) {
    match frame.frame_type {
        witmotion_rs::FrameType::Acceleration => {
            println!(
                "Accel [g]: x={:.3}, y={:.3}, z={:.3}",
                scale_accel(frame.x),
                scale_accel(frame.y),
                scale_accel(frame.z)
            );
        }
        witmotion_rs::FrameType::Gyroscope => {
            println!(
                "Gyro [°/s]: x={:.1}, y={:.1}, z={:.1}",
                scale_gyro(frame.x),
                scale_gyro(frame.y),
                scale_gyro(frame.z)
            );
        }
        witmotion_rs::FrameType::Angle => {
            println!(
                "Angle [°]: x={:.2}, y={:.2}, z={:.2}",
                scale_angle(frame.x),
                scale_angle(frame.y),
                scale_angle(frame.z)
            );
        }
        witmotion_rs::FrameType::Magnetic => {
            println!(
                "Mag [µT]: x={:.1}, y={:.1}, z={:.1}",
                scale_magnetic(frame.x),
                scale_magnetic(frame.y),
                scale_magnetic(frame.z)
            );
        }
        witmotion_rs::FrameType::Unknown(id) => {
            println!("Unknown FrameType({}) {:?}", id, frame);
        }
    }
}

fn scale_accel(v: i16) -> f32 {
    v as f32 / 32768.0 * 16.0 // ±16g
}

fn scale_gyro(v: i16) -> f32 {
    v as f32 / 32768.0 * 2000.0 // ±2000°/s
}

fn scale_angle(v: i16) -> f32 {
    v as f32 / 32768.0 * 180.0 // ±180°, assuming full range
}

fn scale_magnetic(v: i16) -> f32 {
    v as f32 // assume µT, device-dependent
}
