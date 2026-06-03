# ms1-slot — End-of-Cycle R0 Review
**Verdict:** GREEN (0C/0I)

Full cycle diff `git diff 0814ab5..079d573` (subsumes the Phase-3 docs/release review). Last gate before merge→master + tag `mnemonic-toolkit-v0.41.0`.

## Critical (0) / Important (0) / Minor (2)

## 1. Integration correctness
Single shared decode/language helper — no 4-way drift. All three binding loops route decode + language policy through `slot_ms1::resolve_ms1_slot` exactly once: template `resolve_slots` Ms1 arm `bundle.rs:660-707` (before catch-all `:708`); `bundle_run_unified_descriptor` `bundle.rs:1471-1500` (5-tuple, every arm `, None` at `:1407/:1437/:1470`, push `language: emit_lang` `:1524`); `verify_bundle` descriptor loop `verify_bundle.rs:855-884` (5-tuple, annotation extended `:782-787`, push `:904`). verify-bundle template paths call the SAME `crate::cmd::bundle::resolve_slots` (`:363,:453,:557`) — site 10 shared, not duplicated.
Output-symmetry (load-bearing): all three set `ResolvedSlot.language = res.emit_language`; synth rule (`synthesize.rs:298-306`/`:835-847`) English→Entr/else→Mnem; `bip39_to_wire_code ∘ wire_code_to_bip39 == id` → mnem ms1 re-emits same wire code → byte-identical card → whole-card verify closes. Tested entr+mnem (`tests/cli_ms1_slot.rs:435-518`), decode-discriminated.
Byte-identity: entr-ms1 `emit_language=None` ≡ `@N.entropy=` (also `language: None` `:657`) across 5 lengths (`:137-181`).
Helper spot-check: language policy in bip39 space (`slot_ms1.rs:56-69`), `#[non_exhaustive]` `_` arm (`:79-81`), `wire_code_to_bip39` Err on >9 (no panic), `from_entropy_in` Err on bad length (no panic). mnem-English edge tested not fixed.

## 2. Release correctness
Version COMPLETE+consistent: `Cargo.toml:3=0.41.0`, both README markers `:13`/`:9`=0.41.0, `install.sh:32`=`mnemonic-toolkit-v0.41.0`, `Cargo.lock` mnemonic-toolkit 0.41.0. CHANGELOG (`:9-17`) accurate incl. the truthful `[Seedqr,Path]` exit-1→exit-2 normalization framing + GUI lockstep note. Manual `41-mnemonic.md` ms1 in BOTH bundle (`:67`) + verify-bundle (`:535`) `--slot` rows + clap doc (`bundle.rs:103-104`); accurate; `make audit` EXIT=0, zero transcripts re-captured. FOLLOWUP `verify-bundle-descriptor-entropy-slot-gap` (`FOLLOWUPS.md:49-56`) accurate (verify_bundle descriptor loop has no `@N.entropy=` arm, `:788-892`).

## 3. GUI strategy — RECOMMENDATION (a): leave as unmergeable prepared draft + FOLLOWUP; do NOT merge
GUI pins `mnemonic-toolkit-v0.37.3` (`mnemonic-gui/Cargo.toml:42`); `secrets.rs:34` re-exports `SECRET_SLOT_SUBKEYS` from that pin (5 entries, no ms1); the branch bumped the committed snapshot to 6 (`:67-68`); the compile-time `const _: () = assert!(secret_slice_eq(...))` (`:89-99`) → length mismatch → does NOT compile against the current pin. Merging would break GUI master. Redaction (`persistence.rs:91`) filters on the re-export, so `Ms1`-row redaction is effective ONLY once the pin ≥ v0.41.0 — the const-assert correctly PREVENTS shipping a non-redacting Ms1 picker. The picker+snapshot are correct as a prepared draft and must land WITH the pin bump. Toolkit tag v0.41.0 is INDEPENDENT (schema_mirror untouched — ms1 is a free-form `--slot` value; GUI is downstream/lagging-indicator). NOT merging the GUI branch is correct.

## 4. Secret / safety sweep
`Ms1.is_secret_bearing()==true` (`slot_input.rs:85`) → `@N.ms1=-` stdin sentinel (`:110`) + argv-leak advisory inherited; `SECRET_SLOT_SUBKEYS` has "ms1" (`secret_taxonomy.rs:111`, parity-test-enforced). `Ms1SlotResolution.entropy: Zeroizing<Vec<u8>>`, no Debug derive (conflict test matches on Err), `_entropy_pin` (mlock) on every push. No entropy in error text (`slot_ms1.rs:62-66`). No panic on attacker input (malformed/short ms1, share→IsShareNotSingleString→`ms-shares combine` prose exit 2, reserved tag, bad wire code, bad length — all typed Err).

## 5. Ship-readiness
None blocking. Comprehensive tests (canonical-gate exit-2 ms1+seedqr, byte-identity ×5, mnem-japanese, language-conflict bundle+verify, descriptor derive+round-trip entr+mnem, share rejection, mnem-English edge, `--self-check`) + unit + parity/legal-set. Phase-2 gate controller-confirmed (0 failed, clippy exit 0); Phase 3 docs+version only. Versions consistent, lockfile relocked. schema_mirror untouched; GUI downstream draft (not a blocker); manual+clap+CHANGELOG+FOLLOWUP accurate. Working tree: only untracked scratch (acceptable); all tracked release artifacts committed.

### Minor (non-blocking)
- **M1** — `is_legal_set` lists `[Ms1,Fingerprint,Path]` before `[Ms1,Path]` (`slot_input.rs:364-365`), inverting the descending-length convention. Functionally inert (`matches!` arm). Cosmetic.
- **M2** — `slot_input.rs:9` blanket `#![allow(dead_code)]` (pre-existing, not this cycle). Harmless.

## Verdict rationale
Whole-cycle integration correct + consistent across the three binding loops, all funneled through one secret-hygienic helper with correct wire-wins/refuse-on-conflict policy and a directly-tested load-bearing round-trip. Phase-3 release complete + consistent (4 version sites, relocked lock, accurate CHANGELOG/FOLLOWUP/manual). GUI branch is a correct-but-unmergeable prepared draft whose const-assert blocks a non-redacting picker; tag is independent. Secret/safety clean. No Critical/Important; 2 cosmetic Minors. **Cleared to merge→master (ff)→tag `mnemonic-toolkit-v0.41.0`; GUI branch stays unmerged behind its pin-bump FOLLOWUP.**
