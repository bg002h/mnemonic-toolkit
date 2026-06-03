# SPEC — `bundle`/`verify-bundle --slot @N.ms1=<ms1-string>`

**Status:** draft (pre-R0). **Target:** mnemonic-toolkit **v0.41.0** (SemVer MINOR).
**Source SHA (re-grep at impl time):** toolkit `0814ab5` + the `bundle-slot-ms1-input` branch doc-fix tip; ms-codec **0.4.0** (crates.io; local `mnemonic-secret` master `7b9d901` == published source).
**Provenance:** recon `cycle-prep-recon-bundle-slot-ms1-input.md`; pre-SPEC architect design review `design/agent-reports/ms1-slot-pre-spec-design-review.md` (SOUND-WITH-CHANGES — all C/I/M findings folded below). User decisions: (1) wire language authoritative, **refuse on conflict**; (2) **full parity** with phrase legal-sets; (3) **MINOR** bump.

---

## §0. Pinned upstream API (ms-codec 0.4.0 — read from published source, not memory)

`payload.rs:28-57` — `#[non_exhaustive] pub enum Payload`:
- `Entr(Vec<u8>)` — raw BIP-39 entropy, length ∈ {16,20,24,28,32}. No language.
- `Mnem { language: u8, entropy: Vec<u8> }` — `language` is a wire code 0..=9 (0=English … 9=Portuguese, indexing `consts::MNEM_LANGUAGE_NAMES`); `entropy` length ∈ {16,20,24,28,32}.
- `#[non_exhaustive]` ⇒ every `match payload` MUST carry a `_ =>` arm.

`decode.rs:42` — `pub fn decode(s: &str) -> Result<(Tag, Payload)>`. A threshold≠0 K-of-N **share** decodes to `Error::IsShareNotSingleString` (a decode-time *error*, not a Payload variant); reserved tags (`seed`/`xprv`/`prvk`) are likewise rejected by `decode`. So the slot helper only ever sees `Entr`/`Mnem`/`_` on the Ok path.

---

## §1. Surface — `SlotSubkey::Ms1` (no new clap flag, no schema_mirror change)

`--slot`'s clap value is a free-form `String` parsed by `slot_input::parse_slot_input` (`bundle.rs:114-120`, `verify_bundle.rs:117-120`); subkey tokens are validated inside the parser via `from_token`, NOT as a clap `PossibleValuesParser`. Adding `ms1` is therefore **not** a new clap flag-name and **not** a clap value-enum → `schema_mirror` (flag-name + dropdown-value parity) is **untouched**. Confirmed: `gui_schema.rs` projects dropdowns only for true value-enums (`--template`), never the slot subkey set.

Edits in `slot_input.rs`:
1. `enum SlotSubkey` (`:16-42`) — add `Ms1` **immediately after `Entropy`** (`:29`). Resulting `Ord`: `Phrase < Seedqr < Entropy < Ms1 < Xpub < MasterXpub < Fingerprint < Path < Wif < Xprv`.
2. `from_token` (`:45-58`) — `"ms1" => Self::Ms1`.
3. `as_str` (`:59-71`) — `Self::Ms1 => "ms1"`.
4. `is_secret_bearing` (`:72-77`) — add `| Self::Ms1` (true). This is what makes the **stdin sentinel** (`@N.ms1=-`, `is_stdin_sentinel` gates on `is_secret_bearing`, `:100`) and the **argv-leak advisory** inherit for free.
5. unknown-subkey error string (`:160-165`) — append `ms1` to the "expected one of: …" list.
6. `declare_slot_subkey_variants!` macro test list (`:391-401`) — add `Ms1` (or the `_exhaustiveness_check` match fails to compile — intended tripwire).

---

## §2. Shared decode + language-resolution helper (avoids 4-way drift — design-review I2)

The Ms1 decode+language logic appears in **four** places (template `resolve_slots`, `bundle_run_unified_descriptor`, `verify_bundle` descriptor loop, and the already-shipped `convert`). Per `feedback_fix_the_class_hunt_for_second_instance`, factor it into ONE helper rather than hand-inlining the conflict check.

New module `crates/mnemonic-toolkit/src/slot_ms1.rs`:

