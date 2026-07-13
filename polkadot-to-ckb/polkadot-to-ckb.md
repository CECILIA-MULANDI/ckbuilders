# On CKB, You Drive

_A guide for Polkadot developers crossing into the cell model what breaks, what transfers, and what you have to unlearn._

---

The first thing CKB asks you to do is construct a transaction by hand.

Not submit an extrinsic. Not call a contract method. Build the inputs yourself find the cells you want to consume, decide which outputs to create, figure out the capacity, encode the data, attach the right scripts, then sign and broadcast. The chain doesn't do any of that for you. It receives your proposed state transition and either accepts or rejects it.

If you're coming from Polkadot, this feels backwards. On Substrate, constructing state transitions is the runtime's job. You write a pallet that defines valid transitions; FRAME wires it into the block execution pipeline; PAPI gives your frontend a fully typed API to call into it. The division of labor is clear: developers define rules, the chain enforces and executes them. You never manually assemble a storage write. You call `api.tx.Balances.transfer_keep_alive()` and the runtime handles everything underneath.

So why does CKB hand you the keys and make you drive?

The answer isn't that CKB has worse tooling or a steeper learning curve for its own sake. The answer is architectural: CKB and Polkadot have fundamentally different opinions about where logic should live and who should be responsible for state. On Polkadot, logic and state are fused a pallet owns its storage maps, the runtime owns all state transitions, the relay chain owns consensus. On CKB, they're separated entirely. Logic (scripts) and state (cells) are distinct objects that meet briefly at validation time and then go their separate ways. No script owns a cell. No runtime owns the chain's state. The developer constructs the transition because there is no runtime to delegate to.

That separation is the article. Everything else the cell model, the script architecture, the querying model, the upgrade story follows from it.

---

## What Polkadot Gave You (That You Didn't Know You Were Depending On)

Before getting into cells and scripts, it helps to make the Polkadot model explicit, because most developers who've learned Substrate have never had to articulate it from the outside.

Polkadot's development model rests on three fused layers:

**The runtime is a single WASM binary.** Every pallet in your chain compiles into one WASM blob. That blob is the law: it defines every valid state transition, every callable function, every storage layout. Nodes execute that blob to agree on state. When you deploy a parachain, you're deploying a runtime. When you upgrade, you replace the runtime with a new WASM binary via governance forkless, because the upgrade itself is a valid state transition that the old runtime approves.

**Pallets own their storage.** A `StorageMap` in a pallet isn't a raw database table that anyone can write to. It's namespaced to that pallet, and only that pallet's extrinsics and hooks can legitimately mutate it. There's no way for an external caller to reach into another pallet's storage and change a value without going through that pallet's defined API. Ownership is enforced by the Rust module system, at compile time.

**The relay chain absorbs your consensus.** As a parachain developer, you don't implement consensus. You write a runtime, Polkadot's collator/validator machinery handles block production and finality. Your job ends at "define valid state transitions." Everything else is infrastructure.

These three things together single runtime, owned storage, pooled consensus define the Polkadot developer experience. They're so fundamental that most Substrate developers wouldn't list them as assumptions. They're just "how blockchains work."

CKB breaks all three. Not by accident, and not as a limitation. By design, and for reasons.

---

## Part I: For Onchain Developers

_You write pallets or runtime logic. You think in terms of storage maps, dispatchable calls, and extrinsics._

### There Is No Runtime

CKB has no runtime in the Substrate sense. There is no WASM binary that every node executes to determine valid state transitions. There is no single authority that owns storage. There is no place to put your `#[pallet::call]` and `#[pallet::storage]` macros.

What CKB has instead is the **cell model**.

A cell is the atomic unit of state on CKB. It has four fields:

- **`capacity`**: how much CKB the cell holds, and by implication, how many bytes it's allowed to occupy on-chain (1 CKB = 1 byte of on-chain state)
- **`lock_script`**: a program that controls who can spend this cell the ownership rule
- **`type_script`**: an optional program that governs what the cell's data means the logic rule
- **`data`**: arbitrary bytes whatever you want to store

A cell is either unspent or spent. You don't mutate it in place. When you "update" state, you consume one or more cells as transaction inputs and produce new cells as outputs. The consumed cells are destroyed. The new cells exist. This is Bitcoin's UTXO model, but extended with programmable scripts instead of Bitcoin Script's limited stack operations.

Here's what this means for onchain developers specifically: **there is no place where your logic and your state are fused together.**

In Substrate, your pallet is a Rust module that directly references its own storage:

