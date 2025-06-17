use futures::StreamExt;
use serde::Serialize;
use serde_json::{Value, json};
use tokio::sync::mpsc;

use crate::{
    CHANNEL_BUFFER_DEFAULT,
    error::*,
    llm::{ChatCompletionEvent, ChatCompletionStream, provider::*},
    utils::net::join_url,
};

pub struct OpenAiProvider {
    base_url: String,
    api_key: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(base_url: &str, api_key: &str, model: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }
}

impl ChatProvider for OpenAiProvider {
    async fn chat_completion(
        &self,
        messages: &[ChatMessage],
        options: &ChatCompletionOptions,
    ) -> InvmstResult<ChatMessage> {
        let mut content = String::new();
        let mut reasoning_content = String::new();

        let mut stream = self.chat_completion_stream(messages, options).await?;
        while let Some(event) = stream.next().await {
            match event {
                ChatCompletionEvent::Content(delta) => {
                    content.push_str(&delta);
                }
                ChatCompletionEvent::ReasoningContent(delta) => {
                    reasoning_content.push_str(&delta);
                }
                ChatCompletionEvent::Error(err) => {
                    return Err(err);
                }
            }
        }

        Ok(ChatMessage {
            role: Role::Bot,
            content,
            reasoning: if reasoning_content.is_empty() {
                None
            } else {
                Some(reasoning_content)
            },
        })
    }

    async fn chat_completion_stream(
        &self,
        messages: &[ChatMessage],
        options: &ChatCompletionOptions,
    ) -> InvmstResult<ChatCompletionStream> {
        let request_url = join_url(&self.base_url, "/chat/completions")?;

        let mut messages_json_value = messages
            .iter()
            .map(chat_message_to_json_value)
            .collect::<Vec<_>>();
        if self.model.starts_with("qwen3") && !messages.is_empty() {
            if let Some(content) = messages_json_value[messages.len() - 1].get_mut("content") {
                if let Some(content_str) = content.as_str() {
                    let instruction = if options.enable_think {
                        "/think"
                    } else {
                        "/no_think"
                    };
                    *content = format!("{content_str} {instruction}").into();
                }
            }
        }

        let request_body = json!({
            "model": self.model,
            "messages": messages_json_value,
            "temperature": options.temperature,
            "stream": true,
        });

        let client = reqwest::Client::builder().build()?;

        let response = client
            .post(request_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let (sender, receiver) = mpsc::channel(CHANNEL_BUFFER_DEFAULT);

            tokio::spawn(async move {
                let mut stream = response.bytes_stream();
                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(chunk) => {
                            let chunk_str = String::from_utf8_lossy(&chunk);

                            for line in chunk_str.lines() {
                                if let Some(data) = line.strip_prefix("data: ") {
                                    if data == "[DONE]" {
                                        break;
                                    }

                                    match serde_json::from_str::<Value>(data) {
                                        Ok(json) => {
                                            if let Some(delta_content) =
                                                json["choices"][0]["delta"]["content"].as_str()
                                            {
                                                let _ = sender
                                                    .send(ChatCompletionEvent::Content(
                                                        delta_content.to_string(),
                                                    ))
                                                    .await;
                                            } else if let Some(delta_reasoning_content) =
                                                json["choices"][0]["delta"]["reasoning_content"]
                                                    .as_str()
                                            {
                                                let _ = sender
                                                    .send(ChatCompletionEvent::ReasoningContent(
                                                        delta_reasoning_content.to_string(),
                                                    ))
                                                    .await;
                                            }
                                        }
                                        Err(err) => {
                                            let _ = sender
                                                .send(ChatCompletionEvent::Error(err.into()))
                                                .await;
                                        }
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            let _ = sender.send(ChatCompletionEvent::Error(err.into())).await;
                        }
                    }
                }
            });

            Ok(ChatCompletionStream { receiver })
        } else {
            Err(InvmstError::HttpStatusError(format!(
                "{} {}",
                response.status(),
                response.text().await.ok().unwrap_or_default()
            )))
        }
    }
}

#[derive(strum::Display)]
enum OpenAiRole {
    #[strum(serialize = "user")]
    User,

    #[strum(serialize = "assistant")]
    Assistant,

    #[strum(serialize = "system")]
    System,
}

impl From<Role> for OpenAiRole {
    fn from(val: Role) -> Self {
        match val {
            Role::User => OpenAiRole::User,
            Role::Bot => OpenAiRole::Assistant,
            Role::System => OpenAiRole::System,
        }
    }
}

impl Serialize for OpenAiRole {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

fn chat_message_to_json_value(chat_message: &ChatMessage) -> Value {
    json!({
        "role": Into::<OpenAiRole>::into(chat_message.role).to_string(),
        "content": chat_message.content
    })
}
