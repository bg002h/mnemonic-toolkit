# SPEC — Wave-4 L1: verify-bundle ↔ bundle descriptor-mode binding dedup

**Slug:** `verify-bundle-bundle-rs-descriptor-mode-dedup`
**Cycle:** Wave-4 (open-followups maturity program), lane A "dedup-robustness"
**Source SHA pinned for citations:** `940abe9e7cbf55ab005f3aae6541ec42ab7dbd69` (toolkit master, v0.71.0). *(Re-pinned from the original `e6c36f0c` draft; HEAD advanced via the L3 docs commit `940abe9e` — citations below re-grepped against `940abe9e`, gate now at `bundle.rs:1387-1402` / `verify_bundle.rs:1396-1412`, the fp-mismatch refusal at `bundle.rs:1634`.)*
**SemVer:** **NO-BUMP.** Pure internal hygiene refactor. **One narrow observable delta** (the error-precedence reshuffle on *doubly-malformed* emit input — §3.2, §5 item 2): folding the gate into the shared fn re-orders gate-vs-probe AND gate-vs-account-refusal AND gate-vs-row-4-refusal. All three re-orders only change WHICH exit-≠0 refusal message surfaces for malformed input that would be refused either way (over-`n` slot vec AND a parse-failing descriptor / `--account != 0` on canonical / a row-4 `[Phrase,Path]` conflict) — no funds/accept-set/exit-class impact (every path stays exit-≠0). All accepting paths are byte-preserved. No clap surface, no wire-shape, no error-kind enum change, no exit-code-class change. No version-site ritual (READMEs / Cargo.lock / install.sh untouched), no tag.
**CI-coupling:** **NONE.** Internal helpers only (`cmd/bundle.rs`, `cmd/verify_bundle.rs`). No clap flag/subcommand/dropdown change → no GUI `schema_mirror` gate, no manual flag-coverage gate, no sibling-codec lockstep. No secret-bearing `String`/`Xpriv` moves → `lint_zeroize_discipline` / `lint_argv_secret_flags` untouched, lint-floor unchanged.
**R0 status:** This is a **funds-path** refactor (round-trip path-derivation fidelity). FULL R0 GATE MANDATORY per CLAUDE.md — spec R0 to 0C/0I before plan, plan R0 to 0C/0I before any code. Do NOT treat as a mechanical Wave-4 cleanup.

---

## 0. Re-grep mandate (citations decay every merge)

Every line number below is a snapshot at `940abe9e`. **Before writing ANY code, the implementer MUST re-grep each cited span against the worktree's current `HEAD`** and use live line numbers. The recon already caught one decayed citation: the in-code S-VERIFY fold-comment at `verify_bundle.rs:1392-1395` says "this duplicates `bundle.rs:1373-1388`" — that ref is STALE; the gate now lives at `bundle.rs:1387-1402`. Re-grep anchors (stable text, not line numbers):

