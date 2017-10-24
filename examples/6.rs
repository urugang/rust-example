#[macro_use]
extern crate serde_derive;

extern crate futures;
extern crate futures_cpupool;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate tokio_minihttp;
extern crate tokio_proto;
extern crate tokio_service;

use std::io;

use futures::Future;
use futures_cpupool::CpuPool;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use rand::Rng;
use tokio_minihttp::{Request, Response};
use tokio_proto::TcpServer;
use tokio_service::Service;

#[derive(Serialize)]
struct Message {
    id: i32,
    body: String,
}

struct Server {
    thread_pool: CpuPool,
    db_pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager>,
}

impl Service for Server {
    type Request = Request;
    type Response = Response;
    type Error = io::Error;
    type Future = Box<Future<Item = Response, Error = io::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        assert_eq!(req.path(), "/db");

        let random_id = rand::thread_rng().gen_range(1, 5);
        let db = self.db_pool.clone();
        let msg = self.thread_pool.spawn_fn(move || {
            let conn = db.get().map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("timeout: {}", e))
            })?;

            let stmt = conn.prepare_cached("SELECT * FROM greetings WHERE id = $1")?;
            let rows = stmt.query(&[&random_id])?;
            let row = rows.get(0);

            Ok(Message {
                id: row.get("id"),
                body: row.get("body"),
            })
        });

        Box::new(msg.map(|msg| {
            let json = serde_json::to_string(&msg).unwrap();
            let mut response = Response::new();

            response.header("Content-Type", "application/json");
            response.body(&json);
            response
        }))
    }
}

fn main() {
    let addr = "127.0.0.1:8080".parse().unwrap();
    let thread_pool = CpuPool::new(10);

    let db_url = "postgres://postgres@localhost";
    let db_config = r2d2::Config::default();
    let db_manager = PostgresConnectionManager::new(db_url, TlsMode::None).unwrap();
    let db_pool = r2d2::Pool::new(db_config, db_manager).unwrap();

    TcpServer::new(tokio_minihttp::Http, addr).serve(move || {
        Ok(Server {
            thread_pool: thread_pool.clone(),
            db_pool: db_pool.clone(),
        })
    })
}
