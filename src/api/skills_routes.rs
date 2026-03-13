use actix_web::{delete, get, patch, post, web, HttpResponse, Result as WebResult};
use reqwest::Client;

use crate::api::models::{CreateSkillRequest, FetchUrlRequest, PaginationQuery, UpdateSkillRequest};
use crate::db::{service::DbService, DbPool};

// GET /api/skills
#[get("/skills")]
pub async fn list_skills(
    db: web::Data<DbPool>,
    query: web::Query<PaginationQuery>,
) -> WebResult<HttpResponse> {
    let limit = query.limit;
    let offset = query.offset;

    let conn = db.lock().map_err(|_| {
        actix_web::error::ErrorInternalServerError("DB lock failed")
    })?;

    match DbService::list_skills(&conn, limit, offset) {
        Ok(skills) => Ok(HttpResponse::Ok().json(skills)),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

// POST /api/skills
#[post("/skills")]
pub async fn create_skill(
    db: web::Data<DbPool>,
    body: web::Json<CreateSkillRequest>,
) -> WebResult<HttpResponse> {
    let conn = db.lock().map_err(|_| {
        actix_web::error::ErrorInternalServerError("DB lock failed")
    })?;

    let source_url = body.source_url.as_deref();

    match DbService::insert_skill(&conn, &body.name, &body.content, &body.tags, source_url) {
        Ok(skill) => Ok(HttpResponse::Created().json(skill)),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

// GET /api/skills/{id}
#[get("/skills/{id}")]
pub async fn get_skill(
    db: web::Data<DbPool>,
    path: web::Path<i64>,
) -> WebResult<HttpResponse> {
    let id = path.into_inner();
    let conn = db.lock().map_err(|_| {
        actix_web::error::ErrorInternalServerError("DB lock failed")
    })?;

    match DbService::get_skill(&conn, id) {
        Ok(Some(skill)) => Ok(HttpResponse::Ok().json(skill)),
        Ok(None) => Ok(HttpResponse::NotFound().body("Skill not found")),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

// PATCH /api/skills/{id}
#[patch("/skills/{id}")]
pub async fn update_skill(
    db: web::Data<DbPool>,
    path: web::Path<i64>,
    body: web::Json<UpdateSkillRequest>,
) -> WebResult<HttpResponse> {
    let id = path.into_inner();
    let conn = db.lock().map_err(|_| {
        actix_web::error::ErrorInternalServerError("DB lock failed")
    })?;

    match DbService::update_skill(&conn, id, body.name.clone(), body.content.clone(), body.tags.clone()) {
        Ok(Some(skill)) => Ok(HttpResponse::Ok().json(skill)),
        Ok(None) => Ok(HttpResponse::NotFound().body("Skill not found")),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

// DELETE /api/skills/{id}
#[delete("/skills/{id}")]
pub async fn delete_skill(
    db: web::Data<DbPool>,
    path: web::Path<i64>,
) -> WebResult<HttpResponse> {
    let id = path.into_inner();
    let conn = db.lock().map_err(|_| {
        actix_web::error::ErrorInternalServerError("DB lock failed")
    })?;

    match DbService::delete_skill(&conn, id) {
        Ok(()) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

// POST /api/skills/fetch-url
#[post("/skills/fetch-url")]
pub async fn fetch_url(
    db: web::Data<DbPool>,
    body: web::Json<FetchUrlRequest>,
) -> WebResult<HttpResponse> {
    let client = Client::new();

    let response = client
        .get(&body.url)
        .header("User-Agent", "jacox/1.0")
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorBadGateway(e.to_string()))?;

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let raw = response
        .text()
        .await
        .map_err(|e| actix_web::error::ErrorBadGateway(e.to_string()))?;

    // Strip basic HTML tags if the response is HTML
    let content = if content_type.contains("text/html") {
        strip_html_tags(&raw)
    } else {
        raw
    };

    let conn = db.lock().map_err(|_| {
        actix_web::error::ErrorInternalServerError("DB lock failed")
    })?;

    match DbService::insert_skill(
        &conn,
        &body.name,
        &content,
        &body.tags,
        Some(&body.url),
    ) {
        Ok(skill) => Ok(HttpResponse::Created().json(skill)),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

/// Very simple HTML-to-text: removes all tags and decodes common entities.
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    // Collapse whitespace
    result
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(list_skills)
        .service(create_skill)
        .service(fetch_url) // must be before get_skill so /skills/fetch-url isn't caught by /{id}
        .service(get_skill)
        .service(update_skill)
        .service(delete_skill);
}
