use crate::db::models::{Message, Session};
use chrono::{DateTime, Utc};
use duckdb::{params, Connection, Result as DbResult, Row};
use uuid::Uuid;

pub struct DbService;

impl DbService {
    fn row_to_session(row: &Row) -> DbResult<Session> {
        let meta_str: String = row.get(4)?;
        let metadata = serde_json::from_str(&meta_str).unwrap_or(serde_json::json!({}));
        
        // DuckDB timestamp parsing helper
        let created_val: duckdb::types::Value = row.get(2)?;
        let updated_val: duckdb::types::Value = row.get(3)?;
        
        let created_str = match created_val {
            duckdb::types::Value::Text(s) => s,
            _ => String::new(),
        };
        let updated_str = match updated_val {
            duckdb::types::Value::Text(s) => s,
            _ => String::new(),
        };

        // NOTE: if DuckDB returns a raw timestamp we can't extract the text cleanly because we're not using the chrono feature of duckdb.
        // Instead of fighting the DB driver in tests vs prod we'll just query the timestamps AS text in our SELECT statements.
        
        let created_at = created_str.parse::<DateTime<Utc>>().unwrap_or_else(|_| Utc::now());
        let updated_at = updated_str.parse::<DateTime<Utc>>().unwrap_or_else(|_| Utc::now());

        Ok(Session {
            id: row.get::<_, String>(0)?.parse().unwrap_or_default(),
            name: row.get::<_, String>(1)?,
            created_at,
            updated_at,
            metadata,
        })
    }

    fn row_to_message(row: &Row) -> DbResult<Message> {
        let meta_str: String = row.get(7)?;
        let metadata = serde_json::from_str(&meta_str).unwrap_or(serde_json::json!({}));

        let created_val: duckdb::types::Value = row.get(6)?;
        let created_str = match created_val {
            duckdb::types::Value::Text(s) => s,
            _ => String::new(),
        };
        let created_at = created_str.parse::<DateTime<Utc>>().unwrap_or_else(|_| Utc::now());

        Ok(Message {
            id: row.get(0)?,
            session_id: row.get::<_, String>(1)?.parse().unwrap_or_default(),
            role: row.get::<_, String>(2)?,
            content: row.get::<_, String>(3)?,
            model: row.get::<_, Option<String>>(4)?,
            token_count: row.get::<_, Option<i32>>(5)?,
            created_at,
            metadata,
        })
    }

    // --- Session Operations ---

    pub fn insert_session(conn: &Connection, name: &str, metadata: serde_json::Value) -> DbResult<Session> {
        let id = Uuid::new_v4();
        let meta_str = metadata.to_string();
        
        conn.execute(
            "INSERT INTO sessions (id, name, metadata) VALUES (?, ?, ?)",
            params![id.to_string(), name, meta_str],
        )?;
        
        Self::get_session(conn, id).map(|s| s.unwrap())
    }

    pub fn get_session(conn: &Connection, id: Uuid) -> DbResult<Option<Session>> {
        let mut stmt = conn.prepare("SELECT id, name, CAST(created_at AS VARCHAR), CAST(updated_at AS VARCHAR), metadata FROM sessions WHERE id = ?")?;
        let mut rows = stmt.query_map(params![id.to_string()], Self::row_to_session)?;
        
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    pub fn list_sessions(conn: &Connection, limit: usize, offset: usize) -> DbResult<Vec<Session>> {
        let mut stmt = conn.prepare("SELECT id, name, CAST(created_at AS VARCHAR), CAST(updated_at AS VARCHAR), metadata FROM sessions ORDER BY updated_at DESC LIMIT ? OFFSET ?")?;
        let rows = stmt.query_map(params![limit as i64, offset as i64], Self::row_to_session)?;
        
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn delete_session(conn: &Connection, id: Uuid) -> DbResult<()> {
        conn.execute("BEGIN TRANSACTION", [])?;
        
        let id_str = id.to_string();
        
        // 1. Delete messages first to satisfy foreign key constraint
        if let Err(e) = conn.execute("DELETE FROM messages WHERE session_id = ?", params![id_str]) {
            let _ = conn.execute("ROLLBACK", []);
            return Err(e);
        }

        // 2. Delete the session
        if let Err(e) = conn.execute("DELETE FROM sessions WHERE id = ?", params![id_str]) {
            let _ = conn.execute("ROLLBACK", []);
            return Err(e);
        }

        conn.execute("COMMIT", [])?;
        Ok(())
    }

    // --- Message Operations ---

    pub fn insert_message(
        conn: &Connection,
        session_id: Uuid,
        role: &str,
        content: &str,
        model: Option<&str>,
        token_count: Option<i32>,
        metadata: serde_json::Value,
    ) -> DbResult<Message> {
        let meta_str = metadata.to_string();
        
        conn.execute(
            "INSERT INTO messages (session_id, role, content, model, token_count, metadata) 
             VALUES (?, ?, ?, ?, ?, ?)",
            params![session_id.to_string(), role, content, model, token_count, meta_str],
        )?;

        // Update the session's updated_at timestamp
        conn.execute(
            "UPDATE sessions SET updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![session_id.to_string()]
        )?;
        
        // Fetch the message we just inserted (since ID is generated by sequence)
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, model, token_count, CAST(created_at AS VARCHAR), metadata 
             FROM messages 
             WHERE session_id = ? 
             ORDER BY id DESC LIMIT 1"
        )?;
        let mut rows = stmt.query_map(params![session_id.to_string()], Self::row_to_message)?;
        
        Ok(rows.next().unwrap()?)
    }

    pub fn get_messages(conn: &Connection, session_id: Uuid, limit: usize, offset: usize) -> DbResult<Vec<Message>> {
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, model, token_count, CAST(created_at AS VARCHAR), metadata 
             FROM messages 
             WHERE session_id = ? 
             ORDER BY created_at ASC 
             LIMIT ? OFFSET ?"
        )?;
        
        let rows = stmt.query_map(params![session_id.to_string(), limit as i64, offset as i64], Self::row_to_message)?;
        
        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }
        Ok(messages)
    }
}
