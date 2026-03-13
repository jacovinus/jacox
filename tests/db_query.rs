use jacox::db::{connection, service::DbService};
use jacox::config::DatabaseConfig;
use serde_json::json;

#[test]
fn test_raw_sql_query() {
    let config = DatabaseConfig {
        path: ":memory:".to_string(),
    };
    let pool = connection::get_connection(&config).unwrap();
    let conn = pool.lock().unwrap();

    // 1. Simple SELECT
    let res = DbService::query_raw(&conn, "SELECT 1 as id, 'hello' as name").unwrap();
    assert_eq!(res.columns, vec!["id", "name"]);
    assert_eq!(res.rows[0]["id"], json!(1));
    assert_eq!(res.rows[0]["name"], json!("hello"));

    // 2. Querying schema tables
    DbService::insert_session(&conn, "Test Session", json!({})).unwrap();
    let res = DbService::query_raw(&conn, "SELECT name FROM sessions").unwrap();
    assert_eq!(res.rows[0]["name"], json!("Test Session"));

    // 3. Complex types (Boolean, Null)
    let res = DbService::query_raw(&conn, "SELECT true as b, null as n").unwrap();
    assert_eq!(res.rows[0]["b"], json!(true));
    assert_eq!(res.rows[0]["n"], json!(null));
}

#[test]
fn test_query_error_handling() {
    let config = DatabaseConfig {
        path: ":memory:".to_string(),
    };
    let pool = connection::get_connection(&config).unwrap();
    let conn = pool.lock().unwrap();

    // Syntax error
    let res = DbService::query_raw(&conn, "SELECT * FROM non_existent_table");
    assert!(res.is_err());
}
