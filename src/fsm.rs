use std::io;
use std::mem::replace;
use std::cmp::max;
use std::error::Error;
use std::net::SocketAddr;
use std::io::ErrorKind::{Interrupted, WouldBlock};

use rand::{thread_rng, Rng};
use time::{SteadyTime, Duration};
use netbuf::Buf;
use rotor::{Machine, Scope, Response, EventSet, GenericScope, PollOpt};
use rotor::{Timeout};
use rotor::mio::tcp::TcpStream;
use void::{Void, unreachable};

use {Fsm, Internal};

type Deadline = SteadyTime;


pub enum State {
    /// The state is used for mem::replace, can't be when event comes
    Reset,
    Sleeping(Timeout, Deadline),
    Connecting(TcpStream, Timeout, Deadline),
    Normal(TcpStream, Deadline),
}



fn try_write(sock: &mut TcpStream, buffer: &mut Buf) -> bool {
    loop {
        if buffer.len() == 0 {
            return true;
        }
        match buffer.write_to(sock) {
            Ok(0) => {
                // TODO(tailhook) should we log it
                return false;
            }
            Ok(_) => continue,
            Err(ref e) if e.kind() == Interrupted => {
                continue;
            }
            Err(ref e) if e.kind() == WouldBlock => {
                return true;
            }
            Err(_) => {
                // TODO(tailhook) should we log the error
                return false;
            }
        }
    }
}

impl Internal {
    pub fn new<S: GenericScope>(addr: SocketAddr, scope: &mut S)
        -> Result<Internal, io::Error>
    {
        let conn = try!(TcpStream::connect(&addr));
        scope.register(&conn, EventSet::writable(), PollOpt::edge()).unwrap();
        let (ms, time) = next_attempt();
        let timeo = scope.timeout_ms(ms).unwrap();
        Ok(Internal {
            state: State::Connecting(conn, timeo, time),
            address: addr,
            buffer: Buf::new(),
            notify: scope.notifier(),
        })
    }
}

fn next_attempt() -> (u64, SteadyTime) {
    let ms = thread_rng().gen_range(200, 1000);
    return (ms, SteadyTime::now() + Duration::milliseconds(ms as i64));
}

fn clean(buf: &mut Buf) {
    // Let's remove data that may be partially written
    // TODO(tailhook) Can remove only up to \n
    buf.remove_range(..);
}

fn reconnect<S: GenericScope>(min_time: SteadyTime, addr: SocketAddr,
    scope: &mut S) -> State
{
    let now = SteadyTime::now();
    if now < min_time {
        let timeout = scope.timeout_ms(
            max(0, (min_time - now).num_milliseconds()) as u64).unwrap();
        return State::Sleeping(timeout, min_time);
    } else {
        let (ms, time) = next_attempt();
        let timeout = scope.timeout_ms(ms).unwrap();
        match TcpStream::connect(&addr) {
            Ok(sock) => {
                scope.register(&sock, EventSet::writable(), PollOpt::edge())
                    .unwrap();
                return State::Connecting(sock, timeout, time);
            }
            Err(_) => {
                // TODO(tailhook) log the error?
                return State::Sleeping(timeout, time);
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
    fn ready(self, _events: EventSet, scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        {
            use self::State::*;
            let mut me = self.0.lock().unwrap();
            let me = &mut *me;
            me.state = match replace(&mut me.state, Reset) {
                state @ Sleeping(..) => state,
                Connecting(mut sock, timeo, dline) => {
                    scope.clear_timeout(timeo);
                    match sock.take_socket_error() {
                        Ok(()) => {
                            if !try_write(&mut sock, &mut me.buffer) {
                                clean(&mut me.buffer);
                                reconnect(dline, me.address, scope)
                            } else {
                                Normal(sock, dline)
                            }
                        }
                        Err(_) => {
                            // TODO(tailhook) log error?
                            reconnect(dline, me.address, scope)
                        }
                    }
                }
                Normal(mut sock, dline) => {
                    if try_write(&mut sock, &mut me.buffer) {
                        Normal(sock, dline)
                    } else {
                        clean(&mut me.buffer);
                        reconnect(dline, me.address, scope)
                    }
                }
                Reset => unreachable!(),
            }
        }
        Response::ok(self)
    }
    fn spawned(self, _scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        Response::ok(self)
    }
    fn timeout(self, scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        {
            use self::State::*;
            let mut me = self.0.lock().unwrap();
            let me = &mut *me;
            me.state = match replace(&mut me.state, Reset) {
                Sleeping(timeo, dline) => {
                    if SteadyTime::now() < dline {
                        Sleeping(timeo, dline)
                    } else {
                        reconnect(dline, me.address, scope)
                    }
                }
                Connecting(sock, timeo, dline) => {
                    if SteadyTime::now() < dline {
                        Connecting(sock, timeo, dline)
                    } else {
                        reconnect(dline, me.address, scope)
                    }
                }
                Reset => unreachable!(),
                x => x,
            }
        }
        Response::ok(self)
    }
    fn wakeup(self, scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        {
            use self::State::*;
            let mut me = self.0.lock().unwrap();
            let me = &mut *me;
            me.state = match replace(&mut me.state, Reset) {
                Normal(mut sock, dline) => {
                    if try_write(&mut sock, &mut me.buffer) {
                        Normal(sock, dline)
                    } else {
                        clean(&mut me.buffer);
                        reconnect(dline, me.address, scope)
                    }
                }
                Reset => unreachable!(),
                x => x,
            }
        }
        Response::ok(self)
    }
}
