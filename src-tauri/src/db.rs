use rusqlite::{params, Connection as SqliteConnection, Result as SqliteResult};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::models::Connection;

fn decrypt_ports(raw: String) -> String {
    if raw.is_empty() || raw == "[]" { return raw; }
    crate::crypto_store::decrypt_value(&raw).unwrap_or(raw)
}

pub struct Database {
    conn: Mutex<SqliteConnection>,
}

impl Database {
    pub fn new(app_dir: &PathBuf) -> SqliteResult<Self> {
        std::fs::create_dir_all(app_dir).ok();
        let db_path = app_dir.join("knockd.db");
        let db_key = crate::crypto_store::derive_db_key();
        let conn = SqliteConnection::open(&db_path)?;
        conn.execute_batch(&format!("PRAGMA key = \"x'{}'\";", db_key))?;
        let db = Database { conn: Mutex::new(conn) };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS connections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                conn_type TEXT NOT NULL CHECK(conn_type IN ('ssh', 'web')),
                host TEXT NOT NULL,
                port INTEGER,
                username TEXT,
                ssh_client TEXT DEFAULT 'auto',
                knock_ports TEXT NOT NULL DEFAULT '[]',
                knock_protocol TEXT DEFAULT 'udp',
                knock_delay_ms INTEGER DEFAULT 100,
                launch_uri TEXT,
                auth_method TEXT NOT NULL DEFAULT 'knockd',
                spa_site_id TEXT,
                spa_credential TEXT,
                spa_udp_port INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            INSERT OR IGNORE INTO settings (key, value) VALUES ('default_ssh_client', 'auto');
            INSERT OR IGNORE INTO settings (key, value) VALUES ('default_knock_delay', '100');
            INSERT OR IGNORE INTO settings (key, value) VALUES ('custom_ssh_paths', '[]');
            INSERT OR IGNORE INTO settings (key, value) VALUES ('theme', 'system');",
        )?;
        // Migration: add new columns if they don't exist yet
        for (col, col_def) in &[
            ("auth_method", "TEXT NOT NULL DEFAULT 'knockd'"),
            ("spa_site_id", "TEXT"),
            ("spa_credential", "TEXT"),
            ("spa_udp_port", "INTEGER"),
        ] {
            let has_col: bool = conn
                .prepare(&format!("SELECT COUNT(*) FROM pragma_table_info('connections') WHERE name='{}'", col))
                .and_then(|mut s| s.query_row([], |r| r.get::<_, i64>(0)))
                .map(|c| c > 0)
                .unwrap_or(false);
            if !has_col {
                let _ = conn.execute_batch(&format!(
                    "ALTER TABLE connections ADD COLUMN {} {};",
                    col, col_def
                ));
            }
        }
        Ok(())
    }

    pub fn list_connections(&self) -> SqliteResult<Vec<Connection>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, conn_type, host, port, username, ssh_client,
                    knock_ports, knock_protocol, knock_delay_ms, launch_uri,
                    auth_method, spa_site_id, spa_credential, spa_udp_port,
                    created_at, updated_at
             FROM connections ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Connection {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                conn_type: row.get(2)?,
                host: row.get(3)?,
                port: row.get(4)?,
                username: row.get(5)?,
                ssh_client: row.get(6)?,
                knock_ports: decrypt_ports(row.get::<_,String>(7)?),
                knock_protocol: row.get(8)?,
                knock_delay_ms: row.get(9)?,
                launch_uri: row.get(10)?,
                auth_method: row.get(11)?,
                spa_site_id: row.get(12)?,
                spa_credential: row.get(13)?,
                spa_udp_port: row.get(14)?,
                created_at: row.get(15)?,
                updated_at: row.get(16)?,
            })
        })?;
        let mut connections = Vec::new();
        for row in rows {
            connections.push(row?);
        }
        Ok(connections)
    }

    pub fn get_connection(&self, id: i64) -> SqliteResult<Option<Connection>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, conn_type, host, port, username, ssh_client,
                    knock_ports, knock_protocol, knock_delay_ms, launch_uri,
                    auth_method, spa_site_id, spa_credential, spa_udp_port,
                    created_at, updated_at
             FROM connections WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Connection {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                conn_type: row.get(2)?,
                host: row.get(3)?,
                port: row.get(4)?,
                username: row.get(5)?,
                ssh_client: row.get(6)?,
                knock_ports: decrypt_ports(row.get::<_,String>(7)?),
                knock_protocol: row.get(8)?,
                knock_delay_ms: row.get(9)?,
                launch_uri: row.get(10)?,
                auth_method: row.get(11)?,
                spa_site_id: row.get(12)?,
                spa_credential: row.get(13)?,
                spa_udp_port: row.get(14)?,
                created_at: row.get(15)?,
                updated_at: row.get(16)?,
            })
        })?;
        match rows.next() {
            Some(Ok(conn)) => Ok(Some(conn)),
            _ => Ok(None),
        }
    }

    pub fn insert_connection(&self, conn: &Connection) -> SqliteResult<i64> {
        let db = self.conn.lock().unwrap();
        db.execute(
            "INSERT INTO connections (name, conn_type, host, port, username, ssh_client,
             knock_ports, knock_protocol, knock_delay_ms, launch_uri,
             auth_method, spa_site_id, spa_credential, spa_udp_port)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
            params![
                conn.name,
                conn.conn_type,
                conn.host,
                conn.port,
                conn.username,
                conn.ssh_client,
                conn.knock_ports,
                conn.knock_protocol,
                conn.knock_delay_ms,
                conn.launch_uri,
                conn.auth_method,
                conn.spa_site_id,
                conn.spa_credential,
                conn.spa_udp_port,
            ],
        )?;
        Ok(db.last_insert_rowid())
    }

    pub fn update_connection(&self, conn: &Connection) -> SqliteResult<()> {
        let db = self.conn.lock().unwrap();
        db.execute(
            "UPDATE connections SET name=?1, conn_type=?2, host=?3, port=?4, username=?5,
             ssh_client=?6, knock_ports=?7, knock_protocol=?8, knock_delay_ms=?9,
             launch_uri=?10, auth_method=?11, spa_site_id=?12, spa_credential=?13,
             spa_udp_port=?14, updated_at=datetime('now')
             WHERE id=?15",
            params![
                conn.name,
                conn.conn_type,
                conn.host,
                conn.port,
                conn.username,
                conn.ssh_client,
                conn.knock_ports,
                conn.knock_protocol,
                conn.knock_delay_ms,
                conn.launch_uri,
                conn.auth_method,
                conn.spa_site_id,
                conn.spa_credential,
                conn.spa_udp_port,
                conn.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_connection(&self, id: i64) -> SqliteResult<()> {
        let db = self.conn.lock().unwrap();
        db.execute("DELETE FROM connections WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> SqliteResult<Option<String>> {
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query_map(params![key], |row| row.get(0))?;
        match rows.next() {
            Some(Ok(val)) => Ok(Some(val)),
            _ => Ok(None),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> SqliteResult<()> {
        let db = self.conn.lock().unwrap();
        db.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }
}
