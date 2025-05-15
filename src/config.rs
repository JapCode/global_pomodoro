use std::{fs::OpenOptions, io::{BufReader, Write}, path::Path};

use serde::{Serialize, Deserialize};
use serde_json::{self, Error};

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
    if let Some(proj_dir) = dirs::config_dir() {
        let full_path = proj_dir.join("global_pomodoro");
        std::fs::create_dir_all(&full_path).ok(); // asegúrate de que el dir exista
        full_path.join("pomodoro_config.json").to_string_lossy().to_string()
    } else {
        "pomodoro_config.json".to_string() // fallback
    }
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
            time_left: 1 * 60,
            current_phase: Phase::Work,
        }
    }

    pub fn load_or_create() -> Result<Self, Error> {
        if Path::new(&config_path()).exists() {
            let file = OpenOptions::new().read(true).open(config_path()).map_err(serde_json::Error::io)?;
            let reader = BufReader::new(file);
            let config = serde_json::from_reader(reader)?;
            Ok(config)
        } else {
            let config = PomodoroStateConfig::new();
            config.save_config()?; // guardar por primera vez
            Ok(config)
        }
    }

    pub fn save_config(&self) -> Result<(), Error> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(config_path()).map_err(serde_json::Error::io)?;
        file.write_all(json.as_bytes()).map_err(serde_json::Error::io)?;
        Ok(())
    }

    pub fn reset_mut(&mut self) -> Result<(), Error> {
        self.current_cycle = 0;
        self.current_phase = Phase::Work;
        self.time_left = self.work_duration;
        self.is_running = false;
        self.save_config()
    }

    pub fn reset(&mut self) -> Result<(), Error> {
        *self = PomodoroStateConfig::new();
        self.save_config()
    }
}
