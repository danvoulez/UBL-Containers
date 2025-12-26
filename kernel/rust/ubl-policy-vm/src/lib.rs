//! # UBL Policy VM
//!
//! **Title:** SPEC-UBL-POLICY v1.0  
//! **Status:** NORMATIVE  
//! **Change-Control:** STRICT (no retroactive changes)  
//! **Hash:** BLAKE3 | **Signature:** Ed25519  
//! **Freeze-Date:** 2025-12-25  
//! **Governed by:** SPEC-UBL-CORE v1.0, SPEC-UBL-POLICY v1.0  
//!
//! TDLN - Deterministic Translation of Language to Notation
//! Executor WASM determinístico (semantically blind)

#![deny(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors from policy evaluation
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum PolicyError {
    /// Policy not found
    #[error("Policy not found: {0}")]
    PolicyNotFound(String),

    /// Policy execution failed
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// Invalid policy bytecode
    #[error("Invalid bytecode")]
    InvalidBytecode,

    /// Timeout during execution
    #[error("Execution timeout")]
    Timeout,
}

/// Result type for policy operations
pub type Result<T> = std::result::Result<T, PolicyError>;

/// Translation decision from TDLN (SPEC-UBL-POLICY v1.0 §6)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TranslationDecision {
    /// Allow the translation with constraints
    Allow {
        /// Intent class permitted
        intent_class: u8,
        /// Pact required (if any)
        required_pact: Option<String>,
        /// Constraints snapshot
        constraints: Vec<Constraint>,
    },
    /// Deny the translation
    Deny {
        /// Reason for denial
        reason: String,
    },
}

/// A constraint from policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Constraint {
    /// Type of constraint (e.g., "max_delta", "time_window")
    pub kind: String,
    /// Value of the constraint (JSON-serializable)
    pub value: String,
}

/// Policy definition (SPEC-UBL-POLICY v1.0 §4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Policy identifier
    pub policy_id: String,
    
    /// Version
    pub version: String,
    
    /// Hash of the policy bytecode (BLAKE3)
    pub bytecode_hash: String,
    
    /// Compiled WASM bytecode (in production)
    /// For now, we use a simple rule-based system
    #[serde(skip)]
    pub bytecode: Vec<u8>,
    
    /// Human-readable description
    pub description: String,
}

/// Policy evaluation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationContext {
    /// Container ID
    pub container_id: String,
    
    /// Actor (public key or entity ID)
    pub actor: String,
    
    /// Intent payload (JSON)
    pub intent: serde_json::Value,
    
    /// Current state (optional)
    pub state: Option<serde_json::Value>,
    
    /// Timestamp
    pub timestamp: i64,
}

/// Policy VM - executes TDLN policies
pub struct PolicyVM {
    policies: std::collections::HashMap<String, Policy>,
}

impl PolicyVM {
    /// Create a new policy VM
    pub fn new() -> Self {
        Self {
            policies: std::collections::HashMap::new(),
        }
    }

    /// Register a policy
    pub fn register(&mut self, policy: Policy) {
        self.policies.insert(policy.policy_id.clone(), policy);
    }

    /// Evaluate a policy (SPEC-UBL-POLICY v1.0 §6)
    /// 
    /// In a full implementation, this would:
    /// 1. Load the WASM module from bytecode
    /// 2. Execute it in a sandboxed environment
    /// 3. Return the translation decision
    /// 
    /// For now, we implement a simple rule-based system
    pub fn evaluate(
        &self,
        policy_id: &str,
        context: &EvaluationContext,
    ) -> Result<TranslationDecision> {
        let _policy = self
            .policies
            .get(policy_id)
            .ok_or_else(|| PolicyError::PolicyNotFound(policy_id.to_string()))?;

        // Simple rule-based evaluation
        // In production, this would execute WASM
        
        // Extract intent type from context
        let intent_type = context
            .intent
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        // Simple rules based on intent type
        match intent_type {
            "observe" | "read" => Ok(TranslationDecision::Allow {
                intent_class: 0x00, // Observation
                required_pact: None,
                constraints: vec![],
            }),
            
            "transfer" | "send" => {
                // Check for amount limits
                let amount = context
                    .intent
                    .get("amount")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                
                if amount > 10000 {
                    // Large transfers require a pact
                    Ok(TranslationDecision::Allow {
                        intent_class: 0x01, // Conservation
                        required_pact: Some("high_value_transfer".to_string()),
                        constraints: vec![Constraint {
                            kind: "max_amount".to_string(),
                            value: "10000".to_string(),
                        }],
                    })
                } else {
                    Ok(TranslationDecision::Allow {
                        intent_class: 0x01, // Conservation
                        required_pact: None,
                        constraints: vec![],
                    })
                }
            }
            
            "create" | "mint" => Ok(TranslationDecision::Allow {
                intent_class: 0x02, // Entropy
                required_pact: Some("creation_authority".to_string()),
                constraints: vec![],
            }),
            
            "evolve" | "upgrade" => Ok(TranslationDecision::Allow {
                intent_class: 0x03, // Evolution
                required_pact: Some("evolution_l5".to_string()),
                constraints: vec![Constraint {
                    kind: "risk_level".to_string(),
                    value: "L5".to_string(),
                }],
            }),
            
            _ => Ok(TranslationDecision::Deny {
                reason: format!("Unknown intent type: {}", intent_type),
            }),
        }
    }
}

