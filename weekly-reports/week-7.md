# CKB Builder Track - Week 7

**Week Ending:** 2026-05-01

---

## Focus This Week

Started a new repo, **`groth16-ckb`**, a Groth16 zkSNARK verifier for CKB-VM built on arkworks. This is the on-chain verifier I sketched at the end of last week, packaged as general CKB infrastructure rather than a Spectre-internal component. Phase 0 (feasibility) is complete and Phase 1 (verifier core hardening) is underway.

**Repo:** [https://github.com/CECILIA-MULANDI/groth16-ckb](https://github.com/CECILIA-MULANDI/groth16-ckb)

---

## Phase 0 - Feasibility Spike

The question was whether current arkworks (0.5+) can compile to `riscv64imac no_std` and verify a real proof on CKB-VM within budget. Nobody appears to have done this on a maintained codebase since arkworks 0.4, so it was a real decision gate.

I built a trivial `x * x = y` R1CS circuit, generated test vectors with `ark-groth16`, and ran the verifier on `ckb-debugger`.

| Metric            | Result                | Bound    |
| ----------------- | --------------------- | -------- |
| Compiles no_std   | yes, no patches       | required |
| Cycles per verify | ~97.5M                | ≤ 250M   |
| Binary size       | 75,576 bytes (~74 KB) | < 4 MB   |
| Heap              | default 1.5 MB OK     | no panic |

For context, the SECBIT 2022 reference verifier ran at ~121M cycles, so the modern stack comes in slightly cheaper. Three risks (compile failure, cycle budget, heap pressure) are now off the project's risk register.

---

## Phase 1 - Verifier Core Hardening

### Architecture split

Three crates: `verifier-core` (no_std, the audit target), `ckb-script` (on-chain binary), and a host workspace (test vectors, differential harness). Splitting out `verifier-core` keeps the audit boundary clean, so the script entry point can iterate without invalidating the audit.

### Differential test harness

The host crate generates random `(vk, proof, public_inputs)` tuples via `ark-groth16` and asserts `verifier-core`'s output matches `ark_groth16::Groth16::verify`. There's a fast default plus an `--ignored` 1000-sample variant.

### Adversarial input rejection

The verifier now explicitly rejects:

| Class                                      | Why it matters                                         |
| ------------------------------------------ | ------------------------------------------------------ |
| Non-canonical Fr / Fq encodings            | Non-canonical = malleable                              |
| G2 points outside the prime-order subgroup | Off-subgroup points break pairing-based soundness      |
| Points at infinity on proof A, B, C        | Always invalid for Groth16                             |
| Points at infinity on VK points            | Shouldn't appear in a well-formed VK                   |
| Public-input count != `vk.ic.len() - 1`    | Returns `PublicInputCountMismatch`, not garbled bytes  |
| Truncated / empty buffers                  | Pre-checked against expected length before deserialise |
| Oversized length prefixes                  | OOM guard - rejects before allocating                  |

Each has a unit test asserting the _exact_ error variant comes back, so the verifier is failing for the right reason and not just failing.

### No-panic discipline

`verifier-core` denies `unwrap`, `expect`, `panic!`, and `unreachable!` via clippy. CI-enforced.

### Wire format

Drafted `schemas/groth16.mol` defining `VerifyingKey`, `Proof`, and `PublicInputs` as versioned, curve-tagged Molecule tables. Vendored the moleculec output as a `groth16-schema` crate plus a `regen-schema.sh` helper, so the build is hermetic and the auditor sees exactly the bytes that ship.

---

## Key Learning

A unit test asserts _I expected this output and got it._ A differential test asserts _my implementation matches a reference implementation across N inputs I didn't pick._ For a verifier, the second is much stronger. The failure mode I'm worried about is "my verifier accepts a proof that arkworks would reject," and only differential testing finds that.

---

## Current State

| Component                   | Status                                                           |
| --------------------------- | ---------------------------------------------------------------- |
| Phase 0 (feasibility spike) | Complete                                                         |
| `verifier-core`             | In progress - input validation hardened, no-panic lints enforced |
| `ckb-script` binary         | Verifies happy-path proof on ckb-debugger                        |
| Differential test harness   | Working - default + 1000-sample `--ignored` variant              |
| Molecule wire format        | Drafted, bindings vendored                                       |

---

## Next Steps

- Wire the Molecule format through `verifier-core` (currently still consumes raw arkworks bytes).
- Cycle benchmarks across `num_public_inputs ∈ {1, 4, 8, 16, 32, 64}`.
- Phase 2: pick the call interface (type script vs. composable component) and write `ckb-testtool` integration tests.
