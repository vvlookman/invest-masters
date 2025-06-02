use clap::Subcommand;
use invmst::api;

mod config;
mod test;

#[derive(Subcommand)]
pub enum LlmCommand {
    #[command(about = "Configure LLM provider")]
    Config(Box<config::LlmConfigCommand>),

    #[command(about = "Test the default LLM provider")]
    Test(Box<test::LlmTestCommand>),
}

impl LlmCommand {
    pub async fn exec(&self) {
        match self {
            LlmCommand::Config(cmd) => {
                cmd.exec().await;
            }
            LlmCommand::Test(cmd) => {
                cmd.exec().await;
            }
        }
    }
}

fn is_protocol_valid(protocol: &str) -> bool {
    if api::LLM_SUPPORTED_PROTOCOLS.contains(&protocol) {
        return true;
    }

    println!(
        "Invalid protocol '{}', available values: {}",
        protocol,
        api::LLM_SUPPORTED_PROTOCOLS.join("/")
    );

    false
}

fn is_type_valid(r#type: &str) -> bool {
    if api::LLM_SUPPORTED_TYPES.contains(&r#type) {
        return true;
    }

    println!(
        "Invalid type '{}', available values: {}",
        r#type,
        api::LLM_SUPPORTED_TYPES.join("/")
    );

    false
}
