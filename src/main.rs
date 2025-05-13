mod config;
mod server;
mod client;
mod pomodoro;
mod utils;


use std::sync::{Arc, Mutex};

use crate::config::PomodoroStateConfig;
use crate::server::start_server;
use crate::client::send_command;



fn main() {
    let args: Vec<String> = std::env::args().collect();

  // servidor
    if args.len() > 1 && args[1] == "serve" {
        let config_result = PomodoroStateConfig::load_or_create();
        let config = match config_result {
            Ok(cfg) => Arc::new(Mutex::new(cfg)),
            Err(e) => {
                eprintln!("Failed to load or create PomodoroStateConfig: {}", e);
                return;
            }
        };
        start_server(config).unwrap();
        return;
    }

  // cliente
    if args.len() > 1 {
        send_command(&args[1]);
    } else {
        println!("Comandos: start | pause | resume | status | serve");
    }
}