use axum::extract::BodyStream;
use futures_util::StreamExt;
use rusqlite::Result;
use serde::Deserialize;
use std::{fs, io::Write};

pub use rusqlite::Connection;

use crate::error::CreateFileError;

const DATA_DIR: &str = "data";

#[derive(Deserialize)]
pub struct UploadParams {
    server: String,
    owner: String,
    repo: String,
    commit: String,
    path: String,
}

/// Create a file on disk and record the result in the database.
pub async fn create_file(
    params: UploadParams,
    mut stream: BodyStream,
) -> Result<(), CreateFileError> {
    let dir = format!(
        "{}/{}/{}/{}/{}",
        DATA_DIR, params.server, params.owner, params.repo, params.commit
    );
    let path = format!("{}/{}", dir, params.path);

    fs::create_dir_all(dir)?;
    let mut file = fs::File::create(path)?;

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(c) => file.write_all(&c)?,
            Err(e) => return Err(CreateFileError::AxumError(e)),
        }
    }

    Ok(())
}

/// Create a connection to the SQLite database, and prepare the database.
pub fn create_db() -> Result<Connection> {
    let conn = Connection::open("data/artifact.db")?;
    prepare_db(&conn)?;
    Ok(conn)
}

/// Create the database tables if they don't exist.
fn prepare_db(conn: &Connection) -> Result<()> {
    if !exists_table(conn, "system")? {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS system (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            key TEXT NOT NULL,
            value TEXT NOT NULL
            )",
            (),
        )?;

        let db_version = "1";
        conn.execute(
            "INSERT INTO system (key, value) VALUES ('db_version', ?)",
            [db_version],
        )?;
    }

    conn.execute(
        "CREATE TABLE IF NOT EXISTS artifacts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            server TEXT NOT NULL,
            owner TEXT NOT NULL,
            repo TEXT NOT NULL,
            commit_hash TEXT NOT NULL,
            path TEXT NOT NULL,
            size INTEGER NOT NULL,
            created_at TEXT NOT NULL
        )",
        (),
    )?;
    Ok(())
}

fn exists_table(conn: &Connection, table_name: &str) -> Result<bool> {
    let mut stmt =
        conn.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?")?;
    let mut rows = stmt.query([table_name])?;
    let row = rows.next().unwrap();
    Ok(row.is_some())
}

/// Shutdown the database connection.
pub fn shutdown_db(conn: Connection) -> Result<(), (rusqlite::Connection, rusqlite::Error)> {
    conn.close()
}