```rust
pub struct Ms1SlotResolution {
    pub entropy: zeroize::Zeroizing<Vec<u8>>,
    /// Language to DERIVE the seed with (entropy→phrase→PBKDF2 seed).
    pub derive_language: bip39::Language,
    /// Language to stamp on the EMITTED card (drives entr-vs-mnem at synth);
    /// None ⇒ entr card (English), Some(wire) ⇒ mnem card. Feeds ResolvedSlot.language.
    pub emit_language: Option<bip39::Language>,
}

/// Decode an `ms1` slot value into entropy + the derive/emit languages,
/// applying the wire-wins-refuse-on-conflict policy (§3).
pub fn resolve_ms1_slot(
    value: &str,
    flag_language: Option<crate::language::CliLanguage>, // None ⟺ --language absent
    slot_index: u8,
) -> Result<Ms1SlotResolution, crate::error::ToolkitError>;
```

Behavior (modeled on the **decode/payload-match** of `convert.rs:1463-1486`, NOT its language policy — design-review C2):
- `ms_codec::decode(value)?` → `(_, payload)` (errors map via `ToolkitError::from` → friendly prose, incl. the `IsShareNotSingleString` share-rejection at `friendly.rs:110-114`, exit 2).
- `Payload::Entr(bytes)` → `{ entropy: Zeroizing::new(bytes), derive_language: flag_language.unwrap_or_default().into(), emit_language: None }`. (No intrinsic language; matches `@N.entropy=` exactly → byte-identity, §3.)
- `Payload::Mnem { language: wire, entropy }` → `let wire_lang = crate::language::wire_code_to_bip39(wire)?;` **conflict gate:** `if let Some(flag)=flag_language { if Into::<bip39::Language>::into(flag) != wire_lang { return Err(language_conflict(slot_index, wire_lang, flag)); } }` → `{ entropy: Zeroizing::new(entropy), derive_language: wire_lang, emit_language: Some(wire_lang) }`.
- `_ =>` → `ToolkitError::BadInput("ms1 slot decoded to an unknown payload kind".into())` (mirrors `convert.rs:1483`; required by `#[non_exhaustive]`).

The helper is context-free (no template/account/path). Each of the three binding-loop call sites does its OWN derivation by calling `derive_slot::derive_bip32_from_entropy[_at_path](&entropy, pass, derive_language, network, …)` — the `pub(crate)` helper the template `Entropy` arm already uses (`bundle.rs:621-637`) — then sets `ResolvedSlot.language = emit_language`. **Do NOT phrase this as "mirror the Entropy arm" (R0-I1):** `verify_bundle`'s descriptor binding loop (site 15) has NO Entropy arm to mirror — it must call the same `derive_slot` helper directly. `derive_slot::derive_bip32_from_entropy[_at_path]` confirmed signature `(entropy: &[u8], passphrase: &str, language: bip39::Language, network: CliNetwork, {template+account | path: &DerivationPath})` (`derive_slot.rs:42-71`, `pub(crate)`).

---

## §3. Language policy — wire wins, refuse on conflict (decision 3)

- **entr ms1:** no language → derive with `flag_language.unwrap_or_default()` (English default) — identical to the `Entropy` arm (`bundle.rs:621-622`). `emit_language=None` → emitted card is `entr`. **Byte-identity:** `@N.ms1=<entr-ms1 of E>` ≡ `@N.entropy=<hex E>` in xpub AND emitted card, across all five lengths (design-review D/M4).
- **mnem ms1 + `--language` absent:** derive with wire language; `emit_language=Some(wire)` → emitted card is `mnem` preserving the language.
- **mnem ms1 + `--language` == wire:** fine (redundant); same as above.
- **mnem ms1 + `--language` ≠ wire:** **HARD REFUSE.** Use `ToolkitError::SlotInputViolation { kind: "language-conflict", message }` → **exit 2** (design-review I5: reuses the existing FormatViolation-class variant — already exits 2 (`error.rs:519`), already carries a `kind` JSON discriminant, NO `error.rs` edit; precedent = `IsShareNotSingleString`→2, path-mismatch refusal `bundle.rs:1244`→2). Message names both languages + the slot index and tells the user to drop `--language` or set it to match. Comparison is in `bip39::Language` space (`flag.into()` vs `wire_code_to_bip39(wire)`).

