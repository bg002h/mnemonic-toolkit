# Cycle-8 whole-diff review (ms-cli robustness/advisory cluster: H4 · H5 · L26 · L5)

- **HEAD:** `b2657b9` (`feature/cycle8-mscli-panics`)
- **Base:** `44ac71f` (`origin/master`, ms-codec `0.5.0`, ms-cli `0.8.1`)
- **5 commits:** `4117910` (P1 helper) / `d632ff0` (H4 derive) / `2f691cf` (H5 verify) / `3bb6746` (L26 combine) / `b2657b9` (L5 error Debug)
- **Reviewer:** opus software architect (mandatory non-deferrable independent adversarial execution review, per CLAUDE.md (4) + plan §9)
- **Date:** 2026-06-21
- **Scope:** the WHOLE diff `git diff origin/master..HEAD` — R0 validated the plan; this review hunts implementation-introduced defects TDD missed, with an adversarial focus on the non-English fingerprint funds-path.

---

## Method

- Read the full plan (`IMPLEMENTATION_PLAN_cycle8_mscli_panics.md`) + every changed source + test file.
- Built the `ms` binary; **empirically** derived/verified real French / Japanese / English cards.
- **Mutation-tested the funds-path** (helper `wire_cli` → `cli_lang`) on BOTH derive and verify; restored.
- **Mutation-tested L5** (hand-rolled Debug → echoing inner codex32 error); restored.
- Ran full `cargo test -p ms-cli` (215/0) + `cargo test -p ms-codec` (150/0) + `cargo clippy --all-targets -p ms-cli -- -D warnings` (clean), post-restoration.
- Audited every `cli_lang` / `args.language` / `defaulted` read for label-mis-thread.
- Worktree confirmed clean after all mutations reverted (`git status --short` empty; `git diff --stat` on mutated files empty).

---

## Critical

**None.**

---

## Important

**None.**

---

## Minor

### M-1 (informational, not a defect) — verify discards `_effective_lang_defaulted` by design
`verify.rs:72` binds `_effective_lang_defaulted` and never reads it. This is correct: verify has a single
label site (`emit_round_trip_ok` at `:119`, using `effective_lang.as_str()`) and no `(DEFAULT)` text label
(unlike derive). The `_`-prefix correctly silences the unused-binding lint. No action.

### M-2 (informational) — `tag: entr` wire label on a `Mnem` card is codec-internal, not a defect
`ms inspect` on a French `Mnem` card shows `tag: entr` but `kind: mnem`, `language: french`, prefix byte
`0x02`, payload `06…` (leading French language code `0x06`). The `Mnem` payload IS exercised through the
helper's `Mnem` arm (confirmed empirically: French fp `7d53dc37`, Japanese fp `11f73c61`, both wire-correct).
This is the codec's existing transport-tag/prefix-byte scheme, untouched by this cycle. No action.

### M-3 (informational) — version still `0.8.1` in the worktree
The P6 bump to `0.9.0` (Cargo.toml / CHANGELOG / README / tag / publish) has **not** happened yet — correct,
since this review is the gate that PRECEDES P6 step 6. mlock.rs, Cargo.toml, Cargo.lock all untouched in the
diff (g6 byte-share intact; no stray `cargo fmt`). The 5 ignored tests are all pre-existing env-gated
mlock/g6-invariant tests; no `#[ignore]` was added in the diff.

---

## Hunt-item findings (adversarial, per plan §9 + dispatch charge)

### 1. The fingerprint funds-path (THE item) — PASS (mutation-proven, empirically verified)
Every entropy/fp path in derive + verify routes a `Mnem` payload through
`payload_entropy_and_language`, which returns `wire_cli` (`CliLanguage::from_code(wire_code)`), and
`from_entropy_in(effective_lang.into(), …)` builds under that wire language. **No path uses `cli_lang` /
`--language` for `Mnem` entropy.**
- **Empirical:** built real cards → `derive` of a French (`[0;16]`) `Mnem` card → fp `7d53dc37` +
  `language: french`; Japanese (`[0xAB;16]`) → fp `11f73c61` + `language: japanese`; English `Entr` (zeros)
  → fp `73c5da0a` + `language: english (DEFAULT)`. All wire-correct.
