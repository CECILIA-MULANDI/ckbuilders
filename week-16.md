# CKB Builder Track - Week 16

**Week Ending:** 2026-07-06

## Focus This Week

Started **fiber-probe**, a Rust diagnostics CLI for Fiber nodes, as an entry into the [Gone in 60ms: Fiber Network Infrastructure Hackathon](https://talk.nervos.org/t/gone-in-60ms-fiber-network-infrastructure-hackathon-announcement/10418) (active window 2026-07-01 to 2026-07-15). Fiber is CKB's payment-channel network for fast off-chain payments. fiber-probe targets the hackathon's **Node, Routing, Cross-Chain, and Diagnostics Infrastructure** category: operator-facing tooling for monitoring node health and catching channel issues before they cause failed payments.

**Posted below:**

- Repo: [github.com/CECILIA-MULANDI/fiber-probe](https://github.com/CECILIA-MULANDI/fiber-probe)

## The gap I'm targeting

Fiber node operators today debug channel and routing issues by reading raw JSON-RPC output. That's workable for the team building Fiber and unusable for anyone else running a node. fiber-probe is the small, focused "pre-flight check" a merchant or wallet operator runs before pointing real traffic at their node: surface the failures, classify them, and tell the operator what is degraded vs. healthy without reading raw RPC dumps.

Deliberately narrow scope: a CLI, not a dashboard; a diagnostic, not a monitoring service.

## What landed this week

- **Cargo workspace scaffolded** with async JSON-RPC client (`reqwest` + `tokio`) and `thiserror`-based error types.
- **RPC layer.** JSON-RPC envelope types, hex deserializer, `NodeInfo` and `Channel` types, and an `RpcClient` with `node_info` and `list_channels`.
- **CLI (clap).** `status` command surfaces node identity and connection state. `check` command runs a preflight analyzer over the node's channels and returns a classified verdict (rich status output).
- **Preflight analyzer + classifier.** The `check` pipeline reads channel state and classifies findings, so an operator sees what is degraded and what is healthy without reading raw RPC dumps.

## What is next

- Broaden the analyzer's rule set: liquidity thresholds, peer connectivity health, uncooperative-peer detection.
- End-to-end demo against a running Fiber node.
- Video walkthrough, tutorial, and submission packet for the 2026-07-15 deadline.
