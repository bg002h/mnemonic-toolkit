# CONTINUITY — m-format (m*1) incorrect-length recovery

**Written:** 2026-05-24, end of the v0.37.0 session (handoff for a fresh context).
**Repo state at handoff:** `master` = `7106125` (local; origin/master = `7106125` at handoff — this continuity commit will put local 1 ahead until the cycle ships). v0.37.0 SHIPPED + CI-green. Working tree clean except `.claude/`.

---

## The feature the user wants next

**Recover `m*1` strings (md1 / mk1 / ms1) that are of *incorrect length*** — i.e. a hand-copied or steel-engraved card where a character was **inserted (string too long)** or **dropped (string too short)**, so the bech32m string no longer decodes.

This is **distinct from the existing `mnemonic repair`**, which does BCH *substitution* correction at **fixed** length.

### Why it's a genuinely new capability (the key technical framing)
- `mnemonic repair` (shipped v0.22.0+) corrects symbol-VALUE errors via the sibling codecs' `decode_with_correction` (BCH error-correction). `crates/mnemonic-toolkit/src/repair.rs` picks the BCH code variant *from the input length* via `bch_code_for_length`; a wrong length therefore either selects the wrong code (garbage correction) or hard-errors as `RepairError::ReservedInvalidLength` (`repair.rs:406`) / `UnsupportedCodeVariant` (`repair.rs:414`).
- BCH codes correct substitutions at a fixed codeword length. An **insertion/deletion (indel)** shifts every subsequent symbol → the codeword structure breaks → BCH cannot recover it. So length-recovery needs a **different algorithm** layered *around* (not inside) the existing decode.
- `mnemonic inspect` reports `byte_length` (`cmd/inspect.rs:195`) but offers no length-recovery.

### Likely architecture (validate during brainstorm — NOT yet decided)
Toolkit-side **enumerate-and-validate**, reusing the codecs' existing decode as the oracle — analogous to `final_word.rs` (BIP-39 last-word completer enumerates the 2048 wordlist and validates each by checksum):
- **Too long by k:** try deleting each k-subset of positions → keep candidates that decode cleanly (bech32m checksum valid + codec decode Ok).
- **Too short by k:** try inserting each of the 32 bech32 charset symbols at each of the (N+1) positions (k nested) → keep those that decode.
- Validation oracle = bech32m checksum + the codec's `decode` (or `decode_with_correction` if combining with substitution). This is **toolkit-only** if it wraps the existing codec decode per candidate — likely **no sibling-codec changes needed** (cheapest path; confirm in brainstorm).
- Combinatorics are bounded for small k (off-by-1: O(N) deletions or O(32·N) insertions). Off-by-≥2 grows fast — cap the budget.

---

