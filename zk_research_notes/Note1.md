## What I Think Is Possible and Why the Architecture Makes It Tractable

**NOTE:** _These are my personal research notes documenting what I am learning and finding as I explore zero-knowledge proofs on CKB. Discussion and feedback welcome._

## Where this starts

I have been spending time with CKB, reading the specs, building with Molecule, studying the cell model. At some point I started asking a different question: not just what CKB does, but what it could do if you layered zero-knowledge proofs on top of it.

This note is my attempt to think through that question honestly. It is not a tutorial and not a product announcement. It is an exploration of whether the architecture supports what I think it supports, and what becomes possible if it does.

## How CKB is designed

To understand why ZK fits, you need to understand how CKB works at a fundamental level. Three properties matter most here.

**Everything is a cell.**

CKB does not have accounts. It has cells: simple containers with a capacity (CKB tokens locked inside), a lock script (who can spend it), and a data field (any bytes you want to store). Your balance is not a number stored anywhere. It is the sum of capacity across all live cells your key controls.

Transactions consume cells and produce new ones. There is no "update" operation. Old cells die, new cells are born.

```
transaction
  inputs   = cells being consumed
  outputs  = new cells being created
```

This explicit consume-and-produce model means state is always visible, portable, and atomic. Every state transition is a transaction. Every transaction is verifiable.

**Scripts only verify, never compute.**

On Ethereum, smart contracts run computation. They update state, call other contracts, emit events. On CKB, scripts do one thing: they verify. A lock script verifies the spender has the right key. A type script verifies a state transition is valid. Scripts return success or failure. Nothing else.

This is a fundamental difference. CKB was not designed for on-chain computation. It was designed for on-chain [verification](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0002-ckb/0002-ckb.md).

**CKB-VM runs RISC-V.**

Most blockchains run contracts on custom VMs with fixed instruction sets. Adding new operations requires new opcodes, which requires a hard fork.

CKB-VM runs RISC-V, a real hardware instruction set. Any code that compiles to RISC-V can run as a CKB script. No special opcodes needed. No protocol changes needed. The chain does not care what your script does internally; it runs it and charges cycles.

No cryptographic operations are hardcoded. The default signature verification and hash functions ship as deployable scripts, not protocol primitives. New cryptographic primitives are deployed the same way any code is deployed, as a cell. This has an interesting consequence for quantum resistance. If a quantum-safe signature scheme emerges, CKB can adopt it by deploying a new script, no hard fork required. But that is a story for another note.

## Why this maps naturally onto ZK

Zero-knowledge proofs have a prover and a verifier.

```
prover    runs expensive computation off-chain
          produces a short cryptographic proof

verifier  checks the proof cheaply
          never re-runs the computation
          learns the result but not the private inputs
```

Now look at CKB's design again:

```
CKB scripts    verify state transitions
               never run the computation themselves
               return success or failure

ZK verifiers   verify proofs of correct computation
               never re-run the computation themselves
               return valid or invalid
```

They are structurally the same thing. CKB was built as a verification layer. ZK proofs are things that need to be verified. The fit does not feel like a coincidence.

The cell model reinforces this. ZK proofs often need to commit to state: "I am proving something about the current state of this data." On CKB, state is explicit. It lives in cells. A ZK state transition looks like this:

```
input cell    = old state
output cell   = new state
witness       = ZK proof

type script:
  read old state from input cell
  read new state from output cell
  verify the ZK proof
  if proof valid -> accept state transition
  if proof invalid -> reject transaction
```

Old state, new state, proof. Three things. All handled by existing CKB primitives. Nothing special required from the protocol.

And because CKB-VM runs RISC-V, you can implement any ZK verifier (Groth16, PLONK, STARKs, anything) as a native script. You compile your verifier to a RISC-V binary, deploy it as a cell, and type scripts call it. The chain runs it and charges cycles.

## What the cycle cost model means

CKB charges cycles for every RISC-V instruction executed. There are no hidden costs, no gas estimation surprises, no special pricing for specific operations.

This matters for ZK because ZK verifiers are computationally intensive. A Groth16 verifier involves elliptic curve pairings, among the most expensive operations in applied cryptography. On Ethereum, the cost of running a verifier depends on whether a precompile exists for your proof system, how that precompile is priced, and what gas limit the block allows. If no precompile exists for your proof system, you pay full EVM gas for every operation.

