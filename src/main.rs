use http::{Request, Response, StatusCode};
use todo::Server;

fn main() {
    let mut server = Server::new();
    server.get("/".to_string(), handle_request);

    server.serve();
}

fn handle_request(_req: Request<()>) -> Response<String> {
    Response::builder()
        .status(StatusCode::OK)
        .body("Hello, World!".to_string())
        .unwrap()
}
