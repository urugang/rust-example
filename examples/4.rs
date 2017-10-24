extern crate futures;
extern crate futures_cpupool;
extern crate hyper;

use futures::{Async, Future, Poll};
use futures_cpupool::{CpuFuture, CpuPool};
use hyper::{Method, StatusCode};
use hyper::server::{Http, Request, Response, Service};

struct Ping {
    pool: CpuPool,
}

impl Ping {
    fn new() -> Self {
        Self { pool: CpuPool::new_num_cpus() }
    }
}

impl Service for Ping {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<CpuFuture<Self::Response, Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        Box::new(self.pool.spawn(PingResponse::new(req)))
    }
}

struct PingResponse {
    req: Request,
}

impl PingResponse {
    fn new(req: Request) -> Self {
        Self { req }
    }

    fn worker(&self) -> Poll<Response, hyper::Error> {
        let mut response = Response::new();

        match (self.req.method(), self.req.path()) {
            (&Method::Get, "/ping") => {
                response.set_body("pong");
            }
            _ => {
                response.set_status(StatusCode::NotFound);
            }
        };

        Ok(Async::Ready(response))
    }
}

impl Future for PingResponse {
    type Item = Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.worker()
    }
}

fn main() {
    let hostport = "127.0.0.1:3000";
    let addr = hostport.parse().unwrap();
    let server = Http::new()
        .pipeline(true)
        .bind(&addr, || Ok(Ping::new()))
        .unwrap();

    println!("Listening on {}", hostport);

    server.run().unwrap();
}
