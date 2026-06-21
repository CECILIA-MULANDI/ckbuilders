# CKB Builder Track - Week 14

**Week Ending:** 2026-06-19

## Focus This Week

Wrote Note 3 in the research-notes series. Note 1 closed on the open question of privacy in XuJiandong's SP1 voting PoC; Note 2 worked through the dedup primitive. Note 3 takes the privacy question itself, with the upfront framing that privacy is not the existing design's job (treasury governance needs public attribution) but the underlying primitive could serve other voting applications that do need it.

**They are posted below:**

- Repo: [zk_research_notes/Note3.md](zk_research_notes/Note3.md)

## What the note covers

- **Where privacy actually matters:** anti-collusion governance (MACI's domain), whistleblower/politically sensitive votes, and stake-weighted votes with uneven stake distribution.
- **Three leak points in the vote cell:** `lock_hash`, `dao_index`, and `amount`. A nullifier on the vote cell only addresses the first; the leak survives at the other two.
- **Three candidate approaches** (direct nullifier addition, MACI-style separation of identity from stake, mixing-pool / stake-aggregation) with what each fixes and what it doesn't.
- **What this would ask of the primitive:** commitments as first-class, aggregation without per-element disclosure, and nullifier-set management as a shared concern across CKB privacy applications.
