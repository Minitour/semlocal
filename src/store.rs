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
                id         TEXT PRIMARY KEY,
                collection TEXT NOT NULL DEFAULT 'default',
                content    TEXT NOT NULL,
                embedding  BLOB NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_entries_collection ON entries (collection);",
        )?;

        Ok(Self { conn })
    }

    pub fn insert(&self, id: &str, collection: &str, content: &str, embedding: &[f32]) -> Result<()> {
        let blob = embedding_to_blob(embedding);
        self.conn.execute(
            "INSERT INTO entries (id, collection, content, embedding) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![id, collection, content, blob],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str, collection: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "DELETE FROM entries WHERE id = ?1 AND collection = ?2",
            rusqlite::params![id, collection],
        )?;
        Ok(affected > 0)
    }

    pub fn all_entries(&self, collection: &str) -> Result<Vec<Entry>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, content, embedding FROM entries WHERE collection = ?1")?;

        let entries = stmt
            .query_map(rusqlite::params![collection], |row| {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn tmp_store() -> (TempDir, Store) {
        let dir = TempDir::new().unwrap();
        let store = Store::open(dir.path(), true).unwrap();
        (dir, store)
    }

    fn fake_embedding(seed: f32) -> Vec<f32> {
        vec![seed, seed + 1.0, seed + 2.0]
    }

    #[test]
    fn insert_and_retrieve() {
        let (_dir, store) = tmp_store();
        let emb = fake_embedding(1.0);
        store.insert("id1", "default", "hello world", &emb).unwrap();

        let entries = store.all_entries("default").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "id1");
        assert_eq!(entries[0].content, "hello world");
        assert_eq!(entries[0].embedding, emb);
    }

    #[test]
    fn delete_existing_entry() {
        let (_dir, store) = tmp_store();
        store.insert("id1", "default", "text", &fake_embedding(1.0)).unwrap();

        assert!(store.delete("id1", "default").unwrap());
        assert!(store.all_entries("default").unwrap().is_empty());
    }

    #[test]
    fn delete_nonexistent_returns_false() {
        let (_dir, store) = tmp_store();
        assert!(!store.delete("no-such-id", "default").unwrap());
    }

    #[test]
    fn delete_wrong_collection_returns_false() {
        let (_dir, store) = tmp_store();
        store.insert("id1", "col-a", "text", &fake_embedding(1.0)).unwrap();

        assert!(!store.delete("id1", "col-b").unwrap());
        assert_eq!(store.all_entries("col-a").unwrap().len(), 1);
    }

    #[test]
    fn collections_are_isolated() {
        let (_dir, store) = tmp_store();
        store.insert("id1", "col-a", "alpha", &fake_embedding(1.0)).unwrap();
        store.insert("id2", "col-b", "beta", &fake_embedding(2.0)).unwrap();

        let a = store.all_entries("col-a").unwrap();
        let b = store.all_entries("col-b").unwrap();
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].content, "alpha");
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].content, "beta");
    }

    #[test]
    fn empty_collection_returns_no_entries() {
        let (_dir, store) = tmp_store();
        store.insert("id1", "col-a", "text", &fake_embedding(1.0)).unwrap();

        assert!(store.all_entries("col-b").unwrap().is_empty());
    }

    #[test]
    fn deleting_last_entry_empties_collection() {
        let (_dir, store) = tmp_store();
        store.insert("id1", "col-a", "one", &fake_embedding(1.0)).unwrap();
        store.insert("id2", "col-a", "two", &fake_embedding(2.0)).unwrap();

        store.delete("id1", "col-a").unwrap();
        assert_eq!(store.all_entries("col-a").unwrap().len(), 1);

        store.delete("id2", "col-a").unwrap();
        assert!(store.all_entries("col-a").unwrap().is_empty());
    }

    #[test]
    fn open_nonexistent_dir_without_create_fails() {
        let dir = TempDir::new().unwrap();
        let missing = dir.path().join("does-not-exist");
        assert!(Store::open(&missing, false).is_err());
    }

    #[test]
    fn blob_roundtrip() {
        let original = vec![1.0_f32, -2.5, 0.0, 3.14];
        let blob = embedding_to_blob(&original);
        let restored = blob_to_embedding(&blob);
        assert_eq!(original, restored);
    }
}
