use crate::{ServerConfig, types::data::Message};
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
    ) -> Result<Message> {
        self.0.execute(
            "INSERT INTO chat (channel_id, user_id, contents, timestamp)
         VALUES (?1, ?2, ?3, ?4)",
            params![channel_id, user_id, contents, timestamp],
        )?;

        let id = self.0.last_insert_rowid();

        Ok(Message {
            id,
            channel_id: channel_id.to_string(),
            from: user_id.to_string(),
            contents: contents.to_string(),
            timestamp,
        })
    }

    /// Get a message by its ID
    pub fn get_by_id(&self, message_id: usize) -> Result<Option<Message>> {
        let mut stmt = self.0.prepare(
            "SELECT id, channel_id, user_id, contents, timestamp
         FROM chat
         WHERE id = ?1",
        )?;

        let mut rows = stmt.query_map(params![message_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,    // id
                row.get::<_, String>(1)?, // channel_id
                row.get::<_, String>(2)?, // user_id
                row.get::<_, String>(3)?, // contents
                row.get::<_, i64>(4)?,    // timestamp
            ))
        })?;

        if let Some(row) = rows.next() {
            let (id, channel_id, user_id, contents, timestamp) = row?;
            return Ok(Some(Message {
                id,
                channel_id,
                from: user_id,
                contents,
                timestamp,
            }));
        }
        Ok(None)
    }
}
