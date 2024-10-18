use chrono::{DateTime, Datelike, Duration, NaiveDate};
use chrono_tz::Tz;

pub fn last_day_of_month(now: DateTime<Tz>) -> u32 {
    let year = now.year();
    let month = now.month();

    let first_day_of_next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    };

    let last_day_of_current_month = first_day_of_next_month - Duration::days(1);
    last_day_of_current_month.day()
}
