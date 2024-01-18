use http::{header, StatusCode};
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn;
use routerify::{Body, RequestInfo, RequestServiceBuilder, Router};
use std::{io, net::SocketAddr};
use tokio::net::TcpListener;

// A handler for "/" page.
async fn home_handler(_: Request<crate::Body>) -> Result<Response<Body>, io::Error> {
    Err(io::Error::new(io::ErrorKind::Other, "Some errors"))
}

// A handler for "/about" page.
async fn about_handler(_: Request<crate::Body>) -> Result<Response<Body>, io::Error> {
    Ok(Response::new(Body::from("About page")))
}

// Define an error handler function which will accept the `routerify::Error` and the `req_info`
// and generates an appropriate response.
async fn error_handler(err: routerify::RouteError, req_info: RequestInfo) -> Response<Body> {
    eprintln!("{}", err);

    // Access a cookie.
    let cookie = req_info.headers().get(header::COOKIE).unwrap().to_str().unwrap();

    Response::builder()
        .header(header::SET_COOKIE, cookie)
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(format!("Something went wrong: {}", err)))
        .unwrap()
}

fn router() -> Router<Body, io::Error> {
    // Create a router and specify the the handlers.
    Router::builder()
        .get("/", home_handler)
        .get("/about", about_handler)
        // Specify the error handler to handle any errors caused by
        // a route or any middleware.
        .err_handler_with_info(error_handler)
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
