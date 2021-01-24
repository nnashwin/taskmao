extern crate bincode;
extern crate clap;
extern crate dirs;

use bincode::{deserialize, serialize};
use clap::clap_app;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

mod terror;

use terror::*;

#[derive(Debug, Deserialize, Serialize)]
struct Entry {
    end_time: u64,
    start_time: u64,
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Project {
    entries: Vec<Entry>,
    name: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct FileStore {
    projects: Vec<Project>,
    running: bool,
}

fn parse_args() -> clap::ArgMatches {
    let matches = clap_app!(myapp =>
        (version: "1.0")
        (author: "Tyler B. <tyler@tylerboright.com>")
        (about: "The brain gains power through noticing.  Notice how you spend your time.")
        (@arg NAME: +required "Sets the name of a task to execute")
        (@arg CATEGORY: -c --category +takes_value "Sets the category of the task")
       ).get_matches();

    matches
}

fn read(path: &str) -> Result<Vec<u8>> {
    let data: Vec<u8> = fs::read(path)?;
    Ok(data)
}

fn run(args: clap::ArgMatches) -> Result<()> {
    //// create directory and files if they don't exist
    let mut path: PathBuf = match dirs::home_dir() {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(""),
    };

    path.push(".taskmaster");

    fs::create_dir_all(path.as_path())?;

    path.push("entries.bin");

    let f = match read(path.to_str().unwrap_or_default()) {
        Ok(file) => file,
        Err(_err) => Vec::new(),
    };

    let file_store: FileStore = deserialize::<FileStore>(&f[..]).unwrap_or(FileStore {
        projects: vec![Project{ entries: Vec::new(), name: "default".to_string(), }],
        running: false,
    });
    println!("file string: {:?}", file_store);

    if let Some(n) = args.value_of("NAME") {
        println!("Values for name: {}", n);
        let start = SystemTime::now();
        let since_last_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards.  Can not record times");

        println!("time since last epoch: {:?}", since_last_epoch);
    }

    let category: String = match args.value_of("CATEGORY") {
        Some(cat) => cat.to_string(),
        None => "default".to_string(),
    };

    // write file
    match File::create(path.to_str().unwrap_or_default()) {
        Ok(file) => file,
        Err(why) => return Err(TError::new(ErrorKind::Io, why)),
    };

    let encoded: Vec<u8> = serialize(&file_store).unwrap();
    println!("{:?}", encoded);

    std::fs::write(path.to_str().unwrap_or_default(), encoded);

    Ok(())
}

fn main() -> std::io::Result<()> {
    let args = parse_args();
    println!("{:?}", args);
    run(args).map_err(|e| e.exit()).unwrap();
    std::process::exit(0);
}
