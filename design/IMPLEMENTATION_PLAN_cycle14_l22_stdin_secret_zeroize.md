# IMPLEMENTATION PLAN — cycle-14: close L22 (stdin secret read into un-scrubbed `String`)

**Status:** DESIGN ONLY — this plan-doc feeds its **own** mandatory R0 loop (plan R0 → GREEN → TDD). NO code yet.
**Upstream:** the R0-GREEN spec `design/BRAINSTORM_cycle14_l22_stdin_secret_zeroize.md` (R0 GREEN round 2 — `design/agent-reports/cycle14-spec-r0-round2-review.md`). The spec is the source of truth; this plan operationalizes it into RED-first phases and re-pins every citation live.
**SemVer:** toolkit **MINOR v0.67.0** (off `v0.66.0`). md/ms/mk codecs + CLIs **NO-BUMP**. GUI: **NO `schema_mirror` impact** (no clap flag/dropdown/subcommand added/removed/renamed). NO manual mirror. NO cross-repo.
**Source SHA verified against:** `origin/master = 82c61e76950df64bb9cf44e1e0f1b173de254346` (`design(cycle-14): secret-memory-hygiene tail recon — collapses to L22 only`; toolkit `Cargo.toml` version `0.66.0`). **Every line number below was re-grepped via `git show origin/master:<path> | grep -n` / `git grep -n <pat> 82c61e76` at write time.** The working tree is on the v0.60.0 own-account branch — working-tree line numbers were NOT trusted; the implementer MUST branch off `origin/master` and re-grep again at implement time (citations decay every merge).

> **MANDATORY R0 GATE (CLAUDE.md).** No implementation — no code, no implementer dispatch — until THIS plan-doc passes an opus architect R0 review converged to **0 Critical / 0 Important**. Fold → persist the review verbatim to `design/agent-reports/` → re-dispatch → repeat until GREEN (the reviewer-loop continues after every fold; folds introduce drift). The gate then re-applies per-phase during TDD. R0 reviews MUST run the **full `cargo test -p mnemonic-toolkit` suite** (argv/mlock/zeroize sibling lints fire outside any one `--test` target — MEMORY `feedback_r0_review_run_full_package_suite`).

---

## 0. Re-pinned citation table (all vs `82c61e76`)

