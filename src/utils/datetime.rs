use chrono::NaiveDate;

pub fn date_from_days_after_epoch(days: i32) -> Option<NaiveDate> {
    NaiveDate::from_num_days_from_ce_opt(719163 + days)
}
