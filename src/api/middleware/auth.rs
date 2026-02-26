use crate::config::AppConfig;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use std::{
    future::{ready, Future, Ready},
    pin::Pin,
    rc::Rc,
};
use tracing::warn;

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

        // Skip auth for /health, /playground, root landing page, and OPTIONS requests
        if req.method() == actix_web::http::Method::OPTIONS 
            || req.path() == "/health" 
            || req.path() == "/" 
            || req.path() == "/playground" 
        {
            return Box::pin(async move { srv.call(req).await });
        }

        // Get config from app data
        let config = match req.app_data::<actix_web::web::Data<AppConfig>>() {
            Some(c) => c,
            None => {
                warn!("AppConfig missing in app_data");
                return Box::pin(async move {
                    Err(actix_web::error::ErrorInternalServerError("Configuration error"))
                });
            }
        };

        // Extract Authorization header or check query params
        let auth_header = req.headers().get("Authorization");
        
        let valid = if let Some(header_value) = auth_header {
            if let Ok(auth_str) = header_value.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
                    config.auth.api_keys.iter().any(|key| key == token)
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            // Fallback to query param for WebSocket compatibility
            let query = req.query_string();
            let params = qstring::QString::from(query);
            if let Some(token) = params.get("api_key") {
                config.auth.api_keys.iter().any(|key| key == token)
            } else {
                false
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
