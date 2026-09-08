use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use rusqlite::{Connection, params};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TimeEntry {
    pub project: String,
    pub date: DateTime<Local>,
    pub seconds: u32,
}

const DATABASE_FILE_NAME: &str = "timemanager.db";

fn absolute_env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
}

fn data_directory() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    let directory = absolute_env_path("LOCALAPPDATA").or_else(|| absolute_env_path("APPDATA"));

    #[cfg(target_os = "macos")]
    let directory = absolute_env_path("HOME").map(|home| home.join("Library/Application Support"));

    #[cfg(all(unix, not(target_os = "macos")))]
    let directory = absolute_env_path("XDG_DATA_HOME")
        .or_else(|| absolute_env_path("HOME").map(|home| home.join(".local/share")));

    #[cfg(not(any(unix, target_os = "windows")))]
    let directory: Option<PathBuf> = None;

    directory.context("Cannot determine an absolute user application data directory")
}

fn database_path_in(directory: &Path) -> Result<PathBuf> {
    anyhow::ensure!(
        directory.is_absolute(),
        "Application data directory must be absolute"
    );
    let directory = directory.join("TimeManager");
    std::fs::create_dir_all(&directory).with_context(|| {
        format!(
            "Cannot create application data directory {}",
            directory.display()
        )
    })?;
    Ok(directory.join(DATABASE_FILE_NAME))
}

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn relative_data_directory_is_rejected() {
        assert!(database_path_in(Path::new("relative-data")).is_err());
        assert!(database_path_in(Path::new("")).is_err());
    }

    #[test]
    fn application_directory_is_created_and_reused() {
        let root = std::env::temp_dir().join(format!(
            "time-manager-path-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = database_path_in(&root).unwrap();
        assert_eq!(path, root.join("TimeManager").join("timemanager.db"));
        assert!(path.parent().unwrap().is_dir());
        assert_eq!(database_path_in(&root).unwrap(), path);
        std::fs::remove_dir(root.join("TimeManager")).unwrap();
        std::fs::remove_dir(&root).unwrap();
    }
}

pub struct Database;

impl Database {
    fn connect() -> Result<Connection> {
        let path = database_path_in(&data_directory()?)?;
        Connection::open(&path).with_context(|| format!("Cannot open database {}", path.display()))
    }

    fn create_schema() -> Result<()> {
        let conn = Self::connect()?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS time_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project TEXT NOT NULL,
                date TEXT NOT NULL,
                seconds INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    pub fn init() -> Result<()> {
        Self::create_schema()
    }

    pub fn insert_time_entry(entry: &TimeEntry) -> Result<()> {
        let conn = Self::connect()?;

        conn.execute(
            "INSERT INTO time_entries (project, date, seconds) VALUES (?1, ?2, ?3)",
            params![entry.project, entry.date.to_rfc3339(), entry.seconds],
        )?;

        Ok(())
    }

    pub fn get_time_entries() -> Result<Vec<TimeEntry>> {
        let conn = Self::connect()?;

        let mut stmt = conn.prepare("SELECT project, date, seconds FROM time_entries")?;

        let rows = stmt.query_map([], |row| {
            let project: String = row.get(0)?;

            let date_str: String = row.get(1)?;
            let date = DateTime::parse_from_rfc3339(&date_str)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?
                .with_timezone(&Local);

            let seconds_i32: i32 = row.get(2)?;
            let seconds: u32 = seconds_i32.try_into().map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    2,
                    rusqlite::types::Type::Integer,
                    Box::new(e),
                )
            })?;

            Ok(TimeEntry {
                project,
                date,
                seconds,
            })
        })?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }

        Ok(entries)
    }
}
