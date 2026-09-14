// Rust guideline compliant 2026-08-13

//! Capability search engine implementations including lexical, vector, and hybrid ranking.

pub mod hybrid;
pub mod lexical;
pub mod vector;

pub use hybrid::{HybridSearchEngine, SearchEngineInfo, SearchFilter};

#[cfg(test)]
pub(crate) fn dummy_capability(
    server: &str,
    tool: &str,
    summary: &str,
    tags: Vec<&str>,
) -> crate::daemon::CapabilityMeta {
    let mut meta =
        crate::daemon::CapabilityMeta::new(server, tool, summary, summary, serde_json::json!({}));
    meta.tags = tags.into_iter().map(|s| s.to_string()).collect();
    meta
}
