use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use invmst::{api, error::InvmstError};
use tokio::time::Duration;

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
        help = "Ticker to evaluate, e.g. -t 600900"
    )]
    ticker: String,
}

impl EvaluateCommand {
    pub async fn exec(&self) {
        let options = api::EvaluateOptions {
            masters: self.masters.clone(),
        };

        let spinner = ProgressBar::new_spinner();
        spinner
            .set_style(ProgressStyle::with_template("{msg} {spinner:.cyan} [{elapsed}]").unwrap());
        spinner.enable_steady_tick(Duration::from_millis(100));

        match api::evaluate(&self.ticker, &options).await {
            Ok(_) => {
                spinner.finish_with_message(format!("[{}]", self.ticker.green()));
            }
            Err(err) => {
                spinner.finish_with_message(format!("[{}] {}", self.ticker, err.to_string().red()));

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
