use anyhow::Context;
use chrono::Utc;
use chrono_tz::Tz;

use crate::utils::errors::McResult;

pub fn minecraft_date_string() -> McResult<String> {
    let tz_string = iana_time_zone::get_timezone().context("could not get current timezone")?;
    let tz: Tz = tz_string.parse().context("could not parse iana timezone")?;
    let date = Utc::now().with_timezone(&tz);
    Ok(date.format("%a %b %d %H:%M:%S %Z %Y").to_string())
}

pub fn filename_date_string() -> String {
    Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string()
}