impl Default for PolicyVM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_context(intent_type: &str, amount: Option<i64>) -> EvaluationContext {
        let mut intent = json!({"type": intent_type});
        if let Some(amt) = amount {
            intent["amount"] = json!(amt);
        }
        
        EvaluationContext {
            container_id: "test".to_string(),
            actor: "alice".to_string(),
            intent,
            state: None,
            timestamp: 1000,
        }
    }

    #[test]
    fn test_observe_allows_observation() {
        let mut vm = PolicyVM::new();
        vm.register(Policy {
            policy_id: "default".to_string(),
            version: "1.0".to_string(),
            bytecode_hash: "test".to_string(),
            bytecode: vec![],
            description: "Default policy".to_string(),
        });

        let context = make_context("observe", None);
        let decision = vm.evaluate("default", &context).unwrap();

        match decision {
            TranslationDecision::Allow { intent_class, .. } => {
                assert_eq!(intent_class, 0x00);
            }
            _ => panic!("Expected Allow"),
        }
    }

    #[test]
    fn test_small_transfer_no_pact() {
        let mut vm = PolicyVM::new();
        vm.register(Policy {
            policy_id: "default".to_string(),
            version: "1.0".to_string(),
            bytecode_hash: "test".to_string(),
            bytecode: vec![],
            description: "Default policy".to_string(),
        });

        let context = make_context("transfer", Some(100));
        let decision = vm.evaluate("default", &context).unwrap();

        match decision {
            TranslationDecision::Allow {
                intent_class,
                required_pact,
                ..
            } => {
                assert_eq!(intent_class, 0x01); // Conservation
                assert!(required_pact.is_none());
            }
            _ => panic!("Expected Allow"),
        }
    }

    #[test]
    fn test_large_transfer_requires_pact() {
        let mut vm = PolicyVM::new();
        vm.register(Policy {
            policy_id: "default".to_string(),
            version: "1.0".to_string(),
            bytecode_hash: "test".to_string(),
            bytecode: vec![],
            description: "Default policy".to_string(),
        });

        let context = make_context("transfer", Some(20000));
        let decision = vm.evaluate("default", &context).unwrap();

        match decision {
            TranslationDecision::Allow {
                required_pact, ..
            } => {
                assert!(required_pact.is_some());
            }
            _ => panic!("Expected Allow with pact"),
        }
    }

    #[test]
    fn test_evolution_requires_l5_pact() {
        let mut vm = PolicyVM::new();
        vm.register(Policy {
            policy_id: "default".to_string(),
            version: "1.0".to_string(),
            bytecode_hash: "test".to_string(),
            bytecode: vec![],
            description: "Default policy".to_string(),
        });

        let context = make_context("evolve", None);
        let decision = vm.evaluate("default", &context).unwrap();

        match decision {
            TranslationDecision::Allow {
                intent_class,
                required_pact,
                ..
            } => {
                assert_eq!(intent_class, 0x03); // Evolution
                assert_eq!(required_pact, Some("evolution_l5".to_string()));
            }
            _ => panic!("Expected Allow with L5 pact"),
        }
    }

    #[test]
    fn test_unknown_intent_denies() {
        let mut vm = PolicyVM::new();
        vm.register(Policy {
            policy_id: "default".to_string(),
            version: "1.0".to_string(),
            bytecode_hash: "test".to_string(),
            bytecode: vec![],
            description: "Default policy".to_string(),
        });

        let context = make_context("hack_the_planet", None);
        let decision = vm.evaluate("default", &context).unwrap();

        assert!(matches!(decision, TranslationDecision::Deny { .. }));
    }
}