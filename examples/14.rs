extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
extern crate rand;
extern crate threadpool;

use std::ascii::AsciiExt;
use hyper::Chunk;
use futures::stream::Stream;
use futures::sync::oneshot;
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
    tx: Sender<(oneshot::Sender<Vec<u8>>, Vec<u8>)>,
    thread_pool: CpuPool,
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
        let mapping = req.body().map(move |chunk| {
            let uppered = chunk
                .iter()
                .map(|byte| byte.to_ascii_uppercase())
                .collect::<Vec<u8>>();
            let (tx0, rx0) = oneshot::channel();
            tx.send((tx0, uppered)).unwrap();
            rx0.map(|v| Chunk::from(v)).wait().unwrap()
        });

        let body: Box<Stream<Item = _, Error = _>> = Box::new(mapping);
        response.set_body(body);
        Box::new(future::ok(response))
    }
}

fn main() {
    let api_num: usize = std::env::args().nth(1).unwrap().parse().unwrap();
    let mq_num: usize = std::env::args().nth(2).unwrap().parse().unwrap();
    let thread_pool = CpuPool::new(api_num);
    let (tx, rx) = channel();
    let threadpool = threadpool::ThreadPool::new(mq_num);
    thread::spawn(move || loop {
        let (tx0, mut v): (oneshot::Sender<Vec<u8>>, Vec<u8>) = rx.recv().unwrap();
        {
            threadpool.clone().execute(move || {
                thread::sleep_ms(5_000);
                v.push(49u8);
                v.push(50u8);
                //println!("==={:?}", v);
                tx0.send(v);
            });
        }
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
