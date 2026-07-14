# CKB Builder Track - Week 15

**Week Ending:** 2026-06-26

## Focus This Week

Wrote the Spark grant proposal for **zk-Lock**, a reusable CKB lock script that conditions cell spending on a valid Groth16 proof. It builds on the [groth16-ckb](https://github.com/CECILIA-MULANDI/groth16-ckb) verifier I shipped earlier this program; the proposal scopes the missing integration layer (lock script + CLI + reference circuit + tutorial + testnet deployment) that turns the verifier from an on-chain primitive into something developers can actually build with.

**Posted below:**

- Repo: [proposals/spark-proposal.md](proposals/spark-proposal.md)

## What the proposal covers

- **The gap it names.** CKB has a working on-chain Groth16 verifier, but no standard way to lock a cell behind a circuit, no convention for binding cells to specific public inputs, no tooling for a developer to go from "I have a Circom circuit" to "I have CKB locked behind it." Each new use case currently repeats that integration work.
- **Design choices settled during pre-proposal research.** 32-byte `vk_hash` in `lock.args` with the full vkey in a shared `cell_dep` (mirrors the secp256k1 system-script pattern); a `public_inputs_commitment` in `lock.args` so cells bind to specific inputs, not just circuits; a minimal single-spend-path lock with refund/settlement composed via wrapping locks or type scripts rather than baked in; a claimer-binding convention so a proof copied from a pending tx can't be redirected.
- **Scope discipline.** Four weeks solo. $1,704 total ($12/hr × 142h). Testnet-only (Pudge). Web app, wallet integration, mainnet deployment, and public launch event are explicitly out of scope, deferred to keep the Spark milestone defensible.
- **Shipped vs. funded.** `groth16-ckb` is shipped and not funded by this grant; the lock-script primitive, CLI, reference Poseidon-preimage circuit, and tutorial are what the grant funds.
- **Ecosystem alignment.** Complements XuJiandong's SP1 verifier (SP1 for complex computation, zk-Lock for simple circuits); demonstrates cell-model composability (single-purpose lock, layered wrappers) rather than a monolithic vertical stack.
