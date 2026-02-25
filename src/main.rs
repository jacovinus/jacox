use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;
use jacox::config::AppConfig;
use jacox::db;
use jacox::api::middleware::ApiKeyAuth;
use jacox::llm::ProviderFactory;
use jacox::cli::{commands::{Cli, Commands}, run_cli};
use tracing::{error, info};

async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "healthy"}))
}

async fn index() -> impl Responder {
    let html = include_str!("../static/index.html");
    HttpResponse::Ok().content_type("text/html").body(html)
}

async fn playground() -> impl Responder {
    let html = include_str!("../static/playground.html");
    HttpResponse::Ok().content_type("text/html").body(html)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if !matches!(cli.command, Commands::Serve) {
        run_cli(cli.command, cli.config).await;
        return Ok(());
    }

    info!("Starting Jacox LLM Server...");

    let config = match AppConfig::load(&cli.config) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    let db_pool = match db::get_connection(&config.database) {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    let llm_provider = match ProviderFactory::create_default(&config) {
        Some(p) => p,
        None => {
            error!("Failed to initialize LLM Provider from config.yaml mapping");
            std::process::exit(1);
        }
    };

    let host = config.server.host.clone();
    let port = config.server.port;

    info!("Server listening on {}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(llm_provider.clone()))
            .route("/", web::get().to(index))
            .route("/playground", web::get().to(playground))
            .route("/health", web::get().to(health))
            .wrap(ApiKeyAuth)
            .configure(jacox::api::routes::configure)
            .configure(jacox::api::websocket::configure)
            .service(jacox::api::routes_openai::openai_chat_completions)
    })
    .bind((host, port))?
    .run()
    .await
}
