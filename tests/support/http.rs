// Rust guideline compliant 2026-11-18
//! HTTP response assertions shared by unit and integration tests.

/// Checks an HTTP status and parses the JSON response body.
///
/// # Panics
/// Panics when the status differs or the response body is not valid JSON.
pub async fn assert_json_response(
    response: axum::response::Response,
    status: axum::http::StatusCode,
) -> serde_json::Value {
    assert_eq!(response.status(), status);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    serde_json::from_slice(&bytes).expect("valid JSON response")
}
