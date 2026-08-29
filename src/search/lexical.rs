// Rust guideline compliant 2026-08-13

use crate::daemon::CapabilityMeta;
use std::collections::HashSet;

/// Represents a single lexical search match result.
#[derive(Debug, Clone)]
pub struct LexicalMatchResult {
    /// Unique identifier of matched capability.
    pub id: String,
    /// Lexical relevance score.
    pub score: f32,
    /// Match categories triggering this result.
    pub match_types: Vec<String>,
}

/// Scores registered capabilities against a plain-text lexical query.
///
/// # Arguments
/// * `query` - Search query string.
/// * `capabilities` - Slice of capability identifier and metadata pairs.
///
/// # Returns
/// Sorted vector of `LexicalMatchResult` items ordered by score descending.
pub fn score_lexical(
    query: impl AsRef<str>,
    capabilities: &[(String, CapabilityMeta)],
) -> Vec<LexicalMatchResult> {
    let query_clean = query.as_ref().trim().to_lowercase();
    if query_clean.is_empty() {
        return capabilities
            .iter()
            .map(|(id, _)| LexicalMatchResult {
                id: id.clone(),
                score: 1.0,
                match_types: vec!["all".to_string()],
            })
            .collect();
    }

    let query_tokens: Vec<&str> = query_clean
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect();

    let query_token_set: HashSet<&str> = query_tokens.iter().copied().collect();

    let mut results = Vec::new();

    for (id, meta) in capabilities {
        let mut match_types = HashSet::new();
        let mut score: f32 = 0.0;

        // 1. Exact ID / alias match
        if id.eq_ignore_ascii_case(&query_clean) {
            score += 1.0;
            match_types.insert("exact_id".to_string());
        } else if id.to_ascii_lowercase().contains(&query_clean) {
            score += 0.7;
            match_types.insert("id".to_string());
        }

        // 2. Exact or substring server / tool match
        if meta.server.eq_ignore_ascii_case(&query_clean)
            || meta.tool.eq_ignore_ascii_case(&query_clean)
        {
            score += 0.6;
            match_types.insert("server_tool".to_string());
        }

        // 3. Tag matching
        let mut tag_hit = false;
        for tag in &meta.tags {
            if tag.eq_ignore_ascii_case(&query_clean)
                || query_token_set
                    .iter()
                    .any(|qt| tag.eq_ignore_ascii_case(qt))
            {
                tag_hit = true;
                break;
            }
        }
        if tag_hit {
            score += 0.5;
            match_types.insert("tag".to_string());
        }

        // 4. Token overlap scoring over summary & description (tokenizes text stream directly without format! allocation)
        let mut token_matches = 0;
        let mut check_field_tokens = |text: &str| {
            for token in text.split(|c: char| !c.is_alphanumeric()) {
                if !token.is_empty() {
                    for qt in &query_token_set {
                        if token.eq_ignore_ascii_case(qt) {
                            token_matches += 1;
                        }
                    }
                }
            }
        };

        check_field_tokens(id);
        check_field_tokens(&meta.summary);
        check_field_tokens(&meta.description);

        if !query_tokens.is_empty() {
            let overlap_ratio = (token_matches as f32 / query_tokens.len() as f32).min(1.0);
            if overlap_ratio > 0.0 {
                score += overlap_ratio * 0.5;
                match_types.insert("lexical".to_string());
            }
        }

        if score > 0.0 {
            let mut types_vec: Vec<String> = match_types.into_iter().collect();
            types_vec.sort();
            results.push(LexicalMatchResult {
                id: id.clone(),
                score: score.min(1.0),
                match_types: types_vec,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_capability(
        server: &str,
        tool: &str,
        summary: &str,
        tags: Vec<&str>,
    ) -> CapabilityMeta {
        let mut meta = CapabilityMeta::new(server, tool, summary, summary, serde_json::json!({}));
        meta.tags = tags.into_iter().map(|s| s.to_string()).collect();
        meta
    }

    #[test]
    fn exact_id_returns_top_score() {
        let caps = vec![(
            "github.issues.search".to_string(),
            dummy_capability(
                "github",
                "issues.search",
                "Search GitHub issues",
                vec!["git", "issues"],
            ),
        )];

        let matches = score_lexical("github.issues.search", &caps);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].score, 1.0);
        assert!(matches[0].match_types.contains(&"exact_id".to_string()));
    }

    #[test]
    fn tag_match_finds_item() {
        let caps = vec![(
            "obs.logs.search".to_string(),
            dummy_capability(
                "obs",
                "logs.search",
                "Search structured app logs",
                vec!["logs", "read"],
            ),
        )];

        let matches = score_lexical("logs", &caps);
        assert_eq!(matches.len(), 1);
        assert!(matches[0].match_types.contains(&"tag".to_string()));
    }
}
