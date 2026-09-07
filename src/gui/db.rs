use anyhow::Result;
use chrono::{DateTime, Local};
use rusqlite::{Connection, params};

#[derive(Debug, Clone)]
pub struct TimeEntry {
    pub project: String,
    pub date: DateTime<Local>,
    pub seconds: u32,
}

const DATABASE_URL: &str = "timemanager.db";

pub struct Database;

impl Database {
    fn create_schema() -> Result<()> {
        let conn = Connection::open(DATABASE_URL)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS time_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project TEXT NOT NULL,
                date TEXT NOT NULL,
                seconds INTEGER NOT NULL
            )", []
        )?;

        Ok(())
    }

    pub fn init() -> Result<()> {
        let _ = Database::create_schema();
        Ok(())
    }

    pub fn insert_time_entry(entry: &TimeEntry) -> Result<()> {
        let conn = Connection::open(DATABASE_URL)?;

        conn.execute(
            "INSERT INTO time_entries (project, date, seconds) VALUES (?1, ?2, ?3)",
            params![entry.project, entry.date.to_rfc3339(), entry.seconds],
        )?;

        Ok(())
    }

    pub fn get_time_entries() -> Result<Vec<TimeEntry>> {
        let conn = Connection::open(DATABASE_URL)?;

        let mut stmt = conn.prepare("SELECT project, date, seconds FROM time_entries")?;
        
        let rows = stmt.query_map([], |row| {
            let project: String = row.get(0)?;
            
            let date_str: String = row.get(1)?;
            let date = DateTime::parse_from_rfc3339(&date_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Local);
                
            // Busca como i32 e converte com segurança para u32
            let seconds_i32: i32 = row.get(2)?;
            let seconds: u32 = seconds_i32.try_into().map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Integer, Box::new(e))
            })?;

            Ok(TimeEntry { project, date, seconds })
        })?;

        // Coleta os resultados convertendo o iterador de Results em um Result<Vec>
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }

        Ok(entries)
    }
}