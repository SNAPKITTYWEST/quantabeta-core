// Layer 6: Bifrost WORM factor registry
// Every alpha factor sealed with SHA-256 chain + audit manifest.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FactorArtifact {
    pub factor_id:            String,
    pub arithmetic_invariant: String,
    pub proof_hash:           String,
    pub code_hash:            String,
    pub sharpe_low:           String,
    pub sharpe_high:          String,
    pub pnl:                  String,
    pub entropy_signature:    String,
    pub operator:             String,
    pub timestamp:            u64,
    pub previous_seal:        Option<String>,
    pub seal:                 String,
}

impl FactorArtifact {
    pub fn new(
        factor_id: impl Into<String>,
        arithmetic_invariant: impl Into<String>,
        proof_hash: impl Into<String>,
        code_hash: impl Into<String>,
        sharpe_low: impl Into<String>,
        sharpe_high: impl Into<String>,
        pnl: impl Into<String>,
        entropy_signature: impl Into<String>,
        operator: impl Into<String>,
        previous_seal: Option<String>,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let factor_id = factor_id.into();
        let arithmetic_invariant = arithmetic_invariant.into();
        let proof_hash = proof_hash.into();
        let code_hash = code_hash.into();
        let pnl = pnl.into();
        let entropy_signature = entropy_signature.into();
        let operator = operator.into();
        let sharpe_low = sharpe_low.into();
        let sharpe_high = sharpe_high.into();

        let content = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}",
            factor_id, arithmetic_invariant, proof_hash, code_hash,
            pnl, entropy_signature, operator, timestamp,
            previous_seal.as_deref().unwrap_or("GENESIS")
        );
        let seal = format!("{:x}", Sha256::digest(content.as_bytes()));

        Self {
            factor_id, arithmetic_invariant, proof_hash, code_hash,
            sharpe_low, sharpe_high, pnl, entropy_signature, operator,
            timestamp, previous_seal, seal,
        }
    }
}

/// Append-only WORM ledger for alpha factors.
pub struct FactorLedger {
    entries: Vec<FactorArtifact>,
}

impl FactorLedger {
    pub fn new() -> Self { Self { entries: vec![] } }

    pub fn append(&mut self, mut artifact: FactorArtifact) -> &FactorArtifact {
        // Wire previous seal for chain continuity
        if let Some(last) = self.entries.last() {
            artifact.previous_seal = Some(last.seal.clone());
            // Recompute seal with chain link
            let content = format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}",
                artifact.factor_id, artifact.arithmetic_invariant,
                artifact.proof_hash, artifact.code_hash,
                artifact.pnl, artifact.entropy_signature,
                artifact.operator, artifact.timestamp,
                last.seal
            );
            artifact.seal = format!("{:x}", Sha256::digest(content.as_bytes()));
        }
        self.entries.push(artifact);
        self.entries.last().unwrap()
    }

    pub fn verify_chain(&self) -> bool {
        for i in 1..self.entries.len() {
            if self.entries[i].previous_seal.as_deref() != Some(&self.entries[i-1].seal) {
                return false;
            }
        }
        true
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn entries(&self) -> &[FactorArtifact] { &self.entries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worm_chain_integrity() {
        let mut ledger = FactorLedger::new();
        let a1 = FactorArtifact::new(
            "QB-HECKE-VOL-0001", "Hecke_T11_Weight2",
            "0xproof1", "0xcode1", "1.82", "1.91", "4523000",
            "0xentropy1", "Ahmad_Ali_Parr", None
        );
        ledger.append(a1);
        let a2 = FactorArtifact::new(
            "QB-PARTITION-VOL-0002", "Ramanujan_p5k4_Congruence",
            "0xproof2", "0xcode2", "1.65", "1.73", "3100000",
            "0xentropy2", "Ahmad_Ali_Parr", None
        );
        ledger.append(a2);
        assert!(ledger.verify_chain());
        assert_eq!(ledger.len(), 2);
    }
}
