extern crate chrono;
use chrono::prelude::*;
use chrono::{Duration};
use crate::terror::*;
use regex::Regex;
use std::io::{Error, ErrorKind};

use std::cmp;

pub fn convert_to_local_timestamp(utc_date_time: &str) -> Result<String, TError> {
    let parsed_end_time = NaiveDateTime::parse_from_str(utc_date_time, "%Y-%m-%d %H:%M:%S")?;
    let end_dt = DateTime::<Utc>::from_utc(parsed_end_time, Utc);
    let converted_date_time = DateTime::<Local>::from(end_dt);

    Ok(converted_date_time.format("%H:%M:%S").to_string())
}

pub fn convert_to_utc_timestamp(local_date_time: &str) -> Result<String, TError> {
    match is_valid_timestr(local_date_time) {
        true => {
            let now_str = Local::now().format("%H:%M:%S").to_string();
            let date_to_add = if is_time_yesterday(&now_str, &local_date_time) {

                let dt = Local::now() - Duration::days(1);
                dt.format("%Y-%m-%d ").to_string()
            } else {
                Local::now().format("%Y-%m-%d ").to_string()
            };

            let concat_str: String = (date_to_add + local_date_time);
            let parsed_local_time = NaiveDateTime::parse_from_str(&concat_str, "%Y-%m-%d %H:%M:%S")?;

            let start_dt = Local::from_local_datetime(&Local, &parsed_local_time);
            let converted_date_time = DateTime::<Utc>::from(start_dt.unwrap());

            Ok(converted_date_time.format("%Y-%m-%d %H:%M:%S").to_string())
        },
        false => {
            return Err(TError::from(Error::new(ErrorKind::Other, "time specified is an illegal timestamp, timestamp should be of the format HH:MM:SS")))
        }
    }
    
}

fn is_time_yesterday(now_time: &str, compared_time: &str) -> bool {
    let mut now_split: Vec<&str> = now_time.split(':').collect(); 
    let mut compared_split: Vec<&str> = compared_time.split(':').collect();

    if now_split.len() < compared_split.len() {
       now_split.push("00");
    }

    if compared_split.len() < now_split.len() {
        compared_split.push("00");
    }

    let mut digits_same = true;
    for time_count in 0..now_split.len() {
        let now_digit = now_split[time_count].parse::<i32>().unwrap();
        let compared_digit = compared_split[time_count].parse::<i32>().unwrap();

        if now_digit > compared_digit {
            digits_same = false;
            continue;
        } else if digits_same == true && compared_digit > now_digit {
            return true
        }
    }

    false
}

pub fn get_current_time_str() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn get_todays_date() -> String {
    let now_time = Local::now();
    now_time.format("%Y-%m-%d").to_string()
}

fn is_valid_timestr(time_str: &str) -> bool {
    lazy_static! {
        // matches all valid inputs on a 24hr clock
        static ref RE: Regex = Regex::new(r"^\b(0[1-9]|1[0-9]|2[0-3])\b:[0-5][0-9](:[0-5][0-9])?$").unwrap();
    }

    RE.is_match(time_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_local_to_utc() {
        let timest = "15:44:56";
        let utc_time = "05:44:56";
        assert_eq!(convert_to_utc_timestamp(&timest).unwrap().contains(utc_time), true);
    }

    #[test]
    fn test_convert_invalid_local_to_utc_fails() {
        let timest = "100:10:10";
        assert_eq!(convert_to_utc_timestamp(&timest).is_err(), true);
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

    #[test]
    fn test_display_local_timestamp_error_incorrect_format() {
        let timest = "009:009:009";
        assert_eq!(convert_to_local_timestamp(&timest).is_err(), true);
    }

    #[test]
    fn test_get_todays_date() {
        assert_eq!(get_todays_date(), Local::now().format("%Y-%m-%d").to_string());
    }

    #[test]
    fn test_is_time_yesterday() {
        assert_eq!(is_time_yesterday("08:08:08", "09:09:09"), true);
        assert_eq!(is_time_yesterday("08:08:08", "07:59:59"), false);
        assert_eq!(is_time_yesterday("00:01:08", "23:09:09"), true);
        assert_eq!(is_time_yesterday("01:01:08", "01:01:00"), false);
    }

    #[test]
    fn test_is_valid_timestr() {
        let valid_strs = vec!["01:01", "12:12", "23:59", "23:59:40"];
        let invalid_strs = vec!["22:80:00", "21:21:80", "111:12:12"];

        for str in &valid_strs {
            assert_eq!(is_valid_timestr(str), true);
        }

        for str in &invalid_strs {
            assert_eq!(is_valid_timestr(str), false);
        }
    }
}
