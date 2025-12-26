//! # UBL Server v2.0 + PostgreSQL + Identity
//!
//! HTTP API com PostgreSQL append-only ledger
//! SPEC-UBL-LEDGER v1.0 compliant
//! UBL ID (People · LLM · Apps) - PR28
//!
//! Rotas:
//! - GET  /health
//! - GET  /state/:container_id  
//! - POST /link/validate
//! - POST /link/commit
//! - GET  /ledger/:container_id/tail (SSE with LISTEN/NOTIFY)
//! - POST /id/agents (create LLM/App)
//! - POST /id/agents/{sid}/asc (issue ASC)
//! - POST /id/agents/{sid}/rotate (rotate key)
//! - GET  /id/whoami

mod db;
mod sse;
mod id_db;
mod id_routes;
mod auth;
mod rate_limit;
mod metrics;
mod id_ledger;
mod id_session_token;
mod repo_routes;
mod middleware_require_stepup;

use axum::{
    extract::{Path, State},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use db::{LedgerEntry, LinkDraft, PgLedger, TangencyError};
use serde::Serialize;
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};
use webauthn_rs::prelude::*;

// ============================================================================
// APPLICATION STATE
// ============================================================================

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    ledger: PgLedger,
}

// ============================================================================
// TYPES
// ============================================================================

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

#[derive(Serialize)]
struct Decision {
    decision: &'static str,
}

#[derive(Serialize)]
struct CommitSuccess {
    ok: bool,
    entry: LedgerEntry,
}

#[derive(Serialize)]
struct StateResponse {
    container_id: String,
    sequence: i64,
    last_hash: String,
    entry_count: i64,
}

// ============================================================================
// HANDLERS
// ============================================================================

/// GET /health
async fn route_health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        version: "2.0.0+postgres",
    })
}

/// GET /state/:container_id
async fn route_state(
    State(state): State<AppState>,
    Path(container_id): Path<String>,
) -> Result<Json<StateResponse>, (StatusCode, String)> {
    match state.ledger.get_state(&container_id).await {
        Ok(entry) => {
            // Get entry count
            let count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM ledger_entry WHERE container_id = $1"
            )
            .bind(&container_id)
            .fetch_one(&state.pool)
            .await
            .unwrap_or(0);

            Ok(Json(StateResponse {
                container_id: entry.container_id,
                sequence: entry.sequence,
                last_hash: entry.entry_hash,
                entry_count: count,
            }))
        }
        Err(_) => {
            // Genesis state
            Ok(Json(StateResponse {
                container_id,
                sequence: 0,
                last_hash: "0x00".to_string(),
                entry_count: 0,
            }))
        }
    }
}

/// POST /link/validate
/// Basic validation - in production, inject full Membrane here
async fn route_validate(
    State(_state): State<AppState>,
    Json(_link): Json<LinkDraft>,
) -> Json<Decision> {
    // TODO: Apply SPEC-UBL-MEMBRANE v1.0 §V1-V9 validations
    // For now, simplified validation
    Json(Decision {
        decision: "Accept",
    })
}

