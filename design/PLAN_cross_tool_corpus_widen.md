# PLAN — widen the cross-tool md-cli differential corpus (GAP 4a)

**Date:** 2026-06-12 · **Repo:** mnemonic-toolkit · **SemVer:** NO-BUMP (test-only; a found walker divergence would spawn a SEPARATE fix cycle — see §5)
**Source SHA:** mnemonic-toolkit `origin/master` = `1f0eb74`. Companion md-cli pinned at `md-cli-v0.6.2` (descriptor-mnemonic). **Recon:** `cycle-prep-recon-differential-harness-breadth.md`. **FOLLOWUP:** none for harness breadth itself (the GAP-4 trackers were filed in the hygiene pass).

## 1. Problem

`tests/cli_cross_tool_differential.rs` (`#[ignore]`-gated, needs `MNEMONIC_BIN`+`MD_BIN`; CI `cross-tool-differential.yml`) compares the toolkit's emitted md1 ids (`wallet_policy_id` + `descriptor_template_id`) against md-cli's for the same wallet. The harness EXISTS because the two walkers diverged before (the v0.55.0 Check-PkK fix). But its corpus is **8 entries** — wpkh, pkh, 2-of-2 multi, tr-pk-leaf, wsh-pk/pkh/and_v/or_d — all now `Verdict::Match`. It OMITS exactly the fragments where the walkers could re-diverge: `sortedmulti`, multisig with ≥3 cosigners, `thresh`, timelocks (`older`/`after`), hashlocks, `or_i`, `sh(wsh(...))`, and taproot `multi_a`.

## 2. The fix — add `Entry` rows (all `Verdict::Match`) — EMPIRICALLY VERIFIED

Each row = one `Entry { label, toolkit_descriptor (concrete `[fp/path]xpub`), md_template (`@N` form), md_keys, md_path, expect: Match }`. **All seven rows below were run through BOTH tools (the built `mnemonic` + `md` v0.6.2) via a probe (`/tmp/xtool_probe.sh`) and MATCH on (wallet_policy_id, wallet_descriptor_template_id):**

| label | toolkit shape (keys = `[73c5da0a/48'/0'/0'/2']`-bracketed `X0`/`X1`) | probe |
|---|---|---|
| `wsh-sortedmulti-2of2` | `wsh(sortedmulti(2,X0,X1))` sole-child | MATCH ✓ |
| `wsh-thresh-2of2` | `wsh(thresh(2,pk(X0),s:pk(X1)))` | MATCH ✓ |
| `wsh-and_v-older` | `wsh(and_v(v:pk(X0),older(144)))` | MATCH ✓ |
| `wsh-and_v-after` | `wsh(and_v(v:pk(X0),after(800000)))` | MATCH ✓ |
| `wsh-or_i` | `wsh(or_i(pk(X0),pk(X1)))` | MATCH ✓ |
| `wsh-andor-hashlock` | `wsh(andor(pk(X0),older(144),and_v(v:pk(X1),sha256(H))))`, H=`00…01` | MATCH ✓ |
| `sh-wsh-sortedmulti` | `sh(wsh(sortedmulti(2,X0,X1)))` | MATCH ✓ |

Coverage added: sortedmulti (sole-child + nested `sh(wsh)`), thresh+`s:`, both timelock classes (`older`/`after`), `or_i`, a hashlock (`sha256` under `andor`). All `Verdict::Match`. md_path = `m/48'/0'/0'/2'` (matches the shared bracket); keys = the existing `XPUB4_0`/`XPUB4_1` constants.

