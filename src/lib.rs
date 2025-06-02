//! # invmst lib

use std::{collections::HashMap, path::PathBuf, sync::LazyLock};

use directories::ProjectDirs;
use rayon::iter::*;

pub mod api;
pub mod error;
pub mod utils;

/// Options that each item is String in <key>:<value> format
pub struct VecOptions<'a>(pub &'a [String]);

pub async fn init() {
    env_logger::Builder::new()
        .parse_filters(std::env::var("LOG").as_deref().unwrap_or("off"))
        .init();
}

static APP_DATA_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| match ProjectDirs::from("", "", env!("CARGO_PKG_NAME")) {
        Some(proj_dirs) => proj_dirs.data_dir().to_path_buf(),
        None => std::env::current_dir()
            .expect("Unable to get current directory!")
            .join("data"),
    });

static CHANNEL_BUFFER_DEFAULT: usize = 64;
static LLM_CHAT_TEMPERATURE_DEFAULT: f64 = 0.6;

mod data;
mod ds;
mod evaluate;
mod financial;
mod llm;
mod master;
mod ticker;

impl VecOptions<'_> {
    pub fn get(&self, name: &str) -> Option<String> {
        if let Some(option_text) = self.0.par_iter().find_any(|s| {
            s.to_lowercase()
                .starts_with(&format!("{}:", name.to_lowercase()))
        }) {
            let parts: Vec<_> = option_text.splitn(2, ':').collect();
            parts.get(1).map(|s| s.trim().to_string())
        } else {
            None
        }
    }

    pub fn into_map(self) -> HashMap<String, String> {
        let mut map: HashMap<String, String> = HashMap::new();

        for option_text in self.0 {
            let parts: Vec<_> = option_text.splitn(2, ':').collect();
            if parts.len() == 2 {
                map.insert(parts[0].to_string(), parts[1].trim().to_string());
            }
        }

        map
    }

    pub fn into_tuples(self) -> Vec<(String, String)> {
        let mut tuples: Vec<(String, String)> = vec![];

        for option_text in self.0 {
            let parts: Vec<_> = option_text.splitn(2, ':').collect();
            if parts.len() == 2 {
                tuples.push((parts[0].to_string(), parts[1].trim().to_string()));
            }
        }

        tuples
    }
}
