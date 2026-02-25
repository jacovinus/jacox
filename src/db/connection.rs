use crate::config::DatabaseConfig;
use duckdb::{Connection, Result as DbResult};
use std::sync::{Arc, Mutex};
use tracing::info;

pub type DbPool = Arc<Mutex<Connection>>;

const SCHEMA: &str = r#"
CREATE SEQUENCE IF NOT EXISTS seq_messages_id;

CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY,
    name VARCHAR,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata JSON DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS messages (
    id BIGINT PRIMARY KEY DEFAULT nextval('seq_messages_id'),
    session_id UUID,
    role VARCHAR NOT NULL,
    content TEXT NOT NULL,
    model VARCHAR,
    token_count INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata JSON DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id, created_at);
"#;

pub fn get_connection(config: &DatabaseConfig) -> DbResult<DbPool> {
    info!("Connecting to DuckDB at {}", config.path);
    let conn = Connection::open(&config.path)?;
    
    init_schema(&conn)?;
    
    Ok(Arc::new(Mutex::new(conn)))
}

fn init_schema(conn: &Connection) -> DbResult<()> {
    info!("Initializing database schema");
    conn.execute_batch(SCHEMA)?;
    Ok(())
}
