use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::path::PathBuf;

pub struct Embedder {
    model: TextEmbedding,
}

impl Embedder {
    pub fn new() -> Result<Self> {
        let cache_dir = Self::cache_dir()?;
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2)
                .with_show_download_progress(true)
                .with_cache_dir(cache_dir),
        )?;
        Ok(Self { model })
    }

    fn cache_dir() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("could not determine home directory"))?;
        Ok(home.join(".semlocal").join("models"))
    }

    pub fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        let mut embeddings = self.model.embed(vec![text], None)?;
        embeddings
            .pop()
            .ok_or_else(|| anyhow::anyhow!("embedding generation returned no results"))
    }
}
