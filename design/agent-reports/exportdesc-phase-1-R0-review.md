# export-wallet --format descriptor — Phase 1 R0 Review
**Verdict:** GREEN (0C/0I)

Phase 1 diff `git diff dab9d96..d1532e7` (5 files). Gate independently re-confirmed by the controller: `cargo test -p mnemonic-toolkit --no-fail-fast` → 0 FAILED (2661 passed, 12 ignored); `cargo clippy --all-targets -- -D warnings` → exit 0; `cli_export_wallet_descriptor` 8/8.

## Critical (0) / Important (0) / Minor (2)

## A emitter+dispatch — CORRECT
`DescriptorEmitter` (`wallet_export/descriptor.rs:11-26`) implements all 3 trait methods (`mod.rs:397-401`): collect_missing→Vec::new(); emit→`Ok(inputs.canonical_descriptor.to_string())` NO trailing `\n` (the 4 dispatch tails add exactly one — stdout writeln! `:567`/`:823`, file `format!("{emitted}\n")` `:574`/`:830`); extension→"txt". Registered alphabetically (mod `:24`, re-export `:44`, between coldcard+electrum). All FIVE exhaustive match sites carry a `Descriptor` arm (`:56/:58` format_requires_template `=> false` passthrough; `:518` run collect_missing; `:560` run emit; `:774` from-import-json collect_missing; `:816` from-import-json emit) — no `_` left; semantically right (descriptor = passthrough, no template injected).

## B output + 00000000 fp — CORRECT not a bug
`wpkh([00000000/84'/0'/0']xpub6CatW…/<0;1>/*)#d9qwe873`. `00000000` = BIP-32 zero fp because a bare `--slot @0.xpub=` carries no `[fp/path]` origin (`pipeline::key_origin_str` builds `[00000000/<path>]`) — same as every bare-slot-xpub export. `#d9qwe873` is miniscript-computed (build_descriptor_string round-trips from_str→to_string appending the canonical checksum) + binary-captured (not hand-written); the `_exact` test asserts the captured value + the smoke test asserts `#`+8-alnum. Valid BIP-380.

## C round-trip — VERIFIED load-bearing
Both tests exercise the P11A pattern (`import-wallet --blob <fixture> --format <src> --json` → `export-wallet --from-import-json - --format descriptor`) via a faithful local copy of run_export_from_import_envelope. Single-sig fixture core-bip84-mainnet.json; multisig sparrow-multisig-2of3-p2wsh-sortedmulti.json (both real). Asserts `body(out) == body(envelope[0].bundle.descriptor)` (checksum-recompute normalized) — load-bearing (from-import-json sets canonical_descriptor=parsed_ms.to_string() :697; emitter returns exactly that). Taproot NOT round-tripped via from-import-json (refused :677-687) + tested via direct --descriptor passthrough (verbatim re-emit).

## D cli_gui_schema.rs deviation — REQUIRED + correct + class-checked
`export_wallet_has_format_dropdown_with_eight_vendors` (`:262-293`) 10→11 + added "descriptor". (a) REQUIRED: gui-schema reflects the clap enum so the new variant surfaces automatically → count 11. (b) correct edit. (c) class-checked: the only sibling count-assert is bundle `--template` len at `:180` (unrelated, untouched). DISTINCT from the mnemonic-gui-repo EXPORT_FORMATS lockstep (the GUI mini-cycle, not this phase). Toolkit-side gui-schema self-test.

## E tests + no-leak — CLEAN
multisig/flags-ignored/--output tests genuine (not vacuous). No Phase-2 leak (Cargo.toml still 0.41.0; README marker 0.41.0; CHANGELOG top [0.41.0]; no install.sh/manual/mnemonic-gui touch). No secret leak: descriptor output is canonical_descriptor (xpubs+origins+checksum); validate_watch_only (`:273`) + validate_watch_only_resolved (`:368`) refuse phrase/entropy/xprv/wif pre-emit. Public-only.

## F gate — CONTROLLER-CONFIRMED GREEN (reviewer's harness had no shell)
0 failed / 2661 passed; clippy exit 0; descriptor cells 8/8. partition_is_exact (`:846-853`) includes Descriptor in the passthrough array (logic test).

## Minor (2, non-blocking)
- m1: cosmetic — the single-sig vs multisig round-trip fixtures use distinct source formats (bitcoin-core vs sparrow); a doc-comment could note the asymmetry.
- m2: the test fn name `..._eight_vendors` is now stale (says eight, asserts eleven) — pre-existing naming debt (already said eight while asserting 10); renaming out of Phase-1 scope.

## Verdict rationale
Emitter correct (3 methods, single-newline); 5 match arms semantically right; 00000000 fp correct (not a bug); checksum miniscript-computed+captured; round-trip load-bearing against real fixtures + taproot correctly passthrough-only; gui-schema test edit required+correct+class-checked; no leak/over-reach; gate GREEN (controller-confirmed). No C/I; 2 cosmetic Minors. GREEN — proceed to Phase 2.
