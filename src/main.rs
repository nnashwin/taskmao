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
use clap::{arg, Arg, ArgAction, ArgMatches, Command};
use data::*;
use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::{fs, io};
use time::{convert_to_utc_timestr, get_current_utc_string};
use uuid::Uuid;

const DELETE_TEXT: &str = "delete";
const DESCRIPTION_TEXT: &str = "DESC";
const END_TEXT: &str = "end";
const FIND_TEXT: &str = "find";
const INFO_TEXT: &str = "info";
const LIST_TEXT: &str = "list";
const START_TIME_TEXT: &str = "START_TIME";
const PROJECT_TEXT: &str = "project";

fn parse_args() -> ArgMatches {
    let cli = Command::new("taskmao")
        .about("Gain power through noticing.  Notice how you spend your time.")
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .arg(
            Arg::new(DESCRIPTION_TEXT)
                .action(ArgAction::Set)
                .help("sets the description of a task to execute;  only occurs if a subcommand is not matched from the list")
        )
        .arg(
            Arg::new(PROJECT_TEXT)
                .short('p')
                .long("project")
                .default_value("default")
                .help("sets the project of a task")
                .action(ArgAction::Set)
        )
        .arg(
            Arg::new(START_TIME_TEXT)
                .short('t')
                .long("time")
                .help("manually set a start time for new task other than now")
                .action(ArgAction::Set)
        )
        .subcommand(
            Command::new(DELETE_TEXT)
                .about("deletes a task by its unique id")
                .arg(arg!(<TASK_ID> "sets the id of the task that is to be deleted"))
                .arg_required_else_help(true)
        )
        .subcommand(
            Command::new(END_TEXT)
                .about("ends currently executing task")
                .arg(
                    Arg::new("END_TIME")
                    .short('t')
                    .long("time")
                    .help("manually set the end time of the current task")
                )
        )
        .subcommand(
            Command::new(FIND_TEXT)
                .about("finds a previously executed task by id")
                .arg(arg!(<TASK_ID> "sets the id of the task that is to be found"))
                .arg_required_else_help(true)
        )
        .subcommand(
            Command::new(INFO_TEXT)
                .about("returns info on the currently executing task or nothing")
        )
        .subcommand(
            Command::new(LIST_TEXT)
                .about("lists tasks completed / worked on today")
        );


    return cli.get_matches()
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
        Some((DELETE_TEXT, sub_matches)) => {
            let task_id = sub_matches.get_one::<String>("TASK_ID")
                .ok_or(anyhow!("A task id was not entered for the delete command.  Enter a valid task id and try again."))?;

            let task_to_delete = match find_task_by_id(&conn, task_id) {
                Ok(task) => task,
                Err(error) if error.to_string().contains("returned no rows") => {
                    display::custom_message(
                        "there was no tasks found with your id. check your id and try again",
                        &mut io::stdout(),
                    )?;
                    return Ok(());
                },
                Err(error) => {
                    display::custom_message(&(format!("encountered the following sqlite error while trying to delete your task: {}", error.to_string())), &mut io::stdout())?;
                    return Ok(());
                }
            };

            // handle early return when the task to delete is already running
            if task_to_delete.running == "true" {
                display::custom_message(
                        "this task is currently running.  if you want to delete, end the task and try again",
                        &mut io::stdout(),
                    )?;

                return Ok(());
            }

            match delete_task_by_id(&conn, &task_to_delete.unique_id) {
                Ok(()) => {
                    display::custom_message(&(format!("deleted task with id '{}'", task_id)), &mut io::stdout())?;

                    return Ok(());
                },
                Err(error) => {
                    display::custom_message(&(format!("encountered the following sqlite error while trying to delete your task: {}", error.to_string())), &mut io::stdout())?;
                    return Ok(());
                }
            }
        }
        Some((END_TEXT, sub_matches)) => {
            let end_time: String = match sub_matches.get_one::<String>("END_TIME") {
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
        Some((FIND_TEXT, sub_matches)) => {
            let id = sub_matches.get_one::<String>("TASK_ID")
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
        Some((INFO_TEXT, _)) => match get_most_recent_task(&conn) {
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
        Some((LIST_TEXT, _)) => match get_todays_tasks(&conn) {
            Ok(tasks) => {
                display::task_list(tasks, &mut io::stdout())?;
            }
            Err(_err) => {
                display::custom_message("you have no tasks from today", &mut io::stdout())?;
            }
        },
        None => {
            let project = match args.get_one::<String>(PROJECT_TEXT) {
                Some(p) => p,
                None => "default"
            };
            // convert from local datetime to utc string here
            // need to convert start time input to local datetime, then local datetime to utc
            // string
            let start_time: String = match args.get_one::<String>(START_TIME_TEXT) {
                Some(start_time) => convert_to_utc_timestr(start_time)?,
                None => get_current_utc_string(),
            };

            match args.get_one::<String>(DESCRIPTION_TEXT) {
                Some(desc) => {
                    // grab most recent entry from sqlite
                    let new_task = TaskDto {
                        end_time: start_time.clone(),
                        description: desc.to_string(),
                        project_name: project.to_string(),
                        running: "true".to_string(),
                        start_time: start_time.clone(),
                        unique_id: Uuid::new_v4().to_string(),
                    };

                    let mut prev_task = match get_most_recent_task(&conn) {
                        Ok(task) => task,
                        Err(_err) => TaskDto {
                            end_time: start_time.clone(),
                            description: desc.to_string(),
                            project_name: project.to_string(),
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
