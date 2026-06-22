# INDEPENDENT WHOLE-DIFF REVIEW — cycle-13 Lane A (coldcard/jade multisig fidelity H11+H14)

The mandatory independent review (the implementer correctly DEFERRED its own — no dispatch tool in its session, per CLAUDE.md item 5; its self-review note is explicitly labeled non-substitute). Everything verified independently. Worktree `wt-cycle13a`, off `origin/master = 9b2a8ae3` (v0.65.2). Commits `4a681410`/`4bae1491`/`7fd9a0b9`/`a6ea6474`.

## VERDICT: GREEN — 0 Critical / 0 Important / 0 Minor

### Axis 1 — H14 import refuse-matrix (funds-safety): CORRECT
Depth-gated matrix in `wallet_import/coldcard_multisig.rs:383-444`, exhaustive and exactly as specified:
- Discriminator is genuinely `xpub.depth == 0` (`computed_depth` from `x.depth` at `:379`, `xpub_is_master` at `:380`, computed fresh per-cosigner).
- **Core fix:** depth>0/no-XFP → `(None, Some(_)) => return Err(ImportWalletParse(...))` `:420-431` (early return, exit 2, no fall-through). `effective_fp` is the ONLY source for `path_raw` (`:460`) + resolved fp (`:463`); traced — no path lets a depth>0 account fp through as a master fp.
- depth>0/WITH-XFP → arm 1 `(Some(supplied), _) if !xpub_is_master => supplied` `:389` (silent, no warning; first arm shadows the Row-2 warn arm for all depth>0).
- Adversarial inputs all trace: depth-0 match→silent (`:391`), depth-0 mismatch→warn+supplied (`:396`, depth-0 only), depth>0+XFP→silent, depth>0 no-XFP→refuse, mixed→per-cosigner (refuse on first offending `{i}`).
- Jade inherits via shared `parse_text`; `import_jade_depth_gt0_no_xfp_refuses` GREEN (exit 2).

### Axis 2 — H11 export sorted-slot pairing (I-2 critical): CORRECT, no scramble
`wallet_export/coldcard.rs:359-395`: `sorted_paths` built from the post-sort `cosigners.iter()` (`:359-362`) and zipped with the same sorted `cosigners` in the emit loop (`:384`) — path/xpub/fp all from the SAME sorted slot; NO separate slot-order `derivations[i]` vector. Independently decoded the I-2 fixture: slots `[A,B,C]`, xpub-lex sort `[C,B,A]` (fully reversed), distinct paths 0'/1'/2' → `#1b` genuinely exercises sort≠slot and is GREEN for the right reason (a naive `derivations[i]` would pair sorted-pos-0=C's xpub with A's path and fail). Shared single line kept when homogeneous (`:379-382`, byte-identical guard `#2`). Empty origin → `BadInput` exit 1 (`:341-353`), never `m/0'/0'`. Jade delegates (`jade.rs:46`); `#4` GREEN.

### Axis 3 — Q1 arm + I-1 canonicalizer + end-to-end coherence: COHERENT
- Q1: `<XFP>:` arm (`:265`) consumes only `pending_per_cosigner_path.take()`; `shared_derivation` never written. 3-cosigner shared-path test resolves cosigners 2..N via `.or(shared_derivation)` (`:351`). No regression.
- I-1 (`roundtrip.rs:393-446`): paths ride the `(cosigner_line, path)` tuple through the sort — no scramble; per-cosigner on heterogeneous, single shared on homogeneous; idempotent.
- **End-to-end:** traced an H11 divergent export through re-import — per-cosigner blobs carry `<XFP_master>:` → depth>0-WITH-XFP = H14-c silent-accept (no refuse, no warning); per-line paths land in `per_line_path` overriding shared. `roundtrip_divergent_master_fp_and_paths_preserved` + `roundtrip_verify_divergent_coldcard_multisig_passes` GREEN.

### Axis 4 — Fixture-break reconciliation: COMPLETE, no masked-bug weakening, no 7th break
Depth-0 consts independently decoded: `XPUB_D0_A/B/C` genuine depth-0 masters with fps `57ACB302`/`0734F923`/`689B0FA9` = pinned `FP_D0_*` (`depth0_const_fingerprints_pinned`); old `XPUB_A/B/C` depth 4/4/3. `:1042` GENUINE SPLIT — re-pointed depth-0 (H14-a silent) AND new `parse_no_header_depth_gt0_refuses` (H14-b REFUSE, exit 2). `:1418` carries `XFP: DEADBEEF` so coin-type check stays reached; `:990`/`:1265` + CLI `:166`/`:209` re-pointed depth-0 (asserts KEPT). On-disk `coldcard-ms-*.txt` all carry matching per-line `<XFP>:` → H14-c silent, unchanged. No 7th breaking test.

### Axis 5 — Scope / gates / SemVer: CLEAN
Touches only the 10 declared files; file-disjoint from Lane B (`cmd/restore.rs`) + Lane C (`import_wallet.rs`/`bundle.rs`/`electrum.rs`). No version sites, no `error.rs` (reuses `ImportWalletParse`/`BadInput`), no clap flag → no schema_mirror, no new `--flag` in manual prose → manual lint won't fire, no fmt churn. `cargo test -p mnemonic-toolkit` 3388 passed, 0 failed; `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` clean.

## Disposition
GREEN. Lane A clears the gate; integrated into toolkit v0.66.0 with Lanes B + C.
