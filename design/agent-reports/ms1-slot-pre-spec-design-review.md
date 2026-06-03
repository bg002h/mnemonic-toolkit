# ms1-slot — Pre-SPEC Architect Design Review
**Verdict:** SOUND-WITH-CHANGES

Source ground truth read on branch `bundle-slot-ms1-input` (citations verified live against the working tree). The three approved decisions are sound in principle, but the design as written contains one Critical inaccuracy (output symmetry is NOT free for the case the design most cares about — non-trivially, the entr↔mnem distinction breaks verify-bundle round-trip) plus several Important wiring-surface gaps the recon under-counted. None block the decisions; all are foldable into the SPEC.

## Critical (2) — must change before SPEC

### C1. The verify-bundle round-trip claim is WRONG for the entr/mnem axis — `ms1_entropy_match` is a FULL-STRING compare, not an entropy compare.
The design's stated justification (c) is "round-trip symmetry (engrave an ms1 card → feed the same card back)" and decision 3 promises output symmetry so "the emitted ms1 card round-trips kind+language." But the verify-bundle ms1 check is byte-string equality of the WHOLE card, despite its name:
- single-sig: `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:1245` — `if supplied_ms1 == expected_ms1`
- multisig:  `verify_bundle.rs:1639` — `if s == exp_ms1`

Consequence: `verify-bundle --slot @0.ms1=<entr-ms1-of-E> --ms1 <mnem-ms1-of-E>` (or the reverse) will MISMATCH even though both decode to the same entropy E and derive the same xpub — because the expected card's tag byte / language prefix differs from the supplied card's. The whole point of accepting an ms1 card on a slot is so the user can feed back the very card they engraved; if the engraved card is `mnem` (non-English) and the user re-supplies it on `@N.ms1=`, the *expected* card is recomputed and will match ONLY IF the slot arm sets `ResolvedSlot.language = Some(wire_lang)` so the emit path re-emits `mnem`. So:
- **Output symmetry is not a "nice to have" — it is LOAD-BEARING for the round-trip justification.** Setting `ResolvedSlot.language = Some(wire_lang)` for mnem / `None` for entr (decision 3, output symmetry) is REQUIRED, not optional, and the SPEC must mark it so and add the round-trip regression test (entr-ms1-slot → expected entr card; mnem-ms1-slot → expected mnem card; supply-the-same-card-back → VERIFIED).
- The SPEC must NOT drop the output-symmetry claim "for this cycle" — dropping it silently breaks the headline use-case.

### C2. The `resolve_slots` Ms1 arm CANNOT be "modeled VERBATIM on convert.rs:1464-1477" — convert has no conflict check and collapses absent→English.
`convert.rs` Ms1 arm (`crates/mnemonic-toolkit/src/cmd/convert.rs:1463-1486`) takes its language from `let language = args.language.unwrap_or_default();` (`convert.rs:1156`) — i.e. it has ALREADY lost the Some/None distinction before the arm runs, and it does NOT refuse on a `--language`-vs-wire conflict (for mnem it just uses wire and silently ignores `--language`). Decision 3 ("HARD REFUSE on conflict") is therefore NET-NEW behavior with NO existing model. The slot arm must:
1. receive `language: Option<CliLanguage>` (which `resolve_slots` already does — `cmd/bundle.rs:456`), and
2. branch on `payload`:
   - `Entr(bytes)` → `lang_bip39 = language.unwrap_or_default().into()`; route through `derive_bip32_from_entropy[_at_path]` (the existing Entropy arm at `bundle.rs:608-657`); set `ResolvedSlot.language = None`.
   - `Mnem { entropy, language: wire }` → `wire_bip39 = wire_code_to_bip39(wire)?`; **conflict gate**: `if let Some(flag) = language { if flag.into() != wire_bip39 { return Err(...language-conflict...) } }`; derive with `wire_bip39`; set `ResolvedSlot.language = Some(wire_bip39)`.
The SPEC must spell this out and explicitly state convert.rs is the *decode-and-payload-match* model only, NOT the language-policy model.

## Important (5) — should fold into SPEC

