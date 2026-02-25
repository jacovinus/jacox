#[cfg(test)]
mod tests {
    use jacox::db::connection;
    use jacox::db::service::DbService;
    use jacox::config::DatabaseConfig;
    use serde_json::json;
    
    // In memory database just for tests
    fn get_test_db() -> duckdb::Connection {
        let config = DatabaseConfig {
            path: ":memory:".to_string(),
        };
        let _pool = connection::get_connection(&config).unwrap();
        // Since get_connection returns a tokio::sync::Mutex, for simple sync tests
        // we can just bypass the get_connection and initialize it directly, OR we can use tokio.
        // It's cleaner to just recreate a raw connection and run the schema logic explicitly.
        
        let conn = duckdb::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
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
            "#
        ).unwrap();
        conn
    }

    #[test]
    fn test_session_lifecycle() {
        let conn = get_test_db();
        
        // 1. Insert Session
        let session = DbService::insert_session(&conn, "Test Chat", json!({"source": "test"})).unwrap();
        assert_eq!(session.name, "Test Chat");
        
        // 2. Get Session
        let fetched = DbService::get_session(&conn, session.id).unwrap().unwrap();
        assert_eq!(fetched.id, session.id);
        
        // 3. List Sessions
        let list = DbService::list_sessions(&conn, 10, 0).unwrap();
        assert_eq!(list.len(), 1);
        
        // 4. Delete Session
        DbService::delete_session(&conn, session.id).unwrap();
        let deleted = DbService::get_session(&conn, session.id).unwrap();
        assert!(deleted.is_none());
    }

    #[test]
    fn test_message_lifecycle() {
        let conn = get_test_db();
        let session = DbService::insert_session(&conn, "Test Chat 2", json!({})).unwrap();
        
        // 1. Insert Messages
        let msg1 = DbService::insert_message(
            &conn, session.id, "system", "You are a bot", None, None, json!({})
        ).unwrap();
        
        let msg2 = DbService::insert_message(
            &conn, session.id, "user", "Hello!", None, Some(5), json!({})
        ).unwrap();
        
        assert_eq!(msg1.role, "system");
        assert_eq!(msg1.session_id, session.id);
        assert_eq!(msg2.token_count, Some(5));

        // 2. Fetch Messages
        let history = DbService::get_messages(&conn, session.id, 10, 0).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].role, "system");
        assert_eq!(history[1].role, "user");
        
        // 3. Delete Session Cascades (Manually checked in our transaction)
        DbService::delete_session(&conn, session.id).unwrap();
        let empty_history = DbService::get_messages(&conn, session.id, 10, 0).unwrap();
        assert_eq!(empty_history.len(), 0);
    }
}