| Anchor text (grep for this) | Current location @940abe9e |
|---|---|
| `descriptor has n={n} placeholders but --slot vec covers` | `bundle.rs:1394-1395`, `verify_bundle.rs:1404-1405` (the exact-coverage gate, **both** sites) |
| `S-VERIFY (PLAN_constellation_bughunt_fix_program.md §292` | `verify_bundle.rs:1392` (the fold-comment to delete; comment block spans `1386-1395`) |
| `--slot @{idx}.phrase derives master fingerprint {master_fp} but descriptor @{idx} annotation specifies {anno}` | `bundle.rs:1634` (the inline-origin fp-mismatch refusal — load-bearing for §4(2) inline-origin cells; see Finding-1 note) |
| `v0.19.0 SPEC §4.12.g — DESCRIPTOR_WITH_NONZERO_ACCOUNT canonicity-gated` | `bundle.rs:1414` (emit-only refusal, STAYS at call site; the `if` body is `1415-1421`) |
| `v0.19.0 SPEC §6.6 row 4 canonical-mode rejection of [Phrase, Path]` | `bundle.rs:1423` (emit-only refusal, STAYS at call site; the block is `1426-1448`) |
| `v0.19.0 SPEC §4.12.b — default-path inference for non-canonical` | `bundle.rs:1450` (lead-in comment for the shared-core span; `default_script_type` bound at `1461-1462`, `if is_non_canonical` opens at `1464`) |
| `Row 19: if inline` | `bundle.rs:1530` (emit-mode-only refusal, INSIDE the override loop → becomes mode-gated) |
| `F4 fix: collapse identical inferred per-` | `bundle.rs:1550` (shared-core span, collapse) |
| `// v0.19.0 SPEC §4.12 — canonicity-aware verify-bundle round-trip` | `verify_bundle.rs:1414` (start of verify's mirror block; `if is_non_canonical` at `1432`, F4 collapse `1504-1509`) |
| `pub fn compute_default_origin_path` | `bundle.rs:2259` |
| `pub fn derivation_path_to_origin` | `bundle.rs:2291` |
| `pub fn origin_to_derivation_path` | `bundle.rs:2344` |
| `fn emit_default_path_notice` | `bundle.rs:2313` (emit-only, STAYS at call site; notice string at `2333`) |

Document the live HEAD SHA in the plan-doc for future readers.

---

## 1. Problem statement (why this is the one genuinely-open dedup, and why it is funds-path)

`bundle --descriptor` (the **emit** path) and `verify-bundle --descriptor`'s `descriptor_mode_verify_run` (the **round-trip cross-check**) each carry a near-identical descriptor-mode binding block that, for a non-canonical descriptor with bare `@N` placeholders, infers a default BIP-48 cosigner origin path and binds each cosigner xpub at that path. The verify side MUST re-derive **byte-identical** paths to the emit side — any silent divergence means `verify-bundle` blesses (or spuriously rejects) a bundle it should not. That is a funds-fidelity verification gap.

This drift is not hypothetical: the **cycle-11b L24 OOB-panic** was exactly this. `bundle.rs` carried the `max(idx+1) != n` exact-coverage gate; `verify_bundle.rs` omitted it, so a contiguous over-`n` slot set passed `validate_slot_set` (which only checks `0..=max_idx` contiguity, not range-vs-`n`) and reached the per-slot override loop, where `new_paths[idx]` OOB-wrote → panic (a panic-DoS). The fix hand-copied the standalone gate into `verify_bundle.rs:1396-1412` and left a fold-comment (`verify_bundle.rs:1392-1395`) literally citing this slug: "fold both into the shared descriptor-mode binding fn when that dedup lands."

The two blocks are **NOT byte-identical** (unlike the already-resolved `restore-emit-dispatch` / `descriptor-origin-extraction` dedups which were kept byte-identical). They diverge in three deliberate, semantically-load-bearing ways (§3). The value of the dedup is that folding the gate + inference + override + collapse into ONE `pub(crate)` fn makes guard-drift **structurally impossible** — the exact class of bug L24 was.

---

## 2. Current behavior (the two blocks, side by side)

### 2.1 `bundle.rs` — emit path, `bundle_run_unified_descriptor` (`bundle.rs:1363`)

Parameters: `args: &BundleArgs`, `slots: &[crate::slot_input::SlotInput]`. Flow:

1. **`bundle.rs:1383-1385`** — `lex_placeholders(&descriptor_str)?` → `resolve_placeholders(&occs)?` into `let mut resolved_placeholders`; `n = resolved_placeholders.n as usize`. *Error mapping:* bare `?` → `ToolkitError::DescriptorParse` (exit 2).
2. **`bundle.rs:1387-1402`** — exact-coverage gate: `if slots.iter().map(|s| s.index as usize + 1).max().unwrap_or(0) != n { return Err(DescriptorParse(...)) }`. **[SHARED CORE]**
3. **`bundle.rs:1410-1412`** — canonicity probe: `parse_descriptor(&descriptor_str, &[], &[])?` → `is_non_canonical = canonical_origin(&probe.tree).is_none()`. *Error mapping:* bare `?` → `DescriptorParse`.
4. **`bundle.rs:1414-1421`** — §4.12.g refusal: `if !is_non_canonical && args.account != 0 { return Err(ModeViolation{ ..DESCRIPTOR_WITH_NONZERO_ACCOUNT }) }`. **[EMIT-ONLY — STAYS AT CALL SITE]**
5. **`bundle.rs:1423-1448`** — §6.6-row-4 refusal: for canonical descriptors, refuse any slot whose subkey set is `{Phrase|Seedqr|Ms1} ∧ Path` (`SlotInputViolation{ kind:"conflict" }`). **[EMIT-ONLY — STAYS AT CALL SITE]**
6. **`bundle.rs:1461-1462`** — H12 default-script-type: `bip48_script_type_for_root_tag(&canonicity_probe.tree.tag)`. **[SHARED CORE]**
7. **`bundle.rs:1463-1563`** — `let mut defaulted_indices: Vec<u8> = Vec::new();` (`:1463`) then `if is_non_canonical { ... }` (`:1464`): build `new_paths` from `resolved_placeholders.path_decl.paths` (Shared/Divergent), pushing to `defaulted_indices` for each empty entry; per-slot `--slot @N.path=` override loop (phrase-bearing slots only) **with the row-19 inline-vs-slot path-mismatch refusal embedded** (`bundle.rs:1530-1543`, `SlotInputViolation{ kind:"path-mismatch" }`); F4 collapse to `Shared` when `all_same` (`:1550+`). Writes back `resolved_placeholders.path_decl.paths`. **[SHARED CORE — except the row-19 refusal, which is emit-mode-gated]**
8. Post-block: the per-slot binding loop (`bundle.rs:1565+`) reads `resolved_placeholders.path_decl.paths` as the per-`@N` anno-path source. The inline-origin fp-consistency refusal (`bundle.rs:1634`, `--slot @N.phrase derives master fingerprint … but descriptor @N annotation specifies …`) lives in this loop — **[CALL SITE — unchanged]**; it is NOT touched by this dedup, but it is load-bearing for the §4(2) inline-origin parity cells (see Finding-1 note in §4(2)).
9. **`bundle.rs:1866-1875`** — `emit_default_path_notice(stderr, &defaulted_indices, ...)` (suppressed when empty). **[EMIT-ONLY — STAYS AT CALL SITE]**

### 2.2 `verify_bundle.rs` — round-trip path, `descriptor_mode_verify_run` (`verify_bundle.rs:1307`)

Parameters: `args: &VerifyBundleArgs`, … . Flow (the mirror block):

1. **`verify_bundle.rs:1374-1382`** — `lex_placeholders(&descriptor_str).map_err(|e| DescriptorReparseFailed{ detail: e.message() })?` → `resolve_placeholders(&occs).map_err(... DescriptorReparseFailed ...)?` into `let mut descriptor_resolved`; `n = descriptor_resolved.n as usize`. *Error mapping DIVERGES:* maps to `DescriptorReparseFailed` (exit **4**), not `DescriptorParse`.
2. **`verify_bundle.rs:1384`** — `validate_slot_set(&args.slot)?`. (bundle calls `validate_slot_set` earlier, outside this fn; verify calls it here. **Keep at call site.**)
3. **`verify_bundle.rs:1386-1412`** — exact-coverage gate (preceded by the S-VERIFY fold-comment `1386-1395`), byte-identical message, iterating `args.slot`, `ToolkitError::DescriptorParse` (exit 2). **[SHARED CORE — currently hand-copied here, carries the S-VERIFY fold-comment to delete]**
4. **`verify_bundle.rs:1420-1426`** — canonicity probe, `.map_err(... DescriptorReparseFailed ...)?` (preceded by the `// v0.19.0 SPEC §4.12 — canonicity-aware …` comment at `1414`). *Error mapping DIVERGES.*
5. *(NO §4.12.g account refusal — verify omits by design.)*
6. *(NO §6.6-row-4 [Phrase,Path] refusal — verify omits by design.)*
7. **`verify_bundle.rs:1432-1510`** — `if is_non_canonical { ... }` (opens `:1432`): H12 default-script-type (`:1438-1439`), build `new_paths` (`:1445-1464`), per-slot override loop **WITHOUT row-19 refusal and WITHOUT `defaulted_indices`** (`verify_bundle.rs:1483-1497`), F4 collapse (`:1504-1509`). Writes back `descriptor_resolved.path_decl.paths`. **[SHARED CORE — except verify has no row-19, no defaulted_indices]**
8. Post-block: the per-slot binding loop (`verify_bundle.rs:1512+`) reads `descriptor_resolved.path_decl.paths`. **[CALL SITE — unchanged]**
9. *(NO notice — verify is read-only.)*

### 2.3 The three deliberate divergences (these define the shared-core boundary)

| # | Concern | bundle (emit) | verify (round-trip) | Disposition |
|---|---|---|---|---|
| D1 | `defaulted_indices` tracking | built (`bundle.rs:1463,1472,~1485,~1547`), drives notice + row-19 | not tracked | Shared core tracks it internally (needed for the row-19 guard + returned to caller); verify caller **discards** the returned vec |
| D2 | Row-19 inline-vs-slot path-mismatch refusal (`bundle.rs:1530-1543`) | present, INSIDE override loop | absent | **Mode-gated** in the shared core (a `DescriptorBindMode::Emit` / `DescriptorBindMode::Verify` discriminator); emit refuses, verify skips |
| D3 | §4.12.g account refusal + §6.6-row-4 [Phrase,Path] refusal + the stderr notice | present, OUTSIDE the binding block (`bundle.rs:1415-1421`, `1426-1448`, `~1869`) | absent | **Stay at the bundle CALL SITE entirely** — never enter the shared core |

The exact-coverage gate (step 2.1.2 / 2.2.3), canonicity-`is_non_canonical` (step …4), H12 inference, new_paths build, override loop (sans row-19), and F4 collapse are **byte-equivalent modulo iteration source + local var name** → these ARE the shared core. *(Caveat — error-precedence: in **both** current call sites the gate runs BEFORE the canonicity probe and BEFORE the emit-only account/row-4 refusals; folding it into the shared fn moves it AFTER. For singly-malformed input this is invisible; for doubly-malformed emit input it reshuffles which refusal surfaces — see §3.2 and §5 item 2. Accepting paths unaffected.)*

---

## 3. The change — extract one `pub(crate)` shared-core fn

### 3.1 New function (in `bundle.rs`, since `pub mod bundle` is visible to verify_bundle.rs — confirmed `cmd/mod.rs:5`)

Add a small mode discriminator + the shared fn. Place both near the existing `compute_default_origin_path` cluster (`bundle.rs:~2249`), so the helpers it calls (`compute_default_origin_path`, `derivation_path_to_origin`, `origin_to_derivation_path`) are co-located.

```rust
/// Distinguishes the emit (`bundle`) caller from the read-only round-trip
/// (`verify-bundle`) caller. The ONLY behavioral difference inside
/// `bind_descriptor_mode_paths` is the row-19 inline-vs-slot path-mismatch
/// refusal (§6.6 row 19): emit refuses, verify-bundle is read-only and
/// silently accepts (it later compares the re-derived md1 byte-for-byte, so a
/// genuinely inconsistent inline/slot path surfaces as a md1 mismatch, not a
/// refusal). All other emit-only refusals (§4.12.g account, §6.6 row-4
/// [Phrase,Path]) and the stderr default-inference notice stay at the bundle
/// CALL SITE and never enter this fn.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum DescriptorBindMode {
    /// `bundle --descriptor` — emit; enforce the row-19 path-mismatch refusal.
    Emit,
    /// `verify-bundle --descriptor` — read-only round-trip; skip row-19.
    Verify,
}

/// SHARED descriptor-mode path binding for non-canonical `@N` descriptors,
/// consumed by BOTH `bundle.rs::bundle_run_unified_descriptor` (emit) and
/// `verify_bundle.rs::descriptor_mode_verify_run` (round-trip cross-check).
/// Folds the cycle-11b L24 exact-coverage gate + H12 default-script-type
/// inference + new_paths default-path build + per-slot `--slot @N.path=`
/// override loop + F4 identical-path Shared-collapse into one site so the two
/// callers can never drift (the L24 panic was exactly that drift).
///
/// MUTATES `path_decl.paths` in place (default-inference + overrides + F4
/// collapse). RETURNS the `defaulted_indices` (the `@N` that received the
/// default path) for the caller's use — `bundle` feeds it to
/// `emit_default_path_notice`; `verify-bundle` discards it.
///
/// Inputs are already-resolved so the caller keeps its OWN error mapping for
/// the lex/resolve/probe steps (bundle → `DescriptorParse`; verify-bundle →
/// `DescriptorReparseFailed`) — those steps deliberately stay at the call site.
///
/// EMIT-ONLY concerns that DO NOT belong here (caller-side): §4.12.g
/// `--account != 0` refusal, §6.6-row-4 `[Phrase,Path]` refusal, the stderr
/// notice. Importing any of them here would change verify-bundle's accept-set.
pub(crate) fn bind_descriptor_mode_paths(
    mode: DescriptorBindMode,
    slots: &[crate::slot_input::SlotInput],
    path_decl: &mut md_codec::origin_path::PathDecl,
    n: usize,
    is_non_canonical: bool,
    root_tag: &md_codec::tag::Tag,
    network: crate::network::CliNetwork,
    account: u32,
) -> Result<Vec<u8>, ToolkitError> {
    use md_codec::origin_path::{OriginPath, PathDeclPaths};

    // cycle-11b L24 — exact-coverage bounds gate. `validate_slot_set` checks
    // contiguity (`0..=max_idx`) only, NOT range-vs-`n`. Folding this here
    // makes the L24 drift (verify omitted it → OOB panic) impossible.
    let covered = slots.iter().map(|s| s.index as usize + 1).max().unwrap_or(0);
    if covered != n {
        return Err(ToolkitError::DescriptorParse(format!(
            "descriptor has n={n} placeholders but --slot vec covers {covered} slots"
        )));
    }

    let mut defaulted_indices: Vec<u8> = Vec::new();
    if !is_non_canonical {
        return Ok(defaulted_indices); // canonical: no inference; caller-side guards already ran.
    }

    // H12 — taproot-aware default-origin script-type (Tr→3', sh→1', else wsh 2').
    let default_script_type = crate::template::bip48_script_type_for_root_tag(root_tag);
    let default_path = compute_default_origin_path(network, account, default_script_type);

    let mut new_paths: Vec<OriginPath> = match &path_decl.paths {
        PathDeclPaths::Shared(op) => {
            if op.components.is_empty() {
                defaulted_indices.extend(0..(n as u8));
                (0..n).map(|_| default_path.clone()).collect()
            } else {
                (0..n).map(|_| op.clone()).collect()
            }
        }
        PathDeclPaths::Divergent(v) => v
            .iter()
            .enumerate()
            .map(|(i, op)| {
                if op.components.is_empty() {
                    defaulted_indices.push(i as u8);
                    default_path.clone()
                } else {
                    op.clone()
                }
            })
            .collect(),
    };

    // Per-slot `--slot @N.path=` overrides (phrase-bearing slots only).
    let mut by_index_path: std::collections::BTreeMap<u8, &crate::slot_input::SlotInput> =
        std::collections::BTreeMap::new();
    for s in slots {
        if s.subkey == crate::slot_input::SlotSubkey::Path {
            by_index_path.insert(s.index, s);
        }
    }
    let mut by_index_subkeys: std::collections::BTreeMap<
        u8,
        std::collections::BTreeSet<crate::slot_input::SlotSubkey>,
    > = std::collections::BTreeMap::new();
    for s in slots {
        by_index_subkeys.entry(s.index).or_default().insert(s.subkey);
    }
    for (idx, slot_path) in &by_index_path {
        let subkeys = by_index_subkeys.get(idx).cloned().unwrap_or_default();
        if !subkeys.contains(&crate::slot_input::SlotSubkey::Phrase)
            && !subkeys.contains(&crate::slot_input::SlotSubkey::Seedqr)
            && !subkeys.contains(&crate::slot_input::SlotSubkey::Ms1)
        {
            continue;
        }
        let user_path = DerivationPath::from_str(&slot_path.value)
            .map_err(|e| ToolkitError::BadInput(format!("--slot @{idx}.path parse: {e}")))?;
        let user_origin = derivation_path_to_origin(&user_path);
        // §6.6 row 19 — EMIT-ONLY: inline `[fp/path]@N` AND `--slot @N.path=`
        // both supplied, non-empty, and differ → refuse. verify-bundle skips:
        // it re-derives read-only and surfaces a genuine conflict as a md1
        // byte-mismatch downstream, never as a refusal (preserves its accept-set).
        if mode == DescriptorBindMode::Emit
            && !defaulted_indices.contains(idx)
            && !new_paths[*idx as usize].components.is_empty()
            && new_paths[*idx as usize] != user_origin
        {
            let inline_path = origin_to_derivation_path(&new_paths[*idx as usize])?;
            return Err(ToolkitError::SlotInputViolation {
                kind: "path-mismatch",
                message: format!(
                    "slot @{idx} path mismatch: --slot says {user_path}, descriptor inline [.../{inline_path}] disagrees; supply consistent values or remove one source."
                ),
            });
        }
        new_paths[*idx as usize] = user_origin;
        defaulted_indices.retain(|i| i != idx);
    }

    // F4 — collapse identical inferred paths to `Shared` for cross-start
    // byte-convergence with the explicit-origin / wallet-import path.
    let all_same = new_paths.windows(2).all(|w| w[0] == w[1]);
    path_decl.paths = if all_same {
        PathDeclPaths::Shared(new_paths[0].clone())
    } else {
        PathDeclPaths::Divergent(new_paths)
    };

    Ok(defaulted_indices)
}
```

**Re-grep before lifting:** the body above is transcribed from `bundle.rs:1387-1563` @940abe9e. The implementer MUST diff the lifted body against current `HEAD` (the override-loop comment text, the `slot_path.value` deref via `SecretString: Deref<Target=str>`, the F4 comment) and reconcile any drift before committing. The `Tag` type is `md_codec::tag::Tag` (confirmed `template.rs:11` imports it; `bip48_script_type_for_root_tag(tag: &Tag)` at `template.rs:295`); `PathDecl` carries `pub paths: PathDeclPaths` and is the field type at `parse_descriptor.rs:204` (`pub path_decl: PathDecl`). `pub mod bundle` is visible to `verify_bundle.rs` (confirmed `cmd/mod.rs:5`).

### 3.2 Edit `bundle.rs` call site (`bundle_run_unified_descriptor`)

Replace the inline span **`bundle.rs:1387-1402`** (gate) **and `bundle.rs:1450-1563`** (H12 + inference + override + collapse) with:

- KEEP `bundle.rs:1404-1412` (canonicity probe + `is_non_canonical`) at the call site (its `?` → `DescriptorParse` error mapping is caller-specific).
- KEEP `bundle.rs:1414-1421` (§4.12.g account refusal) at the call site.
- KEEP `bundle.rs:1423-1448` (§6.6-row-4 refusal) at the call site.
- KEEP the `default_script_type` binding for the notice (`bundle.rs:1461-1462`) **at the call site** — it is needed by `emit_default_path_notice` at `bundle.rs:1869-1875`. (The shared fn recomputes it internally from `root_tag`; harmless — it is a cheap pure tag→u32 map. Alternatively have the shared fn also return it, but recompute-at-call-site is simpler and keeps the signature lean. The plan-doc picks one; recompute is the default.)
- Replace the gate + the `if is_non_canonical { ... }` inference block with a single call:

```rust
let defaulted_indices = crate::cmd::bundle::bind_descriptor_mode_paths(
    DescriptorBindMode::Emit,
    slots,
    &mut resolved_placeholders.path_decl,
    n,
    is_non_canonical,
    &canonicity_probe.tree.tag,
    args.network,
    args.account,
)?;
```

`defaulted_indices` then flows unchanged into `emit_default_path_notice(stderr, &defaulted_indices, args.network, args.account, default_script_type)` at `bundle.rs:1869`. Downstream consumers of `resolved_placeholders.path_decl.paths` (the per-slot binding loop `bundle.rs:1565+`, the propagation `bundle.rs:1820`) are unchanged.

**Ordering note (load-bearing — THREE re-orders, not one).** In the current emit code the gate runs at `bundle.rs:1387` BEFORE *all three* of: (a) the canonicity probe (`:1410`), (b) the §4.12.g `--account != 0` refusal (`:1415-1421`), (c) the §6.6-row-4 `[Phrase,Path]` refusal (`:1426-1448`). The shared fn does the gate FIRST internally, and the shared fn can only run AFTER the probe (it consumes `is_non_canonical` + `root_tag`, both probe-derived) AND after the call-site account/row-4 refusals (those stay at the call site, ahead of the call). So folding re-orders the gate vs ALL THREE:

- **gate-vs-probe** — for "lex/resolve succeeds but `parse_descriptor` errors AND slot vec over-/under-`n`," the surfaced error flips `DescriptorParse('n placeholders but …')` → `DescriptorParse(probe-parse-error)` (both exit 2).
- **gate-vs-account** — for "over-`n` slot vec AND `--account != 0` on a CANONICAL descriptor," it flips `DescriptorParse('n placeholders but …')` → `ModeViolation(DESCRIPTOR_WITH_NONZERO_ACCOUNT)` (exit 2 → the ModeViolation exit code).
- **gate-vs-row4** — for "over-`n` slot vec AND a `[Phrase,Path]` conflict on a CANONICAL descriptor," it flips `DescriptorParse('n placeholders but …')` → `SlotInputViolation{kind:"conflict"}`.

All three are exit-≠0 refusals on **doubly-malformed** input that gets refused either way — no funds / accept-set / exit-class consequence (every path still exits ≠0). This is the ONLY observable delta of the dedup (§5 item 2). RESOLUTION: accept the re-order; do NOT try to keep a thin pre-probe/pre-refusal gate copy (that would re-introduce the duplicated gate the dedup exists to kill). The regression oracle (§4) must include a cell asserting an over-`n` slot vec on a PARSE-FAILING descriptor still exits ≠0 (either error acceptable; assert non-zero exit, not the message). The plan-doc R0 must adversarially confirm **no existing test pins ANY of the three precedences** — gate-before-probe, gate-before-account-refusal, OR gate-before-row4-refusal. *(Recon finding @940abe9e: NO test combines an over-`n` slot vec with a parse-failing descriptor, with `--account != 0`, OR with a row-4 conflict; the ONLY test asserting the gate message is the verify-side L24 cell `cli_non_canonical_descriptor.rs:338`. The two L24 cells use a well-formed non-canonical descriptor that parses cleanly with `--account` unset and no row-4 conflict, so all three orderings are unobservable for them — they stay GREEN. No new gate cells beyond the parse-failing-exit-≠0 cell are required.)*

### 3.3 Edit `verify_bundle.rs` call site (`descriptor_mode_verify_run`)

- DELETE the hand-copied gate **`verify_bundle.rs:1386-1412`** (the cycle-11b L24 comment block + the `if args.slot.iter()...!= n` gate). The S-VERIFY fold-comment `verify_bundle.rs:1392-1395` is deleted with it.
- DELETE the H12 + inference + override + collapse block **`verify_bundle.rs:1428-1510`** (`if is_non_canonical { ... }`).
- KEEP `verify_bundle.rs:1374-1382` (lex/resolve with `DescriptorReparseFailed` mapping).
- KEEP `verify_bundle.rs:1384` (`validate_slot_set(&args.slot)?`).
- KEEP `verify_bundle.rs:1420-1426` (canonicity probe with `DescriptorReparseFailed` mapping).
- Insert the single call AFTER the probe (verify discards the returned vec):

```rust
let _defaulted = crate::cmd::bundle::bind_descriptor_mode_paths(
    crate::cmd::bundle::DescriptorBindMode::Verify,
    &args.slot,
    &mut descriptor_resolved.path_decl,
    n,
    is_non_canonical,
    &canonicity_probe.tree.tag,
    args.network,
    args.account,
)?;
```

Downstream consumers of `descriptor_resolved.path_decl.paths` (the per-slot binding loop `verify_bundle.rs:1512+`, the propagation `verify_bundle.rs:1748`) are unchanged. `DescriptorBindMode::Verify` ⇒ no row-19 refusal, so verify's accept-set is byte-preserved. The gate now runs after the probe in verify too (verify *already* ran the gate after `validate_slot_set` but BEFORE the probe at `1420`; this is the gate-vs-probe re-order of §3.2 — same disposition. Verify carries NO §4.12.g account refusal and NO §6.6-row-4 refusal, so the gate-vs-account / gate-vs-row4 re-orders are emit-side ONLY and do not apply here).

### 3.4 Imports

- `bundle.rs`: `bind_descriptor_mode_paths` uses `DerivationPath`, `OriginPath`, `PathDeclPaths`, `SlotSubkey`, `Tag` — all already imported or `use`d locally in the body (transcribe the `use` lines from the current inline block). `DescriptorBindMode` is a new `pub(crate) enum` in `bundle.rs`.
- `verify_bundle.rs`: drop now-dead local `use` of `OriginPath`/`PathDeclPaths` if the deleted block was their only consumer (grep — they may still be used in the per-slot loop at `1537`). Reference the shared fn + mode via the fully-qualified `crate::cmd::bundle::` path (as verify already does for `compute_default_origin_path` / `derivation_path_to_origin` — confirmed `verify_bundle.rs:1440,1496`).

### 3.5 No new `ToolkitError` variant

The shared fn reuses existing variants verbatim: `DescriptorParse` (gate), `BadInput` (override-path parse), `SlotInputViolation` (row-19). **No new `enum ToolkitError` variant** → the alphabetical-ordering convention (CLAUDE.md) does not engage. (The DECISIONS prompt's "a typed error" phrasing refers to the EXISTING `DescriptorParse` the gate already returns — it does NOT require adding a variant. Confirm at plan-R0.)

---

## 4. Test / parity surface (the deliverable — parity IS the regression oracle)

All three groups live in `crates/mnemonic-toolkit/tests/cli_non_canonical_descriptor.rs` (new cells) and reuse the existing fixtures `TREZOR_12_ZERO` / `BIP39_TEST_2` / `BIP39_TEST_3` (`tests/cli_non_canonical_descriptor.rs:10-14`).

**Harness model — TWO existing patterns, pick the n≥2-correct one.** The bundle-emit-JSON → verify-bundle round-trip has two reference harnesses, and they treat the cards differently:

- `descriptor_bundle_round_trips_through_verify_bundle` (`tests/cli_descriptor_mode.rs:79-145`) — **SINGLE-SIG** (`wpkh(@0…)`). It reads `ms1` as a scalar (`bundle["ms1"][0]`, `:99`) and `mk1`/`md1` as **FLAT** string arrays (`bundle["mk1"].as_array().iter().map(as_str)`, `:100-111`). It passes `--ms1 <scalar>` plus one `--mk1`/`--md1` per flat entry.
- `bundle_then_verify_flags` (`tests/cli_descriptor_mode.rs:220-247`) — the helper that ALREADY handles the **NESTED** mk1: `for inner in v["mk1"]... for chunk in inner.as_array()` (`:240-242`). It flattens `md1` flat but `mk1` two levels deep.

**CRITICAL (Finding-2 fold) — n≥2 MULTISIG mk1 is NESTED, not flat.** For an n≥2 descriptor bundle the `--json` mk1 is `[[chunk,chunk],[chunk,chunk]]` (outer len = #cosigners, each inner = that cosigner's continuation chunks), while `md1` stays flat `[c,c,c,c]` and `ms1` is a length-N array `[str,str]`. **A parity harness that copies the SINGLE-SIG flat-`mk1` pattern from `descriptor_bundle_round_trips_through_verify_bundle` truncates the per-cosigner continuation chunks** → every wsh/sh-wsh/slot-override (n≥2) cell goes spuriously RED with `result=mismatch` / `fails=['mk1_decode[0]','mk1_decode[1]']`, indistinguishable from a real emit↔verify desync. The new parity harness MUST:
  1. **Recursively flatten the nested `mk1`** (`list[list[str]]`) into individual `--mk1 <chunk>` flags — emit EVERY inner chunk of EVERY cosigner (the `bundle_then_verify_flags` `:240-242` double-loop, NOT the single-sig `:100-105` single-loop). Generalizing `bundle_then_verify_flags` to take a `--slot` vec is acceptable, but note it currently handles only flat `md1`/nested `mk1` for the *watch-only single-sig* path — extend it to also emit `--ms1`.
  2. **Emit EVERY `--ms1` entry** (the `ms1` array is length-N; pass `--ms1 <e>` for each entry, not just `ms1[0]`). The no-`ms1` variant fails `ms1_decode`/`ms1_entropy_match` for these secret-bearing phrase-slot bundles.

**Self-test before trusting any RED (mandatory, mirrored into the plan):** add an assertion that the parity harness reproduces `result=ok` / all-checks-passed for the **wsh n=2 all-elided cell ON CURRENT SOURCE BEFORE the dedup edit** (or as a first-landed cell). A green there proves the harness is correct; only then does a RED elsewhere implicate the dedup rather than a harness card-shape bug.

### (1) KEEP the two existing L24 cells GREEN throughout (the gate-equivalence proof)

- `verify_bundle_descriptor_slot_over_n_rejects_not_panics` (`cli_non_canonical_descriptor.rs:338`) — over-`n` typed `DescriptorParse` refusal, exit 2, message `descriptor has n=2 placeholders but --slot vec covers 3 slots`. MUST stay GREEN (proves the shared gate fires identically on the verify side via `DescriptorBindMode::Verify`).
- `verify_bundle_descriptor_exact_coverage_path_override_does_not_over_fire` (`cli_non_canonical_descriptor.rs:386`) — in-range exact-coverage path-override does NOT trip the gate. MUST stay GREEN (proves the shared gate is exact-coverage `!= n`, not a sloppy `>`).

These are characterization tests of the EXACT span that drifted; they are the dedup's primary safety net.

### (2) ADD the bundle ↔ verify-bundle path-derivation PARITY matrix (the regression oracle)

New test(s): for each cell in the matrix, **emit a bundle (`--json`), extract ALL of `ms1`/`mk1`/`md1`, then `verify-bundle` the SAME cards + same descriptor + same slots, assert `verify["result"] == "ok"` and every `checks[].passed == true`**. Card extraction must follow the n≥2-correct shape (the `cli_descriptor_mode.rs:113-144` assertion logic for the `result==ok`/all-passed check, but the CARD-FLATTENING per the Finding-2 notes: nested `mk1` double-loop `:240-242` + every `--ms1` entry — NOT the single-sig flat `mk1` at `:100-105`). A PASS proves the two derivations agree byte-for-byte — i.e. the dedup did not desync emit from verify.

Matrix = {non-canonical descriptor roots} × {path_decl shapes}:

- **Roots (drives H12 leaf):**
  - `wsh(...)` general-policy → H12 leaf `2'` (wsh). e.g. `wsh(andor(pkh(@0),after(12000000),pk(@1)))` (n=2, the L24 fixture).
  - `sh(wsh(...))` → H12 leaf `1'` (sh-wsh). Construct a non-canonical sh-wsh general policy (re-grep an existing sh-wsh non-canonical fixture or build one; confirm `bip48_script_type_for_root_tag` returns 1 for the sh root tag — see `bundle.rs:3065` `compute_default_origin_path_wsh_and_shwsh_unchanged`).
  - `tr(NUMS,...)` → H12 leaf `3'` (taproot). e.g. `tr(NUMS,and_v(v:pk(@0),after(12000000)))` (n=1, the `tr_nums_default_path_self_check_round_trips` fixture, `cli_non_canonical_descriptor.rs:262`).
- **path_decl shapes (per descriptor where arity allows):**
  - **all-elided-defaulted** — bare `@N`, no inline origin, no `--slot @N.path=` → all slots get the BIP-48 default. (The dominant case; both L24 fixtures are this.)
  - **shared-explicit** — every `@N` carries the SAME inline `[fp/path]@N` origin → Shared, no defaulting.
  - **divergent** — `@N` carry DIFFERENT inline origins → Divergent, no defaulting (n≥2 only).
  - **--slot path override** — bare `@N` + `--slot @N.path=m/...` for a phrase-bearing slot → override replaces the default.
  - **mixed** — some `@N` elided (defaulted), others inline-origin or slot-overridden (n≥2 only).

**CRITICAL (Finding-1 fold) — inline-origin cells MUST embed the slot's DERIVED master fingerprint, or they silently never exercise the path they exist to cover.** `bundle` REFUSES a non-canonical descriptor whose inline `[fp/path]@N` fingerprint does NOT match the phrase slot's derived master fingerprint — at `bundle.rs:1634`: `error: --slot @N.phrase derives master fingerprint <derived> but descriptor @N annotation specifies <anno>` (exit ≠ 0). This refusal is EARLIER and DISTINCT from the binding path the dedup touches. So the **shared-explicit** and **divergent** inline-origin cells MUST use each phrase slot's true derived fingerprint in the `[fp/path]@N` annotation; a placeholder/wrong fp produces a **false-RED** (the fp-mismatch refusal) that masks the intended Shared/Divergent coverage entirely — the cell never reaches `is_non_canonical` binding, so it tests nothing the matrix means to test. **Known-fixture fp table (mainnet, verified live): `@0 = TREZOR_12_ZERO → 73c5da0a`, `@1 = BIP39_TEST_2 → b8688df1`.** Use these in the inline annotations (or compute each slot's fp via `mk`/a fixture helper). With CORRECT fps the divergent cell DOES round-trip (`result=ok`, verified). Implementer note: the inline `[fp/path]@N` annotation must keep the descriptor non-canonical (the `andor(...)`/`and_v(...)` general-policy wrapper does so even with explicit origins) — optionally assert each inline-origin cell still classifies non-canonical so it genuinely enters the `is_non_canonical` shared-core block rather than the canonical caller-side guards.

Minimum coverage: every (root × shape) combination that the root's arity supports. The n=1 `tr` covers all-elided + slot-override; the n=2 `wsh`/`sh-wsh` cover all five. Each cell = one bundle-emit + one verify-bundle, assert PASS. (Implementer may table-drive to keep the cell count manageable; the `bundle_then_verify_flags` helper can be generalized to take the slot vec — but only if extended per the Finding-2 nested-mk1 / `--ms1` notes above.)

**Why this is the oracle, not a tautology:** before the dedup, emit and verify ran SEPARATE copies of the inference. A parity PASS across the matrix proves the single shared fn produces the path both sides expect. If a future edit to the shared fn breaks one side's expectation, the parity cell goes RED — exactly the L24 class, now caught at the unit-of-derivation.

### (3) ADD characterization cells pinning the divergences (D1/D2/D3 stay put)

- **bundle RETAINS §6.6-row-19 path-mismatch refusal** post-dedup: `bundle --descriptor <non-canonical with inline [fp/path]@0> --slot @0.phrase=... --slot @0.path=<DIFFERENT path>` → exit ≠ 0, stderr contains `path mismatch` / `disagrees`. (Construct so `@0` is NOT defaulted — has an inline origin — and the slot path differs.)
- **verify-bundle does NOT acquire the row-19 refusal**: the SAME inline-vs-slot-mismatch inputs fed to `verify-bundle` (with valid `--ms1/--mk1/--md1` from a matching emit, OR empty sentinels to reach the binding) do NOT produce the `path mismatch` refusal — verify either PASSES (if the override happens to agree post-derivation) or fails later as a md1-comparison mismatch, NOT as `SlotInputViolation{kind:"path-mismatch"}`. Assert the stderr does NOT contain `path mismatch`/`disagrees` from the binding stage. *(This is the test that would have RED-flagged accidentally importing the row-19 refusal into the verify path — the precise regression the recon's R0 warns about.)*
- **bundle RETAINS §4.12.g `--account != 0` refusal** for canonical descriptors post-dedup: already pinned by `canonical_wsh_sortedmulti_with_nonzero_account_refuses` (`cli_non_canonical_descriptor.rs:120`) — KEEP GREEN.
- **bundle RETAINS the stderr default-inference notice** post-dedup: already pinned by `non_canonical_wsh_andor_default_path_inference_emits_bundle` (`cli_non_canonical_descriptor.rs:21`, asserts the `info: non-canonical descriptor` notice) + `canonical_descriptor_does_not_emit_default_path_notice` (`:202`) — KEEP GREEN.
- **verify-bundle still emits NO notice** post-dedup: add (or confirm) a cell asserting `verify-bundle --descriptor <non-canonical all-elided>` stderr does NOT contain `info: non-canonical descriptor; defaulting origin path` (the notice string from `emit_default_path_notice`, `bundle.rs:2331-2334`).
- **bundle RETAINS §6.6-row-4 [Phrase,Path] canonical refusal** post-dedup: already pinned by `canonical_descriptor_refuses_phrase_plus_path_subkey_pair` (`cli_non_canonical_descriptor.rs:285`) — KEEP GREEN.

### Test-run discipline

Run the **FULL package suite** (`cargo test -p mnemonic-toolkit`), NOT targeted `--test cli_non_canonical_descriptor` / `--test cli_descriptor_mode`. Per MEMORY `feedback_r0_review_run_full_package_suite`: a CLI/binding-phase edit ripples into argv/schema/version lints (`lint_argv_secret_flags`, `schema_mirror` via the pinned binary, `both_readmes`) that targeted targets miss. The per-phase R0 and the whole-diff post-impl review BOTH run the full suite. Also run `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`.

---

## 5. SemVer + regression-risk handling

**SemVer: NO-BUMP.** No clap surface, no `--json` wire-shape, no error-kind enum change, no exit-code-class change. The dedup is behavior-preserving on every **accepting** path (byte-for-byte) and on every **singly-malformed** refusal. The **only** observable delta is an error-precedence reshuffle on **doubly-malformed** emit input (over-`n` slot vec combined with a parse-failing descriptor, OR `--account != 0` on canonical, OR a row-4 `[Phrase,Path]` conflict): the surfaced refusal message/variant flips, but the input is refused (exit ≠ 0) either way — no funds, accept-set, or exit-class impact. The blanket "zero observable behavior change" phrasing is therefore inaccurate and is replaced by this narrowed claim. The parity matrix + the retained characterization cells prove the accepting/singly-malformed paths are preserved.

**Regression risk: MEDIUM-HIGH (funds-path, NOT mechanical).** Mitigations, in order:

1. **The two blocks are not byte-identical** — three deliberate divergences (D1 `defaulted_indices`, D2 row-19, D3 caller-side refusals/notice). The mode discriminator (`DescriptorBindMode`) is the ONLY behavioral knob; D3 never enters the fn. Plan-R0 must adversarially confirm the shared-core boundary matches §2.3 exactly — that no emit-only refusal leaks into verify, and none is dropped from bundle.
2. **Gate re-order — THREE precedences, not one** (§3.2 note): folding the gate into the shared fn moves it AFTER (a) the canonicity probe, (b) the emit-only §4.12.g `--account != 0` refusal, and (c) the emit-only §6.6-row-4 `[Phrase,Path]` refusal. On **doubly-malformed** emit input the surfaced refusal flips (`DescriptorParse('n placeholders but …')` → the probe-error / `ModeViolation` / `SlotInputViolation{conflict}`), but every path stays exit-≠0. This is the sole observable delta (no funds/accept-set/exit-class change). Mitigations: add a cell asserting an over-`n` slot vec on a parse-failing descriptor still exits ≠0 (assert non-zero, not the message); the plan-doc R0 must adversarially confirm **no existing test pins ANY of the three precedences** — gate-before-probe, gate-before-account-refusal, OR gate-before-row4-refusal. (Recon @940abe9e: none do — no test combines over-`n` with a parse-failing descriptor / `--account != 0` / a row-4 conflict; the only gate-message assertion is the verify-side L24 cell `cli_non_canonical_descriptor.rs:338`, whose fixtures parse cleanly with `--account` unset and no row-4 conflict.)
3. **Error-mapping divergence stays at the call site** — lex/resolve/probe keep their per-caller `DescriptorParse` (bundle) vs `DescriptorReparseFailed` (verify) mapping. The shared fn does NOT do lex/resolve/probe; it takes already-resolved inputs. This is why the fn signature takes `n` + `is_non_canonical` + `root_tag` rather than the descriptor string.
4. **`SecretString` deref** — `slot_path.value` is `SecretString` (`slot_input.rs:105`); `DerivationPath::from_str(&slot_path.value)` works via `Deref<Target=str>` (used identically in both current blocks). No secret-hygiene change; no lint-floor move.
5. **The parity matrix is the structural oracle** — a desync between emit and verify after this or any future edit goes RED at the bundle↔verify cell, which is the exact L24 class.
6. **Whole-diff post-impl R0** (mandatory, non-deferrable) re-reads the full diff for implementation-introduced regressions TDD misses (per CLAUDE.md post-implementation review), running the full suite + clippy.

---

## 6. FOLLOWUPS.md flip (in the shipping commit)

On ship, flip `design/FOLLOWUPS.md:103` from `OPEN` to `✓ RESOLVED — toolkit NO-BUMP <SHA>` and update the body to record the shared fn name + the parity-test name. Per MEMORY `feedback_followup_status_discipline`: flip in the SAME commit that lands the dedup. Delete the now-obsolete S-VERIFY fold-comment in `verify_bundle.rs` (done as part of §3.3).

**Batch-adjacent (out of scope for THIS spec, but flagged by the recon):** four sibling slugs are already-resolved-but-header-unflipped (`restore-emit-dispatch-3way-dedup` :450, `cmd-repair-inspect-helper-duplication` :2777, `synthesize-descriptor-deduplicate-with-unified` :2610; `descriptor-origin-extraction-dedup` already flipped). The recon recommends batching those four header `✓ RESOLVED` flips into one docs commit. That is a SEPARATE docs-only change, not part of this funds-path dedup — keep the commits disjoint.

---

## 7. Exact file inventory

| File | Change |
|---|---|
| `crates/mnemonic-toolkit/src/cmd/bundle.rs` | ADD `pub(crate) enum DescriptorBindMode` + `pub(crate) fn bind_descriptor_mode_paths(...)`; DELETE inline gate (`~1387-1402`) + inference block (`~1450-1563`); INSERT `bind_descriptor_mode_paths(Emit, ...)` call; keep §4.12.g / §6.6-row-4 refusals + notice at call site |
| `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` | DELETE hand-copied gate (`~1386-1412`, incl. S-VERIFY fold-comment) + inference block (`~1428-1510`); INSERT `bind_descriptor_mode_paths(Verify, ...)` call; prune now-dead local `use`s |
| `crates/mnemonic-toolkit/tests/cli_non_canonical_descriptor.rs` | KEEP the 2 L24 cells GREEN; ADD the bundle↔verify parity matrix + the D1/D2/D3 characterization cells (verify-no-notice, verify-no-row19, bundle-retains-row19) |
| `design/FOLLOWUPS.md` | flip `:103` OPEN → ✓ RESOLVED (shipping commit) |

No other files. No `Cargo.toml`/`Cargo.lock`/README/install.sh (NO-BUMP). No manual, no GUI schema.
