use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::LazyLock};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver;

use crate::{
    APP_DATA_DIR, LLM_CHAT_TEMPERATURE_DEFAULT,
    error::{InvmstError, InvmstResult},
    llm::provider::{ChatProvider, open_ai::OpenAiProvider},
};

#[derive(Debug, Default, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Protocol {
    #[default]
    OpenAI,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    protocol: Protocol,
    base_url: String,
    api_key: String,
    model: String,
}

#[derive(Debug)]
pub enum ChatCompletionEvent {
    Content(String),
    ReasoningContent(String),
    Error(InvmstError),
}

pub struct ChatCompletionOptions {
    pub enable_think: bool, // Some multi-mode-models can switch between think/nothink mode, such as qwen3
    pub temperature: f64,
}

pub struct ChatCompletionStream {
    receiver: Receiver<ChatCompletionEvent>,
}

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    pub reasoning: Option<String>,
}

#[allow(dead_code)]
#[derive(strum::Display, strum::EnumString, Copy, Clone, Debug, PartialEq)]
#[strum(ascii_case_insensitive)]
pub enum Role {
    Bot,
    User,
    System,
}

pub async fn chat_completion(
    messages: &[ChatMessage],
    options: &ChatCompletionOptions,
) -> InvmstResult<ChatMessage> {
    let cfg: Config = confy::load_path(&*CHAT_CONFIG_PATH)?;

    let provider = match cfg.protocol {
        Protocol::OpenAI => OpenAiProvider::new(&cfg.base_url, &cfg.api_key, &cfg.model),
    };

    provider.chat_completion(messages, options).await
}

pub async fn chat_completion_stream(
    messages: &[ChatMessage],
    options: &ChatCompletionOptions,
) -> InvmstResult<ChatCompletionStream> {
    let cfg: Config = confy::load_path(&*CHAT_CONFIG_PATH)?;

    let provider = match cfg.protocol {
        Protocol::OpenAI => OpenAiProvider::new(&cfg.base_url, &cfg.api_key, &cfg.model),
    };

    provider.chat_completion_stream(messages, options).await
}

pub async fn config_chat(protocol: &str, options: &HashMap<String, String>) -> InvmstResult<()> {
    let mut cfg: Config = confy::load_path(&*CHAT_CONFIG_PATH).unwrap_or(Config::default());

    cfg.protocol = Protocol::from_str(protocol)?;

    if let Some(base_url) = options.get("base_url") {
        cfg.base_url = base_url.trim().to_string();
    }

    if let Some(api_key) = options.get("api_key") {
        cfg.api_key = api_key.trim().to_string();
    }

    if let Some(model) = options.get("model") {
        cfg.model = model.trim().to_string();
    }

    if cfg.base_url.is_empty() {
        return Err(InvmstError::Required(
            "OPTION_REQUIRED",
            "Required option 'base_url' is missing".to_string(),
        ));
    }

    if cfg.api_key.is_empty() {
        return Err(InvmstError::Required(
            "OPTION_REQUIRED",
            "Required option 'api_key' is missing".to_string(),
        ));
    }

    if cfg.model.is_empty() {
        return Err(InvmstError::Required(
            "OPTION_REQUIRED",
            "Required option 'model' is missing".to_string(),
        ));
    }

    confy::store_path(&*CHAT_CONFIG_PATH, &cfg)?;

    Ok(())
}

mod provider;

static CHAT_CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| APP_DATA_DIR.join("llm-chat.toml"));

impl Default for ChatCompletionOptions {
    fn default() -> Self {
        Self {
            enable_think: false,
            temperature: LLM_CHAT_TEMPERATURE_DEFAULT,
        }
    }
}

impl ChatCompletionOptions {
    pub fn with_enable_think(mut self, enable_think: bool) -> Self {
        self.enable_think = enable_think;
        self
    }

    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = temperature;
        self
    }
}

impl ChatCompletionStream {
    pub fn new(receiver: Receiver<ChatCompletionEvent>) -> Self {
        Self { receiver }
    }

    pub fn close(&mut self) {
        self.receiver.close()
    }

    pub async fn next(&mut self) -> Option<ChatCompletionEvent> {
        self.receiver.recv().await
    }
}
