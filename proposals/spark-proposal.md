# Spark Program | zk-Lock for CKB

## 1. Project overview and positioning / 项目概述与定位

**zk-Lock** is a reusable CKB lock script that conditions cell spending on a valid Groth16 proof. Any developer can write a Circom circuit, commit to its verifying key, and lock CKB cells behind it; to spend the cell, the spender submits a proof that satisfies the circuit.

The primitive is the integration layer that turns the existing on-chain Groth16 verifier into something developers can build with. The Spark-funded scope is the minimal primitive (lock script, CLI, deployed testnet artifact, and tutorial) sufficient for a developer to lock a cell behind their own circuit and unlock it end-to-end. Consumer-facing applications (web app, treasure hunt launch event) are out of scope for this grant.

## 2. Team profile / 团队简介

- **Name:** Cecilia Mulandi
- **GitHub:** [CECILIA-MULANDI](https://github.com/CECILIA-MULANDI)
- **Role:** Solo developer; current participant in the CKB Builders Program (completed 12 weeks)
- **Contact:** mulandicecilia4@gmail.com, Telegram [@ceciliamulandi](https://t.me/ceciliamulandi)
- **Relevant prior work:**
  - [groth16-ckb](https://github.com/CECILIA-MULANDI/groth16-ckb): Groth16/BN254 verifier ported to CKB-VM, authored solo. Confirmed on Pudge testnet 2026-05-15. ~102M cycles per verification. Experimental and not yet audited. Currently in Phase 4 hardening (coverage-guided fuzzing, soundness proptests, threat model, audit prep); mainnet use is audit-gated. Public writeup on Nervos Talk: [groth16-ckb: an on-chain Groth16 verifier for CKB-VM](https://talk.nervos.org/t/groth16-ckb-an-on-chain-groth16-verifier-for-ckb-vm/10288).
  - [Research notes on what zero-knowledge proofs enable on CKB](https://talk.nervos.org/t/research-notes-what-zero-knowledge-proofs-enable-on-ckb/10368), published on Nervos Talk with peer review from the CKB Builders community.

## 3. Project background and problem statement / 项目背景与问题陈述

CKB has a working on-chain Groth16 verifier (load-bearing prior work, already shipped). What is missing is the **integration layer that lets developers actually use it**: there is no standard way to lock a cell behind a circuit, no convention for binding cells to specific public inputs, no tooling for a developer to go from "I have a Circom circuit" to "I have CKB locked behind it."

The result is that zk on CKB is currently reachable only by developers who can wire together the verifier crate, lock-script scaffolding, witness encoding, and tx construction themselves. Each new use case repeats this work. The primitive does not exist as ecosystem infrastructure.

## 4. Solution approach / 解决方案

A single-purpose CKB lock script with one spend path: **the cell unlocks iff a valid Groth16 proof matches the committed verifying key and public inputs.**

Four design choices, settled during pre-proposal research:

- **vkey storage**: 32-byte `vk_hash` in `lock.args`; the full verifying key lives in a shared `cell_dep` referenced by `data_hash`. Mirrors the secp256k1 system-script pattern (small commitment in args, shared data cell), so many cells can reuse one vkey cell instead of each carrying full vkey bytes.
- **Public-input binding**: `lock.args` also carries a 32-byte `public_inputs_commitment`. The cell is bound to specific public inputs, not just to the circuit, which is what makes zk-Lock a primitive rather than a generic verifier wrapper.
- **Minimal lock, compose for the rest**: no refund branch, no creator pubkey, no deadline. Applications that need refund or settlement (treasure hunt, auctions, escrow) layer them on via a wrapping lock or a type script. Keeps the audit surface tight and the primitive usable for sealed-bid auctions, private claims, conditional escrow, and anonymous airdrops without favouring any one of them.
- **Claimer-binding convention**: circuit authors include the claimer's lock-script hash as a public input, so a proof copied from a pending transaction cannot be redirected to another address. Enforced implicitly via `public_inputs_commitment`; templated in the docs and CLI.

## 5. Technical architecture / 技术架构

```
Locked cell
  lock.code_hash:  <zk-Lock script hash>
  lock.args:       vk_hash (32B) || public_inputs_commitment (32B)   = 64 bytes

Cell deps (referenced by the unlock tx)
  zk-Lock script cell                  (the code)
  vkey cell                            (data = serialized vkey; referenced by data_hash)

Witness (unlock tx)
  witness.lock:    proof || public_inputs
```

**Script logic (built on top of the existing `verifier-core` crate):**

1. Read `(vk_hash, public_inputs_commitment)` from `lock.args`.
2. Scan `cell_deps`; find the dep whose `blake2b(data) == vk_hash`. Fail if none, fail if duplicate.
3. Read `(proof, public_inputs)` from `witness.lock`.
4. Check `hash(public_inputs) == public_inputs_commitment`.
5. Call into `groth16-ckb` verifier with `(vk, proof, public_inputs)`.
6. Return success only if all checks pass.

**Off-chain pipeline:**

```
Circom circuit  →  snarkjs proof  →  CCC SDK  →  CKB
```

All off-chain tooling is mature (Circom, snarkjs, CCC). All on-chain logic reuses infrastructure already shipped (groth16-ckb verifier, CKB-VM).

## 6. Weekly execution plan / 执行计划

| Week | Focus | Concrete output |
|---|---|---|
| 1 | Lock script architecture; integrate `groth16-ckb` verifier as a lock | Workspace skeleton compiles; stub lock script reads args |
| 2 | Cell-dep vkey loading; witness parsing; end-to-end test against a known-good fixture | All-paths unit + integration tests pass locally |
| 3 | Deploy to CKB Pudge testnet; minimal CLI for lock/unlock; tutorial draft | Testnet tx hash demonstrating a successful unlock |
| 4 | Reference Circom circuit (Poseidon hash preimage); worked end-to-end demo via CLI; documentation polish | Repository, tutorial, and demo reproducible by a third party from a clean checkout |

Four weeks solo. Conservative by design: explicitly avoids scope I cannot defend within Spark's timeline (web app, wallet integration, public launch event).

## 7. Funding requirements / 所需资金与资金分配明细

**Total request: $1,704.**

| Item | Hours / cost | Subtotal |
|---|---|---|
| Lock script (week 1–2): args parsing, cell-dep vkey lookup, witness decode, integration with `verifier-core`, unit + integration tests | 70h × $12/hr | $840 |
| Testnet deployment + CLI (week 3): deploy to Pudge, lock/unlock subcommands, first successful on-chain unlock tx | 42h × $12/hr | $504 |
| Reference circuit + demo + docs (week 4): Poseidon-preimage Circom circuit, snarkjs proof pipeline, tutorial markdown + short architecture reference | 30h × $12/hr | $360 |
| **Total** | **142h** | **$1,704** |

Payment cadence per Spark norms: 20% upfront at project start, 80% drawn at weekly sync meetings based on demonstrated progress.

## 8. Deliverables and verification methods / 交付物与验证方式

| Deliverable | How to verify |
|---|---|
| zk-Lock script source code | Public GitHub repo; `cargo build --release` produces `riscv64imac` binary; `cargo test` passes |
| Deployed lock script on Pudge testnet | Cell `OutPoint` published in README; explorer link |
| Working end-to-end unlock | Testnet transaction hash showing a cell unlocked by a valid Groth16 proof |
| Reference Circom circuit (Poseidon preimage) | `.circom` source + `snarkjs` build script; reproducible vkey hash |
| CLI tool | `cargo install`-able; `--help` documents lock and unlock subcommands |
| Tutorial | Markdown in repo: "How to lock a cell behind your own Circom circuit"; a third party can follow it from clean checkout to a successful testnet unlock |

**Documentation scope:** repository markdown only (`README.md`, tutorial, short architecture reference, CLI `--help`). No hosted docs site within this grant; a public-facing docs site is out of scope for this milestone.

## 9. Current state vs funded scope / 当前状态 vs. 资助范围

**Already shipped (not funded by this grant):**
- Groth16/BN254 verifier on CKB-VM ([groth16-ckb](https://github.com/CECILIA-MULANDI/groth16-ckb)), confirmed on Pudge testnet 2026-05-15. Experimental and not yet audited.

**Funded by this grant:**
- The lock-script layer that wraps the verifier as a reusable primitive.
- CLI tooling to make the primitive usable from a terminal.
- Reference circuit + tutorial demonstrating end-to-end use.
- Testnet deployment of the lock script itself.

**Explicitly out of scope for this grant:**
- Web app with wallet integration.
- Public treasure hunt launch event.
- Mainnet deployment (audit-gated; pending Phase 4 hardening of the verifier).

## 10. CKB ecosystem alignment / CKB 生态契合度

- **Complements existing team's work.** The SP1 verifier (XuJiandong's team) is for complex computation; zk-Lock is for simple circuits. Both are valid, complementary use cases of zk on CKB.
- **Builds on what is already shipped.** The Groth16 verifier becomes the load-bearing piece of an ecosystem primitive rather than an orphan demo.
- **Unlocks new application categories.** Sealed-bid auctions, private claims, conditional escrow, and anonymous airdrops all reduce to "lock a cell behind a circuit" once the primitive exists.
- **Demonstrates the cell model and Web5 composability.** Single-purpose lock + composition via wrappers and type scripts is idiomatic CKB design and aligned with the Web5 emphasis on small composable primitives layered organically rather than monolithic vertical stacks. The worked demo is intended as a reference for how to compose zk primitives in the cell model.

## Risks and mitigations / 风险与缓解措施

| Risk | Mitigation |
|---|---|
| Verifier cycle cost is ~102M per verification (~2.9% of a block budget), limiting how many unlocks fit per block | Cost is empirically measured and acceptable for the treasure-hunt-style workloads this Spark milestone targets. Optimization (precomputed pairing tables, batched verification) is deferred to a separate effort. |
| Mainnet deployment of the verifier is audit-gated; this grant cannot deliver mainnet artifacts | Scope is explicitly testnet only (Pudge). Mainnet readiness is a separate future effort after audit and out of scope here. |
| `groth16-ckb` lives in a nested Cargo workspace; the git dependency may need a fallback to a local clone if Cargo resolution edge-cases bite | Tested in week 1. Fallback (local-path dep) is documented in the README. |
| Tutorial reproducibility depends on Circom and snarkjs versions, which evolve | Tutorial pins exact versions; CI verifies the recipe still produces the same vkey hash at each commit. |

## Out of scope (explicit) / 明确不在范围内

To keep evaluation crisp, this grant does **not** cover: web app, wallet integration, browser-side proof generation UI, mainnet deployment, public launch event with real CKB, or audit. Those are deliberately deferred to keep the Spark milestone focused on the testnet primitive.
