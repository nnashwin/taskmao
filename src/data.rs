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
    pub unique_id: String,
}

impl TaskDto {
    pub fn end_task(&mut self, end_time: String) {
        self.running = "false".to_string();
        self.end_time = end_time;
    }

    pub fn save_to_db(&self, conn: &Connection) -> Result<(), TError> {
        conn.execute(
            "INSERT INTO tasks (end_time, description, project_name, running, start_time, unique_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(start_time) DO UPDATE SET
                end_time=excluded.end_time,
                description=excluded.description,
                project_name=excluded.project_name,
                running=excluded.running,
                unique_id=excluded.unique_id;",
            params![self.end_time, self.description, self.project_name, self.running, self.start_time, self.unique_id],
        )?;

        Ok(())
    }
}

pub fn get_most_recent_task(conn: &Connection) -> Result<TaskDto, TError> {
    let stmt = "SELECT description, project_name, running, end_time, start_time, unique_id FROM tasks WHERE id = (SELECT MAX(id) FROM tasks) and running = 'true'";
    let task: TaskDto = conn.query_row(stmt, [], |r| {
        Ok(TaskDto {
            description: r.get(0)?,
            project_name: r.get(1)?,
            running: r.get(2)?,
            end_time: r.get(3)?,
            start_time: r.get(4)?,
            unique_id: r.get(5)?,
        })
    })?;

    Ok(task)
}

pub fn get_todays_tasks(conn: &Connection) -> Result<Vec<TaskDto>, TError> {
    let mut stmt = conn.prepare("SELECT description, project_name, running, end_time, start_time, unique_id FROM tasks WHERE tasks.end_time >= DATETIME('now', '-24 hour')")?;
    let mut rows = stmt.query([])?;

    let mut tasks = Vec::new();

    while let Some(r) = rows.next()? {
        tasks.push(TaskDto {
            description: r.get(0)?,
            project_name: r.get(1)?,
            running: r.get(2)?,
            end_time: r.get(3)?,
            start_time: r.get(4)?,
            unique_id: r.get(5)?,
        });
    }

    Ok(tasks)
}

pub fn set_up_sqlite(conn: &Connection) -> Result<()> {
    let create_sql = r"
        CREATE TABLE IF NOT EXISTS tasks (id INTEGER PRIMARY KEY, start_time TEXT UNIQUE, end_time TEXT, project_name TEXT, running TEXT, description TEXT, unique_id TEXT UNIQUE);
        ";

    conn.execute_batch(create_sql)?;

    Ok(())
}
