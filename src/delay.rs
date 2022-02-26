use std::thread;
use std::time::Duration;

pub struct Delay;

impl embedded_hal::blocking::delay::DelayMs<u16> for Delay {
    fn delay_ms(&mut self, ms: u16) {
        thread::sleep(Duration::from_millis(u64::from(ms)));
    }
}
