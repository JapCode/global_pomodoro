mod config;
mod server;
mod client;
mod pomodoro;
mod utils;
mod blocked_sites;

use std::sync::{Arc, Mutex};
use std::fs;
use std::path::PathBuf;

use crate::config::PomodoroStateConfig;
use crate::server::start_server;
use crate::client::send_command;
use tokio::sync::Mutex as AsyncMutex;



#[tokio::main]
async fn main() {
    let build_id = format!("{}", chrono::Utc::now());
    println!("cargo:rustc-env=BUILD_ID={}", build_id);
    let args: Vec<String> = std::env::args().collect();

    // 🖥️ Servidor
    if args.len() > 1 && args[1] == "serve" {
        let config_result = PomodoroStateConfig::load_or_create().await;
        let config = match config_result {
            Ok(cfg) => Arc::new(AsyncMutex::new(cfg)),
            Err(e) => {
                eprintln!("❌ Fallo al cargar configuración: {}", e);
                return;
            }
        };
        if let Err(e) = start_server(config).await {
            eprintln!("❌ Error al iniciar el servidor: {}", e);
        }
        return;
    }

    // 💬 Cliente
    if args.len() > 1 {
        send_command(&args[1]);
    } else {
        println!("ℹ️ Comandos: start | pause | resume | status | serve");
    }
}
