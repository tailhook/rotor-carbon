use std::io::Write;
use std::fmt::Display;

use time::get_time;

use {Sender};

impl<'a> Sender<'a> {
    /// Send a generic convertible to i64
    pub fn add_int<V, I=i64>(&mut self, name: V, value: I)
        where V: Display, I: Into<i64>
    {
        let ts = get_time().sec;
        let val = value.into();
        writeln!(self.0.buffer, "{} {} {}", name, val, ts).unwrap();
    }
    /// The special case for u64, as it's the only type doesn't fit in i64
    pub fn add_u64<V>(&mut self, name: V, value: u64)
        where V: Display
    {
        let ts = get_time().sec;
        writeln!(self.0.buffer, "{} {} {}", name, value, ts).unwrap();
    }
    /// Send a generic convertible to i64 with a timestamp
    pub fn add_int_at<V, I=i64>(&mut self, name: V, value: I, ts: u64)
        where V: Display, I: Into<i64>
    {
        let val = value.into();
        writeln!(self.0.buffer, "{} {} {}", name, val, ts).unwrap();
    }
    /// Send an u64 with a timestamp
    pub fn add_u64_at<V>(&mut self, name: V, value: u64, ts: u64)
        where V: Display
    {
        writeln!(self.0.buffer, "{} {} {}", name, value, ts).unwrap();
    }
}

impl<'a> Drop for Sender<'a> {
    fn drop(&mut self) {
        if self.1 {
            self.0.notify.wakeup().unwrap()
        }
    }
}
