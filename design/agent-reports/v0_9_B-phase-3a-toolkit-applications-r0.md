# R0 Report — Phase 3a Toolkit Applications (v0.9.0 Cycle B)

**Reviewer:** Opus 4.7 (1M context) via `feature-dev:code-reviewer`
**Date:** 2026-05-13
**SPEC:** `design/SPEC_secret_memory_hygiene_v0_9_B.md` (commit `a49386f`, post-Fix-B fold)
**Plan:** `~/.claude/plans/v0_9_B-secret-memory-hygiene-cycle-b.md` (R1 Fix-B; not in git)
**Scope:** Phase 3a — apply slice-fn `pin_pages_for` at toolkit Sites 1-4 + wire `report_at_exit()` in main()
**Phase 2 baseline:** commit `30cd0e6` (CI green: test matrix Ubuntu+macOS + miri + clippy)
**Verdict:** **RE-DRAFT (4 Critical + 3 Important + 2 Nit; see §10)**

---

## Summary

| Severity | Count |
|---|---|
| Critical | 4 |
| Important | 3 |
| Nit | 2 |

The plan and SPEC contain **systematic narrative inaccuracies** about the Cycle A baseline shape and the Phase 3a apply surface. The biggest issue: both repeatedly state Sites 2/3 keep `entropy: Zeroizing<Vec<u8>>` (Cycle A shape), but Cycle A actually settled on plain `Vec<u8>` with explicit `impl Drop` in `DerivedAccount` and `Option<Vec<u8>>` in `ResolvedSlot`. This is exactly the Phase 1 / Phase 2 R0 pattern the framing prompt warned about. Site 1's "12 fields across 6 cmd structs" is also wrong on both axes (no `EncodeArgs` / `VerifyArgs` exist; the secret-bearing field count is different). The implementation must not parrot the plan's claims; it must read the source.

---

## §1 Site 1 enumeration — actual cmd-struct + field landscape

**Plan claim:** "~12 fields across 6 cmd structs: `BundleArgs`, `VerifyBundleArgs`, `ConvertArgs`, `DeriveChildArgs`, `EncodeArgs`, `VerifyArgs`."

**Actual `crates/mnemonic-toolkit/src/cmd/`:**

| File | Args struct |
|---|---|
| `bundle.rs` | `BundleArgs` |
| `verify_bundle.rs` | `VerifyBundleArgs` |
| `convert.rs` | `ConvertArgs` |
| `derive_child.rs` | `DeriveChildArgs` |
| `export_wallet.rs` | `ExportWalletArgs` |
| `gui_schema.rs` | `GuiSchemaArgs` (empty) |
| `mod.rs` | (re-exports only) |

→ **C-1 (conf 95):** `EncodeArgs` and `VerifyArgs` named in the plan do NOT exist as separate cmd structs. They were folded into `BundleArgs` / `VerifyBundleArgs` historically. The plan's "6 cmd structs" is overstated; there are **4 cmd structs that may carry secret-bearing user input** (`BundleArgs`, `VerifyBundleArgs`, `ConvertArgs`, `DeriveChildArgs`), one watch-only (`ExportWalletArgs`), and one inert (`GuiSchemaArgs`). The R0 report must call this out and the implementer must NOT chase phantom `EncodeArgs.phrase` / `VerifyArgs.phrase` files.

### Enumeration by `<struct>.<field>` of every secret-bearing clap field

Direct, named secret-string clap fields:

| # | Struct.field | Type | Source file:line | Secret-bearing because |
|---|---|---|---|---|
| 1 | `BundleArgs.passphrase` | `Option<String>` | `cmd/bundle.rs:42` | BIP-39 passphrase |
| 2 | `VerifyBundleArgs.passphrase` | `Option<String>` | `cmd/verify_bundle.rs:42` | BIP-39 passphrase |
| 3 | `ConvertArgs.passphrase` | `Option<String>` | `cmd/convert.rs:178` | BIP-39 passphrase |
| 4 | `ConvertArgs.bip38_passphrase` | `Option<String>` | `cmd/convert.rs:188` | BIP-38 Scrypt passphrase |
| 5 | `DeriveChildArgs.passphrase` | `Option<String>` | `cmd/derive_child.rs:61` | BIP-39 passphrase |
| 6 | `BundleArgs.slot[i].value` | `String` inside `Vec<SlotInput>` | `cmd/bundle.rs:85` | secret iff subkey ∈ {phrase, entropy, xprv, wif} per `SlotSubkey::is_secret_bearing()` |
| 7 | `VerifyBundleArgs.slot[i].value` | `String` inside `Vec<SlotInput>` | `cmd/verify_bundle.rs:95` | same |
| 8 | `ConvertArgs.from[i].value` | `String` inside `Vec<FromInput>` | `cmd/convert.rs:160` | secret iff node ∈ `is_argv_secret_bearing()` (includes MiniKey) |
| 9 | `DeriveChildArgs.from.value` | `String` (singleton, not Vec) | `cmd/derive_child.rs:26` | secret iff node ∈ {Xprv, Phrase} |

