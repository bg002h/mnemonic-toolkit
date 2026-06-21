# IMPLEMENTATION PLAN — cycle-4 — codec funds-safety fixes (H6 / M4 / I1 / M6)

Phased TDD execution plan for the R0-GREEN brainstorm spec
(`design/BRAINSTORM_cycle4_codec_funds_fixes.md`, **0C/0I** at spec-R0 round 2 —
`design/agent-reports/cycle4-spec-r0-round{1,2}-review.md`). DESIGN ONLY — feeds the
mandatory opus-architect **plan-doc R0 loop to 0C/0I BEFORE any code**. Two registry
codecs → **publish→pin chain**.

## Source-of-truth SHAs (verified live)

| Repo | path | branch | SHA | crate(s) |
|---|---|---|---|---|
| md-codec / md-cli | `/scratch/code/shibboleth/descriptor-mnemonic` | `main` | `58cc9ec` | md-codec **0.37.0** → **0.38.0**; md-cli **0.8.0** → PATCH |
| ms-codec / ms-cli | `/scratch/code/shibboleth/mnemonic-secret` | `master` | `6b28918` | ms-codec **0.4.4** → **0.5.0**; ms-cli **0.8.0** → PATCH |
| mnemonic-toolkit (consumer) | `/scratch/code/shibboleth/mnemonic-toolkit` | `master` | `c578e123` | toolkit **0.62.0** → **0.62.1** PATCH |

**Execution model:** the two tracks are **file/repo-disjoint** and run as **two parallel
single-implementer worktrees** (one per codec repo), strict TDD (RED before GREEN),
**FULL `cargo test` per crate + `cargo clippy --all-targets -D warnings`** at each
phase gate. Per-phase opus review persists to `design/agent-reports/cycle4-*`. The
**convergence phase (toolkit pin-bump)** runs AFTER both codecs publish. Each track's
own R0-gated discipline (per CLAUDE.md): the codec changes are substantial enough to
warrant a brief per-track plan check, but the umbrella spec+plan R0 covers them — a
single implementer per track executes; a whole-diff review per codec precedes its tag.

---

# TRACK A — md-codec (H6 + M4 + I1), worktree off `origin/main` `58cc9ec`

## Phase A1 — H6 encode-side 80-data-symbol cap
**RED first** (`crates/md-codec/src/` tests): `wrap_payload_rejects_over_80_data_symbols` — build a descriptor whose md1 data exceeds 80 symbols, call the default (non-chunked) encode path, assert `Err(Error::PayloadTooLongForSingleString { data_symbols, max: 80 })`. Positive control: an exactly-80-data-symbol (93-codeword) payload still encodes GREEN. RED today (no cap).
**GREEN:** add `REGULAR_DATA_SYMBOLS_MAX = 80`; guard at the TOP of `wrap_payload` (`codex32.rs:67`, the lowest shared chokepoint — `encode_md1_string` `encode.rs:136` inherits) — `data_symbols.len() > 80 → Err(PayloadTooLongForSingleString{..})`. New variant in `error.rs` near `TooManyErrors` (md-codec `Error` is NOT `#[non_exhaustive]`). §4.

## Phase A2 — M4 decode-side `len > 93` rejection (correcting path)
**RED first:** (1) `decode_with_correction_rejects_over_93_symbol_chunk` — hand-craft a `>93`-symbol md1 with ≥1 error so today's `chien_search` aliases (the report's 331-symbol/pos-100 aliasing case); assert `Err(Error::ChunkSymbolCountOutOfRange{..})`. (2) `decode_regular_errors_returns_none_for_len_over_93` (unit, `len=94 → None`). (3) optional `chien_search_returns_none_for_len_over_93`. (4) positive control `valid_chunked_md1_still_repairs` (each chunk ≤93). All RED today (proceed into aliasing).
**GREEN:** typed boundary guard in `decode_with_correction` (`chunk.rs:502`, before residue/correction, runs pre-`residue==0` pass-through) → `Error::ChunkSymbolCountOutOfRange { chunk_index, symbols, max: 93 }`; internal `None`-floor at `decode_regular_errors` top (`bch_decode.rs:403`) + optional `chien_search` top (`:284`). New variant near `ChunkSetEmpty`/`ChunkCountExceedsMax` (**re-grep the exact placement lines at write time — they have drifted a few lines from the `error.rs:262-294` snapshot, plan-R0 m2**). Do NOT overload `TooManyErrors`. RED test #1 MUST assert the **exact post-fix variant** `Err(Error::ChunkSymbolCountOutOfRange { .. })` (not merely "does not cleanly reject"); the RED-today mechanism is that the uncapped `symbols.len()` enters the unbounded `chien_search` loop and mis-corrects at an aliased root (plan-R0 m4). §5.

