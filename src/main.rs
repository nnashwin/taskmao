extern crate chrono;
extern crate clap;
extern crate dirs;
#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate rusqlite;
extern crate uuid;
mod config;
mod data;
mod display;
mod terror;
mod time;

use config::*;
use data::*;
use clap::{clap_app};
use rusqlite::{Connection, Result};
use std::{fs, io};
use std::path::PathBuf;
use terror::*;
use time::{convert_to_utc_timestr, get_current_utc_string};
use uuid::Uuid;

fn get_user_input (message: &str) -> String {
    println!("{}", message);
    let mut ret = String::new();
    io::stdin()
        .read_line(&mut ret)
        .expect("Failed to read the user's response");

    ret.trim().to_string().to_lowercase()
}

fn parse_args() -> clap::ArgMatches {
    let matches = clap_app!(taskmao =>
        (version: "1.0")
        (author: "Tyler B. <tyler@tylerboright.com>")
        (about: "Gain power through noticing.  Notice how you spend your time.")
        (@arg PROJECT: -p --project +takes_value "")
        (@arg START_TIME: -t --time +takes_value "")
        (@arg DESC: "Sets the description of a task to execute")
        (@subcommand config =>
            (about: "Sets the path of the config file")
            (@arg CONF_PATH: -s --set +takes_value "Sets the path of the config file"))
        (@subcommand end =>
            (about: "ends currently executing task")
            (@arg END_TIME: -t --time +takes_value ""))
        (@subcommand info =>
            (about: "returns info on the currently executing task or nothing at all"))
        (@subcommand list =>
            (about: "lists tasks completed / worked on today")
       )).get_matches();

    matches
}

fn run(args: clap::ArgMatches) -> TResult<()> {
    let mut path: PathBuf = match dirs::home_dir() {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(""),
    };

    path.push(".taskmao");

    let mut config = read_config(path.join("settings.toml"))?;

    // create regardless in order to ensure that the dir exists
    fs::create_dir_all(path.as_path())?;

    path.pop();

    let conn = match Connection::open(path.join(&config.get_tasks_file())) {
        Ok(conn) => conn,
        Err(e) => panic!("The sqlite connection couldn't be opened with the following error: {}", e),
    };

    match set_up_sqlite(&conn) {
        Ok(_val) => {},
        Err(err) => {
            display::custom_message(&format!("The data store for tasks could not be set up with the following error: {}", err), &mut io::stdout())?;
        },
    };

    match args.subcommand() {
        Some(("config", config_matches)) => {
            match config_matches.value_of("CONF_PATH") {
                Some(val) => { 
                    match Connection::open(val) {
                        Ok(test_conn) => {
                            let new_path = fs::canonicalize(val)?;
                            let path_str = new_path.to_str().expect("your config filepath was not valid,  please check to make sure the file exists and try again.");

                            if path_str != config.get_tasks_file() {
                                config.set_tasks_file(val.to_string())?;
                                save_config(config)?;

                                let most_recent_task = get_most_recent_task(&conn);

                                if most_recent_task.is_ok() {
                                    let mut task = most_recent_task.unwrap();
                                    let current_time = get_current_utc_string();
                                    match get_user_input(&format!("you currently are running the task: '{}'\nDo you want to add this task to you new config file? [Y\\N]", &task.description)).as_str() {
                                        "y" => {
                                            task.save_to_db(&test_conn)?;
                                            display::task_start(&task.start_time, &task.description, &mut io::stdout())?;

                                            task.end_task(current_time);
                                            task.save_to_db(&conn)?;
                                        },
                                        "n" => {
                                            task.end_task(current_time);
                                            task.save_to_db(&conn)?;
                                            display::custom_message("you have chosen to not save your task in the new database.  your task is closed in the old database and config is changed", &mut io::stdout())?;
                                        },
                                        _ => panic!("invalid response to add task question, not changing config."),
                                    }
                                }

                                display::custom_message(&format!("now using database: {}", path_str), &mut io::stdout())?;
                            } else {
                                display::custom_message("the entered config value matches the config location,  config remains unchanged.", &mut io::stdout())?;
                            }
                        },
                        Err(err) => display::custom_message(&format!("connection with new database file could not be opened. failed with the following error: {}", err), &mut io::stdout())?,
                    };
                },
                None => { 
                    let absolute_path = path.join(&config.get_tasks_file());
                    let file_path_str = absolute_path
                        .to_str()
                        .unwrap_or(config.get_tasks_file());
                    display::task_file_path(file_path_str)?;
                },
            };
        },
        Some(("end", end)) => {
            let end_time: String = match end.value_of("END_TIME") {
                Some(end_time) => {
                    convert_to_utc_timestr(end_time)?
                },
                None => get_current_utc_string(),
            };

            match get_most_recent_task(&conn) {
                Ok(mut prev_task) => { 
                    prev_task.end_task(end_time.to_string());
                    prev_task.save_to_db(&conn)?;
                    display::task_end(&prev_task.end_time, &prev_task.description)?;
                },
                Err(_err) => {
                    display::custom_message("you currently have no task running", &mut io::stdout())?;
                }
            };
        },
        Some(("info", _)) => {
            match get_most_recent_task(&conn) {
                Ok(current_task) => { 
                    display::task_info(current_task, &mut io::stdout())?;
                },
                Err(_err) => {
                    display::custom_message("you currently have no task running", &mut io::stdout())?;
                }
            };
        },
        Some(("list", _)) => {
            match get_todays_tasks(&conn) {
                Ok(tasks) => {
                   display::task_list(tasks)?;
                },
                Err(_err) => {
                    display::custom_message("you have no tasks from today", &mut io::stdout())?;
                }
            }
        },
        None => {
            let project: String = match args.value_of("PROJECT") {
                Some(proj) => proj.to_string(),
                None => "default".to_string(),
            };

            // convert from local datetime to utc string here
            // need to convert start time input to local datetime, then local datetime to utc
            // string
            let start_time: String = match args.value_of("START_TIME") {
                Some(start_time) => {
                    convert_to_utc_timestr(start_time)?
                },
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
                        Err(_err) => {
                            TaskDto {
                                end_time: start_time.clone(),
                                description: desc.to_string(),
                                project_name: project,
                                running: "false".to_string(),
                                start_time: start_time.clone(),
                                unique_id: Uuid::new_v4().to_string(),
                            }
                        }
                    };
                    match &prev_task.running[..] {
                        "true" => { 
                            prev_task.end_task(start_time);
                            prev_task.save_to_db(&conn)?;
                            new_task.save_to_db(&conn)?;

                            display::task_end(&prev_task.end_time, &prev_task.description)?;
                        },
                        _ => {
                            new_task.save_to_db(&conn)?;
                        },
                    }

                    display::task_start(&new_task.start_time, &new_task.description, &mut io::stdout())?;
                },
                None => display::custom_message("a description wasn't entered for your task.  For more help, try '--help'", &mut io::stdout())?
            }
        },
        _ => display::custom_message("try 'taskmao --help' for more information", &mut io::stdout())?,
    };

    Ok(())
}

fn main() -> Result<()> {
    let args = parse_args();
    run(args).map_err(|e| e.exit()).unwrap();
    std::process::exit(0);
}
