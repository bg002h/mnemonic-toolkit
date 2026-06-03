# `bundle`/`verify-bundle --slot @N.ms1=` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Accept a raw `ms1` codex32 string as a first-class secret slot subkey (`--slot @N.ms1=`) on `bundle` + `verify-bundle`, decoded inline and routed through the existing entropy materialization, with wire-language authority + refuse-on-conflict.

**Architecture:** New `SlotSubkey::Ms1` (free-form parser, no clap value-enum → no schema_mirror change). A shared `slot_ms1::resolve_ms1_slot` helper does decode + language-policy and returns `{entropy, derive_language, emit_language}`. The helper is consumed by the THREE hand-rolled binding loops (template `resolve_slots`, `bundle_run_unified_descriptor`, `verify_bundle` descriptor loop); each derives via `derive_slot::derive_bip32_from_entropy[_at_path]` and sets `ResolvedSlot.language = emit_language` so the emitted ms1 card round-trips kind+language (load-bearing — verify-bundle compares whole card strings).

**Tech Stack:** Rust (edition 2021, toolchain 1.85). ms-codec 0.4.0 (`decode`, `Payload::{Entr,Mnem}`). bip39 2.x, bitcoin 0.32. Target **toolkit v0.41.0** (SemVer MINOR).

**Source of truth:** `design/SPEC_bundle_slot_ms1.md` (SPEC R0 GREEN at `7146249`). All citations re-grepped at R0/R1 against branch `bundle-slot-ms1-input`. **NOTE (R0-M1):** `bundle.rs`/`verify_bundle.rs`/`convert.rs` live under `crates/mnemonic-toolkit/src/cmd/`; `slot_input.rs`/`synthesize.rs`/`error.rs`/`language.rs`/`friendly.rs`/`secret_taxonomy.rs`/`derive_slot.rs` under `crates/mnemonic-toolkit/src/`. Re-grep all line numbers before editing (they are `7146249` snapshots).

**Gate per phase:** `cargo test -p mnemonic-toolkit --no-fail-fast` (0 fail) + `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` (clean). NO `cargo fmt` (toolkit has no fmt gate). Mandatory opus R0 per phase + end-of-cycle; re-dispatch after every fold; persist reviews to `design/agent-reports/` before fold-commit.

---

## Phase 1 — `SlotSubkey::Ms1` surface + validation + descriptor-gate widening

**Files:**
- Modify: `crates/mnemonic-toolkit/src/slot_input.rs` (enum, from_token, as_str, is_secret_bearing, error string, is_legal_set, exempted_v0_19_0, macro)
- Modify: `crates/mnemonic-toolkit/src/secret_taxonomy.rs:111`
- Modify: `crates/mnemonic-toolkit/src/cmd/bundle.rs` (canonical gate `:1151-1160`, default-path-override `:1222-1232`)
- Modify: `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (default-path-override `:715-723`)

### Task 1.1 — `SlotSubkey::Ms1` variant + token maps + secret-class

- [ ] **Step 1: Write failing tests** in `slot_input.rs` `#[cfg(test)] mod tests`:

```rust
#[test]
fn parse_happy_ms1() {
    assert_eq!(parse_slot_input("@0.ms1=ms1abc").unwrap(), slot(0, SlotSubkey::Ms1, "ms1abc"));
}
#[test]
fn ms1_is_secret_bearing_and_stdin_sentinel() {
    assert!(SlotSubkey::Ms1.is_secret_bearing());
    let p = parse_slot_input("@0.ms1=-").unwrap();
    assert!(p.is_stdin_sentinel(), "@0.ms1=- must be a stdin sentinel");
}
#[test]
fn ms1_token_round_trips() {
    assert_eq!(SlotSubkey::from_token("ms1"), Some(SlotSubkey::Ms1));
    assert_eq!(SlotSubkey::Ms1.as_str(), "ms1");
}
#[test]
fn unknown_subkey_error_lists_ms1() {
    let e = parse_slot_input("@0.bogus=x").unwrap_err();
    assert!(e.0.contains("ms1"), "expected-tokens list must include ms1");
}
```

- [ ] **Step 2: Run — verify they fail** (`Ms1` variant undefined / not in token map).
  Run: `cargo test -p mnemonic-toolkit --lib slot_input 2>&1 | tail -20`. Expected: compile error / FAIL.