## Phase A3 — I1 non-correcting decode cap (`unwrap_string`)
**RED first:** `unwrap_string_rejects_clean_over_93_symbol_string` — a CLEAN (residue==0, BCH-valid) `>93`-symbol md1 to `decode_md1_string`; assert `Err(Error::StringSymbolCountOutOfRange{..})`. Positive control: a 93-symbol legal string still decodes. RED today (`unwrap_string` accepts, decodes out-of-domain).
**GREEN:** in `unwrap_string` (`codex32.rs:113`), add a too-LONG ceiling `symbols.len() > 93 → Err(StringSymbolCountOutOfRange{symbols, max:93})` **before** `bch_verify_regular` (`:144`), symmetric with the too-short floor (`:151`). New variant (no `chunk_index`). §5.2.3 / D17.

**Track-A gate:** FULL `cargo test -p md-codec` (+ md-cli) GREEN + clippy clean. **md-codec now has 3 new `Error` variants** (`PayloadTooLongForSingleString`, `ChunkSymbolCountOutOfRange`, `StringSymbolCountOutOfRange`) — all additive → MINOR.

## Phase A-ship — md-codec 0.38.0 + md-cli PATCH (publish)
1. Bump `crates/md-codec/Cargo.toml` 0.37.0 → **0.38.0** + CHANGELOG; whole-diff review; tag `descriptor-mnemonic-md-codec-v0.38.0`; **`cargo publish -p md-codec`**.
2. md-cli: hand-edit the EXACT pin `md-codec =0.37.0` → **`=0.38.0`** (exact pin — not a `cargo update`), refresh lock, PATCH bump + CHANGELOG; new errors surface via opaque `CliError::Codec(_)` (exit 1) — no per-variant arm needed (LEAN PATCH). **NOTE the intentional exit-code divergence (plan-R0 m6):** md-cli collapses the new rejects to **exit 1** (opaque wrapper) while the toolkit routes the same codec errors to **exit 2** (§Phase-C). This is intentional (md-cli has no per-variant exit map); state it so it isn't mistaken for a bug. Tag `descriptor-mnemonic-md-cli-v0.8.1`; **`cargo publish -p md-cli`**.

---

# TRACK B — ms-codec (M6), worktree off `origin/master` `6b28918`

## Phase B1 — cross-share polynomial-consistency check in `combine_shares`
**RED first** (`crates/ms-codec/src/shares.rs` tests): (1) `combine_inconsistent_same_id_set_rejected` — two DIFFERENT secrets A,B each split 2-of-3 with SAME hrp/id/threshold/length; combine `[A1,B2]` → assert `Err(Error::InconsistentShareSet)` (RED today: returns garbage/B secret, no error — the funds RED). (2) `combine_valid_exactly_k_unchanged` (positive, MUST stay GREEN — exactly k=2 consistent → correct secret A, byte-identical). (3) `combine_valid_n_gt_k_all_consistent` (positive — all 3 consistent A-shares → A). (4) `combine_inconsistent_extra_share_rejected` — 2 consistent A + 1 same-id B (n>k) → `Err(InconsistentShareSet)`.
**GREEN:** replace step-5 (`shares.rs:263`): after the distinct-index check, take the **first k** of `parsed` as `k_set`, recover `interpolate_at(&k_set, Fe::S)`; then for each extra share `j` in `parsed[k..]`, assert `interpolate_at(&k_set, idx_j) == parsed[j]` (full canonical lowercased `Codex32String` compare), else `Error::InconsistentShareSet`. Reuses the existing arbitrary-index primitive (`shares.rs:153`); no codex32 change. New unit variant `Error::InconsistentShareSet` added near the combine-family **variant declarations** in `error.rs` (the `SecretShareSuppliedToCombine` variant decl is `error.rs:122`; `shares.rs:235` is its *reject* site — plan-R0 m1). **MUST add a `Display` arm** at the exhaustive Display impl (`error.rs:125` — compile-forced, no `_ =>`). §6.

**Track-B gate:** FULL `cargo test -p ms-codec` (+ ms-cli) GREEN + clippy clean. M6 framing: beyond-BIP-93 defense-in-depth; valid exactly-k / all-consistent combines bit-identical.

## Phase B-ship — ms-codec 0.5.0 + ms-cli PATCH (publish)
3. Bump `crates/ms-codec/Cargo.toml` 0.4.4 → **0.5.0** + CHANGELOG; whole-diff review; tag `mnemonic-secret-ms-codec-v0.5.0`; **`cargo publish -p ms-codec`**.
4. ms-cli: hand-edit EXACT pin `ms-codec =0.4.4` → **`=0.5.0`**; **add an explicit `InconsistentShareSet` arm** in `From<ms_codec::Error>` (`crates/ms-cli/src/error.rs:246` — the wildcard otherwise silently maps to `BadInput`/exit 1) → route to exit-2 FormatViolation + accurate message; PATCH bump + CHANGELOG; tag `mnemonic-secret-ms-cli-v0.8.1`; **`cargo publish -p ms-cli`**.

