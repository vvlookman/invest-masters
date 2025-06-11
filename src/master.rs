use std::str::FromStr;

use serde::Serialize;
use serde_json::Value;

use crate::{data::stock::*, error::*, financial::Signal, ticker::Ticker};

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    PartialEq,
    strum::Display,
    strum::EnumIter,
    strum::EnumMessage,
    strum::EnumString,
)]
#[strum(ascii_case_insensitive)]
pub enum Master {
    #[strum(
        message = "Benjamin Graham",
        serialize = "graham",
        serialize = "ben-graham"
    )]
    BenjaminGraham,

    #[strum(
        message = "Warren Buffett",
        serialize = "buffett",
        serialize = "warren-buffett"
    )]
    WarrenBuffett,
}

impl Master {
    pub async fn analyze(
        &self,
        ticker: &Ticker,
        stock_info: &StockInfo,
        trailing_stock_metrics: &[FiscalStockMetrics],
    ) -> InvmstResult<MasterAnalysis> {
        match self {
            Master::BenjaminGraham => todo!(),
            Master::WarrenBuffett => {
                warren_buffett::analyze(ticker, stock_info, trailing_stock_metrics).await
            }
        }
    }
}

#[derive(Debug)]
pub struct MasterAnalysis {
    pub signal: Signal,
    pub rating: u64,
    pub explanation: String,
}

impl MasterAnalysis {
    pub fn from_json(json_str: &str) -> InvmstResult<Self> {
        let json: Value = serde_json::from_str(json_str)?;

        let signal_str = json["signal"].as_str().ok_or(InvmstError::Required(
            "SIGNAL_REQUIRED",
            "Missing signal".to_string(),
        ))?;
        let signal = Signal::from_str(signal_str)?;

        let rating: u64 = json["rating"].as_u64().ok_or(InvmstError::Required(
            "RATING_REQUIRED",
            "Missing rating".to_string(),
        ))?;

        let explanation = json["explanation"]
            .as_str()
            .ok_or(InvmstError::Required(
                "EXPLANATION_REQUIRED",
                "Missing explanation".to_string(),
            ))?
            .to_string();

        Ok(Self {
            signal,
            rating,
            explanation,
        })
    }
}

mod warren_buffett;

static MASTER_ANALYSIS_JSON_PROMPT: &str = r#"
返回的 JSON 格式示例如下：
```
{
    "signal": "Buy" | "Sell" | "Hold",
    "rating": 评分为0到100之间的整数,
    "explanation": "详细阐述分析过程"
}
```

注意以下几点：
- 不要包含任何额外的解释或文本，仅返回 JSON 数据。
- 确保返回的结果是合法的 JSON 格式。
"#;

#[derive(Debug, Serialize)]
struct AnalysisDraft {
    score: Option<f64>,
    assessments: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_analysis() {
        let json_str = r#"
{
    "signal": "sell",
    "rating": 20,
    "explanation": "test"
}
"#;

        match MasterAnalysis::from_json(&json_str) {
            Ok(analysis) => {
                assert_eq!(analysis.signal, Signal::Sell);
                assert_eq!(analysis.rating, 20);
                assert_eq!(analysis.explanation, "test");
            }
            Err(err) => {
                println!("{err:?}");
                assert!(false);
            }
        }
    }
}
