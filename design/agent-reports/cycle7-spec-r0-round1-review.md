# cycle-7 spec — R0 review, round 1

**Spec:** `design/BRAINSTORM_cycle7_m8_build_descriptor.md` (M8 build-descriptor extra-derivation-suffix → silent wrong subtree; + L23 ecies zero-scalar panic)
**Reviewed against SHA:** `8d2fe505` (toolkit 0.64.0; miniscript pinned 13.0.0 @ `95fdd1c`) — verified via `git show origin/master:<path>`
**Reviewer:** opus software architect (adversarial R0; HARD gate — no code until 0C/0I)
**Date:** 2026-06-21

---

## Verification log (independent, against `8d2fe505`)

Every load-bearing citation re-grepped against `origin/master` bytes. **All resolve exactly** (the spec's `79e3387c` re-grep matches `8d2fe505` — the cycle-7 file-zone diff is empty, confirmed):

| Claim | Spec cite | Verified line(s) | Result |
|---|---|---|---|
| `check_secret_key` (origin-strip + xprv-only screen) | `gate.rs:347-361` | `fn` `:347`; `key_part = key.rsplit(']')…` `:348`; `is_xprv` prefix `:349-350`; emits `SecretKey` (inline Diagnostic, not `field_diag`) | ✅ exact |
| `validate_fields` calls `check_secret_key` Pk/Pkh + each Multi/Sortedmulti key | `gate.rs:235`,`:240` | Pk/Pkh `:235`; Multi/Sortedmulti loop `:240` | ✅ exact |
| `validate_with_allow` step-1 runs `validate_fields(&doc.root,…)` | `gate.rs:163-164` | `:157` fn, `:164` call | ✅ exact |
| `DiagnosticKind` enum (gate-step-ordered, `SecretKey` last/step-1) | `gate.rs:90-119` | enum `:90`; `SchemaField` `:99`; `SecretKey` `:117` | ✅ exact |
| `as_str` (stable `--json` discriminant) | `gate.rs:122-135` | `:121` fn; `"schema_field"` `:124`; `"secret_key"` `:133` | ✅ exact |
| `field_diag` → `SchemaField`, `flag: None` | `gate.rs:679-686` | `:679` fn; `kind: DiagnosticKind::SchemaField` `:682` | ✅ exact |
| `child_paths` recurses AndV/OrD/OrI/OrB/Andor/Thresh/Wrap | `gate.rs` (recursion) | `:646` fn; recursion loop `:228-230`; covers all subtree variants | ✅ exact |
| `MULTIPATH_SUFFIX = "/<0;1>/*"` + account-level doc | `ir.rs:21-23` | `:23` const; doc `:21-22` | ✅ exact |
| `with_multipath` = blind concat; `render_keys` maps every key | `ir.rs:218-228` | `:218-219` `format!("{key}{MULTIPATH_SUFFIX}")`; `:222-224` map; Pk/Pkh at `:237-238` | ✅ exact |
| Preset intake → `SpecDoc{root:(def.lower)(&params)}` → `validate_with_allow` | `build_descriptor.rs:282-287` | `:282-287` exact | ✅ exact |
| Spec intake → `SpecDoc::parse` → `validate_with_allow` | `build_descriptor.rs:322-326` | `:323` parse, `:326` gate | ✅ exact |
| gate fail → exit 2 | `build_descriptor.rs` | `:279,:300,:329` emit + `return Ok(2)` at `:330` | ✅ exact |
| L23 `Scalar::from_be_bytes(*privkey)` accepts zero | `electrum_crypto.rs:345` | `:345` exact | ✅ exact |
| L23 `.mul_tweak(…).expect("…never the identity")` panics on zero | `electrum_crypto.rs:349-351` | `.mul_tweak` `:350`, `.expect` `:351` | ✅ exact |
| `EciesDecryptError::InvalidScalar` already exists | `electrum_crypto.rs:247` | `:247` variant; `:278` Display arm | ✅ exact |
| Safe sole caller `derive_storage_eckey` rejects zero | `electrum_crypto.rs:309-310` | `:309-310` `if scalar.iter().all(|&b| b==0) { return Err(InvalidScalar) }` | ✅ exact |
| `ecies_decrypt_message` is `pub fn` | `electrum_crypto.rs:321` | `:321` exact | ✅ exact |

**Key-bearing-field enumeration (the funds crux, independently derived).** `PolicyNode` (`ir.rs:110-145`) has exactly FOUR key-bearing variants: `Pk(String)`, `Pkh(String)`, `Multi(MultiSpec)`, `Sortedmulti(MultiSpec)` (`MultiSpec.keys: Vec<String>`). All other variants carry hashes/timelocks/sub-trees, no keys. `validate_fields` handles `Pk|Pkh` → `check_secret_key` (`:235`); `Multi|Sortedmulti` → per-key `check_secret_key` (`:240`); and recurses into EVERY subtree-bearing variant via `child_paths` (`:646`, covers AndV/OrD/OrI/OrB/Andor/Thresh/Wrap) through the `:228-230` loop. **A key nested anywhere — under `and_v`, `thresh`, `wrap`, `andor` — therefore reaches `check_secret_key`.** No positional/alternate key field exists. Both intake paths (`build_descriptor.rs:287` preset, `:326` spec) construct the `validated` artifact ONLY via `validate_with_allow`; `emit` consumes `&validated`; **there is no render-to-emit path that bypasses the gate** (verified: `:287/:326` are the sole `validate_with_allow` call-sites and the sole producers of `validated`). Presets clone keys verbatim (`archetype.rs` `params.keys.clone()` / `one_key(...).to_string()`, no stripping). **Conclusion: the single guard in `check_secret_key` covers EVERY key-intake path. No bypass.**

**Protocol-fact shape (load-bearing).** The recon verified against the pinned parser (`…/95fdd1c/src/descriptor/key.rs::parse_xkey_deriv`) that a FIXED index before a `<a;b>` BIP-389 token parses as a valid `MultiXPub`, and a wildcard before a further segment is rejected (`InvalidWildcardInDerivationPath`). The spec's bug-mechanism statement is **correctly shaped**: the user's `[fp/path]xpub.../5` is concatenated by the renderer to `…xpub.../5/<0;1>/*`, which parses and type-checks but derives from the user-smuggled `/5` subtree (NOT the account level) → a different, wrong address set; the trailing-`*` variant (`…/*/<0;1>/*`) is the one the parser already rejects. The fix targets the right thing (refuse ANY in-key trailing segment so the renderer's owned suffix is never preceded by a smuggled path). Per the project external-protocol-fact rule, the recon's primary-source verification is accepted; I sanity-checked the claim's shape and it is internally consistent. No re-verification gap.

**Over-rejection control (independently checked).** After `rsplit(']').next()`, a legitimate account-level key's `key_part` is the bare `xpub…`/`tpub…`/SLIP-132 body — all `[fp/path]` `/`-tokens live INSIDE the brackets and are stripped. Test fixtures K1–K5 + XPUB (`tests/cli_build_descriptor.rs:29-33,307`) are all `[fp/path]xpub…` with zero post-`]` `/`. The raw-hex (`:311`) and WIF (`:306`) negative-control keys contain no `/` either, so the new `contains('/')` predicate does NOT perturb their existing refusal mechanisms (xprv-prefix / step-2 `from_str`). **`contains('/')` is true ONLY for the M8 trailing-derivation class; no legitimate descriptor account-level key carries a post-bracket `/`** (the descriptor key grammar is `[origin]xpub` — there is no legitimate account-level form with a trailing `/`; sub-account targeting is expressed inside the origin path, before the `]`). No over-rejection.

---

## Critical

**None.**

The funds-safety core is sound: the guard sits on the unique pre-emission gate, covers all four key-bearing fields plus every nested occurrence, fails closed (exit 2, no descriptor), never leaks the key, and rejects exactly the M8 class without touching any legitimate input. The chosen predicate (`key_part.contains('/')` after origin-strip) is decision-complete and correctly catches `/5`, `/5/6`, `/0h`, `/<0;1>`, `/*`.

## Important

**None.**

Diagnostic-kind decision is correct and conservative: reuse `DiagnosticKind::SchemaField` via `field_diag` → **no new `as_str` discriminant → zero `--json` wire-shape delta → no GUI self-update / paired-PR burden, and correctly NOT a `schema_mirror` trigger** (that gate is clap flag-names + dropdown enums only; the intake flag surface `--key`/`--spec`/`--archetype` is unchanged). L23 reuses the existing `InvalidScalar` variant (no new variant, no wire change), is firewalled (separate file, separate test, no shared code), and is confirmed latent (sole CLI path `import_wallet.rs → ecies_decrypt_storage → derive_storage_eckey` rejects zero before `ecies_decrypt_message`). SemVer MINOR 0.65.0 (fail-closed reject of previously-accepted input = FORMAL, MINOR pre-1.0) is correct; no registry publish; oracle N/A (bitcoind_differential is a `bundle→restore` round-trip keyed on a given descriptor string — it never invokes `build-descriptor`); no manual-mirror gate. TDD matrix is genuinely RED-first: the clean baseline is confirmed (no existing fixture passes a trailing-suffix key), so T1/T2/T5 fail today (accepted-with-wrong-subtree) and pass after the guard; T3 positive control + T4 asymmetry pin + T6 no-leak + T7/T8 L23 are all well-formed.

## Minor

1. **`gate.rs` (preset flag-provenance annotation of the new `SchemaField` reject) — add a pin to T1.** The preset path annotates gate diagnostics via `archetype::resolve_flag(def, &d.node_path, d.kind)` (`build_descriptor.rs:298`; `resolve_flag` at `archetype.rs:395`, `kind`-aware longest-prefix). A `SchemaField` emitted at a key node (e.g. `root.kofn.keys[i]` or `root`) will resolve to `--key` only via the catch-all `(prefix, None, "--key")` provenance entries — and a kind-specific `Some(SchemaField, "--threshold")` override exists for quorum nodes (test at `archetype.rs:743+`). The funds-safety refusal (exit 2) is unaffected regardless of how the flag annotates (a `None` resolution merely omits the flag), so this is purely annotation-quality — but T1 currently asserts only `kind=schema_field` for the preset path. **Recommend T1 additionally assert the preset refusal names the offending `--key` flag** (i.e. that a key-node `SchemaField` resolves to `--key`, not `None`/`--threshold`), to pin that the M8 reject inherits the existing preset-provenance UX. Not a blocker — fold into the test matrix at implementation.

2. **`§3.1` / D4 — note that `check_secret_key` currently builds its `SecretKey` Diagnostic INLINE, not via `field_diag`.** The spec's reuse plan ("emit via `field_diag(...)`") is correct, but the implementer should note that the existing xprv arm constructs the `Diagnostic{…}` struct directly (`gate.rs:351-358`) because it needs `kind: SecretKey`; the new suffix arm will instead call `field_diag(path, msg)` (which hardcodes `SchemaField` + `flag: None`). The `path` passed must be the same node-addressed `path` arg already threaded into `check_secret_key` (for Multi/Sortedmulti that is the `{path}.{kind}.keys[i]` string built at the `:240` call-site — NOT a bare `path`). Cosmetic clarification; the spec's D7 ordering (xprv `if` … else suffix) already implies the right structure.

---

## Verdict

The spec is correct, complete, and decision-complete on every funds-load-bearing axis: guard placement covers all key-intake paths with no bypass, predicate rejects exactly the M8 class with no over-rejection, diagnostic-kind reuse avoids any wire/gate churn, L23 is latent and cleanly typed, and the TDD matrix is genuinely RED-first. The two Minors are test-assertion and implementer-note refinements, not gate blockers.

**R0 ROUND 1: 0C / 0I** — **GREEN (0C/0I)**
