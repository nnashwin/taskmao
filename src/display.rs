extern crate chrono;

use chrono::prelude::*;
use crate::terror::*;

fn convert_to_local_timestamp(utc_date_time: &str) -> Result<String, TError> {
    let parsed_end_time = DateTime::parse_from_rfc3339(utc_date_time)?;
    let converted_date_time = DateTime::<Local>::from(parsed_end_time);

    Ok(converted_date_time.format("%H:%M:%S").to_string())
}

pub fn task_end(task_end_timestamp: &str, task_desc: &str) -> Result<(), TError> {
    let time = convert_to_local_timestamp(task_end_timestamp)?;

    println!("taskmao: stopped running '{}' at {}", task_desc, time);

    Ok(())
}

pub fn task_info(task_start_timestamp: &str, task_desc: &str) -> Result<(), TError> {
    let time = convert_to_local_timestamp(task_start_timestamp)?;

    println!("taskmao: currently running task '{}' that started at '{}'", task_desc, time);

    Ok(())
}

pub fn task_start(task_start_timestamp: &str, task_desc: &str) -> Result<(), TError> {
    let time = convert_to_local_timestamp(task_start_timestamp)?;

    println!("taskmao: started running task '{}' at {}", task_desc, time);

    Ok(())
}

pub fn custom_message(message_to_display: &str) {
    println!("taskmao: {}", message_to_display);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_local_timestamp() {
        let timest = FixedOffset::east(0).ymd(1983, 4, 13).and_hms_milli(12, 9, 14, 274).to_rfc3339();
        assert_eq!(convert_to_local_timestamp(&timest).unwrap(), "22:09:14");
    }

    #[test]
    fn test_display_local_timestamp_error_hit() {
        let timest = "arestneasrtn";
        assert_eq!(convert_to_local_timestamp(&timest).is_err(), true);
    }
}
