# fiber-cond Completion Roadmap

This is the completion checklist for turning the current demonstrable state-machine slice into a real, production-ready conditional payment product.

## Current status

The repository currently has:

- A Rust state-machine prototype in `fiber-types::conditional`.
- Unit tests for matching observations, stale-proof rejection, completion, expiry, and refund state.
- A browser dashboard that demonstrates one sample stream using local in-memory data.

The repository does **not** yet have a live conditional payment protocol integrated into Fiber, a CKB contract implementation, a verifier, or a production dashboard backend.

## P0: Define the protocol before coding

- [ ] Decide the exact condition semantics: what CKB cell or transaction predicate means true.
- [ ] Define whether the condition is checked once per installment or once for the entire stream.
- [ ] Define monotonicity: a condition must not become valid again after invalidation unless explicitly supported.
- [ ] Define finality policy: required confirmation depth, reorg behavior, and indexer/RPC disagreement behavior.
- [ ] Define the expiry rule and the exact refund race at the expiry boundary.
- [ ] Define who can submit a proof and how the payee proves a release request is authorized.
- [ ] Define whether direct two-party streams are the only v1 scope; defer routing, MPP, UDT, and trampoline support unless explicitly required.
- [ ] Version the protocol and document backward-compatibility behavior for peers that do not understand conditional streams.

## P0: CKB contract and on-chain enforcement

- [ ] Add or version the `CommitmentLock` contract in the `fiber-scripts` repository.
- [ ] Add a witness format containing the conditional predicate commitment, stream identity, installment index, expiry, and settlement branch.
- [ ] Enforce the predicate in the lock script, not only in node Rust code.
- [ ] Add the valid-condition release branch.
- [ ] Add the invalid/expired timeout refund branch.
- [ ] Reject reused installment indices and reused condition proofs on-chain.
- [ ] Bind the condition to the channel commitment state so neither party can swap the predicate after signing.
- [ ] Publish and version the contract code hash and cell dependencies.
- [ ] Deploy the contract to devnet/testnet and record deployment metadata.
- [ ] Update mainnet configuration only after an audited deployment exists.
- [ ] Test force-close and settlement transactions directly against the deployed contract.

## P0: Fiber protocol integration

- [ ] Add conditional-stream fields to invoice JSON types and invoice encoding.
- [ ] Add conditional-stream fields to the signed invoice preimage.
- [ ] Add an explicit conditional-payment feature bit.
- [ ] Extend `AddTlc`/`TlcInfo` and the Molecule wire schema with the stream commitment and installment data.
- [ ] Define peer behavior when a remote node does not support the feature.
- [ ] Propagate conditional data through the payment command and network actor.
- [ ] Prevent conditional payments from silently entering ordinary hash-lock settlement.
- [ ] Integrate condition verification before `RemoveTlcFulfill` is emitted.
- [ ] Make failed verification halt the stream without releasing the current installment.
- [ ] Make expiry/cancellation release the normal refund path for all remaining installments.
- [ ] Persist stream state, consumed proof blocks, installment index, and status in the node store.
- [ ] Add migrations and recovery behavior for restart during verification or settlement.
- [ ] Add idempotency for duplicate proof submissions and settlement retries.
- [ ] Add crash recovery for the states: funded, proof-pending, released, halted, refund-pending, refunded, completed.

## P0: CKB verifier service

- [ ] Implement a production CKB RPC/indexer client.
- [ ] Verify script hash and script args against the committed predicate.
- [ ] Verify the proving transaction exists and is included in a block.
- [ ] Verify the transaction actually satisfies the predicate's data and witness rules.
- [ ] Enforce confirmation depth and chain reorganization handling.
- [ ] Reject stale, replayed, conflicting, and duplicate observations.
- [ ] Cache verified observations with invalidation on reorg.
- [ ] Define timeouts, retries, backoff, and fail-closed behavior when CKB is unavailable.
- [ ] Add metrics for verification latency, failures, reorgs, and rejected proofs.

## P0: Watchtower and force-close safety

- [ ] Extend watchtower `SettlementData` and `SettlementTlc` with conditional fields.
- [ ] Teach watchtower witness parsing to recognize conditional streams.
- [ ] Select the release branch only for a valid on-chain condition proof.
- [ ] Select timeout refund for a false or expired condition.
- [ ] Prevent an old commitment from releasing funds under a newer condition.
- [ ] Add watchtower tests with the watchtower offline during force-close.
- [ ] Add tests for stale proofs, conflicting proofs, chain reorgs, and expiry races.

## P1: RPC and application API

