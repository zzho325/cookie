use std::path::PathBuf;

use color_eyre::Result;
use color_eyre::eyre::eyre;
use rusqlite::Connection;

// embed schema
const SCHEMA_SQL: &str = include_str!("./database/schema.sql");
pub fn get_db() -> Result<Connection> {
    let db_path = get_db_path()?;
    let conn = Connection::open(db_path)?;

    conn.pragma_update(None, "foreign_keys", &"ON")?;
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
