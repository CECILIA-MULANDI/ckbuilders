# CKB Builder Track - Week 10

**Week Ending:** 2026-05-22

---

## Focus This Week

Finished Phase 4 of `groth16-ckb`: hardening and audit prep. The verifier already ran end-to-end on testnet last week, so this week was about proving it stays correct under adversarial input. Three strands: fuzzing, property tests, and a written threat model.

**Repo:** [https://github.com/CECILIA-MULANDI/groth16-ckb](https://github.com/CECILIA-MULANDI/groth16-ckb)

---

## Fuzzing

- **`cargo-fuzz` harness** with three libFuzzer targets: `verify_arkworks` (the full verifier) and `decode_vk_molecule` / `decode_witness_molecule` (the two Molecule decoders).
- **Seeded from the committed test vectors** via `scripts/gen-fuzz-seed.sh`. Without seeds the fuzzer just wanders in length-reject territory; with them it gets past the structural checks into the code paths where a real bug would hide.
- **Extracted a `wire-decode` crate.** The Molecule-to-arkworks bridge used to live inside `ckb-script`. I pulled it into its own `no_std` crate so the fuzz targets exercise the exact code that ships on chain, with the same no-panic clippy lints as `verifier-core`.

---

## Property Tests

Added `host/tests/properties.rs` with `proptest`:

- **Soundness probe:** a valid proof must NOT verify against any public-input value other than the one it was generated for. This is the property that matters, since a proof not bound to its public inputs means the verifier is broken.
- **Determinism:** `verify` called twice on the same bytes returns the same `Result`.

The three testing layers now answer different questions: differential ("does it match arkworks on inputs I picked?"), property ("does an invariant hold over inputs I didn't pick?"), and fuzz ("does it ever crash on bytes nobody picked?").

---

## Threat Model

Wrote `docs/threat-model.md` for audit prep: the three on-chain artifacts and spend-time pipeline, assets in severity order, attacker capabilities, and concrete attack scenarios (forged proof, DoS, decoder exploitation, VK substitution, replay) each with its mitigation and a link to the test that exercises it.

The replay section was the one that made me think hardest. The verifier is stateless, so cross-cell replay is a property of *the protocol using the verifier*, not the verifier itself. Writing the threat model honestly meant being precise about what's the verifier's job versus the integrator's.

---

## What I learned

- Fuzzing without good seeds is mostly theatre for structured input. The seed corpus is what turns it from "ran a lot" into "actually explored the decode and pairing paths."
- A threat model is as much about stating non-guarantees as guarantees. The out-of-scope and "integrator's responsibility" parts are what stop a downstream user from assuming protection they don't have.

---

## Current State

| Component                        | Status                                         |
| -------------------------------- | ---------------------------------------------- |
| Phase 0–3                        | Complete (testnet verify confirmed 2026-05-15) |
| `wire-decode` crate              | Extracted, no-panic lints, fuzzed              |
| Fuzz harness (3 targets)         | Seeded from test vectors                       |
| Property tests                   | Soundness + determinism passing                |
| Threat model                     | `docs/threat-model.md`                          |
| Phase 4 (hardening + audit prep) | Complete                                        |
