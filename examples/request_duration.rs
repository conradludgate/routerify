use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn;
// Import the routerify prelude traits.
use routerify::{prelude::*, Body, Middleware, RequestInfo, RequestServiceBuilder, Router};
use std::{convert::Infallible, net::SocketAddr};
use tokio::net::TcpListener;

async fn before(req: Request<crate::Body>) -> Result<Request<crate::Body>, Infallible> {
    req.set_context(tokio::time::Instant::now());
    Ok(req)
}

async fn hello(_: Request<crate::Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Home page")))
}

async fn after(res: Response<Body>, req_info: RequestInfo) -> Result<Response<Body>, Infallible> {
    let started = req_info.context::<tokio::time::Instant>().unwrap();
    let duration = started.elapsed();
    println!("duration {:?}", duration);
    Ok(res)
}

fn router() -> Router<Body, Infallible> {
    Router::builder()
        .get("/", hello)
        .middleware(Middleware::pre(before))
        .middleware(Middleware::post_with_info(after))
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
