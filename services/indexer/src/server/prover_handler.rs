use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct DeprecatedProveResponse {
    success: bool,
    error: String,
    deprecation_notice: String,
    documentation: String,
}

/// Deprecated legacy endpoint that previously generated SP1 proofs server-side.
///
/// Proof generation must now happen client-side. Wallets/frontends should upload
/// artifacts to the SP1 proving service and submit the resulting proof to the relay.
pub async fn generate_proof() -> impl IntoResponse {
    tracing::warn!(
        "Deprecated endpoint /api/v1/prove was called. Proof generation is now client-side."
    );

    let payload = DeprecatedProveResponse {
        success: false,
        error: "The /api/v1/prove endpoint has been deprecated.".to_string(),
        deprecation_notice: "Generate SP1 proofs in the client or wallet. Upload the SP1Stdin to the TEE proving service and submit the resulting proof to the relay."
            .to_string(),
        documentation: "https://docs.cloak.network/architecture/proving".to_string(),
    };

    (StatusCode::GONE, Json(payload))
}
