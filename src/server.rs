use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write};
use std::process::Command;
use std::sync::{Arc, Mutex};
// use crate::pomodoro::{start_timer, pause_timer, resume_timer}; // Ejemplo
use crate::config::PomodoroStateConfig;
use crate::pomodoro::PomodoroHandle;
// use crate::commands::Commands;

pub fn start_server(config: Arc<Mutex<PomodoroStateConfig>>) -> std::io::Result<()> {
  let pomodoro_handle = Arc::new(Mutex::new(PomodoroHandle::new()));

    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("🖧 Servidor Pomodoro escuchando en 127.0.0.1:7878");

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
        let input = line.unwrap();
        let mut handle = pomodoro_handle.lock().unwrap();
        let response = match input.trim() {
            "start" => {
                handle.start(config.clone());
                "✅ Timer started\n"
            },
            "pause" => {
                handle.pause(config.clone());
                "⏸ Timer paused\n"
            },
            "resume" => {
                // resume_pomodoro(Arc::clone(&config));
                handle.resume(config.clone());
                "▶️ Timer resumed\n"
            },
            "status" => &{
                let cfg = config.lock().unwrap();
                format!("🕒 Time left: {}s, Running: {}\n", cfg.time_left, cfg.is_running)
            },
            "reset" => {
                handle.reset(config.clone());
                "🔄 Timer reset\n"
            },
            _ => "❌ Comando desconocido\n",
        };

        stream.write_all(response.as_bytes()).unwrap();
    }
}
