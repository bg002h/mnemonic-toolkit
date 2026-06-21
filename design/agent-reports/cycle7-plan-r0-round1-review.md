# cycle-7 PLAN — R0 review, round 1

**Plan-doc:** `design/IMPLEMENTATION_PLAN_cycle7_m8_build_descriptor.md` (M8 build-descriptor extra-derivation-suffix → silent wrong subtree; + L23 ecies zero-scalar panic)
**Implements (R0-GREEN spec):** `design/BRAINSTORM_cycle7_m8_build_descriptor.md` — verified the plan EXECUTES the spec (spec decisions D1–D16 not re-litigated).
**Spec R0 review the plan folds:** `design/agent-reports/cycle7-spec-r0-round1-review.md` (Minor-1, Minor-2).
**Reviewed against SHA:** `d6398b57` (toolkit 0.64.0; current `origin/master`; cycle-7 file-zone `descriptor_builder/`+`electrum_crypto.rs` diff `8d2fe505..d6398b57` is EMPTY — verified). Miniscript pinned 13.0.0 @ `95fdd1c`.
**Reviewer:** opus software architect (adversarial R0; HARD gate — no code until 0C/0I).
**Date:** 2026-06-21

---

## Verification log (independent, against `d6398b57` + empirical binary probe)

**SHA reconciliation.** Plan basis SHA `8d2fe505`; current `origin/master` is `d6398b57` (plan text says `79e3387c` — that was the spec-author-time master; `d6398b57` is two further design-doc-only commits ahead, incl. cycles 6/8 reports). `git diff --stat 8d2fe505 d6398b57 -- crates/mnemonic-toolkit/src/descriptor_builder/ crates/mnemonic-toolkit/src/electrum_crypto.rs` = **EMPTY** — the cycle-7 code zone is byte-stable; all source citations re-verified against `d6398b57` resolve exactly. The bughunt-report file DID change (`+16/-6`), moving the M8/L23 checkbox lines (see Minor-2 below).

| Plan cite | Claim | Verified line(s) @ `d6398b57` | Result |
|---|---|---|---|
| L1 | `fn check_secret_key(key, path, kind, out)` | `gate.rs:347` | ✅ exact |
| L2 | `let key_part = key.rsplit(']').next().unwrap_or(key);` | `gate.rs:348` | ✅ exact |
| L3 | `is_xprv` screen → inline `Diagnostic{kind: SecretKey,…}` | `gate.rs:349` (is_xprv), `:353` (`kind: SecretKey`), arm `:351-359` | ✅ exact |
| L4 | `validate_fields` Pk/Pkh → `check_secret_key(k, path,…)` | `gate.rs:235` | ✅ exact |
| L5 | Multi/Sortedmulti per-key → `check_secret_key(key, &format!("{path}.{}.keys[{i}]", node.kind()),…)` | `gate.rs:240-244` (path string `:242`) | ✅ exact |
| L6 | `validate_with_allow` step-1 `validate_fields(&doc.root,"root",…)` | `gate.rs:164` | ✅ exact |
| L7 | `child_paths` recurses AndV/OrD/OrI/OrB/Andor/Thresh/Wrap | `gate.rs:646`; **recursion driver `:333-334`** `for (cpath,child) in child_paths(node,path){ validate_fields(child,&cpath,out); }` | ✅ exact |
| L8 | `as_str`: `SchemaField=>"schema_field"` (`:124`), `SecretKey=>"secret_key"` (`:132`) | `gate.rs:124,:132` | ✅ exact |
| L9 | `field_diag(path, message)` → `Diagnostic{kind: SchemaField, flag: None}` | `gate.rs:679-686` (`SchemaField` `:682`) — **takes a `path: &str` arg** | ✅ exact |
| L10 | Preset intake → `SpecDoc{root:(def.lower)(&params)}` → `validate_with_allow`; `resolve_flag(def,&d.node_path,d.kind)` | `build_descriptor.rs:282-298` (resolve_flag `:298`) | ✅ exact |
| L11 | Spec intake → `SpecDoc::parse` → `validate_with_allow` | `build_descriptor.rs:323,:326` | ✅ exact |
| L12 | gate fail → `emit_diagnostics` → `return Ok(2)` | `build_descriptor.rs:279-280,300-301,329-330` | ✅ exact |
| L13 | `resolve_flag` kind-aware longest-prefix; `KEY="--key"` (`:91`), `THRESHOLD="--threshold"` (`:92`) | `archetype.rs:395`; consts `:91-92`; longest-prefix+`k.is_some()` tiebreak `:404-411` | ✅ exact — **but see Important-1** |
| L14 | `Scalar::from_be_bytes(*privkey).map_err(|_| InvalidScalar)?` accepts zero | `electrum_crypto.rs:345` | ✅ exact |
| L15 | `.mul_tweak(&secp,&scalar).expect("…never the identity")` panics on zero | `electrum_crypto.rs:350-351` | ✅ exact |
| L16 | `EciesDecryptError::InvalidScalar` already exists (Display `:278`) | `electrum_crypto.rs:247,:278` | ✅ exact |
| L17 | `derive_storage_eckey` zero-guard `if scalar.iter().all(|&b| b==0){…}` | `electrum_crypto.rs:309-310` | ✅ exact |
| L18 | `ecies_decrypt_message` `pub fn` — sig `(blob_b64:&str, privkey:&[u8;32])` | `electrum_crypto.rs:321-323` | ✅ exact (`privkey:&[u8;32]` ⇒ `privkey.iter().all(...)` valid) |
| L19 | `BIE1_KAT1` valid-blob const | `electrum_crypto.rs:668` | ✅ exact |
| L20 | existing `ecies_decrypt_message_electrum_kat_short_vectors` | `electrum_crypto.rs:700` (call `:703`) | ✅ exact |
| L21–L25 | Cargo.toml:3, READMEs, install.sh:32, fuzz/Cargo.lock:575, CHANGELOG:9 — all `0.64.0` | verified | ✅ exact |
| L28 | `readme_version_current.rs` auto-gates both README markers vs `CARGO_PKG_VERSION` | `:23-28` (`both_readmes_carry_current_version_marker`) | ✅ exact |
| L26/L27 | bughunt M8/L23 checkboxes | **`:724` / `:833`** (plan cites `:721`/`:830` — **stale, +3 each**) | ⚠️ Minor-2 |

