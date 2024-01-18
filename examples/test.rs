use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn;
// Import the routerify prelude traits.
use routerify::Router;
use routerify::{prelude::*, RequestServiceBuilder};
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

mod users {
    use routerify::Body;

    use super::*;

    #[derive(Clone)]
    struct State {
        count: Arc<Mutex<u8>>,
    }

    async fn list(req: Request<Body>) -> Result<Response<Body>, io::Error> {
        let count = req.data::<State>().unwrap().count.lock().unwrap();
        Ok(Response::new(Body::from(format!("Suppliers: {}", count))))
    }

    pub fn router() -> Router<Body, io::Error> {
        let state = State {
            count: Arc::new(Mutex::new(20)),
        };
        Router::builder().data(state).get("/", list).build().unwrap()
    }
}

mod offers {
    use routerify::Body;

    use super::*;

    #[derive(Clone)]
    struct State {
        count: Arc<Mutex<u8>>,
    }

    async fn list(req: Request<Body>) -> Result<Response<Body>, io::Error> {
        let count = req.data::<State>().unwrap().count.lock().unwrap();

        println!("I can also access parent state: {:?}", req.data::<String>().unwrap());

        Ok(Response::new(Body::from(format!("Suppliers: {}", count))))
    }

    pub fn router() -> Router<Body, io::Error> {
        let state = State {
            count: Arc::new(Mutex::new(100)),
        };
        Router::builder().data(state).get("/abc", list).build().unwrap()
    }
}

#[tokio::main]
async fn main() {
    let scopes = Router::builder()
        .data("Parent State data".to_owned())
        .scope("/offers", offers::router())
        .scope("/users", users::router())
        .build()
        .unwrap();

    let router = Router::builder().scope("/v1", scopes).build().unwrap();
    dbg!(&router);

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
