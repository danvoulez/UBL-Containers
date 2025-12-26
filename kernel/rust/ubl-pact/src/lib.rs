//! # UBL Pact
//!
//! **Title:** SPEC-UBL-PACT v1.0  
//! **Status:** NORMATIVE  
//! **Change-Control:** STRICT (no retroactive changes)  
//! **Hash:** BLAKE3 | **Signature:** Ed25519  
//! **Freeze-Date:** 2025-12-25  
//! **Governed by:** SPEC-UBL-CORE v1.0  
//!
//! Authority, Consensus and Risk Specification
//! Determines if a link can cross the boundary based on collective authority

#![deny(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// Errors from pact validation
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum PactError {
    /// Unknown pact ID
    #[error("Unknown pact: {0}")]
    UnknownPact(String),

    /// Pact has expired
    #[error("Pact expired")]
    PactExpired,

    /// Insufficient signatures
    #[error("Insufficient signatures: got {got}, need {need}")]
    InsufficientSignatures { got: usize, need: usize },

    /// Unauthorized signer
    #[error("Unauthorized signer: {0}")]
    UnauthorizedSigner(String),

    /// Risk level mismatch
    #[error("Risk mismatch: intent={intent:?}, pact={pact:?}")]
    RiskMismatch { intent: RiskLevel, pact: RiskLevel },
}

/// Result type for pact operations
pub type Result<T> = std::result::Result<T, PactError>;

/// Pact scope (SPEC-UBL-PACT v1.0 §5)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PactScope {
    /// Valid for one container
    Container = 0,
    /// Valid for a namespace
    Namespace = 1,
    /// Valid globally
    Global = 2,
}

/// Risk level (SPEC-UBL-PACT v1.0 §6)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    /// L0 - Observation
    L0 = 0,
    /// L1 - Low impact
    L1 = 1,
    /// L2 - Local impact
    L2 = 2,
    /// L3 - Financial impact
    L3 = 3,
    /// L4 - Systemic impact
    L4 = 4,
    /// L5 - Sovereignty / Evolution
    L5 = 5,
}

impl RiskLevel {
    /// Map from IntentClass to minimum RiskLevel
    pub fn from_intent_class(intent_class: u8) -> Self {
        match intent_class {
            0x00 => RiskLevel::L0, // Observation
            0x01 => RiskLevel::L2, // Conservation
            0x02 => RiskLevel::L4, // Entropy
            0x03 => RiskLevel::L5, // Evolution
            _ => RiskLevel::L0,
        }
    }
}

/// Time window for pact validity (SPEC-UBL-PACT v1.0 §7)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    /// Unix timestamp - not valid before this
    pub not_before: i64,
    /// Unix timestamp - not valid after this
    pub not_after: i64,
}

impl TimeWindow {
    /// Check if current time is within window
    pub fn is_valid(&self, now: i64) -> bool {
        now >= self.not_before && now <= self.not_after
    }
}

/// Pact definition (SPEC-UBL-PACT v1.0 §4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pact {
    /// Pact identifier (Hash32)
    pub pact_id: String,
    
    /// Protocol version
    pub version: u8,
    
    /// Scope of application
    pub scope: PactScope,
    
    /// Minimum threshold of signatures required
    pub threshold: usize,
    
    /// Authorized signers (public keys in hex)
    pub signers: HashSet<String>,
    
    /// Time window
    pub window: TimeWindow,
    
    /// Risk level this pact authorizes
    pub risk_level: RiskLevel,
    
    /// Optional: container ID if scope is Container
    pub container_id: Option<String>,
}

/// Pact proof attached to a link (SPEC-UBL-PACT v1.0 §8)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactProof {
    /// Reference to the pact
    pub pact_id: String,
    
    /// Signatures from authorized signers
    pub signatures: Vec<PactSignature>,
}

/// A single signature in a pact proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactSignature {
    /// Signer's public key (hex)
    pub pubkey: String,
    
    /// Signature (hex)
    pub signature: String,
}

/// Pact registry for validation
pub struct PactRegistry {
    pacts: std::collections::HashMap<String, Pact>,
}

