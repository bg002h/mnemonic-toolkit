# SPEC — bitcoin-core-receive-change-pair-merge

**Restore standard Bitcoin Core `listdescriptors` import by recombining Core's split receive/change
descriptors into a single `<0;1>/*` multipath entry at parse time.**

- **Author:** (this session) — single-author design per CLAUDE.md phase-2 convention.
- **Source SHA (all line citations grep-verified against this):** `d9063523` (origin/master at recon time).
- **FOLLOWUP slug:** `bitcoin-core-receive-change-pair-merge` (`design/FOLLOWUPS.md`, filed 2026-07-06, split out of Cycle A / v0.76.0 per plan-R0 I-2).
- **Recon:** `cycle-prep-recon-bitcoin-core-receive-change-pair-merge.md` (2026-07-06, SHA `d9063523`).
- **Target release:** `mnemonic-toolkit-v0.77.0` (MINOR). md/ms/mk codecs **NO-BUMP**. Paired `mnemonic-gui` MINOR (wire-shape consumer).
- **Status:** ✅ **R0-GREEN (0C/0I) at round 3**; rev-5 reconciles §4.1/§4.3 extraction+construction to rust-miniscript per PLAN-R0 I1-plan (the lexer cannot read a fixed step — it is the floor that rejects it). Reviews: `design/agent-reports/cycleB-spec-r0-round-{1,2,3}.md` + `cycleB-plan-r0-round-1.md`. In the IMPLEMENTATION_PLAN R0 loop (must converge to 0C/0I before any implementer dispatch, CLAUDE.md Conventions bullet 1).

---

## §0 — Scope, decisions locked with the user (2026-07-06)

| Fork | Decision |
|------|----------|
| Cycle scope | **Full lockstep**: toolkit merge + `bool→Option<bool>` ripple + tests, PLUS paired `mnemonic-gui` `--json` PR AND manual prose fix, all in this cycle. |
| Merge-guard strictness | **Maximally strict** (§4.2 guard matrix). |
| Merged-entry `--select-descriptor` | A merged `<0;1>` entry (`internal = None`) satisfies **both** `active-receive` AND `active-change`. |
| Unmergeable fixed-step | Still **exit-2 reject** (funds-safe floor unchanged), but a distinct-key receive/change-**shaped** near-miss gets a **differentiated** message (§7). |
| Taproot (`tr`) | **Single-key bip86 `tr` IN scope** (merges like `wpkh`; §8.15). **Script-path `tr`** (internal key + tapscript leaves) is **OUT of scope → floor-reject** — Core's standard `listdescriptors` taproot export is key-path bip86 only (`core-bip86-mainnet.json`); a split script-path `tr` is not a real Core shape (R0-round-2 I-a). |

