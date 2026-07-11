# Post-impl whole-diff R0 — T3-b md `wire_golden.rs` — Fable, adversarial

**Persisted verbatim per CLAUDE.md.** Repo `descriptor-mnemonic` @ `b9662e5f`, diff = 1 untracked test file, verified against live source + by execution under pinned 1.85.0. Tree left byte-clean.

## 1. Green
md-codec **435 passed / 0 failed** (24 binaries; new `wire_golden.rs` 4/4); md-cli **259 passed / 0 failed**. Deterministic across 4 executions + post-revert.

## 2. Goldens correct + populated + deterministic
(b)/(c)/(d) `.bytes.hex` + `total_bits` reproduced by the live encoder. Decode-back probe (temp, deleted): (b) carries exactly `pubkeys=Some[(0,0x11-fill‖G)]`; (c) `use_site_path_overrides=Some[(1,<2;3>/*)]`; (d) `Divergent([bip48-type-2, bip84])` IN THAT ORDER. Header divergent bit in raw bytes: (d) leads `0xa0` (bit 4 SET), (b)/(c) `0x20` (CLEAR) — matches `header.rs:31` + `encode.rs:113-117`. Traces confirmed: `tlv.rs:11` `TLV_USE_SITE_PATH_OVERRIDES=0x00`, `:16` `TLV_PUBKEYS=0x02`; pubkeys emit `tlv.rs:149-171`; use-site `:99-121`; Divergent write `origin_path.rs:125-127`; byte split `chunk.rs:267-273`. **(e) byte-identity (load-bearing):** both frozen consts byte-identical (incl. length) to lines 2-3 of committed `tests/vectors/wsh_sortedmulti_2chunk.phrase.txt`; hand-built struct field-matches the committed `descriptor.json` + `src/test_vectors.rs:94-103` MANIFEST (n=8, elided `"m"` origin, `<0;1>/*`, 8 fingerprints, `pubkeys:null`). Not a wrong-but-self-consistent oracle.

## 3. RED-proofs (each mutated → run → reverted checksum-identical)
- **A** — swap `TLV_USE_SITE_PATH_OVERRIDES`↔`TLV_PUBKEYS` values (`tlv.rs:11/16`): (b)+(c) RED + 2 pre-existing FROZEN oracles (`ascending_tag_order_enforced_in_encoder` tlv.rs:605, `renderer_matches_frozen_md_cli_0_11_2_snapshot`) — **zero round-trip failures** (432 passed).
- **B** — symmetric Divergent order reverse (`origin_path.rs:125-127` write `.rev()` + read `.reverse()`): **ONLY (d) RED** (436 passed; `divergent_paths_wallet_policy_2of2_round_trip` re-run GREEN; no pre-existing test fires — corpus has no divergent vector). Strongest gap-closure proof.
- **C** — chunk interior boundary shifted left 1 byte (partition-complete, capacity-safe, `chunk.rs:270-273`): md-codec **ONLY (e) RED** (437 passed, all chunking round-trips GREEN); md-cli **ONLY `vectors_output_matches_committed_corpus` RED** (258 passed) — exactly as predicted (a `diff -r` frozen oracle, `vector_corpus.rs:15`, not a round-trip).

## 4. NO-BUMP + gates (post-revert)
SHA-256 of `tlv.rs`/`origin_path.rs`/`chunk.rs` identical to pre-mutation; probe deleted; `git status` = only `?? wire_golden.rs`; `git diff src/` = 0; Cargo.toml/lock = 0 (no new dep — `hex` already a dev-dep). `cargo +stable fmt --all --check` exit 0; clippy `--workspace --all-targets -D warnings` exit 0; 1.85 active toolchain proven.

## Findings
Critical: none. Important: none.
Minor 1: `wire_golden.rs:20`/`:258` cited `design/RECON_T3b_api_feasibility.md` as a bare repo-relative path — that file lives in the TOOLKIT repo, not descriptor-mnemonic (md ships independently on crates.io → dangling ref). **[FOLDED — prefixed `mnemonic-toolkit:` in the ship.]**
Observations: mutation A overlaps 2 pre-existing frozen oracles (b/c still add absolute-tag-value freezing); (d)+(e) are uniquely load-bearing (nothing pre-diff catches either).

## VERDICT: GREEN (0C/0I)

---
**SHIP (opus, 2026-07-10):** GREEN; Minor-1 (dangling RECON citation) folded (`mnemonic-toolkit:design/…`). T3-b ready. Bundling with T3-a/c (toolkit) once its R0 greens → ship T3 = toolkit + md direct-FF NO-BUMP.