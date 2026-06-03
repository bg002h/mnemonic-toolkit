# Phase 3 R0 — ms K-of-N — round 0

**Scope:** toolkit `mnemonic ms-shares` (Tasks 3.1–3.3), branch `ms-v0.2-kofn-toolkit`,
diff `f95ddc4..HEAD` (3 commits: `958a434` split|combine; `8ad4cd9` friendly+exit; `572b3e0` re-pin).
**Verified against:** `mnemonic-secret/design/SPEC_ms_v0_2_kofn.md` §4 + `IMPLEMENTATION_PLAN_ms_v0_2_kofn.md`
Tasks 3.1–3.3; mirror target `crates/mnemonic-toolkit/src/cmd/slip39.rs`.
**Gate state at review:** workspace `cargo test -p mnemonic-toolkit --no-fail-fast` = **2616 passed / 0 failed**;
`cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` clean; controller smoke recovers all-zero entropy.

**Verdict: RED (0 Critical / 2 Important)**

---

## Critical

None.

## Important

### I1 — `combine --to entropy` silently drops the mnem wordlist language (no advisory); deviates from the slip39 mirror, on the cycle's own steered composition path
- **Where:** `crates/mnemonic-toolkit/src/cmd/ms_shares.rs::run_combine` lines 377–411. The recovered
  payload's language is extracted into `payload_lang` (line 382–385, `Payload::Mnem { language, .. }`),
  but for `MsSharesToShape::Entropy` (line 402) the code just `hex::encode(&entropy[..])` and discards
  `payload_lang` with **no** `non_english_seed_advisory` emission.
- **Mirror deviation:** `slip39.rs::run_combine` lines 654–662 emits exactly this advisory on its
  analogous `--to entropy` path:
  `if matches!(args.to, Slip39ToShape::Entropy) { if let Some(msg) = crate::language::non_english_seed_advisory(args.language, "raw entropy") { writeln!(stderr, "{msg}"); } }`.
  ms_shares omits it. (slip39 split also warns; ms_shares split correctly does NOT need to, because a
  non-English phrase splits as `Payload::Mnem` and the language survives in the shares — that asymmetry is
  correct and not the finding.)
- **Why it matters (runtime-proved, not theoretical):**
  - Probe: split a Japanese 12-word seed (`--language japanese`, all-zero 16-byte entropy) into 2-of-3,
    then `combine --to entropy` → emits `00000000000000000000000000000000` with **only** the argv +
    PrivateKeyMaterial advisories, **no** language warning. The exact slip39 invocation on the same
    secret emits: *"warning: encoding a japanese BIP-39 seed as raw entropy … recovering the entropy
    with English-defaulted software derives a DIFFERENT seed and a DIFFERENT wallet."*
  - The language-loss here is arguably **worse** than slip39's: the user explicitly chose to preserve the
    language (it was carried in every share as `Payload::Mnem`), and `--to entropy` throws away the very
    information the share-set was carrying — with no signal.
  - This is the path the cycle steers users to. SPEC §4 + the headline test
    `cli_ms_shares.rs::ms_shares_combine_to_entropy_composes_into_bundle` (lines 172–210) demonstrate the
    composition as `combine --to entropy` → `bundle --slot @0.entropy=`. Probe confirms
    `bundle --slot @0.entropy=00… ` (English default) derives a **different** `ms1`/`mk1` (different
    wallet) than `bundle --slot @0.entropy=00… --language japanese` (correct wallet). A user who follows
    the steered recipe for a non-English seed and does not re-supply `--language` gets a silently
    different wallet. The only signal that they must is the advisory — which is absent.
- **Not Critical because:** no secret leakage; the entropy is correctly recovered; language-safe paths
  exist (`--to phrase` re-renders the card language; `--to ms1` preserves `payload_kind: Mnem` +
  `language` in the wire bytes — both probe-confirmed byte-identical to the canonical `convert` output);
  and `bundle --slot @0.entropy= --language X` does derive correctly. It is a safety/UX regression vs the
  established mirror, on a load-bearing path, hence Important.
