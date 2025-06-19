use chrono::Local;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use invmst::{api, api::Prospect, error::InvmstError, utils};
use strum::EnumMessage;
use tabled::settings::{Color, Width, measurement::Percent, object::Columns, peaker::Priority};
use tokio::time::Duration;

#[derive(clap::Args)]
pub struct EvaluateCommand {
    #[arg(
        short = 'b',
        long = "backward",
        help = "Days to backward, the default value is 730"
    )]
    backward_days: Option<i64>,

    #[arg(
        short = 'd',
        long = "date",
        help = "The date to evaluate, e.g. -d 2022-01-01"
    )]
    date: Option<String>,

    #[arg(
        short = 'm',
        long = "master",
        help = "Investment master, e.g. -m buffett -m graham"
    )]
    masters: Vec<String>,

    #[arg(help = "Ticker to evaluate, e.g. 600900")]
    ticker: String,
}

impl EvaluateCommand {
    pub async fn exec(&self) {
        let backward_days = self.backward_days.unwrap_or(1100).abs();

        let date = if let Some(date_str) = &self.date {
            let parsed_date = utils::datetime::date_from_str(date_str);
            if parsed_date.is_none() {
                println!(
                    "Can not parse '{}' as date, try format like '{}'",
                    date_str.yellow(),
                    Local::now()
                        .date_naive()
                        .format("%Y-%m-%d")
                        .to_string()
                        .green()
                );
                return;
            }

            parsed_date
        } else {
            None
        };

        let options = api::EvaluateOptions {
            backward_days,
            date,
            masters: self.masters.clone(),
        };

        let spinner = ProgressBar::new_spinner();
        spinner
            .set_style(ProgressStyle::with_template("{msg} {spinner:.cyan} [{elapsed}]").unwrap());
        spinner.enable_steady_tick(Duration::from_millis(100));

        match api::evaluate(&self.ticker, &options).await {
            Ok(evaluation) => {
                spinner.finish_with_message(format!("[{}]", self.ticker.cyan()));

                let mut table_data: Vec<Vec<String>> = vec![];
                for (master, master_analysis) in evaluation.master_analyses {
                    let prospect_symbol = match master_analysis.prospect {
                        Prospect::Bullish => "↑",
                        Prospect::Bearish => "↓",
                        Prospect::Neutral => "-",
                    };
                    let prospect = format!("{prospect_symbol} ({})", master_analysis.rating);

                    table_data.push(vec![
                        master.get_message().unwrap_or_default().to_string(),
                        prospect.to_string(),
                        master_analysis.explanation.to_string(),
                    ]);
                }

                let mut table = tabled::builder::Builder::from_iter(&table_data).build();
                table.modify(Columns::first(), Color::FG_CYAN);
                table.with((
                    Width::wrap(Percent(30)).priority(Priority::max(true)),
                    Width::increase(Percent(30)).priority(Priority::min(true)),
                ));
                println!("{table}");
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
