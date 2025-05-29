use clap::Subcommand;

mod info;
mod masters;

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Display system-wide information")]
    Info(Box<info::InfoCommand>),

    #[command(about = "Display all invest masters")]
    Masters(Box<masters::MastersCommand>),
}