**Output symmetry is LOAD-BEARING, not optional (design-review C1).** verify-bundle compares the **whole emitted card string** (`verify_bundle.rs:1245` single-sig, `:1639` multisig), not entropy. So feeding the engraved card back (`--slot @N.ms1=<that card>`) verifies ONLY because the slot arm sets `ResolvedSlot.language` so the re-emitted card matches. The synth emit rule already honors it for free: `synthesize_unified` (`synthesize.rs:831-839`) and `synthesize_descriptor` (`:299-306`) both compute `slot_lang = s.language.unwrap_or(run_language); if slot_lang == English → Payload::Entr else → Payload::Mnem{ language: bip39_to_wire_code(slot_lang), … }`. `ResolvedSlot.language` is `Option<bip39::Language>` (`synthesize.rs:671`).

**Documented edge — mnem-English (design-review A):** a `Mnem { language: 0 (English) }` ms1 resolves to `emit_language=Some(English)`, and the emit rule collapses English→`Entr`, so it round-trips as an `entr` card (not byte-identical to a mnem-English INPUT card). This is acceptable and documented: English self-recovers, `entr` is the canonical English form, and the ms encoder **never emits** mnem-English (English always routes to entr — ms `mnem` cycle), so a mnem-English ms1 is only third-party-constructible. SPEC includes a test asserting this documented behavior, not a fix.

---

## §4. Validation / legal sets — full parity with phrase (decision 4)

`slot_input.rs`:
- `is_legal_set` (`:330-352`) — add `[Ms1]`, `[Ms1, Path]`, `[Ms1, Fingerprint, Path]` (canonical sorted order: `Ms1 < Fingerprint < Path`, matching the `[Phrase, Fingerprint, Path]` arm spelling).
- `exempted_v0_19_0` matrix (`:289-295`) — add `[SlotSubkey::Ms1, SlotSubkey::Path]` and `[SlotSubkey::Ms1, SlotSubkey::Fingerprint, SlotSubkey::Path]` (else the secret+watch-only conflict refusal `:297` fires before `is_legal_set`). **(Recon missed this as a distinct site — design-review I4.)**

Descriptor-mode canonical rejection + path-override (these do NOT route through `resolve_slots`):
- canonical-mode rejection gate (`bundle.rs:1151-1160`; raises `SlotInputViolation{kind:"conflict"}` → exit 2) — widen `has_phrase && has_path` to `(has_phrase || has_seedqr || has_ms1) && has_path` (add `has_seedqr`/`has_ms1` locals). **For Ms1 this is LOAD-BEARING:** without it, `[Ms1, Path]` against a canonical descriptor would pass the gate and — once the site-13 Ms1 binding arm exists — be silently accepted with the explicit `@N.path=` overriding the descriptor's encoded origin (a real mis-acceptance). **For Seedqr this is an error-class NORMALIZATION, not a new rejection (R0-I2):** `[Seedqr, Path]` is ALREADY rejected today — it passes this gate (`has_phrase=false`) but then hits the binding-loop `else→BadInput` (exit 1); the widened gate moves that rejection up to exit-2 `SlotInputViolation{kind:"conflict"}`, matching `[Phrase, Path]`. CHANGELOG + test 9 state this truthfully.
- default-path-override loop (`bundle.rs:1222-1232`) — extend the `!Phrase && !Seedqr` continue-guard to also pass `Ms1`.

---

## §5. Site enumeration (all sites — descriptor mode is THREE hand-rolled binding loops, design-review B)

Verified against the branch tip:

| # | Site | File:line | Edit |
|---|---|---|---|
| 1-6 | SlotSubkey surface | `slot_input.rs:29,45-77,160-165,391-401` | §1 |
| 7 | `is_legal_set` | `slot_input.rs:330-352` | §4 |
| 8 | `exempted_v0_19_0` | `slot_input.rs:289-295` | §4 |
| 9 | `SECRET_SLOT_SUBKEYS += "ms1"` | `secret_taxonomy.rs:111` | §6 (HARD parity gate) |
| 10 | **template** `resolve_slots` Ms1 arm (shared by bundle+verify) | `bundle.rs:486-657` (catch-all `:711`) | helper §2 + derive like Entropy arm + multisig_acct_path branch + `language=emit_language` |
| 11 | descriptor canonical-mode gate | `bundle.rs:1151-1153` | §4 |
| 12 | descriptor default-path-override | `bundle.rs:1222-1232` | §4 |
| 13 | **`bundle_run_unified_descriptor`** Ms1 arm | `bundle.rs:1305-1430` (else→BadInput `:1408`, push `:1422-1430`) | helper §2 decode; derive via `derive_slot::derive_bip32_from_entropy_at_path`; push `language: emit_language` |
| 14 | verify-bundle default-path-override | `verify_bundle.rs:715-723` | §4 |
| 15 | **`verify_bundle`** descriptor-loop Ms1 arm (NO Entropy arm to mirror — R0-I1) | `verify_bundle.rs:776-867` (else→`DescriptorReparseFailed` `:849`, push `:859-865`) | helper §2 decode; derive via `derive_slot::derive_bip32_from_entropy_at_path(&entropy, pass, derive_language, network, &anno_path)`; push `language: emit_language` |
| 16 | `--slot` clap doc-comment | `bundle.rs:94-113` (verify-bundle shares BundleArgs doc) | add `ms1` line |

