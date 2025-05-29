use colored::Colorize;
use invmst::api;

#[derive(clap::Args)]
pub struct InfoCommand;

impl InfoCommand {
    pub async fn exec(&self) {
        let version = api::info::get_version().await;
        println!("Version: {}", version.cyan().bold());
    }
}
