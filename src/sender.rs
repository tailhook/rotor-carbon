use std::io::Write;
use time::get_time;

use {Sender};

impl<'a> Sender<'a> {
    pub fn add_u64(&mut self, name: &str, value: u64) {
        let ts = get_time().sec;
        writeln!(self.0.buffer, "{} {} {}", name, value, ts).unwrap();
    }
    pub fn add_int<I: Into<i64>=i64>(&mut self, name: &str, value: I) {
        let ts = get_time().sec;
        let val = value.into();
        writeln!(self.0.buffer, "{} {} {}", name, val, ts).unwrap();
    }
}

impl<'a> Drop for Sender<'a> {
    fn drop(&mut self) {
        if self.1 {
            self.0.notify.wakeup().unwrap()
        }
    }
}
