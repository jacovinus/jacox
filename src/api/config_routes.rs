use actix_web::{get, post, web, HttpResponse, Result as WebResult};
use std::sync::Arc;

use crate::llm::{LlmProvider, ProviderManager};
use crate::api::models::{ActiveProviderRequest, ProviderInfo};

#[get("/providers")]
pub async fn list_providers(
    llm: web::Data<Arc<dyn LlmProvider>>,
) -> WebResult<HttpResponse> {
    let manager = llm.get_ref().as_any().downcast_ref::<ProviderManager>();
    
    match manager {
        Some(m) => {
            let active_id = m.get_active_provider_id();
            let mut providers = Vec::new();
            
            for id in m.list_providers() {
                let provider = m.get_provider(&id);
                let models = if let Some(p) = provider {
                    p.supported_models()
                } else {
                    vec![]
                };

                providers.push(ProviderInfo {
                    id: id.clone(),
                    active: id == active_id,
                    supported_models: models,
                    status: "unverified".to_string(),
                });
            }
            Ok(HttpResponse::Ok().json(providers))
        }
        None => {
            // Fallback for single provider configuration
            let models = llm.discover_models().await.unwrap_or_else(|_| llm.supported_models());
            Ok(HttpResponse::Ok().json(vec![ProviderInfo {
                id: llm.name().to_string(),
                active: true,
                supported_models: models,
                status: "online".to_string(),
            }]))
        }
    }
}

#[post("/active-provider")]
pub async fn set_active_provider(
    llm: web::Data<Arc<dyn LlmProvider>>,
    req: web::Json<ActiveProviderRequest>,
) -> WebResult<HttpResponse> {
    let manager = llm.get_ref().as_any().downcast_ref::<ProviderManager>();
    
    match manager {
        Some(m) => {
            match m.set_active_provider(&req.provider_id) {
                Ok(_) => Ok(HttpResponse::Ok().finish()),
                Err(e) => Ok(HttpResponse::BadRequest().body(e)),
            }
        }
        None => Ok(HttpResponse::InternalServerError().body("LLM provider is not a ProviderManager")),
    }
}

#[get("/active-provider")]
pub async fn get_active_provider_info(
    llm: web::Data<Arc<dyn LlmProvider>>,
) -> WebResult<HttpResponse> {
    let manager = llm.get_ref().as_any().downcast_ref::<ProviderManager>();
    
    let (id, active_model_id) = match manager {
        Some(m) => (m.get_active_provider_id(), m.get_active_model_id()),
        None => (llm.name().to_string(), None),
    };
    
    let models = llm.discover_models().await.unwrap_or_else(|_| llm.supported_models());
    let mut effective_model = active_model_id.unwrap_or_else(|| llm.default_model());
    
    if effective_model.is_empty() && !models.is_empty() {
        effective_model = models[0].clone();
    }

    let status = match llm.verify_connection().await {
        Ok(_) => "online",
        Err(_) => "offline",
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "id": id,
        "status": status,
        "supported_models": models,
        "active_model": effective_model,
    })))
}

#[get("/active-model")]
pub async fn get_active_model(
    llm: web::Data<Arc<dyn LlmProvider>>,
) -> WebResult<HttpResponse> {
    let manager = llm.get_ref().as_any().downcast_ref::<ProviderManager>();
    let active_model = manager.and_then(|m| m.get_active_model_id());
    let mut effective_model = active_model.unwrap_or_else(|| llm.default_model());
    
    if effective_model.is_empty() {
        let models = llm.discover_models().await.unwrap_or_else(|_| llm.supported_models());
        if !models.is_empty() {
            effective_model = models[0].clone();
        }
    }
    
    Ok(HttpResponse::Ok().json(serde_json::json!({ "model_id": effective_model })))
}

#[post("/active-model")]
pub async fn set_active_model(
    llm: web::Data<Arc<dyn LlmProvider>>,
    req: web::Json<crate::api::models::ActiveModelRequest>,
) -> WebResult<HttpResponse> {
    let manager = llm.get_ref().as_any().downcast_ref::<ProviderManager>();
    
    match manager {
        Some(m) => {
            m.set_active_model(req.model_id.clone());
            Ok(HttpResponse::Ok().finish())
        }
        None => Ok(HttpResponse::InternalServerError().body("LLM provider is not a ProviderManager")),
    }
}

#[post("/active-provider/verify")]
pub async fn verify_active_provider(
    llm: web::Data<Arc<dyn LlmProvider>>,
) -> WebResult<HttpResponse> {
    match llm.verify_connection().await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "online" }))),
        Err(e) => Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "offline", "error": e.to_string() }))),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/config")
            .service(list_providers)
            .service(set_active_provider)
            .service(get_active_provider_info)
            .service(verify_active_provider)
            .service(get_active_model)
            .service(set_active_model)
    );
}
