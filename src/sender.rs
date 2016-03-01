use std::io::Write;
use std::fmt::Display;

use time::get_time;
use num::Num;
use rotor_stream::ActiveStream;

use {Sender};

impl<'a, C: 'a, S: ActiveStream> Sender<'a, C, S> {
    /// Send a generic number with current timestamp
    pub fn add_value<N, V=i64>(&mut self, name: N, value: V)
        where N: Display, V: Num + Display
    {
        // Unfortunately we skip everything if connection is not established
        // we may consider to buffer things in future
        self.0.transport().map(|mut trans| {
            let ts = get_time().sec;
            let mut buf = trans.output();
            let offset = buf.len();
            writeln!(buf, "{} {} {}", name, value, ts).unwrap();
            debug_assert!(!buf[offset..buf.len()-1].contains(&b'\n'));
        });
    }
    /// Send a generic number with specific timestamp
    pub fn add_value_at<N, V=i64>(&mut self, name: N, value: V, ts: u64)
        where N: Display, V: Num + Display
    {
        // Unfortunately we skip everything if connection is not established
        // we may consider to buffer things in future
        self.0.transport().map(|mut trans| {
            let mut buf = trans.output();
            let offset = buf.len();
            writeln!(buf, "{} {} {}", name, value, ts).unwrap();
            debug_assert!(!buf[offset..buf.len()-1].contains(&b'\n'));
        });
    }
}

impl<'a, C, S: ActiveStream> Drop for Sender<'a, C, S> {
    fn drop(&mut self) {
        self.1.as_mut().map(|x| x.wakeup().unwrap());
    }
}
