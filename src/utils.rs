use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::process::Command;
use std::fs;
use std::sync::{Arc, Mutex};

use crate::config::PomodoroStateConfig;



fn blocked_sites_path() -> String {
    if cfg!(debug_assertions) {
        "blocked_sites.json".to_string()
    } else {
        "/etc/global_pomodoro/blocked_sites.json".to_string()
    }
}

pub struct SiteBlocker {
    pub hosts_file: String,
}

impl SiteBlocker {
    pub fn new(hosts_file: Option<&str>) -> Self {
        Self {
            hosts_file: hosts_file.unwrap_or("/etc/hosts").to_string(),
        }
    }

    fn load_blocked_sites(&self, path: &str) -> std::io::Result<Vec<String>> {
        let data = fs::read_to_string(path)?;
        let sites: Vec<String> = serde_json::from_str(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(sites)
    }

    fn reset_service(&self) {
        if let Err(e) = Command::new("sudo")
            .arg("systemctl")
            .arg("restart")
            .arg("NetworkManager")
            .status()
        {
            eprintln!("Failed to restart NetworkManager: {}", e);
        }
    }

    pub fn block(&self) {
        let content = fs::read_to_string(&self.hosts_file).unwrap_or_default();
        let blocked_sites = self.load_blocked_sites(&blocked_sites_path())
            .expect("Failed to load blocked sites");

        for site in &blocked_sites {
            let redirect_entry = format!("0.0.0.0 {}", site);

            if !content.contains(&redirect_entry) {
                let output = Command::new("sudo")
                    .arg("sh")
                    .arg("-c")
                    .arg(format!("echo '{}' >> {}", redirect_entry, self.hosts_file))
                    .output()
                    .expect("Failed to run sudo command");

                if output.status.success() {
                    println!("Blocked domain: {}", site);
                } else {
                    eprintln!(
                        "Failed to block domain: {} (Error: {})",
                        site,
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
        }
        self.reset_service();
    }

    pub fn unblock(&self) {
        let blocked_sites = self.load_blocked_sites(&blocked_sites_path())
            .expect("Failed to load blocked sites");

        for site in &blocked_sites {
            let redirect_entry = format!("0.0.0.0 {}", site);

            let output = Command::new("sudo")
                .arg("sh")
                .arg("-c")
                .arg(format!("sed -i '/{}/d' {}", redirect_entry, self.hosts_file))
                .output()
                .expect("Failed to run sudo command");

            if output.status.success() {
                println!("Unblocked domain: {}", site);
            } else {
                eprintln!(
                    "Failed to unblock domain: {} (Error: {})",
                    site,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        self.reset_service();
    }
}


pub fn show_notification(title: &str, message: &str) {
Command::new("notify-send")
    .arg("--urgency=normal")
    .arg("--icon=appointment-soon")
    .arg(title)
    .arg(message)
    .status()
    .expect("Failed to send notification");
}

pub fn play_sound(file: &str) {
    #[cfg(debug_assertions)]
    let path = format!("src/sounds/{}", file);

    // In release mode, use the absolute path
    #[cfg(not(debug_assertions))]
    let path = format!("/usr/share/global_pomodoro/sounds/{}", file);

    if let Err(e) = Command::new("mpg123")
        .arg("-q")
        .arg(&path)
        .status()
    {
        eprintln!("❌ Failed to play sound '{}': {}", path, e);
    }
}


pub fn set_config_interactivo(
    stream: &mut TcpStream,
    config: Arc<Mutex<PomodoroStateConfig>>,
) -> &'static str {
    let mut stream_writer = match stream.try_clone() {
        Ok(writer) => writer,
        Err(_) => return "❌ Failed to clone stream writer\n",
    };
    let mut reader = match stream.try_clone().map(BufReader::new) {
        Ok(r) => r,
        Err(_) => return "❌ Failed to clone stream reader\n",
    };

    writeln!(stream_writer, "🛠 Interactive configuration:").ok();

    let mut cfg = config.lock().unwrap().clone(); // clone to edit

    fn ask_and_update(
        label: &str,
        current: u32,
        writer: &mut impl Write,
        reader: &mut impl BufRead,
    ) -> u32 {
        writeln!(
            writer,
            "{} (current: {}s) - enter new value or press ENTER to keep:",
            label, current
        )
        .ok();
        writer.flush().ok();

        let mut input = String::new();
        if reader.read_line(&mut input).is_ok() {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                current
            } else if let Ok(v) = trimmed.parse::<u32>() {
                v
            } else {
                writeln!(writer, "❌ Invalid input. Keeping previous value.").ok();
                current
            }
        } else {
            writeln!(writer, "❌ Error reading input. Keeping previous value.").ok();
            current
        }
    }

    cfg.work_duration = ask_and_update(
        "1. Work duration",
        cfg.work_duration,
        &mut stream_writer,
        &mut reader,
    );
    cfg.break_duration = ask_and_update(
        "2. Short break",
        cfg.break_duration,
        &mut stream_writer,
        &mut reader,
    );
    cfg.long_break_duration = ask_and_update(
        "3. Long break",
        cfg.long_break_duration,
        &mut stream_writer,
        &mut reader,
    );
    cfg.cycles = ask_and_update(
        "4. Number of cycles",
        cfg.cycles,
        &mut stream_writer,
        &mut reader,
    );
    cfg.long_break_interval = ask_and_update(
        "5. Long break interval",
        cfg.long_break_interval,
        &mut stream_writer,
        &mut reader,
    );

    // Save config and update shared state
    match cfg.save_config() {
        Ok(_) => {
            if let Ok(mut state) = config.lock() {
                *state = cfg;
            } else {
                writeln!(
                    stream_writer,
                    "❌ Failed to acquire lock on configuration.\n"
                )
                .ok();
            }
            writeln!(stream_writer, "✅ Configuration saved successfully.\n").ok();
        }
        Err(e) => {
            writeln!(
                stream_writer,
                "❌ Failed to save configuration: {}\n",
                e
            )
            .ok();
        }
    }

    "" // El contenido ya fue enviado manualmente al cliente
}