# CKB Builder Learning Journey

A dev log tracking my progress through the [CKB Builders' track](https://github.com/nervosnetwork) as part of the Community Keeps Building programme.

## About

This repository documents my hands-on learning of CKB (Common Knowledge Base) — the layer 1 blockchain of the Nervos Network. Each project folder contains working code along with notes on what I built and what I learned.

## Progress

### Beginner

| Exercise | Status | Folder |
|---|---|---|
| Transfer CKB | Done | [simple-transfer](./simple-transfer/) |
| Store Data on Cell | In progress | — |
| Create Fungible Token | Not started | — |
| Create DOB (Digital Object) | Not started | — |
| Build a Simple Lock | Not started | — |

### Intermediate

| Topic | Status |
|---|---|
| Script development course | Not started |
| Simple UDTs (sUDT) | Not started |
| Nervos DAO | Not started |
| Spore (DOBs) | Not started |

---

## Projects

### Simple Transfer — [simple-transfer/](./simple-transfer/)

A minimal dApp that connects a CKB wallet, displays the balance, and sends a CKB transfer on-chain.

**Stack:** React, TypeScript, CCC (`@ckb-ccc/core`), Parcel

**What I learned:**
- How the CKB cell model works in practice
- Using CCC (Common Chain Connector) to interact with wallets and broadcast transactions
- The difference between lock scripts and type scripts at a basic level

**Reference:** [Official tutorial](https://docs.nervos.org/docs/dapp/transfer-ckb)

---

## Tools & Resources

- [OffCKB](https://github.com/ckb-ecofund/offckb) — local dev environment
- [CCC Playground](https://app.ckbccc.com) — browser-based testing
- [CKB Academy](https://academy.ckb.dev) — structured lessons
- [Testnet Faucet](https://faucet.nervos.org) — get testnet CKB
- [CKB Debugger](https://github.com/nervosnetwork/ckb-debugger) — script debugging
