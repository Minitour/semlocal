use crate::store::Entry;

pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub score: f32,
}

pub fn search(query_embedding: &[f32], entries: &[Entry], top: usize) -> Vec<SearchResult> {
    let mut scored: Vec<SearchResult> = entries
        .iter()
        .map(|entry| SearchResult {
            id: entry.id.clone(),
            content: entry.content.clone(),
            score: cosine_similarity(query_embedding, &entry.embedding),
        })
        .collect();

    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top);
    scored
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}
