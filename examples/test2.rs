use http::StatusCode;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn;
// Import the routerify prelude traits.
use routerify::{prelude::*, Body, Middleware, RequestInfo, RequestServiceBuilder, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct State(pub i32);

pub async fn pre_middleware(req: Request<Body>) -> Result<Request<Body>, routerify::Error> {
    let data = req.data::<State>().map(|s| s.0).unwrap_or(0);
    println!("Pre Data: {}", data);
    println!("Pre Data2: {:?}", req.data::<u32>());

    Ok(req)
}

pub async fn post_middleware(res: Response<Body>, req_info: RequestInfo) -> Result<Response<Body>, routerify::Error> {
    let data = req_info.data::<State>().map(|s| s.0).unwrap_or(0);
    println!("Post Data: {}", data);

    Ok(res)
}

pub async fn home_handler(req: Request<Body>) -> Result<Response<Body>, routerify::Error> {
    let data = req.data::<State>().map(|s| s.0).unwrap_or(0);
    println!("Route Data: {}", data);
    println!("Route Data2: {:?}", req.data::<u32>());

    Err(routerify::Error::new("Error"))
}

async fn error_handler(err: routerify::RouteError, req_info: RequestInfo) -> Response<Body> {
    let data = req_info.data::<State>().map(|s| s.0).unwrap_or(0);
    println!("Error Data: {}", data);
    println!("Error Data2: {:?}", req_info.data::<u32>());

    eprintln!("{}", err);
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(format!("Something went wrong: {}", err)))
        .unwrap()
}

fn router2() -> Router<Body, routerify::Error> {
    Router::builder()
        .data(111_u32)
        .get("/a", |req| async move {
            println!("Router2 Data: {:?}", req.data::<&str>());
            println!("Router2 Data: {:?}", req.data::<State>().map(|s| s.0));
            println!("Router2 Data: {:?}", req.data::<u32>());
            Ok(Response::new(Body::from("Hello world!")))
        })
        .build()
        .unwrap()
}

fn router3() -> Router<Body, routerify::Error> {
    Router::builder()
        .data(555_u32)
        .get("/h/g/j", |req| async move {
            println!("Router3 Data: {:?}", req.data::<&str>());
            println!("Router3 Data: {:?}", req.data::<State>().map(|s| s.0));
            println!("Router3 Data: {:?}", req.data::<u32>());
            Ok(Response::new(Body::from("Hello world!")))
        })
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() {
    let router: Router<Body, routerify::Error> = Router::builder()
        .data(State(100))
        .scope("/r", router2())
        .scope("/bcd", router3())
        .data("abcd")
        .middleware(Middleware::pre(pre_middleware))
        .middleware(Middleware::post_with_info(post_middleware))
        .get("/", home_handler)
        .err_handler_with_info(error_handler)
        .build()
        .unwrap();

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
