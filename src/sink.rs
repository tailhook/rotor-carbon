use std::fmt::Debug;

use rotor_stream::ActiveStream;

use {Sink, Sender};


impl<C, S: ActiveStream> Sink<C, S>
    where S::Address: Debug
{
    /// Get the `Sender` object to send data
    ///
    /// Rules of thumb:
    ///
    /// 1. Hold it only for sending data, not for fetching actual metrics
    /// 2. If you poll for metrics in the loop, drop the sender while doing
    ///    `sleep()`
    pub fn sender(&self) -> Sender<C, S> {
        let mut data = self.0.lock().unwrap();
        let len = data.transport().map(|mut x| x.output().len()).unwrap_or(0);
        let notify = if len == 0 { Some(self.1.clone()) } else { None };
        Sender(data, notify)
    }
}