- [ ] **Step 3: Implement.** In `slot_input.rs`:
  - Add `Ms1` to `enum SlotSubkey` **immediately after `Entropy`** (`:29`), keeping the doc-comment convention. Resulting Ord: `Phrase < Seedqr < Entropy < Ms1 < Xpub < MasterXpub < Fingerprint < Path < Wif < Xprv`.
  - `from_token` (`:45-58`): add `"ms1" => Self::Ms1,`.
  - `as_str` (`:59-71`): add `Self::Ms1 => "ms1",`.
  - `is_secret_bearing` (`:72-77`): add `| Self::Ms1` to the `matches!`.
  - unknown-subkey error string (`:160-165`): append `, ms1` to the "expected one of: …" list (place it after `entropy` to mirror the Ord).

- [ ] **Step 4: Run — verify pass.** Run: `cargo test -p mnemonic-toolkit --lib slot_input`. Expected: PASS.

- [ ] **Step 5: Commit** `git add crates/mnemonic-toolkit/src/slot_input.rs && git commit` — `feat(slot): add SlotSubkey::Ms1 variant + token maps (P1.1)`.

### Task 1.2 — parity-test macro + `SECRET_SLOT_SUBKEYS`

- [ ] **Step 1: Extend the macro + taxonomy (these ARE the tests).** In `slot_input.rs` `declare_slot_subkey_variants!(…)` (`:391-401`): add `Ms1,` to the list (or the `_exhaustiveness_check` match fails to compile — intended tripwire). In `secret_taxonomy.rs:111`: change `SECRET_SLOT_SUBKEYS` to `&["phrase", "seedqr", "entropy", "ms1", "xprv", "wif"]` (ms1 after entropy).
- [ ] **Step 2: Run** `cargo test -p mnemonic-toolkit --lib -- secret_taxonomy_parity_with_is_secret_bearing secret_taxonomy_entries_round_trip_via_from_token`. Expected: PASS (parity holds: `Ms1.is_secret_bearing()==true` ⟺ "ms1" ∈ SECRET_SLOT_SUBKEYS).
- [ ] **Step 3: Commit** both files — `feat(slot): SECRET_SLOT_SUBKEYS += ms1 + macro variant (P1.2)`.

### Task 1.3 — legal-set matrix + v0.19.0 exemption (full parity with phrase)

- [ ] **Step 1: Write failing tests** in `slot_input.rs` tests:

```rust
#[test]
fn validate_single_ms1_passes() { validate_slot_set(&[slot(0, SlotSubkey::Ms1, "x")]).unwrap(); }
#[test]
fn validate_ms1_plus_path_passes() {
    validate_slot_set(&[slot(0, SlotSubkey::Ms1, "x"), slot(0, SlotSubkey::Path, "48'/0'/0'/2'")]).unwrap();
}
#[test]
fn validate_ms1_plus_fingerprint_plus_path_passes() {
    validate_slot_set(&[slot(0, SlotSubkey::Ms1, "x"), slot(0, SlotSubkey::Fingerprint, "deadbeef"), slot(0, SlotSubkey::Path, "48'/0'/0'/2'")]).unwrap();
}
#[test]
fn validate_ms1_plus_xpub_conflict() {
    let e = validate_slot_set(&[slot(0, SlotSubkey::Ms1, "x"), slot(0, SlotSubkey::Xpub, "y")]).unwrap_err();
    matches!(e, ToolkitError::SlotInputViolation { kind, .. } if kind == "conflict");
}
```

- [ ] **Step 2: Run — verify fail** (Ms1+Path hits the secret+watch conflict before is_legal_set). Run: `cargo test -p mnemonic-toolkit --lib slot_input`.
- [ ] **Step 3: Implement.** In `slot_input.rs`:
  - `exempted_v0_19_0` (`:289-295`): add arms `[SlotSubkey::Ms1, SlotSubkey::Path] | [SlotSubkey::Ms1, SlotSubkey::Fingerprint, SlotSubkey::Path]`.
  - `is_legal_set` (`:330-352`): add `| [Ms1] | [Ms1, Fingerprint, Path] | [Ms1, Path]` (canonical sorted order: `Ms1 < Fingerprint < Path`; spell `[Ms1, Fingerprint, Path]` and `[Ms1, Path]` to match the `[Phrase, Fingerprint, Path]`/`[Phrase, Path]` arm spelling at `:347-348`).
