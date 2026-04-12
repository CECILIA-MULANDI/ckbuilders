# CKB Builder Track: Week 4

**Week Ending:** 2026-04-10

## Focus This Week

I shifted from tutorial exercises to project-driven learning. From here on, the CKB Builder Handbook curriculum is worked through via **Spectre Protocol**, a key recovery system for autonomous AI agents built on CKB. Each handbook topic is studied and immediately applied to the project.

**Repo:** [https://github.com/CECILIA-MULANDI/spectre-protocol-ckb/](https://github.com/CECILIA-MULANDI/spectre-protocol-ckb/)

## Project: Spectre Protocol

### What It Does

AI agents hold private keys to sign transactions, manage wallets, and control assets. When a key is lost or compromised there is no recovery path; redeploy from scratch and lose all state. Spectre solves this.

Spectre is a **recovery protocol**: the agent owner registers an email commitment against their agent key on CKB. If the key is ever compromised, they recover it by proving in zero knowledge that they control the registered email. No private data is revealed on-chain.

### What Goes On-Chain

```
AgentRecord {
  email_hash,          // blake2b(email_address), commitment not plaintext
  identity_commitment, // blake2b(secret_phrase), v1 second factor
  owner_pubkey,        // current controlling key
  timelock_blocks,     // cancel window for fraudulent recovery attempts
  nonce                // replay protection, increments on every rotation
}
```

ZK proofs are submitted in the transaction witness, verified by the script, then discarded. The chain only stores commitments and current state.

## CKB Work: `agent-lock` Script

**Location:** [https://github.com/CECILIA-MULANDI/spectre-protocol-ckb/tree/master/contracts/spectre-contracts](https://github.com/CECILIA-MULANDI/spectre-protocol-ckb/tree/master/contracts/spectre-contracts)

The first CKB script for Spectre is `agent-lock`, a Rust lock script that controls who can spend (or update) the agent cell. Written using `ckb-std` with native secp256k1 signature verification via the `k256` crate.

### How It Works

The lock stores a **blake160 hash of the owner's public key** in `lock.args` (20 bytes). To spend the cell, the owner provides a secp256k1 signature over the transaction hash in the witness.

```
lock.args  ->  blake160(owner_pubkey)         [20 bytes]
witness    ->  recovery_id [1] + r [32] + s [32] + pubkey [33]  [98 bytes total]
```

Verification steps:

1. Load `lock.args`, must be exactly 20 bytes
2. Load witness, must be exactly 98 bytes
3. Load the transaction hash (what the owner signed)
4. Recover the public key from the signature using secp256k1 recovery
5. Compute `blake160(recovered_pubkey)` and assert it equals `lock.args`
6. Assert the pubkey in the witness matches the recovered key (prevents signature reuse attacks)

If all checks pass, return `0` (success). Otherwise return an error code.

## Key Learnings

### Writing CKB Scripts in Rust (no_std)

CKB scripts run in a constrained RISC-V environment with no standard library and no heap by default. The `ckb-std` crate provides:

- `entry!(program_entry)` macro: sets up the RISC-V entry point
- `default_alloc!`: configures a small heap (needed for crates like `k256`)
- `high_level::load_*`: typed wrappers over CKB syscalls

The no_std constraint means you have to be deliberate about which crates you pull in. `k256` supports `no_std` with `default-features = false`. Not all crypto crates do.

### Signature Recovery vs Verification

A standard ECDSA `verify(pubkey, message, signature)` requires the pubkey upfront. CKB's default lock uses this pattern: the pubkey hash is in `lock.args` and the full pubkey is in the witness.

The `k256` crate's `VerifyingKey::recover_from_prehash` takes a message hash, signature, and recovery ID, and returns the pubkey that produced the signature. You then hash that recovered key and compare to `lock.args`. This avoids storing the full pubkey on-chain; only the 20-byte hash lives in args.

The `prehash` variant is important: the tx hash is already a 32-byte Blake2b digest, so we pass it directly without re-hashing.

### blake160

CKB's native address format uses `blake160`: the first 20 bytes of a `blake2b-256` digest of the compressed (33-byte) pubkey. This is the same scheme used by the default secp256k1 lock, so `agent-lock` is compatible with standard CKB address derivation.
