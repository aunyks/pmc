use bno055::{BNO055OperationMode, Bno055};
use env_logger::{Builder, Env};
use log::{debug, error};
use std::thread;
use std::time::Duration;
use xca9548a::{SlaveAddr, Xca9548a};

use crate::delay::Delay;
use rppal::i2c::I2c;

mod delay;

fn main() {
    let logger_env_config = Env::default().filter_or("MOCAP_SUIT_LOG_LEVEL", "trace");
    Builder::from_env(logger_env_config).init();

    let mut delay = Delay;

    // Use I2C1 (default for most modern Pis)
    let i2c = I2c::with_bus(1).expect("Could not grab I2C bus!");

    let mux1 = Xca9548a::new(i2c, SlaveAddr::default());
    let mux1_parts = mux1.split();

    // Connect to the BNO chip on this bus
    let imu1 = Bno055::new(mux1_parts.i2c0).with_alternative_address();

    let mut imus = [imu1];

    // Let the BNO chips warm up
    thread::sleep(Duration::from_millis(500));

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
    loop {
        for (imu_index, imu) in imus.iter_mut().enumerate() {
            match imu.quaternion() {
                Ok(quaternion) => {
                    let w = quaternion.s;
                    let x = quaternion.v.x;
                    let y = quaternion.v.y;
                    let z = quaternion.v.z;

                    debug!("IMU {}\nW: {}, X: {}, Y: {}, Z: {}", imu_index, w, x, y, z);
                }
                Err(_) => {
                    error!(
                        "An error occurred while getting quaternion data for BNO {}!",
                        imu_index
                    );
                }
            }

            if let Ok(is_calibrated) = imu.is_fully_calibrated() {
                if is_calibrated {
                    if let Ok(calib_profile) = imu.calibration_profile(&mut delay) {
                        if let Err(details) = imu.set_calibration_profile(calib_profile, &mut delay)
                        {
                            error!(
                                "An error occurred while setting the IMU calibration profile for BNO {}! {:?}",
                                imu_index,
                                details
                            );
                        }
                    }
                }
            }
        }
    }
}
