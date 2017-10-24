extern crate futures;
extern crate hyper;

use futures::Future;
use hyper::{Method, StatusCode};
use hyper::server::{Http, Request, Response, Service};

struct Ping;

impl Service for Ping {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let mut response = Response::new();

        match (req.method(), req.path()) {
            (&Method::Get, "/ping") => {
                response.set_body("pong");
            }
            _ => {
                response.set_status(StatusCode::NotFound);
            }
        };

        Box::new(futures::future::ok(response))
    }
}

fn main() {
    let hostport = "127.0.0.1:3000";
    let addr = hostport.parse().unwrap();
    let server = Http::new().pipeline(true).bind(&addr, || Ok(Ping)).unwrap();

    println!("Listening on http://{}/", hostport);

    server.run().unwrap();
}
