# DuckDB Expert Skills

This skill provides a set of highly optimized queries and knowledge for managing the internal DuckDB database of Stepbit.

## System Introspection

### Schema Overview
List all tables and their column counts.
```sql
SELECT 
    table_name, 
    count(column_name) as cols
FROM information_schema.columns 
WHERE table_schema = 'main'
GROUP BY table_name;
```

### Table Details
Get detailed information about a specific table's structure.
```sql
PRAGMA table_info('sessions');
```

## Session Analysis

### Latest Sessions
View the most recent 10 sessions.
```sql
SELECT id, status, updated_at 
FROM sessions 
ORDER BY updated_at DESC 
LIMIT 10;
```

### Message Distribution
Count messages per session for active sessions.
```sql
SELECT 
    s.id, 
    count(m.id) as message_count
FROM sessions s
LEFT JOIN messages m ON s.id = m.session_id
GROUP BY s.id
HAVING message_count > 0;
```

## Storage Management

### Database Size
Estimate current database footprint.
```sql
SELECT * FROM duckdb_databases();
```

### Table Sizes
Identify large tables.
```sql
SELECT * FROM duckdb_tables();
```

## Advanced Querying

### JSON Extraction
Example of extracting fields from a JSON column (if any).
```sql
-- Assuming tool_results has a 'data' JSON column
-- SELECT data->'$.status' FROM tool_results;
```

### Time-based Aggregation
Count messages by hour.
```sql
SELECT 
    date_trunc('hour', created_at) as hour,
    count(*) as count
FROM messages
GROUP BY 1
ORDER BY 1 DESC;
```
