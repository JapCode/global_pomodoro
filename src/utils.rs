use std::process::Command;
use std::fs;


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

// pub fn get_sound_directory() -> PathBuf {
//     let mut home = env::var("HOME")
//         .map(PathBuf::from)
//         .unwrap_or_else(|_| env::current_dir().unwrap());
//     home.push("PersonalProjects");
//     home.push("rust-projects");
//     home.push("global_pomodoro");
//     home.push("src");
//     home.push("sounds");
//     home
// }

// pub fn get_sound_path(filename: &str) -> PathBuf {
//     let mut path = get_sound_directory();
//     path.push(filename);
//     path
// }