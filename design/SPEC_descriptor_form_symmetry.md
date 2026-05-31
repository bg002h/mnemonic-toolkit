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

A user holding a raw descriptor from their hardware wallet cannot feed it to `bundle`/`verify-bundle` (the card-producing surfaces), even though the exact converter to accept it — `wallet_import/pipeline.rs:concrete_keys_to_placeholders` — already exists and is used by all 8 `import-wallet` format parsers (and is invoked, for the md_codec tree only, on `bundle`'s `--import-json` path at `bundle.rs:1645` — though that path sources its ResolvedSlots from mk1 cards, not the descriptor; see §3.2).

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
1. matches **both** probes → **mixed-form error** (`"descriptor mixes @N placeholders with inline keys; use one form"`). Explicit and precedes routing — `@\d`-first routing would otherwise silently parse the `@N` and ignore the inline keys. (R0 confirmed `key_regex` cannot match an `@N[fp/path]` annotation — no `]xpub` after the bracket, `pipeline.rs:155-156` — so this rule never false-fires on a legitimate `@N` descriptor.)
2. matches `@\d` only → `AtN`.
3. matches `key_regex` only → `Concrete`.
4. matches **neither** → the **classifier itself** emits a pinned origin-required error [folds R0-I3]: `"descriptor has neither @N placeholders nor [fp/path]-annotated keys; concrete descriptors must carry a key origin, e.g. [<fp>/84h/0h/0h]xpub…"`. md-codec's `MissingExplicitOrigin` is NOT reachable on this branch (it fires only when the converter feeds `parse_descriptor`, which a no-keys input never does), so the error must originate here. This is the origin-less single-sig case (`wpkh(xpub/0/*)`, no brackets) as well as garbage.

The `AtN` arm runs the existing per-surface slot/seed binding; the `Concrete` arm runs §3.2. Both **converge at** `parse_descriptor(&input_or_placeholder, &keys, &fps)` and nowhere earlier. The `ParsedKey` / `ParsedFingerprint` types [M2] live in §3.2's helper signature, not in the discriminant enum.

### 3.2 Inline-keys → ResolvedSlots adapter [folds C1]

The `Concrete` arm must turn the descriptor's inline keys into watch-only `ResolvedSlot`s. **R0 correction:** the pattern to follow is the import parsers, NOT `bundle.rs:1644-1659` (whose ResolvedSlots come from `envelope_to_resolved_slots`/mk1 cards at `bundle.rs:1531`, not from the descriptor). The canonical precedent is `wallet_import/bsms.rs:219-265`: `concrete_keys_to_placeholders` → `parse_descriptor` → per-slot `build_slot_fields(body,i)` (`bsms.rs:399-416`) → `extract_origin_components` (`bsms.rs:362-394`).

**The load-bearing mechanic R0 found unspecified:** `concrete_keys_to_placeholders` returns `parsed_keys: Vec<ParsedKey{i, payload:[u8;65]}>` — but the `[u8;65]` payload is a *lossy* compact xpub (pubkey+chaincode only; no depth / parent-fingerprint / child-number), and it carries **no path**. `ResolvedSlot` requires a *full* `Xpub` plus a `DerivationPath` + `Fingerprint` (`synthesize.rs:617-619`). So both the full xpub and the path must be **re-recovered from the original body's base58**, exactly as `build_slot_fields` does: `slip0132::normalize_xpub_prefix(xpub_str)` → `Xpub::from_str`, and `DerivationPath::from_str("m"+path)`.

```rust
/// Bare-concrete descriptor body → (parsed md_codec Descriptor, watch-only ResolvedSlots).
/// Co-located with concrete_keys_to_placeholders in wallet_import/pipeline.rs.
/// Reused by bundle + verify-bundle (export-wallet's @N path errors, needs none).
fn descriptor_concrete_to_resolved_slots(
    descriptor_body: &str,                 // checksum-stripped
) -> Result<(md_codec::Descriptor, Vec<ResolvedSlot>), ToolkitError>;
```

Implementation:
1. `(placeholder_form, keys, fps) = concrete_keys_to_placeholders(body)` — md_codec tree input.
2. `descriptor = parse_descriptor(&placeholder_form, &keys, &fps)`.
3. **Recover `(fp, path, xpub_str)` per key in one pass over `body` using the WIDENED `key_regex` (§3.3)** — group 1 = fp-hex, group 2 = path, group 3 = xpub literal. This is the §3.3 fix's whole point: driving recovery off `key_regex` (not the import parsers' separate `origin_capture_regex`) keeps the new path `h`-tolerant without touching the 4 `origin_capture_regex` copies [folds C2-option-(b)].
4. Per key: `fp = Fingerprint::from(<group1 bytes>)`; `path = DerivationPath::from_str("m"+group2)`; `(neutral,_) = slip0132::normalize_xpub_prefix(group3)`; `xpub = Xpub::from_str(&neutral)`; push `ResolvedSlot { xpub, fingerprint: fp, path, entropy: None, master_xpub: None, _entropy_pin: None }` (watch-only — `entropy: None`). `debug_assert_eq!(xpub_to_65(&xpub), keys[i].payload)` mirrors `bsms.rs:256`.
5. Return `(descriptor, resolved_slots)`.

