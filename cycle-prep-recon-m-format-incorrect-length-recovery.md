# cycle-prep recon — 2026-05-24 — m-format-incorrect-length-recovery

**Origin/master SHA at recon time:** `925f5ed`
**Local branch:** `master`
**Sync state:** `up-to-date (0 ahead / 0 behind)`
**Untracked:** `.claude/`

Slug verified: `m-format-incorrect-length-recovery`. **Expectation met: ZERO line-number drift** — origin/master `925f5ed` adds only the `docs(continuity)` commit on top of `7106125` (where the FOLLOWUP + continuity doc were written), so every cited source file is byte-unchanged since the citations were filed. One non-line issue surfaced: a sibling-codec version skew (md-codec).

---

## Per-slug verification

### `m-format-incorrect-length-recovery`
- **WHAT (from FOLLOWUPS.md):** Recover an `m*1` string (md1 / mk1 / ms1) where a character was **inserted (too long)** or **dropped (too short)** during hand-copy/engraving, so it no longer decodes. Distinct from `mnemonic repair` (BCH *substitution* at FIXED length); an indel shifts every subsequent symbol and breaks the BCH codeword → needs a different algorithm. Likely toolkit-side **enumerate-and-validate** (delete each position for too-long; insert each of 32 charset symbols at each position for too-short) using the codec `decode` as the validity oracle; probably no sibling-codec change. Full handoff: `design/CONTINUITY_m_format_incorrect_length_recovery.md`.

