extern crate futures;
extern crate tokio_core;
extern crate tokio_io;

use tokio_io::AsyncWrite;
use std::time::Instant;
use std::thread;
use futures::{Future, Stream};
use tokio_io::AsyncRead;
use tokio_io::io::copy;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;

fn main() {
    // Create the event loop that will drive this server
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // Bind the server's socket
    let addr = "127.0.0.1:12345".parse().unwrap();
    let listener = TcpListener::bind(&addr, &handle).unwrap();
    let now = Instant::now().clone();
    // Pull out a stream of sockets for incoming connections
    let server = listener.incoming().for_each(|(sock, _)| {
        // Split up the reading and writing parts of the
        // socket
        let (reader, writer) = sock.split();

        println!("accept: {:?}", now.elapsed());
        // A future that echos the data and returns how
        // many bytes were copied...
        let bytes_copied = copy(reader, writer);
        // ... after which we'll print what happened
        let mut str = 3;
        let handle_conn = bytes_copied
            .map(move |amt| {
                println!("before: {:?}", now.elapsed());
                thread::sleep_ms(3_000);
                println!("after: {:?}", now.elapsed());

            })
            .map_err(|err| println!("IO error {:?}", err));
        // Spawn the future as a concurrent task
        handle.spawn(handle_conn);
        Ok(())
    });

    // Spin up the server on the event loop

    core.run(server).unwrap();
}
