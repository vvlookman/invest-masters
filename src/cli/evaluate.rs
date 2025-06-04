use std::path::PathBuf;

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use invmst::{api, error::InvmstError};
use tokio::time::Duration;

#[derive(clap::Args)]
pub struct EvaluateCommand {
    #[arg(
        short = 'd',
        long = "data-dir",
        help = "Data directory, use the app data directory if not specified"
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
        help = "Ticker to evaluate, e.g. -t 000333 -t 600900"
    )]
    tickers: Vec<String>,
}

impl EvaluateCommand {
    pub async fn exec(&self) {
        let options = api::EvaluateOptions {
            masters: self.masters.clone(),
            tickers: self.tickers.clone(),
        };

        let spinner = ProgressBar::new_spinner();
        spinner
            .set_style(ProgressStyle::with_template("{msg} {spinner:.cyan} [{elapsed}]").unwrap());
        spinner.enable_steady_tick(Duration::from_millis(100));

        match api::evaluate(self.data_dir.as_deref(), &options).await {
            Ok(_) => {
                spinner.finish_with_message(format!("{}", "success".green()));
            }
            Err(err) => {
                spinner.finish_with_message(format!("{}", err.to_string().red()));

                if let InvmstError::NotExists(code, _) = err {
                    if code == "MASTER_NOT_EXISTS" {
                        println!(
                            "[I] Run `{}` command to get master list",
                            "invmst masters".green()
                        );
                    }
                }
            }
        }
    }
}
