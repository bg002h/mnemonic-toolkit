# SPEC — descriptor-form symmetry (Theme A / A1)

**Status:** brainstorm-approved; pending formal SPEC R0.
**Cycle:** A1 of Theme A "wallet interop". **SemVer:** PATCH (toolkit v0.38.1).
**Source SHA at write time:** `ea8ba88` (origin/master). Citations re-grepped at write time per CLAUDE.md.
**Brainstorm-stage architect review:** `design/agent-reports/descriptor-form-symmetry-brainstorm-R0-review.md` (2C/3I/3M, all folded below).

---

## 1. Problem

A wallet descriptor reaches the toolkit's three descriptor-taking surfaces in one of two string forms:

- **`@N` annotated form** — `wsh(sortedmulti(2,@0[fp/84h/0h/0h]/<0;1>/*,@1[…]/<0;1>/*))`. The toolkit's keyless template: the `@N` lexer (`parse_descriptor.rs:lex_placeholders`) carries an optional `[fp/path]` origin but no xpub; keys are sourced separately per-surface.
- **bare concrete form** — `wsh(sortedmulti(2,[fp/84h/0h/0h]xpub…/<0;1>/*,[…]ypub…/<0;1>/*))`. What real wallets (Sparrow, Coldcard, Bitcoin Core `listdescriptors`, Electrum) emit; xpubs inline.

Today the surfaces accept these **asymmetrically** (verified at `ea8ba88`):

