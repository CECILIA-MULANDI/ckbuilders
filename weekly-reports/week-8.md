# CKB Builder Track - Week 8

**Week Ending:** 2026-05-08

---

## Focus This Week

Continued on `groth16-ckb`. Two pieces landed: TypeScript bindings for the Molecule wire format, and the on-chain script's entry point wired to consume those Molecule bytes and call `verifier-core::verify`. This closes the loop between off-chain encoding and on-chain decoding.

**Repo:** [https://github.com/CECILIA-MULANDI/groth16-ckb](https://github.com/CECILIA-MULANDI/groth16-ckb)

---

## TypeScript Bindings

`regen-schema.sh` now emits both Rust and TypeScript bindings from `schemas/groth16.mol`, via `moleculec-es` over the JSON IR that `moleculec` produces. The generated output is committed rather than built on the fly, same reasoning as vendoring the Rust bindings: hermetic build, reviewable bytes.

This unblocks the off-chain side. A CCC-based transaction builder can now produce molecule-encoded `Groth16VerifyingKey` and `Groth16Witness` payloads without hand-rolling layout code.

---

## On-chain Entry Point

The script binary used to be a Phase 0 shim around `verifier-core`. It is now an actual CKB script.

- **VK from `cell_deps`.** The script `args` is a 32-byte blake2b256 of the intended VK cell's data. The script walks `Source::CellDep` for a matching cell, so one deployed verifier accepts proofs only for the circuit it was bound to.
- **Proof + public inputs from the witness.** Loaded from `WitnessArgs.input_type`, decoded as `Groth16Witness`.
- **Decode and verify.** Each Molecule structure is version-checked (`v1` only), re-serialised into arkworks layout, and passed to `verifier_core::verify`. Molecule readers do structural validation at `from_slice` time, so the soundness checks in `verifier-core` are doing real cryptographic work rather than catching layout typos.
- **Distinct exit codes** for each failure (`ERROR_VK_CELL_NOT_FOUND`, `ERROR_VK_MOLECULE_DECODE`, `ERROR_VERSION_MISMATCH`, `ERROR_VERIFICATION_FAILED`, etc.) so the caller can tell *why* a transaction failed without parsing logs.

Curve unions are exhaustive on BN254 today; an `ERROR_UNSUPPORTED_CURVE` slot is reserved for when more curves are added.

---

## Current State

| Component                  | Status                                              |
| -------------------------- | --------------------------------------------------- |
| `ckb-script` binary        | Consumes Molecule VK + witness, calls verifier      |
| Rust schema bindings       | Vendored                                            |
| TypeScript schema bindings | Generated and committed                             |
| `regen-schema.sh`          | Emits both Rust and TypeScript                      |

---

## Next Steps

- `ckb-testtool` integration tests: valid proof, invalid proof, malformed Molecule, missing VK cell, wrong VK reference, version mismatch.
- Cycle benchmarks across `num_public_inputs ∈ {1, 4, 8, 16, 32, 64}`.
- TypeScript helper around the generated bindings: encoders plus a transaction-builder helper that places the VK in a cell_dep and the proof in the witness.
