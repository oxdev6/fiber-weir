//! Conditional payment stream primitives.
//!
//! This module defines the protocol boundary between Fiber's TLC settlement
//! engine and a CKB condition verifier. The verifier is intentionally injected
//! by the node because it must query a CKB indexer/RPC and validate the
//! condition's deployed lock script.

use crate::Hash256;
use ckb_hash::blake2b_256;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A deterministic predicate for a CKB cell that must be true before a release.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CkbCellPredicate {
    /// The deployed lock/type script hash that defines the predicate.
    pub script_hash: Hash256,
    /// Script arguments identifying the stream's condition instance.
    pub script_args: Vec<u8>,
}

impl CkbCellPredicate {
    /// Creates a predicate and rejects unbounded script arguments.
    pub fn new(
        script_hash: Hash256,
        script_args: Vec<u8>,
    ) -> Result<Self, ConditionalPaymentError> {
        if script_args.len() > 1024 {
            return Err(ConditionalPaymentError::ConditionArgsTooLarge);
        }
        Ok(Self {
            script_hash,
            script_args,
        })
    }

    /// Returns the commitment included in an invoice and every stream update.
    pub fn commitment(&self) -> Hash256 {
        let mut encoded = Vec::with_capacity(32 + 4 + self.script_args.len());
        encoded.extend_from_slice(self.script_hash.as_ref());
        encoded.extend_from_slice(&(self.script_args.len() as u32).to_le_bytes());
        encoded.extend_from_slice(&self.script_args);
        Hash256::from(blake2b_256(&encoded))
    }
}

/// A CKB observation supplied by the verifier after checking chain data.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VerifiedCondition {
    /// Predicate commitment this observation satisfies.
    pub predicate_commitment: Hash256,
    /// CKB transaction proving the predicate was satisfied.
    pub transaction_hash: Hash256,
    /// Block height containing the proving transaction.
    pub block_number: u64,
}

/// The lifecycle of a conditional payment stream.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ConditionalPaymentStatus {
    /// The stream can release another installment when verified.
    Active,
    /// The stream has released its full amount.
    Completed,
    /// The condition was not proven before expiry; remaining funds refund.
    Refunded,
}

/// A direct two-party conditional payment stream.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConditionalPaymentStream {
    /// Stable stream identifier chosen by the payer.
    pub stream_id: Hash256,
    /// CKB predicate required for every installment.
    pub predicate: CkbCellPredicate,
    /// Amount released for each valid observation.
    pub installment_amount: u128,
    /// Maximum number of installments.
    pub installment_count: u64,
    /// Latest block height at which a release may occur.
    pub expiry_block: u64,
    /// Number of installments already released.
    pub released_installments: u64,
    /// Highest proof block already consumed by the stream.
    pub last_observation_block: Option<u64>,
    /// Current stream status.
    pub status: ConditionalPaymentStatus,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConditionalPaymentError {
    #[error("installment amount must be greater than zero")]
    ZeroInstallmentAmount,
    #[error("installment count must be greater than zero")]
    ZeroInstallmentCount,
    #[error("condition expiry must be in the future")]
    InvalidExpiry,
    #[error("condition script arguments exceed 1024 bytes")]
    ConditionArgsTooLarge,
    #[error("condition observation does not match the stream predicate")]
    InvalidObservation,
    #[error("condition observation is after stream expiry")]
    ObservationAfterExpiry,
    #[error("condition observation is stale or has already been consumed")]
    StaleObservation,
    #[error("stream is no longer active")]
    StreamClosed,
}

impl ConditionalPaymentStream {
    /// Creates an active stream with a fixed installment schedule.
    pub fn new(
        stream_id: Hash256,
        predicate: CkbCellPredicate,
        installment_amount: u128,
        installment_count: u64,
        expiry_block: u64,
        current_block: u64,
    ) -> Result<Self, ConditionalPaymentError> {
        if installment_amount == 0 {
            return Err(ConditionalPaymentError::ZeroInstallmentAmount);
        }
        if installment_count == 0 {
            return Err(ConditionalPaymentError::ZeroInstallmentCount);
        }
        if expiry_block <= current_block {
            return Err(ConditionalPaymentError::InvalidExpiry);
        }
        Ok(Self {
            stream_id,
            predicate,
            installment_amount,
            installment_count,
            expiry_block,
            released_installments: 0,
            last_observation_block: None,
            status: ConditionalPaymentStatus::Active,
        })
    }