- **Mutation (derive):** helper line `(Zeroizing::new(entropy), cli_lang, false)` → the French funds-safety
  test went RED: `expected French fp 7d53dc37, got: master_fingerprint: 73c5da0a` (the exact wrong-wallet
  English fp). 4/6 derive tests RED. Restored → GREEN.
- **Mutation (verify):** same mutation → `japanese_phrase_round_trip_wire_honored` +
  `round_trip_label_shows_wire_language` RED (Japanese phrase parsed under English fails). Restored → GREEN.
The wire byte is **load-bearing on both legs.**

### 2. Label-threading completeness — PASS
- **derive.rs** label sites `:248` (JSON language), `:249` (language_defaulted), `:261/:265` (text DEFAULT),
  `:273` (text non-default) ALL read `effective_lang` / `effective_lang_defaulted`. The hex/phrase arms set
  `(m, cli_lang, defaulted)` so `effective_*` is one consistent pair across all three source branches.
  `cli_lang.into()` at `:172` feeds only the hex/phrase `from_entropy_in`/`parse_in` (helper input), never a
  label. No raw `cli_lang.as_str()` / `defaulted` survives at any label site.
- **verify.rs:** `args.language` is consumed ONLY by the `match args.language` at `:61`; the two downstream
  consumers (`:110` `effective_lang.into()`, `:119` `effective_lang.as_str()`) both use the wire language.
  No orphaned `.into()`/`.as_str()` on the bare `Option`.
- **Empirical:** French card with no `--language` prints `language: french` (NOT `english (DEFAULT)`) and emits
  no bogus english-default note. Pinned by `french_card_labels_french_not_default`.

### 3. verify Err / exit-3 arms — PASS (preserved verbatim, empirically confirmed)
`verify.rs:76-88` matches `Result<(Tag,Payload),Error>`: the helper is invoked ONLY on the
`Ok((_tag, payload))` arm; the `Err(ReservedTagNotEmittedInV01 { got }) => { emit_future_format(&got, …)?; }`
exit-3 leg and the generic `Err(e) => return Err(e.into())` leg are byte-preserved. No whole-match swap.
- **Empirical:** a reserved-tag (`seed`) future-format string → `ms verify` prints
  `OK: valid future format (v0.2+, tag seed)` and **exits 3**. Pinned by `reserved_tag_still_exits_3`.
- No deadlock: the helper holds `stderr.lock()` while the exit-3 arm writes to **stdout** (`println!`), not
  stderr; the early `return` drops the lock naturally.

### 4. `--language` Option-ization ripple — PASS (funds-safe disagreement direction)
`verify.rs:31-32` is now `#[arg(long)] pub language: Option<CliLanguage>` (dropped `default_value="english"`).
`(cli_lang, defaulted) = match args.language { Some=>(l,false), None=>(English,true) }`.
- Bare verify of a non-English card → `defaulted=true` → helper emits NO note
  (`bare_no_flag_no_spurious_note`).
- `--language french` matching a French card → agreement → no note (`mnem_agreement_no_note`).
- `--language english` disagreeing with a French/Japanese card → advisory note, exit 0, proceeds with the
  **WIRE** language (correct fp) — `explicit_english_on_japanese_card_emits_note`,
  `explicit_wrong_language_wire_wins_with_note`. **Disagreement direction is funds-safe** (wire always wins;
  `--language` can never override the card's true language). No consumer of `args.language` broke (exactly two
  consumers, both re-pointed to `effective_lang`).

### 5. L5 leak — PASS (no leak in Debug OR Display; latent but defensively closed)
`error.rs:124-133` hand-rolls `impl Debug` delegating to `kind()`+`message()`; `friendly_codex32`
(`codex32_friendly.rs:27`) drops `InvalidChecksum.string` via `..`. Display (`:135`) already routes through
`message()`.
- **Mutation:** Debug body → `CliError::Codex32(c) => write!(f, "Codex32({c:?})")` → the L5 test went RED,
  printing the planted `ms1secret_…`. Restored → GREEN. The hand-rolled Debug is load-bearing.
