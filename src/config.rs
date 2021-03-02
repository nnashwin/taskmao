use crate::TError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use toml;

#[derive(Deserialize, Debug, Serialize)]
pub struct Config {
    tasks_file: String,
}

impl Config {
    pub fn get_tasks_file(&self) -> &str {
        &self.tasks_file
    }

    pub fn set_tasks_file(&mut self, new_tasks_file: String) -> Result<(), TError> {
        let new_path = fs::canonicalize(&new_tasks_file)?;
        let path_str = new_path.to_str().expect("your config filepath was not valid,  please check to make sure the file exists and try again.");
        self.tasks_file = path_str.to_string();
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Config { 
        Config {
            tasks_file: ".taskmao/tasks.db3".to_string(),
        }
    }
}

pub fn read_config(file: PathBuf) -> Result<Config, TError> {
    let data = match fs::read_to_string(file) {
        Ok(str) => str,
        Err(_err) => String::from(""),
    };
    let converted_str = match toml::from_str(&data) {
        Ok(conf) => conf,
        Err(_err) => Config::default(),
    };
    Ok(converted_str)
}

pub fn save_config(conf: Config) -> Result<(), TError> {
    let toml_str = toml::to_string(&conf).expect("Could not encode config to TOML value");
    let mut path: PathBuf = match dirs::home_dir() {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(""),
    };
    path.push(".taskmao/settings.toml");
    fs::write(path, toml_str)?;
    Ok(())
}
