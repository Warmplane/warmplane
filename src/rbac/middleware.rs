// Rust guideline compliant 2026-08-19

//! Axum security middleware for Role-Based Access Control (RBAC).
//!
//! Extracts Bearer tokens or `X-Warmplane-Key` headers, authenticates callers against the `RbacEngine`,
//! and attaches the verified `TenantContext` to request extensions.

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::daemon::AppState;

/// Middleware enforcing RBAC authentication and injecting `TenantContext` into request extensions.
pub async fn rbac_auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let headers = req.headers();

    // Extract token from Authorization: Bearer <tok> OR X-Warmplane-Key: <tok>
    let token = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer ").or_else(|| v.strip_prefix("bearer ")))
        .or_else(|| {
            headers
                .get("x-warmplane-key")
                .and_then(|h| h.to_str().ok())
        });

    let base_policy = state.policy.read().await.clone();

    // Authenticate token against RbacEngine
    match state.rbac_engine.authenticate(token, &base_policy) {
        Ok(tenant_context) => {
            // Inject TenantContext into request extensions for downstream handlers
            req.extensions_mut().insert(tenant_context);
            next.run(req).await
        }
        Err(err_code) => {
            // Unauthenticated or invalid token supplied
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "ok": false,
                    "error": err_code,
                    "message": "Valid Bearer token, JWT, or X-Warmplane-Key required"
                })),
            )
                .into_response()
        }
    }
}
