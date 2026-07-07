# SPEC — bitcoin-core-receive-change-pair-merge

**Restore standard Bitcoin Core `listdescriptors` import by recombining Core's split receive/change
descriptors into a single `<0;1>/*` multipath entry at parse time.**

- **Author:** (this session) — single-author design per CLAUDE.md phase-2 convention.
- **Source SHA (all line citations grep-verified against this):** `d9063523` (origin/master at recon time).
- **FOLLOWUP slug:** `bitcoin-core-receive-change-pair-merge` (`design/FOLLOWUPS.md`, filed 2026-07-06, split out of Cycle A / v0.76.0 per plan-R0 I-2).
- **Recon:** `cycle-prep-recon-bitcoin-core-receive-change-pair-merge.md` (2026-07-06, SHA `d9063523`).
- **Target release:** `mnemonic-toolkit-v0.77.0` (MINOR). md/ms/mk codecs **NO-BUMP**. Paired `mnemonic-gui` MINOR (wire-shape consumer).
- **Status:** DRAFT — pending opus-architect R0 loop to 0C/0I before any implementation (mandatory, CLAUDE.md Conventions bullet 1).

---

## §0 — Scope, decisions locked with the user (2026-07-06)

| Fork | Decision |
|------|----------|
| Cycle scope | **Full lockstep**: toolkit merge + `bool→Option<bool>` ripple + tests, PLUS paired `mnemonic-gui` `--json` PR AND manual prose fix, all in this cycle. |
| Merge-guard strictness | **Maximally strict** (§4.2 guard matrix). |
| Merged-entry `--select-descriptor` | A merged `<0;1>` entry (`internal = None`) satisfies **both** `active-receive` AND `active-change`. |
| Unmergeable fixed-step | Still **exit-2 reject** (funds-safe floor unchanged), but a distinct-key receive/change-**shaped** near-miss gets a **differentiated** message (§7). |

**Non-goals (YAGNI — explicitly OUT):**
1. `/**` double-star shorthand (`bip389-double-star-shorthand-support`) — separate cycle.
2. Concrete non-ranged xpub implied-wildcard (`concrete-nonranged-xpub-implied-wildcard`) — separate cycle.
3. Merging across >2 entries, across accounts, or non-adjacent steps — these DO NOT merge (they reject); we do not invent a general multipath-combiner.
4. Any codec (md/mk/ms) change — the toolkit already parses `<0;1>/*` fine (`core_multipath_split_to_receive_change`, `cli_import_wallet_bitcoin_core.rs:369`); this is pure toolkit parse-layer.
5. Emitting native multipath on the toolkit's OWN export path — unrelated; export is untouched.

---

## §1 — Problem & current (v0.76.0) behavior

Bitcoin Core `listdescriptors` exports a wallet's receive and change chains as **two SEPARATE
single-path descriptors** — `…/0/*` (`"internal": false`) and `…/1/*` (`"internal": true`) — never a
combined `<0;1>` multipath (see §2). Since Cycle A (v0.76.0) the descriptor lexer fail-closed-rejects any
**fixed single use-site step**: `parse_descriptor.rs:205-210` emits

> `…the use-site path must be a multipath /<a;b>/*  (or bare /*) as the final step — a fixed single step
> like /0/* (or the /** shorthand) is un-representable (found residue near <residue>)`

Consequently a standard Core two-entry export **hard-fails** today (entry 0's `/0/*` rejects before entry 1
is reached). This is the correct funds-safe floor (md1 cannot hold a fixed step; silently collapsing `/0/*`
→ `/*` was the C1 fund-loss bug Cycle A closed) — but it makes the mainstream Core import path **unusable**
without the documented hand-combine-to-`<0;1>/*` + `--format descriptor` workaround.

**This cycle** adds a parse-time pre-pass that recombines the same-key receive/change pair into one
`<0;1>/*` multipath entry (which the lexer already accepts), restoring standard Core import — WITHOUT
weakening the fixed-step floor for anything that is not a provable same-key adjacent pair.

## §2 — Primary-source protocol basis (verified, not assumed)

Verified against `bitcoin/bitcoin doc/descriptors.md` (master) + Bitcoin Optech output-script-descriptors +
achow101 "0.21 wallets" + PR #22838:

- BIP-389 multipath `<0;1>` is an **import-time** convenience. On import, Core **expands** the multipath
  into two separate single-path descriptors and **stores them independently** (the second implicitly the
  internal/change chain).
