use actix_files::NamedFile;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use clap::Parser;
use jacox::config::AppConfig;
use jacox::db;
use jacox::api::middleware::ApiKeyAuth;
use jacox::llm::ProviderFactory;
use jacox::cli::{commands::{Cli, Commands}, run_cli};
use tracing::{error, info};
use std::path::PathBuf;

async fn health(db: web::Data<jacox::db::DbPool>) -> impl Responder {
    let db_status = match db.lock() {
        Ok(conn) => {
            match conn.execute("SELECT 1", []) {
                Ok(_) => "connected",
                Err(_) => "disconnected",
            }
        },
        Err(_) => "disconnected",
    };
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "api": "connected",
        "database": db_status
    }))
}

async fn index() -> impl Responder {
    let path: PathBuf = "./frontend/dist/index.html".into();
    NamedFile::open(path)
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
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(llm_provider.clone()))
            .route("/health", web::get().to(health))
            .wrap(ApiKeyAuth)
            .wrap(cors)
            .configure(jacox::api::routes::configure)
            .configure(jacox::api::websocket::configure)
            .service(jacox::api::routes_openai::openai_chat_completions)
            // Serve static files from React build
            .service(
                actix_files::Files::new("/", "./frontend/dist")
                    .index_file("index.html")
                    .default_handler(web::get().to(index))
            )
    })
    .bind((host, port))?
    .run()
    .await
}
