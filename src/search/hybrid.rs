// Rust guideline compliant 2026-08-13

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::lexical::score_lexical;
use super::vector::VectorSearchIndex;
use crate::daemon::{CapabilityMeta, Policy};

/// Ranked result item returned by hybrid capability search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySearchResult {
    /// Unique identifier of matched capability.
    pub id: String,
    /// Short summary description.
    pub summary: String,
    /// Server identifier providing this capability.
    pub server: String,
    /// Metadata tags associated with capability.
    pub tags: Vec<String>,
    /// Execution mode (e.g. `read`, `write`, `execute`).
    pub mode: String,
    /// Combined hybrid relevance score.
    pub score: f32,
    /// Match signals contributing to ranking.
    pub match_types: Vec<String>,
}

/// Deterministic query filter criteria for capability search.
#[derive(Debug, Clone, Default)]
pub struct SearchFilter {
    /// Optional server ID filter list.
    pub server_ids: Vec<String>,
    /// Optional tag filter list.
    pub tags: Vec<String>,
    /// Optional execution mode filter list.
    pub modes: Vec<String>,
}

impl SearchFilter {
    /// Creates a new `SearchFilterBuilder` for constructing a filter.
    pub fn builder() -> SearchFilterBuilder {
        SearchFilterBuilder::default()
    }
}

/// Builder for constructing `SearchFilter` instances (`M-INIT-BUILDER`).
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct SearchFilterBuilder {
    server_ids: Vec<String>,
    tags: Vec<String>,
    modes: Vec<String>,
}

#[allow(dead_code)]
impl SearchFilterBuilder {
    /// Adds a server ID filter.
    pub fn server_id(mut self, server_id: impl Into<String>) -> Self {
        self.server_ids.push(server_id.into());
        self
    }

    /// Sets the server ID filters list.
    pub fn server_ids(mut self, server_ids: Vec<String>) -> Self {
        self.server_ids = server_ids;
        self
    }

    /// Adds a tag filter.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Sets the tags filter list.
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Adds an execution mode filter.
    pub fn mode(mut self, mode: impl Into<String>) -> Self {
        self.modes.push(mode.into());
        self
    }

    /// Sets the modes filter list.
    pub fn modes(mut self, modes: Vec<String>) -> Self {
        self.modes = modes;
        self
    }

    /// Builds the `SearchFilter`.
    pub fn build(self) -> SearchFilter {
        SearchFilter {
            server_ids: self.server_ids,
            tags: self.tags,
            modes: self.modes,
        }
    }
}

/// Hybrid search engine combining lexical and vector scores with reciprocal rank fusion (RRF).
pub struct HybridSearchEngine {
    vector_index: Option<VectorSearchIndex>,
}

impl Default for HybridSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl HybridSearchEngine {
    /// Creates a new `HybridSearchEngine` initializing vector index if available.
    ///
    /// # Returns
    /// An initialized `HybridSearchEngine`.
    pub fn new() -> Self {
        let vector_index = VectorSearchIndex::new().ok();
        Self { vector_index }
    }

