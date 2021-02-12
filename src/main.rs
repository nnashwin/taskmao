extern crate chrono;
extern crate clap;
extern crate dirs;
extern crate rusqlite;

mod data;
mod terror;

use chrono::prelude::*;
use data::*;
use clap::clap_app;
use rusqlite::{Connection, Result};
use std::fs;
use std::path::PathBuf;
use terror::*;

#[derive(Debug)]
struct Task {
    end_time: DateTime<FixedOffset>,
    description: String,
    project_name: String,
    running: bool,
    start_time: DateTime<FixedOffset>,
}

fn get_most_recent_task(conn: &Connection) -> Result<TaskDto, TError> {
    let stmt = "SELECT description, project_name, running, end_time, start_time FROM tasks WHERE id = (SELECT MAX(id) FROM tasks)";
    let task: TaskDto = conn.query_row(stmt, [], |r| {
        Ok(TaskDto {
            description: r.get(0)?,
            project_name: r.get(1)?,
            running: r.get(2)?,
            end_time: r.get(3)?,
            start_time: r.get(4)?,
        })
    })?;

    Ok(task)
}

fn parse_args() -> clap::ArgMatches {
    let matches = clap_app!(taskmao =>
        (version: "1.0")
        (author: "Tyler B. <tyler@tylerboright.com>")
        (about: "Gain power through noticing.  Notice how you spend your time.")
        (@arg DESC: "Sets the description of a task to execute")
        (@arg PROJECT: -p --project +takes_value "Sets the project of the task")
        (@subcommand end =>
            (about: "ends currently executing task")
       )).get_matches();

    matches
}

fn set_up_sqlite(conn: &Connection) -> Result<()> {
    let create_sql = r"
        CREATE TABLE IF NOT EXISTS tasks (id INTEGER PRIMARY KEY, start_time TEXT, end_time TEXT, project_name TEXT, running TEXT, description TEXT);
        ";

    conn.execute_batch(create_sql)?;

    Ok(())
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
            println!("{}", "end task"); 
        },
        None => {
            let project: String = match args.value_of("PROJECT") {
                Some(proj) => proj.to_string(),
                None => "default".to_string(),
            };

            if let Some(n) = args.value_of("DESC") {
                // grab most recent entry from sqlite
                let current_time = Local::now().to_rfc3339();

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
                    },
                    _ => {
                        new_task.save_to_db(&conn)?;
                    },
                }

                println!("started running {} at {}", new_task.description, new_task.start_time);
            }

        },
        _ => println!("{}", "taskmao: try 'taskmao --help' for more information"),
    };

    Ok(())
}

fn main() -> Result<()> {
    let args = parse_args();
    run(args).map_err(|e| e.exit()).unwrap();
    std::process::exit(0);
}