```rust
#[pallet::call]
pub fn transfer(origin: OriginFor<T>, to: T::AccountId, amount: u128) -> DispatchResult {
    let from = ensure_signed(origin)?;
    Balances::<T>::mutate(&from, |b| *b -= amount);
    Balances::<T>::mutate(&to, |b| *b += amount);
    Ok(())
}
```

The pallet reaches into storage and mutates it. It owns that storage. The mutation is the point.

On CKB, a script doesn't mutate anything. It's a **validator**. It receives a complete picture of the proposed transaction all inputs, all outputs, all cell data and returns either `0` (valid, let this through) or an error code (reject). It has no ability to reach out and change something. The state change is proposed by the transaction constructor (your wallet, your dapp, your SDK), and the scripts on the consumed/created cells decide whether to allow it.

This is the fundamental shift: from **mutating programs** to **validating programs**.

Here is what a simple token transfer validation looks like as a CKB Type Script, written in Rust targeting RISC-V:

```rust
pub fn main() -> Result<(), Error> {
    // Load all input cells with this type script
    let inputs_amount = load_cells_with_type(Source::Input)?
        .iter()
        .map(|cell| decode_amount(&cell.data))
        .sum::<u128>();

    // Load all output cells with this type script
    let outputs_amount = load_cells_with_type(Source::Output)?
        .iter()
        .map(|cell| decode_amount(&cell.data))
        .sum::<u128>();

    // Conservation: no tokens created out of thin air
    if inputs_amount < outputs_amount {
        return Err(Error::InvalidAmount);
    }

    Ok(())
}
```

There's no storage to mutate. No "transfer" function to call. The script asks one question: "in this proposed transaction, do the token amounts add up correctly?" If yes, the transaction is valid. If no, it's rejected.

The token amounts themselves live in the `data` field of cells, as raw bytes. The cells float freely on-chain, locked to their owners' Lock Scripts. No program "owns" them. The Type Script can validate transitions involving them; it cannot reach out and change them.

### What Replaces the Pallet

When a Substrate developer asks "how do I build a token on CKB?" the instinct is to look for the equivalent of a pallet. There isn't one. What you build instead is a **Type Script**.

The xUDT (extensible User Defined Token) standard is CKB's closest equivalent to ERC-20. It's not a contract with a state. It's a RISC-V binary that enforces one rule: the total token amount in inputs must be greater than or equal to the total token amount in outputs. Every cell carrying xUDT tokens references that binary as its Type Script. When you transfer tokens, the CKB-VM executes the xUDT binary as part of validating your transaction.

The binary is deployed on-chain as a cell just data in a cell, no different in structure from any other cell. Other transactions reference it via `cellDeps`, which is how the CKB-VM knows where to load the script from at validation time.

This is "bring your own cryptography and consensus primitives" taken to its logical conclusion. There are no hardcoded precompiles. There's no privileged pallet registry. Every system script including the default secp256k1 lock that CKB addresses use is just a cell on-chain that anyone can reference. You can write a competing implementation, deploy it, and it works today. No governance vote. No hard fork.

### What About Forkless Upgrades?

Substrate's forkless upgrade story is one of the things Polkadot developers are most proud of, rightly so. Because the runtime is a WASM binary stored on-chain, upgrading it is just a governance-approved extrinsic that replaces the WASM binary. No coordinated hard fork. No nodes going offline. The chain continues, with new logic.

CKB doesn't have this. There's no runtime to replace. There's no on-chain WASM binary governing all state transitions.

But the reason CKB doesn't need forkless upgrades is the same reason it doesn't need a runtime: **logic doesn't own state**. When you want to upgrade your token logic on CKB, you deploy a new Type Script binary and migrate existing cells to reference the new script. That migration is itself just a transaction consume the old cells, produce new cells pointing at the new Type Script. You can make it incremental. You can run old and new logic in parallel during a migration window. The "upgrade" is not a single atomic event that switches all behavior simultaneously; it's a series of cell transitions that you control.

This is less ergonomic than Substrate's one-click runtime upgrade. It requires you to think carefully about migration. But it also means there's no single binary whose upgrade you have to push through governance before you can change behavior. Different cells can reference different versions of a script simultaneously. You don't have to convince an entire chain to upgrade; you just upgrade your own cells.

---

## Part II: For Dapp Developers

_You build frontends, SDKs, indexers, or off-chain tooling. You think in terms of RPC calls, signed transactions, event subscriptions, and contract method calls._

