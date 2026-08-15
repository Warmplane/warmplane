// Rust guideline compliant 2026-08-15

//! Web dashboard user interface static asset delivery handlers.

use axum::{
    http::header,
    response::{IntoResponse, Response},
};

/// Serves the Warmplane Control Deck single-page web UI dashboard.
pub async fn handle_ui_dashboard() -> Response {
    let html = include_str!("../../ui/dist/index.html");
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response()
}
