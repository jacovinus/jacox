use actix_web::{delete, get, patch, post, web, HttpResponse, Result as WebResult};
use std::sync::Arc;
use crate::api::models::{PipelineRequest, PipelineResponse, PipelineExecuteRequest, StepbitCoreStatusResponse};
use crate::db::{service::DbService, DbPool};
use crate::llm::LlmProvider;

#[post("")]
pub async fn create_pipeline(
    pool: web::Data<DbPool>,
    req: web::Json<PipelineRequest>,
) -> WebResult<HttpResponse> {
    let conn = pool.lock().unwrap();
    match DbService::insert_pipeline(&conn, &req.name, req.definition.clone()) {
        Ok(p) => Ok(HttpResponse::Created().json(map_pipeline_to_response(p))),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

#[get("")]
pub async fn list_pipelines(
    pool: web::Data<DbPool>,
    query: web::Query<crate::api::models::PaginationQuery>,
) -> WebResult<HttpResponse> {
    let conn = pool.lock().unwrap();
    match DbService::list_pipelines(&conn, query.limit, query.offset) {
        Ok(pipelines) => Ok(HttpResponse::Ok().json(
            pipelines.into_iter().map(map_pipeline_to_response).collect::<Vec<_>>()
        )),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

#[get("/{id}")]
pub async fn get_pipeline(
    pool: web::Data<DbPool>,
    id: web::Path<i64>,
) -> WebResult<HttpResponse> {
    let conn = pool.lock().unwrap();
    match DbService::get_pipeline(&conn, id.into_inner()) {
        Ok(Some(p)) => Ok(HttpResponse::Ok().json(map_pipeline_to_response(p))),
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

#[patch("/{id}")]
pub async fn update_pipeline(
    pool: web::Data<DbPool>,
    id: web::Path<i64>,
    req: web::Json<PipelineRequest>,
) -> WebResult<HttpResponse> {
    let conn = pool.lock().unwrap();
    match DbService::update_pipeline(&conn, id.into_inner(), Some(req.name.clone()), Some(req.definition.clone())) {
        Ok(Some(p)) => Ok(HttpResponse::Ok().json(map_pipeline_to_response(p))),
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

#[delete("/{id}")]
pub async fn delete_pipeline(
    pool: web::Data<DbPool>,
    id: web::Path<i64>,
) -> WebResult<HttpResponse> {
    let conn = pool.lock().unwrap();
    match DbService::delete_pipeline(&conn, id.into_inner()) {
        Ok(_) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

#[post("/{id}/execute")]
pub async fn execute_pipeline(
    pool: web::Data<DbPool>,
    llm: web::Data<Arc<dyn LlmProvider>>,
    id: web::Path<i64>,
    req: web::Json<PipelineExecuteRequest>,
) -> WebResult<HttpResponse> {
    let conn = pool.lock().unwrap();
    let id = id.into_inner();
    
    let pipeline = match DbService::get_pipeline(&conn, id) {
        Ok(Some(p)) => p,
        Ok(None) => return Ok(HttpResponse::NotFound().body("Pipeline not found")),
        Err(e) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
    };
    
    drop(conn); // Release DB lock before long LLM call

    match llm.execute_pipeline(pipeline.definition, req.question.clone()).await {
        Ok(result) => Ok(HttpResponse::Ok().json(result)),
        Err(e) => Ok(HttpResponse::ServiceUnavailable().body(format!("stepbit-core Error: {}", e))),
    }
}

#[get("/stepbit-core/status")]
pub async fn get_stepbit_core_status(
    llm: web::Data<Arc<dyn LlmProvider>>,
) -> WebResult<HttpResponse> {
    match llm.verify_connection().await {
        Ok(_) => Ok(HttpResponse::Ok().json(StepbitCoreStatusResponse {
            online: true,
            message: "stepbit-core is online".to_string(),
        })),
        Err(e) => Ok(HttpResponse::Ok().json(StepbitCoreStatusResponse {
            online: false,
            message: format!("stepbit-core is offline: {}", e),
        })),
    }
}

fn map_pipeline_to_response(p: crate::db::models::Pipeline) -> PipelineResponse {
    PipelineResponse {
        id: p.id,
        name: p.name,
        definition: p.definition,
        created_at: p.created_at.to_rfc3339(),
        updated_at: p.updated_at.to_rfc3339(),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/pipelines")
            .service(create_pipeline)
            .service(list_pipelines)
            .service(get_pipeline)
            .service(update_pipeline)
            .service(delete_pipeline)
            .service(execute_pipeline)
    );
    cfg.service(get_stepbit_core_status);
}
