extern crate bytes;
extern crate futures;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

use std::io;
use std::str;
use bytes::BytesMut;
use tokio_io::codec::{Encoder, Decoder};
use tokio_proto::multiplex::ServerProto;
use tokio_io::{AsyncRead, AsyncWrite};
use bytes::IntoBuf;
use bytes::Buf;
use bytes::BufMut;
use tokio_io::codec::Framed;
use tokio_service::Service;
use tokio_proto::TcpServer;
use futures::future::{self, Future};
use tokio_proto::multiplex::RequestId;
use bytes::BigEndian;
type MyMultiplexedFrame<T> = (RequestId, T);

pub struct LineCodec;
impl Decoder for LineCodec {
    type Item = (RequestId, String);
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<(RequestId, String)>> {
        // At least 5 bytes are required for a frame: 4 byte
        // head + one byte '\n'
        if buf.len() < 5 {
            // We don't yet have a full message
            return Ok(None);
        }

        // Check to see if the frame contains a new line, skipping
        // the first 4 bytes which is the request ID
        let newline = buf[4..].iter().position(|b| *b == b'\n');
        if let Some(n) = newline {
            // remove the serialized frame from the buffer.
            let mut line = buf.split_to(n + 4);
            println!("line: {:?}", line);
            // Also remove the '\n'
            buf.split_to(1);
            println!("buf: {:?}", buf);
            // Deserialize the request ID
            let id = line.split_to(4).into_buf().get_u32::<BigEndian>();

            // Turn this data into a UTF string and return it in a Frame.
            return match str::from_utf8(&line[..]) {
                Ok(s) => {
                    println!("decode: {} - {}", id, s.to_string());
                    Ok(Some((id as RequestId, s.to_string())))
                }
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "invalid string")),
            };
        }

        // No `\n` found, so we don't have a complete message
        Ok(None)
    }
}
impl Encoder for LineCodec {
    type Item = (RequestId, String);
    type Error = io::Error;

    fn encode(&mut self, msg: (RequestId, String), buf: &mut BytesMut) -> io::Result<()> {
        let (id, msg) = msg;
        println!("encode: {} - {}", id, msg);
        buf.put_u32::<BigEndian>(id as u32);

        //buf.put(format!("{} - {}", id, msg));
        buf.put(msg.as_bytes());
        buf.put("\n");

        Ok(())
    }
}

struct LineProto;
impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for LineProto {
    type Request = String;
    type Response = String;

    // `Framed<T, LineCodec>` is the return value
    // of `io.framed(LineCodec)`
    type Transport = Framed<T, LineCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(LineCodec))
    }
}

struct Echo;
impl Service for Echo {
    // These types must match the corresponding protocol types:
    type Request = String;
    type Response = String;

    // For non-streaming protocols, service errors are always io::Error
    type Error = io::Error;

    // The future for computing the response; box it for simplicity.
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    // Produce a future for computing a response from a request.
    fn call(&self, req: Self::Request) -> Self::Future {
        // In this case, the response is immediate.
        Box::new(future::ok(req))
    }
}

fn main() {
    // Specify the localhost address
    let addr = "0.0.0.0:12345".parse().unwrap();

    // The builder requires a protocol and an address
    let server = TcpServer::new(LineProto, addr);

    // We provide a way to *instantiate* the service for each new
    // connection; here, we just immediately return a new instance.
    server.serve(|| {
        println!("serve");
        Ok(Echo)
    });
}
