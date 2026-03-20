# CKB Builder Learning Journey

A dev log tracking my progress through the [CKB Builders' track](https://github.com/nervosnetwork) as part of the Community Keeps Building programme.

## About

This repository documents my hands-on learning of CKB (Common Knowledge Base); the layer 1 blockchain of the Nervos Network. Each project folder contains working code along with notes on what I built and what I learned.

## Progress

### Beginner

| Exercise                    | Status      | Folder                                      |
| --------------------------- | ----------- | ------------------------------------------- |
| Transfer CKB                | Done        | [simple-transfer](./simple-transfer/)       |
| Store Data on Cell          | Done        | [store-data-on-cell](./store-data-on-cell/) |
| Create Fungible Token       | Not started | —                                           |
| Create DOB (Digital Object) | Not started | —                                           |
| Build a Simple Lock         | Not started | —                                           |

### Intermediate

| Topic                     | Status      |
| ------------------------- | ----------- |
| Script development course | Not started |
| Simple UDTs (sUDT)        | Not started |
| Nervos DAO                | Not started |
| Spore (DOBs)              | Not started |

---

## Projects

### Simple Transfer — [simple-transfer/](./simple-transfer/)

A minimal dApp that connects a CKB wallet, displays the balance, and sends a CKB transfer on-chain.

**What I learned:**

- How the CKB cell model works in practice
- Using CCC (Common Chain Connector) to interact with wallets and broadcast transactions
- The difference between lock scripts and type scripts at a basic level

**Proof of completion:** [View transaction on explorer](https://testnet.explorer.nervos.org/transaction/0xc3a71dd081c3b73df34d667bd05f402e28dde81a4333e64ed91a78909d8d9afc)

**Reference:** [Official tutorial](https://docs.nervos.org/docs/dapp/transfer-ckb)

---

### Store Data on Cell — [store-data-on-cell/](./store-data-on-cell/)

A dApp that writes and reads arbitrary data to/from a cell on the CKB blockchain.

**What I learned:**

- How to store data in the `data` field of a CKB cell
- Reading cell data back from the chain
- The lifecycle of a cell: creation, update, and consumption

**Proof of completion:** [View transaction on explorer](https://testnet.explorer.nervos.org/transaction/0xacf26367645f894d04a32a8dcda26caacff9a6b2bd4c54ed475dd92e23e2680a)

**Reference:** [Official tutorial](https://docs.nervos.org/docs/dapp/store-data-on-cell)

---
