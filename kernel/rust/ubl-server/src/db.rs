//! Database layer - PostgreSQL ledger with SERIALIZABLE transactions
//! SPEC-UBL-LEDGER v1.0 compliant

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use time::OffsetDateTime;

#[derive(Debug, Deserialize)]
pub struct LinkDraft {
    pub version: u8,
    pub container_id: String,
    pub expected_sequence: i64,
    pub previous_hash: String,
    pub atom_hash: String,
    pub intent_class: String,     // "Observation"|"Conservation"|"Entropy"|"Evolution"
    pub physics_delta: String,    // i128 string (já validado na Membrane)
    pub author_pubkey: String,    // hex
    pub signature: String,        // hex
}

#[derive(Debug, Serialize)]
pub struct LedgerEntry {
    pub container_id: String,
    pub sequence: i64,
    pub link_hash: String,
    pub previous_hash: String,
    pub entry_hash: String,
    pub ts_unix_ms: i64,
}

#[derive(Debug)]
pub enum TangencyError {
    InvalidVersion,
    InvalidTarget,
    RealityDrift,
    SequenceMismatch,
}

#[derive(Clone)]
pub struct PgLedger {
    pool: PgPool,
}

impl PgLedger {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Append transacional com SERIALIZABLE + FOR UPDATE
    /// SPEC-UBL-LEDGER v1.0 §7 - Atomicidade: validate → append → commit
    pub async fn append(&self, link: &LinkDraft) -> Result<LedgerEntry, TangencyError> {
        // Begin SERIALIZABLE transaction
        let mut tx: Transaction<Postgres> = self
            .pool
            .begin()
            .await
            .expect("tx begin");
        
        sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;")
            .execute(&mut *tx)
            .await
            .expect("serializable");

        // Lock and get latest entry (FOR UPDATE)
        let rec = sqlx::query!(
            r#"
            SELECT sequence, entry_hash
            FROM ledger_entry
            WHERE container_id = $1
            ORDER BY sequence DESC
            LIMIT 1
            FOR UPDATE
            "#,
            link.container_id
        )
        .fetch_optional(&mut *tx)
        .await
        .expect("select last");

        let (expected_prev, expected_seq) = match rec {
            Some(r) => (r.entry_hash, r.sequence + 1),
            None => ("0x00".to_string(), 1),
        };

        // Validate causality (SPEC-UBL-MEMBRANE v1.0 §V4)
        if link.previous_hash != expected_prev {
            return Err(TangencyError::RealityDrift);
        }

        // Validate sequence (SPEC-UBL-MEMBRANE v1.0 §V5)
        if link.expected_sequence != expected_seq {
            return Err(TangencyError::SequenceMismatch);
        }

        // Validate version (SPEC-UBL-MEMBRANE v1.0 §V1)
        if link.version != 1 {
            return Err(TangencyError::InvalidVersion);
        }

        // Compute entry_hash = blake3(container_id || sequence || atom_hash || previous_hash || ts)
        let ts_unix_ms = (OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as i64;
        let mut h = Hasher::new();
        h.update(link.container_id.as_bytes());
        h.update(expected_seq.to_string().as_bytes());
        h.update(link.atom_hash.as_bytes());
        h.update(expected_prev.as_bytes());
        h.update(ts_unix_ms.to_string().as_bytes());
        let entry_hash = hex::encode(h.finalize().as_bytes());

        // Insert new entry (SPEC-UBL-LEDGER v1.0 §7.1 - Append-only)
        sqlx::query!(
            r#"
            INSERT INTO ledger_entry (container_id, sequence, link_hash, previous_hash, entry_hash, ts_unix_ms, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, '{}'::jsonb)
            "#,
            link.container_id,
            expected_seq,
            link.atom_hash,
            expected_prev,
            entry_hash,
            ts_unix_ms
        )
        .execute(&mut *tx)
        .await
        .expect("insert");

        // Commit transaction
        tx.commit().await.expect("commit");

        Ok(LedgerEntry {
            container_id: link.container_id.clone(),
            sequence: expected_seq,
            link_hash: link.atom_hash.clone(),
            previous_hash: expected_prev,
            entry_hash,
            ts_unix_ms,
        })
    }

    /// Get current state of container
    pub async fn get_state(&self, container_id: &str) -> Result<LedgerEntry, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
            SELECT sequence, link_hash, previous_hash, entry_hash, ts_unix_ms
            FROM ledger_entry
            WHERE container_id = $1
            ORDER BY sequence DESC
            LIMIT 1
            "#,
            container_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(LedgerEntry {
            container_id: container_id.to_string(),
            sequence: rec.sequence,
            link_hash: rec.link_hash,
            previous_hash: rec.previous_hash,
            entry_hash: rec.entry_hash,
            ts_unix_ms: rec.ts_unix_ms,
        })
    }
}
