# CKB Builder Track - Week 12

**Week Ending:** 2026-06-05

## Focus This Week

I have spent time just doing some research on what is possible with zk on ckb. I opted to make the series of my personal research notes available publicly. This is an attempt to share what I learn publicly and also provide the resources for anyone that might be interested in learning too. The first note pulls together what I've been learning across `groth16-ckb`, the Molecule series, and reading around the DAO treasury voting PoC.

**They are posted below:**

- Repo: [zk_research_notes/Note1.md](zk_research_notes/Note1.md)
- Hashnode: [research-notes-zk-on-ckb-note-01](https://zk-on-ckb.hashnode.dev/research-notes-zk-on-ckb-note-01)
- X article: [Note01](https://x.com/kashortgirl/status/2064323052301611491)

## What the note covers

- **Architectural fit.** Why CKB's verify-only scripts, RISC-V VM, and cell model map naturally onto ZK verifiers. Anchored to the `groth16-ckb` cycle numbers (~102M cycles, ~2.9% of a block).
- **Soundness traced through the voting PoC.** Picked XuJiandong's `ckb-vote-poc` and walked two attack scenarios (vote omission, deposit double-counting) through the guest program to the specific lines that defeat them.

## What I learned

- A "verification slot" primitive falls out of the cell model almost for free: a cell whose type script is the verifier, bound to one VK by `type.args`. I had built this in `groth16-ckb` without naming it.
- The strongest soundness arguments come from invariants the prover _cannot_ opt out of, not from checks that happen to be present.
