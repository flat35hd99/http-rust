#![deny(warnings)]

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;

use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{body::Incoming as IncomingBody, Request, Response};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(stream, Svc {}).await {
                println!("Failed to serve connection: {:?}", err);
            }
        });
    }
}

struct Svc {}

impl Service<Request<IncomingBody>> for Svc {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&mut self, req: Request<IncomingBody>) -> Self::Future {
        fn mk_response(s: String) -> Result<Response<Full<Bytes>>, hyper::Error> {
            Ok(Response::builder().body(Full::new(Bytes::from(s))).unwrap())
        }

        let res = match req.uri().path() {
            "/" => mk_response(format!("home")),
            "/posts" => mk_response(format!("posts")),
            "/authors" => mk_response(format!("authors")),
            _ => return Box::pin(async { mk_response("not found".into()) }),
        };

        Box::pin(async { res })
    }
}
