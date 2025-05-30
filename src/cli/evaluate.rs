use colored::Colorize;
use invmst::{api, error::InvmstError};

#[derive(clap::Args)]
pub struct EvaluateCommand {
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

        match api::evaluate(&options).await {
            Ok(_) => {}
            Err(err) => {
                println!("{}", err.to_string().red());

                match err {
                    InvmstError::NotExists(_) => {
                        println!(
                            "[!] run `{}` command to get master list",
                            "invmst masters".yellow()
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}
