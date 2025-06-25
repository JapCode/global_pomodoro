use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use futures_util::stream::SplitSink;
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::{client, Message};
use tokio_tungstenite::{accept_async, WebSocketStream};
use futures_util::{StreamExt, SinkExt};

use crate::blocked_sites::{add_url, list_urls, remove_url};
use crate::config::{self, config_path, PomodoroStateConfig};
use crate::pomodoro::PomodoroHandle;
use crate::utils::play_sound;
use tokio::task;
use tokio::sync::Mutex as AsyncMutex;
// use std::collections::HashSet;

type Client = Arc<AsyncMutex<SplitSink<WebSocketStream<TcpStream>, Message>>>;
type Clients = Arc<AsyncMutex<HashMap<String, Client>>>;

#[derive(Debug, Serialize, Deserialize)]
struct StatusWithUrls {
    #[serde(flatten)]
    config: PomodoroStateConfig,
    blocked_urls: Vec<String>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct StatusWithOptionalUrls {
    #[serde(flatten)]
    pub config: PomodoroStateConfig,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_urls: Option<Vec<String>>,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ResponsePayload {
    Message(String),
    Status(StatusWithOptionalUrls),
    Error(String),
    Help(String),
    List(Vec<String>),
}



pub async fn start_server(config: Arc<AsyncMutex<PomodoroStateConfig>>) -> Result<(), Box<dyn std::error::Error>> {
    let pomodoro_handle = Arc::new(AsyncMutex::new(PomodoroHandle::new()));
    let clients: Clients = Arc::new(AsyncMutex::new(HashMap::new()));



        // 🔁 Lanzar task que emite actualizaciones cada segundo
        let clients_clone = Arc::clone(&clients);
        let config_clone = Arc::clone(&config);
        tokio::spawn(async move {
            loop {
                // let config_json = {
                //     serde_json::to_string(&*config).unwrap()
                // };
                let config = config_clone.lock().await;
                let blocked_urls = match list_urls().await {
                    Ok(urls) => urls,
                    Err(e) => {
                        eprintln!("❌ Error al listar URLs: {}", e);
                        vec![]
                    }
                };

                let status_with_optional_urls = StatusWithOptionalUrls {
                    config: config.clone(),
                    blocked_urls: Some(blocked_urls),
                };

                let config_msg = ResponsePayload::Status(status_with_optional_urls);
                let config_json = serde_json::to_string(&config_msg).unwrap();

                let mut to_remove = vec![];

                {
                    let clients = clients_clone.lock().await;
                    for (id, client_mutex) in clients.iter() {
                        let mut ws = client_mutex.lock().await;
                        if let Err(e) = ws.send(config_json.clone().into()).await {
                            eprintln!("❌ Error al enviar mensaje a {}: {}", id, e);
                            to_remove.push(id.clone());
                        }
                    }
                }

                if !to_remove.is_empty() {
                    let mut clients = clients_clone.lock().await;
                    for id in to_remove {
                        clients.remove(&id);
                        println!("👋 Cliente {} eliminado por desconexión.", id);
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });


    // 🖧 WebSocket server loop
    let listener = TcpListener::bind("127.0.0.1:9001").await?;
    println!("🖧 WebSocket Pomodoro server on ws://127.0.0.1:9001");

    loop {
        let (stream, _) = listener.accept().await?;
        let config = Arc::clone(&config);
        let pomodoro_handle = Arc::clone(&pomodoro_handle);
        let clients = Arc::clone(&clients);
        task::spawn(async move {
            if let Err(e) = handle_connection(stream, config, pomodoro_handle, clients).await {
                eprintln!("❌ Error en conexión: {}", e);
            }
        });
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "command", rename_all = "lowercase")]
enum ClientCommand {
    Start,
    Pause,
    Resume,
    Status,
    MyConfig,
    ResetProgress,
    ResetConfig,
    Test,
    Help,
    Block { url: String },
    Unblock { url: String },
    UpdateConfig { new_config: PomodoroStateConfig },
    ListBlocked,
}

async fn handle_connection(
    stream: TcpStream,
    config: Arc<AsyncMutex<PomodoroStateConfig>>,
    pomodoro_handle: Arc<AsyncMutex<PomodoroHandle>>,
    clients: Clients,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    println!("🔌 Cliente WebSocket conectado");

    let (write, mut read) = ws_stream.split();

    let client_id = format!("Client-{}", clients.lock().await.len() + 1);
    let client = Arc::new(AsyncMutex::new(write));
    clients.lock().await.insert(client_id.clone(), client.clone());

    while let Some(msg) = read.next().await {
        let msg = msg?;
        if msg.is_text() {
            let input = msg.to_text()?.trim();
    
            let command_result: Result<ClientCommand, _> = serde_json::from_str(input);
            let response: ResponsePayload = match command_result {
                Ok(command) => {
                    let mut handle = pomodoro_handle.lock().await;
                    match command {
                        ClientCommand::Start => {
                            handle.start(config.clone());
                            ResponsePayload::Message("✅ Timer started".into())
                        }
                        ClientCommand::Pause => {
                            handle.pause(config.clone()).await;
                            ResponsePayload::Message("⏸ Timer paused".into())
                        }
                        ClientCommand::Resume => {
                            handle.resume(config.clone()).await;
                            ResponsePayload::Message("▶️ Timer resumed".into())
                        }
                        ClientCommand::Status => {
                            let cfg = config.lock().await.clone();
                            let status = StatusWithOptionalUrls {
                                config: cfg,
                                blocked_urls: None,
                            };
                            ResponsePayload::Status(status)
                        }
                        ClientCommand::MyConfig => {
                            let path = config_path();
                            if std::path::Path::new(&path).exists() {
                                ResponsePayload::Message(format!("🗂 Config file found at: {}", path))
                            } else {
                                ResponsePayload::Error("❌ Config file not found".into())
                            }
                        }
                        ClientCommand::ResetProgress => {
                            print!("🔄 Progress reset");
                            handle.reset_progress(config.clone()).await;
                            ResponsePayload::Message("🔄 Progress reset".into())
                        }
                        ClientCommand::ResetConfig => {
                            handle.reset(config.clone()).await;
                            ResponsePayload::Message("🔄 Config set to default".into())
                        }
                        ClientCommand::Test => {
                            play_sound("kuru-kuru-herta-made-with-Voicemod.mp3");
                            ResponsePayload::Message("🔊 Test sound played".into())
                        }
                        ClientCommand::Help => {
                            ResponsePayload::Help(
                                r#"
    🆘 Available Commands:
    { "command": "start" }               Start the Pomodoro timer
    { "command": "pause" }               Pause the timer
    { "command": "resume" }              Resume the timer
    { "command": "status" }              Show current timer status
    { "command": "myconfig" }            Show config file location
    { "command": "reset_progress" }      Reset progress
    { "command": "reset_config" }        Reset entire config
    { "command": "test" }                Play test sound
    { "command": "block", "url": "..." } Block a website
    { "command": "help" }                Show this help message
    "#
                                .into(),
                            )
                        }
                        ClientCommand::Block { url } => {
                            add_url(&url).await;
                            ResponsePayload::Message(format!("🔒 URL {} add to the blocked file", url))
                        },
                        ClientCommand::Unblock { url } => {
                            remove_url(&url).await;
                            ResponsePayload::Message(format!("🔓 URL {} removed from the blocked file", url))
                        }
                        ClientCommand::ListBlocked => {
                            match list_urls().await {
                                Ok(blocked_list) => ResponsePayload::List(blocked_list),
                                Err(e) => ResponsePayload::Error(format!("❌ Failed to list blocked URLs: {}", e)),
                            }
                        },
                        ClientCommand::UpdateConfig { new_config } => {
                            let mut cfg = config.lock().await;
                            *cfg = new_config.clone();
                            cfg.save_config().await.unwrap();
                            ResponsePayload::Message("✅ Config updated".into())
                        }
                    }
                }
                Err(e) => ResponsePayload::Error(format!("❌ Comando inválido: {}", e)),
            };

            let response_text = serde_json::to_string(&response)?;
            client.lock().await.send(Message::Text(response_text.into())).await?;
        }
    }

    Ok(())
}