verify-bundle template-mode `resolve_slots` calls (`:363,:453,:557`) need NO Ms1-specific edit — they inherit site 10. No new `error.rs` variant (I5 reuses `SlotInputViolation`).

---

## §6. Lockstep + SemVer (MINOR → v0.41.0)

- **HARD same-PR gate:** `secret_taxonomy::SECRET_SLOT_SUBKEYS += "ms1"` (`secret_taxonomy.rs:111`) — the `secret_taxonomy_parity_with_is_secret_bearing` test (`slot_input.rs:404`) fails otherwise.
- **`schema_mirror`: NO change** (free-form value; confirmed §1).
- **Paired `mnemonic-gui` PR** (`feedback_gui_schema_secret_projection_lockstep`, `feedback_manual_gui_lockstep`): `src/form/slot_editor.rs::SlotSubkey` picker option + `src/secrets.rs` `SECRET_SLOT_SUBKEYS` snapshot += `"ms1"` (drives slot-value redaction in `persistence.rs:91`). At impl, confirm whether a GUI drift test consumes the toolkit const (auto-gate) or is hand-maintained (discipline-only).
- **Manual:** `docs/manual/src/40-cli-reference/41-mnemonic.md` — document the `ms1` slot subkey (prose; the flag-coverage lint gates long *flags*, not subkey tokens, so this is quality not hard-gate, but DO it + `make -C docs/manual audit`).
- **No sibling-codec change** (consumes published ms-codec 0.4.0; no companion FOLLOWUP).
- **Release-prep (per the Phase-6 checklist):** `Cargo.toml` 0.40.0→0.41.0 + both README `toolkit-version:` markers + CHANGELOG (incl. the `[Seedqr, Path]` canonical-mode normalization: exit-1 BadInput → exit-2 SlotInputViolation, fix-the-class alongside the new `[Ms1, Path]` rejection) + `install.sh:32` self-pin + `Cargo.lock` relock + `readme_version_current` test.

---

## §7. Phasing (mandatory opus R0 on SPEC + each phase + end-of-cycle; 0C/0I before code; re-dispatch after every fold)

- **P1 — surface + validation** (`slot_input.rs` sites 1-8 + `secret_taxonomy.rs` site 9 + the descriptor gate/override sites 11,12,14 + the latent-Seedqr fix). TDD: parse, legal-set/exemption matrix, canonical-gate (phrase/seedqr/ms1 × path), parity test, stdin sentinel.
- **P2 — decode + materialization** (the `slot_ms1` helper §2 + the THREE binding-loop arms sites 10,13,15 + language-conflict §3). TDD: entr-ms1 byte-identity (5 lengths), mnem-ms1 correct-language seed, language-conflict refuse in BOTH bundle & verify-bundle, share-rejection prose, verify-bundle round-trip (engrave→feed-back), mnem-English documented edge, descriptor-mode multisig derivation, `--self-check` with a mnem Ms1 slot.
- **P3 — docs / GUI lockstep + release-prep** (site 16 + manual + paired GUI PR + version bump §6).

P2 is severable from P1 only in review order, not in ship (all one MINOR). verify-bundle round-trip + `--self-check` are per-phase R0 scope (`feedback_verify_bundle_round_trip_per_phase_r0_scope`, `feedback_self_check_bypasses_csi_grouping`).

---

## §8. Tests (the design-review-mandated set)

