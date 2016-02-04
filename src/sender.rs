use std::io::Write;
use std::fmt::Display;

use time::get_time;
use num::Num;

use {Sender};

impl<'a> Sender<'a> {
    /// Send a generic number with current timestamp
    pub fn add_value<N, V=i64>(&mut self, name: N, value: V)
        where N: Display, V: Num + Display
    {
        let ts = get_time().sec;
        let offset = self.0.buffer.len();
        writeln!(self.0.buffer, "{} {} {}", name, value, ts).unwrap();
        debug_assert!(!self.0.buffer[offset..self.0.buffer.len()-1]
                       .contains(&b'\n'));
    }
    /// Send a generic number with specific timestamp
    pub fn add_value_at<N, V=i64>(&mut self, name: N, value: V, ts: u64)
        where N: Display, V: Num + Display
    {
        let offset = self.0.buffer.len();
        writeln!(self.0.buffer, "{} {} {}", name, value, ts).unwrap();
        debug_assert!(!self.0.buffer[offset..self.0.buffer.len()-1]
                       .contains(&b'\n'));
    }
}

impl<'a> Drop for Sender<'a> {
    fn drop(&mut self) {
        if self.1 {
            self.0.notify.wakeup().unwrap()
        }
    }
}
