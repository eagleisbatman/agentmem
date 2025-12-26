use rusqlite::{Connection, Result};
use std::path::Path;
use crate::db::migrations::run_migrations;

pub fn get_connection<P: AsRef<Path>>(path: P) -> Result<Connection> {
    let conn = Connection::open(path)?;
    run_migrations(&conn)?;
    Ok(conn)
}

