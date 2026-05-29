# SPEC — `path_raw` bracketed-vs-bare convention unification (delete the overloaded field)

**Status:** R0-GATE GREEN (R0 RED 2C/4I/4M → R1 RED 0C/1I/2M → R2 GREEN 0C/0I; reviews persisted to `design/agent-reports/path-raw-unification-spec-R{0,1}-review.md` + R2). Cleared for implementation.
**Resolves FOLLOWUP:** `path-raw-bracketed-vs-bare-convention-unification`
**Source ground-truth SHA:** `dd7c228` (origin/master at authoring; all line numbers below grep-verified against it)
**SemVer:** PATCH (`mnemonic-toolkit-v0.37.9`) — see §10.
**Recon:** `cycle-prep-recon-path-raw-bracketed-vs-bare-convention-unification.md`
**Design provenance:** two opus architect consultations + independent grep-verification (all load-bearing claims confirmed; see §11).

---

## §1 Problem

`ResolvedSlot.path_raw: String` (`crates/mnemonic-toolkit/src/synthesize.rs:595`; the struct is aliased `pub type CosignerKeyInfo = ResolvedSlot;` at `synthesize.rs:190`) is an **overloaded** field carrying two mutually-incompatible string conventions:

- **Bracketed** `[fp/path]` (e.g. `[b8688df1/48'/0'/0'/2']`) — written by the entire import/foreign-format surface (~8 producer sites).
- **Bare** `m/path` or bare components — written only by the native seed-/descriptor-mode path (`cmd/bundle.rs::resolve_slots` + `parse_descriptor::push_binding`).

This overload directly caused bug **F5** (v0.37.7): `export-wallet --from-import-json` corrupted cosigner derivations because emitters read `.path_raw` expecting bare but received bracketed. F5 was band-aided at a single boundary — `cmd/export_wallet.rs:646-648` mutates `s.path_raw = format!("m/{}", s.path)` before the emitters run.

### §1.1 The still-live cosmetic bug (confirmed, currently unasserted)

`bundle --import-json --json` emits a polluted origin path. Confirmed live at `dd7c228`:

```
$ ./target/release/mnemonic bundle --import-json \
    crates/mnemonic-toolkit/tests/fixtures/wallet_import/envelope_v0_27_0.json \
    --network mainnet --json | jq '.multisig.cosigners[].origin_path'
"m/[b8688df1/48'/0'/0'/2']"
"m/[5436d724/48'/0'/0'/2']"
"m/[28645006/48'/0'/0'/2']"
```

Root cause: `mk1_card_to_resolved_slot` (`wallet_import/json_envelope.rs:350-352`) stores bracketed `path_raw`; the JSON cosigner emit (`bundle.rs:767`) calls `normalize_origin_path(&s.path_raw)` (`bundle.rs:731`) which prepends `m/` to any non-`m/`-prefixed string → `"m/[fp/path]"`. The fingerprint is **also** carried in the sibling `master_fingerprint` field, so the embedded fp is redundant pollution. **No test pins this output today** (`cli_bundle_import_json.rs` Cell 2 asserts only fingerprints + cosigner count).

---

## §2 Decision: delete `path_raw` (option b-strong)

`ResolvedSlot` already carries the unambiguous source of truth: `fingerprint: Fingerprint` (`synthesize.rs:593`) and typed `path: DerivationPath` (`synthesize.rs:594`). `path_raw` is a **denormalized cache** of `[fingerprint/path]`. We delete the field and replace it with two render methods that derive deterministically from `fingerprint` + `path`. This dissolves both the F5 band-aid and the §1.1 cosmetic bug **structurally** — there is no longer an overloaded string to carry the wrong convention into an emitter. (Per the project lesson *derive from the unambiguous source, not a lossy projection*.)

### §2.1 Why this is safe — four verified pillars

