use hyper::{Body, Request, Response, Server};
// Import the routerify prelude traits.
use routerify::prelude::*;
use routerify::{Router, RouterService};
use std::io;
use std::net::SocketAddr;

// A handler for "/" page.
async fn home_handler(_: Request<crate::Body>) -> Result<Response<Body>, io::Error> {
    Ok(Response::new(Body::from("Home page")))
}

// Define a different module which will have only API related handlers.
mod api {
    use super::*;

    // Define a handler for "/users/:userName/books/:bookName" API endpoint which will have two
    // route parameters: `userName` and `bookName`.
    async fn user_book_handler(req: Request<crate::Body>) -> Result<Response<Body>, io::Error> {
        let user_name = req.param("userName").unwrap();
        let book_name = req.param("bookName").unwrap();

        Ok(Response::new(Body::from(format!(
            "User: {}, Book: {}",
            user_name, book_name
        ))))
    }

    pub fn router() -> Router<Body, io::Error> {
        // Create a router for API and specify the the handlers.
        Router::builder()
            .get("/users/:userName/books/:bookName", user_book_handler)
            .build()
            .unwrap()
    }
}

fn router() -> Router<Body, io::Error> {
    // Create a root router and specify the the handlers.
    Router::builder()
        .get("/", home_handler)
        // Mount the api router at `/api` path.
        // Now the app can handle: `/api/users/:userName/books/:bookName` path.
        .scope("/api", api::router())
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() {
    let router = router();

    // Create a Service from the router above to handle incoming requests.
    let service = RouterService::new(router).unwrap();

    // The address on which the server will be listening.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));

    // Create a server by passing the created service to `.serve` method.
    let server = Server::bind(&addr).serve(service);

    println!("App is running on: {}", addr);
    if let Err(err) = server.await {
        eprintln!("Server error: {}", err);
    }
}