| Symbol / site | Live location (`82c61e76`) | Disposition |
|---|---|---|
| `SlotInput` derive | `src/slot_input.rs:96` `#[derive(Debug, Clone, PartialEq, Eq)]` | **keep all 4** (now satisfied by `SecretString`) |
| `SlotInput` struct | `src/slot_input.rs:97` | — |
| `SlotInput.value` field | `src/slot_input.rs:100` `pub value: String` | **MIGRATE → `SecretString`** |
| `parse_slot_input` ctor write | `src/slot_input.rs:182-185` `Ok(SlotInput { … value: value.to_string() })` | **PRODUCTION write → `SecretString::new(value.to_string())`** (see §2 — beyond the spec's named census) |
| `is_stdin_sentinel` | `src/slot_input.rs:109-110` `self.subkey.is_secret_bearing() && self.value == "-"` | `self.value == "-"` → `&*self.value == "-"` |
| `apply_slot_stdin` reader | `src/slot_input.rs:203-232`; transient `let mut buf = String::new();` `:215`; **field write `slots[stdin_idxs[0]].value = buf;` `:225`** | wrap write: `… = SecretString::new(buf);` (literal differs from spec's `slots[i].value` shorthand — it is `slots[stdin_idxs[0]]`) |
| `slot()` test helper | `src/slot_input.rs:379-383` `value: value.to_string()` | `value: SecretString::new(value.to_string())` |
| `s()` test helper | `src/bundle_unified.rs:123-124` (`#[cfg(test)] mod tests`) `value: v.to_string()` | `value: SecretString::new(v.to_string())` (see §2) |
| `@env:` write-back ×3 | `cmd/bundle.rs:2629`, `cmd/import_wallet.rs:1396`, `cmd/verify_bundle.rs:1883` — all `s.value = resolve_env_var_sentinel(&s.value, &flag)?;` | `s.value = SecretString::new(resolve_env_var_sentinel(&s.value, &flag)?);` — **each is itself L22 secret residue the wrap closes** |
| phrase-overlay clone | `cmd/import_wallet.rs:1229-1234` `Vec<(u8, String)>` … `.map(\|s\| (s.index, s.value.clone()))` (filtered `SlotSubkey::Phrase` `:1232`) | `.map(\|s\| (s.index, s.value.to_string()))` (min; §3-D1 decides whether to make the Vec `Vec<(u8, SecretString)>`) |
| `FromInput.value` | `cmd/convert.rs:131-133` `pub struct FromInput { … pub value: String }` | **DOES NOT MIGRATE** — `from.value == "-"` etc. unchanged |
| `read_stdin_to_string` | `cmd/convert.rs:745-751` `pub(crate) fn … -> Result<String, ToolkitError>` (`Ok(buf.trim()…)` `:750`) | **STAYS `String`** (D1) |
| `read_stdin_passphrase` | `cmd/convert.rs:758-769` `pub(crate) fn … -> Result<String, ToolkitError>` (`Ok(buf)` `:769`) | **STAYS `String`** (D1) |
| 14 `Zeroizing::new(read_stdin_*)` callers | (re-grepped = **14**) `derive_child.rs:140`, `electrum_decrypt.rs:115`, `final_word.rs:68`, `import_wallet.rs:2300`, `ms_shares.rs:267,374`, `seed_xor.rs:172,302`, `seedqr.rs:178,245`, `silent_payment.rs:242`, `slip39.rs:304,424,607` | **UNTOUCHED** (readers stay `String` → no `Zeroizing<Zeroizing<String>>`) |
| convert handler locals | `cmd/convert.rs` `effective_passphrase: Option<String>` `:853`, `effective_bip38_passphrase: Option<String>` `:861`, `primary_value` `:868`; consumers `.as_deref()` `:990-991`, `&primary_value` `:994`, `split_whitespace`/`vec![primary_value.clone()]` `:1020-1025`, `Some(primary_value.as_str())` `:1144`; mlock pins `:880-886` | **P2 local-wrap → `Zeroizing<String>`** (keep mlock pins) |
| restore stdin locals | `cmd/restore.rs` `passphrase: String` `:399,829,1291,…`, `from_value: String` `:410,839,…` (`read_stdin_*` `:399-411,829-840,1291-1300,3045-3053`) | **P2 (conditional, §3-D1 default = include) → `Zeroizing<String>`** |
| addresses stdin locals | `cmd/addresses.rs` `passphrase: String` `:149`, `from_value: String` `:159` (`read_stdin_*` `:150,160`) | **P2 (conditional, default include) → `Zeroizing<String>`** |
| `cfg(fuzzing)` mod gating | `src/main.rs:30` `mod slot_input;` (private bin mod); `src/lib.rs:177-178` `#[cfg(fuzzing)] pub mod slot_input;`; **no `pub use SlotInput`** | MINOR by v0.10.1 precedent, NOT public reachability (D3) |
| `env_sentinel` signature | `src/env_sentinel.rs:56-59` `pub(crate) fn resolve_env_var_sentinel(…) -> Result<String, ToolkitError>` | confirms the `@env:` re-wrap needs explicit `SecretString::new(…)` |
| `secret_string.rs` | `src/secret_string.rs:22-23` `#[derive(Clone)] pub struct SecretString(Zeroizing<String>)`; `Deref` `:32`, `Display` `:39`, redacting `Debug` `:46-47`, `Serialize` `:52`; tests `debug_redacts_the_secret` `:100`, `serializes_byte_identically` (T-B1) | **EXTEND with `PartialEq`/`Eq`** (P1) — no derive on it today |
| zeroize version | `Cargo.lock` resolves `zeroize 1.8.2`; `zeroize-1.8.2/src/lib.rs:622-623` `#[derive(Debug,Default,Eq,PartialEq)] pub struct Zeroizing<Z>(Z)` + `Deref<Target=Z>` `:660` | confirms raw `Zeroizing<String>` `Debug` LEAKS → `SecretString` chosen (D2) |
| lint floor / patterns | `tests/lint_zeroize_discipline.rs:452` `SECRET_FILE_FLOOR = 35`; patterns `:426-430` incl. `"SecretString::new("` `:428`, `": SecretString"` `:430`; row-count assert `(18..=60)` `:375` (stale `(24..=35)` doc `:370`); `secret_string.rs` PRIMITIVE-allowlist `:442`; source→declared gate `every_secret_bearing_src_file_is_declared_or_allowlisted` `:482`; floor assert `:516` | **bump `35→36` (`:452`)** + **add `slot_input.rs` row** (D4) |
| version-sites (ship) | `crates/mnemonic-toolkit/Cargo.toml:3`; `README.md:13`; `crates/mnemonic-toolkit/README.md:9`; `scripts/install.sh:32`; `CHANGELOG.md` (new top entry); `fuzz/Cargo.lock:575`; `Cargo.lock:727` | all `0.66.0 → 0.67.0` |
| L22 report anchor | `design/agent-reports/constellation-bughunt-2026-06-20.md:850` `### - [ ] L22` (L16 won't-fix at `:621`) | **tick at ship — RE-GREP `:850`** |
| Cycle-B Site-1 / parent | `design/FOLLOWUPS.md:1207` `secret-memory-hygiene-cycle-b`; Site-1 list `:1211` (names `convert.rs:668+` for the 3 locals — live = `:853/:861/:868`) | **flip Site-1 *scrub* leg status at ship** |

> **PLAN R0 MUST re-verify, against the resolved `zeroize` version + live source:** (i) `zeroize 1.8.2` `Zeroizing` derives `Debug` non-redacting (re-grep `Cargo.lock` + `~/.cargo/.../zeroize-1.8.*/src/lib.rs:622`); (ii) no security-sensitive `SecretString` equality consumer exists (grep `SecretString` `==`/`PartialEq` consumers — today none, all equality is test `assert_eq!` or the public `"-"` sentinel); (iii) the lint partition count with `slot_input.rs` added (run the scan; confirm `>= 36`).

---

## 1. Execution model

- **Single implementer subagent** (NOT parallel re-implementations — CLAUDE.md per-phase pattern step 3), in a **toolkit worktree off `origin/master = 82c61e76`** (e.g. `git worktree add ../wt-cycle14 origin/master -b feature/cycle14-l22-stdin-secret-zeroize`). Re-grep all §0 line numbers in-worktree before editing (they may have moved).
- **TDD, RED-first:** for each phase, write the failing test(s) FIRST, confirm RED, then the minimal impl to GREEN. Tests live in the **BIN target** → run `cargo test -p mnemonic-toolkit`.
- **NEVER `cargo fmt` the toolkit** (`mlock.rs` is permanently fmt-exempt — MEMORY `project_g6_fmt_exemption_and_asymmetric_pin`). Hand-format new code to match surrounding style.
- **Per-phase gate (BOTH, every phase):** FULL `cargo test -p mnemonic-toolkit` **AND** `cargo clippy --workspace --all-targets -- -D warnings`. The argv-secret / mlock / zeroize sibling lints fire outside any one `--test` target — a targeted run is insufficient (MEMORY `feedback_r0_review_run_full_package_suite`). A phase is not GREEN until both pass.
- **Per-phase architect review** persisted verbatim to `design/agent-reports/cycle14-phase-N-<round>-review.md` BEFORE the fold-and-commit step; reviewer-loop to 0C/0I per phase.
- **Stage paths explicitly** (no `git add -A`). Commit-trailer: `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.

---

## 2. Census correction surfaced at plan-write time (NOT in the GREEN spec — PLAN R0 to confirm)

The spec's edit census (§2.6(b) table) enumerates the `apply_slot_stdin` write, the 3 `@env:` write-backs, the `:1233` clone, and the `slot()` test helper. Re-grepping **all** `SlotInput { … }` struct-literal construction sites (`git grep -nE 'SlotInput \{' 82c61e76 -- crates/mnemonic-toolkit/src`) finds **three** `value:`-writing constructions, two of which the spec did not name explicitly:

1. **`src/slot_input.rs:182-185` — `parse_slot_input` (PRODUCTION).** `Ok(SlotInput { index, subkey, value: value.to_string() })`. This is the **central construction path** — every parsed `--slot` flows through it. With `value: SecretString`, `value: value.to_string()` (a `String`) no longer type-checks → **MUST become `value: SecretString::new(value.to_string())`**. This is a hard compile-fence edit, not optional, and the most load-bearing single edit of P1 (it is where the stdin/`@env:`/literal secret first enters the field-type). **The spec's `is_stdin_sentinel` + `apply_slot_stdin` framing implicitly relies on this site but never names it.**
2. **`src/bundle_unified.rs:123-124` — `s()` test helper** (`#[cfg(test)] mod tests`). `value: v.to_string()` → `value: SecretString::new(v.to_string())`. A compile-fence in the `bundle_unified` test module.
3. `src/slot_input.rs:379-383` — `slot()` test helper (spec DOES cover this).

**Plan disposition:** P1's edit list includes `parse_slot_input:185` and `bundle_unified.rs:124` alongside the spec's named sites. They are pure compile-driven edits (no behavior change), fully consistent with D2; flagging here so PLAN R0 confirms the census is now complete (no 4th hidden construction). This is the analogue of the spec-R0 round-1 I-1 fold (census completeness), one layer deeper.

---

## 3. Resolved scope decisions carried from the spec (re-affirmed)

- **D1 — fix-at-owned-allocation; readers stay `String`.** Wrap the persistent field `SlotInput.value` + the convert handler locals; do NOT flip `read_stdin_passphrase`/`read_stdin_to_string` (the 14 already-wrapping callers would become `Zeroizing<Zeroizing<String>>` — verified 14 sites). **restore/addresses local wraps: default = INCLUDE in P2** (same single `let x: Zeroizing<String> = if … else …` shape, no caller fan-out; completes toolkit Site-1 uniformly). PLAN R0 may elect to DEFER them to FOLLOWUP `restore-addresses-stdin-local-zeroizing` (acceptable — pinned + short-lived). **Decision recorded for R0:** include.
- **D1 sub-decision — `phrase_overlays` Vec type (`import_wallet.rs:1229-1234`).** Minimum: `.value.to_string()` restores compile without regressing today's hygiene (the phrase is already a bare `String` in the Vec today). **Recommended:** keep the minimum (`.to_string()`) for P1 to avoid touching `apply_seed_overlay`'s `&[(u8, String)]` signature (a non-trivial fan-out); file FOLLOWUP `phrase-overlay-secretstring` if R0 wants the overlay Vec itself scrubbed. **Decision recorded for R0:** minimum `.to_string()`; FOLLOWUP for the deeper wrap.
- **D2 — `SlotInput.value: SecretString`** (extend the existing `secret_string.rs` newtype, **NOT** raw `Zeroizing<String>` whose derived `Debug` LEAKS — verified zeroize 1.8.2). Add **plain (non-constant-time) `PartialEq`/`Eq`** (safe: equality is test-only + the public `"-"` sentinel — no auth/timing boundary); keep the existing length-only redacting `Debug`. Keep `#[derive(Debug, Clone, PartialEq, Eq)]` on `SlotInput` (all four satisfied by `SecretString`). `is_stdin_sentinel`: `self.value == "-"` → `&*self.value == "-"` (Deref to `str`; no extra `PartialEq<str>` impl). Prefer **D2-i (extend `SecretString`)** over D2-ii (new local newtype) — reuse; adding `PartialEq`/`Eq` to a `pub` type is additive (no breakage).
- **D3 — MINOR v0.67.0** by the v0.10.1 `cfg(fuzzing)`-gated precedent (`resolved-slot-derived-account-zeroizing-field`), NOT public-API reachability (`slot_input` is a private bin `mod` `main.rs:30`; `pub mod` only under `#[cfg(fuzzing)]` `lib.rs:177-178`; no `pub use SlotInput`). Readers stay `pub(crate) String` → no public-signature change.
- **D4 — lint:** add the mandatory `slot_input.rs` row; bump `SECRET_FILE_FLOOR 35→36` (`:452`); the `convert.rs` doc row is **redundant** (`convert.rs` already has rows; spec-R0 said R0 may collapse) — **plan disposition: OMIT the redundant `convert.rs` doc row**; rely on `convert.rs`'s existing rows (their evidence `"Zeroizing<String>"`/`"Zeroizing::new"` already matches the P2 local wraps). Do NOT touch the stale `:370` doc comment (out of scope, pre-existing).

---

## 4. Phased plan

Each phase is independently **RED → GREEN** and ends GREEN on BOTH the full `-p` suite and `clippy --workspace --all-targets -D warnings`, followed by a persisted per-phase review.

### P1 — `SecretString` traits + `SlotInput.value` migration + all owned-allocation edits + lint (the core)

**RED first** (write/confirm failing, then impl):

- **T2 (RED in `secret_string.rs` tests):** `assert_eq!(SecretString::new("a".into()), SecretString::new("a".into()))` + `assert_ne!(…, SecretString::new("b".into()))` — RED until `PartialEq`/`Eq` land. Keep `debug_redacts_the_secret` (`secret_string.rs:100`) GREEN (proves option-(a)'s leak is avoided: `format!("{:?}", SecretString::new("supersecret".into()))` must NOT contain the secret) and the T-B1 `serializes_byte_identically_to_string` GREEN.
- **T1 (RED, `slot_input.rs` tests):** `fn _assert_value_is_secret_string(s: &SlotInput) -> &SecretString { &s.value }` — compile-fences the field migration. Plus a `Deref<Target=str>` round-trip assertion (`&*slot.value == "expected"`). RED before the field flips.
- **T3 (RED if regressed, `slot_input.rs` tests):** `parse_slot_input("@0.phrase=-").unwrap().is_stdin_sentinel()` is `true`; `@0.xpub=-` is `false`. Fences the `== "-"` → `&*self.value == "-"` rewrite. (The existing `is_stdin_sentinel` tests at `slot_input.rs:506,567` already exercise this — they MUST stay GREEN.)
- **T4 (RED, end-to-end stdin `=-`):** drive `apply_slot_stdin` with a `Cursor` over `b"correct horse\n"` on a `@0.phrase=-` slot; assert `&*slot.value == "correct horse"` (newline stripped) — identical to today. Plus a CLI-level `bundle --slot @0.phrase=-` stdin-pipe smoke producing byte-identical output. Fences the `slots[stdin_idxs[0]].value = SecretString::new(buf)` edit at `:225`.
- **T4b (RED, `@env:` write-back ×3):** set an env var to a phrase; drive the `@env:` resolution (bundle / import-wallet / verify-bundle) on a secret-bearing `@N.phrase=@env:VAR` slot; assert the resolved `&*s.value` Derefs to the env-var phrase — byte-identical to today. Fences `s.value = SecretString::new(resolve_env_var_sentinel(…)?)` at `bundle.rs:2629`, `import_wallet.rs:1396`, `verify_bundle.rs:1883` (bare `s.value = <String>` won't compile against `SecretString`).
- **T4c (RED, overlay clone):** drive the `phrase_overlays` collection on a `SlotSubkey::Phrase` slot; assert the overlay still carries the phrase. Fences `.value.clone() → .value.to_string()` at `import_wallet.rs:1233`.
- **T5 (must stay GREEN):** the existing `parse_slot_input(…).unwrap() == slot(…)` comparisons (`slot_input.rs:483,503,513,521,528,535,542,549,559,566,576,582,…`) prove `SlotInput: PartialEq + Debug` survived via `SecretString`'s new impls. The `slot()` helper (`:379-383`) updates to `value: SecretString::new(value.to_string())`.
- **T7 (must go GREEN):** `every_secret_bearing_src_file_is_declared_or_allowlisted` + `every_canonical_zeroize_row_has_evidence_anchor` pass with the new `slot_input.rs` row; `secret_files.len() >= 36`.
- **T6 (documented, not asserted):** drop-scrub is structurally guaranteed by `Zeroizing<String>`'s `Drop`, evidenced by the type (T1) + the lint row — NOT by a flaky post-drop memory probe (post-drop read is UB). Mirror v0.10.1/v0.33.3.

**Impl (minimal to GREEN):**
1. `src/secret_string.rs`: add `impl PartialEq for SecretString { fn eq(&self, o: &Self) -> bool { self.0 == o.0 } }` (plain) + `impl Eq for SecretString {}`. Keep `Debug`/`Display`/`Deref`/`Serialize`/`Clone` unchanged. (`secret_string.rs` is PRIMITIVE-allowlisted `:442` — no lint row needed for the trait additions.)
2. `src/slot_input.rs`: `pub value: String` → `pub value: SecretString` (`:100`); `use crate::secret_string::SecretString;`. Keep `#[derive(Debug, Clone, PartialEq, Eq)]` (`:96`).
3. `src/slot_input.rs:182-185` (**`parse_slot_input` PRODUCTION** — §2): `value: value.to_string()` → `value: SecretString::new(value.to_string())`.
4. `src/slot_input.rs:225` (`apply_slot_stdin`): `slots[stdin_idxs[0]].value = buf;` → `… = SecretString::new(buf);`.
5. `src/slot_input.rs:110` (`is_stdin_sentinel`): `self.value == "-"` → `&*self.value == "-"`.
6. `src/slot_input.rs:383` (`slot()` test helper) + `src/bundle_unified.rs:124` (`s()` test helper, §2): `value: …to_string()` → `value: SecretString::new(…to_string())`.
7. `cmd/bundle.rs:2629`, `cmd/import_wallet.rs:1396`, `cmd/verify_bundle.rs:1883`: `s.value = SecretString::new(resolve_env_var_sentinel(&s.value, &flag)?);` (+ `use` if needed). **Closes the `@env:` residue leg of L22.**
8. `cmd/import_wallet.rs:1233`: `.map(|s| (s.index, s.value.to_string()))`.
9. `tests/lint_zeroize_discipline.rs`: add the `slot_input.rs` row + bump floor `:452` `35 → 36`:
   ```rust
   ZeroizeRow {
       label: "SlotInput value field is SecretString (Zeroizing<String> inner) — L22",
       source_file: "src/slot_input.rs",
       evidence: &["pub value: SecretString", "SecretString::new"],
   },
   ```
10. **`.value` field-READ sites compile UNCHANGED** (Deref `&SecretString → &str`): `DerivationPath::from_str(&p.value)`, `Fingerprint::from_str(&s.value)`, `normalize_xpub_prefix(&m.value)`, `bundle.rs:2103 &s.value`, and `import_wallet.rs:300` `s.value.is_empty()`/`s.value.starts_with("@env:")` — verify no breakage at compile. **`FromInput.value` sites (`convert.rs:131-133` field + `from.value == "-"` consumers) DO NOT change.**

**Affected files (P1):** `src/secret_string.rs`, `src/slot_input.rs`, `src/bundle_unified.rs`, `src/cmd/bundle.rs`, `src/cmd/import_wallet.rs`, `src/cmd/verify_bundle.rs`, `tests/lint_zeroize_discipline.rs`.

**P1 GREEN gate:** full `cargo test -p mnemonic-toolkit` + `cargo clippy --workspace --all-targets -- -D warnings`. Persist `cycle14-phase-1-<round>-review.md`; reviewer-loop to 0C/0I.

### P2 — convert / restore / addresses handler-local wraps (the remaining Site-1 scrub gap)

**RED first:**
- **T-P2a (no-behavior-change, convert `--passphrase-stdin` / `from -`):** an end-to-end convert smoke over a stdin passphrase + `from -` primary asserting byte-identical output to today (the wraps are purely in-memory). Fences the `effective_passphrase`/`effective_bip38_passphrase`/`primary_value` `Zeroizing<String>` migration + the `as_deref()` / `vec![…]` fixups.
- **T-P2b (restore / addresses, if in scope):** analogous no-behavior-change stdin-passphrase + `from -` smoke over `restore` and `addresses` (each has the same `let passphrase: String = if … else …` shape).

**Impl (minimal):**
1. `cmd/convert.rs`: `effective_passphrase: Option<String>` (`:853`) / `effective_bip38_passphrase: Option<String>` (`:861`) → `Option<Zeroizing<String>>`; `primary_value` (`:868`) → `Zeroizing<String>`. Fix the friction the spec named: `.as_deref()` (`:990-991`) → `.as_deref().map(String::as_str)` (or keep convert locals `Option<String>` and wrap only at the binding — implementer picks the lower-churn unify); `&primary_value` (`:994`) and `primary_value.as_str()` (`:1144`) Deref-absorb; `vec![primary_value.clone()]` (`:1025`) → `vec![primary_value.to_string()]` (or `(*primary_value).clone()`) so both `if/else` arms unify to `Vec<String>` (the `:1020` arm builds `Vec<String>` via `split_whitespace().map(to_string)`). **Keep the mlock pins `:880-886`** (D1 does not remove them — `lint_safety_first_party_mlock` must stay GREEN).
2. `cmd/restore.rs` (`:399,410,829,839,1291,1300,3045,3053`) + `cmd/addresses.rs` (`:149,159`) [**if in scope — D1 default = include**]: `let passphrase: String = if … { read_stdin_passphrase(stdin)? } else { … }` → `let passphrase: Zeroizing<String> = if … { Zeroizing::new(read_stdin_passphrase(stdin)?) } else { Zeroizing::new(…) }` (the v0.34.1 `if/else`-unify pattern); same for `from_value`. No `convert.rs`/`restore.rs`/`addresses.rs` lint rows needed (already `ZEROIZE_ROWS.source_file`s; their existing `Zeroizing` evidence matches).

**Affected files (P2):** `src/cmd/convert.rs`, `src/cmd/restore.rs`, `src/cmd/addresses.rs` (the latter two iff in scope).

**P2 GREEN gate:** full `cargo test -p mnemonic-toolkit` + `cargo clippy --workspace --all-targets -- -D warnings`. Persist `cycle14-phase-2-<round>-review.md`; reviewer-loop to 0C/0I.

> If PLAN R0 elects to DEFER restore/addresses, P2 collapses to the convert locals only + a FOLLOWUP `restore-addresses-stdin-local-zeroizing` filed.

### P3 — whole-diff review + version sweep + ship

- **PE — mandatory, NON-DEFERRABLE whole-diff adversarial execution review** over the entire P1+P2 diff (R0 = plan correctness; PE catches implementation-introduced regressions TDD misses — e.g. a `.value` read site that silently changed semantics, a missed `@env:` re-wrap, a regressed mlock pin). Persist `cycle14-whole-diff-review-<round>.md`; reviewer-loop to 0C/0I. If Agent-API dispatch fails, **flag explicitly** and defer to API recovery — never silently substitute inline self-review (CLAUDE.md step 5).
- **Version sweep `0.66.0 → 0.67.0`** (release-ritual version-sites — NOT all gate-enforced, MEMORY `project_toolkit_release_ritual_version_sites`): `crates/mnemonic-toolkit/Cargo.toml:3`; `README.md:13`; `crates/mnemonic-toolkit/README.md:9`; `scripts/install.sh:32` (self-pin `mnemonic-toolkit-v0.67.0`); `fuzz/Cargo.lock:575`; root `Cargo.lock:727` (self-heal via `cargo build` / `cargo update -p mnemonic-toolkit --precise 0.67.0` then verify); new top `CHANGELOG.md` entry (SemVer-MINOR; L22; SecretString field migration + `@env:` residue; mlock pins preserved; no wire/behavior change; no codec/GUI/manual/schema). **Re-run the full `-p` suite + fuzz build after the bump, before tag** (MEMORY `project_older_timelock_advisory_v0_55_2`).
- **FOLLOWUP / report ticks IN THE SHIPPING COMMIT** (MEMORY `feedback_followup_status_discipline` — verify "open" at decision time):
  - **Tick L22** — `design/agent-reports/constellation-bughunt-2026-06-20.md` `### - [ ] L22` (**RE-GREP `:850` at ship** — the report mutates) → `### - [x] L22` + `<!-- FIXED cycle-14 (toolkit v0.67.0 @<sha>) — SlotInput.value (incl. parse_slot_input ctor) is SecretString (Zeroizing<String> inner, redacting Debug); both the stdin `=-` AND the @env: write-back (bundle/import-wallet/verify-bundle) secret-residue paths scrubbed on drop; convert/restore/addresses handler locals wrapped. mlock pins preserved. No wire/behavior change. Whole-diff review GREEN. -->`.
  - **Flip Cycle-B Site-1 scrub leg** — `design/FOLLOWUPS.md` `secret-memory-hygiene-cycle-b` (`:1207`), Site-1 list (`:1211`): annotate that Site-1's **Zeroizing/scrub** leg is now complete at v0.67.0 (the mlock pin leg was done at v0.10.x); the toolkit's canonical 5-site list is fully closed (Sites 2-4 v0.10.x, Site 5 ms-cli, Site 1 scrub v0.67.0). **RE-GREP the live line at ship.**
  - **Cite, do NOT re-open** the precedents: `resolved-slot-derived-account-zeroizing-field` (v0.10.1), `import-wallet-blob-zeroizing` (v0.33.3), `bsms-decrypt-record-string-zeroizing` (v0.34.1).
- **Stage explicitly; commit; FF-merge to master; tag `mnemonic-toolkit-v0.67.0`.** (Direct-FF + tag per the toolkit lane; no codec publish, no GUI PR — none touched.)
- **Post-cycle FOLLOWUP burndown OFFER** (MEMORY `feedback_post_cycle_followup_burndown`): enumerate any newly-filed slugs (`restore-addresses-stdin-local-zeroizing` if deferred, `phrase-overlay-secretstring`, `stdin-reader-transient-buf-zeroizing` if R0 wants it tracked) + per-slug effort; AskUserQuestion all-or-select.

---

## 5. Out of scope (carried from spec §5)

- `OOS-reader-return-type-flip` — readers stay `String` (D1). A future transient-`buf` scrub = FOLLOWUP `stdin-reader-transient-buf-zeroizing`.
- `OOS-third-party-carriers` — `bip39::Mnemonic`/`Xpriv` interiors (covered by `lint_safety_third_party_blocked`).
- `OOS-emitted-bytes` — stdout/pipe/terminal residue (same allocator-residue limit `secret_string.rs` documents).
- `OOS-phrase-overlay-deep-wrap` — making `phrase_overlays: Vec<(u8, SecretString)>` (touches `apply_seed_overlay` signature) → FOLLOWUP `phrase-overlay-secretstring` if R0 wants it.
- `OOS-stale-lint-doc-comment` — `lint_zeroize_discipline.rs:370` `(24..=35)` vs live `(18..=60)` assert (pre-existing, not this cycle).

---

## 6. Mandatory R0 gate (this plan-doc)

This plan-doc MUST pass an **opus architect R0 review converged to 0 Critical / 0 Important** BEFORE any implementer dispatch. R0 runs the **full `cargo test -p mnemonic-toolkit` suite**, re-verifies the §0 citations against live `origin/master`, confirms (i) the zeroize-1.8.2 `Debug`-leak fact, (ii) the §2 census-completeness correction (`parse_slot_input:185` + `bundle_unified.rs:124` are the full remaining construction set — no 4th), (iii) the lint partition count `>= 36`, and (iv) the §3 scope decisions (restore/addresses include; `phrase_overlays` minimum `.to_string()`). Fold findings → persist the review verbatim to `design/agent-reports/cycle14-plan-r0-round{N}-review.md` → re-dispatch → repeat until GREEN (the reviewer-loop continues after every fold). Only then does P1 begin.