- [ ] Add `create_conditional_stream` RPC.
- [ ] Add `get_conditional_stream` RPC.
- [ ] Add `list_conditional_streams` RPC with pagination and filters.
- [ ] Add `submit_condition_proof` RPC.
- [ ] Add `halt_conditional_stream` RPC for operator cancellation.
- [ ] Add `refund_conditional_stream` RPC with expiry/race validation.
- [ ] Add `get_conditional_stream_events` RPC.
- [ ] Return stable typed errors and machine-readable status values.
- [ ] Add authentication, authorization, request limits, and audit logging.
- [ ] Generate and review RPC documentation.

## P1: Real dashboard product

- [ ] Replace all hard-coded dashboard data with authenticated RPC/API calls.
- [ ] Add a backend adapter that maps Fiber RPC responses to dashboard models.
- [ ] Add create-stream form validation against node capabilities and available balance.
- [ ] Show real payer/payee identities and channel selection.
- [ ] Show predicate construction and a human-readable condition summary.
- [ ] Show funding transaction and commitment transaction links.
- [ ] Show proof verification status, confirmation depth, and last checked block.
- [ ] Show every stream state with clear next actions.
- [ ] Add live updates through polling or WebSocket/SSE subscriptions.
- [ ] Add retry, offline, loading, empty, stale-data, and error states.
- [ ] Add real pagination/filtering for streams and settlement events.
- [ ] Add confirmation dialogs for cancellation and refund actions.
- [ ] Remove demo-only controls and sample identities before release.
- [ ] Add wallet/key-management integration without exposing private keys in the browser.
- [ ] Add responsive accessibility support, keyboard navigation, focus management, and screen-reader labels.
- [ ] Add visual regression tests across desktop and mobile.

## P1: Security and correctness

- [ ] Threat-model payer, payee, verifier, indexer, peer, watchtower, and browser compromise.
- [ ] Audit all proof parsing and script verification paths.
- [ ] Use bounded input sizes and strict serialization limits.
- [ ] Add replay, downgrade, signature, authorization, and double-release tests.
- [ ] Verify arithmetic overflow behavior for total escrow and installment amounts.
- [ ] Verify all status transitions are monotonic and persisted atomically.
- [ ] Add property tests for the stream state machine.
- [ ] Fuzz invoice, Molecule, proof, and witness parsers.
- [ ] Run dependency, secret, and license scanning.
- [ ] Commission an independent smart-contract and protocol review before mainnet use.

## P1: End-to-end test matrix

- [ ] Two-node happy path: fund, verify, release all installments, complete.
- [ ] Two-node false-condition path: fund, fail verification, halt, refund.
- [ ] Partial stream path: release some installments, then expire and refund remainder.
- [ ] Invalid predicate commitment.
- [ ] Invalid script hash or args.
- [ ] Missing transaction inclusion.
- [ ] Insufficient confirmation depth.
- [ ] Replayed proof block.
- [ ] Conflicting proof transactions.
- [ ] Reorg after a proof is accepted.
- [ ] Duplicate RPC requests and node restart at every transition.
- [ ] Peer disconnect during add, proof, release, and refund.
- [ ] Force-close with pending conditional installments.
- [ ] Watchtower offline and then recovering.
- [ ] Unsupported-peer negotiation and clean rejection.
- [ ] Browser/API contract tests against a real node.

## P2: Operations and release

- [ ] Add structured logs with stream ID, channel ID, proof transaction, block, and transition.
- [ ] Add Prometheus metrics and health/readiness endpoints.
- [ ] Add alerts for stuck proof verification, expiry approaching, failed refunds, and reorgs.
- [ ] Add configuration for CKB RPC, indexer, confirmation depth, and verifier timeouts.
- [ ] Add database backup/restore guidance.
- [ ] Add devnet bootstrap scripts and deterministic demo fixtures.
- [ ] Add testnet deployment runbook and rollback plan.
- [ ] Add versioned migration and protocol upgrade documentation.
- [ ] Add user, operator, API, and threat-model documentation.
- [ ] Define support policy and incident response for stuck funds.
- [ ] Produce a release checklist with reproducible builds and signed artifacts.

## Definition of done

The project is not complete until all of the following are true:

1. A payer can create a stream through the real RPC/API.
2. Funds are locked in a real Fiber channel/commitment state.
3. A payee can submit a proof that is independently verified against CKB.
4. A valid, fresh proof releases exactly one authorized installment.
5. An invalid, stale, missing, or reorged proof releases nothing.
6. Expiry or condition failure automatically makes the unreleased balance refundable.
7. Force-close and watchtower settlement enforce the same rules on-chain.
8. The dashboard displays live node state and no longer contains sample-only claims.
9. Two-node and on-chain integration tests pass.
10. The contract and protocol have been reviewed for funds-safety before deployment.
