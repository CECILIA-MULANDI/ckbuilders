# CKB Builder Track - Week 6

**Week Ending:** 2026-04-24

---

## Focus This Week

Completed Phase 3 of Spectre Protocol (guardian-triggered recovery with timelock) and built an on-chain UltraPlonk zk-SNARK verifier for CKB-VM.

**Repos:**

- [spectre-protocol-ckb](https://github.com/CECILIA-MULANDI/spectre-protocol-ckb)
- [ckb-ultraplonk-verifier](https://github.com/CECILIA-MULANDI/ckb-ultraplonk-verifier)

---

## Project: Spectre Protocol - Phase 3 Complete

Phase 3 adds **social recovery** to the agent cell. Guardians can trigger a key rotation on behalf of a locked-out owner, with a timelock cancel window to prevent hostile takeover.

### What was built

- **M-of-N guardian signatures** in `agent-lock` - guardians can collectively authorise a recovery transaction once the threshold is met
- **Recovery state machine** in `agent-type` - enforces valid transitions between `initiate`, `cancel`, and `execute` states
- **Recovery lock script** - a dedicated timelock script that enforces the cancel window and allows the current owner to abort a pending recovery
- **Extended `AgentRecord`** - added `guardians`, `guardian_threshold`, and `pending_owner_pubkey` fields via Molecule
- **Recovery CLI commands** in the relayer - `initiate`, `cancel`, and `execute` subcommands for the full recovery flow
- **CKB Debugger fixtures** - mock fixtures for debugging the agent-type script
- **Integration tests** - covering the recovery flow end-to-end, plus M-of-N guardian signature tests
- **Simple site** - added a basic project site

---

## Project: ckb-ultraplonk-verifier

CKB currently has no production-ready SNARK verifier that runs inside CKB-VM. This is a blocker for any CKB application that needs on-chain ZK verification for it's noir based proofs - including Spectre's future privacy-preserving identity proofs. To try to unblock this, I tinkered with an implemetation from zkVerify and packaged [zkVerify's UltraPlonk verifier](https://github.com/zkVerify/ultraplonk_verifier) (pure Rust, `no_std`) as a deployable CKB script.

**Note:** This is experimental and intended for tinkering/learning only. UltraPlonk has been deprecated in Barretenberg v0.87.0+ in favor of UltraHonk, so newer Noir versions won't produce compatible proofs. A production deployment would need an UltraHonk-compatible verifier.

### Key details

- **147 KB** binary, **~103M cycles** per verification (well within CKB's 3.5B cycle limit)
- Includes test suite with proof test vectors

### Path forward

A more practical near-term path is **Groth16 + Arkworks** - a pure Rust `no_std`-friendly stack. The trade-off is that Noir proof output needs to be converted to an Arkworks-compatible format on the prover side. Similar ideas are being discussed in the community: [noir-lang/discussions#8509](https://github.com/orgs/noir-lang/discussions/8509).

Longer-term, a standalone **UltraHonk-compatible Rust verifier for CKB-VM** would be the proper fix.

---

## Current State

| Component                    | Status   |
| ---------------------------- | -------- |
| Spectre Phase 1 (agent-lock) | Complete |
| Spectre Phase 2 (agent-type) | Complete |
| Spectre Phase 3 (recovery)   | Complete |
| ckb-ultraplonk-verifier      | Complete |

---

## Reviewer Feedback

- **M-of-N bug** in `initiate-recovery.ts` (was only 1-of-N) - fixed in commits `19191ab`, `bd577b7`
- Reviewer confirmed the overall project architecture is correct

---

## Next Steps

- Start on **Noir to Arkworks proof conversion** to unblock Phase 4 via the Groth16 + Arkworks path
- Research approaches from some substrate-based protocols to inform a longer-term standalone UltraHonk verifier for CKB-VM
