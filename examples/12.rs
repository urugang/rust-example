/*
$ cat tests/jsonrpc_performance/config_err_format.json 
{
    "ipandport": [
        "127.0.0.1:1337"
    ],
    "txnum": 1000,
    "threads": 20,
    "code":"60606040523415600e57600080fd5b5b5b5b60948061001f6000396000f30060606040526000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff1680635524107714603d575b600080fd5b3415604757600080fd5b605b6004808035906020019091905050605d565b005b806000819055505b505600a165627a7a72305820c471b4376626da2540b2374e8b4110501051c426ff46814a6170ce9e219e49a80029",
    "contract_address": "",
    "quota": 1000,
    "tx_type": "Correct",
    "tx_format_err": true
}
$ taskset -c 0-3 cargo run --example 1 --release 
$ taskset -c 4-7 target/release/jsonrpc_performance --config tests/jsonrpc_performance/config_err_format.json 
*/
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
    tx: Sender<u64>,
    resp: Arc<RwLock<HashMap<u64, u64>>>,
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
        let sn = self.rng.borrow_mut().next_u64();
        let tx = self.tx.clone();
        let resp = self.resp.clone();

        Box::new(self.thread_pool.spawn_fn(move || {
            tx.send(sn).unwrap();
            loop {
                thread::sleep_ms(1);
                if resp.read().unwrap().contains_key(&sn) {
                    resp.write().unwrap().remove(&sn);
                    break;
                }
            }
            future::ok(
                Response::new()
                    .with_header(ContentLength(PHRASE.len() as u64))
                    .with_body(PHRASE),
            )
        }))
    }
}

fn main() {
    let api_num: usize = std::env::args().nth(1).unwrap().parse().unwrap();
    let mq_num: usize = std::env::args().nth(2).unwrap().parse().unwrap();
    let thread_pool = CpuPool::new(api_num);
    let resp = Arc::new(RwLock::new(HashMap::new()));
    let resp_mq = resp.clone();
    let (tx, rx) = channel();
    let threadpool = threadpool::ThreadPool::new(mq_num);
    thread::spawn(move || loop {
        let sn = rx.recv().unwrap();
        let pool = threadpool.clone();
        let resp2 = resp_mq.clone();
        pool.clone().execute(move || {
            thread::sleep_ms(5);
            {
                resp2.write().unwrap().insert(sn, 0);
            }
        });
    });

    let addr = "127.0.0.1:1337".parse().unwrap();
    let server = Http::new()
        .bind(&addr, move || {
            Ok(HelloWorld {
                tx: tx.clone(),
                rng: RefCell::new(rand::thread_rng()),
                resp: resp.clone(),
                thread_pool: thread_pool.clone(),
            })
        })
        .unwrap();
    server.run().unwrap();
}