**Non-goals (YAGNI — explicitly OUT):**
1. `/**` double-star shorthand (`bip389-double-star-shorthand-support`) — separate cycle.
2. Concrete non-ranged xpub implied-wildcard (`concrete-nonranged-xpub-implied-wildcard`) — separate cycle.
3. Merging across >2 entries sharing a key, across accounts, across differing scripts, non-adjacent/non-uniform steps, or **script-path `tr`** (tapscript-leaf) descriptors — these DO NOT merge (they reject); we do not invent a general multipath-combiner.
4. Any codec (md/mk/ms) change — the toolkit already parses `<0;1>/*` (`core_multipath_split_to_receive_change`, `cli_import_wallet_bitcoin_core.rs:369` — but note that test proves ONLY a single-key wpkh `<0;1>/*` parses; the merged **multisig** and **single-key `tr`** shapes are established independently by §8 tests #10 (multisig) / #15 (tr), R0-round-1 M3). This is pure toolkit parse-layer.
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
| `BitcoinCoreParser::parse` | `wallet_import/bitcoin_core.rs:142` | extraction `:158-177`; aggregate-dropped build loop `:185-197`; NOTICE emit `:198-205`; per-entry loop `:207-211`; **merge pre-pass inserts after the NOTICE emit (`:205`), before `:207`** |
| `parse_entry` | `wallet_import/bitcoin_core.rs:216` | per-entry decode; `verify_checksum` at `:257`; builds `CoreSourceMetadata` at `:337`; sets `internal` at `:339`; reads `active`/`internal` via `parse_bool_field` `:326-327` |
| `parse_bool_field` | `wallet_import/bitcoin_core.rs:361` | absent/null → `false`; non-bool → typed error |
| `struct CoreSourceMetadata` | `wallet_import/mod.rs:353-364` | `pub(crate) internal: bool` at `:355` → becomes `Option<bool>` |
| `apply_select_descriptor` | `wallet_import/mod.rs:394` | `ActiveReceive` arm `:411` (`active && !internal`); `ActiveChange` arm `:427` (`active && internal`); doc `:383-386` |
| `sample_core_metadata` (test) | `wallet_import/mod.rs:534` | test builder, ripples |
| `--json` `source_metadata.internal` | `cmd/import_wallet.rs:1859` | `"internal": meta.internal` |
| text-summary `internal` | `cmd/import_wallet.rs:2265` | `bundles[{i}].internal={}` |
| checksum recompute prior art | `descriptor.rs:246`, `electrum.rs:1027`, `coldcard.rs:515`, `specter.rs:444`, `sparrow.rs:668` | **FIVE** private per-parser copies via `miniscript::descriptor::checksum::Engine` (a bitcoin_core copy = the **6th**) |
| fixed-step floor reject | `parse_descriptor.rs:203-213` (residue `:205`, message `:207-210`) | fires inside `concrete_keys_to_placeholders`→`lex_placeholders` |
| positive merge INPUT fixture | `tests/fixtures/wallet_import/core-mainnet-receive-change-pair.json` | same-key `/0/*`+`/1/*` (KEPT v0.76.0) |
| taproot Core fixture | `tests/fixtures/wallet_import/core-bip86-mainnet.json` | `tr([b8688df1/86'/0'/0']xpub…/<0;1>/*)` — proves Core emits `tr` (already-multipath; split variant built in §8) |
| current reject tests (to FLIP) | `tests/cli_import_wallet_bitcoin_core.rs:926` (file), `:1108` (inline) | assert exit-2 today → become merge-accept |
| distinct-multipath negative (KEEP green) | `tests/cli_import_wallet_bitcoin_core.rs:952` | already-`<0;1>` distinct entries; NOT the merge discriminator |

## §4 — The merge pre-pass

### §4.1 Placement, grouping key, and within-group discriminant

A new function `merge_receive_change_pairs(descriptors: Vec<Value>, stderr) -> Result<Vec<Value>, ToolkitError>`
is invoked in `parse` **after** `descriptors`/`wallet_name` extraction and the aggregate-dropped-fields
NOTICE emit (`bitcoin_core.rs:198-205`; see §4.5) and **before** the ParsedImport loop (`:207`). It operates on
the raw JSON `descriptors` array so a merged `<a;b>/*` string flows through the **unchanged** `parse_entry`
pipeline (checksum-validate → placeholder → parse_descriptor), except that the merged entry's `internal`
provenance is threaded explicitly (§5).

For each entry it extracts, from the **checksum-stripped, checksum-VALIDATED** (§4.4) descriptor body, by
**parsing it with rust-miniscript** — `Descriptor::<DescriptorPublicKey>::from_str(body)` (which ACCEPTS a
fixed step, unlike md1's `lex_placeholders`, which is the very v0.76.0 floor that REJECTS it; PLAN-R0 I1-plan)
— reading per-key `origin` (fp + path), the bare xpub, `derivation_path()` (**the use-site final step**),
`wildcard` (the `/*` hardness), the template kind + threshold, and `Tr::tap_tree()` presence (`None` =
key-path bip86 → mergeable single-key; `Some` = script-path → floor-reject per §4.2 cond. 1/7). NEVER an
ad-hoc regex, and NEVER `lex_placeholders`/`concrete_keys_to_placeholders` for the final step (they reject the
input). (`extract_origin_components` MAY still supply the `(fp, origin path, xpub)` tuple, but the final step
comes from the miniscript parse.)

- **Grouping key (R0-round-1 I4 — the resolved definition)** = the tuple **EXCLUDING the final use-site
  step**: `(script template kind, threshold, ORDERED per-key (fingerprint, full origin path, xpub),
  wildcard-hardness marker)`. The comparison is **positional/ordered**, NOT set-based — a swapped-order
  `multi(...)` yields a different grouping key and does not group. `sortedmulti` vs `multi` differ in
  template kind and do not group.
- **Within-group discriminant** = the differing final steps + disagreeing `internal` flags (conditions 3-5
  below). Two entries with the **same** grouping key but differing final-step + disagreeing internal are a
  candidate receive/change pair.

Entries are bucketed by grouping key preserving first-seen order.

### §4.2 Guard matrix — merge IFF ALL hold (maximally strict)

Two entries `A` and `B` sharing a grouping key merge into one `<stepA;stepB>/*` entry **iff every**
condition holds; ANY deviation ⇒ do **not** merge (leave both entries as-is → they reject at the floor per
§7):

1. **Script/threshold identical** — implied by identical grouping key (same template kind + threshold).
   Covers `wpkh`, `wsh(multi/sortedmulti)`, `sh(...)`, nested `sh(wsh(...))`, and single-key `tr` (key-path
   bip86). Script-path `tr` (tapscript leaves) → floor-reject (§4.2 cond. 7; out of scope).
2. **All key material identical** — for every key expression, identical `(fingerprint, full origin path,
   xpub)`, positionally. Never merge across differing keys/origins/accounts. (Funds-critical discriminator:
   distinct keys are DIFFERENT wallets.)
3. **Each side is a fixed single, UNHARDENED, wildcard final use-site step with matching wildcard-hardness**
   (R0-round-1 M4) — both carry exactly one non-multipath, non-hardened fixed integer final step followed by
   an unhardened `/*` (e.g. `/0/*` and `/1/*`). Reject-to-floor (do NOT merge) if either side is already
   multipath (`<…>`), bare `/*`, a hardened final step (`/0'/*`), or a hardened/mismatched wildcard
   (`/*'`).
4. **Final steps differ** — `stepA != stepB`.
5. **`internal` flags disagree** — exactly one `internal:true`, the other `internal:false` (read via
   `parse_bool_field`: absent/null → `false`, non-bool → typed error; R0-round-1 M5). Two `false` (or two
   `true`) do not pair.
6. **Exactly two** entries share the grouping key. Three-or-more ⇒ ambiguous ⇒ do NOT merge (reject); emit a
   NOTICE naming the ambiguity.
7. **Multi-key uniformity (R0-round-1 I3, R0-round-2 I-a)** — for any multi-key descriptor
   (`multi`/`sortedmulti`), conditions 3-4 apply **per key** and the step change must be the **SAME** across
   ALL keys — every key goes `/<recvStep>/*` → `/<chgStep>/*` together (Core never emits a partially-split
   descriptor). Any per-key non-uniform step ⇒ do NOT merge. A **single-key `tr`** (key-path bip86) is
   single-key and merges like `wpkh` (one key, one step). A **script-path `tr`** (internal key +
   tapscript-leaf keys) is OUT of scope → floor-reject (§0; not a real Core split shape).

Ordering of the emitted `<a;b>`: **by the `internal` flag** — the `internal:false` (receive) side's step is
the **first** alternative, the `internal:true` (change) side's step is the **second**. Use the **actual**
integer step values (never hardcoded `0`/`1`).

### §4.3 Emitted merged descriptor (UNIFORMITY-VERIFIED all-keys replacement)

**R0-round-1 I2(b) + PLAN-R0 I1-plan — the funds-critical construction rule.** Construct the merged body by
replacing, for **EVERY** key expression (all cosigners for multisig; the single key-path key for a single-key
bip86 `tr`), that key's fixed final `/<recvStep>/*` use-site with `/<recvStep;chgStep>/*`. **Sanctioned
mechanism** — `lex_placeholders` CANNOT be used (it is the v0.76.0 floor that REJECTS fixed steps) and a
miniscript AST parse loses byte offsets, so there are no lexer "positions" to anchor on: after
miniscript-verifying **condition-7 uniformity** (§4.1 parse confirms EVERY key in the receive-side body
carries the IDENTICAL `/<recvStep>/*`), a **global** replace of the substring `/<recvStep>/*` →
`/<recvStep;chgStep>/*` on the checksum-stripped receive body is correct and unambiguous — the trailing `/*`
distinguishes a use-site step from any origin-path component (which ends in `']`), so no origin digit is ever
touched. A **first-only / partial** rewrite (leaving `k2/0/*` while rewriting `k1/<0;1>/*`) is FORBIDDEN — it
would expand to WRONG change addresses and verify-bundle would false-pass (the Cycle A C1 class). The
uniformity precondition makes the global replace all-keys-uniform by construction; the §8.4/§8.10
original-anchored oracle is the backstop.

Then **recompute the BIP-380 checksum** (§6) and re-attach `#<csum>`. The merged JSON entry carries:

- `desc` = merged `<recvStep;chgStep>/*` string with fresh checksum.
- `active` = `A.active || B.active` (active on either chain ⇒ active).
- `internal` provenance = **merged** → surfaces as `None` (§5). Threaded via an explicit flag, NOT inferred
  from the multipath shape (R0-round-1 I1).
- `range` (**R0-round-1 I5 — LOCKED**): merged `range` = the **union / widening** of the two entries'
  ranges. **A range difference NEVER blocks the merge** — receive and change chains legitimately carry
  different per-chain scan state (`range`/next-index). If both absent → absent.
- `dropped_fields`: union of both entries' dropped fields (see §4.5 / M6).

### §4.4 Input validation, determinism, and Vec invariant

- **Validate each candidate's own BIP-380 checksum BEFORE consuming it** (R0-round-1 M9 — fail-closed
  hygiene): strip `#<csum>` only after `miniscript::descriptor::checksum::verify_checksum` accepts each input
  entry, so a body-correct/checksum-corrupt entry is refused (not silently "repaired" by the recompute). A
  bad input checksum → the entry is not eligible to merge → floor/ordinary parse surfaces the error.
- **Vec invariant (R0-round-1 M8):** for each merged pair, the pre-pass REMOVES **both** paired entries and
  INSERTS **exactly one** merged entry at the **first** (lowest-index) member's position; unpaired entries
  keep relative order. Holds for the N-pair case (a blob with several independent same-wallet pairs). No
  entry is ever emitted twice.
- **Determinism:** pair detection is independent of input entry order; no `Date`/random. Same input →
  byte-identical merged output.

### §4.5 Aggregate dropped-fields NOTICE ordering (R0-round-1 M6)

The `timestamp`/`next`/`next_index` aggregate-dropped-fields NOTICE loop (`bitcoin_core.rs:185-197`) MUST
run on the **ORIGINAL** `descriptors` array (before the merge pre-pass), OR each merged entry must carry the
UNION of its two members' dropped-field signals — else the NOTICE under-reports fields that were on the
original pair. Default: run the NOTICE loop on the original array (it already precedes `:207`); the pre-pass
runs after it and preserves the union in each merged entry's `dropped_fields` for the per-entry provenance.

## §5 — Metadata & wire-shape

- `CoreSourceMetadata.internal: bool → Option<bool>` (`mod.rs:355`).
- **Provenance mechanism (R0-round-1 I1 — RESOLVED; shape-based detection FORBIDDEN):** the merge pre-pass
  threads an **explicit** merge flag; `parse_entry` gains an `internal: Option<bool>` parameter it uses
  instead of reading `internal` from the JSON itself. A **passthrough single entry** → `Some(parse_bool_field(...))`;
  a **pre-pass-merged entry** → `None`. Invariant: **"pre-pass-merged ⇒ None," NOT "multipath ⇒ None"** — a
  pre-existing already-`<0;1>` Core entry (fixture `:952`) keeps its explicit `Some(false)`/`Some(true)` and
  its `--select-descriptor` semantics are UNCHANGED.
- `apply_select_descriptor` (`mod.rs:394`) — new predicates so a `None` (merged) entry with `active == true`
  satisfies **both** filters, emitted **once** each:
  - `ActiveReceive`: `active && internal != Some(true)` (i.e. `Some(false)` OR `None`).
  - `ActiveChange`: `active && internal != Some(false)` (i.e. `Some(true)` OR `None`).
- `--json` (`import_wallet.rs:1859`): `"internal": meta.internal` serializes `Option<bool>` → `true`/`false`/
  `null`. **Wire-shape change** (`internal` may now be `null`) → paired GUI PR (§9).
- text-summary (`import_wallet.rs:2265`): print `both` when `internal == None`, else `true`/`false` (token
  LOCKED = `both`; R0-round-1 M7).

## §6 — BIP-380 checksum recompute

Add a `recompute_descriptor_checksum(body) -> Result<String, ToolkitError>` local to `bitcoin_core.rs`
(mirror the FIVE existing copies — `descriptor.rs:246`, `electrum.rs:1027`, `coldcard.rs:515`,
`specter.rs:444`, `sparrow.rs:668` — via `miniscript::descriptor::checksum::Engine::new()`, feed body,
`format!("{body}#{csum}")`). Applied to the synthesized merged body; `parse_entry`'s `verify_checksum` at
`:257` re-validates it. **Decision LOCKED = 6th local copy** (lowest risk for a funds-critical cycle; the
5→1 shared-factor refactor is a broader blast radius and is NOT taken here; R0-round-1 M1).

## §7 — Unmergeable / reject behavior (LOCKED)

After the pre-pass, any remaining entry with a fixed single final step still reaches the §1 floor reject
(exit 2) via `concrete_keys_to_placeholders`→`lex_placeholders`. Unchanged and funds-safe.

**Differentiated near-miss message (R0-round-1 M7 — LOCKED default):** the pre-pass, upon detecting a
receive/change-**shaped** pair (conditions 3-5 hold: both fixed-step, steps differ, internal flags disagree)
whose conditions 1-2 FAIL (script/threshold or key material differ), emits its OWN bitcoin-core-scoped error
**directly** (exit 2), e.g.:

> `import-wallet: bitcoin-core: parse error: descriptors[i]/[j] look like a receive/change pair but their
> keys/origins differ — not merged (distinct keys are different wallets); a fixed single step /0/* is
> un-representable. If these ARE one wallet, combine them by hand to /<0;1>/* and import with
> --format descriptor.`

A lone fixed-step entry with no receive/change-shaped partner falls through to the generic §1 floor reject
unchanged.

## §8 — Test / oracle matrix (funds-critical; TDD-first)

All in `tests/cli_import_wallet_bitcoin_core.rs` unless noted. **Bold = the funds-critical oracles R0 must
confirm are non-tautological.**

1. **ADD `core_receive_change_distinct_keys_must_not_merge`** — the missing negative control: `/0/*` under
   `MAINNET_FP_A` + `/1/*` under `MAINNET_FP_B` (distinct keys). MUST NOT merge → exit 2 with the §7
   differentiated distinct-key message. **Linchpin funds oracle.**
2. **FLIP `core_fixture_file_mainnet_receive_change_pair_parses` (`:926`)** — same-key file pair: was
   `code==2` reject → now **merge-accept** (exit 0, `bundles=1`, descriptor carries `<0;1>/*`, checksum
   valid).
3. **FLIP `core_receive_change_pair_rejected_with_workaround` (`:1108`)** — inline-blob twin: same flip.
4. **ADD `core_merged_pair_addresses_match_original_split` (R0-round-1 I2 — the anti-C1 oracle, NON-tautological)**
   — import the same-key `/0/*`+`/1/*` pair → merged `<0;1>/*` bundle. Independently derive addresses
   **from the ORIGINAL split descriptors** (external addrs at indices 0..k from `.../0/*`; internal addrs
   from `.../1/*`) via rust-miniscript `at_derivation_index().address()` (or a pinned known vector), and
   assert they equal the merged bundle's chain-0 / chain-1 addresses **for BOTH chains**. This anchors on
   the pre-merge truth, NOT on a hand-authored `<0;1>` (which is the same construction the merge produces).
   Plus: `verify-bundle` PASSES on the merged output.
5. **ADD `core_merged_pair_select_receive_and_change_both_match`** — `--select-descriptor active-receive`
   AND `active-change` each return the one merged bundle (once each; no double-emit).
6. **ADD `core_lone_receive_fixed_step_still_rejects`** — a single `/0/*` entry (no partner) → generic
   fixed-step reject exit 2 (floor unchanged).
7. **ADD `core_pair_same_step_does_not_merge`** — two `/0/*` entries (identical step) → not a pair → reject.
8. **ADD `core_pair_both_internal_false_does_not_merge`** — internal flags agree → not a pair → reject.
9. **ADD `core_three_entries_sharing_key_ambiguous_no_merge`** — 3 same-key entries → NOTICE + no merge →
   reject.
10. **ADD `core_multisig_receive_change_pair_merges` (with the §8.4 original-anchored oracle)** — `wsh(sortedmulti(2,...))`
    all-keys `/0/*` + all-keys `/1/*` → single merged multisig `<0;1>/*`; addresses independently derived
    from the original split multisig descriptors for both chains and asserted equal (guards misfired
    per-key replacement). verify-bundle PASSES.
11. **ADD `core_multisig_partial_split_does_not_merge`** — within a candidate, one key `/0/*` and another
    key `/1/*` (per-key non-uniform, cond. 7) → no merge → reject.
12. **ADD `core_merged_json_internal_null`** — `--json` merged entry emits `source_metadata.internal: null`;
    text-summary prints `both`.
13. **STRENGTHEN `core_fixture_file_multipath_receive_change_pair_parses` (`:952`) (R0-round-1 I1)** — keep
    `bundles=2`, AND assert `--select-descriptor active-receive` returns exactly the ONE `internal:false`
    entry and `active-change` exactly the ONE `internal:true` entry (proves shape-based None-detection was
    NOT used).
14. **ADD `core_nonstandard_steps_merge_uses_actual_values`** — a `/5/*`+`/6/*` same-key pair merges to
    `<5;6>/*` (actual values, not hardcoded 0/1).
15. **ADD (taproot, R0-round-1 I3, R0-round-2 I-a):**
    - **`core_tr_bip86_receive_change_pair_merges`** (mandatory) — split `tr(key/0/*)` + `tr(key/1/*)` →
      merged `tr(key/<0;1>/*)`; addresses (P2TR) independently derived from the two originals for both
      chains and asserted equal (the §8.4 oracle on a `tr` key-path).
    - **`core_tr_scriptpath_pair_does_not_merge`** — a script-path `tr` (internal key + tapscript leaf keys,
      e.g. `tr(NUMS, sortedmulti_a(...))`) split pair is OUT of scope: the guard does NOT merge it; it falls
      to the floor reject (exit 2). LOCKED behavior (§0 / §4.2 cond. 7), NOT contingent on fixture
      feasibility.
16. **ADD `core_hardened_final_step_does_not_merge`** — `/0'/*` (or `/0/*'`) shaped pair → cond. 3 excludes
    → no merge → reject (R0-round-1 M4).
17. **ADD `core_corrupt_input_checksum_not_merged`** — a mergeable-shaped pair where one entry has a
    corrupt `#<csum>` → refused before merge (fail-closed, R0-round-1 M9), not silently repaired.
18. Zeroize/hygiene: no new secret handling (public xpubs only); confirm no regression in existing
    `lint_zeroize_discipline`-style gates.

Full `cargo test -p mnemonic-toolkit` MUST be green per-phase (memory
`feedback_r0_review_run_full_package_suite`: CLI/parse changes ripple into argv/schema/version lints).

## §9 — Lockstep (full-scope)

1. **`mnemonic-gui` paired PR** — the `import-wallet --json` `source_metadata.internal` is now nullable and
   merged entries collapse two→one. This is the **un-gated wire-shape** class (NOT `schema_mirror`: no clap
   flag/subcommand/dropdown changes, so the flag-name mirror does not fire). Paired PR per the manual
   paired-PR rule; GUI consumer must tolerate `internal: null` and the reduced entry count. Bump GUI MINOR.
   Companion `FOLLOWUPS.md` entries updated in both repos.
2. **Manual prose fix (Continuity #5; R0-round-1 M2)** — `docs/manual/src/40-cli-reference/41-mnemonic.md`:
   the sentence at **`:1404-1405`** ("Bitcoin Core 25+ emits `listdescriptors` output with the `<0;1>/*`
   multipath shape on the canonical receive/change pair") is **factually wrong** (§2). The FOLLOWING sentence
   ("Importing this directly yields one bundle per descriptor entry (use `--select-descriptor
   active-receive`…)") is ALSO now wrong (split import yields ONE merged bundle). **Rewrite the whole worked-
   example block (~`:1401-1410`)**: Core emits the split `/0/*`+`/1/*` pair, which the toolkit now
   **auto-recombines** into one `<0;1>/*` bundle on import (+ note the distinct-key refusal). Re-grep for the
   exact line numbers at impl time (prose churns).
3. **`docs/manual/src/45-foreign-formats.md`** — retire the hand-combine workaround; document the automatic
   merge. Re-grep for other Core-split workaround prose at impl time.
4. **`verify-examples` transcripts** — any manual worked example importing a Core split export changes output
   (was reject → now merged bundle). Regenerate the affected `.out` goldens with the **real new binary**
   (Cycle A gotcha: prose-only is insufficient). Run `make -C docs/manual lint` with the new binary BEFORE
   tag.
5. **`.examples-build/` corpus** — a version site; `examples.yml` re-runs `gen.sh` on crates/Cargo/
   install.sh changes and FATALs on a version mismatch. Refresh with
   `EXAMPLES_BIN_DIR="$PWD/target/debug" bash .examples-build/gen.sh > .examples-build/Examples.md`. Regen if
   any example exercises Core split import.

## §10 — Release ritual / version sites (v0.77.0)

Standard toolkit sites (memory `project_toolkit_release_ritual_version_sites` + Cycle A gotchas): both
`Cargo.toml` + `Cargo.lock` + BOTH READMEs (`<!-- toolkit-version -->`) + `fuzz/Cargo.lock` +
`scripts/install.sh` SELF-pin (line ~32; NOT the frozen md/ms/mk sibling pins) + CHANGELOG (tag-gated) +
re-vendor **iff** a dep bumps (none expected). Ship **direct-FF + tag** (admin bypass of the `examples`
required check expected). **NEVER** `cargo fmt --all` (mlock.rs fmt-exempt); per-package `cargo fmt -p` only.
GUI ships **PR + CI-before-tag**.

## §11 — SemVer & rationale

MINOR (`0.77.0`): restores a capability (standard Core import) + changes the `import-wallet --json`
wire-shape (`internal` nullable; entry-count collapse) + flips a previously-erroring input reject→accept.
Pre-1.0 additive-plus-wire-change ⇒ MINOR. No public Rust API break (`CoreSourceMetadata` is `pub(crate)`).

## §12 — Risks / R0 focus areas (no open design decisions remain)

All prior "R0 to decide" items are now LOCKED (§4.3 range, §5 mechanism, §6 checksum, §7 message, §8.15
taproot scope). **No open design decisions remain.** Residual verification focus for R0:

1. **Wrong-merge = fund loss** — re-verify the §4.2 guard matrix + the §8.1 distinct-key negative control +
   the §8.4/§8.10/§8.15 **original-anchored** address oracles are genuinely non-tautological (the I2 fix).
2. **All-keys structural replacement** (§4.3) — confirm the construction cannot rewrite a partial key set
   (the §8.11 partial-split-no-merge + §8.10 multisig oracle backstop this).
3. **`internal = None` provenance** — confirm the explicit-flag threading (§5) with no shape-based inference
   and the §8.13 strengthened multipath-fixture select assertions.
4. **Taproot scope** (§4.2 cond. 7, §8.15) — LOCKED: single-key bip86 `tr` merge mandatory; script-path
   `tr` OUT of scope → floor-reject (R0-round-2 I-a). No open decision.
5. **`apply_select_descriptor` double-emit** — a merged entry emitted once under either filter (§8.5).

---

*R0 gate: this SPEC must converge to 0 Critical / 0 Important via the opus-architect reviewer loop
(persisted verbatim to `design/agent-reports/`) BEFORE any implementation, per CLAUDE.md. Implementation is
PAUSED at user request pending SPEC convergence + user review.*
