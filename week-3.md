# CKB Builder Track — Week 3

**Week Ending:** 2026-04-05

## Courses & Reading Completed

- **[The Little Book of ckb-js-vm](https://nervosnetwork.github.io/ckb-js-vm/)** — JavaScript VM that runs inside CKB-VM (RISC-V), enabling TypeScript smart contracts
- **[CKB Script Tutorial: Build a Simple Lock](https://docs.nervos.org/docs/dapp/create-a-simple-lock)** — end-to-end tutorial on writing, testing, deploying and using a custom lock script

## Practical Exercises

| Exercise          | Status | Link                          |
| ----------------- | ------ | ----------------------------- |
| Build Simple Lock | Done   | [simple-lock](./simple-lock/) |

### Build a Simple Lock: [simple-lock/](./simple-lock/)

A full-stack dApp implementing a **hash-lock** contract on CKB. Instead of the default secp256k1 signature check, this lock script protects a cell with a secret preimage: anyone who knows the preimage can spend the cell.

**How it works:** The cell's `lock.args` stores a blake2b hash of a secret preimage. To spend the cell, the spender provides the preimage in the transaction witness. The script loads its own args via `loadScript()`, reads the witness via `loadWitnessArgs()`, hashes the provided preimage with `hashCkb()`, and compares it to the stored hash. If they match, the script returns 0 (success) and the cell can be spent. If not, it returns 11 (rejection). The contract is 25 lines of TypeScript compiled to bytecode and executed by `ckb-js-vm` inside CKB-VM.

**Reference:** [Official tutorial](https://docs.nervos.org/docs/dapp/create-a-simple-lock)

## Key Learnings

### ckb-js-vm: JavaScript on RISC-V

- CKB-VM executes RISC-V binaries. `ckb-js-vm` is a JavaScript engine compiled to RISC-V, letting you write scripts in TypeScript.
- The contract source (`index.ts`) is bundled by esbuild into `hash-lock.js`, then compiled to bytecode (`hash-lock.bc`) for execution.
- `@ckb-js-std/bindings` provides low-level syscalls (`loadScript`, `loadWitnessArgs`, `exit`). `@ckb-js-std/core` adds higher-level helpers (`hashCkb`, `bytesEq`).

### Lock Script vs Type Script (Revisited)

The distinction is very clear

|                    | Lock Script                            | Type Script                                     |
| ------------------ | -------------------------------------- | ----------------------------------------------- |
| **Purpose**        | Authorization:who can spend this cell? | Validation: what rules govern this cell's data? |
| **Required?**      | Yes, every cell must have one          | No, optional                                    |
| **When it runs**   | When the cell is consumed as an input  | When the cell appears as input OR output        |
| **Examples built** | hash-lock (this exercise)              | xUDT (token rules), Spore (DOB rules)           |

## Rust SDK Exploration: [ckb-rust-example/](./ckb-rust-example/)

Started learning the CKB Rust SDK (`ckb-sdk` crate) to prepare for writing CKB scripts natively. Built a working example that:

1. **Decodes a CKB address** into its underlying lock script
2. **Constructs a CKB transfer transaction** — collecting inputs, balancing capacity, signing, and building the final transaction

**Key components of the Rust SDK:**

| Rust SDK Component       | What It Does                          | CCC (JS) Equivalent                   |
| ------------------------ | ------------------------------------- | ------------------------------------- |
| `Address::from_str()`    | Parse address → lock script           | `ccc.Address.fromString()`            |
| `DefaultCellCollector`   | Find live cells for inputs            | `tx.completeInputsByCapacity()`       |
| `CapacityBalancer`       | Balance inputs/outputs, handle change | `tx.completeFeeBy()`                  |
| `SecpCkbRawKeySigner`    | Sign with secp256k1 private key       | `SignerCkbPrivateKey`                 |
| `SecpSighashUnlocker`    | Produce witness for default lock      | Handled by `signer.sendTransaction()` |
| `DefaultCellDepResolver` | Resolve cell_deps from genesis        | `addCellDepsOfKnownScripts()`         |

**Key insight:** The Rust SDK is more explicit than CCC: you manually wire up each component (collector, resolver, signer, unlocker), whereas CCC abstracts most of it. The underlying CKB concepts (cells, lock scripts, witnesses, cell_deps) are identical.

## Next Steps: Spectre Protocol on CKB

Discussed the custom application idea with DevRel and have started working on it. Spectre Protocol is a key recovery system for autonomous AI agents using ZK email proofs.

- **Proposal document:** [Spectre Protocol — CKB Design](https://docs.google.com/document/d/1z9aAIuRlqBfhTIjhkqpb0U9Y1P_fPUuvp0i2Zj2lK_c/edit?usp=sharing)
- Begin with a simplified version: agent cell + custom lock script for key rotation
- Graduate to type script validation and ZK verifier as Intermediate/Advanced topics
