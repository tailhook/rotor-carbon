//! The library allows to send data to carbon (also known as graphite)
//!
//! #Example
//!
//! ```ignore
//! extern crate rotor;
//! extern crate rotor_tools;
//!
//! use rotor_tools::loop_ext::LoopExt;
//!
//! let loopinst = rotor::Loop::new(&rotor::Config::new()).instantiate(());
//! let sink = loopinst.add_and_fetch(|scope| {
//!     connect_ip("127.0.0.1:2003".parse().unwrap(), scope)
//! }).unwrap();
//!
//! loopinst.run().unwrap();
//!
//! // Then somewhere else:
//!
//! { // Note sender keeps lock on buffer for it's lifetime
//!     let sender = sink.sender();
//!     sender.add_u64("some.value", 1234);
//! }
//! ```
//!
//! If you need to format the metric name, use `format_args!` instead of
//! `format!` as the former does not preallocate a buffer:
//! ```
//! snd.add_int_at(
//!     format_args!("servers.{}.metrics.{}", hostname, metric),
//!     metric_value, timestamp);
//! ```
//!
extern crate rotor;
extern crate netbuf;
extern crate void;
extern crate time;
extern crate rand;
extern crate num;

mod sink;
mod fsm;
mod sender;

use std::io;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, MutexGuard};

use rotor::{GenericScope, Notifier};

struct Internal {
    state: fsm::State,
    address: SocketAddr,
    buffer: netbuf::Buf,
    notify: Notifier,
}

/// A state machine object, just add in to the loop
pub struct Fsm<C>(Arc<Mutex<Internal>>, PhantomData<*const C>);

unsafe impl<C> Send for Fsm<C> {}

/// This is a wrapper around the machinery to send data
///
/// Use ``sink.sender()`` go to get an actual object you may send to
///
/// Note ``sink.sender()`` holds lock on the underlying buffer and doesn't
/// send data, until sender is dropped. This is useful for sending data in
/// single bulk.
#[derive(Clone)]
pub struct Sink(Arc<Mutex<Internal>>);

/// The sender object, which has convenience methods to send the data
///
/// Note ``Sender()`` holds lock on the underlying buffer and doesn't
/// send data, until sender is dropped. This is useful for sending data in
/// single bulk.
pub struct Sender<'a>(MutexGuard<'a, Internal>, bool);

/// Connect to the socket by IP address
///
/// The method is here while rotor-dns is not matured yet. The better way
/// would be to use dns resolving.
pub fn connect_ip<S: GenericScope, C>(addr: SocketAddr, scope: &mut S)
    -> Result<(Fsm<C>, Sink), io::Error>
{
    let arc = Arc::new(Mutex::new(try!(Internal::new(addr, scope))));
    Ok((Fsm(arc.clone(), PhantomData), Sink(arc)))
}

