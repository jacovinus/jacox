use crate::config::AppConfig;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, web,
};
use std::{
    future::{ready, Future, Ready},
    pin::Pin,
    rc::Rc,
    sync::{Arc, Mutex},
};
use tracing::{warn, debug};
use uuid::Uuid;

pub struct TokenManager {
    current_token: Arc<Mutex<Option<String>>>,
}

impl TokenManager {
    pub fn new() -> Self {
        Self {
            current_token: Arc::new(Mutex::new(None)),
        }
    }

    pub fn validate(&self, token: &str) -> bool {
        let guard = self.current_token.lock().unwrap();
        match guard.as_deref() {
            Some(t) => t == token,
            None => false,
        }
    }

    pub fn rotate(&self) -> String {
        let new_token = Uuid::new_v4().to_string();
        let mut guard = self.current_token.lock().unwrap();
        *guard = Some(new_token.clone());
        new_token
    }
}

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
            || path == "/health"
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

        let token_manager = match req.app_data::<web::Data<TokenManager>>() {
            Some(tm) => tm,
            None => {
                warn!("TokenManager missing in app_data");
                return Box::pin(async move {
                    Err(actix_web::error::ErrorInternalServerError("Security configuration error"))
                });
            }
        };

        let auth_header = req.headers().get("Authorization");
        
        let mut valid = false;
        if let Some(header_value) = auth_header {
            if let Ok(auth_str) = header_value.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
                    // Check against static keys OR the current rotated token
                    if config.auth.api_keys.iter().any(|key| key == token) {
                        valid = true;
                    } else if token_manager.validate(token) {
                        valid = true;
                        debug!("Authenticated via rotated token");
                    }
                }
            }
        } else {
            let query = req.query_string();
            let params = qstring::QString::from(query);
            if let Some(token) = params.get("api_key") {
                if config.auth.api_keys.iter().any(|key| key == token) {
                    valid = true;
                } else if token_manager.validate(token) {
                    valid = true;
                }
            }
        };

        if !valid {
            return Box::pin(async move {
                Err(actix_web::error::ErrorUnauthorized("Invalid or missing API key"))
            });
        }

        let token_manager_clone = token_manager.clone();
        Box::pin(async move {
            let mut res = srv.call(req).await?;
            
            // Generate next token and attach to response
            let next_token = token_manager_clone.rotate();
            res.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static("x-next-token"),
                actix_web::http::header::HeaderValue::from_str(&next_token).unwrap()
            );
            
            Ok(res)
        })
    }
}