- **Fix:** mirror slip39.rs:654–662 — in `run_combine`, for `MsSharesToShape::Entropy`, if the recovered
  payload is `Payload::Mnem { language, .. }` (i.e. the recovered secret carried a wordlist language),
  emit `non_english_seed_advisory(<that language>, "raw entropy")`. Note the source of truth must be the
  **recovered payload's** language (the mnem wire code → CliLanguage), NOT `args.language` — for an
  `entr`-payload combine there is no language to lose, and for a mnem-payload combine the relevant
  language is the card's, not the (ignored) `--language` flag. Add a test:
  `combine --to entropy` of a Japanese mnem share-set emits the advisory; `--to phrase`/`--to ms1` do not;
  an English/entr share-set emits nothing.

### I2 — Planned friendly arms for the surfaced codex32 share errors were not added; combine's primary error modes Debug-dump
- **Where:** `crates/mnemonic-toolkit/src/friendly.rs::friendly_ms_codec`. The `ms_codec::Error::Codex32(c)`
  arm (friendly.rs:49) renders `format!("ms1 codex32: {:?}", c)` — a `Debug` dump.
- **Plan deviation:** `IMPLEMENTATION_PLAN_ms_v0_2_kofn.md` Task 3.3 Step 3 explicitly scopes:
  *"Add explicit arms in `friendly_ms_codec` for `IsShareNotSingleString`, `SecretShareSuppliedToCombine`,
  **and the surfaced `Codex32(Mismatched*/RepeatedIndex/ThresholdNotPassed)`**."* The first two were
  added (correctly); the codex32 share-error sub-arms were **not** — they fall through to the pre-existing
  generic Debug-dump arm.
- **Runtime probes (the two most common combine errors for the new command):**
  - too-few shares (1 of a 2-of-3): `error: ms1 codex32: ThresholdNotPassed { threshold: 2, n_shares: 1 }`
  - duplicate share: `error: ms1 codex32: RepeatedIndex(Fe(0))`  ← `Fe(0)` is opaque to a user.
  - ms-cli renders these as prose via `codex32_friendly.rs`: *"not enough shares: have 1, need 2"* and a
    human `RepeatedIndex` message — the toolkit is inconsistent with its sibling and with the plan.
- **Not Critical because:** the message is technically correct and (crucially) does NOT hit the
  `_ => "unhandled ms_codec::Error variant"` wildcard — the R0-m3 requirement that was the Critical risk is
  satisfied. The codex32 Debug arm is pre-existing (not introduced this cycle). But these are now
  first-class, user-hit error paths for a brand-new command (`combine`), the plan named them as a Task 3.3
  deliverable, and Debug-dumping `Fe(0)` is poor UX. Important.
- **Fix:** add explicit `Codex32(codex32::Error::ThresholdNotPassed { .. } | RepeatedIndex(..) |
  MismatchedHrp(..) | MismatchedId(..) | MismatchedThreshold(..) | MismatchedLength(..))` sub-arms in
  `friendly_ms_codec` with prose mirroring ms-cli's `codex32_friendly.rs:40–56` (toolkit consumes
  `codex32` directly; that ms-cli fn lives in a bin crate so port the strings, don't import). Add a test
  asserting `ThresholdNotPassed`/`RepeatedIndex` produce prose (no `{` / `Fe(` in the message).

## Minor

### M1 — trimmed `shares: Vec<String>` handed to `combine_shares` is non-Zeroizing secret residue
- `ms_shares.rs:361–365` builds `let shares: Vec<String> = share_strings.iter().filter(...).map(|s| s.trim().to_string()).collect();` — plain `String` copies of secret share material (the originals in
  `share_strings` are `Zeroizing<String>` + mlock-pinned, but these trimmed clones are not). `combine_shares(&[String])` forces a `String`, but the collection could deref `Zeroizing<String>` or wrap the
  trimmed copy. Low severity: a share is a distributed backup piece, the codec clones internally anyway,
  and the pinned Zeroizing originals exist. Consider `Vec<Zeroizing<String>>` of the trimmed values, or
  document the rationale. (Not blocking.)