- [ ] **Step 4: Run — verify pass.** `cargo test -p mnemonic-toolkit --lib slot_input`. Expected: PASS.
- [ ] **Step 5: Commit** — `feat(slot): ms1 legal-sets [Ms1]/[Ms1,Path]/[Ms1,Fp,Path] (P1.3)`.

### Task 1.4 — descriptor canonical-gate + default-path-override widening (fix-the-class: phrase|seedqr|ms1)

- [ ] **Step 1: Write failing integration tests** in `crates/mnemonic-toolkit/tests/cli_ms1_slot.rs` (new file). Use a CANONICAL multisig descriptor fixture (origin-annotated) and assert `[Ms1, Path]` + `[Seedqr, Path]` are refused with exit 2 + `kind:"conflict"`:

```rust
use assert_cmd::Command;
// Canonical descriptor + @0.ms1=<some ms1> + @0.path=... → exit 2, SlotInputViolation conflict.
#[test]
fn ms1_plus_path_canonical_descriptor_refused_exit2() { /* build cmd; assert .code(2) + stderr "conflict"/"pick one per slot" */ }
#[test]
fn seedqr_plus_path_canonical_descriptor_refused_exit2() {
    // R0-I2 baseline note: pre-fix this was exit-1 BadInput via binding-loop fall-through;
    // the widened gate normalizes it to exit-2 SlotInputViolation. Assert exit 2 now.
}
```

- [ ] **Step 2: Run — verify fail/wrong-code.** `cargo test -p mnemonic-toolkit --test cli_ms1_slot 2>&1 | tail`. (Ms1+Path: today the binding-loop `else→BadInput` exit 1, NOT the gate; Seedqr+Path: exit 1 today.)
- [ ] **Step 3: Implement.** In `cmd/bundle.rs`:
  - canonical gate (`:1151-1153`): add `let has_seedqr = subkeys.contains(&SlotSubkey::Seedqr); let has_ms1 = subkeys.contains(&SlotSubkey::Ms1);` and change the condition to `if (has_phrase || has_seedqr || has_ms1) && has_path {` (keep the existing `SlotInputViolation{kind:"conflict", message}` body verbatim).
  - default-path-override loop (`:1222-1232`): extend the `!Phrase && !Seedqr` continue-guard to also pass Ms1 (so an Ms1 explicit-origin slot is treated like Phrase/Seedqr for default-path purposes). Match the exact existing guard shape.
  In `cmd/verify_bundle.rs` default-path-override (`:715-723`): apply the same `!Phrase && !Seedqr` → include Ms1 widening.
- [ ] **Step 4: Run — verify pass.** `cargo test -p mnemonic-toolkit --test cli_ms1_slot`. Expected: both refused exit 2.
- [ ] **Step 5: Run full suite + clippy.** `cargo test -p mnemonic-toolkit --no-fail-fast 2>&1 | grep -cE '^test .* FAILED'` (expect 0) + `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`.
- [ ] **Step 6: Commit** — `fix(bundle): widen canonical-mode + default-path gates to phrase|seedqr|ms1 (P1.4)`.

### Phase 1 gate
- [ ] Full suite green, clippy clean. **Persist opus R0 review** to `design/agent-reports/ms1-slot-phase-1-R0-review.md` BEFORE fold; loop to 0C/0I (re-dispatch after each fold).

---

## Phase 2 — shared `slot_ms1` helper + three binding-loop Ms1 arms + language conflict

**Files:**
- Create: `crates/mnemonic-toolkit/src/slot_ms1.rs` (+ `mod slot_ms1;` in `lib.rs`/`main.rs` as the crate wires modules)
- Modify: `crates/mnemonic-toolkit/src/cmd/bundle.rs` (`resolve_slots` arm site 10; `bundle_run_unified_descriptor` arm site 13)
- Modify: `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (descriptor-loop arm site 15)
- Test: `crates/mnemonic-toolkit/tests/cli_ms1_slot.rs`

### Task 2.1 — the shared `slot_ms1::resolve_ms1_slot` helper

- [ ] **Step 1: Write failing unit tests** in `slot_ms1.rs` `#[cfg(test)]`:

