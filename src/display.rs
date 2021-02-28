extern crate chrono;

use chrono::prelude::*;
use chrono::{Local};
use crate::data::*;
use crate::terror::*;

fn convert_to_local_timestamp(utc_date_time: &str) -> Result<String, TError> {
    let parsed_end_time = NaiveDateTime::parse_from_str(utc_date_time, "%Y-%m-%d %H:%M:%S")?;
    let end_dt = DateTime::<Utc>::from_utc(parsed_end_time, Utc);
    let converted_date_time = DateTime::<Local>::from(end_dt);

    Ok(converted_date_time.format("%H:%M:%S").to_string())
}

pub fn todays_date() -> String {
    let now_time = Local::now();
    now_time.format("%Y-%m-%d").to_string()
}

pub fn task_end(task_end_timestamp: &str, task_desc: &str) -> Result<(), TError> {
    let time = convert_to_local_timestamp(task_end_timestamp)?;

    println!("taskmao: stopped running '{}' at {}", task_desc, time);

    Ok(())
}

pub fn task_file_path(file_path: &str) -> Result<(), TError> {
    println!("taskmao: current task file path located at '{}'", file_path);
    Ok(())
}

pub fn task_info(task_start_timestamp: &str, task_desc: &str) -> Result<(), TError> {
    let time = convert_to_local_timestamp(task_start_timestamp)?;

    println!("taskmao: currently running '{}' that started at '{}'", task_desc, time);

    Ok(())
}

pub fn task_list(tasks: Vec<TaskDto>) -> Result<(), TError> {
    let task_str = if tasks.len() == 1 { "task" } else { "tasks" };
    println!("You have completed {} {} in the previous day, {}", tasks.len(), task_str, todays_date());
    for task in &tasks {
        let start_time = convert_to_local_timestamp(&task.start_time)?;
        let end_time = convert_to_local_timestamp(&task.end_time)?;
        if task.running == "true" {
            println!("Currently running task: {}\n    Project: {}\n    Start Time: {}\n", task.description, task.project_name, start_time);
        } else {
            println!("Task: {}\n    Project: {}\n    Start Time: {}\n    End Time: {}\n", task.description, task.project_name, start_time, end_time);
        }
    }

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
        let timest = FixedOffset::east(0).ymd(1983, 4, 13).and_hms_milli(12, 9, 14, 274).format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!(convert_to_local_timestamp(&timest).unwrap(), "22:09:14");
    }

    #[test]
    fn test_display_local_timestamp_error_hit() {
        let timest = "arestneasrtn";
        assert_eq!(convert_to_local_timestamp(&timest).is_err(), true);
    }
}