---

## Confirmations (probe-backed)

1. **Controller smoke GREEN.** `ms-shares split --from entropy=00… --threshold 2 --shares 3` → 3 `ms1…`
   shares; `combine --share s1 --share s3 --to entropy` → `00000000000000000000000000000000`, exit 0.
2. **Workspace 2616/0; clippy clean.** Targeted: `cli_ms_shares` 16/0, `cli_ms_shares_consume` 12/0,
   `lint_argv_secret_flags` 2/0, `cli_gui_schema` 4/0.
3. **Output-class + zeroize (point 2).** Both split and combine emit `OutputClass::PrivateKeyMaterial`
   ("stdout carries private key material (can spend)") — probe-confirmed on stderr for both. Secret bytes
   wrapped `Zeroizing` (from-value, parsed entropy, rendered shares, recovered entropy, output) + per-buffer
   `mlock::pin_pages_for` at every site, including the O(N) per-share emit-loop pin (mirrors slip39's Q6
   discipline). `parse_secret_to_entropy` returns `Zeroizing<Vec<u8>>`; source-parse reuse
   (`parse_from_input`/`FromInput`/`read_stdin_to_string`) is clean — no plaintext copy escapes (one Minor:
   M1). argv-leak advisory fires at the right sites: split inline `--from` (probe-confirmed), combine
   per-occurrence inline `--share` (probe-confirmed two warnings for two inline shares).
4. **Non-English split → `Payload::Mnem` (point 1).** `ms_shares.rs:265–274` routes a non-English
   `phrase=` to `Payload::Mnem { language: cli_language_to_wire_code(...), entropy }`; English / `entropy=`
   → `Payload::Entr`. Probe: Japanese split→`combine --to phrase` (no `--language`) recovers the exact JA
   phrase from the card language (test `ms_shares_japanese_split_combine_preserves_language` passes).
   `--to ms1` of the JA set re-encodes to `ms10entrsqgqs…` with `inspect` showing `payload_kind: Mnem` /
   `language: japanese`, byte-identical to canonical `convert --from phrase --to ms1 --language japanese`.
5. **friendly + exit-code arms (point 3) consistent with ms-cli.**
   - `friendly.rs`: explicit arms for `IsShareNotSingleString` (→ "use `mnemonic ms-shares combine`"),
     `SecretShareSuppliedToCombine`, `InvalidThreshold`, `InvalidShareCount`. None hit the "unhandled"
     wildcard (asserted in 4 new unit tests).
   - `error.rs::ms_codec_exit_code`: `IsShareNotSingleString` + `SecretShareSuppliedToCombine` → 2;
     `InvalidThreshold`/`InvalidShareCount` → wildcard → 1. **Identical to ms-cli** (`ms-cli/src/error.rs`:
     `IsShareNotSingleString`/`SecretShareSuppliedToCombine` → `FormatViolation` → 2;
     `InvalidThreshold`/`InvalidShareCount` → `BadInput` → 1).
   - `From<ms_codec::Error>` (error.rs:823) only special-cases `ReservedTagNotEmittedInV01`; the 4 new
     variants land in `MsCodec(other)` and reach the arms.
   - Probe: `convert --from ms1=<a share> --to phrase` → exit **2** + "this is ONE of a K-of-N share set …
     use `mnemonic ms-shares combine`" (NOT "unhandled variant"). `inspect --ms1 <share>` → same friendly
     surface, exit ≠ 0. `cli_ms_shares_consume.rs` asserts both. No OTHER toolkit wildcard swallows the new
     variants (sole `ms_codec::Error` match sites are `ms_codec_exit_code` + `friendly_ms_codec`, both
     updated). (I2 is the residual gap on the *codex32-wrapped* share errors, separate from these 4.)
