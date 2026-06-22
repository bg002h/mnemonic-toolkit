# R0 REVIEW — cycle-15 Lane M (ms-codec/ms-cli secret zeroize) — Round 1

**Reviewer:** opus architect (R0 mandatory gate).
**Spec under review:** `design/BRAINSTORM_cycle15m_ms_codec_secret_zeroize.md`.
**Sweep input:** `design/agent-reports/sweep-keymat-mnemonic-secret.md`.
**Source verified against:** `mnemonic-secret` `origin/master` @ `6f9f60bcfbf4` (HEAD; matches the spec's stated SHA).
**Lens:** secret-memory-hygiene = first-class bar; ms-codec is the BIP-39-entropy codec.

## VERDICT: GREEN (0 Critical / 0 Important)

Every load-bearing claim verified true against live source. The two priority axes —
(1) InspectReport-Debug-redaction and (2b) the toolkit-decoupling claim — are both
**correct**. Q1/Q2/Q3 are all **ratified as proposed**. Three Minor refinements below
are non-blocking hardening notes to fold into the plan-doc; none gates implementation.

---

## AXIS 1 — InspectReport exposure (D1, the headline) — VERIFIED CORRECT

- **(a) Current surface — CONFIRMED.** `inspect.rs:34 #[derive(Debug, Clone)]`,
  `:35 #[non_exhaustive]`, `:36 pub struct InspectReport`, `:48 pub payload_bytes: Vec<u8>`,
  `:55 pub language: Option<u8>`. No `PartialEq` derived (only `Debug, Clone`) — spec's
  "keep PartialEq underived, no new constraint" is correct. `lib.rs:56 pub use
  inspect::{inspect, InspectKind, InspectReport};` — public re-export confirmed.
- **(b) RULE Z-DEBUG / hand-rolled redacting Debug is MANDATORY — CONFIRMED against the
  vendored zeroize.** `Cargo.lock` pins `zeroize 1.8.2`. In that version,
  `zeroize-1.8.2/src/lib.rs:622 #[derive(Debug, Default, Eq, PartialEq)]` over
  `:623 pub struct Zeroizing<Z: Zeroize>(Z);` — a **derived** Debug on a newtype tuple,
  which forwards to the inner type. There is **no** explicit/redacting `impl Debug for
  Zeroizing` (grep for `Debug for Zeroizing` = 0 hits). Therefore `{:?}` of a
  `Zeroizing<Vec<u8>>` prints `Zeroizing([222, 173, ...])` — the raw bytes. The spec's
  claim that the `Zeroizing` wrap gives scrub-on-drop but NOT Debug-redaction, so the
  hand-rolled redacting Debug is mandatory (not optional), is **exactly right**.
- **(c) Deref keeps ms-cli readers compiling — CONFIRMED.** Consumers in
  `ms-cli/src/cmd/inspect.rs`: `:160 report.payload_bytes.len()`, `:166
  report.payload_bytes.len().saturating_sub(1)`, `:217 hex::encode(&report.payload_bytes)`,
  `:247 hex::encode(&report.payload_bytes)`. All are `.len()` (auto-deref) or
  `&report.payload_bytes` (Deref coercion `&Zeroizing<Vec<u8>>` → `&Vec<u8>` → `&[u8]`).
  All compile unchanged under `Zeroizing<Vec<u8>>`. (Spec's cited reader line numbers
  are off-by-a-few — it says `:160,166,217,247`; live is `:160,166,217,247` — match.)
- **(d) Redacted-Debug field visibility — CONFIRMED structurally non-secret.** The
  non-`payload_bytes` fields are `hrp`, `threshold`, `tag`, `share_index`, `prefix_byte`,
  `checksum_valid`, `kind`, `language: Option<u8>` (the language *index*, not entropy).
  None re-encodes entropy: the `Mnem` language byte that is *also* `payload_bytes[0]`
  is surfaced as the parsed index `language`, which is structural metadata (wordlist id),
  not secret. Keeping these visible and redacting only `payload_bytes` is correct.
- **(e) Design A is genuinely smallest blast radius.** B (newtype) adds a new exported
  type + trait impls with no ergonomic win over A's Deref; C (field removal + accessor)
  is a strictly larger break (every `report.payload_bytes` → `report.payload_bytes()`)
  and an `&[u8]` accessor doesn't even solve the Debug class. Rejection rationale for
  B/C ratified. **Design A confirmed.**

## AXIS 2 — SemVer + downstream corrections (Q3) — ALL FOUR VERIFIED

- **(a) Versions — CONFIRMED.** `ms-codec/Cargo.toml:3 version = "0.5.0"`;
  `ms-cli/Cargo.toml:3 version = "0.9.0"`; `ms-cli` pin `:20 ms-codec = { path =
  "../ms-codec", version = "=0.5.0" }`. So **ms-codec 0.5.0 → 0.6.0 MINOR**,
  **ms-cli 0.9.0 → 0.10.0 MINOR**, re-pin to `=0.6.0`. The brief's "0.39 → 0.40" was
  stale; the §0.1 correction is right.
- **(b) Toolkit-decoupling — CONFIRMED (THE load-bearing claim).** `grep -rn ms_codec
  crates/mnemonic-toolkit/src | grep -iE 'inspect|InspectReport|InspectKind'` = **0 hits**.
  The toolkit's own `inspect-ms1` path is `cmd/inspect.rs:171 let (tag, payload) =
  ms_codec::decode(chunks[0])?;` → goes through `decode()` → `Payload`, NOT `inspect()`.
  All toolkit ms-codec usage is `Payload` / `decode()` / `decode_with_correction()`
  (`slot_ms1.rs`, `language.rs`, `cmd/ms_shares.rs`, `synthesize.rs`, `repair.rs`).
  `decode()` signature `pub fn decode(s) -> Result<(Tag, Payload)>` is byte-stable;
  slug #2's clone removal returns the same `(Tag, Payload)` shape. ⇒ The InspectReport
  reshape is toolkit-INVISIBLE; the lanes are NOT source-coupled. Toolkit pins
  `ms-codec = "0.5"` (crates.io caret dep, `Cargo.lock` 0.5.0) → Lane-T bump is
  `"0.5"`→`"0.6"` + `cargo update` + `cargo test -p mnemonic-toolkit`, no source change.
  **The §0.2 correction is correct and the lane independence holds.**
- **(c) CI = clippy-gated, NOT fmt-gated — CONFIRMED.** `.github/workflows/rust.yml:143
  clippy:` / `:153 cargo clippy --all-targets -p ms-cli -- -D warnings`. No `cargo fmt`
  / `rustfmt --check` step anywhere in `rust.yml`. (`fuzz-smoke.yml` not present / no
  match.) The §0.3 correction is right: gate on clippy `-D warnings`, run fmt as
  courtesy only. Note for impl: removing the `.clone()` in #2 should *quiet* (not trip)
  `clippy::redundant_clone`; watch for needless-borrow on the new Zeroizing wraps.
- **(d) md/mk don't consume ms-codec — CONFIRMED.** `git grep ms_codec\|ms-codec
  origin/master` on `descriptor-mnemonic` and `mnemonic-key` = no matches. No impact.

## AXIS 3 — other slugs — VERIFIED

- **#2 decode clone — CONFIRMED.** `decode.rs:82-83` (Entr) `let scrubbed:
  Zeroizing<Vec<u8>> = Zeroizing::new(data); let p = Payload::Entr((*scrubbed).clone());`
  and `:89-90` (Mnem) identical pattern. The clone-into-bare-Vec is real; moving `data`
  straight into `Payload::Entr(data)` is strictly fewer copies and byte-identical wire.
  Lint re-anchor needed: `ms-codec/tests/lint_zeroize_discipline.rs:49-51` row anchors
  on `"let scrubbed: Zeroizing<Vec<u8>>"` and `:81` asserts row-count 5 — both must be
  re-anchored/decremented per the spec's §4#2 + §6. Negative-anchor (assert no
  `(*scrubbed).clone()`) is the right replacement.
- **#3 share strings — CONFIRMED PARTIAL is correct.** `shares.rs:18 use codex32::
  {Codex32String, Fe};` `:130 secret_s = Codex32String::from_seed(...)` (full secret),
  `:136 defining: Vec<Codex32String>`, `:148 distributed: Vec<String>`, `:115 single`,
  `:195/:210 parsed: Vec<Codex32String>`, `:280-281 secret = ...interpolate_at`. The
  reachable `Vec<u8>` buffers ARE already wrapped: `:139 filler: Zeroizing<Vec<u8>>`,
  `:301 let data: Zeroizing<Vec<u8>> = Zeroizing::new(secret.parts().data());`.
  `Codex32String` is a `String`-newtype in `codex32-0.1.0` with no Drop/Zeroize and the
  crate is dormant — cannot wrap its internal String without vendor/fork. The spec's
  enumerate-and-defer (keep FOLLOWUP `open`, no false GREEN, no passing String-scrub
  test) is the honest call. **Cannot be done cleanly in-repo without forking codex32 —
  holding the String leg with a documented reason is correct.** (See Q2 below.)
- **#5 inspect intake — CONFIRMED.** `ms-cli/src/cmd/inspect.rs:33 let ms1 =
  read_input(args.ms1.as_deref())?;` — bare String, the lone unwrapped ms1-intake.
  Wrap in `Zeroizing<String>`. Add ms-cli lint row.
- **#6 RepairDetail — CONFIRMED, and the Debug-derive is PRESENT (firm, not
  conditional).** `repair.rs:62 #[derive(Debug, Clone)]` over `:63 struct RepairDetail`
  with `:65 original_chunk: String`, `:66 corrected_chunk: String`; `:75 let original =
  read_input(...)`; `:89 original.clone()`; `:90 corrected_chunk.clone()`; `:94 vec!
  [corrected_chunk]`. The spec §4#6 phrases this as a *conditional* "Watch ... if a
  #[derive(Debug)] is present, redact" — the derive IS present (line 62), so the
  hand-rolled redacting Debug on `RepairDetail` is **mandatory**, same class as #1.
  See Minor-1.
- **#7 Xpriv — CONFIRMED.** `derive.rs:217 seed: Zeroizing<[u8;64]>` + `:218 mlock pin`
  (good); `:220 master = Xpriv::new_master(...)`; `:233 acct_xpriv =
  master.derive_priv(...)`; `:236 Xpub::from_priv`. `bitcoin::bip32::Xpriv` has no
  Zeroize → upstream-blocked. Lifetime-min + new `rust-bitcoin-xpriv-zeroize-upstream`
  FOLLOWUP (PARTIAL) is right; cannot fully close.
- **#8 JSON structs — CONFIRMED, and RULE Z-DEBUG resolves CLEAN.** Every `*Json`
  struct in `format.rs` derives **only `Serialize`** (no Debug on any: confirmed by
  grep, 0 Debug derives). So the wrap is pure defense-in-depth with no Debug-leak risk.
  Secret-bearing OWNED fields: `EncodeJson.entropy_hex:String (:61)`,
  `DecodeJson.entropy_hex/.phrase (:100-101)`, `CombineJson.entropy_hex/.phrase/.ms1
  (:86-89)`, `SplitJson.shares:Vec<String> (:69)`, `InspectReportJson.payload_bytes_hex
  (:129)`. Several structs are borrowing (`<'a>` + `&'a str`) — wrap at the OWNER, as
  the spec notes. The repair-JSON `RepairReportJson`/detail structs (`repair.rs:190-206`)
  derive only `Serialize` too — same clean posture.
- **#9 verify to_string temp — CONFIRMED, including the lint false-GREEN.**
  `verify.rs:170 let word_count = _mnemonic.to_string().split_whitespace().count();` in
  `emit_round_trip_ok` (`:169`) — bare full-phrase temp. The main compare path
  (`emit_simple_ok` route) IS wrapped: `:116 supplied_str: Zeroizing<String>`, `:117
  derived_str: Zeroizing<String>`. The ms-cli lint row `lint_zeroize_discipline.rs:87-88`
  labeled "verify success-log derived_mnemonic.to_string() wrapped" anchors on the
  substring `"let derived_str: Zeroizing<String>"` — which exists at `:117` — so the
  lint reads GREEN while the `:170` temp is unwrapped. The spec correctly flags this and
  the fix (count off the already-wrapped value, or wrap the temp + re-point the lint
  row). Row-count at `:102` ("expected 10") bumps with the new ms-cli rows.

## AXIS 4 — wire-format invariant — CONFIRMED GUARDED
Every change is in-memory lifetime/scrub hygiene + the in-process `InspectReport` /
`*Json` struct shapes. `decode()`/`encode()` BCH wire path untouched; `Payload` shape
unchanged (slug #4 deferred). The §5 byte-identity guard test + §8 statement cover it.
No change touches `parse_ms1_symbols` / envelope wire bytes. Confirmed.

## AXIS 5 — publish chain + lint — CONFIRMED
ms-codec → crates.io → bump ms-cli pin `=0.6.0` → ms-cli → crates.io. Two zeroize
lints to update: `ms-codec/tests/lint_zeroize_discipline.rs` (re-anchor decode row #2,
decrement row-count `:81`); `ms-cli/tests/lint_zeroize_discipline.rs` (add #5/#6/#9
rows, bump row-count `:102`). No fmt lint exists. clippy `-D warnings` is the real gate.
No manual / gui-schema-mirror update (no clap flag/subcommand/help change).

## Q1 / Q2 / Q3 — RATIFIED
- **Q1 (slug #1 redacted-Debug visibility): RATIFY AS PROPOSED.** Keep
  `language`/`prefix_byte`/`kind`/`hrp`/`threshold`/`tag`/`share_index`/`checksum_valid`
  visible (structural, non-secret); redact only `payload_bytes`. No diagnostic field
  re-encodes entropy (verified: `language` is the parsed wordlist index, not the byte).
- **Q2 (slug #3 codex32 vendor/fork): RATIFY HOLD.** The String leg genuinely cannot be
  closed in-repo — `Codex32String` is a foreign `String`-newtype with no Drop, and the
  upstream crate is dormant. Vendoring the BCH/Shamir secret-sharing math is a large,
  separately-gated maintenance decision out of scope for an in-memory-hygiene cycle.
  Holding the String leg of #3 with a documented reason (FOLLOWUP stays `open`,
  description → "enumerated; bound to vendor/fork decision") is the correct, honest
  call. No false GREEN.
- **Q3 (framing corrections): RATIFY ALL THREE.** ms-codec 0.5.0 (not 0.39), toolkit
  consumes only `Payload`/`decode()` (not `inspect()`), CI is clippy-only (no fmt gate)
  — all three verified true. The SemVer numbers, the narrowed downstream flag, and the
  gate story all stand.

---

## MINOR (non-blocking — fold into the plan-doc; do NOT gate impl)

- **Minor-1 — make #6's RepairDetail redacting Debug a FIRM requirement, not a "Watch."**
  `repair.rs:62` derives `Debug` on `RepairDetail`; once `original_chunk`/`corrected_chunk`
  become `Zeroizing<String>`, the derived Debug forwards to `String::fmt` and leaks the
  secret chunk (RULE Z-DEBUG). The spec §4#6 phrases this conditionally ("if a derive is
  present"). The derive IS present — so the plan-doc must specify: drop `#[derive(Debug)]`
  on `RepairDetail` and hand-roll a redacting Debug (same as #1), with a Debug-no-echo RED
  test for `RepairDetail`. (Note `RepairDetail` is internal/`pub(crate)`-ish — not a
  public-API break — but the Debug leak is real if anything `{:?}`-prints a detail.)
  Alternatively, since `RepairDetail`'s only consumers are `emit_text`/`emit_json` (which
  borrow by `&str`, never `{:?}`), the team MAY instead drop the `Debug` derive entirely
  (no redacting impl needed) if nothing requires it — confirm no test/log `{:?}`s a
  RepairDetail. Either way: the derived Debug over a Zeroizing-wrapped secret field must
  not survive. Add the marquee Debug-redaction test pattern here too.

- **Minor-2 — slug #3 lifetime-min wording: `secret_s` is MOVED, not held.**
  `shares.rs:130` binds `secret_s` then `:137 defining.push(secret_s)` moves it into
  `defining[0]`. So the "drop `secret_s` early" framing is slightly off — `secret_s`'s
  lifetime is already minimal (immediately consumed into `defining`). The real residual
  String surface is the `defining`/`distributed`/`parsed` vectors and the recovered
  `secret` (`:281`). The plan-doc's lifetime-min comment should target the vector
  bindings (drop `defining`/`parsed` as soon as `distributed`/`secret` is produced),
  not a re-scope of the already-moved `secret_s`. Cosmetic — the enumeration is correct;
  only the "drop early" target needs precision.

- **Minor-3 — #9 lint row: prefer re-pointing the EXISTING row over adding a new one.**
  The ms-cli lint already HAS a "verify success-log ... to_string() wrapped" row
  (`:87-88`) whose anchor (`derived_str` at `:117`) passes against the wrong site,
  masking the `:170` gap. The cleanest fix is to (a) fix the source at `:170` (count off
  the wrapped value or wrap the temp) AND (b) re-point that existing row's evidence
  anchor at the `emit_round_trip_ok` site so it actually guards `:170` — rather than
  adding a *new* row alongside a misleading one. This keeps the row-count delta honest.
  Spec §5/§6 already gestures at "extend the verify row" — make it explicit that the
  existing row's anchor is repaired, not just appended to.

---

## Implementation-gate note
Per the project full-suite rule (feedback_r0_review_run_full_package_suite): per-phase
R0/impl reviews MUST run the FULL `cargo test -p ms-codec` and `cargo test -p ms-cli`
suites (not targeted `--test` targets) — the lint re-anchors (#2 row-count, ms-cli
row-count) and the existing verify-row false-GREEN are exactly the cross-cutting class
that targeted runs miss.

**GATE: GREEN — implementation (plan-doc, then TDD) may proceed.** The three Minors are
plan-doc refinements, not blockers.
