// borrowed from https://github.com/robjtede/actix-web-lab/ (MIT)
use std::{
    future::{ready, Ready},
    panic::AssertUnwindSafe,
    rc::Rc,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error,
};
use futures_core::future::LocalBoxFuture;
use futures_util::FutureExt as _;

/// A middleware to catch panics in wrapped handlers and middleware, returning empty 500 responses.
///
/// **This middleware should never be used as replacement for proper error handling.** See [this
/// thread](https://github.com/actix/actix-web/issues/1501#issuecomment-627517783) for historical
/// discussion on why Actix Web does not do this by default.
///
/// It is recommended that this middleware be registered last. That is, `wrap`ed after everything
/// else except `Logger`.
///
/// # Examples
///
/// ```no_run
/// # use actix_web::App;
/// use actix_web_lab::middleware::CatchPanic;
///
/// App::new().wrap(CatchPanic::default())
///     # ;
/// ```
///
/// ```no_run
/// # use actix_web::App;
/// use actix_web::middleware::{Logger, NormalizePath};
/// use actix_web_lab::middleware::CatchPanic;
///
/// // recommended wrap order
/// App::new()
///     .wrap(NormalizePath::default())
///     .wrap(CatchPanic::default()) // <- after everything except logger
///     .wrap(Logger::default())
///     # ;
/// ```
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct CatchPanic;

impl<S, B> Transform<S, ServiceRequest> for CatchPanic
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = CatchPanicMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CatchPanicMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct CatchPanicMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for CatchPanicMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        AssertUnwindSafe(self.service.call(req))
            .catch_unwind()
            .map(move |res| match res {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(svc_err)) => Err(svc_err),
                Err(_panic_err) => Err(error::ErrorInternalServerError(
                    INTERNAL_SERVER_ERROR_MESSAGE,
                )),
            })
            .boxed_local()
    }
}

const INTERNAL_SERVER_ERROR_MESSAGE: &str = "500 Server Error";

#[cfg(test)]
mod tests {
    use actix_web::{
        body::{to_bytes, MessageBody},
        dev::{Service as _, ServiceFactory},
        http::StatusCode,
        test, web, App, Error,
    };

    use super::*;

    fn test_app() -> App<
        impl ServiceFactory<
            ServiceRequest,
            Response = ServiceResponse<impl MessageBody>,
            Config = (),
            InitError = (),
            Error = Error,
        >,
    > {
        App::new()
            .wrap(CatchPanic::default())
            .route("/", web::get().to(|| async { "content" }))
            .route(
                "/disco",
                #[allow(unreachable_code)]
                web::get().to(|| async {
                    panic!("the disco");
                    ""
                }),
            )
    }

    #[actix_web::test]
    async fn pass_through_no_panic() {
        let app = test::init_service(test_app()).await;

        let req = test::TestRequest::default().to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), StatusCode::OK);
        let body = test::read_body(res).await;
        assert_eq!(body, "content");
    }

    #[actix_web::test]
    async fn catch_panic_return_internal_server_error_response() {
        let app = test::init_service(test_app()).await;

        let req = test::TestRequest::with_uri("/disco").to_request();
        let err = match app.call(req).await {
            Ok(_) => panic!("unexpected Ok response"),
            Err(err) => err,
        };
        let res = err.error_response();
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = to_bytes(res.into_body()).await.unwrap();
        assert_eq!(body, INTERNAL_SERVER_ERROR_MESSAGE)
    }
}
