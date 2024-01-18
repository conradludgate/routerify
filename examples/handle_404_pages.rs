use http::StatusCode;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn;
use routerify::{Body, RequestServiceBuilder, Router};
use std::{io, net::SocketAddr};
use tokio::net::TcpListener;

// A handler for "/" page.
async fn home_handler(_: Request<crate::Body>) -> Result<Response<Body>, io::Error> {
    Ok(Response::new(Body::from("Home page")))
}

// A handler for "/about" page.
async fn about_handler(_: Request<crate::Body>) -> Result<Response<Body>, io::Error> {
    Ok(Response::new(Body::from("About page")))
}

// Define a handler to handle any non-existent routes i.e. a 404 handler.
async fn handler_404(_: Request<crate::Body>) -> Result<Response<Body>, io::Error> {
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Page Not Found"))
        .unwrap())
}

fn router() -> Router<Body, io::Error> {
    // Create a router and specify the the handlers.
    Router::builder()
        .get("/", home_handler)
        .get("/about", about_handler)
        // Add a route to handle 404 routes.
        .any(handler_404)
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
