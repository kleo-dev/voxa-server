use crate::ServerConfig;
use rusqlite::{Connection, Result, params};

pub struct Database {
    pub messages_db: MessagesDb,
}

impl Database {
    pub fn new(config: &ServerConfig) -> Option<Self> {
        let conn = Connection::open("main.db").ok()?;
        let messages_db = MessagesDb(conn);
        messages_db.init(config)?;
        Some(Self { messages_db })
    }
}

unsafe impl Send for Database {}
unsafe impl Sync for Database {}

pub struct MessagesDb(pub Connection);

impl MessagesDb {
    pub fn init(&self, _config: &ServerConfig) -> Option<usize> {
        self.0
            .execute(
                "CREATE TABLE IF NOT EXISTS chat (
                  id          INTEGER PRIMARY KEY AUTOINCREMENT,
                  channel_id  TEXT NOT NULL,
                  user_id     TEXT NOT NULL,
                  contents    TEXT NOT NULL,
                  timestamp   INTEGER NOT NULL
                )",
                [],
            )
            .ok()
    }

    /// Insert a message into the DB
    pub fn insert(
        &self,
        channel_id: &str,
        user_id: &str,
        contents: &str,
        timestamp: i64,
    ) -> Result<usize> {
        self.0.execute(
            "INSERT INTO chat (channel_id, user_id, contents, timestamp)
             VALUES (?1, ?2, ?3, ?4)",
            params![channel_id, user_id, contents, timestamp],
        )
    }

    /// Fetch the latest N messages for a channel
    pub fn fetch_recent(
        &self,
        channel_id: &str,
        limit: usize,
    ) -> Result<Vec<(i64, String, String, String, i64)>> {
        let mut stmt = self.0.prepare(
            "SELECT id, channel_id, user_id, contents, timestamp
             FROM chat
             WHERE channel_id = ?1
             ORDER BY timestamp DESC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![channel_id, limit], |row| {
            Ok((
                row.get::<_, i64>(0)?,    // id
                row.get::<_, String>(1)?, // channel_id
                row.get::<_, String>(2)?, // user_id
                row.get::<_, String>(3)?, // contents
                row.get::<_, i64>(4)?,    // timestamp
            ))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }
}
