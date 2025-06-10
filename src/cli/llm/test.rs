use std::io::{Write, stdout};

use colored::Colorize;
use invmst::{
    VecOptions, api,
    api::*,
    error::{InvmstError, InvmstResult},
};

use crate::cli;

#[derive(clap::Args)]
pub struct LlmTestCommand {
    #[arg(
        short = 'L',
        long = "llm-option",
        help = "Additional option passed to LLM, e.g. -L temperature:0.6"
    )]
    llm_options: Vec<String>,

    #[arg(
        short = 't',
        long = "type",
        default_value = "chat",
        help = "LLM provider's type, the default value is chat, currently supported types: chat"
    )]
    r#type: Option<String>,

    prompt: String,
}

impl LlmTestCommand {
    pub async fn exec(&self) {
        let r#type = self.r#type.as_deref().unwrap_or(LLM_SUPPORTED_TYPES[0]);
        if !cli::llm::is_type_valid(r#type) {
            return;
        }

        let mut chat_completion_options = ChatCompletionOptions::default();
        let llm_options = VecOptions(&self.llm_options);
        if let Some(temperature_str) = llm_options.get("temperature") {
            if let Ok(temperature) = temperature_str.parse() {
                chat_completion_options = chat_completion_options.with_temperature(temperature);
            }
        }

        let prompt = self.prompt.clone();

        let result: InvmstResult<ChatCompletionStream> = match r#type {
            "chat" => {
                api::llm_chat_completion_stream(&prompt, None, &chat_completion_options).await
            }
            _ => Err(InvmstError::Invalid(
                "INVALID_LLM_TYPE",
                format!("Invalid LLM type '{type}'"),
            )),
        };

        match result {
            Ok(mut stream) => {
                let mut has_content = false;
                let mut has_reasoning_content = false;

                while let Some(event) = stream.next().await {
                    match event {
                        ChatCompletionEvent::Content(delta) => {
                            if !has_content && has_reasoning_content {
                                print!("\n\n");
                                stdout().flush().unwrap();
                            }

                            has_content = true;
                            print!("{delta}");
                            stdout().flush().unwrap();
                        }
                        ChatCompletionEvent::ReasoningContent(delta) => {
                            has_reasoning_content = true;
                            print!("{}", delta.bright_black());
                            stdout().flush().unwrap();
                        }
                        ChatCompletionEvent::Error(err) => {
                            println!("{}", err.to_string().red());
                            break;
                        }
                    }
                }

                println!();
            }
            Err(err) => {
                println!("{}", err.to_string().red());
            }
        }
    }
}
