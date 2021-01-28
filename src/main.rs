extern crate chrono;
extern crate clap;
extern crate dirs;
extern crate rusqlite;

mod data;
mod terror;

use chrono::prelude::*;
use data::*;
use clap::clap_app;
use rusqlite::{params, Connection, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use terror::*;

#[derive(Debug)]
struct Task {
    end_time: DateTime<FixedOffset>,
    description: String,
    project_name: String,
    start_time: DateTime<FixedOffset>,
}

#[derive(Debug)]
struct Setting {
    key: String,
    value: String,
}

fn get_most_recent_task(conn: &Connection, current_time: &String) -> Result<TaskDto, TError> {
    let stmt = "SELECT description, project_name, start_time, end_time FROM tasks WHERE id = (SELECT MAX(id) FROM tasks)";
    let task: TaskDto = conn.query_row(stmt, [], |r| {
        Ok(TaskDto {
            description: r.get(0)?,
            project_name: r.get(1)?,
            start_time: r.get(2)?,
            end_time: current_time.clone(),
        })
    })?;

    Ok(task)
}

fn parse_args() -> clap::ArgMatches {
    let matches = clap_app!(myapp =>
        (version: "1.0")
        (author: "Tyler B. <tyler@tylerboright.com>")
        (about: "The brain gains power through noticing.  Notice how you spend your time.")
        (@arg DESC: +required "Sets the description of a task to execute")
        (@arg PROJECT: -p --project +takes_value "Sets the project of the task")
       ).get_matches();

    matches
}

fn set_up_sqlite(conn: &Connection) -> Result<Vec<Setting>, TError> {
    let create_sql = r"
        CREATE TABLE IF NOT EXISTS tasks (id INTEGER PRIMARY KEY, start_time TEXT, end_time TEXT, project_name TEXT, description TEXT);
        CREATE TABLE IF NOT EXISTS settings (key TEXT, value TEXT);
        ";

    conn.execute_batch(create_sql)?;

    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let rows = stmt.query_map([], |row| {
        Ok(Setting {
            key: row.get(0)?,
            value: row.get(1)?,
        })
    })?;

    let mut settings = Vec::new();

    for setting in rows {
        settings.push(setting.unwrap());
    }

    Ok(settings)
}

fn run(args: clap::ArgMatches) -> TResult<()> {
    //// create directory and files if they don't exist
    let mut path: PathBuf = match dirs::home_dir() {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(""),
    };

    path.push(".taskmaster");

    fs::create_dir_all(path.as_path())?;

    path.push("tasks.db3");

    let conn = match Connection::open(&path) {
        Ok(conn) => conn,
        Err(e) => panic!("The sqlite connection couldn't be opened with the following error: {}", e),
    };

    let settings = match set_up_sqlite(&conn) {
        Ok(settings) => {
            let mut map: HashMap<String, String> = HashMap::new();
            for setting in settings.iter() {
                map.insert(
                    setting.key.clone(),
                    setting.value.clone(),
                );
            }

            map
        },
        Err(err) => {
            println!("The settings for the application couldn't be parsed with the following error: {}", err);
            HashMap::new()
        },
    };

    let is_initial_setup: bool = match settings.len() {
        0 => true,
        _ => false,
    };

    if is_initial_setup {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)",
            params!["running", "false"],
        )?;
    }

    let project: String = match args.value_of("PROJECT") {
        Some(proj) => proj.to_string(),
        None => "default".to_string(),
    };

    if let Some(n) = args.value_of("DESC") {
        let running = match settings.get("running") {
            Some(val) => val,
            _ => "false",
        };

        match running {
            "true" => { 
                // grab most recent entry from sqlite
                let current_time = Local::now().to_rfc3339();
                let prev_task = get_most_recent_task(&conn, &current_time)?;
                println!("{:?}", prev_task);

                let new_task = TaskDto {
                    end_time: current_time.clone(),
                    description: n.to_string(),
                    project_name: project,
                    start_time: current_time.clone(),
                };

                prev_task.save_to_db(&conn)?;
                new_task.save_to_db(&conn)?;
            },
            "false" => { 
                let current_time = Local::now().to_rfc3339();
                let new_task = TaskDto {
                    end_time: current_time.clone(),
                    description: n.to_string(),
                    project_name: project,
                    start_time: current_time.clone(),
                };
                
                new_task.save_to_db(&conn);
                conn.execute(
                    "UPDATE settings SET value = 'true' WHERE key = 'running'",
                    params![],
                )?;
            },
            _ => (),
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = parse_args();
    run(args).map_err(|e| e.exit()).unwrap();
    std::process::exit(0);
}