1. `parse_slot_input("@0.ms1=ms1…")` → `SlotSubkey::Ms1`; `@0.ms1=-` is a stdin sentinel.
2. `validate_slot_set`: `[Ms1]`, `[Ms1, Path]`, `[Ms1, Fingerprint, Path]` pass; `[Ms1, Xpub]` conflict; `[Ms1, Entropy]` invalid-set.
3. **entr-ms1 ≡ entropy byte-identity** across {16,20,24,28,32} (xpub + emitted card).
4. **mnem-ms1** (non-English) → correct seed/xpub (cross-check vs the equivalent phrase-in-that-language) + emits a `mnem` card.
5. **language-conflict refuse** (mnem-ms1 + disagreeing `--language`) → exit 2 + `kind:"language-conflict"`, in BOTH bundle and verify-bundle.
6. **share rejection** (`@N.ms1=<a K-of-N share>`) → exit 2 + the `ms-shares combine` friendly prose.
7. **verify-bundle round-trip** — engrave bundle → feed its own ms1 card(s) back on `@N.ms1=` → VERIFIED (entr and mnem cases).
8. **mnem-English documented edge** — `Mnem{language:0}` ms1 → emits `entr` card (asserted, not a bug).
9. **`[Ms1, Path]` AND `[Seedqr, Path]` in canonical descriptor mode** → exit 2 + `SlotInputViolation{kind:"conflict"}` (the widened gate). Baseline note (do NOT re-capture a drifted golden, `feedback_recapture_golden_only_when_current_correct`): pre-fix `[Seedqr, Path]` returned exit-1 `BadInput` via the binding-loop fall-through; the fix normalizes it to exit 2.
10. **descriptor-mode multisig** — Ms1 cosigner derives the correct xpub at the family path.
11. **`--self-check`** with a mnem Ms1 slot round-trips.

ms-codec has its own suite; toolkit gate per phase: `cargo test -p mnemonic-toolkit --no-fail-fast` + `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`. (No `cargo fmt` — toolkit has no fmt gate.)

---

## §9. Footguns / R0-anticipated

- mnem-English → entr-output (§3 edge) — documented + tested, not fixed.
- The Seedqr canonical-gate widening is an error-class NORMALIZATION (R0-I2), NOT a now-refuses-what-it-accepted change: `[Seedqr, Path]` against a canonical descriptor is ALREADY rejected (exit-1 `BadInput` via the binding-loop fall-through); the widened gate makes it exit-2 `SlotInputViolation{kind:"conflict"}` like `[Phrase, Path]`. For Ms1 the gate IS load-bearing (prevents silent mis-acceptance of an explicit path overriding the canonical origin). CHANGELOG + test 9 state the true baseline.
- `verify_bundle`'s descriptor binding loop has NO Entropy arm (pre-existing; `@N.entropy=` is unsupported in verify-bundle descriptor mode, falling through to `DescriptorReparseFailed`). The Ms1 arm (site 15) derives inline via the `derive_slot` helper — it does NOT mirror a nonexistent Entropy arm. File a FOLLOWUP `verify-bundle-descriptor-entropy-slot-gap` for the pre-existing entropy-slot omission (out of scope for this cycle).
- Helper drift: enforce the single `slot_ms1` helper is the ONLY decode+conflict site (the three binding loops call it; convert MAY be refactored onto it but that's optional/out-of-scope-flaggable).
- `ResolvedSlot.language` is `Option<bip39::Language>` (NOT `CliLanguage`) — the helper returns `bip39::Language`.
- Re-grep all §-cited line numbers against current source at impl time (they are `0814ab5`+branch snapshots).

---

## §10. Citations (verified at write time against branch `bundle-slot-ms1-input`, base `0814ab5`)

All file:line citations above were grep-verified during SPEC authoring. ms-codec 0.4.0 `Payload` read from published source (`mnemonic-secret` master `7b9d901` == crates.io 0.4.0): `payload.rs:28-57`. Synth emit rule `synthesize.rs:299-306,831-839`; `ResolvedSlot.language` `synthesize.rs:671`; verify whole-card compare `verify_bundle.rs:1245,1639`; canonical gate `bundle.rs:1151-1153`; convert language-collapse `convert.rs:1156`, ms1 arm `:1473-1483`; `SECRET_SLOT_SUBKEYS` `secret_taxonomy.rs:111`; friendly share prose `friendly.rs:110-114`; `SlotInputViolation` exit-2 `error.rs:519`.