| Surface | `@N` form | bare concrete |
|---|---|---|
| `bundle --descriptor` / `--descriptor-file` | ✅ `lex_placeholders` (`bundle.rs:1068`) | ❌ rejected (`parse_descriptor.rs:136` "descriptor must contain at least one @N placeholder.") |
| `verify-bundle --descriptor` / `--descriptor-file` | ✅ `lex_placeholders` (`verify_bundle.rs:614`) | ❌ rejected (same) |
| `export-wallet --descriptor` | ❌ rejected (`MsDescriptor::from_str`, `export_wallet.rs:332` — miniscript can't parse `@0`) | ✅ passthrough (`export_wallet.rs:332-334`) |

A user holding a raw descriptor from their hardware wallet cannot feed it to `bundle`/`verify-bundle` (the card-producing surfaces), even though the exact converter to accept it — `wallet_import/pipeline.rs:concrete_keys_to_placeholders` — already exists and is used by all 8 `import-wallet` format parsers, and is already wired into `bundle`'s `--import-json` path (`bundle.rs:1644-1656`).

## 2. Goal

Make the **bare concrete form** work on every card/verify surface, so `bundle --descriptor "<bare concrete>"` becomes the "descriptor → engravable cards" door, and `verify-bundle --descriptor "<bare concrete>"` verifies cards against a real descriptor.

Per the brainstorm-stage architect review (I1), **`export-wallet` is NOT broadened to resolve `@N`** — it stays concrete-only (already works) and gains a clear, actionable error on `@N` (the mode-dependent `--slot` coupling and `--from-import-json` overlap make full resolution scope-creep). Consistency is achieved by: bundle + verify-bundle accept both forms; export-wallet accepts concrete + redirects `@N`.

**Non-goals.** No new clap flag/subcommand/value. No `import-wallet --format descriptor` (redundant with broadened `bundle --descriptor`). No new taproot scope. The stderr output-type advisory is a **separate following cycle** (filed; see §9).

## 3. Design

### 3.1 The seam — a dispatch fork (NOT a uniform enum) [folds C1]

A single classifier at the descriptor-parse boundary returns a **payload-free discriminant** via cheap regex probes; each surface **branches**, because the two forms have different downstream key-binding pipelines (the `@N` arm sources keys from `--slot`/seed/cards; the `Concrete` arm carries keys extracted from the descriptor itself). The classifier does **not** run the converter — that happens once, in the §3.2 helper, only on the `Concrete` branch — so there is no double-conversion:

```rust
/// Which descriptor form an input string is. Discriminant only — no payload.
enum DescriptorForm { AtN, Concrete }

/// Classify a user descriptor string via cheap probes. Pure; no I/O; no conversion.
fn classify_descriptor_form(input: &str) -> Result<DescriptorForm, ToolkitError>;
```

Classification rule (two probes: `AT_N_PROBE` = `@\d`; `key_regex()` = inline `[fp/path]xpub`):
1. matches **both** probes → **mixed-form error** [folds I3(b)] (`"descriptor mixes @N placeholders with inline keys; use one form"`). Explicit and precedes routing — `@\d`-first routing would otherwise silently parse the `@N` and ignore the inline keys.
2. matches `@\d` only → `AtN`.
3. matches `key_regex` only → `Concrete`.
4. matches neither → the origin-required / no-keys error, surfaced through md-codec's existing origin policy consistently for single-sig and multisig [folds I3(a)] — not a silent regex non-match.

The `AtN` arm runs the existing per-surface slot/seed binding; the `Concrete` arm runs §3.2. Both **converge at** `parse_descriptor(&input_or_placeholder, &keys, &fps)` and nowhere earlier. The `ParsedKey` / `ParsedFingerprint` types [M2] live in §3.2's helper signature, not in the discriminant enum.

### 3.2 Shared inline-keys → ResolvedSlots adapter [folds C1]

The `Concrete` arm needs to turn its extracted inline xpubs into `ResolvedSlot`s (watch-only). `bundle.rs:1644-1659` already has this shape on the `--import-json` path. Lift it into a shared helper:

```rust
/// Bare-concrete descriptor body → (parsed md_codec Descriptor, watch-only ResolvedSlots).
/// Reused by bundle + verify-bundle (and the export-wallet error path needs none).
fn descriptor_concrete_to_resolved_slots(
    descriptor_body: &str,
) -> Result<(md_codec::Descriptor, Vec<ResolvedSlot>), ToolkitError>;
```

It calls `concrete_keys_to_placeholders` → `parse_descriptor` → builds watch-only `ResolvedSlot`s from the inline `(fp, path, xpub)` triples. `bundle`'s existing `--import-json` block is refactored to call this helper (no behavior change there — pure extraction).

### 3.3 `h`-form hardened-path fix [folds C2]

`concrete_keys_to_placeholders`'s `key_regex` (`pipeline.rs:38`) accepts only apostrophe-hardened paths:

```
\[([0-9a-fA-F]{8})((?:/\d+'?)+)\]([xtyzuvYZUV]pub[A-HJ-NP-Za-km-z1-9]+)
```

Bitcoin Core `listdescriptors`, Sparrow, and others emit `h`-form (`[fp/84h/0h/0h]xpub…`). Widen the path group to match `lex_placeholders`' own `(?:'|h)?` tolerance:

```
((?:/\d+(?:'|h|H)?)+)
```

This is a single-source-of-truth fix benefiting the 8 existing import callers too. It is covered by a focused unit test (the converter is on a hot import path). **Error-prefix remap:** the converter's hardcoded `"import-wallet: bsms: parse error: …"` strings (`pipeline.rs:75,81,108,117`) leak the wrong command name when reached from `bundle`/`verify-bundle`. The new entry points remap the prefix to the calling command (precedent: `electrum.rs:373` rewrites `bsms:`→its name). Implementation: the classifier maps converter errors to a neutral `ToolkitError::DescriptorParse` with command-appropriate context, rather than surfacing the raw `bsms:` string.

### 3.4 Per-surface behavior

| Surface | `@N` form | bare concrete |
|---|---|---|
| `bundle --descriptor` | unchanged (keys from seed via `--phrase`, or watch-only) | **NEW** → `descriptor_concrete_to_resolved_slots` → watch-only md1+mk1 (no ms1, no `--phrase` required — the `SingleSigWatchOnly`/`MultisigWatchOnly` modes at `bundle.rs:1666/1669` already exist) |
| `verify-bundle --descriptor` | unchanged (keys bound from `--slot`) | **NEW** → extract keys, verify cards against them |
| `export-wallet --descriptor` | **NEW** → clear error: `"export-wallet --descriptor accepts only concrete descriptors with inline keys; for keyless @N templates use --template <T> --slot @N.xpub=… or --from-import-json"` [I1] | unchanged (passthrough) |

### 3.5 Error handling (all strings byte-pinned in tests) [folds I4]

- Mixed `@N`+inline-xpub → `DescriptorParse` "mixes @N placeholders with inline keys".
- Origin-less concrete key → md-codec origin-required policy error (consistent single/multisig).
- export-wallet `@N` → the redirect error in §3.4.
- `h`-form now accepted (no error).
- SLIP-0132 prefixes (ypub/zpub/…) in concrete input → normalized by the converter (already supported), no error.

## 4. SemVer / lockstep [folds I4, M1]

- **PATCH** (v0.38.1): broadens *acceptance* of existing `--descriptor` flags; no new clap flag/value/subcommand.
- **GUI `schema_mirror`:** untouched (flag-NAME parity only; nothing added). **No GUI lockstep.**
- **`--json` wire-shapes:** unchanged — `Concrete` normalizes to the same internal `@N`→md1 representation, so output is byte-identical to the equivalent `@N`+keys input.
- **`cli-subcommands.list` / flag-coverage lint:** untouched (no subcommand/flag added) [M1]. State explicitly so R0 does not re-litigate.
- **Manual mirror (required):** the three `--descriptor` prose blocks under `docs/manual/src/40-cli-reference/` (`41-mnemonic.md` bundle/verify-bundle/export-wallet) document one accepted form each and must update to state both-forms (bundle/verify-bundle) / concrete-only-with-redirect (export-wallet). Any chapter-45/40 worked example that now round-trips is execution-gated by `docs/manual/tests/verify-examples.sh` — capture, not author.

## 5. Testing

TDD, tests-before-impl per phase. Targets:

1. **Unit (`pipeline.rs` C2):** `h`-form `[fp/84h/0h/0h]xpub…` now parses; apostrophe form still parses; error-prefix no longer says `bsms:` when remapped.
2. **Unit (`classify_descriptor_form`):** `@N`→`AtN`; bare concrete→`Concrete`; mixed→error; origin-less→origin policy error; empty/garbage→error.
3. **Integration `bundle --descriptor <bare concrete>`:** single-sig + multisig (sortedmulti, sh-wsh) → watch-only md1+mk1 cards, exit 0, no ms1.
4. **Integration `verify-bundle --descriptor <bare concrete>`:** verifies a bundle produced from the equivalent `@N`+`--slot`.
5. **Integration `export-wallet --descriptor <@N>`:** the §3.4 redirect error, exit code pinned.
6. **Convergence [I2]:** out-of-lexicographic-order `sortedmulti`: `bundle --descriptor <concrete>` vs `bundle --descriptor <@N> --slot @N.xpub=…` → byte-identical md1/mk1 cards. Home: `tests/cli_wallet_cross_format_convergence.rs`.
7. **Convergence [M3]:** concrete `tr(NUMS, leaf-xpubs)` (Coldcard taproot-multisig shape) → NUMS untouched by `key_regex`, leaf xpubs bound; same cards as `@N` equivalent.
8. **No-secret invariant:** bare-concrete `bundle`/`verify-bundle` stdout never carries xprv/seed (watch-only).

## 6. Phases (for the plan-doc)

- **Phase 0** — C2: widen `key_regex` to `h`-form + error-prefix remap seam. Tests 1.
- **Phase 1** — `classify_descriptor_form` + mixed/origin-less guards. Tests 2.
- **Phase 2** — `descriptor_concrete_to_resolved_slots` shared helper (extract from `bundle.rs:1644-1659`); refactor the `--import-json` caller onto it. Regression: existing `--import-json` tests stay green.
- **Phase 3** — wire `bundle` + `verify-bundle --descriptor` through the classifier. Tests 3,4,8.
- **Phase 4** — export-wallet `@N` redirect error. Tests 5.
- **Phase 5** — convergence cells. Tests 6,7.
- **Phase 6** — manual prose (3 `--descriptor` blocks) + any execution-gated example; version bump v0.38.1; end-of-cycle R0.

## 7. Risks

- **R1 — `h`-form widening on a hot import path.** The 8 import parsers call `concrete_keys_to_placeholders`; widening the regex must not change their behavior for apostrophe input. Mitigated by Phase-0 unit tests asserting both forms + the existing import integration suite staying green.
- **R2 — origin-less single-sig descriptors.** Real single-sig descriptors sometimes ship origin-less (`wpkh(xpub/0/*)`). Per I3, these error via md-codec's origin policy (md1/BIP-388 needs origins). Documented as a known limitation; not silently mis-parsed.
- **R3 — refactor regression in `bundle --import-json`.** Phase 2 extracts a helper from a live path; covered by the existing convergence + import-json suites.

## 8. Open questions
None blocking. (export-wallet `@N` full resolution and origin-less-key auto-synthesis are explicitly deferred.)

## 9. Filed follow-on
`output-type-stderr-advisory` — the next cycle (B). Stderr one-line classification of what landed on stdout: `private key material (can spend)` / `watch-only` / `template`. Subsumes the D9 secret-on-stdout advisory by complement (keep D9's exact pinned text; add positive lines; one shared helper; consolidate the 4 inlined literals). All ~12 output-producing commands or none (half-coverage is worse than none). PATCH; transcript re-capture is the bulk. See the brainstorm-stage architect review §(B) for the per-surface feasibility table.
