use std::{
    io::{BufRead, BufReader, Read},
    net::TcpStream,
};

use http::{Request, Response, StatusCode};
use todo::Server;

fn main() {
    let mut server = Server::new();
    server.get("/", handle_request);

    server.serve();
}

fn handle_request(req: Request<BufReader<&mut TcpStream>>) -> Response<String> {
    let mut buf: String = Default::default();
    let mut body = req.body().to_owned();
    let body = body.read_line(&mut buf).unwrap();
    println!("{body}");
    Response::builder()
        .status(StatusCode::OK)
        .body("Hello, World!".to_string())
        .unwrap()
}
