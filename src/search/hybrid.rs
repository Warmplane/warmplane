use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use crate::daemon::{CapabilityMeta, Policy};
use super::lexical::score_lexical;
use super::vector::VectorSearchIndex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySearchResult {
    pub id: String,
    pub summary: String,
    pub server: String,
    pub tags: Vec<String>,
    pub mode: String,
    pub score: f32,
    pub match_types: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SearchFilter {
    pub server_ids: Vec<String>,
    pub tags: Vec<String>,
    pub modes: Vec<String>,
}

pub struct HybridSearchEngine {
    vector_index: Option<VectorSearchIndex>,
}

impl HybridSearchEngine {
    pub fn new() -> Self {
        let vector_index = VectorSearchIndex::new().ok();
        Self { vector_index }
    }

    pub fn search(
        &self,
        query: &str,
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

        // 2. Perform Lexical Search
        let lexical_results = score_lexical(query, &candidates);

        // 3. Perform Vector Search (if index available)
        let vector_results = match &self.vector_index {
            Some(idx) => idx.search(query, &candidates),
            None => vec![],
        };

        // 4. Combine via Reciprocal Rank Fusion (RRF) & Weighted Score
        let mut score_map: HashMap<String, (f32, HashSet<String>)> = HashMap::new();

        // RRF constant k
        let k = 60.0f32;

        for (rank, lex) in lexical_results.iter().enumerate() {
            let rrf_score = 1.0 / (k + rank as f32 + 1.0);
            let entry = score_map.entry(lex.id.clone()).or_insert((0.0, HashSet::new()));
            entry.0 += rrf_score + (lex.score * 0.5);
            for m in &lex.match_types {
                entry.1.insert(m.clone());
            }
        }

        for (rank, vec_res) in vector_results.iter().enumerate() {
            let rrf_score = 1.0 / (k + rank as f32 + 1.0);
            let entry = score_map.entry(vec_res.id.clone()).or_insert((0.0, HashSet::new()));
            entry.0 += rrf_score + (vec_res.score * 0.5);
            entry.1.insert("semantic".to_string());
        }

        // 5. Convert to final response list
        let candidate_map: HashMap<String, CapabilityMeta> = candidates.into_iter().collect();
        let mut final_results = Vec::new();

        for (id, (raw_score, match_types)) in score_map {
            if let Some(meta) = candidate_map.get(&id) {
                let mut match_vec: Vec<String> = match_types.into_iter().collect();
                match_vec.sort();

                // Normalize score to 0.0 .. 1.0
                let normalized_score = (raw_score * 10.0).min(1.0);
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
                    match_types: match_vec,
                });
            }
        }

        final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        final_results.truncate(limit);
        final_results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_capability(server: &str, tool: &str, summary: &str, tags: Vec<&str>) -> CapabilityMeta {
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
            dummy_capability("github", "issues.search", "Search GitHub issues", vec!["git", "issues"]),
        );
        caps.insert(
            "obs.logs.search".to_string(),
            dummy_capability("obs", "logs.search", "Search application logs", vec!["logs", "read"]),
        );

        let engine = HybridSearchEngine::new();
        let policy = Policy::default();

        let results = engine.search(
            "issues",
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
        let mut policy = Policy::default();
        policy.deny = vec!["db.delete".to_string()];

        let results = engine.search(
            "delete",
            10,
            &SearchFilter::default(),
            &caps,
            &policy,
        );

        assert!(results.is_empty());
    }
}