/// POST /link/commit
/// Atomic append with SERIALIZABLE transaction + ASC validation
async fn route_commit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(link): Json<LinkDraft>,
) -> Result<Json<CommitSuccess>, (StatusCode, String)> {
    info!(
        "📝 COMMIT seq={} container={} class={}",
        link.expected_sequence, link.container_id, link.intent_class
    );

    // ASC Validation (PR29)
    if let Some(auth_header) = headers.get("authorization") {
        let auth_str = auth_header.to_str().map_err(|_| {
            (StatusCode::BAD_REQUEST, "Invalid authorization header".to_string())
        })?;

        // Extract SID
        let sid = auth::extract_sid_from_header(auth_str).map_err(|e| {
            error!("❌ AUTH ERROR: {}", e.message());
            (e.status_code(), e.message())
        })?;

        // Validate ASC
        let asc_context = auth::validate_asc(&state.pool, &sid).await.map_err(|e| {
            error!("❌ ASC VALIDATION FAILED: {}", e.message());
            (e.status_code(), e.message())
        })?;

        // Validate commit scopes
        auth::validate_commit_scopes(
            &asc_context,
            &link.container_id,
            &link.intent_class,
            &link.physics_delta,
        ).map_err(|e| {
            error!("❌ SCOPE VIOLATION: {}", e.message());
            (e.status_code(), e.message())
        })?;

        info!("✅ ASC VALIDATED sid={} containers={:?}", sid, asc_context.containers);
    } else {
        // No ASC provided - allow for now (TODO: make required in production)
        info!("⚠️  No ASC provided (dev mode - allowing)");
    }

    match state.ledger.append(&link).await {
        Ok(entry) => {
            info!("✅ ACCEPTED seq={} hash={}", entry.sequence, &entry.entry_hash[..8]);
            
            Ok(Json(CommitSuccess {
                ok: true,
                entry,
            }))
        }
        Err(TangencyError::RealityDrift) => {
            error!("❌ REJECTED: RealityDrift");
            Err((StatusCode::CONFLICT, "RealityDrift".into()))
        }
        Err(TangencyError::SequenceMismatch) => {
            error!("❌ REJECTED: SequenceMismatch");
            Err((StatusCode::CONFLICT, "SequenceMismatch".into()))
        }
        Err(TangencyError::InvalidVersion) => {
            error!("❌ REJECTED: InvalidVersion");
            Err((StatusCode::BAD_REQUEST, "InvalidVersion".into()))
        }
        Err(TangencyError::InvalidTarget) => {
            error!("❌ REJECTED: InvalidTarget");
            Err((StatusCode::BAD_REQUEST, "InvalidTarget".into()))
        }
    }
}

/// GET /ledger/:container_id/tail
/// SSE stream with PostgreSQL LISTEN/NOTIFY (PR10)
async fn route_tail(
    State(state): State<AppState>,
    Path(container_id): Path<String>,
) -> impl IntoResponse {
    info!("📡 SSE tail requested for: {}", container_id);
    sse::sse_tail(state.pool.clone(), container_id).await
}

// ============================================================================
// MAIN
// ============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("ubl_server=info".parse().unwrap()),
        )
        .init();

    // Connect to PostgreSQL
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://ubl_dev@localhost:5432/ubl_dev".to_string());

    info!("🔌 Connecting to PostgreSQL...");
    let pool = PgPool::connect(&database_url).await?;
    info!("✅ PostgreSQL connected");

    let state = AppState {
        ledger: PgLedger::new(pool.clone()),
        pool: pool.clone(),
    };

    // Initialize WebAuthn
    let rp_id = std::env::var("WEBAUTHN_RP_ID")
        .unwrap_or_else(|_| "localhost".to_string());
    let rp_origin = std::env::var("WEBAUTHN_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    info!("🔐 WebAuthn: rpId={}, origin={}", rp_id, rp_origin);
    
    let rp_origin_url = Url::parse(&rp_origin)
        .expect("Invalid WEBAUTHN_ORIGIN URL");
    
    let webauthn = WebauthnBuilder::new(&rp_id, &rp_origin_url)
        .expect("Failed to create WebAuthn builder")
        .rp_name("UBL Identity")
        .build()
        .expect("Failed to build WebAuthn");

    let id_state = id_routes::IdState { 
        pool,
        webauthn,
        rate_limiter: rate_limit::RateLimiter::new(),
    };

    // CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/health", get(route_health))
        .route("/state/:container_id", get(route_state))
        .route("/link/validate", post(route_validate))
        .route("/link/commit", post(route_commit))
        .route("/ledger/:container_id/tail", get(route_tail))
        .route("/metrics", get(metrics::metrics_handler))
        .with_state(state.clone())
        .merge(id_routes::id_router().with_state(id_state))
        .merge(id_session_token::router().with_state(state.clone()))
        .merge(repo_routes::router().with_state(state.clone()))
        .layer(cors);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    info!("🚀 UBL Server v2.0 + PostgreSQL + Identity");
    info!("   Listening: http://{}", addr);
    info!("   Database: {}", database_url.split('@').last().unwrap_or("postgres"));
    info!("   Features: SERIALIZABLE transactions + LISTEN/NOTIFY SSE + UBL ID");
    info!("   Chains: Foundation + Persistence + Identity");
    info!("   Features: SERIALIZABLE transactions + LISTEN/NOTIFY SSE + UBL ID");
    info!("   Chains: Foundation + Persistence + Identity");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