- **Reachability:** genuinely **latent** — the only production error-emit site (`main.rs::emit_error`
  `:220-248`) uses `kind()`/`message()`/`details()` (JSON) and Display (text); no `{:?}` on `CliError` exists
  in production. The `error.rs:277` `{:?}` fallback Debug-prints `other`, but `ms_codec::Error::Codex32(c)` is
  explicitly handled at `:153` (→ `CliError::Codex32`) and never reaches that `_` arm, so the secret-bearing
  codex32 error cannot leak through it. Empirically, `ms verify <corrupted-secret-ms1>` prints only
  `BCH checksum invalid (short code)…` (text) / a sanitized JSON envelope — no echo.

### 6. L26 — PASS (arm-selective, stderr-only, language preserved)
`combine.rs:109-127` emits `non_english_seed_advisory` ONLY on the `CombineTo::Entropy` arm.
- **Empirical:** Japanese shares `--to entropy` → advisory
  (`warning: encoding a japanese BIP-39 seed as raw entropy …`) on stderr + correct hex on stdout; `--to phrase`
  and `--to ms1` → NO advisory.
- `--json --to entropy` → stdout `language: null` unchanged (advisory stderr-only; no wire-shape change) —
  `json_wire_shape_unchanged_advisory_on_stderr`.
- `--to ms1` re-encodes the `&payload` → round-trips to a Japanese `Mnem` carrying the same language byte
  (`japanese_to_ms1_preserves_language_byte`). English shares → no advisory.

### 7. The swapped test — PASS (sound; Entr-vs-Mnem DEFAULT-label split fully pinned)
`ms encode --language english` canonicalizes to an `Entr` card (English = universal default, no language byte),
so a constructible "English Mnem" doesn't exist. The implementer replaced that contrast with
`french_card_explicit_matching_language_no_default_label` (French Mnem + explicit `--language french`
= agreement, no note, no DEFAULT label). The DEFAULT split is fully pinned:
- **Entr English = DEFAULT:** `english_entr_card_default_label_preserved` (`(DEFAULT)` + english-default note).
- **Mnem (any non-English) = NOT DEFAULT:** the French/Japanese mnem tests (`effective_lang_defaulted==false`).
No coverage hole.

### 8. No regression — PASS
Full `cargo test -p ms-cli` = **215 passed / 0 failed / 5 ignored** (all 5 pre-existing env-gated mlock/g6
tests; none newly disabled). `cargo test -p ms-codec` = **150 / 0** (untouched, NO-BUMP intact). `cargo clippy
--all-targets -p ms-cli -- -D warnings` clean. encode/split/decode/inspect/repair exercised transitively by the
new combine/verify tests + the unchanged suite. mlock.rs unformatted; Cargo.toml/lock untouched.

---

## Verdict

**CYCLE-8 WHOLE-DIFF: 0C / 0I** — **GREEN (0C/0I, cleared to tag/publish).**

The H4/H5 funds-path uses ONLY the wire language byte (mutation-proven RED on both derive and verify when
flipped to `cli_lang`; empirically derives the correct French/Japanese fingerprints). All label sites read
`effective_lang`/`effective_lang_defaulted`. verify's exit-3 future-format leg is preserved verbatim
(empirically exits 3). The L26 advisory is arm-selective, stderr-only, with the `--to ms1` language byte
preserved. L5's hand-rolled Debug never echoes the secret in Debug OR Display (mutation-proven; latent but
closed). 215/0 ms-cli + 150/0 ms-codec + clippy clean. No mutations left in the tree.

Proceed to P6 (0.9.0 bump + CHANGELOG/README sweep + tag `ms-cli-v0.9.0` + `cargo publish -p ms-cli` +
bughunt-report H4/H5/L26/L5 tick).