### There Are No Contract Methods

The Polkadot dapp developer's mental model is service-oriented. The modern standard is PAPI (Polkadot-API) a light-client-first TypeScript library whose types are generated directly from on-chain metadata. You run `npx papi add dot` and your editor immediately knows every storage key, every extrinsic, every event on the chain. Querying a balance looks like this:

```typescript
import { dot } from "@polkadot-api/descriptors";
import { createClient } from "polkadot-api";

const client = createClient(provider);
const api = client.getTypedApi(dot);

const account = await api.query.Balances.Account.getValue(address);
console.log(account.free); // fully typed, BigInt
```

Submitting a transfer:

```typescript
await api.tx.Balances.transfer_keep_alive({
  dest: MultiAddress.Id(to),
  value: amount,
}).signAndSubmit(signer);
```

The key thing PAPI gives you: the chain's entire surface area is statically typed from metadata. Your IDE knows what `Balances.Account` returns before you run a single line of code. The chain is a typed service you call.

Solidity contracts on Polkadot (via `pallet-revive`, which executes Ethereum-compatible bytecode on the Polkadot Virtual Machine) extend this further: you call a contract method, pass typed arguments, the contract's on-chain code handles the state change.

On CKB, none of this exists. There is no `api.tx.token.transfer()`. There is no metadata. There is no schema. The dapp developer's job changes from **calling a typed service** to **constructing a state transition from raw bytes**.

This is what a token transfer looks like from the dapp side on CKB:

1. **Collect input cells**: scan the live cell set for cells owned by the sender whose Type Script matches the token type you want to send
2. **Calculate amounts**: sum the token amounts in those cells' `data` fields
3. **Construct output cells**: build new cells one for the recipient (locked to their key), one for change back to the sender if the input amount exceeded the transfer amount
4. **Attach cellDeps**: include a reference to the xUDT script cell so the CKB-VM can load the validation logic
5. **Sign and submit**: sign the transaction with the sender's key (satisfying the Lock Scripts on the input cells) and broadcast

There's no "transfer" function. You're not calling into a contract. You're building the entire state transition yourself, proposing it to the network, and the network's validation layer (the scripts) either accepts or rejects it.

This is closer to how a Bitcoin wallet works than how an Ethereum or Polkadot dapp works. The dapp developer is responsible for transaction construction, not just signing.

### Querying State

On Polkadot, querying state means reading from a well-defined storage layout that the runtime exposes and PAPI makes this frictionless because it generates TypeScript types directly from that metadata. The chain tells PAPI its schema; PAPI tells your IDE. You never manually decode storage keys or guess at data layouts. When the runtime upgrades and storage changes, you re-run `npx papi add` and your descriptors update. The type system catches breakage at compile time.

On CKB, there is no metadata. There is no schema the chain publishes. There is no equivalent of `npx papi add`. The CKB node exposes an RPC to query the live cell set with filters by Lock Script, by Type Script, by capacity range but the meaning of the bytes in each cell's `data` field is entirely up to you to know and decode. A typical query: "give me all unspent cells whose Lock Script matches this address and whose Type Script matches this token type."

```javascript
const cells = await rpc.getCells({
  script: {
    codeHash: SECP256K1_LOCK_HASH,
    hashType: "type",
    args: senderLockArgs,
  },
  scriptType: "lock",
  filter: {
    script: {
      codeHash: XUDT_CODE_HASH,
      hashType: "type",
      args: tokenArgs,
    },
  },
});
```

The result is a list of live cells. You sum the amounts in their `data` fields to get the balance. There is no typed query that returns a single number. There is no metadata telling you the balance is a `u128` at byte offset 0. You have to know that, because you wrote the Type Script, or because you read the xUDT spec. Balance is a derived quantity computed from raw bytes exactly like Bitcoin's UTXO model, and exactly unlike PAPI's compile-time-typed storage reads.

For production dapps, this is where indexers come in. CKB's Lumos SDK and services like CKB Explorer provide indexed views over the cell set, so you're not doing raw cell scanning in your frontend. But understanding the underlying model balance as a sum of cells, not a database row changes how you think about state design in your application.

### The Ownership Model Is Explicit, Not Implicit

On Polkadot, access control to state is mostly handled by the runtime's origin system. A dispatchable call checks `ensure_signed(origin)?` and then has full permission to mutate whatever storage it references, as long as the runtime's logic allows it.

