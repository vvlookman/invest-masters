use chrono::NaiveDate;

pub fn date_from_days_after_epoch(days: i32) -> Option<NaiveDate> {
    NaiveDate::from_num_days_from_ce_opt(719163 + days)
}

pub fn days_after_epoch(date: &NaiveDate) -> Option<i32> {
    let num_days = date.signed_duration_since(EPOCH).num_days();
    let days: i32 = num_days.try_into().ok()?;

    Some(days)
}

static EPOCH: NaiveDate = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
