extern crate chrono;
extern crate clap;
extern crate dirs;
extern crate rusqlite;

mod terror;

use chrono::prelude::*;
use clap::clap_app;
use rusqlite::{params, Connection, Result};
use std::fs;
use std::path::PathBuf;
use terror::*;

#[derive(Debug)]
struct Entry {
    end_time: DateTime<Local>,
    name: String,
    project_name: String,
    start_time: DateTime<Local>,
}

#[derive(Debug)]
struct Application {
    running: bool,
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

fn set_up_sqlite(conn: &Connection) -> Result<Vec<String>> {
    let create_sql = r"
        CREATE TABLE IF NOT EXISTS entries (id INTEGER PRIMARY KEY, start_time TEXT, end_time TEXT, project_name TEXT, name TEXT);
        CREATE TABLE IF NOT EXISTS settings (key TEXT, value TEXT);
        ";

    conn.execute_batch(create_sql)?;

    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?")?;
    let rows = stmt.query_and_then(["running"], |row| row.get::<_, String>(0))?;

    let mut names = Vec::new();

    for name_result in rows {
        names.push(name_result?);
    }

    Ok(names)
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

    let is_initial_setup: bool = match set_up_sqlite(&conn) {
        Ok(strings) => strings.len() == 0,
        Err(err) => true,
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

    if let Some(n) = args.value_of("NAME") {
        let current_time = Local::now();
        let ent = Entry {
            end_time: current_time,
            name: n.to_string(),
            project_name: project,
            start_time: current_time,
        };

        conn.execute(
            "INSERT INTO entries (end_time, name, project_name, start_time) VALUES (?1, ?2, ?3, ?4)",
            params![ent.end_time.to_rfc3339(), ent.name, ent.project_name, ent.start_time.to_rfc3339()],
        )?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = parse_args();
    run(args).map_err(|e| e.exit()).unwrap();
    std::process::exit(0);
}