**DROPPED (with reason — note in the test, not silently):**
- **`wsh-multi-2of3` / `wsh-thresh-2of3` (≥3 cosigners)** — md-cli enforces `xpub depth == path depth` (probe: `md: --key @2: expected depth 4 for this script context, got 3`). The corpus has only TWO depth-4 xpubs (`XPUB4_0`, `XPUB4_1`); a 3rd would need deriving a new depth-4 account xpub. The existing `wsh-multi-2of2` + the new `wsh-thresh-2of2` cover the multi/thresh families; the n≥3 key-index surface is incremental → deferred (add a 3rd depth-4 const in a follow-up if wanted).
- **`tr(NUMS,multi_a(...))` / `tr(NUMS,sortedmulti_a(...))`** — taproot-NUMS spelling parity (toolkit H-point internal vs md-cli template NUMS) is unresolved and tangled with the GAP-1 sortedmulti_a gap (md-codec 13.0.0 lacks `Terminal::SortedMultiA`). The existing `tr-pk-leaf` row already covers the tap-collapse path. Taproot multisig differential is deferred to a tr-specific cycle (it belongs with the GAP-1 taproot work). Documented, not silently omitted.

**`sortedmulti` SOLE-CHILD only** (not in a combinator) — the combinator form is the GAP-3 round-trip asymmetry; sole-child is the clean parity test.

## 3. Anti-vacuity / harness (unchanged)
The existing guards already cover the widened corpus: `n_match>=1`, `saw_match`, `n_both_error==0 && n_tool_error==0`. No harness-logic change — every new row is a `Match` declaration, and a real divergence flips it to a `failures` entry (loud). A new row that lands `ToolError` (md-cli can't build it) trips `n_tool_error==0` → caught at impl, not shipped.

## 4. Verification (impl) — REQUIRES both binaries
Build `mnemonic` (`cargo build -p mnemonic-toolkit`) + `md` (`cargo build -p md-cli` in descriptor-mnemonic), then `MNEMONIC_BIN=… MD_BIN=… cargo test -p mnemonic-toolkit --test cli_cross_tool_differential -- --ignored --nocapture`. A one-shot probe FIRST (run a single new row, read the `[label] Verdict OK toolkit=… md-cli=…` line) confirms md-cli accepts each new template + the right `md_path` depth (the bitcoind corpus uses `m/86'/0'/N'` for tap; multi uses `m/48'/0'/0'/2'`). **GREEN gate:** all rows `Match`, anti-vacuity green, `cargo build` clean, `cargo fmt --all --check` clean (NOTE: per the mlock-rs-fmt-exempt rule, do NOT touch mlock.rs; this cycle doesn't).

## 5. Found-divergence contingency (the point of the cycle)
If ANY new row lands `Diverge` (toolkit ids ≠ md-cli ids), that is a **real walker divergence = a found bug** (like Check-PkK). Do NOT paper over it by declaring the row `Diverge`/expected — STOP, characterize which id differs (policy vs template) and why (which walker is non-canonical vs SPEC v0.30), and spin a SEPARATE fix cycle (possibly a MINOR wire change, per the v0.55.0 precedent). This cycle ships only the rows that legitimately Match; a divergence row is held back into its own fix cycle.

## 6. Lockstep / SemVer
- NO-BUMP (one test file). No clap change → no manual/GUI/schema_mirror. No md-cli/md-codec change. The CI workflow `cross-tool-differential.yml` already triggers on the test path — no workflow edit.

## 7. R0 questions — ANSWERED (R0 GREEN round 1)
1. **Row set** → 7 probe-MATCHED rows + R0-M2 added 2 more probe-MATCHED (`and_b`+`a:`, `t:or_c`) = **9 rows**, all empirically Match through both built binaries. Final.
2. **tr-multi_a NUMS** → DROPPED (md-cli/toolkit internal-key-spelling parity unresolved; would manufacture a spurious Diverge tangled with GAP-1). Deferred to a tr-specific cycle.
3. **hashlock literal** → CONFIRMED (fixed 64-hex `00…01`, probe-matched; carried opaquely by both tools).
4. **Found divergence** → ship the Match rows, carve out + separate-cycle any diverging row (none triggered — all 9 Match). Plus R0-M4: pin the corpus count (`assert_eq!(entries.len(), 17)`) so a deleted row is loud.
