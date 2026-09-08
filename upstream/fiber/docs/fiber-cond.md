# fiber-cond: Conditional Payment Stream Slice

## What is implemented

`fiber-types::conditional` contains the protocol state machine for a direct, two-party CKB conditional payment stream:

- A stream commits to a CKB script hash and script arguments.
- The predicate commitment is deterministic and can be embedded in an invoice/TLC update.
- Each installment requires a matching `VerifiedCondition` observation.
- A mismatched observation cannot release funds.
- A stream becomes `Refunded` after expiry and returns the remaining amount.
- A completed or refunded stream cannot be released or refunded again.

The tests cover invalid observations, valid installment release, completion, expiry, and refund behavior.

## Security invariant

Funds are released only after the stream receives an observation whose predicate commitment exactly matches the stream commitment. The state machine never accepts an arbitrary boolean from the caller.

The `VerifiedCondition` value is the boundary for the CKB adapter. A production node must construct it only after querying CKB and verifying the deployed predicate script, transaction inclusion, and confirmation policy.

## Integration boundary

This repository currently contains Fiber's TLC/hold-invoice settlement machinery but does not contain the external `fiber-scripts` contract source needed to enforce a new arbitrary predicate during force-close. Therefore this slice deliberately keeps the contract adapter explicit rather than pretending local Rust validation is on-chain enforcement.

The remaining integration work is:

1. Add a versioned `CommitmentLock` witness variant carrying the predicate commitment.
2. Add the conditional fields to the invoice/TLC wire schemas with a compatibility/version bit.
3. Implement the CKB verifier adapter and use it before `RemoveTlcFulfill`.
4. Update the watchtower settlement witness and deployed testnet/mainnet script configuration.
5. Add a two-node integration test against the upgraded contract.

The direct state machine is ready to be used by those boundaries; routing, MPP, UDTs, and trampoline payments are intentionally out of scope for the first slice.
