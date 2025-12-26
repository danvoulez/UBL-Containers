use serde::{Serialize, Deserialize};
use sqlx::{PgPool};
use blake3::Hasher;

#[derive(Debug, Serialize, Deserialize)]
pub enum SubjectKind { Person, Llm, App }

#[derive(Debug, Serialize, Deserialize)]
pub struct Subject {
    pub sid: String,
    pub kind: String,
    pub display_name: String,
}

pub async fn create_agent(pool: &PgPool, kind: &str, display_name: &str, public_key_hex: &str) -> sqlx::Result<Subject> {
    // sid = "ubl:sid:" + blake3(pubkey_hex | kind)
    let mut h = Hasher::new();
    h.update(public_key_hex.as_bytes());
    h.update(kind.as_bytes());
    let sid = format!("ubl:sid:{}", hex::encode(h.finalize().as_bytes()));

    sqlx::query!("INSERT INTO id_subject (sid, kind, display_name) VALUES ($1,$2,$3) ON CONFLICT DO NOTHING",
        sid, kind, display_name).execute(pool).await?;

    sqlx::query!("INSERT INTO id_credential (sid, credential_kind, public_key, key_version) VALUES ($1,'ed25519', decode($2,'hex'), 1)",
        sid, public_key_hex).execute(pool).await?;

    Ok(Subject { sid, kind: kind.to_string(), display_name: display_name.to_string() })
}