6. **combine→bundle composition + `@0.ms1=` plan inaccuracy (point 4) correctly adjudicated.** Probe
   confirms `bundle` has **no** `ms1` slot subkey (`unknown slot subkey "ms1"; expected one of: phrase,
   seedqr, entropy, xpub, master_xpub, fingerprint, path, wif, xprv`). The plan/SPEC §4 `--slot @0.ms1=`
   citation is wrong; the implementer's `combine --to entropy` → `@0.entropy=` (and `--to phrase` →
   `@0.phrase=`) composition is the correct realization. No direct `ms1`-slot door needed this cycle. The
   residual language-loss on the steered `--to entropy` recipe is I1 above (the language-safe doc path is
   `combine --to phrase` → `bundle --slot @0.phrase= --language X`, or `--to entropy … --language X`,
   both probe-confirmed to derive the correct JA wallet).
7. **Lint gates (point 5) correctly + completely updated.**
   - `lint_argv_secret_flags.rs`: `ms-shares-combine --share` added to FLAG_ROUTES; `ms-shares-split
     --from` added to FROM_ROUTES (evidence strings present in source). Test passes.
   - `cli_gui_schema.rs::gui_schema_lists_all_subcommands`: 26→28, both `ms-shares-combine` +
     `ms-shares-split` inserted alphabetically; count + prose updated. Test passes.
   - `secrets.rs::flag_is_secret` already classifies `--share` (secret) and `--passphrase`; `--from`
     non-secret at flag-name level (secrecy is value-prefix/node-type). ms-shares introduces **no new
     secret flag-name** → no `flag_is_secret` update owed (reuses the established classification). No other
     toolkit closure (secret-flag set-equality, help-pointer, schema closure) trips on the two new
     subcommands.
8. **TEMP override (point 6).** `Cargo.toml [patch.crates-io] ms-codec = { path = "../mnemonic-secret/
   crates/ms-codec" }` present, clearly commented "REMOVE AT SHIP". Crate dep `ms-codec = "0.4.0"` (caret);
   ms-cli tag pins `ms-cli-v0.7.0` at `scripts/install.sh:38` + `.github/workflows/manual.yml:88`.
   Cargo.lock staged with `ms-codec 0.4.0` resolved. Toolkit version is **still 0.39.0** (both README
   `toolkit-version:` markers 0.39.0) — the v0.40.0 bump is correctly **held** for after this R0 (Task
   3.4).
9. **Nothing masked (point 7).** No weakened test, no secret on a clearly-wrong non-Zeroizing path beyond
   M1, no new clippy-allow introduced by the diff, no lint closure loosened (both were extended, not
   relaxed). The `Codex32(_)` Debug arm is pre-existing, not a this-cycle loosening (it is the subject of
   I2 as a missing planned extension).

## Deferred / out of Phase-3 scope (NOT findings)
- Manual `docs/manual/src/40-cli-reference/` mirror of `mnemonic ms-shares` and `mnemonic-gui`
  `schema/mnemonic.rs` + `schema/ms.rs` mirror are **P4** per SPEC §7 (docs/GUI lockstep), a separate
  phase. Verified the manual `cli-subcommands.list` does NOT yet list `ms-shares`, so the manual
  flag-coverage CI gate will not fire red before P4 adds it. (Flagged here only so P4 is not forgotten —
  the paired-PR / mirror invariants in CLAUDE.md must be satisfied before tag.)

---

**Re-dispatch after fold:** per CLAUDE.md, after folding I1 + I2 (and optionally M1), re-run the full
`-p mnemonic-toolkit --no-fail-fast` + clippy gate, persist the round-1 review, and re-dispatch the
architect until 0C/0I before advancing to Task 3.4 / Phase 4.
