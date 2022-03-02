use bno055::{BNO055OperationMode, Bno055};
use env_logger::{Builder, Env};
use log::{debug, error, info, trace, warn};
use std::env;
use std::io::prelude::*;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;
use xca9548a::{SlaveAddr, Xca9548a};

struct SuitCommand;
impl SuitCommand {
    pub const ID: [u8; 4] = [b'M', b'S', b'I', b'D'];
    pub const READY: [u8; 4] = [b'R', b'E', b'D', b'Y'];
    pub const DATA: [u8; 4] = [b'D', b'A', b'T', b'A'];
}

use crate::delay::Delay;
use rppal::i2c::I2c;

mod delay;

fn main() {
    let logger_env_config = Env::default().filter_or("MOCAP_SUIT_LOG_LEVEL", "trace");
    Builder::from_env(logger_env_config).init();
    let server_bind_address =
        env::var("MOCAP_SUIT_BIND_ADDRESS").unwrap_or(String::from("127.0.0.1:7810"));

    let mut delay = Delay;
    // Use I2C1 (default for most modern Pis)
    let i2c = I2c::with_bus(1).expect("Could not grab I2C bus!");
    // Connect to the mux
    let mux0 = Xca9548a::new(i2c, SlaveAddr::default());
    let mux0_parts = mux0.split();
    // Connect to chips on the first mux
    let imu0 = Bno055::new(mux0_parts.i2c0).with_alternative_address();
    let imu1 = Bno055::new(mux0_parts.i2c1).with_alternative_address();
    // Group the chips up
    let mut imus = [imu0, imu1];

    // Let the BNO chips warm up
    thread::sleep(Duration::from_millis(500));
    // Set them up
    for (imu_index, imu) in imus.iter_mut().enumerate() {
        // Initialize it
        imu.init(&mut delay)
            .expect(format!("An error occurred while initializing BNO {}!", imu_index).as_str());
        // Now let's configure it a bit
        imu.set_mode(BNO055OperationMode::NDOF, &mut delay).expect(
            format!(
                "An error occurred while setting BNO {} to NDOF mode!",
                imu_index
            )
            .as_str(),
        );
    }

    debug!("Mocap suit bound to address {}", server_bind_address);
    let tcp_listener = TcpListener::bind(server_bind_address).unwrap();
    for incoming_stream in tcp_listener.incoming() {
        let mut stream = incoming_stream.unwrap();
        let mut input_buffer: [u8; 4] = [0; 4];
        stream
            .read(&mut input_buffer)
            .expect("Could not read from input buffer!");
        match input_buffer {
            SuitCommand::ID => {
                info!("Received ID input message.");
                stream
                    .write(&[0, 0, 0])
                    .expect("Could not write bytes to TCP buffer!");
                // stream.flush().expect("Could not flush TCP write buffer!");
            }
            SuitCommand::READY => {
                info!("Received READY input message.");
                // Send a 1 for yes, 0 would be for no
                stream
                    .write(&[1 as u8])
                    .expect("Could not write bytes to TCP buffer!");
                // stream.flush().expect("Could not flush TCP write buffer!");
            }
            SuitCommand::DATA => {
                info!("Received DATA input message.");
                let mut quaternion_slices: [[u8; 16]; 2] = [[0; 16]; 2];
                for (imu_index, imu) in imus.iter_mut().enumerate() {
                    trace!("Getting IMU index {}", imu_index);
                    match imu.quaternion() {
                        Ok(quaternion) => {
                            let w_bytes = quaternion.s.to_le_bytes();
                            let x_bytes = quaternion.v.x.to_le_bytes();
                            let y_bytes = quaternion.v.y.to_le_bytes();
                            let z_bytes = quaternion.v.z.to_le_bytes();
                            quaternion_slices[imu_index].clone_from_slice(
                                [w_bytes, x_bytes, y_bytes, z_bytes].concat().as_slice(),
                            );
                        }
                        Err(_) => {
                            error!(
                                "An error occurred while getting quaternion data for BNO {}!",
                                imu_index
                            );
                            let error_bytes: [u8; 32] = [0; 32];
                            stream
                                .write(&error_bytes)
                                .expect("Could not write bytes to TCP buffer!");
                            // stream.flush().expect("Could not flush TCP write buffer!");
                            continue;
                        }
                    }
                    if let Ok(is_calibrated) = imu.is_fully_calibrated() {
                        if is_calibrated {
                            if let Ok(calib_profile) = imu.calibration_profile(&mut delay) {
                                if let Err(details) =
                                    imu.set_calibration_profile(calib_profile, &mut delay)
                                {
                                    warn!(
                                "An error occurred while setting the IMU calibration profile for BNO {}! {:?}",
                                imu_index,
                                details
                            );
                                }
                            }
                        }
                    }
                }
                let mut quaternion_bytes: [u8; 32] = [0; 32];
                quaternion_bytes.copy_from_slice(quaternion_slices.concat().as_slice());
                stream
                    .write(&quaternion_bytes)
                    .expect("Could not write bytes to TCP buffer!");
                // stream.flush().expect("Could not flush TCP write buffer!");
            }
            _ => {
                // Ignore this connection if we can't understand it
                debug!("Unrecognized magic bytes received from client!");
            }
        }
    }
}
