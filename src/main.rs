extern crate chrono;
extern crate clap;
extern crate dirs;
extern crate rusqlite;

mod terror;

use chrono::prelude::*;
use clap::clap_app;
use rusqlite::{params, Connection, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use terror::*;

#[derive(Debug)]
struct Entry {
    end_time: String,
    name: String,
    project_name: String,
    start_time: String,
}

#[derive(Debug)]
struct Setting {
    key: String,
    value: String,
}

fn parse_args() -> clap::ArgMatches {
    let matches = clap_app!(myapp =>
        (version: "1.0")
        (author: "Tyler B. <tyler@tylerboright.com>")
        (about: "The brain gains power through noticing.  Notice how you spend your time.")
        (@arg NAME: +required "Sets the name of a task to execute")
        (@arg PROJECT: -p --project +takes_value "Sets the project of the task")
       ).get_matches();

    matches
}

fn set_up_sqlite(conn: &Connection) -> Result<Vec<Setting>, TError> {
    let create_sql = r"
        CREATE TABLE IF NOT EXISTS entries (id INTEGER PRIMARY KEY, start_time TEXT, end_time TEXT, project_name TEXT, name TEXT);
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

    path.push("entries.db3");

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
            params!["running", "false"]
        )?;
    }

    println!("local time {:?}", Local::now());
    println!("local time {:?}", Utc::now());

    let project: String = match args.value_of("PROJECT") {
        Some(proj) => proj.to_string(),
        None => "default".to_string(),
    };

    // grab most recent entry from sqlite
    let mut stmt = conn.prepare("SELECT name, project_name, start_time, end_time FROM entries WHERE id = (SELECT MAX(id) FROM entries)")?;
    let entry_iter = stmt.query_map([], |row| {
        Ok(Entry {
            name: row.get(0)?,
            project_name: row.get(1)?,
            start_time: row.get(2)?,
            end_time: row.get(3)?,
        })
    })?;

    for entry in entry_iter {
        println!("{:?}", entry.unwrap());
    }

    if let Some(n) = args.value_of("NAME") {
        let running = match settings.get("running") {
            Some(val) => val,
            _ => "false",
        };

        match running {
            "true" => { 
                println!("{}", "task is running") ;
                conn.execute(
                    "UPDATE settings SET value = 'false' WHERE key = 'running'",
                    params![],
                )?;
            },
            "false" => { 
                println!("{}", "task is not running");
                conn.execute(
                    "UPDATE settings SET value = 'true' WHERE key = 'running'",
                    params![],
                )?;
            },
            _ => (),
        }

        println!("{}", running);
        let current_time = Local::now();
        let ent = Entry {
            end_time: current_time.to_rfc3339(),
            name: n.to_string(),
            project_name: project,
            start_time: current_time.to_rfc3339(),
        };

        conn.execute(
            "INSERT INTO entries (end_time, name, project_name, start_time) VALUES (?1, ?2, ?3, ?4)",
            params![ent.end_time, ent.name, ent.project_name, ent.start_time],
        )?;

        
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = parse_args();
    run(args).map_err(|e| e.exit()).unwrap();
    std::process::exit(0);
}
