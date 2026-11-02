# mnemonic-toolkit

Paths below are relative to this Git root, including when working inside a crate.

## Start here

Read [CLAUDE.md](CLAUDE.md) explicitly; AGENTS.md does not load it automatically.
Read applicable ancestor guidance too. Existing process rules remain applicable;
verify historical architecture/status statements against source.
Read the shared [guide](docs/architecture/README.md) and [workflow](docs/architecture/WORKFLOW.md).
For sibling checkouts/worktrees, prefer local `../mnemonic-toolkit/docs/architecture/`;
remote fallbacks pin a delivered guide revision. See its workflow for refreshing the pin.
Check Git status, the current branch, Cargo.toml and Cargo.lock before editing.
Preserve unrelated work and use explicit task-owned paths when staging.

## Where behavior lives

- [main.rs](crates/mnemonic-toolkit/src/main.rs) wires CLI workflows;
  [lib.rs](crates/mnemonic-toolkit/src/lib.rs) exports only selected reusable utilities.
- Start bundle/recovery changes at `crates/mnemonic-toolkit/src/cmd/`, then
  [synthesize.rs](crates/mnemonic-toolkit/src/synthesize.rs) and
  [format.rs](crates/mnemonic-toolkit/src/format.rs).
  Preserve slot ordering, missing-material sentinels, origin/language/passphrase
  distinctions and verification against ground truth.
- [wc-codec](crates/wc-codec/src/lib.rs) owns public word-card transport;
  [the adapter](crates/mnemonic-toolkit/src/word_card_adapter.rs) owns mk/md conversion.
  Preserve source kind and md exact bit count.
- Consumer codec pins can differ from sibling checkout versions. Inspect Cargo.lock
  and vendored code; don't assume the newest sibling API is available here.

## Contracts and checks

The [architecture source map](docs/architecture/SOURCE_MAP.md) routes to specs and
bundle/restore/verification/word-card tests. Use current CI definitions, including
vendor provenance, shared drift and cross-tool checks, for implementation validation.
CLI changes must update [the shared manual](docs/manual/README.md) in lockstep;
coordinate mnemonic-gui schema, JSON consumers, binary pins and [GUI manual](docs/manual-gui/README.md).
Update the architecture guide with ownership or contract changes; refresh its dated
inventory deliberately. Keep init instructions small and route detail into the guide.
