extern crate rusqlite;

use crate::TError;
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

pub fn get_most_recent_task(conn: &Connection) -> Result<TaskDto, TError> {
    let stmt = "SELECT description, project_name, running, end_time, start_time FROM tasks WHERE id = (SELECT MAX(id) FROM tasks) and running = 'true'";
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

pub fn get_todays_tasks(conn: &Connection) -> Result<Vec<TaskDto>, TError> {
    let stmt = "SELECT description, project_name, running, end_time, start_time FROM tasks WHERE id = (SELECT MAX(id) FROM tasks) and running = 'true'";

    Ok(Vec::new())
}