    /// Releases exactly one installment after the chain verifier approves it.
    pub fn release(
        &mut self,
        observation: &VerifiedCondition,
        current_block: u64,
    ) -> Result<u128, ConditionalPaymentError> {
        if self.status != ConditionalPaymentStatus::Active {
            return Err(ConditionalPaymentError::StreamClosed);
        }
        if current_block > self.expiry_block {
            self.status = ConditionalPaymentStatus::Refunded;
            return Err(ConditionalPaymentError::ObservationAfterExpiry);
        }
        if observation.predicate_commitment != self.predicate.commitment() {
            return Err(ConditionalPaymentError::InvalidObservation);
        }
        if self
            .last_observation_block
            .is_some_and(|last_block| observation.block_number <= last_block)
        {
            return Err(ConditionalPaymentError::StaleObservation);
        }
        self.last_observation_block = Some(observation.block_number);
        self.released_installments += 1;
        if self.released_installments == self.installment_count {
            self.status = ConditionalPaymentStatus::Completed;
        }
        Ok(self.installment_amount)
    }

    /// Closes the stream and returns the amount that must be refunded.
    pub fn refund(&mut self, current_block: u64) -> Result<u128, ConditionalPaymentError> {
        if self.status != ConditionalPaymentStatus::Active {
            return Err(ConditionalPaymentError::StreamClosed);
        }
        if current_block <= self.expiry_block {
            return Err(ConditionalPaymentError::InvalidExpiry);
        }
        self.status = ConditionalPaymentStatus::Refunded;
        Ok(self.remaining_amount())
    }

    /// Returns the amount that has not yet been released.
    pub fn remaining_amount(&self) -> u128 {
        self.installment_amount.saturating_mul(
            self.installment_count
                .saturating_sub(self.released_installments) as u128,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(value: u8) -> Hash256 {
        Hash256::from([value; 32])
    }

    fn stream() -> ConditionalPaymentStream {
        let predicate = CkbCellPredicate::new(hash(1), vec![2, 3, 4]).unwrap();
        ConditionalPaymentStream::new(hash(9), predicate, 10, 3, 100, 10).unwrap()
    }

    #[test]
    fn releases_only_after_matching_condition_observation() {
        let mut stream = stream();
        let invalid = VerifiedCondition {
            predicate_commitment: hash(8),
            transaction_hash: hash(7),
            block_number: 20,
        };
        assert_eq!(
            stream.release(&invalid, 20),
            Err(ConditionalPaymentError::InvalidObservation)
        );
        assert_eq!(stream.released_installments, 0);

        let valid = VerifiedCondition {
            predicate_commitment: stream.predicate.commitment(),
            transaction_hash: hash(7),
            block_number: 20,
        };
        assert_eq!(stream.release(&valid, 20), Ok(10));
        assert_eq!(
            stream.release(&valid, 20),
            Err(ConditionalPaymentError::StaleObservation)
        );
        assert_eq!(stream.released_installments, 1);
        assert_eq!(stream.status, ConditionalPaymentStatus::Active);
    }

    #[test]
    fn expired_stream_refunds_all_remaining_installments() {
        let mut stream = stream();
        let valid = VerifiedCondition {
            predicate_commitment: stream.predicate.commitment(),
            transaction_hash: hash(7),
            block_number: 20,
        };
        assert_eq!(stream.release(&valid, 20), Ok(10));
        assert_eq!(stream.refund(101), Ok(20));
        assert_eq!(stream.status, ConditionalPaymentStatus::Refunded);
        assert_eq!(stream.remaining_amount(), 20);
    }

    #[test]
    fn final_release_completes_stream_and_blocks_refund() {
        let mut stream = stream();
        let valid = VerifiedCondition {
            predicate_commitment: stream.predicate.commitment(),
            transaction_hash: hash(7),
            block_number: 20,
        };
        assert_eq!(stream.release(&valid, 20), Ok(10));
        assert_eq!(stream.release(&valid, 21), Ok(10));
        assert_eq!(stream.release(&valid, 22), Ok(10));
        assert_eq!(stream.status, ConditionalPaymentStatus::Completed);
        assert_eq!(
            stream.refund(101),
            Err(ConditionalPaymentError::StreamClosed)
        );
    }

    #[test]
    fn release_after_expiry_marks_stream_refunded() {
        let mut stream = stream();
        let valid = VerifiedCondition {
            predicate_commitment: stream.predicate.commitment(),
            transaction_hash: hash(7),
            block_number: 20,
        };
        assert_eq!(
            stream.release(&valid, 101),
            Err(ConditionalPaymentError::ObservationAfterExpiry)
        );
        assert_eq!(stream.status, ConditionalPaymentStatus::Refunded);
    }
}
