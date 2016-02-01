use std::error::Error;

use rotor::{Machine, Scope, Response, EventSet};
use void::{Void, unreachable};

use {Fsm, Internal};

fn try_write(fsm: &mut Internal) {
    loop {
        match fsm.buffer.write_to(&mut fsm.socket) {
            Ok(0) => return,
            Ok(_) => continue,
            Err(e) => {
                // TODO(tailhook) process error, reconnect, etc
                panic!("Disconnected from carbon: {}", e);
            }
        }
    }
}

impl<C> Machine for Fsm<C> {
    type Context = C;
    type Seed = Void;
    fn create(seed: Self::Seed, _scope: &mut Scope<Self::Context>)
        -> Result<Self, Box<Error>>
    {
        unreachable(seed);
    }
    fn ready(self, _events: EventSet, _scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        try_write(&mut self.0.lock().unwrap());
        Response::ok(self)
    }
    fn spawned(self, _scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        Response::ok(self)
    }
    fn timeout(self, _scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        Response::ok(self)
    }
    fn wakeup(self, _scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        try_write(&mut self.0.lock().unwrap());
        Response::ok(self)
    }
}
