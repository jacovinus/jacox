pub mod commands;

use tokio::sync::mpsc;
use std::io::{self, Write};

use crate::config::AppConfig;
use crate::db::{service::DbService, get_connection};
use crate::llm::{
    models::{ChatOptions, Message as LlmMessage},
    ProviderFactory,
};
use crate::cli::commands::{Commands, SessionAction};
use uuid::Uuid;

pub async fn run_cli(command: Commands, config_path: String) {
    let config = AppConfig::load(&config_path).expect("Failed to load config");
    
    match command {
        Commands::Serve => {
            panic!("Serve command should be intercepted by main.rs to boot actix-web");
        }
        Commands::Session { action } => {
            let pool = get_connection(&config.database).expect("DB error");
            let conn = pool.lock().unwrap();
            
            match action {
                SessionAction::Create { name } => {
                    match DbService::insert_session(&conn, &name, serde_json::json!({})) {
                        Ok(session) => println!("Created Session: {} ({})", session.name, session.id),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                SessionAction::List => {
                    match DbService::list_sessions(&conn, 50, 0) {
                        Ok(sessions) => {
                            if sessions.is_empty() {
                                println!("No sessions found.");
                            } else {
                                println!("{:<38} | {:<20} | {}", "ID", "Created At", "Name");
                                println!("{:-<38}-+-{:-<20}-+-{:-<20}", "", "", "");
                                for s in sessions {
                                    println!("{:<38} | {:<20} | {}", s.id.to_string(), s.created_at, s.name);
                                }
                            }
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                SessionAction::Delete { id } => {
                    match DbService::delete_session(&conn, id) {
                        Ok(_) => println!("Deleted session {}", id),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                SessionAction::Export { id, path } => {
                    let session = match DbService::get_session(&conn, id) {
                        Ok(Some(s)) => s,
                        _ => { eprintln!("Session {} not found.", id); return; }
                    };
                    let messages = DbService::get_messages(&conn, id, 1000, 0).unwrap_or_default();
                    
                    let export_path = path.unwrap_or_else(|| format!("session_{}.txt", id));
                    let mut file = std::fs::File::create(&export_path).expect("Failed to create file");
                    
                    writeln!(file, "Session: {}", session.name).unwrap();
                    writeln!(file, "ID: {}", session.id).unwrap();
                    writeln!(file, "Created At: {}", session.created_at).unwrap();
                    writeln!(file, "---").unwrap();
                    
                    for m in messages {
                        writeln!(file, "[{}]: {}", m.role.to_uppercase(), m.content).unwrap();
                        writeln!(file, "---").unwrap();
                    }
                    
                    println!("Session exported successfully to: {}", export_path);
                }
                SessionAction::Import { path } => {
                    let content = std::fs::read_to_string(&path).expect("Failed to read file");
                    let mut lines = content.lines();
                    
                    let name = lines.next()
                        .and_then(|l| l.strip_prefix("Session: "))
                        .unwrap_or("Imported Session");
                        
                    match DbService::insert_session(&conn, name, serde_json::json!({})) {
                        Ok(session) => {
                            println!("Created new session: {}", session.id);
                            
                            let mut current_role = String::new();
                            let mut current_content = String::new();
                            
                            for line in lines {
                                if line == "---" {
                                    if !current_role.is_empty() && !current_content.is_empty() {
                                        let _ = DbService::insert_message(
                                            &conn, session.id, &current_role.to_lowercase(), 
                                            &current_content.trim(), None, None, serde_json::json!({})
                                        );
                                        current_content.clear();
                                    }
                                } else if line.starts_with("[") && line.contains("]: ") {
                                    if let (Some(start), Some(end)) = (line.find('['), line.find(']')) {
                                        current_role = line[start+1..end].to_string();
                                        current_content = line[end+2..].to_string();
                                    }
                                } else {
                                    current_content.push_str("\n");
                                    current_content.push_str(line);
                                }
                            }
                            println!("Import completed successfully.");
                        }
                        Err(e) => eprintln!("Failed to create session for import: {}", e),
                    }
                }
            }
        }
        Commands::Chat { session } => {
            run_repl(session, config).await;
        }
    }
}

async fn run_repl(session_id: Uuid, config: AppConfig) {
    let pool = get_connection(&config.database).expect("DB Error");
    
    // Verify session
    let session_exists = {
        let conn = pool.lock().unwrap();
        DbService::get_session(&conn, session_id).unwrap_or(None).is_some()
    };
    
    if !session_exists {
        eprintln!("Session {} not found.", session_id);
        return;
    }
    
    let llm = ProviderFactory::create_default(&config).expect("Failed to init LLM provider");
    
    println!("--- Jacox Terminal Chat ---");
    println!("Connected to Session: {}", session_id);
    println!("Type /exit to quit.");
    println!("---------------------------");

    loop {
        print!("\nUser> ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let text = input.trim();
        
        if text.is_empty() { continue; }
        if text == "/exit" || text == "/quit" { break; }
        
        // Save user message
        {
            let conn = pool.lock().unwrap();
            if let Err(e) = DbService::insert_message(&conn, session_id, "user", text, None, None, serde_json::json!({})) {
                eprintln!("Failed to save message: {}", e);
                continue;
            }
        }
        
        // Fetch history
        let history = {
            let conn = pool.lock().unwrap();
            DbService::get_messages(&conn, session_id, 50, 0).unwrap_or_default()
        };
        
        let llm_messages: Vec<LlmMessage> = history.into_iter().map(|m| LlmMessage {
            role: m.role,
            content: m.content,
        }).collect();
        
        let (tx, mut rx) = mpsc::channel::<String>(100);
        let llm_clone = llm.clone();
        
        print!("Jacox> ");
        io::stdout().flush().unwrap();
        
        tokio::spawn(async move {
            let _ = llm_clone.chat_streaming(&llm_messages, ChatOptions::default(), tx).await;
        });
        
        let mut response_text = String::new();
        while let Some(chunk) = rx.recv().await {
            print!("{}", chunk);
            io::stdout().flush().unwrap();
            response_text.push_str(&chunk);
        }
        println!();
        
        // Save assistant content
        {
            let conn = pool.lock().unwrap();
            let _ = DbService::insert_message(&conn, session_id, "assistant", &response_text, Some(llm.name()), None, serde_json::json!({}));
        }
    }
}
