use axum::{Json, extract::State, routing::post, Router};
use serde::{Serialize, Deserialize};
use sqlx::PgPool;
mod id_db;
use id_db::create_agent;

#[derive(Clone)]
pub struct IdState { pub pool: PgPool }

#[derive(Deserialize)]
pub struct CreateAgentReq {
    pub kind: String,           // "llm" | "app"
    pub display_name: String,
    pub public_key: String      // hex Ed25519
}

#[derive(Serialize)]
pub struct CreateAgentResp {
    pub sid: String,
    pub kind: String,
    pub public_key: String,
}

pub async fn route_create_agent(State(st): State<IdState>, Json(req): Json<CreateAgentReq>) -> Json<CreateAgentResp> {
    let subj = create_agent(&st.pool, &req.kind, &req.display_name, &req.public_key).await.unwrap();
    Json(CreateAgentResp { sid: subj.sid, kind: subj.kind, public_key: req.public_key })
}

pub fn id_router(state: IdState) -> Router {
    Router::new().route("/id/agents", post(route_create_agent)).with_state(state)
}