- `listdescriptors` lists the **separately-stored** descriptors → it therefore re-emits the **split**
  `/0/*` + `/1/*` form, never a combined `<0;1>`. Multipath was import-only (PR #22838).
- **Robustness caveat (design-safe either way):** ongoing Core work trends toward native-multipath storage
  (doc PR #34100 rewrites examples to multipath). If a future Core emits `<0;1>` directly, that entry has
  **no fixed final step**, never enters the merge pre-pass (§4.2 requires a fixed step), and is handled by
  the existing multipath parse path. So this design needs no version gate.

## §3 — Current-source anchor points (grep-verified @ `d9063523`)

| Symbol / site | Location | Role |
|---|---|---|
| `BitcoinCoreParser::parse` | `wallet_import/bitcoin_core.rs:142` | per-entry loop at `:207-211`; **merge pre-pass inserts before `:207`** |
| `parse_entry` | `wallet_import/bitcoin_core.rs:216` | per-entry decode; `verify_checksum` at `:257`; builds `CoreSourceMetadata` at `:337`; sets `internal` at `:339` |
| `struct CoreSourceMetadata` | `wallet_import/mod.rs:353-364` | `pub(crate) internal: bool` at `:355` → becomes `Option<bool>` |
| `apply_select_descriptor` | `wallet_import/mod.rs:394` | `ActiveReceive` = `active && !internal`; `ActiveChange` = `active && internal` (doc `:383-386`) |
| `sample_core_metadata` (test) | `wallet_import/mod.rs:534-537` | test builder, ripples |
| `--json` `source_metadata.internal` | `cmd/import_wallet.rs:1859` | `"internal": meta.internal` |
| text-summary `internal` | `cmd/import_wallet.rs:2265` | `bundles[{i}].internal={}` |
| checksum recompute prior art | `wallet_import/descriptor.rs:246`, `electrum.rs:1027` | private per-parser copies via `miniscript::descriptor::checksum::Engine` |
| fixed-step floor reject | `parse_descriptor.rs:205-210` | fires inside `concrete_keys_to_placeholders`→`lex_placeholders` |
| positive merge INPUT fixture | `tests/fixtures/wallet_import/core-mainnet-receive-change-pair.json` | same-key `/0/*`+`/1/*` (KEPT v0.76.0) |
| current reject tests (to FLIP) | `tests/cli_import_wallet_bitcoin_core.rs:926` (file), `:1108` (inline) | assert exit-2 today → become merge-accept |
| distinct-multipath negative (KEEP green) | `tests/cli_import_wallet_bitcoin_core.rs:952` | already-`<0;1>` distinct entries; NOT the merge discriminator |

## §4 — The merge pre-pass

### §4.1 Placement & shape

A new function, `merge_receive_change_pairs(descriptors: &[Value], stderr) -> Result<Vec<Value>, ToolkitError>`
(or an in-place `Vec<Value>` rewrite), invoked in `parse` **after** `descriptors`/`wallet_name` are
extracted (`bitcoin_core.rs:163-177`) and **before** the ParsedImport loop (`:207`). It operates on the raw
JSON `descriptors` array so the merged `<a;b>/*` string flows through the **unchanged** `parse_entry`
pipeline (checksum-validate → placeholder → parse_descriptor). This keeps one decode path and avoids a
second `ParsedImport`-level merge.

For each entry it must extract a **merge key** from the descriptor body (checksum-stripped): the ordered
list of `(fingerprint, full origin path, xpub)` per key expression, the script template, the threshold, and
the **use-site final step(s)** per key. Reuse existing extractors where possible
(`extract_origin_components`, `build_slot_fields`) rather than a new ad-hoc parser.

### §4.2 Guard matrix — merge IFF ALL hold (maximally strict)

Two entries `A` and `B` merge into one `<stepA;stepB>/*` entry **iff every** condition holds; ANY deviation
⇒ do **not** merge (leave both entries as-is; they then reject at the floor per §7):

1. **Script/threshold identical** — same descriptor template (e.g. both `wpkh(...)`, or both
   `wsh(sortedmulti(2,...))` with the same threshold and same key order).
2. **All key material identical** — for every key expression, identical `(fingerprint, full origin path,
   xpub)`. Never merge across differing keys/origins/accounts. (This is the funds-critical discriminator —
   distinct keys are DIFFERENT wallets.)
3. **Each side is a fixed single final use-site step** — both carry exactly one non-multipath, non-wildcard
   fixed integer final step (e.g. `/0/*` and `/1/*`). An already-multipath (`<…>`) or bare-`/*` side never
   participates.
4. **Final steps differ** — `stepA != stepB` (a pair with identical steps is not a receive/change pair).
5. **`internal` flags disagree** — exactly one has `internal:true`, the other `internal:false`. (A
   missing/absent `internal` defaults `false` per `parse_bool_field`; two `false` do not pair.)
6. **Exactly two** entries share the merge key. Three-or-more sharing the key ⇒ ambiguous ⇒ do NOT merge
   (reject); emit a NOTICE naming the ambiguity.
7. **Multi-key (multisig) uniformity** — for a multisig descriptor, condition 3-4 apply **per key** and the
   step change must be the **same** across all keys (all keys go `/0/*`→`/1/*` together — Core never emits a
   partially-split multisig). Mismatched per-key steps ⇒ do NOT merge.

Ordering of the emitted `<a;b>`: **by the `internal` flag** — the `internal:false` (receive) side's step is
the **first** alternative, the `internal:true` (change) side's step is the **second**. Use the **actual**
integer step values (never hardcoded `0`/`1`).

### §4.3 Emitted merged descriptor

Construct the merged body by replacing each key's fixed final step `/<step>/*` with `/<recvStep;chgStep>/*`
(preserving script, origins, threshold, key order, network prefix), then **recompute the BIP-380 checksum**
(§6) and re-attach `#<csum>`. The merged JSON entry carries: `desc` = merged string;
`active` = `A.active || B.active` (active on either chain ⇒ active); `internal` = **absent/None**
(represents both — see §5); `range` = the union/identical range (both Core entries carry the same range in
practice; if they differ, take the widening union and record nothing lossy — R0 to confirm range handling);
`dropped_fields` union.

### §4.4 Determinism

The pre-pass MUST be order-stable: pair detection independent of input entry order; the merged entry takes
the position of the **first** (lowest-index) member of the pair; unpaired entries keep relative order. No
`Date`/random.

## §5 — Metadata & wire-shape

- `CoreSourceMetadata.internal: bool → Option<bool>` (`mod.rs:355`). `parse_entry` sets `Some(bool)` for a
  normal single entry; the merge pre-pass path yields `None` for a merged entry. (Because merge happens at
  the JSON layer, `parse_entry` needs a way to know the entry was merged — carry a sentinel: e.g. the merged
  JSON entry omits `internal` entirely, and `parse_bool_field`-absent already yields a default; BUT absent
  currently means `false`, which collides. **Resolution:** merge pre-pass sets an explicit marker the
  parser reads — cleanest is to keep merge at JSON level but have `parse_entry` detect a multipath use-site
  (`<…>`) → `internal = None`; a single fixed/`/*` step → `Some(active-derived)`. R0 decision point:
  detect-`None`-from-multipath vs thread an explicit flag. Either way the invariant is: **multipath entry ⇒
  `internal = None`; single-path entry ⇒ `Some(bool)`**.)
- `apply_select_descriptor` (`mod.rs:394`): update `ActiveReceive`/`ActiveChange` arms so a `None` (merged)
  entry with `active == true` satisfies **both** predicates. New semantics:
  - `ActiveReceive` matches `active && internal != Some(true)` (i.e. `Some(false)` OR `None`).
  - `ActiveChange` matches `active && internal != Some(false)` (i.e. `Some(true)` OR `None`).
  - A merged entry is emitted **once** under either filter (guard against double-emit — it is a single
    `ParsedImport`).
- `--json` (`import_wallet.rs:1859`): `"internal": meta.internal` serializes `Option<bool>` → `true`/`false`/
  `null`. **Wire-shape change** (`internal` may now be `null`) → paired GUI PR (§9).
- text-summary (`import_wallet.rs:2265`): print `both` (or `merged`) when `internal == None`, else
  `true`/`false`. (Bikeshed the token at R0; `both` reads clearest.)

## §6 — BIP-380 checksum recompute

Add a `recompute_descriptor_checksum(body) -> Result<String, ToolkitError>` for bitcoin-core (mirror
`descriptor.rs:246` / `electrum.rs:1027`: `miniscript::descriptor::checksum::Engine::new()`, feed body,
`format!("{body}#{csum}")`). Applied to the synthesized merged body BEFORE it re-enters `parse_entry`
(whose `verify_checksum` at `:257` will then validate it). **R0 note:** prefer factoring the three copies
into one shared `wallet_import` helper IF R0 deems the extra blast-radius acceptable; otherwise a third
local copy matches the established per-parser idiom. Default: **third local copy** (lowest risk for a
funds-critical cycle).

## §7 — Unmergeable / reject behavior

After the pre-pass, any remaining entry with a fixed single final step still reaches the §1 floor reject
(exit 2) via `concrete_keys_to_placeholders`→`lex_placeholders`. This is unchanged and funds-safe.

**Differentiated near-miss message (user decision):** when the pre-pass detects a receive/change-**shaped**
pair (conditions 3-5 hold: both fixed-step, steps differ, internal flags disagree) but conditions 1-2 fail
(script/threshold or key material differ), it MUST surface a bitcoin-core-scoped explanation of WHY it did
not merge — e.g.:

> `import-wallet: bitcoin-core: parse error: descriptors[i]/[j] look like a receive/change pair but their
> keys/origins differ — not merged (distinct keys are different wallets); a fixed single step /0/* is
> un-representable. If these ARE one wallet, combine them by hand to /<0;1>/* and import with
> --format descriptor.`

(exit 2). A lone fixed-step entry with no receive/change-shaped partner falls through to the generic §1 floor
reject unchanged. R0 to finalize exact wording + which exit path carries it (pre-pass emits directly vs sets
context consumed at the floor).

## §8 — Test / oracle matrix (funds-critical; TDD-first)

All in `tests/cli_import_wallet_bitcoin_core.rs` unless noted. **Bold = the funds-critical oracles R0 must
confirm are non-tautological.**

1. **ADD `core_receive_change_distinct_keys_must_not_merge`** — the missing negative control: `/0/*` under
   `MAINNET_FP_A` + `/1/*` under `MAINNET_FP_B` (distinct keys). MUST NOT merge → both hit the fixed-step
   reject → exit 2 with the §7 differentiated distinct-key message. **This is the linchpin funds oracle.**
2. **FLIP `core_fixture_file_mainnet_receive_change_pair_parses` (`:926`)** — same-key file pair: was
   `code==2` reject → now **merge-accept** (exit 0, `bundles=1`, descriptor carries `<0;1>/*`, checksum
   valid).
3. **FLIP `core_receive_change_pair_rejected_with_workaround` (`:1108`)** — inline-blob twin: same flip.
4. **ADD `core_merged_pair_verify_bundle_roundtrip`** — the anti-C1 oracle: import the same-key pair →
   merged `<0;1>/*` → bundle → **`verify-bundle` PASSES on the merged output AND the emitted card derives
   the SAME addresses as a hand-authored `<0;1>/*` import** (guards against a wrong-merge silently
   false-passing, the Cycle A C1 class). Cross-check addresses against a known vector.
5. **ADD `core_merged_pair_select_receive_and_change_both_match`** — `--select-descriptor active-receive`
   AND `active-change` each return the one merged bundle (once each; no double-emit).
6. **ADD `core_lone_receive_fixed_step_still_rejects`** — a single `/0/*` entry (no partner) → generic
   fixed-step reject exit 2 (floor unchanged).
7. **ADD `core_pair_same_step_does_not_merge`** — two `/0/*` entries (identical step) → not a pair → reject.
8. **ADD `core_pair_both_internal_false_does_not_merge`** — internal flags agree → not a pair → reject.
9. **ADD `core_three_entries_sharing_key_ambiguous_no_merge`** — 3 same-key entries → NOTICE + no merge →
   reject.
10. **ADD `core_multisig_receive_change_pair_merges`** — `wsh(sortedmulti(2,...))` `/0/*`-all + `/1/*`-all
    → single merged multisig `<0;1>/*` entry; addresses verified.
11. **ADD `core_multisig_partial_split_does_not_merge`** — one key `/0/*`, another key `/1/*` within an
    entry mismatch (per-key non-uniform) → no merge → reject.
12. **ADD `core_merged_json_internal_null`** — `--json` merged entry emits `source_metadata.internal:
    null`; text-summary prints `both`.
13. **KEEP green `core_fixture_file_multipath_receive_change_pair_parses` (`:952`)** — distinct already-
    multipath entries still `bundles=2` (not conflated by the pre-pass).
14. **ADD `core_nonstandard_steps_merge_uses_actual_values`** — a `/5/*`+`/6/*` same-key pair (if
    plausible) merges to `<5;6>/*` (actual values, not hardcoded 0/1). If Core never emits non-0/1, keep as
    a synthetic guard on the "actual step values" invariant.
15. Zeroize/hygiene: no new secret handling (public xpubs only); confirm no regression in existing
    `lint_zeroize_discipline`-style gates.

Full `cargo test -p mnemonic-toolkit` MUST be green per-phase (memory: R0 reviews run the FULL package
suite — CLI/parse changes ripple into argv/schema/version lints).

## §9 — Lockstep (full-scope)

1. **`mnemonic-gui` paired PR** — the `import-wallet --json` `source_metadata.internal` is now nullable and
   merged entries collapse two→one. This is the **un-gated wire-shape** class (NOT `schema_mirror`: no clap
   flag/subcommand/dropdown changes, so the flag-name mirror does not fire). Paired PR per the manual
   paired-PR rule; GUI consumer must tolerate `internal: null` and the reduced entry count. Bump GUI MINOR.
   Companion `FOLLOWUPS.md` entries updated in both repos.
2. **Manual prose fix (Continuity #5)** — `docs/manual/src/40-cli-reference/41-mnemonic.md:1402-1404`
   currently claims *"Bitcoin Core 25+ emits `listdescriptors` output with the `<0;1>/*` multipath shape"* —
   **factually wrong** (§2) and contradicted by v0.76.0 behavior. Correct it to: Core emits the split
   `/0/*`+`/1/*` pair, which the toolkit now **auto-recombines** into one `<0;1>/*` bundle on import.
3. **`docs/manual/src/45-foreign-formats.md`** — retire the hand-combine workaround; document the automatic
   merge (+ the distinct-key refusal). Re-grep for other Core-split workaround prose at SPEC-impl time
   (prose churns).
4. **`verify-examples` transcripts** — any manual worked example that imports a Core split export changes
   output (was reject → now merged bundle). Regenerate the affected `.out` goldens with the **real new
   binary** (Cycle A gotcha: prose-only is insufficient). Run `make -C docs/manual lint` with the new
   binary BEFORE tag.
5. **`.examples-build/` corpus** — a version site; `examples.yml` re-runs `gen.sh` on crates/Cargo/
   install.sh changes and FATALs on a version mismatch. Refresh with
   `EXAMPLES_BIN_DIR="$PWD/target/debug" bash .examples-build/gen.sh > .examples-build/Examples.md`. Check
   whether any example exercises Core split import (regen if so).

## §10 — Release ritual / version sites (v0.77.0)

Standard toolkit sites (memory `project_toolkit_release_ritual_version_sites` + Cycle A gotchas): both
`Cargo.toml` + `Cargo.lock` + BOTH READMEs (`<!-- toolkit-version -->`) + `fuzz/Cargo.lock` +
`scripts/install.sh` SELF-pin (line ~32; NOT the frozen md/ms/mk sibling pins) + CHANGELOG (tag-gated) +
re-vendor **iff** a dep bumps (none expected). Ship **direct-FF + tag** (admin bypass of the `examples`
required check expected). **NEVER** `cargo fmt --all` (mlock.rs fmt-exempt). GUI ships **PR + CI-before-tag**.

## §11 — SemVer & rationale

MINOR (`0.77.0`): restores a capability (standard Core import) + changes the `import-wallet --json`
wire-shape (`internal` nullable; entry-count collapse) + flips a previously-erroring input reject→accept.
Pre-1.0 additive-plus-wire-change ⇒ MINOR. No public Rust API break (`CoreSourceMetadata` is `pub(crate)`).

## §12 — Risks / R0 focus areas

1. **Wrong-merge = fund loss** — the guard matrix (§4.2) is the whole safety story. R0 must pressure-test
   each condition and the distinct-key negative control (§8.1) + the verify-bundle-on-merged oracle (§8.4).
2. **`internal = None` provenance signal** — how `parse_entry` learns an entry was merged (detect-from-
   multipath vs explicit flag). R0 decision (§5).
3. **`apply_select_descriptor` double-emit** — ensure a merged entry is not emitted twice under any filter.
4. **Range field on merge** (§4.3) — identical-vs-differing range handling.
5. **Checksum helper placement** (§6) — third copy (default) vs shared factor.
6. **Message wording + exit path** for the §7 differentiated reject.
7. **Multisig uniformity** (§4.2 cond. 7) — partial-split refusal correctness.

---

*R0 gate: this SPEC must converge to 0 Critical / 0 Important via the opus-architect reviewer loop
(persisted verbatim to `design/agent-reports/`) BEFORE any implementation, per CLAUDE.md. Implementation is
PAUSED at user request pending SPEC review.*
