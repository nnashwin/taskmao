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