## Open design questions for the brainstorm (the real decisions)
1. **Surface:** a flag on `mnemonic repair` (e.g. `--allow-length` / `--max-indel N`) vs a new subcommand (e.g. `mnemonic recover-length`)? `repair` already owns m*1 correction + the `CardArgs` trait + exit-code conventions — a flag is the likely fit. **Any new flag/subcommand ⇒ GUI `schema_mirror` lockstep + manual mirror.**
2. **Indel direction & budget:** deletions only / insertions only / both? Default off-by-1; configurable budget? Refuse beyond a cap (combinatorial blowup + false-positive risk).
3. **Which HRPs:** md1, mk1, ms1 all, or stage? Each differs: md1 has **chunked** multi-string forms (chunk lengths are themselves structural); mk1 has **long codes**; ms1 is **secret-bearing**.
4. **Ambiguity handling:** if multiple candidate strings decode validly, emit ALL (like `final_word`) or refuse as ambiguous? bech32m checksum (~30 bits) makes false-positives rare but possible at short lengths / large budgets. Define the output contract (JSON shape, exit codes — mirror `repair`'s `0`/`5`/`2`).
5. **Combine with substitution?** A card could have BOTH wrong length AND a flipped symbol. Likely **scope to length-only first** (indel with otherwise-valid post-indel checksum), defer combined indel+BCH.
6. **Secret hygiene:** ms1 candidates on stdout = secret-on-stdout (the D9 advisory class; and `flag_is_secret` / argv-leak discipline). mk1/md1 are public. Mirror `repair`/`seed-xor` advisories.
7. **bech32 vs bech32m:** the m-format uses **bech32m** (BIP-350) checksum constant — confirm against the codecs' decode path; the charset is the 32-symbol bech32 ALPHABET (`repair.rs:28` imports `ALPHABET`).
8. **Chunked md1:** which chunk is wrong-length, and does the chunk header encode the expected length (letting you target the bad chunk)?

---

## Prior art / pointers (re-grep line numbers at cycle-prep time — they decay)
- `crates/mnemonic-toolkit/src/repair.rs` — BCH substitution correction; `RepairError` enum (`:388`), `bch_code_for_length`, `CardArgs` trait, `validate_flag_hrp`. The likely home.
- `crates/mnemonic-toolkit/src/cmd/repair.rs` + `cmd/inspect.rs` — CLI surface; `CardArgs` shared via `repair.rs` (per `cmd-repair-inspect-shared-card-arg-helpers`, resolved v0.25.0).
- `crates/mnemonic-toolkit/src/cmd/final_word.rs` — **the enumerate-candidates-validate-by-checksum pattern** (closest analogue).
- Sibling codecs' `decode_with_correction` (md-codec `chunk.rs`, ms-codec, mk-codec) — substitution oracle; `decode` = the clean-decode oracle.
- v0.22.0/v0.22.1 cycles (`mnemonic repair` + `inspect` + auto-fire + D19 Levenshtein suggestions) — see memory `project_v0_22_0_repair_shipped` / `project_v0_22_1_verify_bundle_auto_fire_shipped`.
- bech32 upstream correction primitive is STILL unavailable (v0.11.1; no `Corrector`) — FOLLOWUP `bech32-upstream-corrector-migration` documents this; do NOT plan on it. Use the codecs' vendored primitives / toolkit-side enumeration.

## FOLLOWUP filed
`m-format-incorrect-length-recovery` in `design/FOLLOWUPS.md` (Status: open) — the slug for cycle-prep.

---

## Project discipline reminders (carry into the new session)
- **Mandatory pre-impl R0 gate:** brainstorm-spec + plan-doc each pass an opus architect review to **0 Critical / 0 Important** BEFORE any code; fold → persist verbatim to `design/agent-reports/` → re-dispatch every round until GREEN. No code/phase-advance/tag past an open C/I.
- **BIN-target tests:** `repair`/`inspect`/`wallet_*`/`final_word` unit+integration tests run under `cargo test -p mnemonic-toolkit --bins` / `--test <name>`, NOT `--lib` (the modules are bin-private).
- **Lockstep:** any clap flag/value/subcommand NAME change ⇒ update `mnemonic-gui/src/schema/mnemonic.rs` (schema_mirror) + the manual under `docs/manual/src/40-cli-reference/` IN the same PR. Behavior-only (no flag-name change) ⇒ no GUI lockstep.
- **Phase-6 release-prep checklist:** Cargo.toml + Cargo.lock + BOTH README `<!-- toolkit-version: X -->` markers + `scripts/install.sh:32` self-pin + CHANGELOG + FOLLOWUP status flip. (memory `feedback_phase_6_release_prep_checklist_readme_markers`)
- **SemVer:** new top-level subcommand = MINOR; additive flag/value = PATCH; behavior change to an existing default = its own SemVer call.
- Stage paths explicitly (no `git add -A`); commit trailer `Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>`; clean working tree before the ship sequence.

## Kickoff (after `/clear`)
Issue: **`/cycle-prep m-format-incorrect-length-recovery`**
(cycle-prep reads this FOLLOWUP + this continuity doc, verifies citations against current origin/master, and produces the recon → then brainstorm → R0 → plan → R0 → phased TDD → ship.)
