# AgentMem

Agent Memory System for Persistent Context in AI Coding Agents.

## Setup

```bash
cargo build --release
cp target/release/am /usr/local/bin/am
```

## Usage

```bash
am init
am task create "Fix auth bug"
am context --query "auth bug"
```

Refer to [AgentMem-PRD.md](AgentMem-PRD.md) for full details.

