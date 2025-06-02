use std::str::FromStr;

use chrono::NaiveDate;
use serde::Serialize;
use serde_json::Value;

use crate::{data::stock::*, error::*, financial::Prospect};

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
        stock_info: &StockInfo,
        stock_events: &StockEvents,
        stock_metrics: &[StockFiscalMetrics],
        options: &MasterAnalyzeOptions,
    ) -> InvmstResult<MasterAnalysis> {
        match self {
            Master::BenjaminGraham => todo!(),
            Master::WarrenBuffett => {
                warren_buffett::analyze(stock_info, stock_events, stock_metrics, options).await
            }
        }
    }
}

#[derive(Debug)]
pub struct MasterAnalyzeOptions {
    pub backward_days: i64,
    pub date: Option<NaiveDate>,
}

#[derive(Debug)]
pub struct MasterAnalysis {
    pub prospect: Prospect,
    pub rating: u64,
    pub explanation: String,
}

impl MasterAnalysis {
    pub fn from_json(json_str: &str) -> InvmstResult<Self> {
        let json: Value = serde_json::from_str(json_str)?;

        let prospect_str = json["prospect"].as_str().ok_or(InvmstError::Required(
            "PROSPECT_REQUIRED",
            "Missing prospect".to_string(),
        ))?;
        let prospect = Prospect::from_str(prospect_str)?;

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
            prospect,
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
    "prospect": "Bullish" | "Bearish" | "Neutral",
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
    "prospect": "bearish",
    "rating": 20,
    "explanation": "test"
}
"#;

        match MasterAnalysis::from_json(&json_str) {
            Ok(analysis) => {
                assert_eq!(analysis.prospect, Prospect::Bearish);
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
