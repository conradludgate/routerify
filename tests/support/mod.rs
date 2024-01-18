use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn,
};
use routerify::{RequestServiceBuilder, Router};
use std::{net::SocketAddr, pin::pin};
use tokio::{net::TcpListener, select};
use tokio_util::sync::CancellationToken;

pub struct Serve {
    addr: SocketAddr,
    shutdown: CancellationToken,
}

impl Serve {
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn new_request(&self, method: &str, route: &str) -> http::request::Builder {
        http::request::Request::builder()
            .method(method.to_ascii_uppercase().as_str())
            .uri(format!("http://{}{}", self.addr(), route))
    }

    pub fn shutdown(self) {
        self.shutdown.cancel()
    }
}

pub async fn serve<B, E>(router: Router<B, E>) -> Serve
where
    B: hyper::body::Body + Send + Sync + 'static,
    E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static,
    <B as hyper::body::Body>::Data: Send + Sync + 'static,
    <B as hyper::body::Body>::Error: Into<Box<dyn std::error::Error + Send + Sync>> + 'static,
{
    let builder = RequestServiceBuilder::new(router).unwrap();
    let shutdown = CancellationToken::new();

    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = TcpListener::bind(addr).await.unwrap();
    let addr = listener.local_addr().unwrap();

    let shutdown2 = shutdown.clone();
    tokio::spawn(async move {
        loop {
            let (stream, remote_addr) = select! {
                res = listener.accept() => res.unwrap(),
                _ = shutdown2.cancelled() => break,
            };
            let io = TokioIo::new(stream);
            let service = builder.build(remote_addr);

            let shutdown = shutdown2.clone();
            tokio::task::spawn(async move {
                let builder = conn::auto::Builder::new(TokioExecutor::new());
                let mut conn = pin!(builder.serve_connection(io, service));
                let res = select! {
                    _ = shutdown.cancelled() => {
                        conn.as_mut().graceful_shutdown();
                        conn.await
                    }
                    res = conn.as_mut() => res,
                };
                if let Err(err) = res {
                    println!("Error serving connection: {:?}", err);
                }
            });
        }
    });

    Serve { addr, shutdown }
}

pub async fn into_text(body: Incoming) -> String {
    String::from_utf8_lossy(&body.collect().await.unwrap().to_bytes()).to_string()
}
