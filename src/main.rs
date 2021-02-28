extern crate chrono;
extern crate clap;
extern crate dirs;
extern crate rusqlite;

mod config;
mod data;
mod display;
mod terror;

use chrono::prelude::*;
use config::*;
use data::*;
use clap::{clap_app};
use rusqlite::{Connection, Result};
use std::fs;
use std::path::PathBuf;
use terror::*;

fn get_current_time_str() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn parse_args() -> clap::ArgMatches {
    let matches = clap_app!(taskmao =>
        (version: "1.0")
        (author: "Tyler B. <tyler@tylerboright.com>")
        (about: "Gain power through noticing.  Notice how you spend your time.")
        (@arg PROJECT: -p --project +takes_value "")
        (@arg DESC: "Sets the description of a task to execute")
        (@subcommand config =>
            (about: "Sets the path of the config file")
            (@arg CONF_PATH: -s --set +takes_value "Sets the path of the config file"))
        (@subcommand end =>
            (about: "ends currently executing task"))
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
            println!("The data store for tasks could not be set up with the following error: {}", err);
        },
    };

    match args.subcommand() {
        Some(("config", config_matches)) => {
            println!("{}", "within config");
            match config_matches.value_of("CONF_PATH") {
                Some(val) => { 
                    match Connection::open(val) {
                        Ok(test_conn) => {
                            match test_sqlite_file(&test_conn) {
                                Ok(()) => {
                                    config.set_tasks_file(val.to_string())?;
                                    save_config(config)?;
                                },
                                Err(_err) => {
                                    display::custom_message("the new config file path has an issue and can't be set.  Add a valid file path and try again");
                                },
                            };
                        },
                        Err(err) => println!("{}", err),
                    };
                },
                None => { 
                    let absolute_path = path.join(&config.get_tasks_file());
                    let file_path_str = absolute_path.to_str();
                    display::task_file_path(file_path_str.unwrap_or(config.get_tasks_file()));
                },
            };
        },
        Some(("end", _)) => { 
            let current_time = get_current_time_str();

            match get_most_recent_task(&conn) {
                Ok(mut prev_task) => { 
                    prev_task.end_task(current_time.to_string());
                    prev_task.save_to_db(&conn)?;
                    display::task_end(&prev_task.end_time, &prev_task.description)?;
                },
                Err(_err) => {
                    display::custom_message("you currently have no task running");
                }
            };
        },
        Some(("info", _)) => {
            match get_most_recent_task(&conn) {
                Ok(current_task) => { 
                    display::task_info(&current_task.start_time, &current_task.description)?;
                },
                Err(_err) => {
                    display::custom_message("you currently have no task running");
                }
            };
        },
        Some(("list", _)) => {
            match get_todays_tasks(&conn) {
                Ok(tasks) => {
                   display::task_list(tasks)?;
                },
                Err(_err) => {
                    display::custom_message("you have no tasks from today");
                }
            }
        },
        None => {
            let project: String = match args.value_of("PROJECT") {
                Some(proj) => proj.to_string(),
                None => "default".to_string(),
            };

            match args.value_of("DESC") {
                Some(desc) => {
                    // grab most recent entry from sqlite
                    let current_time = get_current_time_str();

                    let new_task = TaskDto {
                        end_time: current_time.clone(),
                        description: desc.to_string(),
                        project_name: project.clone(),
                        running: "true".to_string(),
                        start_time: current_time.clone(),
                    };

                    let mut prev_task = match get_most_recent_task(&conn) {
                        Ok(task) => task,
                        Err(_err) => {
                            TaskDto {
                                end_time: current_time.clone(),
                                description: desc.to_string(),
                                project_name: project,
                                running: "false".to_string(),
                                start_time: current_time.clone(),
                            }
                        }
                    };
                    match &prev_task.running[..] {
                        "true" => { 
                            prev_task.end_task(current_time);
                            prev_task.save_to_db(&conn)?;
                            new_task.save_to_db(&conn)?;

                            display::task_end(&prev_task.end_time, &prev_task.description)?;
                        },
                        _ => {
                            new_task.save_to_db(&conn)?;
                        },
                    }

                    display::task_start(&new_task.start_time, &new_task.description)?;
                },
                None => display::custom_message("a description wasn't entered for your task.  For more help, try '--help'")
            }
        },
        _ => display::custom_message("try 'taskmao --help' for more information"),
    };

    Ok(())
}

fn main() -> Result<()> {
    let args = parse_args();
    run(args).map_err(|e| e.exit()).unwrap();
    std::process::exit(0);
}
