extern crate chrono;
extern crate clap;
extern crate dirs;
extern crate rusqlite;

mod data;
mod display;
mod terror;

use chrono::prelude::*;
use data::*;
use clap::clap_app;
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
        (@arg DESC: "Sets the description of a task to execute")
        (@arg PROJECT: -p --project +takes_value "Sets the project of the task")
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
    //// create directory and files if they don't exist
    let mut path: PathBuf = match dirs::home_dir() {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(""),
    };

    path.push(".taskmao");

    fs::create_dir_all(path.as_path())?;

    path.push("tasks.db3");

    let conn = match Connection::open(&path) {
        Ok(conn) => conn,
        Err(e) => panic!("The sqlite connection couldn't be opened with the following error: {}", e),
    };

    match set_up_sqlite(&conn) {
        Ok(_val) => {
        },
        Err(err) => {
            println!("The data store for tasks could not be set up with the following error: {}", err);
        },
    };

    match args.subcommand_name() {
        Some("end") => { 
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
        Some("info") => {
            match get_most_recent_task(&conn) {
                Ok(current_task) => { 
                    display::task_info(&current_task.start_time, &current_task.description)?;
                },
                Err(_err) => {
                    display::custom_message("you currently have no task running");
                }
            };
        },
        Some("list") => {
            println!("{}", "list");
            match get_todays_tasks(&conn) {
                Ok(tasks) => {
                    println!("{:?}", tasks);
                },
                Err(_err) => {
                    display::custom_message("you currently have no tasks that have ran today");
                }
            }
        },
        None => {
            let project: String = match args.value_of("PROJECT") {
                Some(proj) => proj.to_string(),
                None => "default".to_string(),
            };

            if let Some(n) = args.value_of("DESC") {
                // grab most recent entry from sqlite
                let current_time = get_current_time_str();

                let new_task = TaskDto {
                    end_time: current_time.clone(),
                    description: n.to_string(),
                    project_name: project.clone(),
                    running: "true".to_string(),
                    start_time: current_time.clone(),
                };

                let mut prev_task = match get_most_recent_task(&conn) {
                    Ok(task) => task,
                    Err(_err) => {
                        TaskDto {
                            end_time: current_time.clone(),
                            description: n.to_string(),
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
