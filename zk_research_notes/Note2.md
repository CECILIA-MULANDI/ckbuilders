## Dedup without nullifiers: what this design borrows from CKB

**NOTE:** _These are my personal research notes documenting what I am learning and finding as I explore zero-knowledge proofs on CKB. Discussion and feedback welcome._

## Continuing from Note 1

We closed [Note 1](https://talk.nervos.org/t/research-notes-what-zero-knowledge-proofs-enable-on-ckb/10368) on an open question. The soundness story of this voting design holds up: votes can't be omitted, deposits can't be double-counted. Privacy is a separate question, and the answer there is different. A public observer watching DAO deposits and vote cells over a voting window can correlate the two and read off who voted what. I left it there because layering anonymity onto a working design is a different question than the one I started with.

This note is not a privacy proposal. Before asking "how would we add privacy?", I wanted to understand what dedup primitive this design uses instead of nullifiers, and why it didn't need to manufacture an anonymous one. The answer turns out to be a useful design lesson in its own right. The privacy question itself, how to layer anonymity onto a working design, is for a later note.

## Nullifiers: what they are, what they do

A nullifier is a one-time tag that says _"this secret has been used."_ It's the workhorse primitive behind anonymous double-vote-resistant voting systems like Semaphore and MACI, borrowed from how Zcash prevents double-spending.

_(For a working code example of nullifier derivation and verification, I built one [here](https://github.com/CECILIA-MULANDI/nullifiers). The rest of this post assumes you know the basic mechanism.)_

The recipe is usually:

```
nullifier = hash(secret_key, context)
```

where `context` scopes the action: a proposal ID, an epoch, whatever identifies "the thing you're voting on." From this construction you get four properties:

- **Deterministic**: same secret + same context produces the same nullifier
- **Unique**: different secrets produce different nullifiers
- **Unlinkable**: the nullifier reveals nothing about the secret or the voter
- **Unforgeable**: only the secret-holder can produce the right nullifier

In a voting flow, the voter's secret commitment sits in a Merkle tree of eligible voters. When they cast a vote, they publish the nullifier alongside a zk proof of two things:

- **Membership**: they know a secret whose commitment is in the tree
- **Honest derivation**: the nullifier was computed from that secret using the recipe above, not invented

The verifier maintains a seen-set; if the nullifier is already there, the second vote is rejected.

The reason this primitive is everywhere in zk voting is what it manufactures: **unique-but-anonymous identity**. The chain can tell _"whoever this is, they've voted"_ without knowing _who_ they are.

---

## But this design skips them

So I went looking for the nullifier scheme in this design. Specifically: a Merkle tree of voter commitments, a `nullifier` field on each vote, a `seen_tags` set in the proof's public values, a circuit deriving a one-time tag from a voter secret. I mean the usual furniture!

None of it is there.

The proposal cell's `args` hold a Type ID and an SP1 verifying key hash. The vote cell's data is `{ vote, amount, dao_index }`. The proof's `PublicValues` are `{ proposal, start_block_hash, end_block_hash, proposal_script, passed, yes_vote, no_vote }`. There is no Poseidon hash anywhere in the SP1 guest, no Merkle membership proof, no commitment to a set of secrets.

The guest does build Merkle trees, but for a different job: verifying each block's `transactions_root` field against the actual transactions in that block. Same data structure, different purpose. Nothing about voter eligibility, nothing about hiding identity.

And yet it is double-vote resistant. [Note 1](https://talk.nervos.org/t/research-notes-what-zero-knowledge-proofs-enable-on-ckb/10368) verified that.

So if there's no nullifier doing the dedup, what is? That's the rest of this note.

---

## What CKB does instead

The dedup mechanism is three lines of Rust in the SP1 guest.

```rust
// voter lock hash -> (direction: 0=NO / 1=YES, amount in shannon)
let mut vote_map: BTreeMap<[u8; 32], (u8, u64)> = BTreeMap::new();
```

A `BTreeMap`. No nullifier set, no Merkle tree, no Poseidon. The key is a 32-byte voter identity. The value is what they voted and how much stake they hold.

When the guest finds a vote cell during the block walk, the voter identity is computed as:

```rust
let voter_lock_hash = blake2b_256(output.lock().as_slice());
```

The blake2b hash of the cell's lock script. The lock script is the public expression of "who can spend this cell," typically a signature scheme bound to a specific public key. Two cells with the same lock script are owned by the same private-key holder.

Then the dedup itself:

```rust
vote_map.insert(voter_lock_hash, (direction, amount));
```

A standard `BTreeMap::insert`. If a key already exists, the new value replaces it. That single semantic is the entire dedup primitive: a second vote from the same `voter_lock_hash` silently overwrites the first. The "vote retraction" feature in the spec is a free side-effect of this, not a separate mechanism.

What stops a voter from forging this identity? The on-chain vote type script verifies that the vote cell's lock matches the DAO deposit's lock. By the time the guest sees a vote cell in a block, CKB consensus has already proven that the voter controls the keys to that lock. The guest never needs to verify identity. The chain did.

---

## Why it works: UTXO as the dedup primitive

Nullifiers manufacture **unique-but-anonymous identity**. The chain learns that _someone_ voted, without learning _who_.

CKB's UTXO model already provides **unique-but-public identity**. The chain learns that this _specific_ lock script voted, and it knows whose key controls that lock script.

That asymmetry is the whole story. If your dedup primitive needs anonymity, you have to manufacture the identity inside the circuit, because the chain can't tell you who's who without learning who they are. Nullifiers do exactly that. If your dedup primitive does not need anonymity, you can skip the manufacturing step entirely and read identity straight off the substrate.

CKB hands you that for free. A lock script is signature-gated: only the holder of the corresponding key can authorize a transaction that creates a cell with that lock. Consensus enforces it every time a block is accepted. Two cells with the same lock script provably came from the same key holder. That is the same uniqueness guarantee a nullifier provides, except it comes from consensus instead of cryptography, and it shows up as a 32-byte hash you can use as a `BTreeMap` key.

You don't add a primitive. You borrow what's already there.

---

## The trade-off

The way I have come to see it, the choice between the two primitives reduces to a single question: is anonymity in the threat model? If it is, the design has to manufacture identity in zero-knowledge (nullifiers). If it isn't, the design can read identity off the chain directly (lock-script hashes here).

| Property         | Nullifiers manufacture | UTXO already provides |
| ---------------- | ---------------------- | --------------------- |
| Voter uniqueness | yes                    | yes                   |
| Anonymity        | yes                    | no                    |
| Unforgeability   | via secret-knowledge   | via signature gating  |
| Circuit cost     | high                   | low                   |
| Works on         | any chain              | UTXO-style chains     |

Nullifiers are the right primitive when "observers learning who voted" is itself a failure case. The canonical examples I have come across are whistleblower votes, anti-collusion DAO governance (the use case MACI was built for), corporate board votes where retaliation is real, and identity systems where membership must be provable without revealing which member. The cost is real: an extra primitive, a larger circuit, a Merkle tree to maintain, an audit surface that grows with all of it.

Where public attribution is acceptable, or part of the design intent, none of that machinery is needed. For stake-weighted public governance, the vote cell has to publish `dao_index` (pointing at the voter's DAO deposit) and `amount` (the stake weight) so the guest can verify the tally. Both fields identify the voter. The `dao_index` points at a publicly owned cell. The `amount` correlates against publicly indexable stake balances. Adding a nullifier on the vote cell can't hide what the protocol requires the cell to publish in the first place. That is the trade this design made.

---

## The extra defense: per-DAO-deposit dedup

The lock-script dedup catches identity reuse. It doesn't, on its own, catch "withdraw the deposit, redeposit at a fresh address, vote again." Different lock script, different `voter_lock_hash`, two entries in `vote_map`. Same 1,000 CKB of stake. Double weight.

[Note 1](https://talk.nervos.org/t/research-notes-what-zero-knowledge-proofs-enable-on-ckb/10368) covered this attack in Question 2. Here is the code-level mechanism that defeats it.

The guest maintains a second map alongside `vote_map`:

```rust
// DAO deposit outpoint (36 bytes) -> voter lock hash
let mut dao_outpoint_to_voter: BTreeMap<[u8; 36], [u8; 32]> = BTreeMap::new();
```

When a vote cell is recorded, every DAO deposit outpoint it references is also recorded, mapped to that voter's lock hash. Then every transaction in the voting window gets scanned:

```rust
for input in raw.inputs().iter() {
    let op_bytes: [u8; 36] = input.previous_output().as_slice().try_into().expect(...);
    if let Some(voter_lock_hash) = dao_outpoint_to_voter.remove(&op_bytes) {
        vote_map.remove(&voter_lock_hash);
    }
}
```

If any input consumes a DAO outpoint that backed a vote, the corresponding vote is evicted from `vote_map`. The withdraw-and-revote attack dies at the moment of withdrawal. The instant Alice's first deposit shows up as a transaction input, her first vote vanishes. By the time her second vote registers from the new address, she is a fresh voter with 1,000 CKB of stake, not 2,000.

This is the kind of check the on-chain vote type script structurally cannot do. The type script runs at vote-creation time and sees one transaction. It can verify the vote cell's lock matches the DAO deposit it references, but it cannot see what happens to that deposit eight blocks later. Spatial integrity is the type script's job. Temporal integrity, what changes across the voting window, is the guest's.

---

## What I am taking from this

The design doc lists nine features. Anonymity is not one of them, and it is not named as an exclusion either. This note was an attempt to name the trade-off implicit in that omission: the design uses CKB's UTXO-native identity as its dedup primitive, gets stake-weighted uniqueness without manufacturing a new cryptographic primitive in the circuit, and accepts public attribution as the cost.

What stays with me is not "skip nullifiers." It is something more like: know what your substrate provides before reaching for new primitives. CKB already handed this design one-vote-per-voter through lock-script identity. Adding nullifiers on top would have been redundant for uniqueness, and ineffective for the privacy the stake mechanism leaks regardless.

The harder question [Note 1](https://talk.nervos.org/t/research-notes-what-zero-knowledge-proofs-enable-on-ckb/10368) closed on, how to layer privacy onto this design without breaking what already works, is still open. That is the next note, not this one.

---

### References:

- XuJiandong's voting PoC: [github.com/XuJiandong/ckb-vote-poc](https://github.com/XuJiandong/ckb-vote-poc).
- My nullifiers explainer with worked code: [github.com/CECILIA-MULANDI/nullifiers](https://github.com/CECILIA-MULANDI/nullifiers).

---

_Note 2 of an ongoing series on the SP1 voting PoC on CKB. Previous: [Note 1](https://talk.nervos.org/t/research-notes-what-zero-knowledge-proofs-enable-on-ckb/10368)._
