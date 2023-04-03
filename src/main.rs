use todo::{Server};
use http::{Request, Response, StatusCode};

fn main() {
    let mut server = Server::new();
    server.GET("/".to_string(), handle_request);

    server.serve();
}

fn handle_request(_req: Request<()>) -> Response<String> {
    // let res = Response::builder().status(StatusCode::OK);
    // res.body("Hello, World!\n".to_string())
    Response::builder().status(StatusCode::OK).body("hgoe".to_string()).unwrap()
}
