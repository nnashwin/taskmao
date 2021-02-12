extern crate rusqlite;

use rusqlite::{params, Connection, Result};

#[derive(Debug)]
pub struct TaskDto {
    pub end_time: String,
    pub description: String,
    pub project_name: String,
    pub running: String,
    pub start_time: String,
}

impl TaskDto {
    pub fn end_task(&mut self, current_time: String) {
        self.running = "false".to_string();
        self.end_time = current_time;
    }

    pub fn save_to_db (&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO tasks (end_time, description, project_name, running, start_time) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![self.end_time, self.description, self.project_name, self.running, self.start_time],
        )?;

        Ok(())
    }
}