On CKB, the cost is whatever your verifier costs to execute in RISC-V cycles. Old proof system, new proof system, experimental proof system, all use the same pricing model, the same deployment process, the same rules.

I built a [Groth16/BN254 verifier](https://github.com/CECILIA-MULANDI/groth16-ckb) to test this concretely. These are the numbers from the production call path, with the verifying key decoded from a `cell_dep`, the proof read from the witness, and the full pairing check running on riscv64imac CKB-VM:

```
cycles per verification   ~102 million
CKB block cycle limit     3.5 billion
block usage per verify    ~2.9%
```

2.9% of a block per verification. That is practically usable. It means a deployable, measurable Groth16 verifier exists on CKB today, and the same approach generalizes to any other proof system that compiles to RISC-V.

## What becomes possible

With this foundation, a few categories of application start to look natural on CKB. Most of them are technically possible on other chains. The difference is that the cost and ergonomics on EVM chains depend on whether a precompile happens to exist for your proof system, and the state layout has to be squeezed into a key-value abstraction. On CKB, the verifier is just code deployed like anything else, and the state is just bytes in cells.

**Private state transitions.**

A type script can verify a ZK proof without knowing the private inputs that generated it. The proof goes in the witness. Public commitments such as a nullifier, a new state root, or an output commitment go into the output cell's data field. The chain sees that a valid proof was submitted and that the new commitments are well-formed. It does not see what was proved.

**Membership proofs without identity disclosure.**

Prove you are in a set without revealing which member you are. The eligible set is committed to publicly as a Merkle root, stored as bytes in a cell's data field. The proof shows you know a path from a leaf to that root. A type script on the cell verifies the inclusion proof and accepts the spend if valid. Nullifier sets that need to grow get their own cell, updated by the same script. No registry contract, no precompile, just cells and a verifier script.

This is directly relevant to governance, where the use case is proving voting eligibility without revealing voter identity.

**Verifiable computation with private inputs.**

Run a computation off-chain. Generate a proof that the computation was done correctly. Submit the proof and the public outputs on-chain. The chain verifies the proof. Anyone can confirm the result is correct without re-running the computation or seeing the inputs. The honest constraint is that the computation has to be expressible as a circuit your prover supports. Within that constraint, the on-chain story stays the same.

**Proof-aggregated batched updates.**

Aggregate many state transitions off-chain. Generate a single proof that all of them are valid. Submit one transaction with one input cell carrying the old aggregate state, one output cell carrying the new aggregate state, and one witness carrying the aggregation proof. The cell model handles the verification side cleanly because the inputs and outputs already represent state before and after.

A full rollup is more than this. It also needs data availability and an exit mechanism that does not depend on operator cooperation. Those are separate problems that a verifier alone does not solve. But the proof-checking layer that every rollup-style design relies on slots into CKB without anything custom from the protocol.

**A concrete primitive: the verification slot.**

While building the Groth16 verifier I ended up with a small composability primitive worth naming. A cell sits on chain whose type script is the verifier, bound to one specific verifying key by its type-script args. The verifier permits the cell to be created without a proof, then requires a valid proof to spend it. The cell becomes an open verification slot, bound to exactly one computation. Anyone holding a valid proof for that computation can spend it.

This kind of "stateful slot anyone can satisfy by proving X" composes naturally on CKB because cells are first-class objects with their own type script and their own data. It is harder to express cleanly on chains where verification is a function call against a fixed contract.

## My findings

The most concrete current use of ZK on CKB is the [voting PoC](https://github.com/XuJiandong/ckb-vote-poc) for the Nervos DAO treasury, which uses the SP1 zkVM. While reading around it, two questions stood out to me. I treated each as an attack scenario and traced through the guest program to see where the attack dies.

**Question 1. Can a prover selectively omit unfavorable votes?**

The setup: the prover wants to leave NO votes out of the tally so a proposal passes that should fail. There are four obvious avenues.

| Attack avenue                                         | What stops it                                                                                                                                             |
| ----------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Tamper with a block body to delete a vote transaction | The header commits to `transactions_root`. The Merkle root recomputed from the modified body no longer matches the header.                                |
| Skip entire blocks to omit a range of votes           | Each block's `parent_hash` is checked against the previous block's hash. A gap breaks the chain.                                                          |
| Bend the filter so honest NO votes look invalid       | The filter is a pure function of the cell's own `code_hash`, `hash_type`, and `args`. A real vote cell has the values it has.                             |
| Lie about voting duration to feed fewer blocks        | Duration is read from the proposal cell's own data, anchored to the real chain via the start block hash. The guest demands exactly `duration + 1` blocks. |

The check that does most of the work lives in `verify_block_integrity`:

```rust
let prev_hash = header_hash(prev_block.header());
let parent_hash = byte32_to_arr(current_block.header().raw().parent_hash());
if prev_hash != parent_hash {
    return Err(Error::ParentHashMismatch { block_index: i });
}
```

These are not ad-hoc patches. They follow from one property: a block header hash commits to its body and to its parent. Break the body and the root mismatches. Break the chain and the parent hash mismatches. The prover has no flex.

**Question 2. Can a voter double-count a DAO deposit across withdrawals?**

The setup: Alice deposits 1000 CKB to address₁, votes YES, withdraws, redeposits 1000 CKB to address₂, votes YES again. Goal: 2000 CKB of weight from 1000 CKB of stake.

The guest program tracks two maps. `dao_outpoint_to_voter` records which deposit belongs to which voter. `vote_map` records which voter chose what. When Alice withdraws her first deposit, that deposit shows up as a transaction input. The guest treats every input as a potential spend event:

```rust
for input in raw.inputs().iter() {
    let op_bytes: [u8; 36] = input.previous_output().as_slice().try_into().expect(...);
    if let Some(voter_lock_hash) = dao_outpoint_to_voter.remove(&op_bytes) {
        vote_map.remove(&voter_lock_hash);
    }
}
```

The moment Alice's old deposit appears as an input, her first vote is removed from the tally. By the time her second vote registers from address₂, she is a fresh voter with 1000 CKB of stake. The final count is one YES vote, not two.

| Variation                                | Outcome                                                                                |
| ---------------------------------------- | -------------------------------------------------------------------------------------- |
| Withdraws after the voting window closes | Vote stayed valid. The window is over.                                                 |
| Votes with someone else's deposit        | Rejected on chain by the vote type script (the lock must match the DAO deposit owner). |
| Two voters somehow share a deposit       | Same on-chain check rejects it. The guest also dedups at the outpoint level.           |

## The pattern these answers share

Both attacks fail for the same fundamental reason. The design forces the prover through cryptographic checkpoints whose values cannot be lied about, and uses each checkpoint to enforce an invariant. For Question 1, every block must hash to a value that chains to its neighbor and Merkle-roots to its header; the prover cannot pick what is in a block. For Question 2, every spend in the range is processed and cross-referenced against active votes; the prover cannot quietly forget a withdrawal.

What ties them together is the immutability of historical block data. The prover does not get to summarize, edit, or omit. They are forced to replay the chain honestly because every step they take has a cryptographic anchor that the verifier checks independently.

## What I am taking from this

The architectural fit I wrote about earlier is what makes this kind of design possible in the first place. CKB-VM running RISC-V meant one could deploy a real SP1 verifier without protocol changes. The cell model meant the proposal cell, the vote cells, and the proof check compose without any registry contract. The result is a soundness story that holds up to scrutiny.

What stays genuinely open in this design space is privacy, not soundness. The proof's intermediate state links voter identities to vote choices, and a public observer watching DAO deposits and vote cells over a voting window can correlate the two. Whether ZK can layer anonymity on top of this design without breaking what is already working is a different question than the one I started with, and one worth more thought before I claim anything about it.

---

### References:

- The groth16-ckb repository is at [github.com/CECILIA-MULANDI/groth16-ckb](https://github.com/CECILIA-MULANDI/groth16-ckb).

- The DAO treasury discussion is on [talk.nervos.org](https://talk.nervos.org/t/pre-rfc-discussion-activating-the-nervos-dao-treasury/10143/18).

- XuJiandong's voting PoC is at [github.com/XuJiandong/ckb-vote-poc](https://github.com/XuJiandong/ckb-vote-poc).