    /// Performs hybrid reciprocal rank fusion search over capabilities.
    ///
    /// # Arguments
    /// * `query` - Search query string.
    /// * `limit` - Maximum number of results to return.
    /// * `filter` - Deterministic search filter parameters.
    /// * `capabilities` - Registered capabilities map.
    /// * `policy` - Access control policy.
    ///
    /// # Returns
    /// Sorted vector of `CapabilitySearchResult` items.
    pub fn search(
        &self,
        query: impl AsRef<str>,
        limit: usize,
        filter: &SearchFilter,
        capabilities: &HashMap<String, CapabilityMeta>,
        policy: &Policy,
    ) -> Vec<CapabilitySearchResult> {
        // 1. Filter capabilities by policy and user-provided deterministic criteria
        let candidates: Vec<(String, CapabilityMeta)> = capabilities
            .iter()
            .filter(|(id, meta)| {
                if !policy.allows(id) {
                    return false;
                }
                if !filter.server_ids.is_empty() && !filter.server_ids.contains(&meta.server) {
                    return false;
                }
                if !filter.tags.is_empty() && !meta.tags.iter().any(|t| filter.tags.contains(t)) {
                    return false;
                }
                let mode = if meta.tags.contains(&"write".to_string()) {
                    "write"
                } else {
                    "read"
                };
                if !filter.modes.is_empty() && !filter.modes.contains(&mode.to_string()) {
                    return false;
                }
                true
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        if candidates.is_empty() {
            return vec![];
        }

        let q_str = query.as_ref();

        // 2. Perform Lexical Search
        let lexical_results = score_lexical(q_str, &candidates);

        // 3. Perform Vector Search (if index available)
        let vector_results = match &self.vector_index {
            Some(idx) => idx.search(q_str, &candidates),
            None => vec![],
        };

        // 4. Combine via Reciprocal Rank Fusion (RRF) & Weighted Score
        // Structure: id -> (combined_score, match_types_vec)
        let mut score_map: HashMap<String, (f32, Vec<String>)> =
            HashMap::with_capacity(candidates.len());

        // RRF constant k
        let k = 60.0f32;

        for (rank, lex) in lexical_results.into_iter().enumerate() {
            let rrf_score = 1.0 / (k + rank as f32 + 1.0);
            let entry = score_map
                .entry(lex.id)
                .or_insert_with(|| (0.0, Vec::with_capacity(4)));
            entry.0 += rrf_score + (lex.score * 0.5);
            for m in lex.match_types {
                if !entry.1.contains(&m) {
                    entry.1.push(m);
                }
            }
        }

        for (rank, vec_res) in vector_results.into_iter().enumerate() {
            let rrf_score = 1.0 / (k + rank as f32 + 1.0);
            let entry = score_map
                .entry(vec_res.id)
                .or_insert_with(|| (0.0, Vec::with_capacity(4)));
            entry.0 += rrf_score + (vec_res.score * 0.5);
            let sem = "semantic".to_string();
            if !entry.1.contains(&sem) {
                entry.1.push(sem);
            }
        }

        // 5. Convert to candidate list for sorting
        let candidate_map: HashMap<String, CapabilityMeta> = candidates.into_iter().collect();
        let mut scored_items: Vec<(String, f32, Vec<String>)> = score_map
            .into_iter()
            .map(|(id, (score, matches))| (id, score, matches))
            .collect();

        // Sort descending by score
        scored_items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Truncate to limit before building detailed objects
        scored_items.truncate(limit.min(100));

        let mut final_results = Vec::with_capacity(scored_items.len());
        for (id, raw_score, mut match_types) in scored_items {
            if let Some(meta) = candidate_map.get(&id) {
                match_types.sort();
                let normalized_score = (raw_score * 0.1).min(1.0);
                let mode = if meta.tags.contains(&"write".to_string()) {
                    "write".to_string()
                } else {
                    "read".to_string()
                };

                final_results.push(CapabilitySearchResult {
                    id,
                    summary: meta.summary.clone(),
                    server: meta.server.clone(),
                    tags: meta.tags.clone(),
                    mode,
                    score: (normalized_score * 100.0).round() / 100.0,
                    match_types,
                });
            }
        }

        final_results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_capability(
        server: &str,
        tool: &str,
        summary: &str,
        tags: Vec<&str>,
    ) -> CapabilityMeta {
        CapabilityMeta {
            server: server.to_string(),
            tool: tool.to_string(),
            summary: summary.to_string(),
            description: summary.to_string(),
            input_schema: serde_json::json!({}),
            tags: tags.into_iter().map(|s| s.to_string()).collect(),
            examples: vec![],
        }
    }

    #[test]
    fn hybrid_search_filters_and_ranks() {
        let mut caps = HashMap::new();
        caps.insert(
            "github.issues.search".to_string(),
            dummy_capability(
                "github",
                "issues.search",
                "Search GitHub issues",
                vec!["git", "issues"],
            ),
        );
        caps.insert(
            "obs.logs.search".to_string(),
            dummy_capability(
                "obs",
                "logs.search",
                "Search application logs",
                vec!["logs", "read"],
            ),
        );

        let engine = HybridSearchEngine::new();
        let policy = Policy::default();

        let results = engine.search(
            "github issues",
            10,
            &SearchFilter::default(),
            &caps,
            &policy,
        );

        assert!(!results.is_empty());
        assert_eq!(results[0].id, "github.issues.search");
    }

    #[test]
    fn hybrid_search_enforces_policy_blocking() {
        let mut caps = HashMap::new();
        caps.insert(
            "db.delete".to_string(),
            dummy_capability("db", "delete", "Delete database contents", vec!["db"]),
        );

        let engine = HybridSearchEngine::new();
        let policy = Policy {
            deny: vec!["db.delete".to_string()],
            ..Default::default()
        };

        let results = engine.search("delete", 10, &SearchFilter::default(), &caps, &policy);

        assert!(results.is_empty());
    }
}
