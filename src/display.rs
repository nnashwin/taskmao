extern crate chrono;

use crate::data::TaskDto;
use crate::terror::TError;
use crate::time::{
    convert_to_local_timestamp, get_local_datetime, get_time_between_stamps, get_todays_date,
};

pub fn create_duration_str(duration: chrono::Duration) -> String {
    let seconds = if duration.num_seconds() < 60 {
        duration.num_seconds()
    } else {
        duration.num_seconds() % 60
    };
    let minutes = if duration.num_minutes() < 60 {
        duration.num_minutes()
    } else {
        duration.num_minutes() % 60
    };
    let hours = if duration.num_hours() < 24 {
        duration.num_hours()
    } else {
        duration.num_hours() % 24
    };

    format!(
        "{} days, {} hours, {} minutes and {} seconds",
        duration.num_days(),
        hours,
        minutes,
        seconds
    )
}

pub fn custom_message(
    message_to_display: &str,
    mut writer: impl std::io::Write,
) -> Result<(), TError> {
    writeln!(writer, "taskmao: {}", message_to_display)?;
    Ok(())
}

pub fn task_end(task_end_timestamp: &str, task_desc: &str) -> Result<(), TError> {
    let time = convert_to_local_timestamp(task_end_timestamp, false)?;

    println!("taskmao: stopped running '{}' at {}", task_desc, time);

    Ok(())
}

pub fn task_file_path(file_path: &str) -> Result<(), TError> {
    println!("taskmao: current task file path located at '{}'", file_path);
    Ok(())
}

pub fn task_info(task: TaskDto, mut writer: impl std::io::Write) -> Result<(), TError> {
    let time = convert_to_local_timestamp(&task.start_time, true)?;

    let duration = get_time_between_stamps(&time, &get_local_datetime())?;
    writeln!(
        writer,
        "taskmao: currently running '{}' that started at '{}'\n    Duration: {}",
        task.description,
        time,
        create_duration_str(duration)
    )?;

    Ok(())
}

pub fn task_list(tasks: Vec<TaskDto>) -> Result<(), TError> {
    let task_str = if tasks.len() == 1 { "task" } else { "tasks" };

    println!(
        "\nYou have completed {} {} on the previous day, {}",
        tasks.len(),
        task_str,
        get_todays_date()
    );
    for task in &tasks {
        let start_time = convert_to_local_timestamp(&task.start_time, true)?;
        let end_time = convert_to_local_timestamp(&task.end_time, true)?;
        let duration = get_time_between_stamps(&start_time, &end_time)?;
        let seconds = if duration.num_seconds() < 60 {
            duration.num_seconds()
        } else {
            duration.num_seconds() % 60
        };
        let minutes = if duration.num_minutes() < 60 {
            duration.num_minutes()
        } else {
            duration.num_minutes() % 60
        };
        let hours = if duration.num_hours() < 24 {
            duration.num_hours()
        } else {
            duration.num_hours() % 24
        };

        if task.running == "true" {
            println!(
                "Current Task: {}\n    Project: {}\n    Start Time: {}\n    Task Id: {}\n",
                task.description, task.project_name, start_time, task.unique_id
            );
        } else {
            let dur_str = format!(
                "{} days, {} hours, {} minutes and {} seconds",
                duration.num_days(),
                hours,
                minutes,
                seconds
            );
            println!("Task: {}\n    Project: {}\n    Start Time: {}\n    End Time: {}\n    Duration: {}\n    Task Id: {}\n", task.description, task.project_name, start_time, end_time, dur_str, task.unique_id);
        }
    }

    Ok(())
}

pub fn task_start(
    task_start_timestamp: &str,
    task_desc: &str,
    mut writer: impl std::io::Write,
) -> Result<(), TError> {
    let time = convert_to_local_timestamp(task_start_timestamp, false)?;

    writeln!(
        writer,
        "taskmao: started running task '{}' at {}",
        task_desc, time
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

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
    #[test]
    fn test_create_duration_str() {
        let secs_duration = Duration::seconds(59);
        let mins_duration = Duration::seconds(159);
        let hours_duration = Duration::seconds(3790);
        let days_duration = Duration::seconds(87000);

        assert_eq!(
            create_duration_str(secs_duration),
            format!("{} days, {} hours, {} minutes and {} seconds", 0, 0, 0, 59)
        );
        assert_eq!(
            create_duration_str(mins_duration),
            format!("{} days, {} hours, {} minutes and {} seconds", 0, 0, 2, 39)
        );
        assert_eq!(
            create_duration_str(hours_duration),
            format!("{} days, {} hours, {} minutes and {} seconds", 0, 1, 3, 10)
        );
        assert_eq!(
            create_duration_str(days_duration),
            format!("{} days, {} hours, {} minutes and {} seconds", 1, 0, 10, 0)
        );
    }
}
