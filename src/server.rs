use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
// use crate::pomodoro::{start_timer, pause_timer, resume_timer}; // Ejemplo
use crate::config::{config_path, PomodoroStateConfig};
use crate::pomodoro::PomodoroHandle;
use crate::utils::{play_sound, set_config_interactivo};
// use crate::commands::Commands;

pub fn start_server(config: Arc<Mutex<PomodoroStateConfig>>) -> std::io::Result<()> {
  let pomodoro_handle = Arc::new(Mutex::new(PomodoroHandle::new()));

    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("🖧 Pomodoro server listening on 127.0.0.1:7878");

    for stream in listener.incoming() {
        let stream = stream?;
        let config = Arc::clone(&config);
        let pomodoro_handle = Arc::clone(&pomodoro_handle);
        std::thread::spawn(move || {
            handle_client(stream, config, pomodoro_handle);
        });
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream, config: Arc<Mutex<PomodoroStateConfig>>, pomodoro_handle: Arc<Mutex<PomodoroHandle>>) {
    let reader = BufReader::new(stream.try_clone().unwrap());

    for line in reader.lines() {
        let input = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("❌ Error reading from client: {}", e);
                break;
            }
        };

        let mut handle = pomodoro_handle.lock().unwrap();
        let response = match input.trim() {
            "start" | "s" => {
                handle.start(config.clone());
                "✅ Timer started\n".to_string()
            },
            "pause" | "p" => {
                handle.pause(config.clone());
                "⏸ Timer paused\n".to_string()
            },
            "resume" | "r" => {
                handle.resume(config.clone());
                "▶️ Timer resumed\n".to_string()
            },
            "status" => {
                let cfg = config.lock().unwrap();
                format!(
                    "🕒 Time left: {}s, Running: {}, Cycle: {}, Phase: {:?}\n",
                    cfg.time_left, cfg.is_running, cfg.current_cycle, cfg.current_phase
                )
            },
            "test" => {
                play_sound("kuru-kuru-herta-made-with-Voicemod.mp3");
                "🔊 Test sound played\n".to_string()
            },
            "my_config" => {
                let config_path_value = config_path();
                let path = Path::new(&config_path_value);
                if path.exists() {
                    format!("🗂 Config file found at: {}\n", config_path_value)
                } else {
                    "❌ Config file not found\n".to_string()
                }
            }
            "reset_progress" => {
                handle.reset_progress(config.clone());
                "🔄 Progress reset\n".to_string()
            },
            "reset_config" => {
                handle.reset(config.clone());
                "🔄 Timer reset\n".to_string()
            },
            "help" | "-h" | "--help" => {
                r#"
        🆘 Available Commands:
            start | s             Start the Pomodoro timer
            pause | p             Pause the timer
            resume | r            Resume the timer
            status                Show current timer status
            test                  Play test sound
            my_config             Show config file location
            reset_progress        Reset progress
            reset_config          Reset entire config
            help | -h | --help    Show this help message

        "#
                .to_string()
            },
            _ => "❌ Unknown command. Type 'help' for available commands. \n".to_string(),
        };


        if let Err(e) = stream.write_all(response.as_bytes()) {
            eprintln!("❌ Error writing to client: {}", e);
            break;
        }
    }

}
