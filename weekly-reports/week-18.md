# CKB Builder Track - Week 18

**Week Ending:** 2026-07-20

## Focus This Week

Three threads. Submitted the polkadot-to-ckb article to the Nervos Education Hub as a pull request, produced a revised technical spec for the the zklock project, and drafted a short research note on claimer binding in ZK-gated locks.

**Posted below:**

- Article PR: [NervosEducationHub/EducationHubArticles#295](https://github.com/NervosEducationHub/EducationHubArticles/pull/295)
- Revised Spark proposal: [proposals/proposal2.md](proposals/proposal2.md)
- Research note: [zk_research_notes/Note4.md](zk_research_notes/Note4.md)

## Polkadot-to-CKB article: submitted

The Week 17 draft is now open as a PR against the Nervos Education Hub content repo. Ready for editorial review.

## Spark proposal: revised technical details

Produced `proposal2.md`, a v2 with a tightened technical spec. Scope, timeline, and funding are unchanged; the changes are precision improvements a technical reviewer would reasonably ask for:

- **Static-PI model made explicit.** `public_inputs_commitment` binds all public inputs at lock creation time. The consequence for open-participation flows (where the claimer is unknown at lock creation) is now called out directly, with dynamic-PI support scoped as a follow-up milestone rather than implied.
- **Witness encoding pinned.** A BN254 Groth16 proof is 256 bytes (A and C in G1 at 64 bytes each, B in G2 at 128 bytes), followed by `N * 32`-byte field elements for public inputs.
- **Hash function pinned.** `blake2b-256` with CKB personalization `"ckb-default-hash"` throughout, matching what `load_cell_data_hash` returns. Off-chain tooling must use the identical personalized hash.
- **vkey cell lifecycle addressed.** The tutorial will recommend deploying vkey cells under an immutable or governance lock so accidental spending doesn't orphan every zk-Lock cell referencing it.
- **Cell-dep matching softened.** Multiple `cell_dep`s with a matching data hash are now accepted (they resolve to identical data by definition), rather than treated as a hard failure.
- **Build target triple corrected** to `riscv64imac-unknown-none-elf` in the deliverables section.

## Research note: claimer binding in ZK-gated locks

`Note4.md`: a short design-space write-up on the question any ZK-gated CKB lock has to answer, namely _who gets to send the unlocked value?_ This is a general problem for ZK primitives on CKB, not specific to zk-Lock, and worth naming while I am actively working through it.

The note distinguishes two regimes:

- **Claimer known at lock creation time**: direct escrow, sealed-bid auctions with pre-registered bidders. Handled by committing `recipient_hash` as a public input.
- **Claimer unknown at lock creation time**: open airdrops, open puzzles. Cannot be handled by a static-PI lock alone.

Three approaches for the second regime are compared with honest trade-offs: (a) split public inputs into a static committed portion and a dynamic portion checked against transaction state by the lock; (b) wrapping type script that arbitrates winner selection before the ZK-lock is spent; (c) commit-reveal at the mempool layer. Not mutually exclusive.

Draft aimed at Nervos Talk. Ends on open questions rather than conclusions, since the point of posting is to surface others thinking about the same problem (XuJiandong's team on SP1, in particular) and converge on a standard pattern for the next round of ZK primitives on CKB.
