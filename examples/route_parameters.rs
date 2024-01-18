use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn;
// Import the routerify prelude traits.
use routerify::{prelude::*, Body, RequestServiceBuilder, Router};
use std::{io, net::SocketAddr};
use tokio::net::TcpListener;

// A handler for "/" page.
async fn home_handler(_: Request<crate::Body>) -> Result<Response<Body>, io::Error> {
    Ok(Response::new(Body::from("Home page")))
}

// Define a handler for "/users/:userName/books/:bookName" page which will have two
// route parameters: `userName` and `bookName`.
async fn user_book_handler(req: Request<crate::Body>) -> Result<Response<Body>, io::Error> {
    let user_name = req.param("userName").unwrap();
    let book_name = req.param("bookName").unwrap();

    Ok(Response::new(Body::from(format!(
        "User: {}, Book: {}",
        user_name, book_name
    ))))
}

fn router() -> Router<Body, io::Error> {
    // Create a router and specify the the handlers.
    Router::builder()
        .get("/", home_handler)
        .get("/users/:userName/books/:bookName", user_book_handler)
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() {
    let router = router();

    let builder = RequestServiceBuilder::new(router).unwrap();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("App is running on: {}", addr);

    loop {
        let (stream, remote_addr) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);
        let service = builder.build(remote_addr);
        tokio::task::spawn(async move {
            let builder = conn::auto::Builder::new(TokioExecutor::new());
            let res = builder.serve_connection(io, service).await;
            if let Err(err) = res {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
