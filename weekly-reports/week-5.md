# CKB Builder Track - Week 5

**Week Ending:** 2026-04-17

---

## Focus This Week

This week I finished Phase 2 of Spectre Protocol. The main goal was to add a type script alongside the lock script I built last week, so the agent cell now enforces both who can spend it and what the resulting state has to look like. I also had to learn Molecule serialisation properly since the type script reads and validates on-chain data.

**Repo:** [https://github.com/CECILIA-MULANDI/spectre-protocol-ckb/](https://github.com/CECILIA-MULANDI/spectre-protocol-ckb/)

---

## Courses & Reading Completed

- **[Understanding Molecule and Serialization](https://docs.nervos.org/docs/serialization/molecule)** - CKB's binary encoding format; I needed this to write the type script correctly
- **[Script Development Course - Class 6: Type ID](https://nervosnetwork.github.io/capsule-cookbook/script-development/type-id.html)** - how to give a script cell a stable identity so agent cells can still reference it after an upgrade
- **[CKB Script Tutorial: Validation Model](https://nervosnetwork.github.io/ckb-script-tutorials/validation-model.html)** - a clearer picture of how lock and type scripts run together on the same transaction

---

## Project: Spectre Protocol - Phase 2 Complete

Phase 2 adds a **state enforcement layer** to the agent cell. The cell now carries two scripts: `agent-lock` handles who can spend it, and `agent-type` handles what the output state has to be. Together they make sure key rotations are valid and can't be replayed.

---

### `agent-type` - CKB Type Script

This is a Rust type script that runs whenever a transaction touches the agent cell. It checks that the transition from input state to output state is actually valid.

What it enforces on-chain:

- The output cell data has to be a properly encoded `AgentRecord`
- The `nonce` in the output must be exactly `nonce + 1` from the input - this is what prevents replay attacks
- The `owner_pubkey` in the record has to match `lock.args` on the output cell, so the lock and type scripts always agree on who the owner is

```
contracts/spectre-contracts/agent-type/src/main.rs
```

### `spectre-types` - Shared Molecule Bindings

I pulled all the Molecule-generated types (`AgentRecord`, `Uint32`, `Bytes32`, `Bytes33`) into their own shared crate. Both the on-chain script and the off-chain CLI use it, so there's no risk of them disagreeing on the wire format.

**Molecule schema:**

```molecule
table AgentRecord {
  email_hash:          Bytes32,
  identity_commitment: Bytes32,
  owner_pubkey:        Bytes33,
  timelock_blocks:     Uint32,
  nonce:               Uint32,
}
```

### Integration Tests

I wrote integration tests using `ckb-testtool` to cover the cases that matter:

| Test Case | Expected |
| --- | --- |
| Valid key rotation (nonce increments, pubkey updates) | Pass |
| Stale nonce (output nonce = input nonce) | Reject |
| Lock/type pubkey mismatch | Reject |
| Malformed cell data | Reject |

### Relayer CLI: Molecule Encode/Decode

I updated the TypeScript relayer so the `create` and `rotate` commands encode `AgentRecord` correctly before building the transaction. There's a separate `molecule.ts` module for the encoding logic so it stays clean and separate from the transaction construction code.

---

## Key Learnings

### Molecule Serialisation

Molecule is CKB's canonical binary encoding, built for deterministic zero-copy decoding inside CKB-VM. Before this week I had only seen it from the outside; actually using it made the design choices click.

- `struct` is for fixed-size types - fields are packed tightly with no headers, fast to decode
- `table` is for dynamic-size types - there's a header that stores field offsets so the reader can jump directly to any field. `AgentRecord` uses `table` so fields can be added later without breaking existing cells
- You don't write serialisation code by hand; the `moleculec` compiler generates typed Rust builders and readers from the `.mol` schema
- The reason I put the bindings in a shared crate is that if the on-chain script and the off-chain CLI are generated from different schemas, you get silent data corruption. One crate, one schema.

### Type ID

Type ID is a CKB primitive that gives a cell a **stable identity** even as its contents change. The ID is assigned when the cell is first created and never changes, even if you update the script code inside the cell.

This is important for Spectre because agent cells reference `agent-type` by its Type ID. If I need to upgrade the type script logic, I update the script cell but the Type ID stays the same - all existing agent cells pick up the new logic automatically with no migration needed.

### How Lock and Type Scripts Compose

I had a rough understanding of this before but building both scripts for the same cell made it concrete. The lock script only sees inputs - it authorises the spend. The type script sees both inputs and outputs for any cell carrying its Type ID - this is what lets me enforce the nonce increment. The type script reads the nonce from the input `AgentRecord`, checks the output nonce is exactly one higher, and rejects anything else.

---

## Current State

| Component | Status |
| --- | --- |
| `agent-lock` | Complete - secp256k1 sig check, blake160 args |
| `agent-type` | Complete - nonce enforcement, state validation, Type ID |
| `spectre-types` | Complete - shared Molecule bindings |
| Relayer CLI | Updated - Molecule encode/decode, testnet deploy/create/rotate |
| Integration tests | Passing |

---

## Next Steps

Moving into Phase 3: guardian-triggered recovery with a time-lock.

- Set up CKB Debugger for on-chain script debugging
- Design and implement m-of-n guardian signature scheme
- Build the staged recovery flow with a cancel window (the timelock blocks field in `AgentRecord` finally gets used here)
- Add CLI commands for guardian registration and recovery initiation
