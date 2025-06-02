use std::collections::HashMap;

use strum::IntoEnumIterator;

use crate::{
    error::{InvmstError, InvmstResult},
    evaluate, financial, llm,
    llm::Role,
    master::Master,
};

pub static LLM_SUPPORTED_TYPES: &[&str] = &["chat"];
pub static LLM_SUPPORTED_PROTOCOLS: &[&str] = &["openai"];

pub type ChatCompletionEvent = llm::ChatCompletionEvent;
pub type ChatCompletionOptions = llm::ChatCompletionOptions;
pub type ChatCompletionStream = llm::ChatCompletionStream;
pub type ChatMessage = llm::ChatMessage;
pub type EvaluateOptions = evaluate::EvaluateOptions;
pub type Evaluation = evaluate::Evaluation;
pub type Prospect = financial::Prospect;

pub async fn evaluate(ticker: &str, options: &EvaluateOptions) -> InvmstResult<Evaluation> {
    evaluate::run(ticker, options).await
}

pub async fn llm_chat_completion(
    prompt: &str,
    system: Option<&str>,
    options: &ChatCompletionOptions,
) -> InvmstResult<ChatMessage> {
    let mut messages: Vec<ChatMessage> = vec![];

    if let Some(system) = system {
        messages.push(ChatMessage {
            role: Role::System,
            content: system.to_string(),
            reasoning: None,
        });
    }

    messages.push(ChatMessage {
        role: Role::User,
        content: prompt.to_string(),
        reasoning: None,
    });

    llm::chat_completion(&messages, options).await
}

pub async fn llm_chat_completion_stream(
    prompt: &str,
    system: Option<&str>,
    options: &ChatCompletionOptions,
) -> InvmstResult<ChatCompletionStream> {
    let mut messages: Vec<ChatMessage> = vec![];

    if let Some(system) = system {
        messages.push(ChatMessage {
            role: Role::System,
            content: system.to_string(),
            reasoning: None,
        });
    }

    messages.push(ChatMessage {
        role: Role::User,
        content: prompt.to_string(),
        reasoning: None,
    });

    llm::chat_completion_stream(&messages, options).await
}

pub async fn llm_config(
    r#type: &str,
    protocol: &str,
    options: &HashMap<String, String>,
) -> InvmstResult<()> {
    match r#type {
        "chat" => llm::config_chat(protocol, options).await,
        _ => Err(InvmstError::Invalid(
            "INVALID_LLM_TYPE",
            format!("Invalid LLM type '{type}'"),
        )),
    }
}

pub async fn masters() -> Vec<Master> {
    Master::iter().collect()
}
