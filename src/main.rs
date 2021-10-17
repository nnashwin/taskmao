extern crate chrono;
extern crate clap;
extern crate dirs;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate rusqlite;
extern crate uuid;
mod data;
mod display;
mod time;

use anyhow::{anyhow};
use clap::{clap_app, crate_version, ArgMatches};
use data::*;
use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::{fs, io};
use time::{convert_to_utc_timestr, get_current_utc_string};
use uuid::Uuid;

fn parse_args() -> ArgMatches {
    let matches = clap_app!(taskmao =>
     (version: crate_version!())
     (author: "Tyler B. <tyler@tylerboright.com>")
     (about: "Gain power through noticing.  Notice how you spend your time.")
     (@arg PROJECT: -p --project +takes_value "")
     (@arg START_TIME: -t --time +takes_value "")
     (@arg DESC: "Sets the description of a task to execute")
     (@subcommand delete =>
        (about: "deletes a task that has the specified id")
        (@arg TASK_ID: +required "Sets the id of the task that is to be completed"))
     (@subcommand end =>
         (about: "ends currently executing task")
         (@arg END_TIME: -t --time +takes_value ""))
     (@subcommand find =>
        (about: "finds a previously executed task by id")
        (@arg TASK_ID: +required "Sets the id of the task that is to be found"))
     (@subcommand info =>
         (about: "returns info on the currently executing task or nothing"))
     (@subcommand list =>
         (about: "lists tasks completed / worked on today")
    ))
    .get_matches();

    matches
}

fn run(args: ArgMatches) -> Result<(), anyhow::Error> {
    let mut path: PathBuf = match dirs::home_dir() {
        Some(path) => path,
        None => PathBuf::from(""),
    };

    path.push(".config");
    path.push("taskmao");

    // create regardless in order to ensure that the dir exists
    fs::create_dir_all(path.as_path())?;

    let conn = match Connection::open(path.join("base.sql3")) {
        Ok(conn) => conn,
        Err(e) => panic!(
            "The sqlite connection couldn't be opened with the following error: {}",
            e
        ),
    };

    match set_up_sqlite(&conn) {
        Ok(_val) => {}
        Err(err) => {
            display::custom_message(
                &format!(
                    "The data store for tasks could not be set up with the following error: {}",
                    err
                ),
                &mut io::stdout(),
            )?;
        }
    };

    match args.subcommand() {
        Some(("delete", delete)) => {
            let task_id = delete.value_of("TASK_ID")
                .ok_or(anyhow!("A task id was not entered for the delete command.  Enter a valid task id and try again."))?;

            let task_to_delete = get_tasks_start_with(&conn, task_id)?;

            if task_to_delete.is_empty() {
                display::custom_message(
                    "there were no tasks found with your id. Check your id and try again.",
                    &mut io::stdout(),
                )?;
                return Ok(());
            }

            
                
        }
        Some(("end", end)) => {
            let end_time: String = match end.value_of("END_TIME") {
                Some(end_time) => convert_to_utc_timestr(end_time)?,
                None => get_current_utc_string(),
            };

            match get_most_recent_task(&conn) {
                Ok(mut prev_task) => {
                    prev_task.end_task(end_time.to_string());
                    prev_task.save_to_db(&conn)?;
                    display::task_end(&prev_task.end_time, &prev_task.description)?;
                }
                Err(_err) => {
                    display::custom_message(
                        "you currently have no task running",
                        &mut io::stdout(),
                    )?;
                }
            };
        }
        Some(("find", search_id)) => {
            let id = search_id.value_of("TASK_ID")
                .ok_or(anyhow!("A search id was not entered for the find command.  Enter a valid search id and try again."))?;

            match get_tasks_start_with(&conn, id) {
                Ok(tasks) => { 
                    display::task_find(tasks, id, &mut io::stdout())?;
                },
                Err(_err) => { 
                    display::custom_message(&(format!("no tasks were found for the id: {}", id)), &mut io::stdout())?;
                }
            };
        }
        Some(("info", _)) => match get_most_recent_task(&conn) {
            Ok(current_task) => {
                display::task_info(current_task, &mut io::stdout())?;
            }
            Err(_err) => {
                display::custom_message(
                    "you currently have no task running",
                    &mut io::stdout(),
                )?;
            }
        },
        Some(("list", _)) => match get_todays_tasks(&conn) {
            Ok(tasks) => {
                display::task_list(tasks, &mut io::stdout())?;
            }
            Err(_err) => {
                display::custom_message("you have no tasks from today", &mut io::stdout())?;
            }
        },
        None => {
            let project = args.value_of("PROJECT").unwrap_or("default").to_string();
            // convert from local datetime to utc string here
            // need to convert start time input to local datetime, then local datetime to utc
            // string
            let start_time: String = match args.value_of("START_TIME") {
                Some(start_time) => convert_to_utc_timestr(start_time)?,
                None => get_current_utc_string(),
            };

            match args.value_of("DESC") {
                Some(desc) => {
                    // grab most recent entry from sqlite
                    let new_task = TaskDto {
                        end_time: start_time.clone(),
                        description: desc.to_string(),
                        project_name: project.clone(),
                        running: "true".to_string(),
                        start_time: start_time.clone(),
                        unique_id: Uuid::new_v4().to_string(),
                    };

                    let mut prev_task = match get_most_recent_task(&conn) {
                        Ok(task) => task,
                        Err(_err) => TaskDto {
                            end_time: start_time.clone(),
                            description: desc.to_string(),
                            project_name: project,
                            running: "false".to_string(),
                            start_time: start_time.clone(),
                            unique_id: Uuid::new_v4().to_string(),
                        },
                    };
                    match &prev_task.running[..] {
                        "true" => {
                            prev_task.end_task(start_time);
                            prev_task.save_to_db(&conn)?;
                            new_task.save_to_db(&conn)?;

                            display::task_end(&prev_task.end_time, &prev_task.description)?;
                        }
                        _ => {
                            new_task.save_to_db(&conn)?;
                        }
                    }

                    display::task_start(
                        &new_task.start_time,
                        &new_task.description,
                        &mut io::stdout(),
                    )?;
                }
                None => display::custom_message(
                    "a description wasn't entered for your task.  For more help, try '--help'",
                    &mut io::stdout(),
                )?,
            }
        }
        _ => display::custom_message(
            "try 'taskmao --help' for more information",
            &mut io::stdout(),
        )?,
    };

    Ok(())
}

fn main() -> Result<()> {
    let args = parse_args();
    run(args).map_err(|e| format!("error code: {}", e)).unwrap();
    std::process::exit(0);
}
