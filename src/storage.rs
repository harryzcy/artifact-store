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
pub fn create_and_prepare_db(filename: &str) -> Result<Connection> {
    let conn = create_db(filename)?;
    prepare_db(&conn)?;
    Ok(conn)
}

fn create_db(filename: &str) -> Result<Connection> {
    let conn = Connection::open(filename)?;
    Ok(conn)
}

/// Create the database tables if they don't exist.
fn prepare_db(conn: &Connection) -> Result<()> {
    if !exists_table(conn, "system")? {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS system (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            version INTEGER NOT NULL
        )",
            (),
        )?;

        let version = 1;
        conn.execute("INSERT INTO system (version) VALUES (?)", [version])?;
    }

    conn.execute(
        "CREATE TABLE IF NOT EXISTS commits (
            sha TEXT NOT NULL PRIMARY KEY,
            server TEXT NOT NULL,
            owner TEXT NOT NULL,
            repo TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS artifacts (
            commit_sha TEXT NOT NULL,
            path TEXT NOT NULL,
            hash TEXT NOT NULL,
            hash_type TEXT NOT NULL,
            created_at TEXT NOT NULL,
            UNIQUE (commit_sha, path)
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    #[test]
    fn create_and_prepare_db_test() {
        let conn = create_and_prepare_db(":memory:").unwrap();
        assert_eq!(exists_table(&conn, "system").unwrap(), true);
        assert_eq!(exists_table(&conn, "commits").unwrap(), true);
        assert_eq!(exists_table(&conn, "artifacts").unwrap(), true);

        let mut stmt = conn
            .prepare("SELECT version FROM system WHERE id = 1")
            .unwrap();
        let mut rows = stmt.query(params![]).unwrap();
        let row = rows.next().unwrap().unwrap();
        let db_version: i32 = row.get(0).unwrap();
        assert_eq!(db_version, 1);
    }

    #[test]
    fn create_db_test() {
        let conn = create_db(":memory:").unwrap();
        shutdown_db(conn).unwrap();
    }

    #[test]
    fn exists_table_test() {
        let conn = create_db(":memory:").unwrap();
        conn.execute("CREATE TABLE test (id INTEGER)", ()).unwrap();
        assert_eq!(exists_table(&conn, "test").unwrap(), true);
    }
}