This duplicates the `build_slot_fields`/`extract_origin_components` shape (a 7th copy); consolidating all of them onto `key_regex` is deferred — see §9 FOLLOWUP `descriptor-origin-extraction-dedup`. The existing `--import-json` block is **not** refactored onto this helper (its slots come from cards, not the descriptor — the earlier framing was wrong).

**Placement / imports / checksum [M-i, M-ii, M-iii, M1].** The helper, `classify_descriptor_form`, and a **NEW** `AT_N_PROBE` constant (`Regex::new(r"@\d")`, introduced by this feature — it does not pre-exist) are co-located in `wallet_import/pipeline.rs` (`pub(crate)`), reaching the private `fn key_regex()` directly. export-wallet's `@N`-only guard (§3.4) also calls `AT_N_PROBE`. The helper adds in-crate `use`s already used by sibling `bsms.rs` (no layering violation): `crate::synthesize::{ResolvedSlot, xpub_to_65}`, `md_codec::Descriptor`, `crate::parse_descriptor::parse_descriptor`, `crate::slip0132::normalize_xpub_prefix`. **Checksum:** the helper takes a checksum-stripped body; the caller (each surface's `Concrete` arm) strips the trailing `#<csum>` via `descriptor_body_no_csum` (`json_envelope.rs:448`) first — required because `concrete_keys_to_placeholders` rewrites the body to `@N` form, which would invalidate the original BIP-380 checksum that `parse_descriptor`'s inner `MsDescriptor::from_str` validates.

### 3.3 `h`-form hardened-path fix [folds C2]

`concrete_keys_to_placeholders`'s `key_regex` (`pipeline.rs:38`) accepts only apostrophe-hardened paths:

```
\[([0-9a-fA-F]{8})((?:/\d+'?)+)\]([xtyzuvYZUV]pub[A-HJ-NP-Za-km-z1-9]+)
```

