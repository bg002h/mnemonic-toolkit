# Mini-R0 Review — toolkit descriptor fuzz target (round 1)

Reviewer: Fable 5 architect agent (a9695f1b5ef70f3ba), 2026-06-12.
Target: design/BRAINSTORM_toolkit_descriptor_fuzz_target.md @ toolkit master.
Persisted verbatim per CLAUDE.md convention.

## Verdict: GREEN

The mount COMPILES and FUZZES — the agent built it and ran 5000 iterations clean. The central worry (closure pulls in clap/`main`-only state) does NOT materialize: the true closure is **19 modules, excludes `cmd/` entirely**, references no `crate::Cli`/clap/`main`. One non-obvious wrinkle (shared bin+lib files use the self-name path `mnemonic_toolkit::mlock::…` which breaks when compiled as the lib) has a clean zero-source-edit fix: `#[cfg(fuzzing)] extern crate self as mnemonic_toolkit;`. NO-BUMP.

## Critical / Important
- none. (The self-name path would have been an Important blocker without the `extern crate self` fix; with it, fully resolved + proven.)

## Minor
- **M1 (plan-doc):** 3 non-test closure files (`derive.rs:11`, `synthesize.rs:17`, `derive_slot.rs:103`) use the external-crate self-name path `mnemonic_toolkit::mlock::…`. Do NOT rewrite to `crate::mlock` (breaks the normal BIN build — main.rs reaches mlock only via the external path). Fix = `#[cfg(fuzzing)] extern crate self as mnemonic_toolkit;` in lib.rs (self-name resolves inside the lib too, zero edits to shared files). Proven.
- **M2:** under fuzzing the lib emits ~13 unused-import warnings (wallet_export re-exports unused once cmd/ isn't compiled) — warnings only, no `#![deny(warnings)]`, fuzz build doesn't pass `-D warnings`. Leave them; do NOT `-D warnings` the fuzz build.
- **M3:** `error.rs` mounted `pub mod error` makes `ToolkitError` pub UNDER cfg(fuzzing) ONLY (binary-private in every normal/shipped build). No real API change; note it so the "binary-private by design" invariant isn't thought violated.
- **M4:** repo-root `fuzz/` with own `[workspace]` + committed `fuzz/Cargo.lock` (matches md/ms/mk pattern); lock resolves miniscript to the git rev independently of root.

## The make-or-break: COMPILES (recipe)
Built clean exit 0 in 39s; fuzzed 5000 runs, cov 4211, no crash. Recipe:
1. **lib.rs** cfg-gated block (self-alias load-bearing):
   `#[cfg(fuzzing)] extern crate self as mnemonic_toolkit;` then 19
   `#[cfg(fuzzing)] pub mod <m>;` for: cost, derive, derive_address,
   derive_slot, error, format, friendly, indel, language, network, parse,
   parse_descriptor, repair, secret_advisory, slip0132, slot_input,
   synthesize, template, wallet_export.
2. **crates/mnemonic-toolkit/Cargo.toml:**
   `[lints.rust]\nunexpected_cfgs = { level = "warn", check-cfg = ['cfg(fuzzing)'] }`.
3. **fuzz/Cargo.toml:** own `[workspace]`, `[package.metadata] cargo-fuzz=true`,
   `[patch.crates-io] miniscript = { git=…, rev="95fdd1c5773bd918c574d2225787973f63e16a66" }`,
   libfuzzer-sys 0.4, path-dep mnemonic-toolkit, `[[bin]]` test/doc/bench=false. Commit fuzz/Cargo.lock.
4. **fuzz/fuzz_targets/descriptor_parse.rs:** `String::from_utf8_lossy(data)` →
   `mnemonic_toolkit::parse_descriptor::parse_descriptor(&s, &[], &[])` (all pub; empty slices).
5. `cd fuzz && cargo +nightly fuzz build --target x86_64-unknown-linux-gnu descriptor_parse`.

## Answers to OQs 2-7
- **OQ2 closure:** 19 modules / ~17.7k lines (NOT 35/56k — the Cycle-C figure counted the whole cmd/ tree, but the only crate::cmd ref in the closure is a DOC COMMENT in wallet_export/mod.rs:160). The heavy tail (wallet_export/cost) is pulled by error.rs (ToolkitError embeds crate::wallet_export::{…} + crate::cost::CompareCostError in variants). Closed set verified.
- **OQ3 normal-build unaffected (cfg off):** `cargo build` OK; `clippy --all-targets -D warnings` exit 0 (the lint line is load-bearing — removing it HARD-FAILS clippy on unexpected_cfgs); `--lib` 117 passed; `cargo metadata --locked` exit 0; root Cargo.lock byte-unchanged.
- **OQ4 patch:** fuzz Cargo.lock resolves miniscript git rev 95fdd1c; compiles Terminal::SortedMultiA (parse_descriptor.rs:595, only in that rev) → patch took.
- **OQ5 less-invasive:** none thinner (closure not in lib graph) — but the full mount is much smaller than feared + zero edits to shared files. As non-invasive as a bin-private fuzz target gets.
- **OQ6 scope:** NO-BUMP (only the lint line compiles with cfg off; cfg block + extern-self entirely absent; ToolkitError pub only under fuzzing). Matches sibling codec-fuzz precedent.
- **OQ7 local feasibility:** confirmed (nightly 1.97.0-nightly 2026-04-27 + cargo-fuzz 0.13.2; built AND ran here).

## Evidence log
- Closure: transitive crate::<mod> walk (comments + cfg(test) stripped) → 19 modules, cmd/ excluded. No crate::Cli/clap/main/include! in any of the 19.
- Scratch mount → first build (no alias) 3× E0433 "cannot find crate mnemonic_toolkit" at derive.rs:11/synthesize.rs:17/derive_slot.rs:103 (self-name mlock paths; the cfg(test) ones didn't compile). Added extern-crate-self → Finished 39.05s exit 0, binary 70MB.
- fuzz run -runs=5000 -max_len=256 → Done, cov 4211, no crash (real instrumented fuzzer).
- Normal-build all green cfg-off; lint line load-bearing; root lock unchanged.
- Reverted all scratch; HEAD unchanged; tree as found.

GREEN — implementation cleared with the exact proven recipe.
