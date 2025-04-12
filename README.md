# Witmotion BWT901CL Rust Port

A native Rust library for interfacing with the Witmotion BWT901CL IMU sensor using its proprietary serial protocol.

Features
	+ Supports Acceleration, Gyroscope, Angle, and Magnetic field data frames.
	+ Parses sensor data into structured WitmotionFrame types.
	+ Handles serial communication with configurable baud rates.
	+ Includes temperature data parsing from sensor frames. ￼

Installation

Add this to your Cargo.toml:

[[dependencies]]
witmotion-rs = "0.1.0"

Usage: 

use witmotion_rs::{open_serial, send_config_sequence, read_bytes, extract_frames};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port_path = "/dev/ttyUSB0";
    let baud_rate = 115200;

    let mut port = open_serial(port_path, baud_rate)?;
    send_config_sequence(&mut *port)?;

    loop {
        let buffer = read_bytes(&mut *port, 256);
        let frames = extract_frames(&buffer);

        for frame in frames {
            println!("{:?}", frame);
        }
    }
}

Protocol Overview

The BWT901 sensor communicates over a serial interface using a proprietary protocol. Each data frame is 11 bytes long and starts with a 0x55 header byte, followed by a frame type identifier, sensor data, temperature data, and a checksum.

When starting the communication with the controller make sure to set the protocol to normal
[[0xFF, 0xAA, 0x03, 0x00]]

+ 0x03 → protocol register
+ proprietary binary protocol
+ Other values (e.g., 0x01, 0x05) might indicate Modbus, CAN, havent figured this out yet

0x50 = frame ID for time-related data.
0x03 = register for protocol selection.
0x00 = proprietary protocol.

Frame types include:
	+ 0x51: Acceleration
	+ 0x52: Gyroscope
	+ 0x53: Angle
	+ 0x54: Magnetic field

Temperature is included in each frame as two bytes (TL and TH). The temperature in degrees Celsius is calculated using the formula:
Temperature (°C) = ((TH << 8) | TL) / 100

For example, if TH = 0x0E and TL = 0x48, then:

Temperature = ((0x0E << 8) | 0x48) / 100 = (0x0E48) / 100 = 3656 / 100 = 36.56°C
 
This calculation aligns with the sensor’s data format as specified in the sensor specification: https://fcc.report/FCC-ID/2AZAR-BWT901CL/5176673.pdf

Further information regarding the Witmotion BWT901CL can be found in the the manufacture web site: https://fcc.report/FCC-ID/2AZAR-BWT901CL/5176673.pdf

Notes on Temperature Data

The temperature data is embedded in each sensor frame and should be interpreted using the formula provided above. Ensure that your parsing logic correctly extracts the TL and TH bytes and applies the formula to obtain accurate temperature readings.

License

This project is licensed under the MIT License.
