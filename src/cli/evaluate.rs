use std::path::PathBuf;

use colored::Colorize;
use invmst::{api, error::InvmstError};

#[derive(clap::Args)]
pub struct EvaluateCommand {
    #[arg(
        short = 'd',
        long = "data-dir",
        help = "Data directory, use the default data directory if not specified"
    )]
    data_dir: Option<PathBuf>,

    #[arg(
        short = 'm',
        long = "master",
        help = "Investment master, e.g. -m buffett -m graham"
    )]
    masters: Vec<String>,

    #[arg(
        short = 't',
        long = "ticker",
        help = "Ticker to evaluate, e.g. -t AAPL -t MSFT"
    )]
    tickers: Vec<String>,
}

impl EvaluateCommand {
    pub async fn exec(&self) {
        let options = api::EvaluateOptions {
            masters: self.masters.clone(),
            tickers: self.tickers.clone(),
        };

        match api::evaluate(self.data_dir.as_deref(), &options).await {
            Ok(_) => {}
            Err(err) => {
                println!("{}", err.to_string().red());

                if let InvmstError::NotExists(code, _) = err {
                    if code == "MASTER_NOT_EXISTS" {
                        println!(
                            "[i] Run `{}` command to get master list",
                            "invmst masters".green()
                        );
                    }
                }
            }
        }
    }
}