**No-bypass (the funds crux) — independently re-derived + empirically confirmed.** `validate_fields` (`:232`) handles the four key-bearing variants (`Pk|Pkh`→`:235`; `Multi|Sortedmulti`→per-key `:240`) AND, at its tail (`:333-334`), recurses into EVERY subtree via `child_paths` (AndV/OrD/OrI/OrB/Andor/Thresh.subs[i]/Wrap.sub). So a key nested under and_v/thresh/andor/wrap reaches `check_secret_key`. Both intake paths construct `validated` ONLY via `validate_with_allow` (preset `:287`, spec `:326`); the sole production `emit(&validated,…)` calls (`:317`,`:335`) consume that artifact. Every other `render_descriptor`/`with_multipath` reference is inside `#[cfg(test)]` or is invoked from within `validate_with_allow` itself (step-2, `gate.rs:170` — AFTER step-1 would have rejected). **No render-to-emit path skips the step-1 field gate. The single guard in `check_secret_key` covers every key-intake path.** Empirically: `--archetype hashlock-gated --key '[…]xpub…/5'` → exit 0, `keys_info` shows the smuggled `…/5` (wrong subtree) TODAY (RED); the same via `--spec {pk:"…/5"}` → emits `pk(…/5/<0;1>/*)` (RED). Nested `/5/6` and `/0h` both accepted today (RED).

**Over-rejection control — empirically confirmed.** `--spec` bare `xpub…` (no origin) builds; `[fp/path]xpub` builds; distinct-key positive control builds (exit 0). After `rsplit(']')`, a legitimate account-level body has zero `/` (the only legitimate `/`-bearing token, `[origin]`, is stripped). `key_part.contains('/')` is true ONLY for the trailing-derivation class. **No legitimate input over-rejected.**

**L23 — confirmed latent + cleanly typed.** Sig is `privkey:&[u8;32]` so `privkey.iter().all(|&b| b==0)` is valid and mirrors `derive_storage_eckey`'s guard (`:309`); placed before `:345` it makes `.expect` (`:351`) provably hold for the zero case. Reuses the existing `InvalidScalar` (`:247`) — no new variant, no wire change. Sole CLI path (`import_wallet → ecies_decrypt_storage → derive_storage_eckey`) rejects zero before `ecies_decrypt_message`, so the panic is unreachable from the CLI today — robustness only.