### I1. Descriptor-mode is a SECOND, hand-rolled binding loop that does NOT call `resolve_slots` — Ms1 must be wired there too, and it sets `language` differently.
`bundle_run_unified_descriptor` (`cmd/bundle.rs:1088`) has its own per-slot decode loop (`bundle.rs:1305-1415`) with arms for Phrase (`:1305`), Xpub (`:1345`), Entropy (`:1375`), and `else → BadInput` (`:1408`). It does NOT route through `resolve_slots`. So an Ms1 slot in descriptor mode falls through to the BadInput at `:1408` unless a dedicated Ms1 arm is added here. Note this loop builds `CosignerKeyInfo` (alias of `ResolvedSlot`) and currently pushes `language: None` (`bundle.rs:1428`), with the language later re-derived via `Some(c.language.unwrap_or(run_language))` at `bundle.rs:1486-1487`. For a mnem Ms1 slot the new arm MUST push `language: Some(wire_bip39)` at the cosigner-push (line 1422-1430) so `synthesize_descriptor` (`synthesize.rs:298`) re-emits `mnem`. This is the same emit rule as template mode (`synthesize.rs:831`), so symmetry holds once `language` is populated.

### I2. verify-bundle has a THIRD hand-rolled binding loop — Ms1 must be wired there too.
`verify_bundle.rs:781-855` is the descriptor-mode mirror of I1 (it DOES handle Seedqr at `:781-782`, unlike bundle.rs's loop which omits Seedqr). It also pushes `language: None` (`verify_bundle.rs:865`). An Ms1 arm must be added with the same conflict-gate + `language: Some(wire)` semantics. So the full Ms1-decode-and-derive logic appears in FOUR places: `resolve_slots` (template, shared by bundle+verify), `bundle_run_unified_descriptor`, `verify_bundle` descriptor loop, and (already shipped) `convert`. The SPEC must factor the decode+conflict logic into a shared helper (e.g. `slot_ms1::decode_and_resolve_language(value, language: Option<CliLanguage>, idx) -> Result<(Zeroizing<Vec<u8>>, Option<bip39::Language>), ToolkitError>`) to avoid four-way drift — per `feedback_fix_the_class_hunt_for_second_instance`. Do NOT hand-inline the conflict check four times.

### I3. The canonical-mode rejection gate is `has_phrase && has_path` ONLY — `[Ms1, Path]` would slip past it.
`bundle.rs:1139-1162` rejects `[Phrase, Path]`/`[Phrase, Fingerprint, Path]` in canonical descriptor mode by checking `has_phrase && has_path` (`:1151-1153`). It already MISSES Seedqr (a latent pre-existing narrowness — Seedqr+Path in canonical mode is not rejected here either). If the design grants `[Ms1, Path]`/`[Ms1, Fingerprint, Path]` full parity (decision 4), the SPEC must extend this canonical-mode gate to `(has_phrase || has_seedqr || has_ms1) && has_path` — otherwise an Ms1 explicit-origin slot against a canonical descriptor reaches the default-path-override loop (`bundle.rs:1228-1232`, which also only checks Phrase/Seedqr) inconsistently. Flag the pre-existing Seedqr omission as a sibling find (fix-the-class). Note: the descriptor binding-loop `else→BadInput` would still catch an un-armed Ms1, but once Ms1 IS armed the gate gap becomes live.

### I4. Ord position has a concrete `is_legal_set` consequence — `Ms1` after `Entropy` makes the canonical arms `[Ms1]`, `[Ms1, Path]`, `[Ms1, Fingerprint, Path]`.
Decision 1 places `Ms1` right after `Entropy`. Derived `Ord` then yields `Phrase < Seedqr < Entropy < Ms1 < Xpub < MasterXpub < Fingerprint < Path < Wif < Xprv`. `validate_slot_set` sorts subkeys ascending (`slot_input.rs:265`), so the canonical arms are `[Ms1]`, `[Ms1, Fingerprint, Path]`, `[Ms1, Path]` — i.e. **`Fingerprint` precedes `Path`** (matching the existing `[Phrase, Fingerprint, Path]` arm order at `slot_input.rs:348`). The design's arm spelling is correct. ALSO: the `exempted_v0_19_0` matrix (`slot_input.rs:289-295`) must gain `[Ms1, Path]` and `[Ms1, Fingerprint, Path]` or the secret+watch conflict refusal (`:297`) fires before `is_legal_set` is consulted. The recon listed `is_legal_set` but the SPEC must ALSO list the `exempted_v0_19_0` block as a distinct edit site.

### I5. Exit code for the conflict — recommend exit 2 (FormatViolation/SlotInputViolation-class), NOT exit 1.
The design proposes exit 1 (BadInput-class). I disagree, with precedent: a well-formed codex32 string whose wire language contradicts a supplied `--language` is a *format/shape contradiction between two well-formed inputs*, not a malformed input. The directly-analogous precedents in this repo are exit 2:
- `ms_codec::Error::IsShareNotSingleString` (well-formed string, wrong shape for the op) → exit 2 (`error.rs:369-370`).
- `SlotInputViolation` (all kinds incl. path-mismatch between `--slot` and descriptor inline) → exit 2 (`error.rs:519`).
- The descriptor-vs-slot path-mismatch refusal (`bundle.rs:1244-1249`) is `SlotInputViolation{kind:"path-mismatch"}` → exit 2.
A wire-vs-flag language contradiction is the language analogue of that path-mismatch. **Strongest option: reuse `SlotInputViolation { kind: "language-conflict", message }` rather than adding a new variant** — it already exits 2, already carries a `kind` JSON discriminant (`error.rs:797`), is already in the alphabetical block, and needs NO error.rs edit. If a dedicated variant is preferred for message-template clarity, place `Ms1SlotLanguageConflict` alphabetically (between `ModeViolation` at `error.rs:240` and `SlotInputViolation` at `:284`) and route it exit 2 in both `exit_code` (`:471`) and `kind` (`:529`) match blocks. Either way, NOT exit 1.

## Minor (4)

### M1. `friendly.rs` IsShareNotSingleString prose is confirmed present (`friendly.rs:110-114`) and the share-rejection routes exit 2 (`error.rs:369`). Decision/finding E claim holds — the prose says "use `mnemonic ms-shares combine`", correct for the share case. No change needed; the SPEC should cite `friendly.rs:110` as the inherited footgun-guard and add ONE test (`@N.ms1=<a-share>` → exit 2 + share prose).

### M2. The stdin sentinel (`@N.ms1=-`) + argv-leak advisory ARE inherited for free once `is_secret_bearing()→true` (`slot_input.rs:100` gates on `is_secret_bearing`) AND `SECRET_SLOT_SUBKEYS` gains "ms1" (`secret_taxonomy.rs:111`). The parity test `secret_taxonomy_parity_with_is_secret_bearing` (`slot_input.rs:404`) is a HARD same-PR gate. The `declare_slot_subkey_variants!` macro (`slot_input.rs:391-401`) must gain `Ms1` or the `_exhaustiveness_check` match goes non-exhaustive (compile fail) — the intended tripwire.

### M3. Emit helper choice: template + descriptor emit both key on `slot.language: Option<bip39::Language>` and call `bip39_to_wire_code` (`synthesize.rs:836`, `:303`, `language.rs:120`). So the slot arm must store `Some(bip39::Language)` (NOT `Some(CliLanguage)` — `ResolvedSlot.language` is `Option<bip39::Language>` per `synthesize.rs:671`). Use `wire_code_to_bip39(wire)` (`language.rs:96`) to produce it. The existing `payload_bip39_language` helper (`language.rs:143`) takes `CliLanguage` (not Option) so it CANNOT host the conflict check — write the shared helper from I2 fresh.

### M4. entr-ms1 byte-identity (finding D) holds: `Entr(bytes)` yields exactly the entropy E, routed through the SAME `derive_bip32_from_entropy[_at_path]` the `Entropy` arm uses (`bundle.rs:625/632`), with `language.unwrap_or_default()` → English default identical to the Entropy arm (`bundle.rs:621`). So `@N.ms1=<entr-ms1-of-E>` ≡ `@N.entropy=<hex E>` byte-for-byte in xpub AND emitted card. All five BIP-39 lengths (16/20/24/28/32) decode identically (`friendly.rs:101-102`). Add a parametric byte-identity test across all five lengths.

## Site enumeration the SPEC must cover (the full Phrase/Seedqr-parity list, with file:line)
Verified against branch `bundle-slot-ms1-input`:

**slot_input.rs (surface):**
1. `enum SlotSubkey` — add `Ms1` after `Entropy` — `slot_input.rs:29`
2. `from_token` — `"ms1" => Self::Ms1` — `slot_input.rs:45-58`
3. `as_str` — `Self::Ms1 => "ms1"` — `slot_input.rs:59-71`
4. `is_secret_bearing` — add `| Self::Ms1` — `slot_input.rs:72-77`
5. unknown-subkey error string — append `ms1` — `slot_input.rs:160-165`
6. `is_legal_set` — add `[Ms1]`, `[Ms1, Path]`, `[Ms1, Fingerprint, Path]` — `slot_input.rs:330-352`
7. `exempted_v0_19_0` matrix — add `[Ms1, Path]`, `[Ms1, Fingerprint, Path]` — `slot_input.rs:289-295` (recon MISSED this as a distinct site)
8. `declare_slot_subkey_variants!` macro — add `Ms1` — `slot_input.rs:391-401`

**secret_taxonomy.rs:**
9. `SECRET_SLOT_SUBKEYS += "ms1"` — `secret_taxonomy.rs:111`

**bundle.rs (template mode — shared by bundle + verify):**
10. `resolve_slots` — new Ms1 arm (decode + conflict + derive + `language` set), with `multisig_acct_path` branch — `bundle.rs:486-657` (insert a new `else if subkeys.contains(&SlotSubkey::Ms1)` arm; the catch-all is at `:711`)

**bundle.rs (descriptor mode — NOT routed through resolve_slots):**
11. canonical-mode rejection gate — extend `has_phrase && has_path` to include Ms1 (+ fix latent Seedqr omission) — `bundle.rs:1142-1162`
12. default-path-override loop — extend the `!Phrase && !Seedqr` continue to also pass Ms1 — `bundle.rs:1222-1232`
13. per-slot binding loop — new Ms1 arm before the `else→BadInput`; push `language: Some(wire)` at cosigner push — `bundle.rs:1305-1430` (BadInput at `:1408`, push at `:1422`)

**verify_bundle.rs (descriptor mode — third hand-rolled loop):**
14. default-path-override loop — `!Phrase && !Seedqr` continue → also pass Ms1 — `verify_bundle.rs:715-723`
15. per-slot binding loop — new Ms1 arm before the `else→` error; push `language: Some(wire)` — `verify_bundle.rs:776-867` (Seedqr handling at `:781-782`, error-equiv at `:848`, push at `:859`)
   (verify_bundle template-mode resolve_slots calls at `:363,:453,:557` need NO Ms1-specific edit — they inherit the shared `resolve_slots` arm from site 10.)

**error.rs (only if a dedicated variant is chosen over reusing SlotInputViolation — I5 prefers reuse):**
16-19. `Ms1SlotLanguageConflict` variant + `exit_code`(→2) + `kind` + `Display`/JSON detail — `error.rs:240/471/529/590,786`

**Lockstep / docs:**
20. `--slot` clap `verbatim_doc_comment` — add ms1 line — `bundle.rs:94-113` + verify_bundle `--slot` doc (`verify_bundle.rs:117-120`)
21. manual `docs/manual/src/40-cli-reference/41-mnemonic.md` — `--slot` subkey prose
22. paired `mnemonic-gui` PR — `src/form/slot_editor.rs::SlotSubkey` picker + `src/secrets.rs` SECRET_SLOT_SUBKEYS snapshot (confirm at impl whether a GUI drift test consumes the toolkit const or is hand-maintained)

**Shared helper (per I2, strongly recommended):**
23. new `slot_ms1` helper hosting decode + language-conflict logic, consumed by sites 10, 13, 15 (and optionally refactor convert.rs:1463 onto it).

## Answers to A–F (each with evidence)

**A. Output symmetry feasibility — FEASIBLE and partially free, but with a sharp edge.**
Template emit (`synthesize_unified`, `synthesize.rs:824-847`) and descriptor emit (`synthesize_descriptor`, `synthesize.rs:295-313`) BOTH already implement the rule `slot_lang = s.language.unwrap_or(run_language); if slot_lang == English → Payload::Entr else → Payload::Mnem`. So once `resolve_slots` sets `ResolvedSlot.language = Some(wire_bip39)` for a mnem slot, the mnem card IS re-emitted for free — the recon's worry that this is wired "only through `--import-json`" is HALF right: the EMIT side is generic (both synthesize fns honor `slot.language`); only the POPULATION side currently sets `language` non-None solely in the import-json arms (`bundle.rs:1687,:1754`). The template `resolve_slots` arms set `None` (`bundle.rs:535,605,655,698`) and the descriptor loop sets `None` (`bundle.rs:1428`). The Ms1 arm changes exactly that population. **Sharp edge:** if `wire_lang == English` (a mnem-English ms1), the emit rule emits `Entr` (because effective lang == English → Entr branch). So a mnem-English INPUT card round-trips as an entr OUTPUT card — NOT byte-identical to the supplied card, and verify-bundle's full-string compare (C1) MISMATCHES. The SPEC must document that mnem-English is the one non-round-tripping case (acceptable: English self-recovers, and `bip39_to_wire_code(English)=0` is the canonical entr; and per the mnem cycle the encoder never EMITS mnem-English — English always routes to entr — so a mnem-English ms1 is only constructible by a third party, an edge worth a documented note + test, not a blocker).

**B. Full-parity wiring surface — LARGER than the recon implied: THREE binding loops, not one shared arm.**
The recon framed this as "one `resolve_slots` arm + Phrase/Seedqr peer-checks." Reality: `resolve_slots` (`bundle.rs:451`) covers ONLY template mode. Descriptor mode has two independent hand-rolled loops — `bundle_run_unified_descriptor` (`bundle.rs:1305-1415`) and `verify_bundle` (`verify_bundle.rs:776-855`) — each with its own arms and `else→error`. Decision 4's "full parity at EVERY special-case site" therefore means an Ms1 arm in all THREE loops plus the canonical-mode gate (`bundle.rs:1142`) and the two default-path-override `!Phrase && !Seedqr` continues (`bundle.rs:1228`, `verify_bundle.rs:719`). Hidden complexity confirmed: the `[Phrase, Path]` canonical-mode post-parse rejection (`bundle.rs:1139-1162`) is real and currently checks only `has_phrase && has_path` — it must be widened for Ms1 (and is latently narrow for Seedqr today). This is a MEDIUM surface (≈8 code edit sites), not small. The full site list is enumerated above; the SPEC MUST enumerate all of them or descriptor-mode Ms1 will hit `else→BadInput` (`bundle.rs:1408`) inconsistently between subcommands.

**C. Language-conflict mechanics — clean, with one helper choice.**
`resolve_slots` DOES receive `language: Option<CliLanguage>` (`bundle.rs:456`); `--language` has NO clap `default_value` on bundle (`bundle.rs:41-42`) NOR verify-bundle (`verify_bundle.rs:40-41`), so `None ⟺ absent` holds. verify-bundle passes the SAME `args.language` (Option) into all three `resolve_slots` calls (`verify_bundle.rs:368,:458,:562`), so the refuse fires symmetrically. Cleanest comparison: in `bip39::Language` space — `flag.into() != wire_code_to_bip39(wire)?` (using `language.rs:96` + `CliLanguage→bip39` at `language.rs:153`). For exit code: exit 2 (FormatViolation-class), NOT exit 1 — see I5. Recommend reusing `SlotInputViolation{kind:"language-conflict"}`.

**D. entr-ms1 byte-identity — CONFIRMED.** See M4.

**E. Reserved/edge payloads — CONFIRMED.** `ms_codec::decode` returns `(Tag, Payload)` and the toolkit already pattern-matches `Entr`/`Mnem` with `_ => BadInput("unknown payload kind")` at `convert.rs:1481-1485` (mirror this `_` arm in the slot helper). Reserved tags / shares are rejected by `decode` itself: `IsShareNotSingleString` is a decode-time Error (not a Payload variant), routes exit 2 (`error.rs:369`), and has shipped friendly prose (`friendly.rs:110-114`). Stdin sentinel + argv-leak advisory inherit for free from `is_secret_bearing()→true` (`slot_input.rs:100`) + SECRET_SLOT_SUBKEYS membership. (Could NOT independently re-confirm ms-codec 0.4.0's exact Payload reserved-variant list from docs.rs — the SPEC must pin the ms-codec 0.4.0 `payload.rs` variant set by reading the published crate source, NOT memory, per `feedback_verify_cited_apis_against_docs_rs`.)

**F. Anything else.**
- `convert.rs:1464-1477` is the right *decode/payload-match* model but the WRONG *language-policy* model (C2).
- convert's Ms1 arm has NO multisig path (projection-only) — model the DERIVATION on the EXISTING Entropy arm (`bundle.rs:608-657`, which has the `multisig_acct_path` branch), NOT convert. SPEC: "decode like convert, derive like the Entropy arm."
- Read-only boundary: CONFIRMED in-scope — decode→derive xpub→emit cards, no signing.
- Tests the design omits: (1) entr/mnem round-trip via verify-bundle (C1 regression), (2) entr-ms1 ≡ entropy byte-identity across 5 lengths (M4), (3) language-conflict refusal in BOTH bundle and verify-bundle, (4) mnem-English → entr-output documented edge (A), (5) `[Ms1, Path]` in canonical descriptor mode rejected (I3), (6) Ms1 descriptor-mode multisig derivation (I1), (7) share-rejection prose (M1), (8) `--self-check` round-trip with a mnem Ms1 slot per `feedback_self_check_bypasses_csi_grouping` / `feedback_verify_bundle_round_trip_per_phase_r0_scope`.