1. **No SemVer cost (PATCH).** `ResolvedSlot` is binary-private: `main.rs:27` declares `mod synthesize;` (no `pub`), and `lib.rs` does not re-export it. `CosignerKeyInfo` is a private `pub type` alias. No external crate can name the type → field deletion is not a public-API break. (`grep` confirmed: zero `ResolvedSlot`/`CosignerKeyInfo` hits in `lib.rs`.)

2. **`path_raw` is pure denormalization — the "round-trip fidelity" intent is STALE.** The struct doc-comment (`synthesize.rs:588-590`) and `parse_descriptor.rs:1201-1203` both claim `path_raw` "preserves the user-supplied raw path string for SPEC §4.11.b raw-equality." That contract was **deliberately reversed in v0.5**: distinctness now compares typed `.path` (`check_key_vector_distinctness`, `parse_descriptor.rs:1207-1219` → `cs[i].path == cs[j].path`), and the test `bip388_h_vs_apostrophe_paths_collide_under_typed_equality_v0_5` (`parse_descriptor.rs:1931`) explicitly asserts that `48h/0h/0h/2h` (in `path_raw`) and `48'/0'/0'/2'` (in typed `path`) **collide** — i.e. `path_raw`'s `h`-notation is NOT consulted. The only helper that decouples `path_raw` from `path.to_string()` (`cinfo_raw`, `parse_descriptor.rs:1843`) exists *solely* to prove non-consultation. No production consumer reads `path_raw` for source-byte fidelity (see §5 — every consumer is bare-deriving or display-only).

3. **`path_raw.is_empty()` ⟺ `path == DerivationPath::default()` at every producer.** The only `path == default()` producer is the WIF/degenerate slot (`bundle.rs:674-678`: `path: DerivationPath::default()` + `path_raw: String::new()`). Every bracketed import producer builds `path` and `path_raw` from the *same* captured inner string, so a non-empty bracket ⟹ non-default path. No producer sets non-empty `path_raw` with default `path` (or vice-versa). `bitcoin 0.32.8` (pinned, `Cargo.lock`) renders `DerivationPath::default().to_string() == ""`. So the absent-path sentinel can key on `path == DerivationPath::default()` — no `Option<DerivationPath>`/`has_origin` discriminator needed. (Pinned in a unit test, §8 T5.)

4. **Bracket-fp always equals `slot.fingerprint`.** Every bracketed producer derives both the bracket fp and the struct `fingerprint` from one binding. The flagged risk case — `mk1_card_to_resolved_slot`'s `origin_fingerprint`-absent substitution (`json_envelope.rs:337-348`) — computes the substituted fp once, then feeds **both** `path_raw` (`:350-352`) and `ResolvedSlot.fingerprint` (`:360`). So `bracketed_origin()` rebuilt from `self.fingerprint` reproduces every current bracket byte-for-byte **for all path-sensitive consumers; fingerprint casing is normalized to lowercase** (R0 M-1). The import capture regex permits uppercase-hex (`[0-9a-fA-F]{8}`), so a foreign descriptor with `[ABCD1234/…]` would have its rebuilt bracket lowercased — benign, because every emit consumer (C4 post-fix, C11) already lowercases or discards the bracket fp.

---

## §3 New method surface (`impl ResolvedSlot`, `synthesize.rs:622`)

```rust
impl ResolvedSlot {
    /// Bare BIP-32 derivation path in `m/...` form, or `""` for the
    /// pathless/degenerate slot (`path == DerivationPath::default()`, e.g. the
    /// WIF slot). Replaces the former bare-convention reads of `path_raw`.
    /// The `""` return reproduces the old `path_raw.is_empty()` sentinel that
    /// 6 consumers branch on.
    pub fn origin_path_bare(&self) -> String {
        if self.path == DerivationPath::default() {
            String::new()
        } else {
            format!("m/{}", self.path)
        }
    }

    /// BIP-380 bracketed origin annotation `[fp/comps]` (lowercase fp, no `m/`
    /// inside), or `[fp]` for the pathless slot. Reproduces every current
    /// bracketed `path_raw` producer; for descriptor-key construction.
    pub fn bracketed_origin(&self) -> String {
        let fp = self.fingerprint.to_string().to_lowercase();
        if self.path == DerivationPath::default() {
            format!("[{fp}]")
        } else {
            format!("[{fp}/{}]", self.path) // DerivationPath Display = comps joined by '/', no 'm'
        }
    }
}
```

