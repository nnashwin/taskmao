extern crate chrono;

use crate::data::*;
use crate::terror::*;
use crate::time::{convert_to_local_timestamp, get_todays_date};

pub fn task_end(task_end_timestamp: &str, task_desc: &str) -> Result<(), TError> {
    let time = convert_to_local_timestamp(task_end_timestamp, false)?;

    println!("taskmao: stopped running '{}' at {}", task_desc, time);

    Ok(())
}

pub fn task_file_path(file_path: &str) -> Result<(), TError> {
    println!("taskmao: current task file path located at '{}'", file_path);
    Ok(())
}

pub fn task_info(task_start_timestamp: &str, task_desc: &str) -> Result<(), TError> {
    let time = convert_to_local_timestamp(task_start_timestamp, true)?;

    println!("taskmao: currently running '{}' that started at '{}'", task_desc, time);

    Ok(())
}

pub fn task_list(tasks: Vec<TaskDto>) -> Result<(), TError> {
    let task_str = if tasks.len() == 1 { "task" } else { "tasks" };
    println!("You have completed {} {} in the previous day, {}", tasks.len(), task_str, get_todays_date());
    for task in &tasks {
        let start_time = convert_to_local_timestamp(&task.start_time, true)?;
        let end_time = convert_to_local_timestamp(&task.end_time, true)?;
        if task.running == "true" {
            println!("Currently running task: {}\n    Project: {}\n    Start Time: {}\n    Task Id: {}\n", task.description, task.project_name, start_time, task.unique_id);
        } else {
            println!("Task: {}\n    Project: {}\n    Start Time: {}\n    End Time: {}\n    Task Id: {}\n", task.description, task.project_name, start_time, end_time, task.unique_id);
        }
    }

    Ok(())
}

pub fn task_start(task_start_timestamp: &str, task_desc: &str, mut writer: impl std::io::Write) -> Result<(), TError> {
    let time = convert_to_local_timestamp(task_start_timestamp, false)?;

    writeln!(writer, "taskmao: started running task '{}' at {}", task_desc, time)?;

    Ok(())
}

pub fn custom_message(message_to_display: &str, mut writer: impl std::io::Write) -> Result<(), TError> {
    writeln!(writer, "taskmao: {}", message_to_display)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_message_printout() -> Result<(), TError> {
        let mut result = Vec::new();
        let input = "this is my custom message";
        custom_message(input, &mut result)?;
        assert_eq!(result, b"taskmao: this is my custom message\n");
        Ok(())
    }

    #[test]
    fn test_task_start_errors_when_time_is_invalid() {
        let result = Vec::new();
        let input = "101010101010:10:10";
        let desc = "this is a test task";
        let res = task_start(input, desc, result);
        assert_eq!(res.is_err(), true);
    }
}