Direct named secret-string clap fields (rows 1-5) = **5**. Repeating-flag value strings (rows 6-9) are variable per-invocation. Implementation surface: **5 direct pins + 3 vec-iteration pin blocks** per cmd handler — NOT 12 named fields.

→ **C-2 (conf 90):** The plan/SPEC's exact field list (`BundleArgs: passphrase, phrase, slot[*].value`) is wrong:
- There is no `BundleArgs.phrase` field. Phrases come via the slot Vec (`@N.phrase=` subkey).
- Same for `VerifyBundleArgs.phrase`.
- `ConvertArgs.bip38_passphrase` is missing from the plan's enumeration.

**Corrected Site 1 apply surface** (replace the plan's wording verbatim):

```rust
// In cmd::bundle::run, after clap parse + emit_secret_in_argv_advisories
// + apply_stdin_substitutions:
let _passphrase_pin =
    args.passphrase.as_ref().map(|p| mnemonic_toolkit::mlock::pin_pages_for(p.as_bytes()));
let _slot_pins: Vec<mnemonic_toolkit::mlock::PinnedPageRange> = args
    .slot
    .iter()
    .filter(|s| s.subkey.is_secret_bearing() && !s.is_stdin_sentinel())
    .map(|s| mnemonic_toolkit::mlock::pin_pages_for(s.value.as_bytes()))
    .collect();
```

Mirror in `cmd::verify_bundle::run`; in `cmd::convert::run` swap `slot` for `from` and use `is_argv_secret_bearing()`; in `cmd::derive_child::run` pin the singleton `args.from.value` when the node is secret-bearing plus `args.passphrase`.

→ **I-1 (conf 85):** The `--passphrase-stdin` / `--bip38-passphrase-stdin` / `--from <node>=-` / `--slot @N.<secret>=-` synthetic-args path mutates `owned.passphrase` / `owned.slot[i].value` / `owned.from.value` after `read_to_string`. Site 1 should pin the SYNTHETIC args (the post-stdin-substitution clone) too, not just the originally-parsed args. Affects:
- `cmd::bundle.rs:1227 apply_stdin_substitutions` produces `synthetic_args: BundleArgs` whose `.passphrase` / `.slot[*].value` may now hold the stdin-derived secret.
- Same for `cmd::verify_bundle.rs:565`.
- Same for `cmd::convert.rs` (lines 652-664: `effective_passphrase` / `effective_bip38_passphrase`).
- Same for `cmd::derive_child.rs:98-122` (`from_value`, `stdin_passphrase` — both already `Zeroizing<String>`).

The pin must happen AFTER the last mutation. Recommendation: lift the pin block to AFTER post-stdin substitution, into a single place per cmd handler.

---

## §2 Site 2 cascade — `ResolvedSlot`

**Plan claim:** `ResolvedSlot.entropy: Option<Zeroizing<Vec<u8>>>` (Cycle A shape).

**Actual** (`synthesize.rs:585`): `pub entropy: Option<Vec<u8>>`. Plain `Option<Vec<u8>>` — **NOT** Zeroizing.

→ **C-3 (conf 100):** SPEC §2 row 5 + SPEC §4 P3a + plan §"Phase 3a P3a.T3 step 1" all say "`ResolvedSlot` keeps `entropy: Option<Zeroizing<Vec<u8>>>` (Cycle A shape, no change)". This is FACTUALLY WRONG. Cycle A did NOT change `ResolvedSlot.entropy` to `Zeroizing<Vec<u8>>`. The field is bare `Option<Vec<u8>>`. There is also NO `impl Drop for ResolvedSlot` (verified via grep — only `DerivedAccount` got a Drop impl in Cycle A).

**Consequence:** if Phase 3a just adds an `_entropy_pin` sibling field without addressing the zeroize gap, the field drops in declaration order (entropy: `Vec<u8>` drops first → does NOT scrub bytes; then `_entropy_pin` munlocks now-non-scrubbed pages). The G4.a "zeroize-while-still-pinned" invariant SPEC §6 G4.a documents is unsatisfiable on this field as-written.

**Two options for Phase 3a:**

a. **Convert the field to `Option<Zeroizing<Vec<u8>>>`** (matching what SPEC says it already is). Cleanest path — field type is self-documenting; no risk of a future contributor adding another field that breaks the ordering.

b. **Add `impl Drop for ResolvedSlot`** that zeroizes `entropy` (mirroring `DerivedAccount` Cycle A pattern), then add `_entropy_pin` sibling. Per RFC 1857: `Drop::drop` runs first, then fields drop in declaration order. Works ordering-wise; preserves Cycle A discipline literally.

R0 must lock one. **Recommendation: (a)** for self-documentation; future contributors can't accidentally break the invariant by adding fields in wrong order.

### ResolvedSlot construction sites (cascade enumeration)

Per grep — **6 construction sites:**

| # | File:line | Notes |
|---|---|---|
| 1 | `synthesize.rs:1184` | test helper |
| 2 | `cmd/bundle.rs:348` | resolve_slots phrase arm (full mode) |
| 3 | `cmd/bundle.rs:417` | resolve_slots xpub arm (watch-only) |
| 4 | `cmd/bundle.rs:449` | resolve_slots entropy arm (full mode) |
| 5 | `cmd/bundle.rs:491` | resolve_slots wif arm (degenerate) |
| 6 | `cmd/bundle.rs:1065` | bundle_run_unified_descriptor (Phase L) post-loop reconstruction |

Each site producing `entropy: Some(...)` must also construct the new `_entropy_pin` field (sites 2, 4, 6). Sites 3, 5 produce `entropy: None` and `_entropy_pin: None`. Site 1 (test) needs same treatment.

**Mutation audit:** zero matches for `entropy.push` / `extend` / `reserve` / `resize` / `clear` / `truncate`. `ResolvedSlot.entropy` is never mutated after construction. Reallocation-immunity holds.

→ **I-2 (conf 85):** `ResolvedSlot` derives `Clone`. After Phase 3a's change, `_entropy_pin: Option<PinnedPageRange>` makes the derive fail (raw `*const u8` is not `Clone`; even if it were, two Drop impls trying to munlock the same range = UAF or double-munlock risk). Three options:
- Remove `Clone` derive (sweep callers; `cmd/bundle.rs:1062-1073` clones cosigners and resolved slots for bridging — non-trivial).
- Hand-write `Clone` such that the clone gets `_entropy_pin: None` and re-establishes pin when consumed. Silent threat-model degradation.
- Wrap `_entropy_pin` in `Option<Arc<PinnedPageRange>>` so clones share the pin and Drop runs only once.

**Recommendation: `Option<Arc<PinnedPageRange>>`** — preserves Clone semantics; pins the page exactly once.

---

## §3 Site 3 cascade — `DerivedAccount`

**Plan claim:** `DerivedAccount.entropy: Zeroizing<Vec<u8>>` (Cycle A shape).

**Actual** (`derive.rs:21`): `pub entropy: Vec<u8>`. Plain `Vec<u8>`. Cycle A wrapped the lifecycle via `impl Drop for DerivedAccount` (`derive.rs:49-58`) which calls `self.entropy.zeroize()`. Mismatch with SPEC §2 row 5 + SPEC §4 P3a + plan claim.

→ **C-4 (conf 100):** Same kind of narrative inaccuracy as C-3. The field is plain `Vec<u8>`. Cycle A used impl Drop, not Zeroizing<>. R0 must lock:

a. **Convert field to `Zeroizing<Vec<u8>>` and DELETE `impl Drop`** (since Zeroizing scrubs). Add `_entropy_pin: PinnedPageRange` sibling. The plan says "no type signature change" but this is the only way to match the SPEC's own §6 G4.a "entropy drops first via Zeroizing, then `_entropy_pin` munlocks" wording.

b. **Keep `Vec<u8>` + impl Drop, add `_entropy_pin: PinnedPageRange` AFTER the field declaration.** RFC 1857 ordering: struct's `Drop::drop` (zeroizes `self.entropy`) runs first, then fields drop in declaration order. `_entropy_pin` drops after zeroize. Then `into_parts()` becomes a hazard: it does `std::mem::take(&mut self.entropy)` which moves the Vec out; the Drop of the orphaned husk runs at end of caller's scope, scrubbing nothing meaningful. The `_entropy_pin` Drop runs when the original `DerivedAccount` drops (immediately, since `into_parts` consumes `self`). So `_entropy_pin` munlocks BEFORE the moved-out Vec is scrubbed at the caller. That's the post-munlock-pre-zeroize window the SPEC documents as acceptable for Site 4 — but here it's on every full-mode derive.

**Recommendation: (a)** — cleaner. Remove `impl Drop for DerivedAccount`, convert field. `into_parts()` body: `mem::take(&mut self.entropy)` becomes `mem::take(&mut *self.entropy)` (Deref through Zeroizing) — same outward signature.

### DerivedAccount construction sites

Per grep: **1 construction site** (`derive_slot.rs:77` inside `derive_bip32_from_entropy`). `derive_full` in `derive.rs` calls `derive_bip32_from_entropy` and returns its result — no second construction.

### DerivedAccount.entropy mutation audit

Zero `.push` / `.extend` / `.reserve` / `.resize` / `.clear` / `.truncate` matches. Only mutation: `std::mem::take(&mut self.entropy)` inside `into_parts()` (replaces with empty Vec) — happens during move-out, AFTER any pin would have been created at construction and BEFORE the moved-out Vec is re-wrapped at the caller. Pin is bound to OLD Vec's heap pages; `mem::take` replaces with `Vec::new()` (no allocation). Caller takes old Vec's bytes (still on original pinned pages) and wraps in `Some(entropy)` — bytes still on pinned pages. So `_entropy_pin: PinnedPageRange` declared as struct sibling field will Drop (munlock) when `DerivedAccount` drops (immediately after `into_parts`), but bytes have already moved to caller's `ResolvedSlot.entropy`. **Pin coverage breaks across the `into_parts` boundary** — caller needs its OWN pin on moved-out Vec's pages (Site 2's pin, fortunately).

→ **I-3 (conf 80):** Plan doesn't call out this `into_parts()` cross-boundary handoff. Without explicit pinning at BOTH `DerivedAccount` AND `ResolvedSlot`, there's a window between `into_parts` returning the Vec and caller wrapping it in a new `ResolvedSlot { entropy: Some(...), _entropy_pin: Some(pin_pages_for(...)) }` where bytes are NOT mlocked. Phase 3a impl must construct `ResolvedSlot` such that `_entropy_pin` is established AS PART of the struct literal (same statement). Locally-bound `entropy` Vec lives for two statements at most — acceptable per SPEC threat model.

---

## §4 Site 4 — bip85's 7 `format_*` functions

**Plan claim:** 7 `format_*` functions, each adds `let _pin = pin_pages_for(&entropy[..]);` immediately after the `derive_entropy(...)?` binding.

**Actual (verified by grep):** 7 functions, all bind `let entropy = derive_entropy(...)?;`:

| # | Function | Line | Entropy use pattern |
|---|---|---|---|
| 1 | `format_bip39_phrase` | `bip85.rs:73` | `entropy[..bytes]` borrowed by `Mnemonic::from_entropy_in` |
| 2 | `format_hd_seed_wif` | `bip85.rs:100` | `entropy[..32]` borrowed by `SecretKey::from_slice` |
| 3 | `format_xprv_child` | `bip85.rs:127` | `entropy[..32]` + `entropy[32..]` borrowed twice |
| 4 | `format_hex_bytes` | `bip85.rs:158` | `entropy[..num_bytes]` borrowed by `hex::encode` |
| 5 | `format_password_base64` | `bip85.rs:175` | `entropy[..]` borrowed by `base64_standard` |
| 6 | `format_password_base85` | `bip85.rs:189` | `entropy[..]` borrowed by `base85_btc` |
| 7 | `format_dice_rolls` | `bip85.rs:214` | `entropy[..]` borrowed by `shake.update`; SHAKE reader then drives a loop |

Every function uses `entropy` via borrow only (no `push`/`extend`/`reserve`) — reallocation immunity is automatic. **`derive_entropy` returns `Zeroizing<Vec<u8>>` with `out.copy_from_slice(mac.as_byte_array())` populating a `vec![0u8; 64]` allocation. No subsequent mutation.** Pin is safe immediately after the bind.

→ **Site 4 verdict: locked design works as-written.** The plan is correct here.

---

## §5 `main()` wiring for `report_at_exit`

`src/main.rs` is short (98 lines). Exit paths:

| Line | Path | Coverage by `report_at_exit()`? |
|---|---|---|
| 62 | `return ExitCode::from(if e.exit_code() == 0 { 0 } else { 64 });` (clap parse error) | NO — early return before any mlock callsite is reached; OK to skip |
| 90 | `Ok(code) => ExitCode::from(code)` (normal success) | needs wiring |
| 94 | `ExitCode::from(e.exit_code())` (toolkit error path) | needs wiring |

→ **N-1 (conf 70):** Most robust placement: wire `report_at_exit()` BEFORE the `match result` block returns. E.g.:

```rust
let exit_code = match result {
    Ok(code) => ExitCode::from(code),
    Err(e) => {
        let _ = writeln!(io::stderr(), "{}", e);
        ExitCode::from(e.exit_code())
    }
};
mnemonic_toolkit::mlock::report_at_exit();
exit_code
```

→ **N-2 (conf 60):** Plan's "as the last line of main() (before any process exit)" is ambiguous. Lock the exact code-shape above in R0.

---

## §6 Vec-reallocation-immunity audit per site

| Site | Buffer | Mutated post-pin? | Verdict |
|---|---|---|---|
| 1 (clap String) | `String` in `args.passphrase` / `slot[i].value` etc. | NO mutation in cmd/*.rs reads. EXCEPT: synthetic-args path mutates `owned.passphrase = Some(buf)` (full replacement). If pin happens AFTER substitution, new String is pinned. | Safe IFF I-1 fix applied |
| 2 (`ResolvedSlot.entropy`) | `Option<Vec<u8>>` (or `Option<Zeroizing<Vec<u8>>>` post-§2 fix) | Zero mutation matches. `Clone` derive issue (I-2). | Safe IFF I-2 resolved |
| 3 (`DerivedAccount.entropy`) | `Vec<u8>` (or `Zeroizing<Vec<u8>>` post-§3 fix) | `std::mem::take` in `into_parts` swaps in empty Vec; old bytes move to caller. (I-3) | Safe IFF I-3 resolved |
| 4 (bip85 local) | `Zeroizing<Vec<u8>>` | NO. `out.copy_from_slice` once; thereafter only borrowed. | Safe |

---

## §7 Test strategy — Phase 2 deferrals (G2.2 enomem, G2.3-release, G2.5 stderr summary)

Phase 2 deferred these because no production callsite existed. With Phase 3a's Sites 1-4 active, subprocess test vehicle is viable.

### Recommended subprocess vehicle

`mnemonic derive-child --from xprv=<canonical-test-xprv> --application bip39 --length 12 --index 0` — minimal dependency surface (no slot resolution, no descriptor parsing), fastest start-up; aligns Site 1 + Site 4.

### Three new subprocess tests (file: `crates/mnemonic-toolkit/tests/cli_mlock_g2_subprocess.rs`)

1. **`g2_2_enomem_subprocess_increments_failure_count_and_emits_summary`** — `assert_cmd::Command::cargo_bin("mnemonic")` with env `MNEMONIC_TEST_MLOCK_FAIL_MODE=enomem`. Invoke `derive-child --from xprv=... ...`. Assert: exit code 0 (mlock soft-fail does NOT propagate to ToolkitError); stderr contains SPEC §6 G2.5 5-line summary with `ENOMEM`.

2. **`g2_3_einval_release_subprocess_soft_fails`** — Same with env `einval`. Release-build subprocess test: assert exit code 0; stderr summary present with `EINVAL`. **Requires `cargo test --release` invocation in CI workflow.**

3. **`g2_5_off_no_summary_no_stderr_warning`** — Same with env `off`. Assert stderr does NOT contain `secret regions could not be locked`. Control case.

→ **I-3 alt (conf 75):** G2.3-release test needs explicit doctrine on release-build invocation. `cargo test` builds tests in debug by default; `assert_cmd::cargo_bin` invokes the binary built by `cargo build` in the same profile. To exercise release behavior, CI must run `cargo test --release --test cli_mlock_g2_subprocess` in a separate job. R0 should lock this.

### G1 integration test expansion for Sites 2-4

The existing `tests/mlock_unit.rs` tests `pin_pages_for` directly, not through Sites 2/3/4. Phase 3a should add in-process integration tests:

- `site_2_resolvedslot_construction_pins_entropy_pages` — call `cmd::bundle::resolve_slots(...)`, walk returned Vec, for each ResolvedSlot with Some(entropy) assert via `/proc/self/smaps` (Linux) that entropy buffer's page is `Locked > 0`.
- `site_3_derivedaccount_entropy_is_mlocked_during_lifetime` — call `derive::derive_full(...)`, hold DerivedAccount, assert page locked. Drop, assert munlocked.
- `site_4_bip85_format_bip39_entropy_is_mlocked_in_fn_body` — harder because `format_*` consumes entropy inside fn body. Recommendation: defer Site 4 in-process smaps test to a `#[cfg(test)]` instrumentation point, use subprocess test (#1 above) as primary G1 coverage for Site 4.

---

## §8 G7 SHA-pin discipline

Changes touch: `src/cmd/{bundle,verify_bundle,convert,derive_child}.rs`, `src/synthesize.rs`, `src/derive.rs`, `src/bip85.rs`, `src/main.rs`. None are wire-format producers.

mlock affects kernel page-pinning bookkeeping, not byte output. `pin_pages_for` returns a `PinnedPageRange` that goes into `let _ = pin` (or struct field) and is never serialized. `report_at_exit` writes stderr only; the SHA pin reproduction operates on test fixture files (not stdout).

**Risk:** if Site 3 changes `DerivedAccount.entropy` from `Vec<u8>` to `Zeroizing<Vec<u8>>` (recommended C-4 option a), `into_parts()`'s `mem::take` call needs `mem::take(&mut *self.entropy)` (Deref through Zeroizing). Mechanical; won't affect stdout.

→ Run `shasum -a 256 .../v0_X/*.txt | sort | shasum -a 256` after P3a.T3 GREEN; pins should match `81828299...` and `a381761656...`. P3a.T4 already specifies this.

**G7 verdict: PASS in principle; mechanical.**

---

## §9 Risks and open questions

1. **Critical narrative discrepancy** (C-1, C-2, C-3, C-4): plan/SPEC repeatedly use shapes that don't match the actual codebase. Same off-by-N pattern as Phase 1/Phase 2 R0. Implementer must read source.

2. **`ResolvedSlot: Clone` collision with `PinnedPageRange`** (I-2). Needs design lock before P3a.T2 RED.

3. **G2.3-release CI job shape** (I-3 alt). Subprocess release-build coverage needs explicit CI matrix entry.

4. **Site 4 in-process smaps test feasibility** — defer to subprocess (G2.* tests cover Site 4 indirectly via aggregate failure_count).

5. **Synthetic-args mutation window** (I-1). Pin must land AFTER `apply_stdin_substitutions` / `apply_slot_stdin` returns.

6. **`into_parts` cross-boundary handoff** (I-3): both `DerivedAccount._entropy_pin` AND `ResolvedSlot._entropy_pin` are needed; brief unpinned window during the move. Acceptable per SPEC threat model but must be deliberate.

---

## §10 Verdict

**RE-DRAFT.** Four Critical findings (3 narrative inaccuracies misrepresenting the Cycle A baseline; the 4th is the `EncodeArgs`/`VerifyArgs` phantom struct issue). R0 must NOT be approved until SPEC + plan are folded to match reality:

- SPEC §2 row 5: replace "`ResolvedSlot` keeps `entropy: Option<Zeroizing<Vec<u8>>>` (Cycle A shape)" with actual Cycle A state + chosen Phase 3a transformation.
- SPEC §2 row 5: replace "`DerivedAccount` keeps `entropy: Zeroizing<Vec<u8>>` (Cycle A shape)" similarly.
- SPEC §2 row 5 + §4 P3a + plan: replace "~12 fields across 6 cmd structs: BundleArgs/VerifyBundleArgs/ConvertArgs/DeriveChildArgs/EncodeArgs/VerifyArgs" with corrected 4-struct enumeration above.
- Plan P3a.T1 / P3a.T3: add Clone-vs-PinnedPageRange decision (Arc wrap recommended) and synthetic-args ordering note.
- Plan P3a.T2 (RED) + P3a.T3 (GREEN): split RED tests for Site 2/3 field-type-change-or-impl-Drop into named tests.

Once fold lands and re-review returns 0C/0I, P3a.T2 RED may proceed.
