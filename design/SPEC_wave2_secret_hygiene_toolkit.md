---
title: "SPEC — Wave-2 secret-hygiene (toolkit lane): T1-T4"
status: R0-PENDING (this spec must pass the mandatory opus R0 gate to 0C/0I BEFORE any implementation)
cycle: wave2-secret-hygiene-toolkit
repo: mnemonic-toolkit (crates/mnemonic-toolkit) — toolkit-internal, unpublished; no sibling-codec API touched
source_sha_pin: 34d3a724e8ac0ccb10ad13cbb5293b9bc844ae3c  (branch master HEAD at authoring; == origin/master)
crate_version_at_authoring: 0.70.1
target_version: 0.71.0 (MINOR)
ship_mechanism: direct FF-merge to master + tag mnemonic-toolkit-v0.71.0
ms_codec_pin: 0.6.0 (crates.io; Cargo.lock checksum 835040e2…; Payload has Entr + Mnem, both #[non_exhaustive]; PayloadKind: Copy)
---

# Wave-2 secret-hygiene — toolkit lane (T1-T4)

Four secret-memory-hygiene follow-ups, bundled into one toolkit release. All
honor the FIRST-CLASS secret-hygiene bar (zeroize-on-drop + redacting Debug +
off-argv — never deferrable). **All citations below were re-grepped against the
pinned SHA 34d3a724; FOLLOWUPS.md line numbers had drifted and are NOT used —
the live lines in this doc are authoritative.**

Precedents REUSED (do NOT reinvent):
- `ScrubbedXpriv` — move-only RAII newtype, shipped v0.70.0 at
  `src/derive_slot.rs:195` (`pub struct ScrubbedXpriv(Xpriv)`; `new`/`xpub`/
  `fingerprint` accessors; `impl Drop` = `private_key.non_secure_erase()` +
  volatile 32-byte chain_code zero-write; compile-time `!Clone`/`!Copy`). The
  type is deliberately escape-hatch-free — the in-source rule at
  `src/derive_slot.rs:187` reads `DO NOT add Clone/Copy/into_inner/Deref<Xpriv>`.
- `SecretString` — newtype over `Zeroizing<String>` with a **length-only
  REDACTING Debug**, transparent Display/Deref<str>/Serialize, derived Clone,
  structural PartialEq/Eq, shipped v0.67.0 at `src/secret_string.rs`
  (`pub struct SecretString(Zeroizing<String>)`, ctor `SecretString::new`).
- `Zeroizing<Vec<u8>>` move-into idiom for decoded ms1 entropy — canonical
  template `bundle.rs::bundle_run_from_import_json` (live `src/cmd/bundle.rs:2057-2089`)
  and `src/cmd/silent_payment.rs:149-151`.

SemVer policy applied (project precedent v0.10.1 / v0.67.0 / v0.68.0 / v0.69.0):
a `pub`/`pub(crate)`-stable-contract secret-TYPE/signature migration = **MINOR**
even with no public wire-shape change; a pure function-local `Zeroizing` wrap
with no signature change = **NO-BUMP/PATCH**. The bundle is **MINOR (0.71.0)**
because **T1** migrates the `pub` `DerivedAccount.account_xpriv` field type AND
the `pub fn into_parts` return-tuple arity AND adds a `pub`
`ScrubbedXpriv::expose_xprv_string` method. T2/T3/T4 are folded NO-BUMP/PATCH
legs that ride the same release.

**No CLI flag / subcommand / dropdown / `--json` wire-shape change anywhere in
this lane.** Therefore: NO `gui-schema` / `schema_mirror` trip and NO
`docs/manual/` flag-mirror trip. CONFIRMED by inspection — T1's `--to xprv`
output string is byte-identical; T2 keeps all inspect output byte-identical;
T3/T4 are transparent (SecretString/Zeroizing Display/Deref are pass-through).

---

## Bundled SemVer + ALL version/ship sites

Target: **v0.71.0** (MINOR). Update EVERY site below in the implementing PR
(several are NOT gate-enforced — silent drift). Gates that fire: the
`readme_version_current` test (`both_readmes_carry_current_version_marker`)
gates BOTH READMEs; the `changelog-check.yml` workflow fires on the **tag** and
greps `^## mnemonic-toolkit \[0.71.0\]` in `CHANGELOG.md`. The Cargo.lock /
fuzz/Cargo.lock / install.sh sites are NOT test-gated — update by hand.

1. `crates/mnemonic-toolkit/Cargo.toml:3` — `version = "0.70.1"` → `"0.71.0"`.
2. `README.md:13` — `<!-- toolkit-version: 0.70.1 -->` → `0.71.0` (+ status line if present).
3. `crates/mnemonic-toolkit/README.md:9` — `<!-- toolkit-version: 0.70.1 -->` → `0.71.0`.
4. `scripts/install.sh:32` — `…mnemonic-toolkit-v0.70.1…` → `…mnemonic-toolkit-v0.71.0…` (self-pin).
5. `Cargo.lock:727` — `name = "mnemonic-toolkit" / version = "0.70.1"` → `0.71.0` (regen via `cargo build`/`cargo update -p mnemonic-toolkit --precise 0.71.0` then verify diff is the single line).
6. `fuzz/Cargo.lock:575` — same `mnemonic-toolkit` entry → `0.71.0`.
7. `CHANGELOG.md` — add `## mnemonic-toolkit [0.71.0] — <date>` section ABOVE the
   `[0.70.1]` section, SemVer-MINOR, describing the secret-hygiene migration
   (T1-T4). The changelog-check gate fires on the tag; missing section = red CI.

Tag: `mnemonic-toolkit-v0.71.0` after FF-merge to master, full suite GREEN,
clippy clean, fuzz build re-checked.

**Re-run order at ship (per the release ritual memory):** bump all 7 sites →
`cargo test -p mnemonic-toolkit` (full suite, not targeted) → `cargo clippy
-p mnemonic-toolkit` → re-check `fuzz/` builds → commit → FF-merge → tag.

---

## T1 — `derive-slot-account-xpriv-scrub-confinement` (the 7-site lift) — MINOR

### Current behaviour (live @ 34d3a724)
- `src/derive.rs:27` — `pub account_xpriv: Xpriv` — a bare, un-scrubbed owned
  secret field on `pub struct DerivedAccount`. `Xpriv` is `Copy`, so any read
  is a leaking copy; the field drops without erase.
- `src/derive.rs:47-57` — `pub fn into_parts(mut self) -> (Vec<u8>, Fingerprint,
  Xpub, Xpriv, DerivationPath)`. The 4th tuple element copies
  `self.account_xpriv` (a `Copy` leak) out as a bare `Xpriv` (live source line
  `:54`). Doc comment `:45-46` says "three are `Copy`".
- `src/derive_slot.rs:85-95` — single construction site
  (`derive_bip32_from_entropy_at_path`): `account_xpriv = master.derive_priv(...)`,
  then `account_xpub = Xpub::from_priv(&secp, &account_xpriv)`, then the network
  guard, then stored bare into the struct at `:103-110`.
- **into_parts callers (8, ALL bind `_xpriv` and DISCARD — zero genuine reads):**
  - `src/cmd/bundle.rs:574` — `let (entropy, fingerprint, xpub, _xpriv, path) = acc.into_parts();`
  - `src/cmd/bundle.rs:687` — `let (_acc_entropy, fingerprint, xpub, _xpriv, path) = …`
  - `src/cmd/bundle.rs:733` — `let (_acc_entropy, fingerprint, xpub, _xpriv, path) = …`
  - `src/cmd/bundle.rs:1754` — `let (_acc_entropy, master_fp, xpub, _xpriv, _path) = …`
  - `src/cmd/bundle.rs:2871` — `let (entropy, fp, xpub, _xpriv, _path) = …`
  - `src/synthesize.rs:1376` — `let (entropy, master_fingerprint, account_xpub, _xpriv, _path) = …`
  - `src/cmd/verify_bundle.rs:1656` — `let (_acc_entropy, master_fp, xpub, _xpriv, _path) = …`
  - `src/cmd/verify_bundle.rs:1686` — `let (_acc_entropy, master_fp, xpub, _xpriv, _path) = …`
- **The ONLY genuine reader:** `src/cmd/convert.rs:1314` —
  `Xprv => derived.as_ref().unwrap().account_xpriv.to_string()`. A DELIBERATE
  user-requested secret emission (`convert --to xprv`; `Xprv` is in
  `is_secret_bearing()` at `src/cmd/convert.rs:94-100`). `derived` is
  `Option<DerivedAccount>`. There is a SECOND `Xprv` arm at
  `src/cmd/convert.rs:1461` but it is the `--from xprv` INPUT-decode path
  (parses a user-supplied xprv string into `Xpub`/`Fingerprint`); it does NOT
  read `account_xpriv` → **untouched by this migration**.
- `src/cmd/restore.rs:572,577` — reads only `acct.account_xpub`; comment at `:577`
  `// NB: acct (and its account_xpriv) is dropped here — never emitted.` The
  field-type change is transparent here; this is the best argument for the
  field migration (a real caller holding an un-scrubbed Xpriv for no reason).

### Exact change
**(1) Field migration** — `src/derive.rs:27`:
`pub account_xpriv: Xpriv` → `pub account_xpriv: ScrubbedXpriv`. Import
`use crate::derive_slot::ScrubbedXpriv;` at the top of `derive.rs`. Update the
doc comment `src/derive.rs:45-46` ("three are `Copy`") — after the migration the
xpriv is NOT Copy and is no longer in the tuple at all (see (2)); reword to
reflect the dropped element.

**(2) `into_parts` — DROP the Xpriv element entirely** (RECOMMENDED, R0 to
confirm). New signature: `pub fn into_parts(mut self) -> (Vec<u8>, Fingerprint,
Xpub, DerivationPath)`. Keep `mut self` (the `std::mem::take(&mut *self.entropy)`
needs it, AND the lint evidence anchor `pub fn into_parts(mut self)` at
`tests/lint_zeroize_discipline.rs:62` survives only if `mut self` is kept). The
`ScrubbedXpriv` field is not moved into the tuple → it stays in `self` and DROPS
in place at the end of `into_parts` (scrub fires). Update all 8 callers to drop
the `_xpriv` binding: `let (entropy, fingerprint, xpub, path) = acc.into_parts();`
(one fewer binding). Re-grep the 8 line numbers at impl time — they drift.

> ALTERNATIVE (rejected): re-type the 4th element to `ScrubbedXpriv`. This hands
> a movable Drop-type to 8 callers for zero benefit and re-exposes the secret
> handle. DROP-the-element is strictly better hygiene.

**(3) Wrap at construction** — `src/derive_slot.rs:103-110`
(`derive_bip32_from_entropy_at_path` ctor): the `account_xpub` read at `:87`
and the network guard at `:89-95` happen BEFORE the wrap, so no accessor churn.
Change the struct literal field from `account_xpriv,` to
`account_xpriv: ScrubbedXpriv::new(account_xpriv),`.

**(4) convert.rs:1314 — the design fork (R0 MUST rule).** RECOMMENDED option (a):
add a narrow, audited reader to `ScrubbedXpriv`:
```rust
/// CONTROLLED escape hatch for the DELIBERATE `convert --to xprv` emission
/// (Xprv ∈ is_secret_bearing). Returns the rendered xprv string wrapped in
/// `SecretString` (length-only redacting Debug + scrub-on-drop) so the
/// rendered secret never lingers un-scrubbed and never leaks via {:?}/panic.
/// String-only — NO `Xpriv` handle escapes.
// DO NOT widen to expose the Xpriv handle (would re-open the Copy-escape).
pub fn expose_xprv_string(&self) -> SecretString {
    SecretString::new(self.0.to_string())
}
```
(import `crate::secret_string::SecretString` in `derive_slot.rs`). Then
`src/cmd/convert.rs:1314` becomes
`Xprv => derived.as_ref().unwrap().account_xpriv.expose_xprv_string().to_string()`
— the `.to_string()` (via SecretString's Display) yields the SAME `String` the
old `Xpriv::to_string()` produced; pushed into the `out` Vec exactly as before.
**Output is byte-identical.** (If R0 prefers the `SecretString` to flow further
without a re-allocating `.to_string()`, the `out` Vec element type would have to
change — it is `(NodeType, String)`; keep it `String` to avoid widening the
`convert` plumbing. The single `.to_string()` re-alloc is the cost of byte-for-
byte fidelity and is acceptable; the source `Xpriv` string was already a fresh
alloc.)

> Option (b) — a separate bare-`Xpriv` derivation path for convert — is MORE
> code (convert would re-derive via a new `derive_account_xprv` helper) and
> re-introduces a bare-Xpriv leak window. REJECTED unless R0 finds option (a)'s
> narrow widening unacceptable.

### SemVer — MINOR (v0.71.0)
`DerivedAccount`, `account_xpriv`, `into_parts` are all `pub`; the field type +
`into_parts` arity are part of the (toolkit-internal but stable) contract →
secret-type migration = MINOR per precedent. `expose_xprv_string` is a `pub`
API addition (still MINOR). Runtime `--to xprv` wire output BYTE-IDENTICAL → no
GUI `--json` wire drift, no schema_mirror, no manual mirror.

### Pub-struct-Drop trap — DOES NOT trigger
`DerivedAccount` gains a FIELD whose type (`ScrubbedXpriv`) has `impl Drop`, but
`DerivedAccount` ITSELF gets no `impl Drop`. Adding a Drop-typed field does NOT
synthesize a struct-level `impl Drop`, so external move-out destructure is not
newly blocked at the `DerivedAccount` level. (It is already constrained by the
non-Copy `_entropy_pin: PinnedPageRange` field, and the canonical consume path
is `into_parts(self)`.) Toolkit is unpublished → even a true struct-Drop would
be contained, but it does not arise here.

### Test surface (TDD — RED first)
1. `tests/lint_zeroize_discipline.rs:48-66` (the DerivedAccount rows): UPDATE.
   Replace/extend the field row with `evidence: &["pub account_xpriv: ScrubbedXpriv"]`
   (new anchor). VERIFY the `into_parts` row anchor `pub fn into_parts(mut self)`
   (`:62`) still matches after the arity drop — it does IF `mut self` is kept;
   if R0 elects `self` (non-`mut`), re-anchor to `pub fn into_parts(self)`. The
   `bundle resolve_slots arms use into_parts` row (anchor `acc.into_parts()`,
   `~:163`) stays valid (the call text is unchanged). The row-count guard
   `18..=66` (`:428`) absorbs the +0/+1 rows; SECRET_FILE_FLOOR (`:517`)
   unaffected (derive.rs/derive_slot.rs already in the partition).
2. `src/derive.rs` unit tests (`:115-284`): they read only
   `acc.entropy`/`account_xpub`/`master_fingerprint`, NEVER `account_xpriv`, so
   they COMPILE UNCHANGED (free regression net). ADD one test asserting the
   migrated field is move-only / not `Copy` (mirror the `AmbiguousIfImpl<_>`
   compile-time pattern already in `derive_slot.rs:380-398`, OR a runtime
   witness `let _moved = …; /* second use of the field is a move error */`).
3. Byte-identical golden (the funds-fidelity guard for option (a)): assert the
   `convert --to xprv` output String is byte-identical pre/post migration for a
   fixed entropy vector (extend the TDD-3 spirit at `derive_slot.rs:436`). This
   is the LOAD-BEARING test — it proves `expose_xprv_string().to_string()` ==
   the old `account_xpriv.to_string()`.
4. `tests/cli_restore.rs:752-753` and `:1041-1042` already assert
   `!stream.contains("account_xpriv")` — they REMAIN GREEN (free negative leak
   guards; no edit).
5. NEW `ScrubbedXpriv` test (option (a)):
   `expose_xprv_string_debug_is_redacting_and_display_is_verbatim` —
   `format!("{:?}", x.expose_xprv_string())` is length-only (no xprv substring),
   AND `x.expose_xprv_string().to_string()` == the canonical xprv string. Lives
   in `mod scrub_tests` (`derive_slot.rs:355`).

### Already-shipped (do NOT redo)
`ScrubbedXpriv` + `derive_account_xpub_only` (`:251`) / `derive_accounts_xpub_only`
(`:272`) / private helpers + the `scrub_tests` TDD-1..TDD-4 suite are SHIPPED
v0.70.0 (additive, `#[allow(dead_code)]` markers removed when the own-account P2
consumer wires them — track separately). v0.10.1 deleted `DerivedAccount`'s old
`impl Drop` and made `entropy` a `Zeroizing<Vec<u8>>`. This T1 lift is ONLY the
remaining field/into_parts/convert migration.

---

## T2 — `self-check-ms1-decode-not-zeroizing` (2 sites) — NO-BUMP (folded)

### Current behaviour (live @ 34d3a724)
- **Site A** `src/cmd/bundle.rs::self_check_bundle` (fn def `:2365`; loop body
  `:2523-2537`): `let (_tag, payload) = ms_codec::decode(ms)…?;` at `:2526`
  yields a BARE `ms_codec::Payload`; used ONLY as an equality oracle at `:2530`
  (`if payload.as_bytes() != expected_bytes`), then dropped UN-SCRUBBED each
  iteration. `expected_bytes` is a caller-owned `&[u8]` borrow (sig
  `expected_entropy: &[Option<&[u8]>]` at `:2368`) — NEVER re-wrap it.
- **Site B** `src/cmd/inspect.rs`: `enum InspectPayload::Ms1 { tag:
  ms_codec::Tag, payload: ms_codec::Payload }` (`:159-166`), built in
  `decode_card` at `:171-172` via `ms_codec::decode(chunks[0])`, lives for the
  whole per-card scope (`decode_card` callers in the `for (kind, chunks)` loop
  at `:108-146`), read by `emit_inspect_text` (`:185-216`) and
  `emit_inspect_json` (`:300-323`) — both call `payload.as_bytes()`,
  `payload.kind()` (rendered `{:?}`), and the `Payload::Mnem { language }`
  discriminant — then dropped UN-SCRUBBED. The `entropy_hex` output is
  `--reveal-secret`-gated (text `:206-208`, json `:317-321`) → NOT an output
  leak; only the in-memory husk. `inspect.rs` currently has ZERO secret patterns
  (VERIFIED: `grep -c 'Zeroizing|SecretString|ScrubbedXpriv' inspect.rs` == 0).
  `InspectPayload`/`decode_card`/`emit_inspect_*` are NOT re-exported in
  `lib.rs`/`main.rs`/`cmd/mod.rs` (VERIFIED empty) → Site B reshape is fully
  contained to `inspect.rs`.

> ms-codec FACT (verified against the pinned 0.6.0 crates.io source
> `payload.rs`): `Payload` = `Entr(Vec<u8>)` | `Mnem { language: u8, entropy:
> Vec<u8> }`, both under `#[non_exhaustive]`; `PayloadKind` = `Entr | Mnem`,
> derives `Copy`; the caller-wrap contract (payload.rs doc) explicitly gives the
> `_`-arm snippet `Zeroizing::new((*p.as_bytes()).to_vec())`. So the move-out
> match must cover `Entr`, `Mnem`, AND a `_` arm.

### Exact change
**Site A** (`self_check_bundle`, after the `:2526` decode): move entropy out of
`payload` into a fn-local `Zeroizing<Vec<u8>>`, then compare:
```rust
let scrubbed: zeroize::Zeroizing<Vec<u8>> = match payload {
    ms_codec::Payload::Entr(b) => zeroize::Zeroizing::new(b),
    ms_codec::Payload::Mnem { entropy, .. } => zeroize::Zeroizing::new(entropy),
    _ => zeroize::Zeroizing::new(payload.as_bytes().to_vec()),
};
if scrubbed.as_slice() != expected_bytes { /* existing BundleMismatch */ }
```
Use fully-qualified `zeroize::Zeroizing::new(...)` (matches the
`bundle.rs:581/694` style); `self_check_bundle` does NOT inherit the fn-local
`use zeroize::Zeroizing;` that lives inside `bundle_run_from_import_json`. (Note
the `_` arm cannot bind `payload` by move and also call `payload.as_bytes()` —
because `Entr`/`Mnem` are the only current variants, the `_` arm is unreachable
today but REQUIRED for `#[non_exhaustive]`; bind it as `other => Zeroizing::new(other.as_bytes().to_vec())`
to keep the borrow valid, OR keep `payload` un-moved in the `_` arm — the
implementer picks whichever the borrow-checker accepts; the `Entr`/`Mnem` arms
MOVE, the `_` arm CLONES-then-the-husk-drops.)

**Site B** (`inspect.rs`): reshape the `Ms1` variant to carry the scrubbed
entropy + the small display bits the emit fns need (so the bare `Payload` husk
is dropped at decode time):
```rust
Ms1 {
    tag: ms_codec::Tag,
    entropy: zeroize::Zeroizing<Vec<u8>>,
    kind: ms_codec::PayloadKind,   // Copy
    language: Option<u8>,          // Some(code) for Mnem, None for Entr
},
```
In `decode_card` (`:171-172`), after `ms_codec::decode`, MOVE the entropy:
```rust
let (tag, payload) = ms_codec::decode(chunks[0])?;
let kind = payload.kind();
let language = match &payload { ms_codec::Payload::Mnem { language, .. } => Some(*language), _ => None };
let entropy = match payload {
    ms_codec::Payload::Entr(b) => zeroize::Zeroizing::new(b),
    ms_codec::Payload::Mnem { entropy, .. } => zeroize::Zeroizing::new(entropy),
    _ => zeroize::Zeroizing::new(payload.as_bytes().to_vec()),
};
Ok(InspectPayload::Ms1 { tag, entropy, kind, language })
```
(read `kind`/`language` BEFORE the move-match, since the move consumes
`payload`). `emit_inspect_text`/`emit_inspect_json` then read `&entropy` (for
`bytes`, `byte_length`, `bit_strength`, the `--reveal-secret hex::encode`),
`kind` (for `payload_kind: {:?}` — `PayloadKind: Copy`/`Debug`), and `language`
(the `Option<u8>` → `MNEM_LANGUAGE_NAMES.get(code as usize)`). **ALL output
strings — kind, language name, byte_length, bit_strength, gated entropy_hex —
stay byte-identical.** Do NOT widen any pub cross-crate API or wire-shape.

Reuse RAW `Zeroizing<Vec<u8>>` (NOT SecretString — the secret is raw entropy
BYTES, never Debug-printed; the precedent sites all use raw `Zeroizing`).

### SemVer — NO-BUMP (folded into 0.71.0)
Site A = function-local `Zeroizing` wrap (no signature change). Site B =
bin-private, non-re-exported enum-field reshape with NO external caller. No wire
/ `--json` / flag / cross-crate change. Folds into the bundle release.

### Test surface (TDD — RED first)
1. Site-A existing guards (`bundle.rs` `mod tests`):
   `self_check_detects_at0_only_ms1_regression`, `self_check_passes_watch_only_all_empty_ms1`,
   `self_check_detects_wrong_entropy_ms1` — must stay GREEN (they exercise the
   compare path).
2. Site-B existing guards: `tests/cli_inspect.rs`
   `cell_17_reveal_secret_gate_on_ms1_entropy_hex` + the inspect-envelope serde
   unit test `inspect_envelope_ms1_serializes_…` (`inspect.rs:356`) — must stay
   GREEN (output byte-identical).
3. `tests/lint_zeroize_discipline.rs` — the load-bearing lint work (do this
   FIRST → RED, then land the wraps → GREEN):
   - (a) ADD a `ZEROIZE_ROWS` row for Site A:
     `ZeroizeRow { label: "self_check_bundle ms1-decode oracle moves entropy into Zeroizing",
     source_file: "src/cmd/bundle.rs", evidence: &["Zeroizing::new(ms… ")] }` — anchor
     on a needle present in the new wrap (pick a unique substring distinct from
     bundle.rs's 14 existing matches).
   - (b) **MANDATORY** ADD a `ZEROIZE_ROWS` row for `src/cmd/inspect.rs` — once
     inspect.rs matches a `SECRET_PATTERN` (`Zeroizing::new(` / `: Zeroizing<`)
     it becomes "secret-bearing" and the SOURCE-direction scan
     `every_secret_bearing_src_file_is_declared_or_allowlisted` (`:577`) FAILS
     unless inspect.rs is a declared row (it is NOT currently a row, NOT
     allowlisted; today it has zero secret patterns so it's skipped).
   - (c) **BUMP `SECRET_FILE_FLOOR` 37 → 38** (`:517`) — the partition gains
     inspect.rs as the +1 secret-bearing file; the floor guard at `:614` fires
     if not bumped. The row-count guard `18..=66` (`:428`) easily absorbs the +2
     rows. bundle.rs needs no floor/partition change (already a declared,
     secret-bearing row).
   - TDD order: add the two rows + flip the floor FIRST (RED — inspect.rs has no
     wrap anchor yet), then land the wraps to turn GREEN.

### Already-shipped (do NOT redo / do NOT confuse)
4 toolkit ms1-decode sites already move entropy into Zeroizing:
`slot_ms1.rs`, `cmd/silent_payment.rs:149-151`, `wallet_import/overlay.rs`,
`bundle.rs::bundle_run_from_import_json` (`:2073/:2082`). The `convert.rs::Ms1`
arm is a 5th already-correct site (its own lint row exists) — DISTINCT from the
two T2 targets. The lint harness (dual-direction scan, TEST_ONLY tier, floor
guard) is shipped — only TWO rows + a floor bump.

---

## T3 — `phrase-overlay-secretstring` — PATCH (folded)

### Current behaviour (live @ 34d3a724)
- `src/cmd/import_wallet.rs:1229-1238` — `let phrase_overlays: Vec<(u8, String)> =
  args.slot.iter().filter(|s| s.subkey == SlotSubkey::Phrase).map(|s|
  (s.index, s.value.to_string())).collect();` — copies the phrase OUT of
  `SlotInput.value` (already a `SecretString`) into a bare `String` via
  `.to_string()` (the cycle-14 minimal compile-restore edit). The ONLY
  production caller of `apply_seed_overlay` (call at `:1239`).
- `src/cmd/import_wallet.rs:1225-1228` — `let mut ms1_args: Vec<Option<String>> =
  …; for v in &args.ms1 { ms1_args.push(Some(v.clone())); }` — sibling bare-String
  copy of `args.ms1` (`Vec<String>`, clap-derived). Master-secret-equivalent ms1
  codec material. SAME gap class.
- `src/wallet_import/overlay.rs:58-64` — `pub(crate) fn apply_seed_overlay(parsed:
  &mut [ParsedImport], ms1_args: &[Option<String>], phrase_overlays: &[(u8,
  String)], language: CliLanguage, stderr: &mut dyn Write) -> Result<…>`. ONE
  in-crate caller; `pub(crate)`, NOT `pub` — no external SemVer surface.
- `src/wallet_import/overlay.rs:67-70` — `enum Source { Ms1(String),
  Phrase(String) }` — a FUNCTION-LOCAL enum (the `Source` in
  `src/cmd/xpub_search/seed_intake.rs` is a SEPARATE unrelated type — zero
  coupling). Bare `String` in both variants.
- `src/wallet_import/overlay.rs:80` `Source::Ms1(s.clone())`, `:96`
  `Source::Phrase(phrase.clone())` — a SECOND bare-String copy when building the
  `by_index` map.
- Consumer arms: `Source::Ms1(s)` at `:118` (`ms_codec::decode(s)` — `decode(s:
  &str)` per 0.6.0 source), `Source::Phrase(phrase)` at `:162`
  (`bip39::Mnemonic::parse_in(lang, phrase)`). The derived entropy is ALREADY
  `Zeroizing<Vec<u8>>` (`:133/:148/:169/:207`).

### Exact change (REUSE `SecretString`; NEVER raw `Zeroizing<String>` — its
derived Debug LEAKS into `{:?}`/panic, the cycle-14 KEY LESSON)
1. `import_wallet.rs:1229-1238` — `Vec<(u8, SecretString)>`, map `.value.clone()`
   (SecretString: Clone) instead of `.to_string()`. Phrase never leaves the
   scrubbing newtype.
2. `import_wallet.rs:1225-1228` — `Vec<Option<SecretString>>`, map
   `SecretString::new(v.clone())` off `args.ms1: Vec<String>` (the clap arg stays
   `String`). (This is the ms1 COMPLETENESS arm — see scope fork below.)
3. `overlay.rs:60-63` — signature: `ms1_args: &[Option<SecretString>]`,
   `phrase_overlays: &[(u8, SecretString)]`.
4. `overlay.rs:67-70` — `enum Source { Ms1(SecretString), Phrase(SecretString) }`.
5. `overlay.rs:80` `Source::Ms1(s.clone())`, `:96` `Source::Phrase(phrase.clone())`
   — now `SecretString::clone()` (scrubs on drop).
6. Consumer arms `:118`/`:162` — `ms_codec::decode(s)` / `parse_in(lang, phrase)`
   take `&str`; `SecretString: Deref<Target=str>` deref-coerces transparently.
   Add `&*` / `&**` ONLY where the coercion does not auto-fire (verify at
   compile; avoid gratuitous deref noise per the v0.69.0 deref-coercion ruling).
   The derived entropy at `:169`/etc. is ALREADY `Zeroizing` → unchanged.

**Scope fork (R0 to rule):** RECOMMENDED — wrap BOTH Phrase AND ms1 arms in one
pass (identical pattern, same two functions, fan-out already mapped: 1 signature
+ 1 call site + 1 fn-local enum + 1 test). Phrase-only is a valid tighter first
cut matching the slug's literal scope, but then the ms1 arm MUST be filed as
explicit residue. `args.ms1` is bare master-secret-equivalent material — leaving
it bare while wrapping Phrase is an asymmetric half-fix.

### SemVer — PATCH (folded into the 0.71.0 MINOR)
`apply_seed_overlay` is `pub(crate)` with ONE in-crate caller, the `Source` enum
is fn-local, zero wire/behavior change. Closer to the function-local-wrap
NO-BUMP/PATCH precedent than the pub-secret-migration-as-MINOR precedent. The
crate is unpublished → the version call is ritual, not a compatibility
constraint. Folded into the bundle (the bundle is already MINOR via T1).

### Pub-struct-Drop trap — NOT triggered. No new `impl Drop`, no new pub struct;
SecretString's scrub comes from `Zeroizing<String>`'s Drop. Double-Zeroizing
AVOIDED: do NOT re-wrap `SlotInput.value` (already SecretString — just `.clone()`
it); do NOT touch the entropy locals at `:133/:148/:169/:207` (already Zeroizing).

### Test surface (TDD)
1. UPDATE existing T4c `src/cmd/import_wallet.rs:2688`
   (`phrase_overlay_collection_carries_phrase_via_to_string`): flip its mirror
   `Vec` to `Vec<(u8, SecretString)>` and assert via `&*phrase_overlays[0].1 ==
   phrase` (Deref) since SecretString has no `(u8, &str)` tuple-eq; OR rename to
   `…carries_phrase_via_secretstring`. It currently PINS the `.to_string()`
   shape — it MUST be UPDATED, not added.
2. NO mandatory `lint_zeroize_discipline.rs` change: the overlay row
   (`:399-403`) is OR-anchored on `["Zeroizing<Vec<u8>>", "Zeroizing::new"]` —
   still satisfied by the entropy wraps; overlay.rs is ALREADY in the
   secret-bearing partition (`Zeroizing::new(` at `:133/:148/:169`) so adding
   `SecretString` does NOT change partition membership or the count →
   SECRET_FILE_FLOOR=37 UNAFFECTED. OPTIONAL: add a dedicated overlay row
   anchored on a `SecretString` needle to positively assert the new wrap.
3. Behavior regression coverage already exists — must stay GREEN unchanged:
   `tests/cli_import_wallet_seed_overlay.rs` (phrase + ms1 match/mismatch/conflict
   cases), `tests/cli_bundle_import_json.rs`
   (`bundle_import_json_seed_overlay_via_slot_phrase…`),
   `tests/cli_argv_leakage.rs` cell-1/cell-4 (phrase stdin).
4. Run the FULL `cargo test -p mnemonic-toolkit` suite (CLI/secret edits ripple
   into argv/schema/zeroize lints outside any one target — the R0-full-suite
   rule).

### Already-shipped (do NOT redo)
`SlotInput.value` is ALREADY `SecretString` (v0.67.0) and IS the upstream owner
— the phrase IS scrubbed up to the collection point. SecretString newtype +
ScrubbedXpriv are shipped (reuse). The derived entropy in `apply_seed_overlay`
is ALREADY `Zeroizing<Vec<u8>>`. The T4c fence test EXISTS (update, not add).
This is the explicitly-deferred deep-wrap leg of cycle-14/L22 Site-1.

### Path correction (slug decay)
The slug's `src/wallet_import/overlay.rs` is crate-root-relative and decayed —
the REAL path is `crates/mnemonic-toolkit/src/wallet_import/overlay.rs` (there
is no top-level `src/`). All paths in THIS doc are repo-relative and verified.

---

## T4 — `stdin-reader-transient-buf-zeroizing` (2 readers) — NO-BUMP (folded)

### Current behaviour (live @ 34d3a724)
- `src/cmd/convert.rs:745-751` — `read_stdin_to_string<R: Read>(stdin: &mut R)
  -> Result<String, ToolkitError>`: `let mut buf = String::new();` →
  `stdin.read_to_string(&mut buf)?` → `Ok(buf.trim().to_string())`. `buf` is a
  bare un-scrubbed scratch; `.trim().to_string()` allocates a SECOND fresh bare
  String that is returned.
- `src/cmd/convert.rs:758-770` — `read_stdin_passphrase<R: Read>(stdin: &mut R)
  -> Result<String, ToolkitError>`: `let mut buf = String::new();` →
  `read_to_string(&mut buf)?` → pops a single trailing `\r?\n` IN PLACE
  (`:763-768`) → returns bare `Ok(buf)` BY MOVE (`:769`). (The slug's "return
  `buf.trim().to_string()`" is WRONG for this fn — it returns `buf` by move.)
- `std::io::Read` is in scope; `zeroize` is a dependency and used elsewhere in
  convert.rs.

### Exact change
In BOTH fns: `let mut buf = String::new();` →
`let mut buf = zeroize::Zeroizing::new(String::new());`. `read_to_string(&mut
buf)` works via `DerefMut`.
- `read_stdin_to_string`: return stays `Ok(buf.trim().to_string())` (`.trim()`
  via `Deref<Target=String> → str`; allocates the same fresh bare String).
- `read_stdin_passphrase`: the in-place `buf.ends_with`/`buf.pop()` work via
  `DerefMut`; the final `Ok(buf)` MUST become `Ok(buf.to_string())` (or
  `Ok((*buf).clone())`) because `buf` is now `Zeroizing<String>` and the return
  type stays `String`. This is a real, intended change to that return
  expression — NOT a no-op.

**RETURN TYPE STAYS `String` in BOTH** — DO NOT flip to `Zeroizing<String>`.

### SemVer — NO-BUMP (folded). Pure function-local `Zeroizing` wraps, ZERO
signature change (return stays `String`, both stay `pub(crate)`). Byte-identical
behavior (same trimming, same CRLF-strip, same returned bytes).

### Return-type-flip trap (the load-bearing risk — why the return MUST stay String)
There are 42 reader call sites across the crate (recon's "28/14" are stale
undercounts). They split into THREE classes, ALL of which break if the return
type is narrowed:
- (a) DIRECT-WRAP sites `Zeroizing::new(read_stdin_*(...))` → would become
  illegal `Zeroizing<Zeroizing<String>>` (convert.rs, final_word.rs,
  derive_child.rs, seedqr.rs, seed_xor.rs, slip39.rs, ms_shares.rs,
  electrum_decrypt.rs, import_wallet.rs, silent_payment.rs).
- (b) MIXED-IF-ARM sites where the reader feeds one arm of `Zeroizing::new(if …
  { read_stdin_*() } else { … })` — flipping the reader type TYPE-MISMATCHES the
  arms (restore.rs, addresses.rs).
- (c) BARE NON-SECRET consumer: `electrum_decrypt.rs:120` reads the ENCRYPTED
  ciphertext into a bare `let ciphertext: String` (`:119` comment: `Not secret →
  no advisory`) — narrowing would force an unwanted wrap on non-secret data.
Keeping `-> String` avoids ALL of these and needs ZERO caller edits. Double-
Zeroizing is also avoided: the RETURN expression deliberately produces a bare
`String` so callers' existing `Zeroizing::new(...)` wraps don't double-wrap.

### Residual-scope caveat (set expectations, not a blocker)
This scrubs the `read_to_string` SCRATCH buffer only; it does NOT scrub the
returned fresh String (already the caller's responsibility — every secret-class
call site already wraps it). Explicitly defense-in-depth; do not over-scope into
flipping the return type.

### Test surface (TDD)
1. Behavior-preservation: `read_stdin_to_string` still trims surrounding
   whitespace; `read_stdin_passphrase` still strips exactly one trailing `\r?\n`
   while preserving leading/trailing spaces, internal NUL, tabs (the docstring
   contract at convert.rs:752-757). Existing tests live near the convert.rs
   `#[cfg(test)]` (first at `~:1887`) and `tests/cli_slip39_stdin.rs` (contrasts
   the two readers' trim/preserve semantics) — re-run; must stay GREEN.
2. `lint_zeroize_discipline.rs`: the in-fn `Zeroizing::new(String::new())` adds a
   `Zeroizing::new(` occurrence to convert.rs, which is ALREADY a
   `ZEROIZE_ROWS.source_file` and ALREADY in the partition — so the
   source-direction scan, SECRET_FILE_FLOOR (`:517`), and the row-count guard
   (`:428`) are ALL unaffected (no file enters/leaves the partition). OPTIONAL
   documentation row anchored on `Zeroizing::new(String::new())` (distinct
   needle); NOT gate-required.
3. `tests/lint_argv_secret_flags.rs:203` references the literal
   `read_stdin_to_string` as an evidence anchor — the fn NAME + `== "-"` dispatch
   are unchanged → stays GREEN.
4. Run the FULL `cargo test -p mnemonic-toolkit` suite.

### Already-shipped (do NOT redo)
The OWNERS of the lingering stdin secret are ALL already scrubbed (v0.67.0/cycle-14:
`SlotInput.value` = SecretString; every handler-scope stdin/`@env:` local =
`Zeroizing<String>`). cycle-14's D1 DELIBERATELY left these two readers returning
bare `String` to avoid breaking the wrapping callers; THIS fix is the scoped
continuation (wrapping ONLY the internal `buf`). No part has shipped yet.

---

## Deferred / blocked (excluded from this lane)

- **OUT OF SCOPE — upstream-blocked (unchanged caveat):** `rust-bitcoin`'s
  `Xpriv` has no Drop+Zeroize (`rust-bitcoin-xpriv-zeroize-upstream`);
  `ScrubbedXpriv::Drop`'s `non_secure_erase` is best-effort. NOT a blocker for
  T1 — the newtype is the best available confinement; documented, not actionable
  here.
- **T1 convert reader & T3 scope are DESIGN-GATED, not build-blocked** — both
  must be ruled at R0 BEFORE coding (see Open questions). They are IN this lane,
  not deferred.

## Post-ship bookkeeping (in the shipping commit)
Flip ALL FOUR FOLLOWUP slugs to RESOLVED in `design/FOLLOWUPS.md` in the shipping
commit (verify-status-at-decision-time discipline):
`derive-slot-account-xpriv-scrub-confinement`,
`self-check-ms1-decode-not-zeroizing`, `phrase-overlay-secretstring`,
`stdin-reader-transient-buf-zeroizing`. If T3 lands Phrase-only, file the ms1
arm as explicit residue instead of fully resolving T3.

## Mirror-invariant confirmation (no lockstep trips)
- NO clap flag / subcommand / option / dropdown-value addition/removal/rename →
  NO `mnemonic-gui/src/schema/mnemonic.rs` schema_mirror update; NO
  `docs/manual/src/40-cli-reference/` flag-mirror update. CONFIRMED by
  inspection of all four items.
- NO `--json` wire-shape change (T1 `--to xprv` byte-identical; T2 inspect output
  byte-identical; T3/T4 transparent). No GUI `--json` consumer paired-PR needed.

## R0 gate (MANDATORY — no code before GREEN 0C/0I)
This spec MUST pass the opus architect R0 review to 0 Critical / 0 Important
BEFORE any implementation, per project convention — even for the NO-BUMP legs.
The two design forks (T1 convert reader; T3 scope) are exactly the funds-/secret-
hygiene calls the R0 gate exists for. After folding findings: persist the review
verbatim to `design/agent-reports/`, re-dispatch, repeat until GREEN. Then a
SINGLE TDD implementer executes in a worktree; a mandatory post-impl
whole-diff adversarial review follows.