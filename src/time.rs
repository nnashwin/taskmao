extern crate chrono;

use chrono::prelude::*;
use crate::terror::*;

pub fn convert_to_local_timestamp(utc_date_time: &str) -> Result<String, TError> {
    let parsed_end_time = NaiveDateTime::parse_from_str(utc_date_time, "%Y-%m-%d %H:%M:%S")?;
    let end_dt = DateTime::<Utc>::from_utc(parsed_end_time, Utc);
    let converted_date_time = DateTime::<Local>::from(end_dt);

    Ok(converted_date_time.format("%H:%M:%S").to_string())
}

pub fn convert_to_utc_timestamp(local_date_time: &str) -> Result<String, TError> {
    println!("{}", local_date_time);
    let parsed_local_time = NaiveDateTime::parse_from_str(local_date_time, "%H:%M:%S")?;
    println!("{:?}", parsed_local_time);
    let start_dt = Local::from_local_datetime(&Local, &parsed_local_time);
    println!("{:?}", start_dt);
    let converted_date_time = DateTime::<Utc>::from(start_dt.unwrap());
    println!("{:?}", converted_date_time);

    Ok(converted_date_time.format("%Y-%m-%d %H:%M:%S").to_string())
}

pub fn get_todays_date() -> String {
    let now_time = Local::now();
    now_time.format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_todays_date() {
        assert_eq!(get_todays_date(), Local::now().format("%Y-%m-%d").to_string());
    }

    #[test]
    fn test_display_local_timestamp() {
        let timest = FixedOffset::east(0).ymd(1983, 4, 13).and_hms_milli(12, 9, 14, 274).format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!(convert_to_local_timestamp(&timest).unwrap(), "22:09:14");
    }

    #[test]
    fn test_display_local_timestamp_error_hit() {
        let timest = "arestneasrtn";
        assert_eq!(convert_to_local_timestamp(&timest).is_err(), true);
    }
}
