extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
extern crate net2;
extern crate rand;
extern crate threadpool;
extern crate tokio_core;

use net2::unix::UnixTcpBuilderExt;
use std::env;
use futures::future::FutureResult;
use tokio_core::net::TcpListener;
use futures::future::Future;
use futures::future;
use futures::stream::Stream;
use futures::sync::oneshot;
use futures_cpupool::CpuPool;
use hyper::Chunk;
use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};
use rand::ThreadRng;
use self::rand::Rng;
use std::ascii::AsciiExt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Error as IoError;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use tokio_core::reactor::Core;

struct HelloWorld {
    tx: Arc<Mutex<Sender<(oneshot::Sender<Chunk>, Chunk)>>>,
    rng: RefCell<ThreadRng>,
}

const PHRASE: &'static str = "Hello, World!";

impl Service for HelloWorld {
    type Request = Request;
    type Response = Response<Box<Stream<Item = Chunk, Error = Self::Error>>>;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;
    fn call(&self, req: Request) -> Self::Future {
        let mut response = Response::new();
        let tx = self.tx.clone();

        let mapping = req.body().and_then(move |chunk| {
            let (tx0, rx0) = oneshot::channel();
            // operation before sending request 
            tx.lock().unwrap().send((tx0, chunk)).unwrap();
            rx0.map_err(|e| hyper::Error::Timeout)
            // operation after receiving response
        });

        let body: Box<Stream<Item = _, Error = _>> = Box::new(mapping.into_inner());
        response.set_body(body);
        Box::new(future::ok(response))
    }
}


fn serve(
    addr: &SocketAddr,
    protocol: &Http,
    tx: Arc<Mutex<Sender<(oneshot::Sender<Chunk>, Chunk)>>>,
) {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let listener = net2::TcpBuilder::new_v4()
        .unwrap()
        .reuse_port(true)
        .unwrap()
        .bind(addr)
        .unwrap()
        .listen(128)
        .unwrap();
    let listener = TcpListener::from_listener(listener, addr, &handle).unwrap();



    core.run(
        listener
            .incoming()
            .for_each(|(socket, addr)| {
                protocol.bind_connection(
                    &handle,
                    socket,
                    addr,
                    HelloWorld {
                        tx: tx.clone(),
                        rng: RefCell::new(rand::thread_rng()),
                    },
                );
                Ok(())
            })
            .or_else(|e| -> FutureResult<(), ()> {
                panic!("TCP listener failed: {}", e);
            }),
    ).unwrap();
}

fn start_server(nb_instances: usize, addr: &str) {
    let addr = addr.parse().unwrap();

    let api_num: usize = std::env::args().nth(1).unwrap().parse().unwrap();
    let mq_num: usize = std::env::args().nth(2).unwrap().parse().unwrap();
    let (tx, rx) = channel();
    let tx = Arc::new(Mutex::new(tx));
    let threadpool = threadpool::ThreadPool::new(mq_num);
    thread::spawn(move || loop {
        let (tx0,d chunk): (oneshot::Sender<Chunk>, Chunk) = rx.recv().unwrap();
        threadpool.clone().execute(move || {
            thread::sleep_ms(5);
            tx0.send(chunk);
        });
    });
    let protocol = Arc::new(Http::new());
    {
        for _ in 0..nb_instances - 1 {
            let protocol = protocol.clone();
            let tx0 = tx.clone();
            thread::spawn(move || serve(&addr, &protocol, tx0));
        }
    }
    serve(&addr, &protocol, tx.clone());
}


fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        panic!("Please state the number of threads to start");
    }
    let n = usize::from_str_radix(&args[1], 10).unwrap();

    start_server(n, "0.0.0.0:1337");
}
