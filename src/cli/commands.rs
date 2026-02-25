use clap::{Parser, Subcommand};
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "jacox", version, about = "Jacox LLM Chat Server", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Override the config file path globally
    #[arg(short, long, global = true, default_value = "config.yaml")]
    pub config: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the HTTP API and WebSocket server
    Serve,
    
    /// Enter interactive CLI chat REPL mode
    Chat {
        /// The UUID of the session to connect to
        #[arg(short, long)]
        session: Uuid,
    },
    
    /// Manage Jacox chat sessions
    Session {
        #[command(subcommand)]
        action: SessionAction,
    }
}

#[derive(Subcommand)]
pub enum SessionAction {
    /// Create a new session
    Create {
        #[arg(short, long)]
        name: String,
    },
    
    /// List all sessions
    List,
    
    /// Delete a session
    Delete {
        id: Uuid,
    },
    
    /// Export a session to a .txt file
    Export {
        /// The UUID of the session to export
        id: Uuid,
        /// The path to the output file (optional)
        #[arg(short, long)]
        path: Option<String>,
    },
    
    /// Import a session from a .txt file
    Import {
        /// The path to the .txt file to import
        #[arg(short, long)]
        path: String,
    }
}
