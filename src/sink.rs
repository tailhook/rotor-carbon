use {Sink, Sender};

impl Sink {
    /// Get the `Sender` object to send data
    ///
    /// Rules of thumb:
    ///
    /// 1. Hold it only for sending data, not for fetching actual metrics
    /// 2. If you poll for metrics in the loop, drop the sender while doing
    ///    `sleep()`
    pub fn sender(&self) -> Sender {
        let data = self.0.lock().unwrap();
        let notify = data.buffer.len() == 0;
        Sender(data, notify)
    }
}
