use http::StatusCode;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn;
// Import the routerify prelude traits.
use core::fmt;
use routerify::{Body, RequestServiceBuilder, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;

// Define a custom error enum to model a possible API service error.
#[derive(Debug)]
enum ApiError {
    #[allow(dead_code)]
    Unauthorized,
    Generic(String),
}

impl std::error::Error for ApiError {}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::Unauthorized => write!(f, "Unauthorized"),
            ApiError::Generic(s) => write!(f, "Generic: {}", s),
        }
    }
}

// Router, handlers and middleware must use the same error type.
// In this case it's `ApiError`.

// A handler for "/" page.
async fn home_handler(_: Request<crate::Body>) -> Result<Response<Body>, ApiError> {
    // Simulate failure by returning `ApiError::Generic` variant.
    Err(ApiError::Generic("Something went wrong!".into()))
}

// Define an error handler function which will accept the `routerify::RouteError`
// and generates an appropriate response.
async fn error_handler(err: routerify::RouteError) -> Response<Body> {
    // Because `routerify::RouteError` is a boxed error, it must be
    // downcasted first. Unwrap for simplicity.
    let api_err = err.downcast::<ApiError>().unwrap();

    // Now that we've got the actual error, we can handle it
    // appropriately.
    match api_err.as_ref() {
        ApiError::Unauthorized => Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::empty())
            .unwrap(),
        ApiError::Generic(s) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(s.to_string()))
            .unwrap(),
    }
}

fn router() -> Router<Body, ApiError> {
    // Create a router and specify the the handlers.
    Router::builder()
        .get("/", home_handler)
        // Specify the error handler to handle any errors caused by
        // a route or any middleware.
        .err_handler(error_handler)
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
