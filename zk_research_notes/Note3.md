## Beyond treasury governance: what privacy would ask of this primitive

**NOTE:** _These are my personal research notes documenting what I am learning and finding as I explore zero-knowledge proofs on CKB. I welcome discussion and feedback_.

## Continuing from Note 2

[Note 2](https://talk.nervos.org/t/research-notes-what-zero-knowledge-proofs-enable-on-ckb/10368/2) closed on a question I had handed forward from [Note 1](https://talk.nervos.org/t/research-notes-what-zero-knowledge-proofs-enable-on-ckb/10368): how to layer privacy onto this design without breaking what already works. Before going there, I want to be honest about what I think the answer isn't.

The current [voting PoC](https://github.com/XuJiandong/ckb-vote-poc) is designed for [Nervos DAO treasury governance](https://talk.nervos.org/t/pre-rfc-discussion-activating-the-nervos-dao-treasury/10143/18). In that context, public attribution is not a bug. It is a feature. If community funds are being spent, the spending decisions should be auditable. Stake-weighted votes should be publicly tied to stakes. No voter should be in a position where they want to hide their decision and can't. I think privacy in that setting would actively undermine accountability.

So I want to say upfront: the implementors made the right call for the use case they were building for. This note is not a privacy proposal for the existing design. It is a different question. The primitive underneath the voting PoC (zkVM-verified history proof over a block range) is general. It could serve other voting and governance applications. Some of those would need privacy. What would the primitive have to look like to serve them?

That is the design space this note explores. The current implementation is the floor it is built on, not the target it is critiquing.

---

## Use cases that would need privacy

Three scenarios come to mind where privacy has proven to be needed, or clearly is.

**Anti-collusion governance (MACI's domain).** When votes are public, a briber can pay only on confirmed delivery. _"Vote yes on proposal X, I pay you."_ The verification is free if the vote is on-chain. The threat is structural in any DAO where individual votes carry economic weight.

**Whistleblower and politically sensitive votes.** Boards voting on internal misconduct, members voting on controversial issues, workers voting on union representation. Retaliation is real, and the public ledger becomes the attacker's tool.

**Stake-weighted votes with uneven stake distribution.** A nullifier on the vote cell wouldn't help here. The `amount` field fingerprints against publicly indexable balances. If three voters hold 1000, 2000, and 5000 CKB respectively, a vote with `amount: 5000` identifies the third one. The leak is in the stake, not the cell. (See [Note 2](https://talk.nervos.org/t/research-notes-what-zero-knowledge-proofs-enable-on-ckb/10368/2).)

The first two are what I named "membership proofs without identity disclosure" in [Note 1](https://talk.nervos.org/t/research-notes-what-zero-knowledge-proofs-enable-on-ckb/10368) The third asks for more: hiding voter weight, not just identity.

---

## The three leak points to address

Three places in the vote cell where identity becomes public. Any privacy proposal has to address all three, or the leak survives.

1. **`lock_hash`** on the vote cell. The current design uses it as the dedup key. Anyone reading the chain learns who voted.

2. **`dao_index`** in the cell's data. Points at the voter's DAO deposit cell, which has its own publicly attributable lock script. Even if the vote cell hid the voter's lock, this pointer would walk back to it.

3. **`amount`** in the cell's data. The stake weight, published so the guest can verify the tally. Cross-referenced against publicly indexable DAO deposits, it fingerprints the voter when stake amounts are non-uniform.

A nullifier on the vote cell only addresses (1). The leak survives at (2) and (3) regardless. Any honest privacy proposal has to address all three. ([Note 2](https://talk.nervos.org/t/research-notes-what-zero-knowledge-proofs-enable-on-ckb/10368/2) walks through this in more depth.)

---

## Candidate approaches

There are at least three starting points I have worked through. Each addresses some of the leak points and not others, and none of them feel clean to me.

**Direct nullifier addition.** Replace `lock_hash` on the vote cell with a nullifier derived from a voter secret, with a Merkle tree of voter commitments anchoring eligibility. Vote cells carry `(nullifier, vote, amount, dao_index)`.

This fixes leak (1). It does nothing about leaks (2) and (3). The `dao_index` still walks back to the staker's deposit; the `amount` still fingerprints. A reader of the chain still learns who voted, just through a different path. I read this as a building block for the approaches below, not a solution on its own.

**MACI-style separation of identity from stake.** Identity proven via Merkle membership over voter commitments (anonymous). Stake proven separately via a commitment scheme: voters pre-commit to _"I have X stake in this set"_ rather than referencing a specific deposit. The vote cell publishes a nullifier, a stake-range proof, and the vote choice.

This addresses all three leaks if done right. It requires more circuit work and a separate stake-commitment registry on-chain. The hard part is keeping stake commitments honest. The DAO deposit still has to be verifiable, but the link between deposit and voter has to be hidden. This is the most promising path I can see, and also the most engineering work.

**Mixing-pool / stake-aggregation.** Voters deposit into a shared anonymous pool. Voting eligibility comes from pool membership, not from individual DAO deposit references. Nullifiers prevent double-voting. The vote cell carries a pool-membership proof instead of `dao_index`.

This addresses leaks (1) and (2) cleanly but changes the deposit mechanism fundamentally. The existing Nervos DAO interaction would have to be rewired through the pool, and there are liquidity questions (when can voters exit the pool?). It is a well-understood pattern from Tornado-style mixers, but in my read, a heavier change to the DAO model than the other two.

None of these is a free upgrade. My honest reading is that adding privacy to a stake-weighted voting design is a redesign of the stake mechanism, not just the vote cell.

---

## What this would ask of the primitive

The primitive underneath the voting PoC is general by design: prove something happened in blocks N through M, commit the conclusion publicly. The voting design is one application of it. Privacy applications would be others. The question is what would have to change at the primitive layer rather than the application layer to make those work.

Three changes feel load-bearing to me.

**Commitments as first-class.** Today the primitive proves things about cell data directly: the guest reads `amount`, sums it, commits the total. For privacy, that has to become: the guest reads a _commitment to_ `amount`, aggregates commitments, commits an aggregate commitment. The primitive needs to be comfortable working with hidden values, not just cleartext ones.

**Aggregation without per-element disclosure.** The voting design currently computes `yes_vote + no_vote` and publishes both. For weight-hiding use cases, the primitive would need to support proving the aggregate is correct without exposing the per-vote weights that produced it. Homomorphic commitment schemes and proof composition are the obvious candidates.

**Nullifier-set management as a shared concern.** The voting design handles dedup application-specifically. If privacy applications across CKB share infrastructure, nullifier sets need a standard primitive form (how they are stored, updated, and queried) rather than each application reinventing one.

These changes don't have to land all at once. They are also separable from the voting application itself. My read is that this is the conversation worth having about generalizing the primitive: not "should the voting design have privacy," but "what does the primitive owe its other applications."

---

## What I am taking from this

I want this to land clearly: privacy is not the voting PoC's job. The treasury governance use case it was built for is a poor fit for privacy, and the implementors made the right call.

The primitive underneath, though, has a separate life. It can serve voting applications with very different threat models. The privacy-needing use cases I listed are real. Adding privacy to those applications won't come from a small change to the voting design. It will come from a redesign of the stake mechanism plus targeted changes at the primitive layer.

I might be wrong about which changes matter most, and where the line between primitive and application actually sits. I would be glad to hear from anyone closer to the design. The point of this note was to name the design space, not close it.

---

### References:

- XuJiandong's voting PoC: [github.com/XuJiandong/ckb-vote-poc](https://github.com/XuJiandong/ckb-vote-poc).
- Nervos DAO treasury discussion: [talk.nervos.org](https://talk.nervos.org/t/pre-rfc-discussion-activating-the-nervos-dao-treasury/10143/18).
- MACI (Minimum Anti-Collusion Infrastructure): [github.com/privacy-scaling-explorations/maci](https://github.com/privacy-scaling-explorations/maci).
- Semaphore: [semaphore.pse.dev](https://semaphore.pse.dev/).
- My nullifiers explainer: [github.com/CECILIA-MULANDI/nullifiers](https://github.com/CECILIA-MULANDI/nullifiers).

---

_Note 3 of an ongoing series on the SP1 voting PoC on CKB. Previous: [Note 2](https://talk.nervos.org/t/research-notes-what-zero-knowledge-proofs-enable-on-ckb/10368/2)._
