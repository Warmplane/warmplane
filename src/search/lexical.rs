use crate::daemon::CapabilityMeta;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct LexicalMatchResult {
    pub id: String,
    pub score: f32,
    pub match_types: Vec<String>,
}

pub fn score_lexical(
    query: &str,
    capabilities: &[(String, CapabilityMeta)],
) -> Vec<LexicalMatchResult> {
    let query_clean = query.trim().to_lowercase();
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
        let id_lower = id.to_lowercase();
        let server_lower = meta.server.to_lowercase();
        let tool_lower = meta.tool.to_lowercase();
        let summary_lower = meta.summary.to_lowercase();
        let desc_lower = meta.description.to_lowercase();

        let mut match_types = HashSet::new();
        let mut score: f32 = 0.0;

        // 1. Exact ID / alias match
        if id_lower == query_clean {
            score += 1.0;
            match_types.insert("exact_id".to_string());
        } else if id_lower.contains(&query_clean) {
            score += 0.7;
            match_types.insert("id".to_string());
        }

        // 2. Exact or substring server / tool match
        if server_lower == query_clean || tool_lower == query_clean {
            score += 0.6;
            match_types.insert("server_tool".to_string());
        }

        // 3. Tag matching
        let mut tag_hit = false;
        for tag in &meta.tags {
            let tag_lower = tag.to_lowercase();
            if tag_lower == query_clean || query_tokens.contains(&tag_lower.as_str()) {
                tag_hit = true;
                break;
            }
        }
        if tag_hit {
            score += 0.5;
            match_types.insert("tag".to_string());
        }

        // 4. Token overlap scoring over summary & description
        let full_text = format!("{} {} {}", id_lower, summary_lower, desc_lower);
        let text_tokens: Vec<&str> = full_text
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .collect();

        let mut token_matches = 0;
        for token in &query_token_set {
            if text_tokens.contains(token) {
                token_matches += 1;
            }
        }

        if !query_tokens.is_empty() {
            let overlap_ratio = token_matches as f32 / query_tokens.len() as f32;
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
