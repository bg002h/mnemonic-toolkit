## R0 Review — SPEC Wave-2 secret-hygiene (toolkit lane): T1–T4

**Verdict: GREEN (0 Critical / 0 Important, 2 Minor).** The gate may proceed to implementation. The two Minors are fold-before-code clarifications (floor arithmetic + one named deref site); neither blocks, but both should be folded so the single TDD implementer does not hit an avoidable RED.

Pin verified: HEAD == origin/master == `34d3a724e8ac0ccb10ad13cbb5293b9bc844ae3c`; crate version 0.70.1. ms-codec resolves to 0.6.0 (Cargo.lock:766-769, checksum `835040e2…` matches). All citations below were re-grepped against the live tree.

---

### Critical
_None._

### Important
_None._

### Minor

**M1 — T2 `SECRET_FILE_FLOOR` arithmetic premise is stale (live partition is 38, not 37).**
The spec states the partition is "exactly 37 secret-bearing files" and instructs "BUMP `SECRET_FILE_FLOOR` 37 → 38." Ground truth: the live partition is **38** (counted with the test's own substring patterns `Zeroizing::new(` / `SecretString::new(` / `: Zeroizing<` / `: SecretString`). `SECRET_FILE_FLOOR = 37` at `tests/lint_zeroize_discipline.rs:517`; the lint passes today (`every_secret_bearing_src_file_is_declared_or_allowlisted` GREEN: all 38 are declared/allowlisted, 38 ≥ 37). After T2 wraps `inspect.rs` the partition becomes **39**; the spec's "→ 38" still passes (39 ≥ 38) but leaves the floor 2 below the partition, perpetuating drift. Tight value is **39**. Non-gate-breaking (`>=` floor; no un-scrubbed residue); floor-hygiene only.
*Fold:* change "37 → 38" to "37 → 39" (or instruct the implementer to recompute the live count after wrapping inspect.rs and note the documented "37" is stale).

**M2 — T3 deref-coercion under-specified at the one site that does NOT auto-fire (`parse_in`).**
Verified: `ms_codec::decode(s: &str)` (`ms-codec-0.6.0/src/decode.rs:42`) is a **concrete** `&str` param → `&SecretString` deref-coerces exactly as today's `&String` does (overlay.rs:~131 arm OK). BUT `bip39::Mnemonic::parse_in<S: Into<Cow<'a, str>>>` (`bip39-2.2.2/src/lib.rs:521`) is a **generic** param — deref-coercion does not fire for generics, and `SecretString` impls neither `Into<Cow<str>>` nor `AsRef<str>` (only `Deref<str>`/Display/Serialize/Clone/Eq, per `src/secret_string.rs:32-71`). So `overlay.rs:164`'s `parse_in(language.into(), phrase)` with `phrase: &SecretString` will **not** compile as-is — it needs `&**phrase`. The spec's catch-all ("add `&*`/`&**` where coercion does not auto-fire; verify at compile") operationally covers this, but names neither site, risking a TDD-RED surprise.
*Fold:* name the `parse_in` arm explicitly as the `&**phrase` site (generic param, no coercion) vs the `decode` arm (concrete `&str`, coerces). Resolves it pre-code.

---

### Verified-correct (load-bearing claims checked against live source)

**T1 — derive-slot account-xpriv scrub-confinement (MINOR).**
- `DerivedAccount.account_xpriv: Xpriv` at `derive.rs:27`; `into_parts` at `derive.rs:47-57` (4th tuple elem = bare `Xpriv` copy at `:54`); ctor sole construction site `derive_slot.rs:103-110` — all exactly as cited.
- `Xpriv` derives `Copy, Clone` (`bitcoin-0.32.8/src/bip32.rs:72-74`) → the "any read is a leaking copy; field drops without erase" premise is correct.
- **Fan-out fully mapped (no missed caller).** Only struct-literal construction site is `derive_slot.rs:103` (no test builds it by literal; no `DerivedAccount { account_xpriv, .. }` destructure anywhere). All 8 `into_parts` callers bind `_xpriv` and discard (bundle.rs:574/687/733/1754/2871, verify_bundle.rs:1656/1686, synthesize.rs:1376 — the last is inside a `#[cfg(test)] fn fixture_full`; arity-drop edit is identical, harmless). The ONLY genuine `account_xpriv` reader is `convert.rs:1314`; `convert.rs:1461` is the `--from xprv` INPUT-decode path (parses a user string; never touches the field) — correctly excluded.
- `ScrubbedXpriv` is escape-hatch-free as described: `pub struct ScrubbedXpriv(Xpriv)` at `derive_slot.rs:195`; in-source rule "DO NOT add Clone/Copy/into_inner/Deref<Xpriv>" at `:187`; `impl Drop` = `private_key.non_secure_erase()` + volatile 32-byte chain_code zero (`:217-239`); compile-time `!Clone` witness `AmbiguousIfImpl` at `:379-399`. Adding `expose_xprv_string(&self) -> SecretString` is a sound, narrow, string-only widening (no `Xpriv` handle escapes).
- Option (a) byte-identical reasoning holds: `out` Vec is `Vec<(NodeType, String)>` (`out.push((t, v))` at convert.rs:1457); `expose_xprv_string().to_string()` via SecretString's Display yields the same String as `Xpriv::to_string()`. `Xprv ∈ is_secret_bearing()` confirmed (convert.rs:94-100).
- SemVer MINOR correct (pub field type migration + pub `into_parts` arity change + pub method addition; no `--to xprv` wire change). Pub-struct-Drop trap correctly does NOT trigger — a Drop-typed FIELD does not synthesize a struct-level `impl Drop` on `DerivedAccount`; the only construction is the in-crate ctor.
- Scope matches the FOLLOWUP slug exactly: `derive-slot-account-xpriv-scrub-confinement` is `open` but NARROWED at v0.70.0 (ScrubbedXpriv + xpub-only helpers already shipped); the slug's "REMAINING open scope is the broader 7-site lift: replace `DerivedAccount.account_xpriv: Xpriv` … + `into_parts`" is precisely what T1 does. The "do NOT redo the already-shipped helpers" guidance is correct.
- Lint anchor `pub fn into_parts(mut self)` (`lint_zeroize_discipline.rs:62`) survives because the spec keeps `mut self`. Doc-comment `derive.rs:45-46` ("three are `Copy`") correctly flagged for update. `cli_restore.rs:752/1041` negative `!contains("account_xpriv")` guards stay GREEN (free leak nets).
- Dead-code note (informational, not a finding): `ScrubbedXpriv`'s impl block and the xpub-only helpers carry `#[allow(dead_code)]` (P2 not yet wired). Adding `expose_xprv_string` inside the allowed impl block inherits the allow; `new`/`Drop` become live via the ctor/convert path; the unused `xpub`/`fingerprint`/4 helpers keep their existing allows. No warning churn — the spec correctly does not remove them.

**T2 — self-check + inspect ms1-decode (NO-BUMP, folded).**
- Site A: `bundle.rs:2526` bare `Payload` used only as equality oracle at `:2530`, dropped un-scrubbed; `expected_entropy: &[Option<&[u8]>]` borrow — correctly never re-wrapped.
- Site B: `InspectPayload::Ms1 { tag, payload }` (`inspect.rs:159-166`), read by `emit_inspect_text` (`:185-216`: `as_bytes`, `kind()` `{:?}`, `Mnem{language}`) and `emit_inspect_json` (`:300-323`: same + `payload_kind: format!("{:?}",…)`, `language` Option). Containment verified: `InspectPayload`/`decode_card`/`emit_inspect_*` are NOT re-exported in lib.rs/main.rs/cmd/mod.rs.
- ms-codec 0.6.0 protocol fact verified against crates.io source `ms-codec-0.6.0/src/payload.rs`: `Payload = Entr(Vec<u8>) | Mnem { language: u8, entropy: Vec<u8> }`, both `#[non_exhaustive]`; `PayloadKind` derives `Copy`. The reshape's `language: Option<u8>` and the **mandatory `_` arm** on the move-out match are both correct.
- Existing test anchors exist: `cli_inspect.rs:105 cell_17_reveal_secret_gate_on_ms1_entropy_hex`, `inspect.rs:356 inspect_envelope_ms1_serializes_…`. New lint row for inspect.rs is genuinely required (inspect.rs currently has 0 secret patterns → not yet a declared row; once it gains `Zeroizing::new(` it becomes secret-bearing and the source-direction scan at `:577` fails until declared). bundle.rs is already a declared, secret-bearing row → no floor/partition change for Site A (correct). Row-count guard `18..=66` (`:428`) absorbs the +rows.

**T3 — phrase-overlay SecretString (PATCH, folded).**
- import_wallet.rs:1225-1238 (ms1 bare-String copy + phrase `.to_string()`), overlay.rs:58-70 (`pub(crate)` sig + fn-local `enum Source { Ms1(String), Phrase(String) }`), `Source::Ms1(s.clone())`@:80 / `Source::Phrase(phrase.clone())`@:96, consumer arms @:118/:162, entropy already `Zeroizing` @:117/:133/:148/:169 — all exactly as cited. `apply_seed_overlay` has ONE in-crate caller; not `pub` → no external SemVer surface.
- `SlotInput.value` is already `SecretString` (upstream owner confirmed via lint row `slot_input.rs` at `:406-410`), so the fix is `.clone()` not re-wrap (double-Zeroizing avoided). REUSE-SecretString-not-raw-Zeroizing instruction is correct per the cycle-14 lesson (raw `Zeroizing<String>` leaks via derived Debug; SecretString redacts — verified at secret_string.rs:60-65).
- T4c fence test `phrase_overlay_collection_carries_phrase_via_to_string` exists at import_wallet.rs:2688; spec correctly says UPDATE (it PINS the `.to_string()` shape). Overlay lint row OR-anchored on `["Zeroizing<Vec<u8>>","Zeroizing::new"]` (`:399-403`) stays satisfied; overlay.rs already in partition → SECRET_FILE_FLOOR unaffected by T3 (correct).
- Scope fork (wrap both Phrase + ms1) is the right call — `args.ms1` is master-secret-equivalent; wrapping Phrase only is an asymmetric half-fix. If Phrase-only lands, filing the ms1 arm as explicit residue (not resolving the slug) is the correct discipline.
- Referenced regression tests exist: cli_import_wallet_seed_overlay.rs, cli_bundle_import_json.rs, cli_argv_leakage.rs. Path-correction (crate-relative `crates/mnemonic-toolkit/src/wallet_import/overlay.rs`) is accurate.

**T4 — stdin-reader transient-buf Zeroizing (NO-BUMP, folded).**
- `read_stdin_to_string` (convert.rs:745-751, returns `Ok(buf.trim().to_string())`) and `read_stdin_passphrase` (convert.rs:758-770, returns `Ok(buf)` BY MOVE). Spec correctly catches the slug's WRONG claim that read_stdin_passphrase returns `buf.trim().to_string()`; the `Ok(buf)` → `Ok(buf.to_string())` change after wrapping is correctly identified as real (not a no-op).
- Return-type-flip trap analysis is sound: keeping `-> String` avoids all caller churn. Verified 42 reader call sites (spec correctly says recon's 28/14 are undercounts); electrum_decrypt.rs:119-120 reads bare non-secret `ciphertext: String` ("Not secret → no advisory") — narrowing the reader would force an unwanted wrap there; restore.rs/addresses.rs feed `Zeroizing::new(read_stdin_*())` direct-wrap and mixed-if-arm sites that would break under a narrowed return. NONE of the 42 callers change under `-> String`.
- Lint anchor `read_stdin_to_string` at lint_argv_secret_flags.rs:203 unchanged (fn name + `== "-"` dispatch preserved) → stays GREEN. convert.rs already a ZEROIZE_ROWS source_file → adding `Zeroizing::new(String::new())` does not change partition membership.

**Version / ship sites + gates (all 7 verified).**
- Cargo.toml:3 (`0.70.1`), README.md:13 + crates/mnemonic-toolkit/README.md:9 (`<!-- toolkit-version: 0.70.1 -->`), install.sh:32 (`mnemonic-toolkit-v0.70.1`), Cargo.lock:726-727, fuzz/Cargo.lock:574-575 — all present at the cited locations.
- `readme_version_current.rs:23 both_readmes_carry_current_version_marker` reads `CARGO_PKG_VERSION` dynamically and gates BOTH READMEs (will RED if either marker stale).
- `changelog-check.yml` fires on tag push `mnemonic-toolkit-v*` (lines 17-20) and greps `## mnemonic-toolkit [$VERSION]`; CHANGELOG top section is currently `[0.70.1]` — spec's "add `[0.71.0]` ABOVE `[0.70.1]`" is correct. Missing section = RED on tag.
- All 4 FOLLOWUP slugs exist in design/FOLLOWUPS.md (`self-check-…`@:29, `phrase-overlay-…`@:4436, `stdin-reader-…`@:4446, `derive-slot-…`@:4528); post-ship flip-to-RESOLVED is valid (with the Phrase-only residue caveat for T3).
- Mirror invariants: no clap flag/subcommand/dropdown/`--json` wire change in any of T1–T4 → no schema_mirror trip, no manual flag-mirror trip. Confirmed: T1 `--to xprv` byte-identical (option a), T2 inspect output byte-identical, T3/T4 transparent.

---

### Implementation guidance carried into the plan
1. Fold M1 (floor → 39, or recompute live) and M2 (name the `&**phrase` parse_in site) into the spec before the plan-doc R0.
2. TDD order is correctly specified (RED lint rows/floor first for T2; UPDATE the T4c fence test for T3; byte-identical golden for T1 option-a is the load-bearing funds-fidelity guard).
3. Re-grep all 8 `into_parts` caller line numbers + the convert.rs:1314 line at impl time (they drift).
4. Run the FULL `cargo test -p mnemonic-toolkit` suite (not targeted) per the R0-full-suite rule — CLI/secret edits ripple into argv/schema/zeroize lints outside any one target.