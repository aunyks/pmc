# mocap-suit

An open source motion capture suit using a Raspberry Pi, TI a TCA9548A I2C Multiplexer, and some Bosch BNO055 IMUs.

## Building

If you're not compiling this code on a Raspberry Pi, install the following compilation target. If you are, remove the `.cargo/` directory:

```
rustup target install armv7-unknown-linux-gnueabihf
```

To build this project, simply run `cargo build` (or `cargo build --release`, for optimized release builds).

To run the project after a build, run `./target/release/mocap-suit`.

To do the build and run step all at once, run `cargo run` (or `cargo run --release`, for optimized release builds).

## Pi Setup

This project will run on any recent (as of Feb 26, 2022) Raspberry Pi OS version.

This project relies on your Pi's I2C port (using GPIO pins 2 [SDA] and 3 [SCL]). To configure your Pi to enable the I2C port for this project, follow these steps:

1. Run `sudo raspi-config` from the terminal
2. Select "Interfacing Options"
3. Select "I2C"
4. Select "Yes"
5. Select "Ok"
6. If prompted to reboot the Pi, select "Yes"

These steps can be found with screenshots [here](https://www.raspberrypi-spy.co.uk/2014/11/enabling-the-i2c-interface-on-the-raspberry-pi/).

Copyright (C) 2022 Gerald "aunyks" Nash
