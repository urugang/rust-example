extern crate hyper;
extern crate futures;

use std::thread;
use std::ascii::AsciiExt;
use futures::future::Future;
use futures::stream::Stream;
use hyper::{Method, StatusCode};
use hyper::header::ContentLength;
use hyper::Body;
use hyper::server::{Http, Request, Response, Service};
use futures::future;
use hyper::Chunk;
use std::boxed::Box;
const PHRASE: &'static str = "Hello, World!";
struct HelloWorld;
impl Service for HelloWorld {
    // boilerplate hooking up hyper's server types
    type Request = Request;
    type Response = Response<Box<Stream<Item = Chunk, Error = Self::Error>>>;
    type Error = hyper::Error;
    // The future representing the eventual Response your call will
    // resolve to. This can change to whatever Future you need.
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let mut response = Response::new();
        //let mapping = req.body().map(to_uppercase as fn(Chunk) -> Chunk);
        let mapping = req.body().map(|chunk| {
            let uppered = chunk
                .iter()
                .map(|byte| byte.to_ascii_uppercase())
                .collect::<Vec<u8>>();
            println!("chunk: {:?}", chunk);
            thread::sleep_ms(1_000);
            Chunk::from(uppered)
        });
        let body: Box<Stream<Item = _, Error = _>> = Box::new(mapping);
        response.set_body(body);
        Box::new(future::ok(response))
    }
}

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let server = Http::new().bind(&addr, || Ok(HelloWorld)).unwrap();
    server.run().unwrap();
}
fn to_uppercase(chunk: Chunk) -> Chunk {
    let uppered = chunk
        .iter()
        .map(|byte| byte.to_ascii_uppercase())
        .collect::<Vec<u8>>();
    println!("chunk: {:?}", chunk);
    thread::sleep_ms(1_000_000);
    Chunk::from(uppered)
}