- **Citations:**
  - `crates/mnemonic-toolkit/src/repair.rs:28` — `ALPHABET` import (bech32 32-symbol charset = validity-oracle charset) — **ACCURATE**. `use mk_codec::string_layer::bch::{` opens at :27; `ALPHABET, BchCode, GEN_LONG, …` is the first item on :28.
  - `crates/mnemonic-toolkit/src/repair.rs:406` — `RepairError::ReservedInvalidLength` variant — **ACCURATE** (`406:    ReservedInvalidLength {`).
  - `crates/mnemonic-toolkit/src/repair.rs:414` — `RepairError::UnsupportedCodeVariant` variant — **ACCURATE** (`414:    UnsupportedCodeVariant {`).
  - `crates/mnemonic-toolkit/src/repair.rs:388` — `enum RepairError` decl (continuity doc `:388`) — **ACCURATE** (`388:pub enum RepairError {`).
  - `crates/mnemonic-toolkit/src/repair.rs:140` — `validate_flag_hrp` (continuity doc, no line) — **ACCURATE** (`140:pub(crate) fn validate_flag_hrp(`).
  - `bch_code_for_length` picks the BCH code variant *from the input length*; wrong length → `ReservedInvalidLength` (94/95) or out-of-range — **ACCURATE**. `repair.rs:559` `let code = match bch_code_for_length(values.len())`; the `None if values.len() == 94 || 95` arm returns `ReservedInvalidLength` at :562. This is the precise mechanism that makes wrong-length inputs un-repairable by the existing path — the FOLLOWUP's core motivation is correct.
  - `crates/mnemonic-toolkit/src/cmd/inspect.rs:195` — `inspect` reports `byte_length` but offers no recovery — **ACCURATE** (`195:            writeln!(stdout, "byte_length: {}", bytes.len())`).
  - `crates/mnemonic-toolkit/src/cmd/final_word.rs` — the enumerate-candidates-validate-by-checksum analogue — **ACCURATE**. `final_word_candidates(...)` at :80 enumerates, validates by checksum, emits sorted list to stdout, with a secret-material stderr advisory (:104) — the exact pattern the new feature should mirror (incl. the secret-on-stdout advisory for ms1).
  - sibling codecs' `decode` / `decode_with_correction` as the per-candidate oracle — **ACCURATE (with a load-bearing signature caveat — see Cross-cutting #2/#3)**. Confirmed present:
    - ms-codec: `decode(s: &str)` `decode.rs:27`; `decode_with_correction(s: &str)` `decode.rs:188` — single string.
    - md-codec: `decode_md1_string(s: &str)` `decode.rs:79`; `decode_with_correction(...)` `chunk.rs:492` — single string + chunked path (matches continuity's "md-codec `chunk.rs`").
    - mk-codec: `decode(strings: &[&str])` `key_card.rs:114` — takes a **SLICE** (multi-string / long codes); plus `decode_string(s: &str)` `bch.rs:650`.
  - `scripts/install.sh:32` self-pin — **ACCURATE** (`:32` = `echo "mnemonic-toolkit|…|mnemonic-toolkit-v0.37.0|no|"`).
  - "DO NOT plan on bech32 upstream `Corrector` — still unavailable (v0.11.1)" — **NOT RE-VERIFIED here** (no bech32 upstream pin inspected); documented in companion FOLLOWUP `bech32-upstream-corrector-migration`. Treat as a standing constraint; brainstorm need not re-litigate.

- **Action for brainstorm spec:** Citations are clean — lift them directly, citing source SHA `925f5ed`. Two things the brainstorm/plan MUST do beyond the FOLLOWUP body:
  1. **Re-verify md-codec decode signatures against the PINNED `0.34.0`**, not the local dev `0.35.0` checkout this recon read (Cross-cutting #2). ms-codec (`0.2.0`) and mk-codec (`0.3.1`) local == pinned, so those are trustworthy as-read.
  2. **Design the enumerate-and-validate harness around the divergent oracle signatures** (Cross-cutting #3): ms1 = single `&str`; md1 = single `&str` + chunked forms (which chunk is wrong-length?); mk1 = `&[&str]` slice (long codes). A single uniform oracle wrapper will not fit all three — this is the substance behind open-decision #3 (which HRPs / staging).

---

## Cross-cutting observations

1. **No DRIFTED-by-N findings.** Every cited line is exact. Reason: origin/master `925f5ed` is `7106125` + one `docs(continuity)` commit only; no source file changed since the citations were filed. This is the cleanest possible recon — citations have had zero merges to decay through.

2. **Sibling-codec version skew (md-codec).** Toolkit pins (Cargo.lock): `ms-codec 0.2.0`, `mk-codec 0.3.1`, `md-codec 0.34.0` (all `registry+crates.io`, checksummed). Local dev checkouts: ms `0.2.0` ✓, mk `0.3.1` ✓, **md `0.35.0` ≠ pinned `0.34.0`**. The md-codec decode APIs verified above came from the local `0.35.0` HEAD. Per `feedback_verify_cited_apis_against_docs_rs`, the brainstorm/plan must confirm `decode_md1_string` / `chunk.rs::decode_with_correction` signatures against the pinned `0.34.0` (docs.rs or `cargo` registry-src extraction) before locking the design. (The functions are foundational and almost certainly identical 0.34→0.35, but the discipline is verify-don't-assume.)

3. **Oracle-signature divergence is the real design driver.** The three codecs expose structurally different decode entry points (ms single-string · md single-string + chunked · mk slice-of-strings). This directly substantiates open-decision #3 (md1 chunked / mk1 long-codes / ms1 secret-bearing each differ) and open-decision #8 (chunked md1: which chunk is wrong-length, does the chunk header encode expected length?). Staging by HRP is therefore not just convenience — the per-HRP oracle dispatch is genuinely different code.

4. **SemVer is genuinely open (= open-decision #1, surface).** FOLLOWUP states "MINOR if new subcommand; PATCH if additive flag on `repair`." Recon cannot resolve; the brainstorm must pick the surface first, then the SemVer falls out. Lean: the ambiguity-output contract (#4) + ms1 secret-on-stdout advisory (#6) argue for a clean surface; a new subcommand (MINOR) is the tidier home, but a `--max-indel` flag on `repair` (PATCH) reuses `CardArgs` + exit-code conventions. Decide in brainstorm.

5. **CLAUDE.md doc staleness (non-blocking).** CLAUDE.md still says siblings are "git deps until they hit crates.io in lockstep with v0.1." They are **now crates.io registry deps** (ms 0.2.0 / mk 0.3.1 / md 0.34.0). Not a recon blocker; worth a one-line CLAUDE.md fix in some future cycle.

6. **Secret-handling lockstep risk (ms1).** ms1 is secret-bearing; candidate strings on stdout = the D9 secret-on-stdout advisory class (mirror `final_word` / `repair` / `seed-xor`). If the chosen surface adds a **new secret-carrying clap flag**, the GUI-schema `secret` projection (`secrets::flag_is_secret`) must update in lockstep (`feedback_gui_schema_secret_projection_lockstep`) — this is caught only at GUI-pin time and historically forced a patch (v0.33.1). Flag this in the brainstorm's lockstep checklist.

---

## Recommended brainstorm-session scope

- **Single-slug cycle.** This is one well-bounded item; per `feedback_smaller_cycle_scope_reduces_citation_surface` (1–3 items ship in ~1 architect round) it should converge fast — citation surface is tiny and already pristine.
- **Rough sizing: medium.** New toolkit-side enumerate-and-validate module (~150–300 LOC: deletion/insertion candidate generation + per-HRP oracle dispatch + ambiguity contract), CLI surface (flag-on-`repair` or new subcommand), TDD tests (canonical indel fixtures per HRP). Bulk of the work is the per-HRP oracle dispatch (#3) and the ambiguity/exit-code output contract (#4), not the combinatorics (off-by-1 is O(N) deletions / O(32·N) insertions — cheap; cap the budget for k≥2).
- **SemVer:** **MINOR** if new subcommand; **PATCH** if additive `--max-indel`/`--allow-length` flag on `repair`. Open-decision #1 decides; resolve surface first.
- **Mandatory locksteps (all conditional on the surface decision):**
  - ANY new clap flag/option/subcommand/dropdown-value NAME ⇒ **GUI `schema_mirror`** (`mnemonic-gui/src/schema/mnemonic.rs`) + **manual mirror** (`docs/manual/src/40-cli-reference/`) IN the same PR (paired-PR rule).
  - If a new flag carries a secret (ms1 path) ⇒ **GUI-schema `secret` projection** lockstep (`secrets::flag_is_secret`) — Cross-cutting #6.
  - **No sibling-codec FOLLOWUP companion expected** — the design is toolkit-only (wrap existing `decode`). File a companion ONLY if the brainstorm discovers a codec needs a new decode entry-point (e.g. a chunk-length introspection helper for md1 #8).
- **Sequencing within the cycle:** consider staging by HRP if combinatorics/secret-handling diverge (ms1 secret-bearing; md1 chunked; mk1 slice) — but that's a brainstorm call, not a recon mandate.
- **Pre-brainstorm carry-ins:** (a) re-verify md-codec `0.34.0` decode signatures (Cross-cutting #2); (b) treat bech32-upstream `Corrector` as unavailable — use vendored/toolkit enumeration (FOLLOWUP `bech32-upstream-corrector-migration`).

**Next gate after brainstorm/plan:** mandatory opus architect **R0 → 0C/0I before any code** (CLAUDE.md first Convention). cycle-prep is recon only; it stops here.