> **Display semantics (R0 M-2 — confirmed against `bitcoin 0.32.8` source, no longer a TODO):** `DerivationPath`'s `Display` renders components `/`-joined with **no** leading `m`/`/` and hardened as `'` (apostrophe, non-alternate). So `bracketed_origin()` yields `[fp/48'/0'/0'/2']` (single leading slash after fp), and it reproduces the producers' `format!("[{fp_hex}{path_raw_inner}]")` byte-for-byte (their `path_raw_inner` carries a leading `/` from the import regex capture group `(?:/\d+'?)+`). `bracketed_origin()` supplies that leading `/` via its own `/` in the format string, so the byte-identity holds. T5(d) pins `DerivationPath::default().to_string() == ""` as a regression guard. The method is the single rendering chokepoint.

The existing `is_secret_bearing()` method in the same `impl` block is untouched.

---

## §4 Producer changes — stop storing `path_raw`

Delete the `path_raw` field from the struct (`synthesize.rs:595`) and remove its initializer at **every** `ResolvedSlot { … }` construction site. Two shapes:

**(a) Direct field-init sites** — drop the `path_raw: …` / `path_raw,` line:
- `synthesize.rs:1141`, `synthesize.rs:1262`, `synthesize.rs:1375` — **ALL `#[cfg(test)]`** (the test mod begins at `synthesize.rs:778`; R0 I-4 corrected the earlier mislabel of `:1141`/`:1262` as production). Test-target field-init drops.
- `cmd/bundle.rs:678` (WIF slot, `String::new()`).
- `cmd/bundle.rs:1382` (**R0 C-1 — was missing**) — descriptor-mode `CosignerKeyInfo` push; `path_raw` is `p.value.clone()` (user-raw via `--slot @N.path=`) or `anno_path.to_string()` (`bundle.rs:1322-1333`). Feeds `emit_unified` → see Amendment A3.
- `cmd/bundle.rs:1442` (`c.path_raw.clone()` re-wrap — line removed entirely).
- `cmd/verify_bundle.rs:830` (**R0 C-1 — was missing**) — `CosignerKeyInfo` push; its `path_raw` is NEVER read for emission/distinctness (verify-bundle uses typed `.path`), so this is a pure mechanical field-init drop.
- `parse_descriptor.rs:1829` test helper `cinfo` (`p.to_string()`).
- `parse_descriptor.rs:1843` test helper `cinfo_raw` (`raw.to_string()`) — this helper's *raison d'être* (decoupling raw from typed) disappears; collapse `cinfo_raw` into `cinfo` or delete it (see §8 T6 / Phase 4).
- `wallet_import/coldcard_multisig.rs:489` (copies local `c.path_raw`).
- Multisig cosigner-push literals (R0 M-A — line numbers tightened to the `path_raw,` field-init line): `specter.rs:255`, `sparrow.rs:437`, `bsms.rs:259`, `electrum.rs:401`, `bitcoin_core.rs:298`, `coldcard.rs:340`, `json_envelope.rs:362`, plus the native `bundle.rs:505/581/630/674` and `parse_descriptor::push_binding` (~`:1245`).

**(b) Tuple-returning single-sig parsers** — collapse the 4-tuple `(xpub, fp, path, path_raw)` to `(xpub, fp, path)` and drop the now-dead `let path_raw = format!("[{fp_hex}{path_raw_inner}]")` line:
- `specter.rs:387/388`, `sparrow.rs:607/608`, `coldcard.rs:534`, `electrum.rs:946`, `bitcoin_core.rs:438/439`, `bsms.rs:389/390`. Update each function's return type + every call-site destructure.

> **Tuple 4th-element disposition (R0 M-3 — CLOSED, no longer Phase-0 deferred):** verified the 4th tuple element (`path_raw`) is consumed *only* to populate `ResolvedSlot` (e.g. `sparrow.rs:431` `out.push((fp, path, path_raw, xpub_str))` → consumed solely into the `ResolvedSlot` push; the descriptor body is built from a separate `substituted` string fed to `concrete_keys_to_placeholders`). The pattern is identical across all six parsers. Safe to collapse the tuple 4→3 without further per-file checks.

The 8 bracketed `format!("[…]")` builders (`json_envelope.rs:350`, `coldcard_multisig.rs:422`, `specter.rs:387`, `sparrow.rs:607`, `coldcard.rs:534`, `electrum.rs:946`, `bitcoin_core.rs:438`, `bsms.rs:389`) are deleted where they fed `ResolvedSlot`; the **descriptor-key** bracket build in `coldcard_multisig.rs:422→658` operates on the **local** `ResolvedCosigner` type and is OUT OF SCOPE (§7).

---

## §5 Consumer contract — every `.path_raw` read site

| # | Site | Current read | Need | New call |
|---|---|---|---|---|
| C1 | `wallet_export/electrum.rs:163-164` | `if !slot.path_raw.is_empty() { slot.path_raw.clone() }` | bare + sentinel | `let d = slot.origin_path_bare(); if !d.is_empty() { … }` |
| C2 | `wallet_export/coldcard.rs:321` | `normalize_path(&s.path_raw)` | bare | `normalize_path(&s.origin_path_bare())` |
| C3 | `wallet_export/sparrow.rs:130` | `normalize_derivation(&s.path_raw, …)` | bare | `normalize_derivation(&s.origin_path_bare(), …)` |
| C4 | `wallet_export/pipeline.rs:33-44` (`key_origin_str`) | branches on `path_raw.is_empty()`: empty → `[fp/<fallback_path>]`, non-empty → `[fp/<path_raw stripped of m/>]` | bracketed, **path-bearing on BOTH arms** | **MUST keep the fallback branch (R0 C-2).** `bracketed_origin()` alone returns `[fp]` for a default-path slot, which would DROP the origin path and corrupt the exported descriptor key. Rewrite: `if slot.origin_path_bare().is_empty() { let raw = fallback_path.trim_start_matches("m/").trim_start_matches('m').trim_start_matches('/'); format!("[{}/{}]", slot.fingerprint.to_string().to_lowercase(), raw) } else { slot.bracketed_origin() }`. Keep the `fallback_path` parameter. Pin with T5(e). |
| C5 | `cmd/bundle.rs:755` (n=1 JSON origin_path) | `origin_path_for_json(&resolved[0].path_raw)` | bare→`Option` | `origin_path_for_json(&resolved[0].origin_path_bare())` |
| C6 | `cmd/bundle.rs:767` (**cosmetic BUG**, multisig CosignerEntry) | `normalize_origin_path(&s.path_raw)` | bare | `s.origin_path_bare()` — **fixes §1.1** |
| C7 | `cmd/bundle.rs:1000-1003` (SlotCardBlock origin_path) | `if s.path_raw.is_empty() {None} else {Some(s.path_raw.clone())}` | bare + sentinel | `match s.origin_path_bare() { e if e.is_empty() => None, p => Some(p) }` |
| C8 | `cmd/bundle.rs:403` (`check_resolved_slots_distinctness`) | `slots[i].path_raw == slots[j].path_raw` | typed | `slots[i].path == slots[j].path` — **Amendment A2** |
| C9 | `cmd/bundle.rs:1630` (ImportWalletSeedMismatch `path` field) | `resolved_slots[i].path_raw.clone()` | display | `resolved_slots[i].origin_path_bare()` — **A1** |
| C10 | `wallet_import/overlay.rs:179` (ImportWalletSeedMismatch `path` field) | `bundle.cosigners[i].path_raw.clone()` | display | `bundle.cosigners[i].origin_path_bare()` — **A1** |
| C11 | `cmd/import_wallet.rs:1409,1431` (`origin_path_from_bracket`) | strip `[…]` → `m/…` | bare | `c.origin_path_bare()` / `s.origin_path_bare()`; **delete `origin_path_from_bracket`** (`:1945-1951`). **R0 I-3 reachability:** `origin_path_from_bracket("[fp]")` (no inner slash) returns `"m"`, whereas `origin_path_bare()` returns `""` for a default-path slot — a `"m"`→`""` divergence. It is **provably dead**: the foreign-format import regex requires ≥1 path component (`(?:/\d+'?)+`), so every cosigner reaching this site has a non-default `path` ⟹ `bracketed_origin()` is always path-bearing ⟹ `origin_path_bare()` is always non-empty. No reachable input hits the divergent arm. |
| C12 | `wallet_import/sparrow.rs:960,1152` (tests) | `p.cosigners[0].path_raw.contains("86'"/"84'")` | assert | `…bracketed_origin().contains("86'"/"84'")` (already `'`-form) |

Helper-function disposition:
- `key_origin_str` (`pipeline.rs:33`) takes a `fallback_path: &str` (R0 C-2 — both callers pass a real path-bearing fallback: `bip388.rs:73` and `pipeline.rs:75`, each `key_origin_str(s, &fallback)`). The `fallback_path` parameter is **retained**; the rewrite is in the C4 row above. NOT a Phase-0 spot-check — it is a settled contract change.
- `normalize_origin_path` (`bundle.rs:731`): becomes dead iff C5+C6 are its only callers — `grep` at Phase 3; delete if dead, else keep.
- `origin_path_from_bracket` (`import_wallet.rs:1945`): deleted (C11 sole consumer).

---

## §6 Amendments (behavior changes — in scope, called out for R0)

**A1 — error-message canonicalization (display-only).** C9 + C10 feed the `path` field of `ToolkitError::ImportWalletSeedMismatch`. **R0 I-2 correction:** today both sites carry the **bracketed `[fp/48'/0'/0'/2']`** form (from `envelope_to_resolved_slots` / the import-wallet decode, no band-aid on these paths), so the `at path {path}` clause currently prints `[fp/…]`; under `origin_path_bare()` it becomes bare `m/…`. The change is bracketed→bare (a larger string change than mere `'`-notation), but still **display-only**: `cli_import_wallet_seed_overlay.rs:136` asserts only the substring `"supplied seed produces xpub"` (before the `path` clause), so no test pins it. Documented so R0 does not flag as regression.

**A2 — distinctness convergence (latent-divergence fix).** C8: `check_resolved_slots_distinctness` (`bundle.rs:399-413`) still compares `path_raw`; its descriptor-mode twin `check_key_vector_distinctness` (`parse_descriptor.rs:1207`) compares typed `.path`. Today these *disagree* for `h`-vs-`'` paths (bundle.rs path would not collide `48h…` vs `48'…`; parse_descriptor would). Deleting `path_raw` forces convergence on the documented v0.5 typed-equality contract. Intentional, in-scope. Requires a new collision test mirroring `bip388_h_vs_apostrophe_paths_collide_under_typed_equality_v0_5` for the ResolvedSlot-vector path (§8 T6).

**A3 — descriptor-mode `--slot @N.path=<raw>` → canonical `bundle --json` origin_path (R0 C-1, NEW).** In `bundle` descriptor mode, an `Xpub` slot with a user-supplied `--slot @N.path=48h/0h/0h/2h` stores the **raw user bytes** in `path_raw` (`bundle.rs:1322-1333`, the `Some(p) => (parsed, p.value.clone())` arm), while the typed `path` is canonical. That `path_raw` flows through `emit_unified` → C5/C6/C7 → the `bundle --json` `origin_path`. After deletion, `origin_path_bare()` renders the **canonical** `m/48'/0'/0'/2'`, dropping the user's `h`-notation (and any other non-canonical spelling). This is an intentional wire-**value** change (key unchanged), same class as A1/A2. **Verified no test pins it:** `cli_compare_cost.rs:770`'s `48h/…/2h` is a `--descriptor` string input (canonicalized through `parse_descriptor`, unaffected); a grep finds no test asserting a `--slot @N.path=`-injected non-canonical `origin_path` in `bundle --json`. Covered by §8 T9.

**A4 — `export-wallet --slot @N.path=<raw>` → canonical export wire-values (R1 I-A, NEW — the same class as A3 on the `export-wallet` direct surface).** `export-wallet` accepts `--slot @N.path=` (`export_wallet.rs:111-116`) and resolves through the SAME `resolve_slots` (`export_wallet.rs:347`), whose `Xpub` arm stores raw `p.value.clone()` at `bundle.rs:547`. So a non-canonical `--slot @N.path=48h/0h/0h/2h` reaches the export consumers C1–C4, which today do NOT fold `h`→`'`: coldcard `normalize_path` (`coldcard.rs:308-317`) only prepends `m/` → emits `m/48h/...`; `key_origin_str` (C4) only strips `m/` → emits `[fp/48h/...]` in the descriptor key + bip388 `keys_info`; sparrow multisig (`sparrow.rs:243-251`) preserves verbatim; electrum (C1) emits verbatim with no `m/`. After `path_raw` deletion, all four render canonical (`m/48'/...`, `[fp/48'/...]`) via `origin_path_bare()`/`bracketed_origin()`. **Intentional wire-value change** on a user-facing wallet-export surface. **Functionally benign** — key material unchanged; canonical form is accepted by miniscript and the target wallets. Currently untested (every `export-wallet` test passes canonical `@N.path=m/...`; suite stays green). Covered by §8 T10; rides the same §10 CHANGELOG note.

---

## §7 Explicitly out of scope

- **`coldcard_multisig.rs`'s local `ResolvedCosigner.path_raw`** (struct field `:534`, built `:422`, consumed `:658` to assemble `[fp/path]xpub/<0;1>/*` descriptor keys). This is a *different type* and a *legitimate bracketed* descriptor-key use. It is copied into `ResolvedSlot.path_raw` at `:489` (that copy is removed per §4), but the local field and its `:658` consumer stay. (Optional future cleanup: rename it `origin_annotation` to reduce confusion — NOT this cycle.)
- No new CLI flags / subcommands / dropdown values.

---

## §8 Test matrix (TDD — tests precede impl each phase)

| ID | Test | Pins |
|---|---|---|
| **T1** | NEW: invoked **specifically via the `bundle --import-json` route** (`bundle_run_from_import_json`, R0 I-1 — `emit_unified` is shared by 3 entry paths; only the import-json route exercises the bracketed-`path_raw` bug) with multisig fixture `envelope_v0_27_0.json` → `multisig.cosigners[].origin_path == "m/48'/0'/0'/2'"` (NOT `m/[…]`) | the §1.1 bug fix; regression guard |
| **T2** | NEW: same envelope → SlotCardBlock `slots[]`/`mk1` block origin_path bare (C7) | C7 branch |
| **T3** | `import-wallet --json \| export-wallet --from-import-json --format {electrum,sparrow,coldcard,coldcard-multisig,jade}` cosigner `derivation`/origin fields are bare `m/…` — **independent of the F5 band-aid** | F5 stays fixed after band-aid deletion |
| **T4** | WIF/pathless slot: `bundle --slot @0.wif=… --json` → `origin_path: null` (default-path → `""` → `None`) | §2.1 pillar 3 sentinel |
| **T5** | unit: `origin_path_bare()` / `bracketed_origin()` on (a) a normal slot → `m/48'/0'/0'/2'` & `[fp/48'/0'/0'/2']`; (b) default-path slot → `""` & `[fp]`; (c) no-double-bracket / single-fp property; (d) `DerivationPath::default().to_string()==""` assertion; **(e) `key_origin_str(default_path_slot, "84'/0'/0'") == "[fp/84'/0'/0']"`** (R0 C-2 — the fallback branch stays path-bearing; assert before+after) | method correctness + pillar-3 invariant + C4 fallback |
| **T6** | unit: two ResolvedSlots, same xpub, paths `48h/0h/0h/2h` vs `48'/0'/0'/2'` → collide under `check_resolved_slots_distinctness` (A2); mirror of the parse_descriptor test | A2 convergence |
| **T7** | extend `cli_wallet_cross_format_convergence.rs`: a `--from-import-json` hop asserts bare origins across all 5 emitters with the band-aid gone (re-decay guard) | durability |
| **T8** | retarget `sparrow.rs:960/1152` to `bracketed_origin()` (C12) | keep existing coverage green |
| **T9** | NEW (R0 C-1/A3): `bundle --descriptor … --slot @N.path=48h/0h/0h/2h --json` → cosigner `origin_path == "m/48'/0'/0'/2'"` (canonical; `h`→`'`). Documents the intentional descriptor-mode wire-value change | A3 |
| **T10** | NEW (R1 I-A/A4): `export-wallet --template … --slot @0.xpub=… --slot @0.path=48h/0h/0h/2h --format coldcard` → `derivation`/descriptor-key canonical (`m/48'/0'/0'/2'` / `[fp/48'/0'/0'/2']`). Documents the export-surface wire-value change | A4 |

Full suite (`cargo test`, sibling-gated tests via `-- --include-ignored` in CI) must stay green; `cargo clippy` clean; `make -C docs/manual audit` (lint + verify-examples + anchor-check) green.

---

## §9 Phased implementation plan (per-phase TDD + per-phase opus review → 0C/0I, persisted to `design/agent-reports/`)

- **Phase 0 — verifications & method scaffolding.** Resolve the three Phase-0 verifications (§3 Display rendering; §4(b) tuple-4th-element-only-feeds-slot; §5 `key_origin_str` fallback semantics + `normalize_origin_path` deadness + A1 no-test-pins). Write T5 (red) → add `origin_path_bare()`/`bracketed_origin()` (T5 green). No call-site changes yet.
- **Phase 1 — bundle JSON emit + sentinel (the bug fix + A3).** T1, T2, T4, **T9** (red) → switch C5/C6/C7 to `origin_path_bare()` (green). This is where §1.1 dies; T9 documents the A3 descriptor-mode wire-value change through the same C5/C6/C7 consumers.
- **Phase 2 — export emitters + F5 band-aid removal.** T3, T7, **T5(e)**, **T10** (red) → switch C1/C2/C3 to the new methods and rewrite C4 (`key_origin_str`) keeping the path-bearing fallback branch (R0 C-2), delete the `export_wallet.rs:646-648` band-aid (green). Highest-risk phase (export contract now exercises `bracketed_origin()`/`origin_path_bare()` directly). T10 pins the A4 non-canonical `@N.path=` export. Smoke-check that miniscript still parses the emitted descriptor keys (canonical + the T10 non-canonical-input case).
- **Phase 3 — import_wallet + distinctness + producers + field deletion.** C11 (delete `origin_path_from_bracket`), C8 (A2) with T6 (red→green), C9/C10 (A1). Delete all §4 producer `path_raw` field-inits — including the R0 C-1 additions `bundle.rs:1382` and `verify_bundle.rs:830` — collapse §4(b) tuples + delete the struct field. Full build must compile (the field deletion is the forcing function — every missed site is a compile error, which is the safety net that would surface any further unlisted producer).
- **Phase 4 — test helpers + cleanup.** Collapse/delete `cinfo_raw` (`parse_descriptor.rs:1843`), retarget T8 (C12), drop dead helpers (`normalize_origin_path` if dead). Update the struct doc-comment (`synthesize.rs:588-590`) + `parse_descriptor.rs:1201-1203` stale "round-trip fidelity" comments. `overlay.rs:25` module-doc line referencing `.path_raw`.
- **Phase 5 — full verification + end-of-cycle opus R0.** `cargo test` (+`--include-ignored`), `clippy`, `make audit`; transcript/verify-examples re-capture if any sample shows the old polluted `origin_path`. End-of-cycle architect R0 → 0C/0I.
- **Phase 6 — release prep + ship.** §10.

---

## §10 Release, lockstep, manual

- **SemVer:** PATCH → `mnemonic-toolkit-v0.37.9`. User-visible changes: §1.1 `origin_path` cosmetic correction + A1 error text + A2 distinctness alignment + A3 descriptor-mode `bundle --json` origin_path canonicalization + A4 `export-wallet` origin-annotation canonicalization (both A3/A4 only on non-canonical `--slot @N.path=` input) (R2 M-1). `--json` wire-shape: `origin_path` *value* changes (key unchanged).
- **GUI `schema_mirror`:** NOT triggered — no clap flag-name/value/subcommand change (gate is flag-NAME parity only per CLAUDE.md). **Decision (R0 M-4):** the `bundle --json` `origin_path` *value* changes (the §1.1 fix, the A3 descriptor-mode canonicalization, AND the A4 `export-wallet` canonicalization) are not schema_mirror-gated. Disposition: **CHANGELOG note + GUI self-updates via the paired-PR rule; NO hard lockstep PR this cycle.** (Not posed as an open question.)
- **Manual mirror:** NOT triggered by code (no surface change). BUT grep `docs/manual/` + `transcripts/` + verify-examples corpus for any `bundle --import-json --json` sample showing `m/[…]`, and any `export-wallet`/`bundle` sample using a non-canonical `--slot @N.path=` (R2 M-2); re-capture if present (the manual-prose gate will otherwise flag drift).
- **Phase 6 checklist** (per project convention): `Cargo.toml` version + `Cargo.lock` (staged together) + both README `<!-- toolkit-version: X -->` markers + `scripts/install.sh` self-pin `TAG=` + CHANGELOG entry + FOLLOWUP `path-raw-bracketed-vs-bare-convention-unification` Status `open → resolved <sha>`. NO GUI-schema change (no flag delta). Clean working tree before checkout→ff→tag→push.

---

## §11 Source citations (verified at SHA `dd7c228`)

Struct/impl: `synthesize.rs:591` (struct), `:595` (field), `:622` (impl), `:190` (alias). Binary-private: `main.rs:27`. Producers: §4 list. Consumers: §5 table. Stale-fidelity: `synthesize.rs:588-590`, `parse_descriptor.rs:1201-1203`, test `:1931`, helper `cinfo_raw` `:1843`. Distinctness twin: `parse_descriptor.rs:1207`. Band-aid: `export_wallet.rs:646-648`. Cosmetic emit: `bundle.rs:731,767,1000-1003`. `bitcoin 0.32.8` pinned in `Cargo.lock`.

---

## §12 R0 agenda (what the architect must stress)

1. The four §2.1 safety pillars — re-challenge each against source.
2. §4(b) tuple-collapse completeness — is the 4th element ever consumed beyond `ResolvedSlot`?
3. §3 method semantics — `DerivationPath` Display form; the `bracketed_origin` no-`m`-inside contract; default-path edge (`[fp]` vs `[fp/]`).
4. §5 completeness — is every `.path_raw` read site in the table? (129 grep hits; confirm none missed.)
5. A1/A2 framing — are these correctly scoped as intentional in-cycle behavior changes with tests?
6. §8 test matrix — does T1–T8 cover the regression surface (esp. the previously-unasserted §1.1 bug)?
7. Phase ordering — is "delete the field last (Phase 3)" the right forcing function, or should it lead?