```rust
// entr ms1 → entropy + English derive + emit None.
// mnem ms1 (non-English) + no --language → wire derive + emit Some(wire).
// mnem ms1 + --language == wire → ok.
// mnem ms1 + --language != wire → Err SlotInputViolation{kind:"language-conflict"}.
// a K-of-N share → Err (IsShareNotSingleString via ms_codec, mapped to ToolkitError).
```
Build fixtures with `ms_codec::encode(Tag::ENTR, &Payload::Entr(E))` and the mnem encode path (or hard-code known ms1 strings from the ms `mnem` cycle fixtures).

- [ ] **Step 2: Run — verify fail** (module/helper absent).
- [ ] **Step 3: Implement** `slot_ms1.rs`:

```rust
//! `--slot @N.ms1=` decode + language-resolution helper (SPEC §2/§3).
use zeroize::Zeroizing;
use crate::error::ToolkitError;
use crate::language::{CliLanguage, wire_code_to_bip39};

pub struct Ms1SlotResolution {
    pub entropy: Zeroizing<Vec<u8>>,
    pub derive_language: bip39::Language,
    pub emit_language: Option<bip39::Language>,
}

/// Decode an ms1 slot value → entropy + derive/emit languages.
/// `flag_language` is `None` iff `--language` was absent (so a Some/None
/// distinction is possible — `--language` has no clap default).
pub fn resolve_ms1_slot(
    value: &str,
    flag_language: Option<CliLanguage>,
    slot_index: u8,
) -> Result<Ms1SlotResolution, ToolkitError> {
    let (_tag, payload) = ms_codec::decode(value).map_err(ToolkitError::from)?;
    match payload {
        ms_codec::Payload::Entr(bytes) => Ok(Ms1SlotResolution {
            entropy: Zeroizing::new(bytes),
            derive_language: flag_language.unwrap_or_default().into(),
            emit_language: None,
        }),
        ms_codec::Payload::Mnem { language: wire, entropy } => {
            let wire_lang = wire_code_to_bip39(wire)?;
            if let Some(flag) = flag_language {
                let flag_lang: bip39::Language = flag.into();
                if flag_lang != wire_lang {
                    return Err(ToolkitError::SlotInputViolation {
                        kind: "language-conflict",
                        message: format!(
                            "slot @{slot_index}.ms1= carries wordlist language {:?} but --language {:?} was supplied; \
                             omit --language or set it to {:?}",
                            wire_lang, flag_lang, wire_lang
                        ),
                    });
                }
            }
            Ok(Ms1SlotResolution {
                entropy: Zeroizing::new(entropy),
                derive_language: wire_lang,
                emit_language: Some(wire_lang),
            })
        }
        // ms-codec Payload is #[non_exhaustive] (SPEC §0).
        _ => Err(ToolkitError::BadInput("ms1 slot decoded to an unknown payload kind".into())),
    }
}
```
Register the module (`mod slot_ms1;` where the crate declares modules; ms_codec error already maps via `ToolkitError::from` per `convert.rs:1464`).

- [ ] **Step 4: Run — verify pass.** `cargo test -p mnemonic-toolkit --lib slot_ms1`. Expected: PASS (incl. the share→Err and conflict→Err cases).
- [ ] **Step 5: Commit** — `feat(slot): slot_ms1::resolve_ms1_slot decode+language helper (P2.1)`.

### Task 2.2 — template `resolve_slots` Ms1 arm (site 10; shared by bundle + verify-bundle)

- [ ] **Step 1: Write failing integration tests** (`tests/cli_ms1_slot.rs`):

```rust
// (a) byte-identity: bundle --template wpkh --slot @0.ms1=<entr-ms1 of E> == bundle --slot @0.entropy=<hex E> (stdout identical), across 16/20/24/28/32.
// (b) mnem ms1 (japanese) → bundle derives the same xpub as @0.phrase=<japanese phrase> --language japanese, AND emits a mnem ms1 card.
// (c) mnem ms1 + --language english → exit 2 language-conflict.
```

