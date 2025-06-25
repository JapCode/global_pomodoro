use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::{sync::Mutex as AsyncMutex, task::JoinHandle, time::{sleep, Duration}};
use std::io::{stdout, Write};

use crate::{
    config::{Phase, PomodoroStateConfig},
    utils::{play_sound, show_notification},
};

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

    pub fn start(&mut self, config: Arc<AsyncMutex<PomodoroStateConfig>>) {
        let is_running = self.is_running.clone();
        is_running.store(true, Ordering::SeqCst);

        let handle: JoinHandle<()> = tokio::spawn(async move {
            loop {
                {
                    let cfg = config.lock().await;
                    if !is_running.load(Ordering::SeqCst) || cfg.current_phase == Phase::Idle {
                        break;
                    }
                }

                run_phase(Arc::clone(&config), is_running.clone()).await;

                let cfg = config.lock().await;
                if cfg.current_phase == Phase::Idle || !cfg.is_running {
                    break;
                }
            }

            println!("🛑 Ciclo de Pomodoro detenido.");
            // ❌ NO vuelvas a llamar run_phase aquí
        });

        self.handle = Some(handle);
    }

    pub async fn pause(&mut self, config: Arc<AsyncMutex<PomodoroStateConfig>>) {
        let mut cfg = config.lock().await;
        cfg.is_running = false;
        let _ = PomodoroStateConfig::save_config(&cfg).await;
        self.is_running.store(false, Ordering::SeqCst);
        println!("⏸ Pomodoro pausado");
    }

    pub async fn resume(&mut self, config: Arc<AsyncMutex<PomodoroStateConfig>>) {
        if !self.is_running.load(Ordering::SeqCst) {
            println!("▶️ Reanudando Pomodoro...");
            self.start(config);
        }
    }

    pub async fn reset_progress(&mut self, config: Arc<AsyncMutex<PomodoroStateConfig>>) {
        let mut cfg = config.lock().await;
        if let Err(e) = cfg.reset_mut().await {
            eprintln!("❌ Error al hacer reset_mut: {}", e);
        }
    }

    pub async fn reset(&mut self, config: Arc<AsyncMutex<PomodoroStateConfig>>) {
        let mut cfg = config.lock().await;
        if let Err(e) = cfg.reset().await {
            eprintln!("❌ Error al hacer reset_mut: {}", e);
        }
    }
}

async fn run_timer(
    config: Arc<AsyncMutex<PomodoroStateConfig>>,
    duration_secs: u32,
    is_running: Arc<AtomicBool>,
) {
    {
        let mut cfg = config.lock().await;
        cfg.time_left = if cfg.time_left > 0 { cfg.time_left } else { duration_secs };
        cfg.is_running = true;
    }

    println!("⏱ Timer iniciado");

    loop {
        {
            let mut cfg = config.lock().await;
            if !cfg.is_running || !is_running.load(Ordering::SeqCst) || cfg.time_left == 0 {
                break;
            }

            cfg.time_left -= 1;
            let _ = PomodoroStateConfig::save_config(&cfg).await;
            print!("\r⏳ Tiempo restante: {} segundos", cfg.time_left);
            stdout().flush().unwrap();
        }

        sleep(Duration::from_secs(1)).await;
    }

    {
        let cfg = config.lock().await;
        if cfg.time_left == 0 {
            println!("\n✅ Fase terminada");
        } else {
            println!("\n⏸ Timer pausado en {}s", cfg.time_left);
        }
    }
}

async fn run_phase(config: Arc<AsyncMutex<PomodoroStateConfig>>, is_running: Arc<AtomicBool>) {
    let phase = {
        config.lock().await.current_phase
    };

    match phase {
        Phase::Work => {
            println!("🔨 Trabajando...");
            show_notification("🔨 Trabajando...", "Pomodoro en curso");
            play_sound("kuru-ring-herta-made-with-Voicemod.mp3");
            let duration = config.lock().await.work_duration;
            run_timer(Arc::clone(&config), duration, is_running).await;
        }
        Phase::ShortBreak => {
            println!("☕ Pausa corta...");
            show_notification("☕ Pausa corta...", "Tómate un descanso corto");
            play_sound("kuru-kuru-herta-made-with-Voicemod.mp3");
            let duration = config.lock().await.break_duration;
            run_timer(Arc::clone(&config), duration, is_running).await;
        }
        Phase::LongBreak => {
            println!("🛌 Pausa larga...");
            show_notification("🛌 Pausa larga...", "Tómate un descanso largo");
            play_sound("kuru-kuru-herta-made-with-Voicemod.mp3");
            let duration = config.lock().await.long_break_duration;
            run_timer(Arc::clone(&config), duration, is_running).await;
        }
        Phase::Idle => {
            show_notification("🕒 Pomodoro finalizado", "Pomodoro finalizado o en espera");
            play_sound("aqua-crying-green-screen-with-crying-sounds-made-with-Voicemod.mp3");
            println!("🕒 Pomodoro finalizado o en espera.");
            return;
        }
    }

    let mut cfg = config.lock().await;
    if cfg.time_left == 0 {
        cfg.current_phase = next_phase(&mut cfg);
        cfg.time_left = 0;
        PomodoroStateConfig::save_config(&cfg).await;
    }
}

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
