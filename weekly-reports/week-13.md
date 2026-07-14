# CKB Builder Track - Week 13

**Week Ending:** 2026-06-12

## Focus This Week

Continued the personal research-notes series with Note 2. Note 1 closed on the open question of privacy in XuJiandong's SP1 voting PoC. Note 2 takes one step before that: it asks what dedup primitive the design uses _instead_ of nullifiers, and why it didn't need to manufacture an anonymous one.

**They are posted below:**

- Repo: [zk_research_notes/Note2.md](zk_research_notes/Note2.md)

## What the note covers

- **What nullifiers manufacture:** unique-but-anonymous identity, the workhorse of Semaphore and MACI.
- **What CKB lets the design skip.** A `BTreeMap` keyed by `blake2b_256(lock_script)` is the entire dedup primitive. UTXO already provides unique-but-public identity, enforced by consensus, so nothing needs to be manufactured in the circuit.
- **The trade-off.** Anonymity for circuit cost. Nullifiers when observers learning who voted is itself a failure case; lock-script identity when public attribution is acceptable.
- **Withdraw-and-revote defense.** A second `BTreeMap` (DAO outpoint → voter lock hash) plus an input scan evicts a vote the instant its backing deposit is consumed.
