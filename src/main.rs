mod config;
mod server;
mod client;
mod pomodoro;
mod utils;

use std::sync::{Arc, Mutex};
use std::fs;
use std::path::PathBuf;

use crate::config::PomodoroStateConfig;
use crate::server::start_server;
use crate::client::send_command;

fn ensure_user_config_exists() {
    if let Some(config_base) = dirs::config_dir() {
        let target_dir = config_base.join("global_pomodoro");
        if !target_dir.exists() {
            println!("📁 Creando configuración de usuario en {:?}", target_dir);
            fs::create_dir_all(&target_dir).expect("No se pudo crear el directorio de configuración");

            let default_dir = PathBuf::from("./default_config");

            fs::copy(
                default_dir.join("pomodoro_config.json"),
                target_dir.join("pomodoro_config.json"),
            )
            .expect("No se pudo copiar pomodoro_config.json");

            fs::copy(
                default_dir.join("blocked_sites.json"),
                target_dir.join("blocked_sites.json"),
            )
            .expect("No se pudo copiar blocked_sites.json");

            println!("✅ Archivos de configuración copiados.");
        }
    }
}

fn main() {
    let build_id = format!("{}", chrono::Utc::now());
    println!("cargo:rustc-env=BUILD_ID={}", build_id);
    let args: Vec<String> = std::env::args().collect();

    // 🛠️ Paso previo: asegurarse de que ~/.config/global_pomodoro existe
    ensure_user_config_exists();

    // 🖥️ Servidor
    if args.len() > 1 && args[1] == "serve" {
        let config_result = PomodoroStateConfig::load_or_create();
        let config = match config_result {
            Ok(cfg) => Arc::new(Mutex::new(cfg)),
            Err(e) => {
                eprintln!("❌ Fallo al cargar configuración: {}", e);
                return;
            }
        };
        start_server(config).unwrap();
        return;
    }

    // 💬 Cliente
    if args.len() > 1 {
        send_command(&args[1]);
    } else {
        println!("ℹ️ Comandos: start | pause | resume | status | serve");
    }
}