impl PactRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            pacts: std::collections::HashMap::new(),
        }
    }

    /// Register a pact
    pub fn register(&mut self, pact: Pact) {
        self.pacts.insert(pact.pact_id.clone(), pact);
    }

    /// Get a pact by ID
    pub fn get(&self, pact_id: &str) -> Option<&Pact> {
        self.pacts.get(pact_id)
    }

    /// Validate a pact proof (SPEC-UBL-PACT v1.0 §9)
    pub fn validate(
        &self,
        proof: &PactProof,
        intent_class: u8,
        now: i64,
    ) -> Result<()> {
        // Get the pact
        let pact = self
            .get(&proof.pact_id)
            .ok_or_else(|| PactError::UnknownPact(proof.pact_id.clone()))?;

        // Check time window
        if !pact.window.is_valid(now) {
            return Err(PactError::PactExpired);
        }

        // Check risk level
        let required_risk = RiskLevel::from_intent_class(intent_class);
        if pact.risk_level < required_risk {
            return Err(PactError::RiskMismatch {
                intent: required_risk,
                pact: pact.risk_level,
            });
        }

        // Count valid signatures
        let mut valid_count = 0;
        let mut seen_pubkeys = HashSet::new();

        for sig in &proof.signatures {
            // Check for duplicates
            if !seen_pubkeys.insert(&sig.pubkey) {
                continue;
            }

            // Check if signer is authorized
            if !pact.signers.contains(&sig.pubkey) {
                return Err(PactError::UnauthorizedSigner(sig.pubkey.clone()));
            }

            // In a real implementation, we'd verify the signature here
            // For now, we trust the signature is valid
            valid_count += 1;
        }

        // Check threshold
        if valid_count < pact.threshold {
            return Err(PactError::InsufficientSignatures {
                got: valid_count,
                need: pact.threshold,
            });
        }

        Ok(())
    }
}

impl Default for PactRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pact(threshold: usize, signers: Vec<&str>) -> Pact {
        Pact {
            pact_id: "pact_test".to_string(),
            version: 1,
            scope: PactScope::Container,
            threshold,
            signers: signers.into_iter().map(|s| s.to_string()).collect(),
            window: TimeWindow {
                not_before: 0,
                not_after: i64::MAX,
            },
            risk_level: RiskLevel::L2,
            container_id: Some("test".to_string()),
        }
    }

    #[test]
    fn test_valid_pact() {
        let mut registry = PactRegistry::new();
        registry.register(make_pact(2, vec!["alice", "bob", "charlie"]));

        let proof = PactProof {
            pact_id: "pact_test".to_string(),
            signatures: vec![
                PactSignature {
                    pubkey: "alice".to_string(),
                    signature: "sig1".to_string(),
                },
                PactSignature {
                    pubkey: "bob".to_string(),
                    signature: "sig2".to_string(),
                },
            ],
        };

        let result = registry.validate(&proof, 0x01, 1000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_insufficient_signatures() {
        let mut registry = PactRegistry::new();
        registry.register(make_pact(3, vec!["alice", "bob", "charlie"]));

        let proof = PactProof {
            pact_id: "pact_test".to_string(),
            signatures: vec![PactSignature {
                pubkey: "alice".to_string(),
                signature: "sig1".to_string(),
            }],
        };

        let result = registry.validate(&proof, 0x01, 1000);
        assert!(matches!(
            result,
            Err(PactError::InsufficientSignatures { got: 1, need: 3 })
        ));
    }

    #[test]
    fn test_unauthorized_signer() {
        let mut registry = PactRegistry::new();
        registry.register(make_pact(1, vec!["alice", "bob"]));

        let proof = PactProof {
            pact_id: "pact_test".to_string(),
            signatures: vec![PactSignature {
                pubkey: "eve".to_string(),
                signature: "sig1".to_string(),
            }],
        };

        let result = registry.validate(&proof, 0x01, 1000);
        assert!(matches!(result, Err(PactError::UnauthorizedSigner(_))));
    }

    #[test]
    fn test_expired_pact() {
        let mut registry = PactRegistry::new();
        let mut pact = make_pact(1, vec!["alice"]);
        pact.window.not_after = 1000;
        registry.register(pact);

        let proof = PactProof {
            pact_id: "pact_test".to_string(),
            signatures: vec![PactSignature {
                pubkey: "alice".to_string(),
                signature: "sig1".to_string(),
            }],
        };

        let result = registry.validate(&proof, 0x01, 2000);
        assert!(matches!(result, Err(PactError::PactExpired)));
    }

    #[test]
    fn test_risk_mismatch() {
        let mut registry = PactRegistry::new();
        let mut pact = make_pact(1, vec!["alice"]);
        pact.risk_level = RiskLevel::L1; // Too low for Conservation
        registry.register(pact);

        let proof = PactProof {
            pact_id: "pact_test".to_string(),
            signatures: vec![PactSignature {
                pubkey: "alice".to_string(),
                signature: "sig1".to_string(),
            }],
        };

        let result = registry.validate(&proof, 0x01, 1000); // Conservation requires L2
        assert!(matches!(result, Err(PactError::RiskMismatch { .. })));
    }
}