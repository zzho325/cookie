use color_eyre::Result;
use color_eyre::eyre::eyre;
use rusqlite::Connection;
use std::sync::mpsc::{self, Sender};
use std::{path::PathBuf, thread::JoinHandle};

pub type Job = Box<dyn FnOnce(&mut Connection) + Send + 'static>;

pub fn spawn_db_thread(mut conn: Connection) -> (JoinHandle<()>, Sender<Job>) {
    let (job_tx, job_rx) = mpsc::channel::<Job>();

    let db_thread_handle = std::thread::spawn(move || {
        for job in job_rx {
            job(&mut conn);
        }
    });

    (db_thread_handle, job_tx)
}

// embed schema
const SCHEMA_SQL: &str = include_str!("./database/schema.sql");
pub fn get_db_conn() -> Result<Connection> {
    let db_path = get_db_path()?;
    if let Some(dir) = db_path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    // Open by default disables per-connection mutex.
    let conn = Connection::open(db_path)?;

    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.execute_batch(SCHEMA_SQL)?;
    Ok(conn)
}

/// Returns the DB path (using $XDG_DATA_HOME if exists or the platform’s standard local data
/// directory).
fn get_db_path() -> Result<PathBuf> {
    const XDG_DATA_HOME: &str = "XDG_DATA_HOME";
    const COOKIE_DB_FILE: &str = "cookie/sqlite.db";

    let config_dir = std::env::var(XDG_DATA_HOME)
        .map(std::path::PathBuf::from)
        .or_else(|_| dirs::data_local_dir().ok_or_else(|| eyre!("failed to get local data dir")))?;
    Ok(config_dir.join(COOKIE_DB_FILE))
}