- [ ] **Step 2: Run — verify fail** (Ms1 falls to the `resolve_slots` catch-all `else→BadInput` at `:709-714`).
- [ ] **Step 3: Implement** the Ms1 arm in `cmd/bundle.rs::resolve_slots` — insert a new `else if subkeys.contains(&SlotSubkey::Ms1) { … }` arm BEFORE the catch-all (`:709`), modeled on the `Entropy` arm (`:606-655`):
  - find the `@N.ms1=` value; `let res = crate::slot_ms1::resolve_ms1_slot(value, language, idx)?;`
  - derive via the SAME `multisig_acct_path` branch the Entropy arm uses, calling `derive_slot::derive_bip32_from_entropy_at_path(&res.entropy, pass, res.derive_language, network, p)` / `derive_bip32_from_entropy(&res.entropy, pass, res.derive_language, network, template, account)`.
  - `let (_acc_entropy, fingerprint, xpub, _xpriv, path) = acc.into_parts();` push `ResolvedSlot { xpub, fingerprint, path, entropy: Some(res.entropy), master_xpub: None, language: res.emit_language, _entropy_pin: Some(Rc::new(pin_pages_for(&res.entropy[..]))) }`.
- [ ] **Step 4: Run — verify pass.** `cargo test -p mnemonic-toolkit --test cli_ms1_slot`. Expected: byte-identity (a), mnem-derivation+card (b), conflict (c) all PASS.
- [ ] **Step 5: Commit** — `feat(bundle): resolve_slots Ms1 arm — decode+derive+emit-language (P2.2)`.

### Task 2.3 — `bundle_run_unified_descriptor` Ms1 arm (site 13)

- [ ] **Step 1: Failing test** — canonical-key descriptor with a concrete cosigner replaced by `@0` placeholder + `--slot @0.ms1=<entr-ms1>` (non-canonical/explicit-origin descriptor form) derives the cosigner xpub; mnem ms1 emits a mnem card. (Mirror an existing `bundle --descriptor` phrase test, swapping `@0.phrase=` → `@0.ms1=`.)
- [ ] **Step 2: Run — verify fail** (descriptor loop `else→BadInput` `:1408`).
- [ ] **Step 3: Implement** the Ms1 arm in `bundle_run_unified_descriptor`'s binding loop (`:1305-1408`), before the `else→BadInput`. Decode via `slot_ms1::resolve_ms1_slot(value, args.language, idx)?`; derive via `derive_slot::derive_bip32_from_entropy_at_path(&res.entropy, pass, res.derive_language, network, &anno_path)` (anno_path in scope `:1294-1301`); push `CosignerKeyInfo{…, language: res.emit_language}` at the cosigner push (`:1422-1430`, currently `language: None`).
- [ ] **Step 4: Run — verify pass.** `cargo test -p mnemonic-toolkit --test cli_ms1_slot`.
- [ ] **Step 5: Commit** — `feat(bundle): descriptor-mode Ms1 arm in bundle_run_unified_descriptor (P2.3)`.

### Task 2.4 — `verify_bundle` descriptor-loop Ms1 arm (site 15) + verify-bundle round-trip

- [ ] **Step 1: Failing tests** (`tests/cli_ms1_slot.rs`):

```rust
// (a) verify-bundle round-trip: bundle emits ms1 card(s); feed each back via verify-bundle --slot @N.ms1=<that card> → VERIFIED (exit 0). Cover entr (English) AND mnem (japanese).
// (b) mnem ms1 + --language conflict in verify-bundle → exit 2 language-conflict (symmetry with bundle).
// (c) descriptor-mode verify-bundle with @N.ms1= cosigner → VERIFIED.
```

- [ ] **Step 2: Run — verify fail** (verify_bundle descriptor loop `else→DescriptorReparseFailed` `:849`; template path already covered by site 10 but verify-bundle round-trip exercises the whole-card compare `:1245/:1639`).
- [ ] **Step 3: Implement** the Ms1 arm in `cmd/verify_bundle.rs`'s descriptor binding loop (`:776-849`), before `else→DescriptorReparseFailed`. **(R0-I1: this loop has NO Entropy arm — do NOT mirror one.)** Decode via `slot_ms1::resolve_ms1_slot(value, args.language, idx)?`; derive the comparison xpub via `derive_slot::derive_bip32_from_entropy_at_path(&res.entropy, pass, res.derive_language, network, &anno_path)` (anno_path bound `:766-774`) → `into_parts()` → xpub; push with `language: res.emit_language` (`:859-865`, currently `language: None`).
- [ ] **Step 4: Run — verify pass.** `cargo test -p mnemonic-toolkit --test cli_ms1_slot`. Expected: round-trip VERIFIED (entr + mnem), conflict exit 2, descriptor verify VERIFIED.
- [ ] **Step 5: Commit** — `feat(verify-bundle): descriptor-loop Ms1 arm + round-trip (P2.4)`.

