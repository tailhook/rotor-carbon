use std::marker::PhantomData;
use std::time::Duration;
use std::error::Error;

use rotor::Scope;
use rotor_stream::{ActiveStream, Protocol, Intent, Transport, Exception};


const IDLE_TIMEOUT: u64 = 86_400;
const FLUSH_TIMEOUT: u64 = 30;


pub struct CarbonProto<C, S: ActiveStream>(PhantomData<*const (C, S)>);

unsafe impl<C, S:ActiveStream> Send for CarbonProto<C, S> {}

impl<C, S: ActiveStream> Protocol for CarbonProto<C, S> {
    type Context = C;
    type Socket = S;
    type Seed = ();
    fn create(_seed: (), _sock: &mut S, scope: &mut Scope<Self::Context>)
        -> Intent<Self>
    {
        Intent::of(CarbonProto(PhantomData)).expect_flush()
        .deadline(scope.now() + Duration::new(IDLE_TIMEOUT, 0))
    }
    fn bytes_read(self, _transport: &mut Transport<S>, _end: usize,
        _scope: &mut Scope<Self::Context>)
        -> Intent<Self>
    {
        unreachable!();
    }
    fn bytes_flushed(self, _transport: &mut Transport<S>,
        scope: &mut Scope<Self::Context>)
        -> Intent<Self>
    {
        Intent::of(CarbonProto(PhantomData)).sleep()
        .deadline(scope.now() + Duration::new(IDLE_TIMEOUT, 0))
    }
    fn timeout(self, _transport: &mut Transport<S>,
        _scope: &mut Scope<Self::Context>)
        -> Intent<Self>
    {
        Intent::done()
    }
    fn wakeup(self, transport: &mut Transport<Self::Socket>,
        scope: &mut Scope<Self::Context>)
        -> Intent<Self>
    {
        if transport.output().len() > 0 {
            Intent::of(CarbonProto(PhantomData)).expect_flush()
            .deadline(scope.now() + Duration::new(FLUSH_TIMEOUT, 0))
        } else {
            Intent::of(CarbonProto(PhantomData)).sleep()
            .deadline(scope.now() + Duration::new(IDLE_TIMEOUT, 0))
        }
    }
    fn exception(self, _transport: &mut Transport<Self::Socket>,
        reason: Exception, _scope: &mut Scope<Self::Context>)
        -> Intent<Self>
    {
        info!("Connection error: {}", reason);
        Intent::done()
    }
    fn fatal(self, reason: Exception, _scope: &mut Scope<Self::Context>)
        -> Option<Box<Error>>
    {
        info!("Connection error: {}", reason);
        None
    }
}