On CKB, ownership is enforced by the Lock Script on every cell, evaluated independently by every full node in the network. There's no runtime with privileged access. There's no "the contract allows this, so it's fine." If your Lock Script says a transaction requires signature X, then no transaction consuming that cell will be valid without signature X not because nodes are well-behaved, but because every node runs the script independently and rejects anything that fails.

This makes ownership composable in ways that Polkadot's model isn't. You can put a cell under a multisig Lock Script that requires keys from two different parties, with no central contract coordinating. You can put a cell under a time-lock that opens after a certain block number. You can put a cell under a ZK-proof-based lock that only opens if the spender can prove membership in a set, without revealing which member. These are all just RISC-V programs. Any Rust or C code that compiles to RISC-V and returns 0 or an error is a valid Lock Script.

For dapp developers, this means the locking logic lives in the asset, not in a contract the dapp calls. When you're designing your application's access control, you're designing scripts that get embedded in cells, not functions you call on a contract.

---

## The Mental Model Break, Stated Plainly

If you come from Polkadot, the single hardest thing to internalize about CKB is this:

**On Polkadot, logic owns state. On CKB, logic validates state. Nothing owns state.**

In Substrate, your pallet is a module that has a storage namespace it controls. External code cannot legitimately modify that storage except through the pallet's defined API. The pallet _is_ the authority over its state.

On CKB, a script is a pure function: transaction in, 0 or error out. It runs, it checks, it exits. The cells it validated still exist after it exits. They'll be consumed by a future transaction, which will invoke whatever scripts are relevant to that transaction, and those scripts will run, check, and exit too. No script accumulates state. No script owns a cell. Ownership is encoded in the lock on the cell itself, not in any program's module boundary.

This feels like a loss of structure when you first encounter it. On Substrate, the pallet is the natural organizing principle: one pallet for tokens, one for governance, one for staking. On CKB, there's no equivalent. Your application's logic is scattered across scripts deployed as cells, invoked at transaction time, then silent.

But what you get in exchange is significant:

**Parallelism is natural.** Two transactions that don't consume the same input cells can be validated completely independently, simultaneously. There's no shared mutable runtime state to serialize around. This is why CKB can eventually support parallel transaction execution at the base layer without needing the elaborate scheduler that EVM-based chains require.

**Composability is structural, not contractual.** On Polkadot, composing two pallets requires their types to align and their storage layouts to be compatible. On CKB, composing two scripts means building a transaction that satisfies both scripts simultaneously. The composition happens at the transaction level, not at the code level. You can compose scripts that were written independently, by different teams, without coordination.

**Cryptography is not privileged.** CKB's native address format uses secp256k1 the same curve as Bitcoin and Ethereum. But secp256k1 is not hardcoded at the protocol level. It's implemented as a Lock Script in a system cell. JoyID's passkey-based wallet uses WebAuthn signatures instead, with a completely different Lock Script. A ZK-based account recovery protocol can use a proof system's verification key as a Lock Script. All of these are first-class citizens. There are no precompiles, no hard forks required, no permission needed.

---

## Who Should Make the Jump

If you're a Polkadot developer who has hit the friction of cross-parachain composability the complexity of XCM, the difficulty of building features that span multiple pallets on different chains CKB's cell model deserves serious attention. The transaction is the composition primitive. There's no protocol to route messages across; you just build a transaction that touches the relevant cells.

If you're a Polkadot dapp developer who's tired of RPC abstraction hiding what's actually happening on-chain, CKB will feel uncomfortably transparent at first, and then clarifying. You will understand exactly what your dapp is doing at every step. There are no implicit state changes from an extrinsic you didn't fully audit.

If you're building a ZK application a nullifier-based privacy protocol, a ZK proof verifier, a credential system CKB's architecture is worth understanding deeply. A ZK verifier is just a Type Script. It receives the transaction, reads the proof from a cell's data field, runs the verification logic (compiled to RISC-V), and returns 0 if the proof is valid. No precompile required. No gas limit that makes on-chain verification expensive at the base layer. The verification runs in CKB-VM, which is a real RISC-V processor.

The shift from Polkadot to CKB is not a lateral move. The tooling is different, the mental model is different, and the composability story is different. But the underlying question CKB is answering how do you build a programmable UTXO chain where ownership is enforced by code rather than assumed by convention is a question worth understanding, whatever you end up building on.

---

_Further reading: [CKB RFC on the Cell Model](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0002-ckb/0002-ckb.md) · [CKB Script Tutorial](https://docs.nervos.org/docs/script/intro-to-script) · [Lumos SDK](https://lumos.io/)_
