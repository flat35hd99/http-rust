use todo::{Server};
use http::{Request, Response, StatusCode};

fn main() {

    let mut server = Server::new();
    server.GET("/".to_string(), handle_request);

    server.serve();
}

fn handle_request(_req: Request<()>) -> http::Result<Response<()>, > {
    let res = Response::builder().status(StatusCode::OK);
    res.body(())
}
