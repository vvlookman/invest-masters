use colored::Colorize;
use invmst::{VecOptions, api};

use crate::cli;

#[derive(clap::Args)]
pub struct LlmConfigCommand {
    #[arg(
        short = 'O',
        long = "option",
        help = "LLM provider's option, e.g. -O base_url:https://api.openai.com/v1 -O api_key:sk-xxx -O model:gpt-3.5-turbo"
    )]
    options: Vec<String>,

    #[arg(
        short = 'p',
        long = "protocol",
        help = "LLM provider's protocol, the default value is openai"
    )]
    protocol: Option<String>,

    #[arg(
        short = 't',
        long = "type",
        help = "LLM provider's type, the default value is chat"
    )]
    r#type: Option<String>,
}

impl LlmConfigCommand {
    pub async fn exec(&self) {
        let protocol = self
            .protocol
            .as_deref()
            .unwrap_or(api::LLM_SUPPORTED_PROTOCOLS[0]);
        if !cli::llm::is_protocol_valid(protocol) {
            return;
        }

        let r#type = self
            .r#type
            .as_deref()
            .unwrap_or(api::LLM_SUPPORTED_TYPES[0]);
        if !cli::llm::is_type_valid(r#type) {
            return;
        }

        let options_map = VecOptions(&self.options).into_map();

        if let Err(err) = api::llm_config(r#type, protocol, &options_map).await {
            println!("{}", err.to_string().red());
        } else {
            println!("LLM for '{type}' has been configured");
        }
    }
}
