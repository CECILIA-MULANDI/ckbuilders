# CKB Builder Track - Week 9

**Week Ending:** 2026-05-15

---

## Focus This Week

Finished Phases 2 and 3 of `groth16-ckb`. The verifier now runs end-to-end on CKB Pudge testnet: a real transaction that submits a Groth16 proof and gets accepted on chain.

**Repo:** [https://github.com/CECILIA-MULANDI/groth16-ckb](https://github.com/CECILIA-MULANDI/groth16-ckb)

**Evidence:** testnet verify tx [`0xc658f9e8…3c96`](https://pudge.explorer.nervos.org/transaction/0xc658f9e8a846747a3aad373b2145ac49d920aba00c88885219d6c25f3dc53c96)

---

## Phase 2 - Integration tests, benchmarks, reproducible build

- **`ckb-testtool` harness** covering happy path plus rejection cases (forged proof, malformed Molecule, missing VK cell, wrong VK reference, version mismatch).
- **Cycle benchmark sweep** across `num_public_inputs ∈ {1, 4, 8, 16, 32, 64}`. The fixed pairing cost is ~102M cycles at N=1, and each extra public input adds ~270k–300k cycles for one G1 scalar multiplication. At N=64 it's still under 50% of the 250M project bound.

| num_public_inputs |      cycles | % of 250M |
| ----------------: | ----------: | --------: |
|                 1 | 102,419,769 |      41.0 |
|                 4 | 103,234,891 |      41.3 |
|                 8 | 104,285,448 |      41.7 |
|                16 | 106,588,487 |      42.6 |
|                32 | 111,343,646 |      44.5 |
|                64 | 121,128,923 |      48.5 |

- **Reproducible build.** Pinned toolchain via `rust-toolchain.toml`, `--locked` dependencies, and source-path remapping so build paths don't leak into the binary. `scripts/verify-reproducible.sh` does two clean builds and confirms byte-identical output. This matters for the eventual audit, since the auditor needs to be able to rebuild and match the code-hash.

---

## Phase 3 - Host SDKs + reference example on testnet

- **Host Rust crate** now emits Molecule fixtures plus the blake2b256 `data_hash` of the VK (which is what the script `args` has to be).
- **TypeScript SDK** (`sdk/ts`) wraps the generated bindings: encoders for `Groth16VerifyingKey` / `Groth16Witness` and a transaction-builder helper that places the VK in a `cell_dep` and the proof in `WitnessArgs.input_type`.
- **`examples/square-root`** is the end-to-end reference flow: deploy verifier, deploy VK, create a trigger cell, spend it with a proof. Has a dry-run mode that prints the transactions against placeholder OutPoints, plus a live testnet mode.
- **Cell-creation gate.** I had to change the ckb-script so creation is permitted when there's no `Source::GroupInput`. Without this the verifier would refuse to let the trigger cell exist in the first place, because there's no proof to check at creation time. Verification happens on spend.

---

## What was tricky

- **The "creation vs spend" framing.** I initially had the verifier reject any transaction where it couldn't find a witness, which broke cell creation. The fix once I saw it is small (return 0 when there's no group input), but figuring out _why_ my deploy transaction was failing took a while. Two-transaction flows are a different mental model from a single-call verifier.
- **Reproducible builds.** Source-path remapping was the non-obvious piece. Without `--remap-path-prefix`, the absolute path of `~/.cargo/registry/...` ends up baked into debug info, and the binary hash diverges between machines even with the same toolchain and lockfile.

---

## What I learned

- A type script's job at _creation_ and at _spend_ are genuinely different. CKB's model lets you express that asymmetry directly (group input vs. no group input), which is cleaner than I expected.
- Cycle cost scales linearly in public-input count with a tiny slope. Useful framing for downstream users: pick your circuit so the fixed pairing cost dominates, and stop worrying about a few extra public inputs.
- Phase 0's 97.5M-cycle estimate was for the bare verifier; the production call path with Molecule decoding sits at ~102M for the same circuit shape. The Molecule overhead is small (~5M cycles), which is what I was hoping for when I picked it.

---

## Current State

| Component                         | Status                                            |
| --------------------------------- | ------------------------------------------------- |
| Phase 0 (feasibility)             | Complete                                          |
| Phase 1 (verifier-core hardening) | Complete                                          |
| Phase 2 (script + tests + bench)  | Complete                                          |
| Phase 3 (SDKs + testnet example)  | Complete - testnet verify tx confirmed 2026-05-15 |
| Phase 4 (audit prep)              | Next                                              |

---

## Next Steps

- Start Phase 4: fuzzing (`cargo-fuzz`), property tests (`proptest`), and a written `THREAT_MODEL document`.
