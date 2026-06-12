# R0 Review — Cycle D cross-tool differential (round 2)

Reviewer: Fable 5 architect agent (aaccfba7e88bc6d28), 2026-06-12.
Target: design/BRAINSTORM_stress_cycle_d_cross_tool_differential.md (R1 fold) @ toolkit master.
Persisted verbatim per CLAUDE.md convention.

## Verdict: GREEN

The four round-1 Important findings (I1–I4) and the five Minors (M1–M5) are all folded correctly and verified empirically. The make-or-break O2 oracle mechanic WORKS: `md inspect` consumes the toolkit's multi-chunk md1 array and yields a comparable `wallet_policy_id`; all claimed-Match controls actually match and all claimed-Diverge entries actually diverge through both tools; multi-key pairing is constructible. The I1 canonicity direction (md-cli conformant / toolkit deviant) is proven at the wire level. Residual gaps are presentational SPEC-text imprecisions (Minor), not design defects. 0 Critical / 0 Important → GREEN.

## Critical / Important
- (none)

## Minor (fold at implementation time)
- **m1 — `--json` field names are `wallet_policy_id.hex` / `wallet_descriptor_template_id.hex` (snake_case, nested under `.hex`), NOT the hyphenated text-form labels** the SPEC names. An implementer grepping for the hyphenated names extracts nothing.
- **m2 — pass each md1 chunk as a SEPARATE positional arg to `md inspect`/`md decode`, never space-joined** (joining → `character '1' not in codex32 alphabet`, the in-chunk `md1` HRP separator mis-tokenized). Spread the JSON `.md1` array into argv.
- **m3 — md-cli `encode --json` emits `.phrase` (single string) OR `{chunk_set_id, chunks:[...]}` for large/force-chunked policies.** The listed corpus all fits `.phrase`, but extraction should handle both defensively.
- **m4 — `tr(NUMS, multi_a(...))` needs the explicit-internal-key template form on the md-cli side** (`md encode 'tr(multi_a(...))'` errors "internal key must have no children"); use `tr(<NUMS_H_point>, multi_a(...))` or `--unspendable-key`, matched to the toolkit's tr-multi-a NUMS descriptor.
- **m5 — corpus xpubs MUST be FROZEN LITERALS (depth-3 for wpkh/SingleSig, depth-4 for wsh/MultiSig — md-cli rejects depth mismatches).** `mnemonic convert --template … --path …` cannot derive depth-4 (ignores the path override, returns the depth-3 account xpub). Ship depth-matched abandon-mnemonic xpubs as literals (deterministic). The depth-4 key `xpub6DkFAXWQ2dHxq…KFrf` (mfp `73c5da0a`) reproduces round-1's.
- **m6 — citation off-by-one: the `tap_context` gate is at `parse_descriptor.rs:604`, not `:603`** (fn sig `tap_context: bool` at :560; deliberate test `walk_check_kept_in_non_tap_context` at :2568).
- **m7 — the FOLLOWUP `toolkit-check-pkk-non-tap-non-canonical` is described in the brainstorm but not yet in either repo's FOLLOWUPS.md.** descriptor-mnemonic FOLLOWUPS.md:562 ALREADY has a related entry ("Walker context-dependent Check(PkK) mangling … a v2 design could normalize uniformly at the walker") — file the new toolkit-pointed entry and cross-link/retarget the existing md-cli-side note (impl-phase action).

## Fold verification table
| Round-1 finding | Resolved? | Notes |
|---|---|---|
| I1 Canonicity inverted → toolkit deviant, SPEC §5.1, FOLLOWUP retargeted | YES (proven) | Wire proof: md-cli wsh(pk) tree = `Wsh→[PkK]` bare; toolkit = `Wsh→[Check→[PkK]]`; payload-bits 656 vs 662. SPEC §5.1 = SPEC_v0_30_wire_format.md:246. Fix points at toolkit. (m6 :603→:604; m7 FOLLOWUP not yet filed.) |
| I2 Input-form pairing (md-cli triple, depth-matched) | YES | wpkh matches only with bare xpub + `--fingerprint` + `--path m/84'/0'/0'` → `1c0170fe`==`1c0170fe`; md-cli `--path` is single shared path (multi-key entries use one common origin). (m5 xpub provenance.) |
| I3 Four-arm Expect enum + both-exit-0-parseable gate | YES | Enum exactly 4 arms; and_v/or_d parse through both → genuine Diverge not ToolError. |
| I4 Primary oracle wallet-policy-id + template-id, O1 demoted | YES (make-or-break proven) | `md inspect --json` on toolkit chunk array → both IDs. All values reproduced: wsh(pk) `9ad78e4f`/`ef980fcc` vs `58d18033`/`9208f590`; wpkh `1c0170fe`/`45775d4d` match; wsh(multi 2-of-2) `3ea3fdf6`/`a235ee75` match; tr(pk-leaf) `d01703cf`/`c8fe87cd` match; wsh(pkh)/and_v/or_d diverge. (m1 json names; m2 separate-arg.) |
| M1 version skew wire-neutral; pin md tag | YES | only to_miniscript.rs delta (render direction). manual.yml:86 tag-pinned. |
| M2 ignore-gated test + env; CI md from manual-lint | YES | manual.yml:86 install + :106 MD_BIN=md reusable. |
| M3 extraction surfaces | YES (m3 caveat) | toolkit bundle --json .md1 array (stdout; advisory stderr); md-cli encode --json .phrase. |
| M4 curated corpus + match controls | YES | wpkh/wsh(multi)/tr(pk-leaf) MATCH; wsh(pk)/wsh(pkh)/and_v/or_d DIVERGE — non-vacuous. (m4 tr-multi-a form.) |
| M5 scope surface+pin+file NO-BUMP, fix deferred | YES | both md1s md decode to identical descriptor; interop not funds-loss. |

## Evidence log
(Both trees left as found; no commits; scratch in /tmp deleted. Binaries: toolkit mnemonic 0.54.4, md 0.6.2/md-codec 0.35.1.)
- O2 mechanic: toolkit `bundle --descriptor 'wsh(pk([73c5da0a/48'/0'/0'/2']xpub6DkF…/<0;1>/*))' --network mainnet --json`→`.md1` 3-chunk; `md inspect --json <c0> <c1> <c2>` (separate args)→`wallet_policy_id.hex`/`wallet_descriptor_template_id.hex`. Space-joined chunks FAIL.
- CASE wsh(pk) Diverge `9ad78e4f`/`ef980fcc` vs `58d18033`/`9208f590`; wpkh Match `1c0170fe`/`45775d4d`; wsh(multi 2-of-2) Match `3ea3fdf6`/`a235ee75` (two --key/--fingerprint + one shared --path m/48'/0'/0'/2'); wsh(pkh) Diverge; tr(pk-leaf) Match `d01703cf`/`c8fe87cd`; and_v/or_d Diverge (both exit 0).
- I1 wire: md-cli `Wsh→[PkK]`, toolkit `Wsh→[Check→[PkK]]`; 656 vs 662 bits. SPEC §5.1 SPEC_v0_30_wire_format.md:246.
- Citations: parse_descriptor.rs:604 `if tap_context`; :560 fn; :2568 test. Skew: git diff --stat md-cli-v0.6.2 HEAD -- crates/md-codec/src/ = to_miniscript.rs|17+ only. CI manual.yml:86 install / :106 MD_BIN=md.
- m5: depth-4 keys via raw BIP-32 reproduce round-1's xpub; mnemonic convert can't derive depth-4 → frozen literals.
