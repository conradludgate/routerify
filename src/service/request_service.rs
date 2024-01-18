use crate::helpers;
use crate::router::Router;
use crate::types::{RequestContext, RequestInfo, RequestMeta};
use crate::Error;
use http_body::Body;
use hyper::body::Incoming;
use hyper::service::Service;
use hyper::{Request, Response};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

pub struct RequestService<B, E> {
    pub(crate) router: Arc<Router<B, E>>,
    pub(crate) remote_addr: SocketAddr,
}

impl<B: Body + Send + Sync + 'static, E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static>
    Service<Request<Incoming>> for RequestService<B, E>
{
    type Response = Response<B>;
    type Error = crate::RouteError;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        self.call(req.map(crate::Body::new))
    }
}

impl<B: Body + Send + Sync + 'static, E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static>
    Service<Request<crate::Body>> for RequestService<B, E>
{
    type Response = Response<B>;
    type Error = crate::RouteError;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn call(&self, mut req: Request<crate::Body>) -> Self::Future {
        let router = self.router.clone();
        let remote_addr = self.remote_addr;

        let fut = async move {
            helpers::update_req_meta_in_extensions(req.extensions_mut(), RequestMeta::with_remote_addr(remote_addr));

            let mut target_path = helpers::percent_decode_request_path(req.uri().path())
                .map_err(|e| Error::new(format!("Couldn't percent decode request path: {}", e)))?;

            if target_path.is_empty() || target_path.as_bytes()[target_path.len() - 1] != b'/' {
                target_path.push('/');
            }

            let mut req_info = None;
            let should_gen_req_info = router
                .should_gen_req_info
                .expect("The `should_gen_req_info` flag in Router is not initialized");

            let context = RequestContext::new();

            if should_gen_req_info {
                req_info = Some(RequestInfo::new_from_req(&req, context.clone()));
            }

            req.extensions_mut().insert(context);

            router.process(target_path.as_str(), req, req_info.clone()).await
        };

        Box::pin(fut)
    }
}

#[derive(Debug)]
pub struct RequestServiceBuilder<B, E> {
    router: Arc<Router<B, E>>,
}

impl<B: Body + Send + Sync + 'static, E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static>
    RequestServiceBuilder<B, E>
{
    pub fn new(mut router: Router<B, E>) -> crate::Result<Self> {
        // router.init_keep_alive_middleware();

        router.init_global_options_route();
        router.init_default_404_route();

        router.init_err_handler();

        router.init_regex_set()?;
        router.init_req_info_gen();
        Ok(Self {
            router: Arc::from(router),
        })
    }

    pub fn build(&self, remote_addr: SocketAddr) -> RequestService<B, E> {
        RequestService {
            router: self.router.clone(),
            remote_addr,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Body;
    use crate::{Error, RequestServiceBuilder, Router};
    use http::Method;
    use http_body_util::BodyExt;
    use hyper::service::Service;
    use hyper::{Request, Response};
    use std::net::SocketAddr;
    use std::str::FromStr;

    #[tokio::test]
    async fn should_route_request() {
        const RESPONSE_TEXT: &str = "Hello world!";
        let remote_addr = SocketAddr::from_str("0.0.0.0:8080").unwrap();
        let router: Router<Body, Error> = Router::builder()
            .get("/", |_| async move { Ok(Response::new(Body::from(RESPONSE_TEXT))) })
            .build()
            .unwrap();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/")
            .body(crate::Body::empty())
            .unwrap();
        let builder = RequestServiceBuilder::new(router).unwrap();
        let service = builder.build(remote_addr);

        let resp: Response<Body> = service.call(req).await.unwrap();
        let body = resp.into_body();
        let body = String::from_utf8(body.collect().await.unwrap().to_bytes().to_vec()).unwrap();
        assert_eq!(RESPONSE_TEXT, body)
    }
}
