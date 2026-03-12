use anyhow::{Context, Result};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Entry {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f32>,
}

pub struct Store {
    conn: Connection,
}

impl Store {
    /// Open (or create) the database inside `dir`. Creates the directory if `create` is true.
    pub fn open(dir: &Path, create: bool) -> Result<Self> {
        let db_path = Self::db_path(dir);

        if !dir.exists() {
            if create {
                fs::create_dir_all(dir)
                    .with_context(|| format!("failed to create directory: {}", dir.display()))?;
            } else {
                anyhow::bail!("index directory does not exist: {}", dir.display());
            }
        }

        if !create && !db_path.exists() {
            anyhow::bail!("index database not found at: {}", db_path.display());
        }

        let conn = Connection::open(&db_path)
            .with_context(|| format!("failed to open database: {}", db_path.display()))?;

        conn.pragma_update(None, "journal_mode", "WAL")?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS entries (
                id        TEXT PRIMARY KEY,
                content   TEXT NOT NULL,
                embedding BLOB NOT NULL
            );",
        )?;

        Ok(Self { conn })
    }

    pub fn insert(&self, id: &str, content: &str, embedding: &[f32]) -> Result<()> {
        let blob = embedding_to_blob(embedding);
        self.conn.execute(
            "INSERT INTO entries (id, content, embedding) VALUES (?1, ?2, ?3)",
            rusqlite::params![id, content, blob],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "DELETE FROM entries WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(affected > 0)
    }

    pub fn all_entries(&self) -> Result<Vec<Entry>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, content, embedding FROM entries")?;

        let entries = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let content: String = row.get(1)?;
                let blob: Vec<u8> = row.get(2)?;
                let embedding = blob_to_embedding(&blob);
                Ok(Entry {
                    id,
                    content,
                    embedding,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    fn db_path(dir: &Path) -> PathBuf {
        dir.join("store.db")
    }
}

fn embedding_to_blob(embedding: &[f32]) -> Vec<u8> {
    embedding
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect()
}

fn blob_to_embedding(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}