### Task 2.5 — share-rejection + mnem-English edge + `--self-check` tests

- [ ] **Step 1: Write tests** (`tests/cli_ms1_slot.rs`):

```rust
// (a) @N.ms1=<a K-of-N share> → exit 2 + friendly "ms-shares combine" prose (friendly.rs:110-114).
// (b) mnem-English edge: a Mnem{language:0} ms1 → bundle emits an ENTR card (documented, SPEC §3/§9). Assert the emitted card is the entr form, not mnem.
// (c) --self-check with a mnem ms1 slot round-trips (exit 0).
```
For (b), construct a `Mnem{language:0}` ms1 directly via `ms_codec::encode`/`Payload::Mnem{language:0,…}` if constructible (or document if the encoder refuses it — then assert decode-side behavior only).

- [ ] **Step 2: Run — verify fail/confirm.** `cargo test -p mnemonic-toolkit --test cli_ms1_slot`.
- [ ] **Step 3: Implement** — no new code expected (these exercise already-built paths); if (b) reveals the emitted card differs from the documented edge, reconcile the SPEC §3 edge note + the arm. (`friendly.rs` IsShareNotSingleString prose is already present `:110-114`.)
- [ ] **Step 4: Run — verify pass + full suite + clippy.** `cargo test -p mnemonic-toolkit --no-fail-fast` + clippy.
- [ ] **Step 5: Commit** — `test(ms1-slot): share-rejection + mnem-English edge + self-check (P2.5)`.

### Phase 2 gate
- [ ] Full suite green, clippy clean. **Persist opus R0** to `design/agent-reports/ms1-slot-phase-2-R0-review.md` BEFORE fold; exercise verify-bundle round-trip + `--self-check` in the R0 (`feedback_verify_bundle_round_trip_per_phase_r0_scope`, `feedback_self_check_bypasses_csi_grouping`); loop to 0C/0I.

---

## Phase 3 — docs / GUI lockstep + release-prep (v0.41.0)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/bundle.rs:94-113` (`--slot` doc-comment) + optional `src/error.rs:285` doc-comment (R0-M2)
- Modify: `docs/manual/src/40-cli-reference/41-mnemonic.md`
- Modify (paired GUI PR): `mnemonic-gui` `src/form/slot_editor.rs` + `src/secrets.rs`
- Modify: `crates/mnemonic-toolkit/Cargo.toml`, `Cargo.lock`, `README.md`, `crates/mnemonic-toolkit/README.md`, `CHANGELOG.md`, `scripts/install.sh:32`
- New: `design/FOLLOWUPS.md` entry `verify-bundle-descriptor-entropy-slot-gap`

### Task 3.1 — clap doc-comment + optional error doc-comment

- [ ] **Step 1:** Add the `ms1   BIP-93 codex32 secret (entropy or mnemonic; language-preserving)` line to the `--slot` `verbatim_doc_comment` subkey list (`bundle.rs:94-113`, after the `entropy` line). verify-bundle defers to BundleArgs (`verify_bundle.rs:117-118`) — no edit there.
- [ ] **Step 2 (optional, R0-M2):** append `| "language-conflict"` to the `SlotInputViolation.kind` doc-comment enumeration at `error.rs:285`.
- [ ] **Step 3: Commit** — `docs(bundle): document ms1 slot subkey in --slot help (P3.1)`.

### Task 3.2 — manual (`41-mnemonic.md`) + audit

- [ ] **Step 1:** Document the `ms1` slot subkey in the bundle/verify-bundle `--slot` prose in `docs/manual/src/40-cli-reference/41-mnemonic.md` (mirror the `seedqr`/`entropy` entries; note language preservation + the share-rejection pointer).
- [ ] **Step 2:** Build the 4 CLIs (mnemonic=this branch; ms/md/mk per `docs/manual/Makefile` BIN vars) and run `make -C docs/manual audit MNEMONIC_BIN=… MD_BIN=… MS_BIN=… MK_BIN=… ; echo "EXIT=$?"` — capture the literal EXIT (do NOT pipe to tail). Fix any anchor/flag-coverage failure. Re-capture transcripts ONLY if provably correct.
- [ ] **Step 3: Commit** — `docs(manual): document --slot @N.ms1= (P3.2)`.

