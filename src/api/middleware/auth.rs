use crate::config::AppConfig;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, web,
};
use std::{
    future::{ready, Future, Ready},
    pin::Pin,
    rc::Rc,
};
use tracing::warn;

// TokenManager removed

pub struct ApiKeyAuth;

impl<S, B> Transform<S, ServiceRequest> for ApiKeyAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiKeyAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiKeyAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct ApiKeyAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ApiKeyAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();

        let path = req.path();
        if req.method() == actix_web::http::Method::OPTIONS
            || path == "/api/health"
            || path == "/"
            || path == "/playground"
            || (!path.starts_with("/api") && !path.starts_with("/ws") && !path.starts_with("/sessions"))
        {
            return Box::pin(async move { srv.call(req).await });
        }

        let config = match req.app_data::<web::Data<AppConfig>>() {
            Some(c) => c,
            None => {
                warn!("AppConfig missing in app_data");
                return Box::pin(async move {
                    Err(actix_web::error::ErrorInternalServerError("Configuration error"))
                });
            }
        };

        let auth_header = req.headers().get("Authorization");
        
        let mut valid = false;
        if let Some(header_value) = auth_header {
            if let Ok(auth_str) = header_value.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
                    if config.auth.api_keys.iter().any(|key| key == token) {
                        valid = true;
                    }
                }
            }
        } else {
            let query = req.query_string();
            let params = qstring::QString::from(query);
            if let Some(token) = params.get("api_key") {
                if config.auth.api_keys.iter().any(|key| key == token) {
                    valid = true;
                }
            }
        };

        if !valid {
            return Box::pin(async move {
                Err(actix_web::error::ErrorUnauthorized("Invalid or missing API key"))
            });
        }

        Box::pin(async move {
            let res = srv.call(req).await?;
            Ok(res)
        })
    }
}
