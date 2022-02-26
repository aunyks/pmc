use bno055::{BNO055OperationMode, Bno055};
use env_logger::{Builder, Env};
use log::{debug, error};
use std::thread;
use std::time::Duration;

use crate::delay::Delay;
use rppal::i2c::I2c;

mod delay;

fn main() {
    let logger_env_config = Env::default().filter_or("MOCAP_SUIT_LOG_LEVEL", "trace");
    Builder::from_env(logger_env_config).init();

    let mut delay = Delay;

    // Use I2C1 (default for most modern Pis)
    let i2c = I2c::with_bus(1).expect("Could not grab I2C bus!");

    // Connect to the BNO chip on this bus
    let mut imu = Bno055::new(i2c).with_alternative_address();
    // Let the BNO chip warm up
    thread::sleep(Duration::from_millis(500));
    // Initialize it
    imu.init(&mut delay)
        .expect("An error occurred while initializing the BNO");
    // Now let's configure it a bit
    imu.set_mode(BNO055OperationMode::NDOF, &mut delay)
        .expect("An error occurred while setting to NDOF mode");
    loop {
        match imu.quaternion() {
            Ok(quaternion) => {
                let w = quaternion.s;
                let x = quaternion.v.x;
                let y = quaternion.v.y;
                let z = quaternion.v.z;

                debug!("W: {}, X: {}, Y: {}, Z: {}", w, x, y, z);
            }
            Err(_) => {
                error!("An error occurred while getting quaternion data!");
            }
        }

        if let Ok(is_calibrated) = imu.is_fully_calibrated() {
            if is_calibrated {
                if let Ok(calib_profile) = imu.calibration_profile(&mut delay) {
                    if let Err(details) = imu.set_calibration_profile(calib_profile, &mut delay) {
                        error!(
                            "An error occurred while setting the IMU calibration profile! {:?}",
                            details
                        );
                    }
                }
            }
        }
    }
}
