use std::fmt::Display;

use chrono::{Datelike, NaiveDate};

#[derive(Debug, strum::Display)]
pub enum Quarter {
    Q1,
    Q2,
    Q3,
    Q4,
}

pub struct FiscalQuarter {
    pub year: i32,
    pub quarter: Quarter,
}

pub fn date_from_days_after_epoch(days: i32) -> Option<NaiveDate> {
    NaiveDate::from_num_days_from_ce_opt(719163 + days)
}

pub fn days_after_epoch(date: &NaiveDate) -> Option<i32> {
    let num_days = date.signed_duration_since(EPOCH).num_days();
    let days: i32 = num_days.try_into().ok()?;

    Some(days)
}

pub fn prev_fiscal_quarter(date: &NaiveDate) -> FiscalQuarter {
    if date.month() < 4 {
        FiscalQuarter::new(date.year() - 1, Quarter::Q4)
    } else if date.month() < 7 {
        FiscalQuarter::new(date.year(), Quarter::Q1)
    } else if date.month() < 10 {
        FiscalQuarter::new(date.year(), Quarter::Q2)
    } else {
        FiscalQuarter::new(date.year(), Quarter::Q3)
    }
}

static EPOCH: NaiveDate = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();

impl FiscalQuarter {
    pub fn new(year: i32, quarter: Quarter) -> Self {
        Self { year, quarter }
    }
}

impl Display for FiscalQuarter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.year, self.quarter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quarter_to_string() {
        assert_eq!(Quarter::Q1.to_string().as_str(), "Q1");
        assert_eq!(Quarter::Q2.to_string().as_str(), "Q2");
        assert_eq!(Quarter::Q3.to_string().as_str(), "Q3");
        assert_eq!(Quarter::Q4.to_string().as_str(), "Q4");
    }

    #[test]
    fn test_fiscal_quarter_to_string() {
        assert_eq!(
            FiscalQuarter::new(2025, Quarter::Q1).to_string().as_str(),
            "2025Q1"
        );
    }
}
