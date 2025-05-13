use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::io::{stdout, Write};

use crate::{config::{Phase, PomodoroStateConfig}, utils::{ play_sound, show_notification, SiteBlocker}};

pub struct PomodoroHandle {
    pub is_running: Arc<AtomicBool>,
    pub handle: Option<JoinHandle<()>>,
}

impl PomodoroHandle {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    pub fn start(&mut self, config: Arc<Mutex<PomodoroStateConfig>>) {
        let is_running = self.is_running.clone();
        is_running.store(true, Ordering::SeqCst);

        let handle = thread::spawn(move || {
            loop {
                {
                    let cfg = config.lock().unwrap();
                    if !is_running.load(Ordering::SeqCst) || cfg.current_phase == Phase::Idle {
                        break;
                    }
                }

                run_phase(Arc::clone(&config), is_running.clone());

                let cfg = config.lock().unwrap();
                if cfg.current_phase == Phase::Idle || !cfg.is_running {
                    break;
                }
            }

            println!("🛑 Ciclo de Pomodoro detenido.");
            run_phase(config, is_running);
        });

        self.handle = Some(handle);
    }

    pub fn pause(&mut self, config: Arc<Mutex<PomodoroStateConfig>>) {
        self.is_running.store(false, Ordering::SeqCst);
        config.lock().unwrap().is_running = false;
        let cfg = config.lock().unwrap();
        PomodoroStateConfig::save_config(&cfg).unwrap();
        let blocker = SiteBlocker::new(None);
        SiteBlocker::unblock(&blocker);
        println!("⏸ Pomodoro pausado");
    }

    pub fn resume(&mut self, config: Arc<Mutex<PomodoroStateConfig>>) {
        if !self.is_running.load(Ordering::SeqCst) {
            let blocker = SiteBlocker::new(None);
            SiteBlocker::block(&blocker);
            println!("▶️ Reanudando Pomodoro...");
            self.start(config);
        }
    }
    pub fn reset(&mut self, config: Arc<Mutex<PomodoroStateConfig>>) {
        let mut cfg = config.lock().unwrap();
        let blocker = SiteBlocker::new(None);
        SiteBlocker::unblock(&blocker);
        let _ = PomodoroStateConfig::reset(&mut cfg);
    }
}

// Ejecuta un temporizador según la duración restante en la fase actual
fn run_timer(config: Arc<Mutex<PomodoroStateConfig>>, duration_secs: u32, is_running: Arc<AtomicBool>) {
    {
        let mut cfg = config.lock().unwrap();
        cfg.time_left = if cfg.time_left > 0 { cfg.time_left } else { duration_secs };
        cfg.is_running = true;
    }

    println!("⏱ Timer iniciado");

    loop {
        {
            let mut cfg = config.lock().unwrap();
            if !cfg.is_running || !is_running.load(Ordering::SeqCst) || cfg.time_left == 0 {
                break;
            }

            cfg.time_left -= 1;
            PomodoroStateConfig::save_config(&cfg).unwrap();
            print!("\r⏳ Tiempo restante: {} segundos", cfg.time_left);
            stdout().flush().unwrap();
        }

        thread::sleep(Duration::from_secs(1));
    }

    {
        let cfg = config.lock().unwrap();
        if cfg.time_left == 0 {
            println!("\n✅ Fase terminada");
        } else {
            println!("\n⏸ Timer pausado en {}s", cfg.time_left);
        }
    }
}

// Maneja cada fase del ciclo de Pomodoro
fn run_phase(config: Arc<Mutex<PomodoroStateConfig>>, is_running: Arc<AtomicBool>) {
    let phase;
    {
        phase = config.lock().unwrap().current_phase;
    }

    match phase {
        Phase::Work => {
            println!("🔨 Trabajando...");
            show_notification("🔨 Trabajando...", "Pomodoro en curso");
            play_sound("kuru-ring-herta-made-with-Voicemod.mp3");
            let blocker = SiteBlocker::new(None);
            SiteBlocker::block(&blocker);
            let duration = config.lock().unwrap().work_duration;
            run_timer(Arc::clone(&config), duration, is_running);
        }
        Phase::ShortBreak => {
            println!("☕ Pausa corta...");
            show_notification("☕ Pausa corta...", "Tómate un descanso corto");
            play_sound("kuru-kuru-herta-made-with-Voicemod.mp3");
            let blocker = SiteBlocker::new(None);
            SiteBlocker::unblock(&blocker);
            let duration = config.lock().unwrap().break_duration;
            run_timer(Arc::clone(&config), duration, is_running);
        }
        Phase::LongBreak => {
            println!("🛌 Pausa larga...");
            show_notification("🛌 Pausa larga...", "Tómate un descanso largo");
            let blocker = SiteBlocker::new(None);
            SiteBlocker::unblock(&blocker);
            play_sound("kuru-kuru-herta-made-with-Voicemod.mp3");
            let duration = config.lock().unwrap().long_break_duration;
            run_timer(Arc::clone(&config), duration, is_running);
        }
        Phase::Idle => {
            show_notification("🕒 Pomodoro finalizado", "Pomodoro finalizado o en espera");
            let blocker = SiteBlocker::new(None);
            SiteBlocker::unblock(&blocker);
            play_sound("aqua-crying-green-screen-with-crying-sounds-made-with-Voicemod.mp3");
            println!("🕒 Pomodoro finalizado o en espera.");
            return;
        }
    }

    let mut cfg = config.lock().unwrap();
    if cfg.time_left == 0 {
        cfg.current_phase = next_phase(&mut cfg);
        cfg.time_left = 0; // Limpiar para la próxima fase
        PomodoroStateConfig::save_config(&cfg).unwrap();
    }
}

// Determina la siguiente fase del ciclo Pomodoro
fn next_phase(cfg: &mut PomodoroStateConfig) -> Phase {
    match cfg.current_phase {
        Phase::Work => {
            cfg.current_cycle += 1;
            if cfg.current_cycle == cfg.cycles {
                return Phase::Idle;
            }
            if cfg.current_cycle % cfg.long_break_interval == 0 {
                Phase::LongBreak
            } else {
                Phase::ShortBreak
            }
        }
        Phase::ShortBreak | Phase::LongBreak => {
            if cfg.current_cycle >= cfg.cycles {
                Phase::Idle
            } else {
                Phase::Work
            }
        }
        Phase::Idle => Phase::Idle,
    }
}
