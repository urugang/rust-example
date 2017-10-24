extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
extern crate rand;
extern crate threadpool;

use futures::future;
use futures::future::Future;
use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};
use std::thread;
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use self::rand::Rng;
use futures_cpupool::CpuPool;
use rand::ThreadRng;
use std::cell::RefCell;
struct HelloWorld {
    tx: Sender<Sender<u64>>,
    thread_pool: CpuPool,
    rng: RefCell<ThreadRng>,
}

const PHRASE: &'static str = "Hello, World!";

impl Service for HelloWorld {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;
    fn call(&self, _req: Request) -> Self::Future {
        let tx = self.tx.clone();
        let sn = self.rng.borrow_mut().next_u64();

        Box::new(future::lazy(move || {
            let (tx0, rx0) = channel();
            tx.send(tx0).unwrap();
            rx0.recv().unwrap();
            future::ok(
                Response::new()
                    .with_header(ContentLength(PHRASE.len() as u64))
                    .with_body(PHRASE),
            )
        }))
    }
}

fn main() {
    let thread_pool = CpuPool::new(1);
    let (tx, rx) = channel();
    let threadpool = threadpool::ThreadPool::new(4);
    thread::spawn(move || loop {
        let tx0: Sender<u64> = rx.recv().unwrap();
        threadpool.clone().execute(move || {
            thread::sleep_ms(5);
            tx0.send(1u64);
        });
    });

    let addr = "127.0.0.1:1337".parse().unwrap();
    let server = Http::new()
        .bind(&addr, move || {
            Ok(HelloWorld {
                tx: tx.clone(),
                rng: RefCell::new(rand::thread_rng()),
                thread_pool: thread_pool.clone(),
            })
        })
        .unwrap();
    server.run().unwrap();
}
