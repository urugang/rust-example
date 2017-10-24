extern crate futures;
extern crate tokio_core;
extern crate tokio_io;

use futures::stream::Stream;
use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;

fn main() {
    let mut core = Core::new().unwrap();
    let address = "0.0.0.0:12345".parse().unwrap();
    let listener = TcpListener::bind(&address, &core.handle()).unwrap();

    let connections = listener.incoming();
    let welcomes = connections.and_then(|(socket, _peer_addr)| {
        println!("xxxxxxxxxx");
        tokio_io::io::write_all(socket, b"Hello, world!\n")
    });
    let server = welcomes.for_each(|(_socket, _welcome)| Ok(()));

    core.run(server).unwrap();
}