### Task 3.3 — paired mnemonic-gui PR (branch `bundle-slot-ms1-gui`)

- [ ] **Step 1:** On `/scratch/code/shibboleth/mnemonic-gui` branch `bundle-slot-ms1-gui`: add `Ms1` to `src/form/slot_editor.rs::SlotSubkey` (the slot-row picker) and `"ms1"` to `src/secrets.rs` `SECRET_SLOT_SUBKEYS` snapshot (drives `persistence.rs:91` redaction). Confirm whether a GUI drift test consumes the toolkit const (run the GUI suite if it builds against the local toolkit). NO version bump, NO tag.
- [ ] **Step 2: Commit on the GUI branch** — `schema: add ms1 slot subkey (picker + secret snapshot) for toolkit v0.41.0`.

### Task 3.4 — FOLLOWUP + version bump v0.40.0 → v0.41.0

- [ ] **Step 1:** File `design/FOLLOWUPS.md` entry `verify-bundle-descriptor-entropy-slot-gap` (pre-existing: `verify_bundle` descriptor loop has no `@N.entropy=` arm; out of scope — R0-I1 note).
- [ ] **Step 2:** Bump `crates/mnemonic-toolkit/Cargo.toml:3` → `0.41.0`; both README `<!-- toolkit-version: -->` markers; `CHANGELOG.md` (new `--slot @N.ms1=` entry + the `[Seedqr, Path]` canonical-mode exit-1→exit-2 normalization note); `scripts/install.sh:32` self-pin → `mnemonic-toolkit-v0.41.0`; `cargo build` to relock + stage `Cargo.lock`; run `cargo test -p mnemonic-toolkit --test readme_version_current`.
- [ ] **Step 3: Commit** — `release(toolkit): v0.41.0 — --slot @N.ms1= (P3.4)`.

### Phase 3 gate
- [ ] `make audit` EXIT=0; full suite green; clippy clean; readme_version_current PASS. **Persist opus R0** to `design/agent-reports/ms1-slot-phase-3-R0-review.md`; loop to 0C/0I.

---

## End-of-cycle + ship (authorization-gated)

- [ ] **End-of-cycle opus R0** across the toolkit diff + the GUI diff → `design/agent-reports/ms1-slot-end-of-cycle-R0-review.md`; loop to 0C/0I.
- [ ] **Ship** (tag-only; toolkit not on crates.io): commit design audit trail → merge `bundle-slot-ms1-input` → master (ff) → tag `mnemonic-toolkit-v0.41.0` → push master + tag → merge `bundle-slot-ms1-gui` → GUI master (no tag) → flip SPEC/FOLLOWUP statuses → update CONTINUITY.md + save a memory record.

---

## Self-review (spec coverage)

- SPEC §1 surface → Tasks 1.1, 1.2. §2 helper → 2.1. §3 language policy → 2.1 (helper) + 2.2/2.4 (conflict tests). §4 legal-sets + gates → 1.3, 1.4. §5 16-site map → 1.1-1.4 (sites 1-9,11,12,14) + 2.2/2.3/2.4 (sites 10,13,15) + 3.1 (site 16). §6 lockstep → 1.2 (parity), 3.1-3.4. §7 phasing → P1/P2/P3. §8 tests 1-11 → distributed across 1.1/1.3/1.4/2.1-2.5. §9 footguns → 2.5 (mnem-English), 1.4 (Seedqr baseline), 3.4 (FOLLOWUP). §10 citations → re-grep at impl per each task.
- No placeholders: all code/test steps carry actual code or exact commands. Type consistency: `resolve_ms1_slot` returns `Ms1SlotResolution{entropy: Zeroizing<Vec<u8>>, derive_language: bip39::Language, emit_language: Option<bip39::Language>}` consumed identically at sites 10/13/15; `ResolvedSlot.language: Option<bip39::Language>` (= `emit_language`). `SlotInputViolation{kind:"language-conflict"}` exit 2, no error.rs variant added.
