use std::{collections::HashSet, env, fs::OpenOptions, io::{BufReader, Write}, path::Path};
use tokio::fs;
use serde::{Serialize, Deserialize};
use serde_json::{self, Error};


pub fn blocked_sites_path() -> String {
  if is_debug() {
      // En desarrollo, usar archivo local
      "blocked_sites.json".to_string()
  } else if let Some(proj_dir) = dirs::config_dir() {
      // En producción, usar ruta del sistema
      let full_path = proj_dir.join("global_pomodoro");
      std::fs::create_dir_all(&full_path).ok(); // asegurarse de que el dir exista
      full_path.join("blocked_sites.json").to_string_lossy().to_string()
  } else {
      // Fallback
      "blocked_sites.json".to_string()
  }
}

fn is_debug() -> bool {
  cfg!(debug_assertions) || env::var("DEV_MODE").is_ok()
}

pub async fn read_urls() -> HashSet<String> {
  if !Path::new(&blocked_sites_path()).exists() {
      return HashSet::new();
  }

  let content = fs::read_to_string(&blocked_sites_path()).await.unwrap_or_default();
  serde_json::from_str(&content).unwrap_or_default()
}

pub async fn save_urls(urls: &HashSet<String>) {
  let list: Vec<&String> = urls.iter().collect();
  let json = serde_json::to_string_pretty(&list).unwrap();
  fs::write(&blocked_sites_path(), json).await.unwrap();
}

pub async fn add_url(new_url: &str) {
  let mut urls = read_urls().await;
  if urls.insert(new_url.to_string()) {
      save_urls(&urls).await;
      println!("✅ URL añadida: {}", new_url);
  } else {
      println!("⚠️  La URL ya existe: {}", new_url);
  }
}

pub async fn remove_url(target: &str) {
  let mut urls = read_urls().await;
  if urls.remove(target) {
      save_urls(&urls).await;
      println!("🗑️  URL eliminada: {}", target);
  } else {
      println!("❌ URL no encontrada: {}", target);
  }
}

pub async fn list_urls() -> Result<Vec<String>, Box<dyn std::error::Error>> {
  let urls = read_urls().await;
  let urls_vec: Vec<String> = urls.into_iter().collect();
  Ok(urls_vec)
}
