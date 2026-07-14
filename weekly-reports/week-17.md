# CKB Builder Track - Week 17

**Week Ending:** 2026-07-13

## Focus This Week

Two workstreams. Closed out the fiber-probe hackathon build with the operator-facing `diagnose` command, a wallet-SDK integration example, and submission polish. In parallel, drafted the first pass of a long-form article introducing Polkadot developers to CKB.

**Posted below:**

- fiber-probe repo: [github.com/CECILIA-MULANDI/fiber-probe](https://github.com/CECILIA-MULANDI/fiber-probe)
- Article: [polkadot-to-ckb/polkadot-to-ckb.md](polkadot-to-ckb/polkadot-to-ckb.md)

## fiber-probe (Fiber hackathon)

Continuing from Week 16, the tool now has enough surface to submit. What landed:

- **`diagnose` command wired.** The preflight analyzer from last week is now surfaced through a dedicated subcommand, so an operator runs one thing to get a classified verdict on their node's channel state.
- **`wallet_gate` SDK integration example.** A worked example showing how a wallet or merchant integration calls into fiber-probe programmatically, not just via the CLI. This is the piece that makes it credible as _infrastructure_ rather than a one-off script.
- **README + submission-prep housekeeping.** License, metadata, roadmap, and a devcontainer so a judge can run it from a clean checkout.

## Polkadot-to-CKB article

Title: _On CKB, You Drive_. A guide for Polkadot developers crossing into the cell model: what breaks, what transfers, and what you have to unlearn.

The framing the article commits to: CKB and Polkadot disagree architecturally about where logic lives and who owns state. Polkadot fuses runtime, storage, and consensus into one integrated stack; CKB separates logic (scripts) from state (cells) so cleanly that there is no runtime to delegate to. That separation is the article. Everything else (cell model, scripts, querying, upgrades) follows from it.

Structure of the draft:

- **What Polkadot gave you (that you didn't know you were depending on).** Makes the Substrate model explicit: single WASM runtime, pallet-owned storage, relay-chain consensus. Names the assumptions before challenging them.
- **Part I: For onchain developers.** Pallet-writer's perspective. From _mutating programs_ to _validating programs_. Type Scripts, lock scripts, transaction construction.
- **Part II: For dapp developers.** Frontend/SDK perspective. What replaces PAPI, extrinsics, and the runtime metadata API.
- **The mental model break, stated plainly.**
- **Why you should consider making the jump.**
- **Further reading + getting started.**

Tone kept to honest tradeoffs rather than adversarial framing. The article names what Polkadot does well and what a developer has to give up (and gain) by moving.
