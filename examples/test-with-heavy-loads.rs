use hyper::Response;
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn,
};
use routerify::{Body, Middleware, RequestServiceBuilder, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;

fn router() -> Router<Body, routerify::Error> {
    let mut builder = Router::builder();

    for i in 0..3000_usize {
        builder = builder.middleware(
            Middleware::pre_with_path(format!("/abc-{}", i), move |req| async move {
                // println!("PreMiddleware: {}", format!("/abc-{}", i));
                Ok(req)
            })
            .unwrap(),
        );

        builder = builder.get(format!("/abc-{}", i), move |_req| async move {
            // println!("Route: {}, params: {:?}", format!("/abc-{}", i), req.params());
            Ok(Response::new(Body::from(format!("/abc-{}", i))))
        });

        builder = builder.middleware(
            Middleware::post_with_path(format!("/abc-{}", i), move |res| async move {
                // println!("PostMiddleware: {}", format!("/abc-{}", i));
                Ok(res)
            })
            .unwrap(),
        );
    }

    builder.build().unwrap()
}

#[tokio::main]
async fn main() {
    let router = router();

    let builder = RequestServiceBuilder::new(router).unwrap();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
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
