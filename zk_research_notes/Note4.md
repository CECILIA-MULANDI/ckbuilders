# Design space: binding claimers in ZK-gated locks on CKB

*Draft research note. Feedback welcome.*

## The question

Any lock script that unlocks on a valid zero-knowledge proof has to answer one design question the proof itself does not: **who gets to send the unlocked value?**

If a proof is valid, anyone holding a copy of it can submit a transaction spending the locked cell. Without extra binding, a spender's proof can be pulled from the mempool and redirected by any observer. The lock verifies the proof; it does not verify the intent.

The natural fix is to make the recipient's address a public input to the circuit. A proof is then only valid *for that recipient*, and copying it does not help an attacker. But this only works if the recipient is known when the proof is generated, which forces a further question: **is the recipient known at lock creation time, or only at spend time?**

## Two regimes

**Claimer known at lock time.** Direct escrow, sealed-bid auctions with pre-registered bidders, one-shot grants to a specific address. Here `recipient_hash` can be a public input committed into `lock.args` at cell creation. The lock's public-input commitment is enough on its own: any proof that verifies is by construction bound to the pre-agreed recipient. Front-running is impossible because a redirected proof carries different public inputs and fails verification.

**Claimer unknown at lock time.** Open airdrops (anyone in a Merkle-committed set can claim), open puzzles (anyone who knows the preimage can claim), any bounty-style structure. Here `recipient_hash` cannot be committed in advance because there is no pre-agreed recipient. A lock that commits *all* public inputs at creation cannot express this class of application without further mechanism.

The two regimes are qualitatively different, and I think it is worth naming them explicitly because they cost different things to support.

## Design space for the open regime

Three approaches, each with an honest cost:

**(a) Split public inputs into static and dynamic.** Lock args commit only the "static" portion of public inputs (Merkle root, puzzle hash). One or more "dynamic" public inputs (the claimer hash chief among them) are provided at spend time and checked by the lock script against transaction state, for example `dynamic_pi[0] == outputs[0].lock.hash`.

- *Pro:* small on-chain code; the ZK-lock stays self-contained.
- *Con:* couples the lock to a specific transaction-shape convention (which output holds the unlocked value, in which position). Every application layered on top has to match that convention.

**(b) Wrapping type script arbitrates winner selection.** The ZK-lock is spent by a transaction that also consumes a "claim registry" cell governed by a type script. The type script decides which claim wins (first valid submitter, highest bidder, oldest timestamp) and mutates its own state accordingly.

- *Pro:* idiomatic CKB composition; the ZK-lock stays single-purpose; arbitration logic is separately auditable.
- *Con:* adds a state cell that has to be maintained; higher on-chain footprint per claim.

**(c) Commit-reveal at the mempool layer.** Front-running is prevented off-chain by using private mempools, hash-based commit phases, or MEV-resistant relays. The ZK-lock itself is unchanged.

- *Pro:* zero on-chain cost.
- *Con:* trust in user tooling; not a property of the lock itself.

These are not mutually exclusive. A well-designed open-participation app might use (a) for base claimer binding and (b) for arbitration.

## What the cell model uniquely offers

Compared to account-based ZK-gated escrows (Ethereum-style), the cell model makes (b) unusually cheap. A type script for winner arbitration is just another cell in the same transaction: no proxy contract, no delegatecall, no upgradability puzzle. The ZK-lock and the arbitration logic are independently deployed, independently audited, and freely recomposed. This is the argument for keeping the base ZK-lock minimal: the composition surface is already there.

There is also a partial precedent worth noting. Nervos DAO's two-phase withdrawal (deposit cell to withdrawing cell to claim) is a protocol-level commit-reveal for a very specific use case. It is not directly reusable for ZK claims, but it is a proof point that CKB's cell lifecycle can be used to serialize intent before value moves.

## Open questions

Things I would genuinely like input on:

1. Is a single split-PI convention (approach a) tractable enough to standardise, or does every application end up wanting its own tx-shape? If the latter, does that push us toward pattern (b) as the default?
2. For the wrapping-type-script pattern, is there a general-purpose "ZK claim registry" worth writing once, or is arbitration inherently application-specific?
3. Are there cell-model-native mitigations for front-running I am missing? Are there existing scripts (beyond Nervos DAO) whose lifecycle patterns could inspire a claim-serialization layer?

Happy to talk this through with anyone else building ZK primitives on CKB. If the community converges on a preferred pattern, that becomes the standard the next round of primitives can build on.