---

# CONVERGENCE — Phase C: toolkit PATCH pin-bump 0.62.1 (after BOTH codecs on crates.io)

**BLOCKING caret-pin hand-edits** (`cargo update` will NOT cross these — `^0.37`<0.38, `^0.4.4`<0.5.0):
- `crates/mnemonic-toolkit/Cargo.toml:36` `md-codec = "0.37"` → **`"0.38"`**; `:29` `ms-codec = "0.4.4"` → **`"0.5"`**. (`codex32 = "=0.1.0"` `:34` UNCHANGED.) Then `cargo update -p md-codec -p ms-codec` / build to refresh `Cargo.lock` + `fuzz/Cargo.lock`.

**Lockstep arms:**
- **md side (COMPILE-FORCED):** `md_codec_exit_code` (`error.rs:464`) is an EXHAUSTIVE match (no `_ =>`); the 3 new md-codec variants WILL fail to compile until arms are added. Add `PayloadTooLongForSingleString`, `ChunkSymbolCountOutOfRange`, `StringSymbolCountOutOfRange` → **exit 2** (decode/format-reject class, alongside the `TooManyErrors` group at `:516`). The compiler catches a miss.
- **ms side (SILENT — explicit, no compiler):** `ms_codec_exit_code` ends `_ => 1` (`error.rs:419`) → add an explicit `InconsistentShareSet => 2` arm in the funds/format group (`:417`). ALSO add `friendly_ms_codec` prose (`friendly.rs`) for the new variant. `From<ms_codec::Error> for ToolkitError` (`:929`, wildcard `:939`) maps automatically — no edit. The toolkit call site `cmd/ms_shares.rs:409` inherits via the pin.

**Toolkit characterization tests (RED→GREEN against the bumped pins):** (a) `mnemonic repair --md1 <over-93 clean/dirty>` → non-zero exit 2; (b) **`mnemonic restore --md1 <clean over-93 string>` → exit 2** — I1's `unwrap_string` cap also fires via the `md_codec::chunk::reassemble` → `unwrap_string` surface used by restore/inspect/bundle, NOT just `repair` (plan-R0 m3); test this path too so the non-correcting cap is covered where users actually hit it; (c) `mnemonic ms-shares combine <inconsistent same-id set>` → exit 2 + friendly prose, AND a valid set still combines to the right secret.

> **Scope note (plan-R0 m5):** the toolkit `repair --md1 --max-indel ≥ 1` indel-search path does its own length exploration and is NOT gated by the codec's M4 cap — it is **out of cycle-4 scope** (a future cycle item; do not assume cycle-4 covers it). Noted so the next cycle doesn't presume coverage.

**Ship:** bump toolkit 0.62.0 → **0.62.1** + CHANGELOG; release ritual — **BOTH READMEs + `fuzz/Cargo.lock`** version sites (per `project_toolkit_release_ritual_version_sites`); re-run FULL suite + fuzz before tag; whole-diff review; tag `mnemonic-toolkit-v0.62.1`. (toolkit is NOT a registry crate — tag only, no publish.)

**Version-collision note (M-min-1):** toolkit `origin/master` is 0.62.0; cut this PATCH off `origin/master` → 0.62.1. The unmerged `feature/own-account-subset-search` branch is live at 0.60.0 and *plans* to renumber when it ships — that renumber is THAT cycle's concern; this cycle simply takes 0.62.1 and does not touch the paused branch. (The "0.63.0" figure for own-account is speculative — do not hard-code a dependency on it.)

---

## Branch / worktree discipline (multi-instance)
The main toolkit checkout is parked on the paused `feature/own-account-subset-search` — **do NOT commit there**. All cycle-4 work in worktrees off the respective `origin` default branches; toolkit design-trail + Phase-C pin-bump via a toolkit master worktree (as cycle-3). Stage paths explicitly. **NEVER `cargo fmt`** any constellation crate that is fmt-exempt (toolkit `mlock.rs`; check each repo's convention — md-codec/ms-codec follow their own).

## FOLLOWUP slugs (file/flip in the shipping commits)
`encode-no-regular-code-length-cap` (H6), `chien-search-unbounded-length` (M4, note I1 non-correcting facet folded), `w2-ms-slip39-gf256-1` (M6) → RESOLVED with fixing tags. Tick **H6/M4/M6** `[ ]`→`[x]` in `design/agent-reports/constellation-bughunt-2026-06-20.md` citing the codec tags. Optional new `md-codec-exit-code-exhaustive-match-lockstep` note.

## Phase order & gates
A and B fully parallel → each publishes independently → **C is the single join** (needs both on crates.io). Per-phase: RED→GREEN→FULL suite+clippy. Per-codec whole-diff review before its tag. Toolkit whole-diff review before 0.62.1. **Plan-R0 must converge 0C/0I before ANY Phase-A1/B1 code.**
