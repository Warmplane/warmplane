use crate::daemon::CapabilityMeta;

#[derive(Debug, Clone)]
pub struct VectorMatchResult {
    pub id: String,
    pub score: f32,
}

#[cfg(feature = "semantic-search")]
pub struct VectorSearchIndex {
    model: fastembed::TextEmbedding,
}

#[cfg(feature = "semantic-search")]
impl VectorSearchIndex {
    pub fn new() -> anyhow::Result<Self> {
        let model = fastembed::TextEmbedding::try_new(Default::default())?;
        Ok(Self { model })
    }

    pub fn search(&self, query: &str, capabilities: &[(String, CapabilityMeta)]) -> Vec<VectorMatchResult> {
        if capabilities.is_empty() || query.trim().is_empty() {
            return vec![];
        }

        let doc_texts: Vec<String> = capabilities
            .iter()
            .map(|(id, meta)| format!("{}: {}. {}", id, meta.summary, meta.description))
            .collect();

        let doc_embeddings = match self.model.embed(doc_texts, None) {
            Ok(embeds) => embeds,
            Err(_) => return vec![],
        };

        let query_embeddings = match self.model.embed(vec![query.to_string()], None) {
            Ok(embeds) if !embeds.is_empty() => embeds[0].clone(),
            _ => return vec![],
        };

        let mut results = Vec::new();
        for (idx, (id, _)) in capabilities.iter().enumerate() {
            if let Some(doc_vec) = doc_embeddings.get(idx) {
                let similarity = cosine_similarity(&query_embeddings, doc_vec);
                if similarity > 0.1 {
                    results.push(VectorMatchResult {
                        id: id.clone(),
                        score: similarity,
                    });
                }
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
}

#[cfg(not(feature = "semantic-search"))]
pub struct VectorSearchIndex;

#[cfg(not(feature = "semantic-search"))]
impl VectorSearchIndex {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }

    pub fn search(&self, _query: &str, _capabilities: &[(String, CapabilityMeta)]) -> Vec<VectorMatchResult> {
        vec![]
    }
}

#[cfg(any(feature = "semantic-search", test))]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_similarity_identical_vectors() {
        let v1 = vec![1.0, 2.0, 3.0];
        let v2 = vec![1.0, 2.0, 3.0];
        assert!((cosine_similarity(&v1, &v2) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn cosine_similarity_orthogonal_vectors() {
        let v1 = vec![1.0, 0.0];
        let v2 = vec![0.0, 1.0];
        assert!((cosine_similarity(&v1, &v2) - 0.0).abs() < 1e-5);
    }
}
