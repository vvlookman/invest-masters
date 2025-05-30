use clap::Subcommand;

mod evaluate;
mod masters;

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Evaluate investments")]
    #[clap(visible_aliases = &["eval"])]
    Evaluate(Box<evaluate::EvaluateCommand>),

    #[command(about = "Display all investment masters")]
    Masters(Box<masters::MastersCommand>),
}