**TDD RED-first — confirmed non-vacuous (empirical).** T1/T2/T5/T5b: accepted-with-wrong-subtree today → exit 2 after. T4 (`/*`): currently `type_error` at step-2 (`'*' may only appear as last element`) — plan's "tolerant of which step" framing is correct. T3 positive controls build today and must keep building. Tests live in the BIN target (`tests/cli_build_descriptor.rs` + `gate.rs`/`electrum_crypto.rs` `#[cfg(test)]`) → run under `cargo test -p mnemonic-toolkit`, NOT `--lib` (the plan's per-phase gate `:14-16` correctly uses the full-package suite).

---

## Critical

**None.**

The funds-safety core is sound and executes the GREEN spec faithfully: the guard sits on the unique pre-emission gate, reaches all four key-bearing fields plus every nested occurrence (recursion driver `:333-334` independently confirmed), fails closed (exit 2, no descriptor emitted) on BOTH intakes, never leaks the key, and rejects exactly the M8 class with no over-rejection. L23 is latent and cleanly typed. No funds-loss path survives the plan as written.

## Important

**I-1. Minor-1 (T1b) is folded INCORRECTLY — for the three quorum archetypes the M8 `SchemaField` reject resolves to `--threshold`, NOT `--key`; T1b as scoped (single-key preset) passes while masking the defect, and the plan's stated behavior (P1 line 102 / spec D4) is FALSE for multi keys.** `gate.rs` + `archetype.rs:395-411` / `build_descriptor.rs:298`; **required fix below.**

The plan (P1 impl note, line 102): *"the preset path's `resolve_flag` annotates `--key` downstream (L10/L13) — pinned by T1b"* and T1b (line 82): *"assert the emitted `SchemaField` at the key node annotates `flag=--key` (NOT `None`, NOT `--threshold`)."* The spec's D4 makes the same general claim. **This is true ONLY for the two single-key archetypes and FALSE for the three quorum archetypes.** Empirically (probe against `resolve_flag` at `d6398b57`):

```
hashlock-gated   Pk    root.andor[0]              SchemaField -> Some("--key")        ✓
simple-timelocked Pk   root.or_d[0]               SchemaField -> Some("--key")        ✓
kofn-recovery    Multi root.or_d[0].multi.keys[0] SchemaField -> Some("--threshold")  ✗
tiered-recovery  Multi root.or_i[0].multi.keys[0] SchemaField -> Some("--threshold")  ✗
decaying-multisig Multi root.andor[0].multi.keys[0] SchemaField -> Some("--threshold") ✗
   (compare) kofn  Multi root.or_d[0].multi.keys[0] SecretKey   -> Some("--key")       ✓  ← the EXISTING xprv reject
```

Mechanism: each quorum archetype carries a provenance entry `("root.<quorum>[0]", Some(SchemaField), THRESHOLD)` alongside `("root.<quorum>[0]", None, KEY)` (e.g. `archetype.rs:182-183` kofn). A `Multi.keys[i]` reject's `node_path` is `root.<quorum>[0].multi.keys[i]`, which `strip_prefix("root.<quorum>[0]")` matches (`.multi…` boundary). Both entries share the same prefix length, so `max_by_key((prefix.len(), k.is_some()))` (`:411`) breaks the tie in favour of the `Some(SchemaField)` entry → `--threshold`. The **existing** xprv reject escapes this because it carries `kind: SecretKey`, which the threshold override's kind-filter (`k.map_or(true, |k| k==kind)`, `:409`) rejects — so the catch-all `--key` wins. **The D4 decision to reuse `SchemaField` for the M8 reject is exactly what regresses the flag annotation on quorum multis.**

Funds impact: NONE — the refusal (exit 2, no descriptor) fires regardless of how the flag annotates. This is annotation-quality. But it is (a) a real wrong-pointer UX defect (the user is told the problem is at `--threshold` when their offending key is at `--key`), and (b) the plan ASSERTS the opposite as a pinned property while choosing a test (single-key archetype) that cannot expose it. A GREEN T1b would give false confidence that all preset M8 rejects name `--key`.

**Required fix (pick one, fold into the plan before GREEN):**
- **(a) Correct the test matrix + the claim (minimal, recommended).** Make T1b cover BOTH a single-key archetype (assert `--key`) AND a quorum archetype's `Multi.keys[i]` reject — and assert the ACTUAL resolution (`--threshold`) there, documenting it as accepted annotation behavior. Strike the unqualified "annotates `--key`" claim at P1 line 102 / restate D4 as "single-key presets annotate `--key`; quorum-key presets annotate `--threshold` (the quorum-node override wins) — refusal is unaffected either way." This keeps the funds property and stops the plan asserting a falsehood.
- **(b) Make the M8 reject resolve to `--key` on multis too** (more work, better UX): e.g. emit the M8 reject under a kind whose provenance filter excludes the threshold override (mirroring how `SecretKey` already resolves correctly to `--key` on multis), OR add an `&` clause so the threshold override does not match `*.keys[i]` sub-paths. This re-opens the D4 reuse decision (a new `DiagnosticKind` is a `--json` wire delta) and must be weighed against D4/D5 — likely heavier than warranted for an annotation nicety; (a) is preferred.

