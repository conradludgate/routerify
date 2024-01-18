use crate::regex_generator::generate_exact_match_regex;
use crate::Error;
use hyper::Request;
use regex::Regex;
use std::fmt::{self, Debug, Formatter};
use std::future::Future;
use std::pin::Pin;

type Handler<E> = Box<dyn Fn(Request<crate::Body>) -> HandlerReturn<E> + Send + Sync + 'static>;
type HandlerReturn<E> = Box<dyn Future<Output = Result<Request<crate::Body>, E>> + Send + 'static>;

/// The pre middleware type. Refer to [Pre Middleware](./index.html#pre-middleware) for more info.
///
/// This `PreMiddleware<E>` type accepts a single type parameter: `E`.
///
/// * The `E` represents any error type which will be used by route handlers and the middlewares. This error type must implement the [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html).
pub struct PreMiddleware<E> {
    pub(crate) path: String,
    pub(crate) regex: Regex,
    // Make it an option so that when a router is used to scope in another router,
    // It can be extracted out by 'opt.take()' without taking the whole router's ownership.
    pub(crate) handler: Option<Handler<E>>,
    // Scope depth with regards to the top level router.
    pub(crate) scope_depth: u32,
}

impl<E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static> PreMiddleware<E> {
    pub(crate) fn new_with_boxed_handler<P: Into<String>>(
        path: P,
        handler: Handler<E>,
        scope_depth: u32,
    ) -> crate::Result<PreMiddleware<E>> {
        let path = path.into();
        let (re, _) = generate_exact_match_regex(path.as_str()).map_err(|e| {
            Error::new(format!(
                "Could not create an exact match regex for the pre middleware path: {}",
                e
            ))
        })?;

        Ok(PreMiddleware {
            path,
            regex: re,
            handler: Some(handler),
            scope_depth,
        })
    }

    /// Creates a pre middleware with a handler at the specified path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::{Router, Middleware, PreMiddleware};
    /// use hyper::{Request, Body};
    /// use std::convert::Infallible;
    ///
    /// # fn run() -> Router<Body, Infallible> {
    /// let router = Router::builder()
    ///      .middleware(Middleware::Pre(PreMiddleware::new("/abc", |req| async move { /* Do some operations */ Ok(req) }).unwrap()))
    ///      .build()
    ///      .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn new<P, H, R>(path: P, handler: H) -> crate::Result<PreMiddleware<E>>
    where
        P: Into<String>,
        H: Fn(Request<crate::Body>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Request<crate::Body>, E>> + Send + 'static,
    {
        let handler: Handler<E> = Box::new(move |req: Request<crate::Body>| Box::new(handler(req)));
        PreMiddleware::new_with_boxed_handler(path, handler, 1)
    }

    pub(crate) async fn process(&self, req: Request<crate::Body>) -> crate::Result<Request<crate::Body>> {
        let handler = self
            .handler
            .as_ref()
            .expect("A router can not be used after mounting into another router");

        Pin::from(handler(req)).await.map_err(Into::into)
    }
}

impl<E> Debug for PreMiddleware<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{ path: {:?}, regex: {:?} }}", self.path, self.regex)
    }
}
