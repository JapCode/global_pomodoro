use std::{env, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::{self, Error};
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Work,
    ShortBreak,
    LongBreak,
    Idle,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PomodoroStateConfig {
    pub work_duration: u32,
    pub break_duration: u32,
    pub long_break_duration: u32,
    pub cycles: u32,
    pub current_cycle: u32,
    pub is_running: bool,
    pub long_break_interval: u32,
    pub time_left: u32,
    pub current_phase: Phase,
}

pub fn config_path() -> String {
    if is_debug() {
        "pomodoro_config.json".to_string()
    } else if let Some(proj_dir) = dirs::config_dir() {
        let full_path = proj_dir.join("global_pomodoro");
        std::fs::create_dir_all(&full_path).ok();
        full_path.join("pomodoro_config.json").to_string_lossy().to_string()
    } else {
        "pomodoro_config.json".to_string()
    }
}

fn is_debug() -> bool {
    cfg!(debug_assertions) || env::var("DEV_MODE").is_ok()
}

impl PomodoroStateConfig {
    pub fn new() -> Self {
        Self {
            work_duration: 25 * 60,
            break_duration: 5 * 60,
            long_break_duration: 10 * 60,
            cycles: 4,
            current_cycle: 0,
            is_running: true,
            long_break_interval: 2,
            time_left: 25 * 60,
            current_phase: Phase::Work,
        }
    }

    pub async fn load_or_create() -> Result<Self, Error> {
        let path = config_path();
        if Path::new(&path).exists() {
            let file = OpenOptions::new().read(true).open(&path).await.map_err(serde_json::Error::io)?;
            let mut reader = BufReader::new(file);
            let mut contents = String::new();
            reader.read_to_string(&mut contents).await.map_err(serde_json::Error::io)?;
            let config = serde_json::from_str(&contents)?;
            Ok(config)
        } else {
            let config = PomodoroStateConfig::new();
            config.save_config().await?;
            Ok(config)
        }
    }

    pub async fn save_config(&self) -> Result<(), Error> {
        let json = serde_json::to_string_pretty(self)?;
        let path = config_path();
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .await
            .map_err(serde_json::Error::io)?;
        file.write_all(json.as_bytes()).await.map_err(serde_json::Error::io)?;
        Ok(())
    }

    pub async fn reset_mut(&mut self) -> Result<(), Error> {
        self.current_cycle = 0;
        self.current_phase = Phase::Work;
        self.time_left = self.work_duration;
        self.is_running = false;
        self.save_config().await
    }

    pub async fn reset(&mut self) -> Result<(), Error> {
        *self = PomodoroStateConfig::new();
        self.save_config().await
    }
}
