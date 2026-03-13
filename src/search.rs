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

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, embedding: Vec<f32>) -> Entry {
        Entry {
            id: id.to_string(),
            content: format!("content-{id}"),
            embedding,
        }
    }

    #[test]
    fn identical_vectors_have_score_one() {
        let a = vec![1.0, 2.0, 3.0];
        let score = cosine_similarity(&a, &a);
        assert!((score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn orthogonal_vectors_have_score_zero() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let score = cosine_similarity(&a, &b);
        assert!(score.abs() < 1e-6);
    }

    #[test]
    fn opposite_vectors_have_negative_score() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![-1.0, -2.0, -3.0];
        let score = cosine_similarity(&a, &b);
        assert!((score + 1.0).abs() < 1e-6);
    }

    #[test]
    fn zero_vector_returns_zero() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![1.0, 2.0, 3.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
        assert_eq!(cosine_similarity(&b, &a), 0.0);
    }

    #[test]
    fn search_returns_top_k_sorted_by_score() {
        let query = vec![1.0, 0.0, 0.0];
        let entries = vec![
            entry("far", vec![0.0, 1.0, 0.0]),
            entry("close", vec![1.0, 0.1, 0.0]),
            entry("mid", vec![1.0, 1.0, 0.0]),
        ];

        let results = search(&query, &entries, 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "close");
        assert_eq!(results[1].id, "mid");
        assert!(results[0].score >= results[1].score);
    }

    #[test]
    fn search_with_top_greater_than_entries() {
        let query = vec![1.0, 0.0];
        let entries = vec![entry("only", vec![1.0, 0.0])];

        let results = search(&query, &entries, 10);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_empty_entries() {
        let query = vec![1.0, 0.0];
        let results = search(&query, &[], 5);
        assert!(results.is_empty());
    }
}