Bitcoin Core `listdescriptors`, Sparrow, and others emit `h`-form (`[fp/84h/0h/0h]xpub…`). Widen the path group to match `lex_placeholders`' own `(?:'|h)?` tolerance exactly [M1 — `h` only, no `H`; live `lex_placeholders` at `parse_descriptor.rs:70` is `(?:'|h)?`]:

```
((?:/\d+(?:'|h)?)+)
```

**Scope of the widening [folds C2-option-(b)]:** widen **only** `key_regex` (`pipeline.rs:38`). Both the §3.2 recovery (step 3) and `concrete_keys_to_placeholders` drive off `key_regex`, so this single change makes the entire new `--descriptor` path `h`-tolerant. The import parsers' separate, byte-identical `origin_capture_regex` copies (`bsms.rs:516`, `specter.rs:362`, `sparrow.rs:582`, `bitcoin_core.rs:413`) are **left untouched** — the new helper does not call `extract_origin_components`, so they are irrelevant to this feature. (Whether the *import-wallet* parsers should also accept `h`-form wallet files is pre-existing scope → §9 FOLLOWUP `import-parser-hform-origin-tolerance`.) Existing import behavior is unchanged: the widening is a strict superset (apostrophe still matches), proven by a Phase-0 unit test asserting both forms.

**Error-prefix remap:** the converter's hardcoded `"import-wallet: bsms: parse error: …"` strings (`pipeline.rs`, in the `ImportWalletParse(format!())` bodies at `:74-76, :80-83, :107-110, :116-119`) [M2 — re-pinned] leak the wrong command name when reached from `bundle`/`verify-bundle`. The new entry points map converter errors to a neutral `ToolkitError::DescriptorParse` with command-appropriate context (precedent: `electrum.rs:373` rewrites `bsms:`→its name), rather than surfacing the raw `bsms:` string.

### 3.4 Per-surface behavior + wiring [folds I1, I2]

| Surface | `@N` form | bare concrete |
|---|---|---|
| `bundle --descriptor` | unchanged (keys from `--slot @N.phrase=` / `--ms1`, or watch-only via `--slot @N.xpub=`) | **NEW** → `descriptor_concrete_to_resolved_slots` → watch-only md1+mk1 (no ms1) |
| `verify-bundle --descriptor` | unchanged (keys bound from `--slot`) | **NEW** → extract keys, verify cards against them |
| `export-wallet --descriptor` | **NEW** → clear error: `"export-wallet --descriptor accepts only concrete descriptors with inline keys; for keyless @N templates use --template <T> --slot @N.xpub=… or --from-import-json"` | unchanged (passthrough) |

**Wiring [folds R0-I1 + R1-C-A].** A bare-concrete `bundle --descriptor` carries NO `--slot`; if it entered `bundle_run_unified` it would die at `detect_bundle_mode`'s empty-slot gate (`bundle.rs:313` → `bundle_unified.rs:35-40`) **before** any descriptor handling. The `@N` descriptor path additionally requires `--slot` per `@N` and is deeply coupled to default-path inference + Shared/Divergent collapse (`bundle_run_unified_descriptor`, `bundle.rs:1130-1224`). So the `Concrete` arm must **early-fork in `run()`**, exactly as `--import-json` does — NOT at the `:338` dispatch (which is unreachable for the no-slot case).

- **Insertion point [R1-C-A, R2-I2]:** in `run()`, immediately after the `--import-json` early fork (`bundle.rs:223` `if args.import_json.is_some() { return bundle_run_from_import_json(...) }`) and the existing `descriptor_mode` computation (`bundle.rs:227`): if `descriptor_mode`, **first materialize the body** — `let body = match (&args.descriptor, &args.descriptor_file) { (Some(s),_) => s.clone(), (_,Some(f)) => std::fs::read_to_string(f)?.trim_end().to_string(), … }` (the read at `bundle.rs:227` is NEW — the existing `:1056-1064` read lives inside `bundle_run_unified_descriptor`, off the Concrete path) — then `if classify_descriptor_form(&body)? == Concrete → return bundle_run_concrete_descriptor(args, body, …)`, **before** `bundle_run_unified` is entered. The `@N` descriptor (`AtN`) falls through to `bundle_run_unified` → `bundle_run_unified_descriptor` unchanged (which re-reads the file at `:1058` as today — a negligible double-read on the `--descriptor-file` `@N` case).
- **`bundle_run_concrete_descriptor` (NEW):** receives the already-read `body` → strip checksum (`descriptor_body_no_csum`, §3.2) → `descriptor_concrete_to_resolved_slots(body)` → **`check_resolved_slots_distinctness(&resolved_slots)` [R1-I-A — `bundle.rs:402`, the `&[ResolvedSlot]` variant; template runs it at `:367`, `@N` runs `check_key_vector_distinctness` at `:1407`; the `from_import_json` tail does NOT (trusted cards) — so it must be added here or duplicate cosigners pass concrete while failing `@N`]** → `synthesize_descriptor(&descriptor, &resolved_slots, args.privacy_preserving)`. **BundleMode selection [M2]:** reuse the `bundle_run_from_import_json` selector `match (n, any_secret, any_watch)` (`bundle.rs:1664-1670`) verbatim — since every Concrete slot is `entropy:None`, `any_secret` is always false and it collapses to `SingleSigWatchOnly` / `MultisigWatchOnly`. **Emit [M3]:** the real descriptor is already in `args.descriptor`/`args.descriptor_file`, so `emit_unified`'s `descriptor_field` (`:802-808`) picks it up — do NOT copy `from_import_json`'s synthetic `emit_args.descriptor` injection (`:1680-1681`). (`validate_watch_only_resolved` is tautological — every slot `entropy:None` — optional/cosmetic; distinctness is load-bearing.)
- **verify-bundle:** the analogous fork ahead of `lex_placeholders` (`verify_bundle.rs:614`; verify-bundle uses `validate_slot_set` which is Ok on empty, so no empty-slot gate precedes it — the `:614` fork is reachable and post-read, `descriptor_str` at `:603-612`): `AtN` → existing slot-bound verify; `Concrete` → `descriptor_concrete_to_resolved_slots` + **`check_resolved_slots_distinctness`** (mirror), then verify the bundle's cards against the extracted slots (the concrete descriptor is the expected value; the cards are the subject).
- **export-wallet [R2-I1 — branch on the `@N` probe ALONE, NOT the rule-4-bearing classifier]:** export-wallet's `--descriptor` (`export_wallet.rs:328-334`) accepts origin-LESS concrete (`wpkh(xpub/0/*)`) via bare `MsDescriptor::from_str` — its md-codec backend has no origin requirement — so it must NOT call `classify_descriptor_form` (whose rule 4 would reject those). Guard with the `@N` probe only: `if AT_N_PROBE.is_match(desc) → redirect error`; else → the existing `MsDescriptor::from_str` passthrough (handles concrete with/without origins; errors on garbage via miniscript). Only bundle/verify-bundle (md1/BIP-388 needs origins) surface rule 4.

### 3.5 Error handling (all strings byte-pinned in tests) [folds I4]

- Mixed `@N`+inline-xpub → `DescriptorParse` "mixes @N placeholders with inline keys".
- Origin-less key (`wpkh(xpub/0/*)`, no brackets) → the **classifier's** rule-4 origin-required error (§3.1) — emitted by the classifier itself; md-codec's `MissingExplicitOrigin` is unreachable on the no-keys branch [folds R0-I3].
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
6. **Convergence [I2]:** out-of-lexicographic-order `sortedmulti`, **both inputs explicitly origin-bearing** [M3 — else `@N` default-path inference (`bundle.rs:1130-1224`) could diverge from explicit-origin concrete]: `bundle --descriptor <concrete with [fp/path]>` vs `bundle --descriptor <@N[fp/path]> --slot @N.xpub=…` → byte-identical md1/mk1 cards. Home: `tests/cli_wallet_cross_format_convergence.rs`.
7. **Convergence [M3]:** concrete `tr(NUMS, leaf-xpubs)` (Coldcard taproot-multisig shape) → NUMS untouched by `key_regex`, leaf xpubs bound; same cards as `@N` equivalent.
8. **No-secret invariant:** bare-concrete `bundle`/`verify-bundle` stdout never carries xprv/seed (watch-only).
9. **Distinctness [R1-I-A]:** a concrete `sortedmulti` with two identical `(xpub, path)` cosigners → `bundle --descriptor` rejects with `Bip388Distinctness` (parity with the `@N`+`--slot` path), exit code pinned. Same assertion for `verify-bundle --descriptor`.

## 6. Phases (for the plan-doc)

- **Phase 0** — C2: widen `key_regex` to `h`-form + error-prefix remap seam. Tests 1.
- **Phase 1** — `classify_descriptor_form` + mixed/origin-less guards. Tests 2.
- **Phase 2** — `descriptor_concrete_to_resolved_slots` helper in `wallet_import/pipeline.rs` (mirrors `bsms.rs:219-265` + `build_slot_fields`; recovers full xpub+path via the widened `key_regex` per §3.2). Unit test: a 2-of-3 concrete body → 3 watch-only `ResolvedSlot`s with correct typed `(xpub, fp, path)`. (NOT a refactor of `--import-json` — its slots come from cards.)
- **Phase 3** — classifier early-fork in `run()` (~`bundle.rs:223`, after the `--import-json` fork) + `verify_bundle.rs:614` fork; new `bundle_run_concrete_descriptor` (strip checksum → helper → `check_resolved_slots_distinctness` → synth, mirroring the `bundle_run_from_import_json` tail); verify Concrete arm mirrors distinctness. Tests 3,4,8,9.
- **Phase 4** — export-wallet `@N` redirect error. Tests 5.
- **Phase 5** — convergence cells. Tests 6,7.
- **Phase 6** — manual prose (3 `--descriptor` blocks) + any execution-gated example; version bump v0.38.1; end-of-cycle R0.

## 7. Risks

- **R1 — `h`-form widening of `key_regex` on a hot path.** All 8 import parsers call `concrete_keys_to_placeholders` (→ `key_regex`); the widening is a strict superset (apostrophe still matches) so their behavior is unchanged for existing input, but it now ALSO accepts `h`-form on the import path — a benign broadening. Mitigated by a Phase-0 unit test asserting both forms + the existing import integration suite staying green. (The parsers' separate `origin_capture_regex` is NOT widened, so their *path*-recovery stays apostrophe-only — a real but pre-existing asymmetry, filed as `import-parser-hform-origin-tolerance`.)
- **R2 — origin-less single-sig descriptors.** Real single-sig descriptors sometimes ship origin-less (`wpkh(xpub/0/*)`). Per R0-I3 these hit classifier rule 4 → the classifier's pinned origin-required error (md1/BIP-388 needs origins). Documented as a known limitation; not silently mis-parsed.
- **R3 — `bundle_run_concrete_descriptor` divergence from `bundle_run_from_import_json`.** The new fn mirrors the import-json synthesis tail but **deliberately adds `check_resolved_slots_distinctness`** [R1-I-A] — the tail omits it (trusted cards) but a user-pasted descriptor is untrusted and must match the `@N`/template paths' BIP-388 invariant (Test 9). Convergence cells (Tests 6,7) guard byte-identity against the `@N`+`--slot` path; no live path is refactored (the earlier "extract from `--import-json`" plan was dropped per R0-C1).

## 8. Open questions
None blocking. (export-wallet `@N` full resolution and origin-less-key auto-synthesis are explicitly deferred.)

## 9. Filed follow-ons
- `output-type-stderr-advisory` — the next cycle (B). Stderr one-line classification of what landed on stdout: `private key material (can spend)` / `watch-only` / `template`. Subsumes the D9 secret-on-stdout advisory by complement (keep D9's exact pinned text; add positive lines; one shared helper; consolidate the 4 inlined literals). All ~12 output-producing commands or none (half-coverage is worse than none). PATCH; transcript re-capture is the bulk. See the brainstorm-stage architect review §(B) for the per-surface feasibility table.
- `descriptor-origin-extraction-dedup` [M4] — `build_slot_fields` is duplicated in 6 import parsers and `extract_origin_components` in 4; §3.2 adds a 7th origin-extraction (reusing `key_regex`). Consolidate all onto the single canonical (widened) `key_regex` + one shared `extract_origin_components`/`build_slot_fields` in `pipeline.rs`, parameterizing the per-parser error prefix. Deferred (per-parser error-prefix differences make it a moderate refactor; out of scope for this PATCH).
- `import-parser-hform-origin-tolerance` — the import-wallet parsers' `origin_capture_regex` copies (`bsms.rs:516` et al.) remain apostrophe-only; whether `import-wallet` should also accept `h`-form wallet-file descriptors is pre-existing scope, deferred. (Dissolved automatically if `descriptor-origin-extraction-dedup` lands.)
