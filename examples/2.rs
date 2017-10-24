#![feature(core_intrinsics)]

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            extern crate core;
            unsafe { core::intrinsics::type_name::<T>() }
        }
        let name = type_name_of(f);
        &name[6..name.len() - 4]
    }}
}

extern crate futures;
extern crate tokio_core;
extern crate tokio_io;

use std::thread;
use futures::future::{ok, loop_fn, Future, FutureResult, Loop};
use std::io::Error;
use futures::Async;
#[derive(Debug)]
struct Client {
    ping_count: u8,
}

impl Client {
    fn new() -> Self {
        Client { ping_count: 0 }
    }

    fn send_ping(self) -> FutureResult<Self, Error> {
        println!("{}", function!());
        ok(Client { ping_count: self.ping_count + 1 })
    }

    fn receive_pong(self) -> FutureResult<(Self, bool), Error> {
        println!("{}", function!());
        let done = self.ping_count >= 5;
        ok((self, done))
    }
}


fn main() {
    let mut ping_til_done = loop_fn(Client::new(), |client| {
        println!("loop_fn");
        client
            .send_ping()
            .and_then(|client| client.receive_pong())
            .and_then(|(client, done)| {
                println!("done");
                if done {
                    Ok(Loop::Break(client))
                } else {
                    Ok(Loop::Continue(client))
                }
            })
    });

    match ping_til_done.poll() {
        Ok(Async::Ready(t)) => {
            println!("t: {:?}", t);
        }
        Ok(Async::NotReady) => print!("."),
        Err(e) => print!(".{}", e),
    }
}
