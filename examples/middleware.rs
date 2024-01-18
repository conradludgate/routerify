use hyper::{
    header::{self, HeaderValue},
    Request, Response,
};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn,
};
// Import the routerify prelude traits.
use routerify::{prelude::*, Body, RequestServiceBuilder};
use routerify::{Middleware, RequestInfo, Router};
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpListener;

// A handler for "/" page.
async fn home_handler(_: Request<crate::Body>) -> Result<Response<Body>, io::Error> {
    Ok(Response::new(Body::from("Home page")))
}

// A handler for "/about" page.
async fn about_handler(_: Request<crate::Body>) -> Result<Response<Body>, io::Error> {
    Ok(Response::new(Body::from("About page")))
}

// Define a pre middleware handler which will be executed on every request and
// logs some meta.
async fn logger_middleware(req: Request<crate::Body>) -> Result<Request<crate::Body>, io::Error> {
    println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
    Ok(req)
}

// Define a post middleware handler which will be executed on every request and
// adds a header to the response.
async fn my_custom_header_adder_middleware(mut res: Response<Body>) -> Result<Response<Body>, io::Error> {
    res.headers_mut()
        .insert("x-custom-header", HeaderValue::from_static("some value"));
    Ok(res)
}

// Define a post middleware handler which will be executed on every request and
// accesses request information and adds the session cookies to manage session.
async fn my_session_middleware(mut res: Response<Body>, req_info: RequestInfo) -> Result<Response<Body>, io::Error> {
    // Access a cookie.
    let cookie = req_info
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    res.headers_mut()
        .insert(header::SET_COOKIE, HeaderValue::from_str(cookie).unwrap());

    Ok(res)
}

fn router() -> Router<Body, io::Error> {
    // Create a router and specify the the handlers.
    Router::builder()
        // Create a pre middleware using `Middleware::pre()` method
        // and attach it to the router.
        .middleware(Middleware::pre(logger_middleware))
        // Create a post middleware using `Middleware::post()` method
        // and attach it to the router.
        .middleware(Middleware::post(my_custom_header_adder_middleware))
        // Create a post middleware which will require request info using `Middleware::post_with_info()` method
        // and attach it to the router.
        .middleware(Middleware::post_with_info(my_session_middleware))
        .get("/", home_handler)
        .get("/about", about_handler)
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