Either way the plan must STOP claiming the M8 preset reject universally names `--key`, and T1b must test the case that can actually fail. (Not Critical — no funds path; but Important — the plan pins a false property with a test blind to its own counter-example.)

## Minor

**M-1. Bughunt-report checkbox line numbers are stale (+3).** The plan cites M8 `:721` / L23 `:830` (`8d2fe505` snapshot); at `d6398b57` they are **`### - [ ] M8 ·` `:724`** and **`### - [ ] L23 ·` `:833`** (the report file took `+16/-6` of cycle-6/8 design churn). The plan already instructs "re-grep the current line numbers at impl time" (lines 144, 154), so this is covered by the plan's own discipline — but update the snapshot cites to `:724`/`:833` for accuracy.

**M-2. `--spec` intake has no `resolve_flag` annotation — T2's `node_path` assertion is the right surface (no action; confirming).** T2 asserts the `--spec` Multi reject's `node_path` is `root.multi.keys[0]`. The `--spec` path does NOT run `resolve_flag` (only the preset path does, `:298`), so T2 correctly asserts the raw `Diagnostic.node_path` (built at `:242`) rather than a flag. This is consistent with Minor-2's fold (`field_diag(path,…)` carries the threaded node-addressed `path`). `field_diag` confirmed to take a `path: &str` arg (`:679`) — so the plan's "pass `path` through, no path-aware ctor needed" is correct as written. No change.

**M-3. SemVer 0.65.0 is the next free MINOR — confirm at impl time.** Master is `0.64.0`; CHANGELOG shows `0.64.0`/`0.63.0`/`0.62.1` heads. `0.65.0` is free, and the plan's "renumber if another in-flight cycle lands first" (own-account → 0.65.x per MEMORY) is the right hedge. No action unless a sibling cycle merges first.

---

## Scope confirmations (all hold)

- **Funds property:** M8 fails closed (exit 2, no descriptor) on every key-bearing field + every nested subtree + both intakes — no bypass (structurally + empirically verified). ✅
- **No over-rejection:** bare xpub / `[origin]xpub` / SLIP-132 all build; `contains('/')` is exactly the M8 class. ✅
- **Minor-2 (path-fidelity):** correctly folded — `field_diag` takes the `path` arg; pass-through is correct. ✅
- **Minor-1 (flag-provenance):** folded but **INCORRECTLY** — see I-1. ❌
- **No `--json`/schema_mirror/manual/codec trigger:** reuse of `SchemaField` adds no `as_str` discriminant (enum at `gate.rs:90-135` unchanged); no clap flag/subcommand/dropdown change. ✅
- **L23:** typed `InvalidScalar` before `mul_tweak().expect()`; latent; firewalled; no new variant, no valid-path behavior change. ✅
- **Version sites:** Cargo.toml + both READMEs (auto-gated) + install.sh + fuzz/Cargo.lock + CHANGELOG all enumerated and verified at `0.64.0` → bump to `0.65.0`. ✅
- **TDD:** genuinely RED-first / non-vacuous (empirically reproduced); BIN-target suite. ✅

---

## Verdict

The funds-safety engineering is correct, complete, and bypass-free — the guard placement, predicate, fail-closed behavior, no-leak, both-intake and nested coverage, and L23 typing all execute the GREEN spec faithfully and survive adversarial + empirical probing. The one blocker is **I-1**: the plan folds spec Minor-1 by asserting (and pinning with T1b) that the preset M8 reject universally annotates `--key`, but for the three quorum archetypes it resolves to `--threshold` (the `Some(SchemaField)` quorum-override wins the `resolve_flag` tiebreak) — and T1b's single-key scoping cannot expose this. The plan must stop asserting the false property and test the case that can fail (fix (a)). Non-funds, but it is an Important plan-correctness/test-blindness defect under the R0 "tests must be non-vacuous and claims must be true" bar.

**PLAN R0 ROUND 1: 0C / 1I — RED**
