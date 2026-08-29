// Rust guideline compliant 2026-08-13

use crate::daemon::CapabilityMeta;

/// Represents a single vector semantic search match result.
#[derive(Debug, Clone)]
pub struct VectorMatchResult {
    /// Unique identifier of matched capability.
    pub id: String,
    /// Cosine similarity semantic score.
    pub score: f32,
}

/// Maximum number of candidate capabilities embedded per semantic vector search query to prevent CPU/memory DoS.
pub const MAX_VECTOR_SEARCH_CANDIDATES: usize = 250;

/// Semantic vector search index wrapping embedding models when enabled.
#[cfg(feature = "semantic-search")]
pub struct VectorSearchIndex {
    model: fastembed::TextEmbedding,
}

#[cfg(feature = "semantic-search")]
impl VectorSearchIndex {
    /// Creates a new `VectorSearchIndex` initializing FastEmbed models.
    ///
    /// # Errors
    /// Returns an error if the model failed to initialize after 5 attempts.
    pub fn new() -> anyhow::Result<Self> {
        let mut attempts = 0;
        loop {
            match fastembed::TextEmbedding::try_new(Default::default()) {
                Ok(model) => return Ok(Self { model }),
                Err(err) => {
                    attempts += 1;
                    if attempts >= 5 {
                        return Err(err);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(300));
                }
            }
        }
    }

    /// Performs semantic vector search over registered capabilities.
    ///
    /// # Arguments
    /// * `query` - Search query string.
    /// * `capabilities` - Capability identifier and metadata pairs.
    ///
    /// # Returns
    /// Sorted vector of `VectorMatchResult` items ordered by semantic score descending.
    pub fn search(
        &self,
        query: impl AsRef<str>,
        capabilities: &[(String, CapabilityMeta)],
    ) -> Vec<VectorMatchResult> {
        let q_str = query.as_ref();
        if capabilities.is_empty() || q_str.trim().is_empty() {
            return vec![];
        }

        // Bound candidate count to prevent CPU/memory exhaustion on large catalogs
        let capped_capabilities = if capabilities.len() > MAX_VECTOR_SEARCH_CANDIDATES {
            &capabilities[..MAX_VECTOR_SEARCH_CANDIDATES]
        } else {
            capabilities
        };

        let doc_texts: Vec<String> = capped_capabilities
            .iter()
            .map(|(id, meta)| format!("{}: {}. {}", id, meta.summary, meta.description))
            .collect();

        let mut all_texts = vec![q_str.to_string()];
        all_texts.extend(doc_texts);

        let embeddings = match self.model.embed(all_texts, None) {
            Ok(emb) if emb.len() == capped_capabilities.len() + 1 => emb,
            _ => return vec![],
        };

        let query_emb = &embeddings[0];
        let mut results = Vec::new();

        for (i, (id, _)) in capped_capabilities.iter().enumerate() {
            let doc_emb = &embeddings[i + 1];
            let sim = cosine_similarity(query_emb, doc_emb);
            if sim > 0.15 {
                results.push(VectorMatchResult {
                    id: id.clone(),
                    score: sim,
                });
            }
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }
}

/// Fallback dummy vector index when semantic search feature is disabled.
#[cfg(not(feature = "semantic-search"))]
pub struct VectorSearchIndex;

#[cfg(not(feature = "semantic-search"))]
impl VectorSearchIndex {
    /// Creates a dummy fallback vector search index.
    ///
    /// # Errors
    /// Never returns an error in dummy implementation.
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }

    /// Performs no-op vector search returning an empty result set.
    pub fn search(
        &self,
        _query: impl AsRef<str>,
        _capabilities: &[(String, CapabilityMeta)],
    ) -> Vec<VectorMatchResult> {
        vec![]
    }
}

/// Computes cosine similarity between two numeric feature vectors.
///
/// # Arguments
/// * `a` - First vector.
/// * `b` - Second vector.
///
/// # Returns
/// Cosine similarity value between 0.0 and 1.0 (or 0.0 if empty or zero norm).
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

    #[cfg(feature = "semantic-search")]
    #[test]
    fn test_real_fastembed_model_embedding() {
        let index = VectorSearchIndex::new().expect("Failed to initialize FastEmbed ONNX model");
        let caps = vec![(
            "github.issues.search".to_string(),
            crate::daemon::CapabilityMeta::new(
                "github",
                "issues.search",
                "Search open GitHub issues",
                "Search open GitHub issues",
                serde_json::json!({}),
            ),
        )];

        let results = index.search("find git bugs", &caps);
        assert!(
            !results.is_empty(),
            "FastEmbed ONNX inference returned empty results"
        );
        assert_eq!(results[0].id, "github.issues.search");
        assert!(
            results[0].score > 0.3,
            "Expected high semantic similarity score"
        );
    }
}
