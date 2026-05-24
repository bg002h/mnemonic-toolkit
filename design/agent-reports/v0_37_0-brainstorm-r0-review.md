# R0 ARCHITECT REVIEW — `BRAINSTORM_v0_37_0_from_import_json_template_reemit.md`

**Round:** R0 (pre-implementation gate)
**Date:** 2026-05-24
**Reviewer:** feature-dev:code-reviewer (opus)
**Spec SHA basis:** `36e6bfa`
**Verdict:** RED (1 Critical / 5 Important)

---

Reviewed the spec against source ground truth at the cited SHA `36e6bfa` (local tree confirmed at same SHA, 0 ahead/0 behind). Read `export_wallet.rs` (full), `wallet_export/mod.rs`, `template.rs`, all 10 emitters (`sparrow`/`bip388`/`coldcard`/`jade`/`electrum`/`green`/`specter`/`bitcoin_core`/`bsms`), the `json_envelope.rs` deser mirror, the `envelope_v0_27_0.json` fixture, the existing `cli_export_wallet_from_import_json.rs` test suite (1210 lines), and the chapter-45 manual recipes.

The core design (derive `CliTemplate` from the parsed descriptor, inject only for template-requiring formats, fix the threshold trap) is **sound and correctly reasoned** on the points the prompt asked me to stress-test. The substring-ordering hazard is handled correctly, the passthrough partition is exhaustive and accurate, the taproot wall holds, and guard #2 is correct. But the spec has **one Critical omission** (it will land RED-tested CI: existing pinned-refusal tests invert) and several Important gaps.

Confirmed the complete set of `inputs.template` readers in emitters:
- **sparrow.rs:42, :104** — refuses on None (template-requiring). In set.
- **coldcard.rs:44, :111, :257** — refuses on None (template-requiring). In set.
- **bip388.rs:33** — branches Some/None, different output. NOT in `format_requires_template` set (stays None). Correct.
- **jade.rs:36** — refuses on None (template-requiring). In set.
- **electrum.rs:52, :109, :167** — refuses on None (template-requiring). In set.
- **green.rs** — only references in comments; reads `script_type`, not `template`. NOT in set. Correct.
- **bitcoin_core, bsms, specter** — no `inputs.template` reads at all. Correct.

The partition is sound: `{Sparrow, Coldcard, ColdcardMultisig, Jade, Electrum}` are exactly the formats whose emitters refuse on `template==None`. `threshold_user_supplied` has exactly one reader (sparrow.rs:43). The spec's analysis on guards #1 and #2 is technically correct.

---

## CRITICAL

### C1 — The fix inverts 40+ existing pinned-refusal test cells; the spec's test plan does not update them
`crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json.rs` contains assertions that **pin the current refusal behavior** as a regression guard. This feature deliberately changes that behavior, so these cells will **fail on GREEN** unless rewritten in the same PR:

- **`p11c_refusal_matrix_strict_template_only_dests`** (`:840-874`) — asserts `cell_count == 40` and that all 8 sources × `TEMPLATE_ONLY_DESTS = {coldcard, coldcard-multisig, electrum, jade, sparrow}` (`:592-593`) refuse. After the fix, the singlesig sources (`bitcoin-core`/`coldcard`/`electrum` bip84) → sparrow/coldcard/electrum, and the multisig `wsh(sortedmulti)` sources (`jade`/`sparrow`/`specter`) → sparrow/coldcard-multisig/jade/electrum, will SUCCEED. Most of the 40 cells flip from refuse to success.
- **`p11a_helper_returns_nonzero_exit_on_template_only_dest_refusal`** (`:610-622`) — `bsms` source → `sparrow`. The bsms fixture is `sh(multi(...))` → `P2shMulti` → `Err(BadInput)`, so this *particular* cell still refuses, but with a NEW message ("legacy bare P2SH…"), not the asserted refusal pattern set — and the helper test's intent is now stale.
- **Cell 3 `export_wallet_from_import_json_to_template_only_format_refuses_with_helpful_message`** (`:96-119`) — uses `envelope_v0_27_0.json` (`sh(multi(2,…))`) and asserts sparrow/jade/coldcard/electrum refuse with `"requires --template" || "descriptor passthrough is not supported"`. After the fix these refuse via `template_from_descriptor`'s P2shMulti `Err`, whose message contains NEITHER substring → **assertion fails**.

**Remedy:** §5/§6 MUST add an explicit task: rewrite the inverted cells. Specifically — re-partition `p11c` into (a) genuine still-refusing cells (taproot already covered by `p_slug4`; `sh(multi)`→template-format now a *distinct* P2shMulti refusal) and (b) the now-succeeding round-trips folded into `p11b`-style success assertions; update Cell 3 and the `p11a` refusal helper; and confirm the `REFUSAL_STDERR_PATTERNS` set (`:814-822`) gains the new P2shMulti refusal literal. This is the single largest blast-radius item and the spec is silent on it. Per the project lesson "fix the class, hunt for the second instance", the pinned-behavior tests ARE the second instance of the contract being changed.

---

## IMPORTANT

### I1 — `template_from_descriptor` line citations are systematically stale (off by 3–12 lines) even against the declared SHA
The spec cites the `EmitInputs` construction points, but they do not match `export_wallet.rs` at `36e6bfa`:
- `template: None` is at **`:666`**, spec says `:678` (off by 12).
- `threshold_user_supplied: false` is at **`:671`**, spec says `:674` (off by 3).
- `let threshold = …` is at **`:659`**, spec says `:665` (off by 6).
- `parsed_ms` is at **`:613`**, spec says `:614` (off by 1).
- taproot refusal block is **`:629-639`**, spec says `:629-642`.

(`conflicts_with_all` `:171` and `--account != 0` `:554` are exact.) Per CLAUDE.md "Plan-doc + spec citations are grep-verified at write time", these must be re-grepped and corrected before the plan-doc is locked, or Phase 1 will edit the wrong lines.

### I2 — Round-trip byte/semantic-equality (§5 step 4) is NOT achievable for several pairs because the from-import-json path forces `wallet_name = "imported-descriptor"`
On the direct path, `wallet_name` defaults to `<template-human-name>-<account>` (e.g. `bip84-0`, `wsh-sortedmulti-0`) at `export_wallet.rs:435`. On the from-import-json path it is hardcoded `"imported-descriptor"` (`export_wallet.rs:656`) when `--wallet-name` is absent. This name is **emitted into the output** for the very formats this cycle unblocks:
- sparrow: `name` + every keystore `label` (`sparrow.rs:125,137`).
- coldcard multisig text: `Name:` line (`coldcard.rs:302,353`).
- electrum: `keystore.label` (`electrum.rs:122`) / per-cosigner `label` (`electrum.rs:181`).

So `reemit.out` will NOT byte-equal `direct.out` for sparrow/coldcard/electrum name fields. The spec's §5 step-4 ("semantically equals … else byte-equal") under-specifies this. **Remedy:** either (a) the round-trip test must pass matching `--wallet-name` on BOTH sides and assert byte-equality, or (b) explicitly scope the assertion to descriptor/keystore/script-type fields and exclude the name field (document which fields are compared per format). Decide this in the spec, not at test-write time. (This is also why the existing manual specter recipe at `45:405` passes `--wallet-name` explicitly.)

### I3 — The manual lockstep set is incomplete: the coldcard-multisig prose recipe at `45:577-578` is not enumerated
The spec's §3 lists 5 recipe lines to strip. But chapter-45 carries an additional `--template`/`--threshold` usage in prose at **`45:577-578`**: "`--format coldcard-multisig --template wsh-sortedmulti --threshold 2` is equivalent on v0.28.4+." Since `--from-import-json` + `--template` is a clap conflict, this prose is also stale/broken on this path and must be updated in lockstep. Hunt the whole chapter for `--from-import-json … --template` co-occurrences (and the `coldcard-multisig` alias), not just the 5 indexed recipe heads. Also note the recipe-head line numbers in the spec table (313/481/564/639/752) point at the `mnemonic export-wallet` line; the actual `--template …` token sits on the NEXT line (314/482/565/640/753) — the strip must target both lines of each fenced command.

### I4 — Spec must state behavior for the `coldcard-multisig` format on this path (it is in `format_requires_template` but has a pre-template-derivation guard)
`--format coldcard-multisig` dispatches through a `match inputs.template { Some(multisig…) => emit, _ => Err("requires a multisig --template") }` guard at `export_wallet.rs:713-735` BEFORE `ColdcardEmitter::emit`. With the fix, a `wsh(sortedmulti)` envelope derives `Some(WshSortedMulti)` → passes the guard → emits. Good. But a **singlesig** (`wpkh`) envelope to `--format coldcard-multisig` derives `Some(Bip84)` → hits the `_ => Err` arm → refuses with "requires a multisig --template". The spec's §2.3 lists `ColdcardMultisig` in the template-requiring set but never works through this singlesig-to-multisig-format interaction. Confirm the desired behavior (refuse singlesig→coldcard-multisig with the existing pointer) and add a test cell; otherwise the `p11c` rewrite (C1) will be ambiguous about which coldcard-multisig cells flip.

### I5 — `format_requires_template` must be `#[allow(dead_code)]`-free and reachable from BOTH dispatch sites, or the predicate is silently never exercised on the template path
The spec scopes the predicate to `run_from_import_json` only (correct — the template path already has `Some`). But note `CliExportFormat` is the same enum used by `run()`. The spec's §7-Q1 recommendation (exhaustive `match` with no `_` arm) is the right call and I endorse it — but the spec should also state that the predicate lives in `cmd/export_wallet.rs` (not `wallet_export/mod.rs`) so it can `match` on `CliExportFormat` (which is defined in `cmd/export_wallet.rs:22`, not the `wallet_export` module). As written, §2.3 says "Add `fn format_requires_template(f: CliExportFormat) -> bool`" without naming the module; placing it in `wallet_export/mod.rs` would require importing `CliExportFormat` across the module boundary (currently `cmd::export_wallet` imports FROM `wallet_export`, not vice-versa) — a layering inversion. Pin the location.

---

## MINOR

- **M1 — `template_from_descriptor` should match the existing `script_type_from_descriptor` structure-walk, not a whole-descriptor `contains`.** The existing `Wsh(_)` arm (`mod.rs:229`) does NOT render — only the `Tr` arm does (`mod.rs:237`). For `Wsh`/`Sh(Wsh)`, render the **inner** node OR walk `Terminal::Multi` vs `Terminal::SortedMulti` structurally. A whole-`parsed_ms.to_string()` `contains("sortedmulti(")` is *also* correct here (no taproot reaches this path), but the spec should pick one and be precise. roundtrip.rs:1041/1048/1141 confirm miniscript-13 Display renders `wsh(sortedmulti(2,` and `sh(wsh(sortedmulti(2,`, so the substring approach is verified-sound either way.

- **M2 — §2.2 table row `ShInner::Wpkh` → `Bip49`.** Correct (`mod.rs:220`), just terse. No change required.

- **M3 — bip388 "different output" claim verified.** `bip388.rs:33` `Some` → `@N/**` placeholder render; `None` → `descriptor_to_bip388_wallet_policy` passthrough. Keeping bip388 in passthrough (None) set is correct. The §5 passthrough byte-identity regression test is well-targeted.

- **M4 — §7 open-question answers:** (Q1) Endorse exhaustive `match` no-`_` arm. (Q2) Endorse keeping `conflicts_with_all` as-is — descriptor authoritative; override flag is YAGNI and re-introduces the ambiguity this design dissolves. (Q3) Drop `--threshold` from recipes — genuinely ignored on this path (envelope-derived at `:659`). All three correct.

- **M5 — SemVer MINOR is correct.** No clap flag/value/subcommand add/remove. No GUI `schema_mirror` lockstep. The `40-cli-reference/41-mnemonic.md:669` `--from-import-json` row should get the one-line "auto-derives the template" note (in-scope, correctly identified in §3).

---

## VERDICT

**VERDICT: RED (1 Critical / 5 Important)**

The architecture is sound — the descriptor-driven derivation genuinely dissolves the inverse-ambiguity, the substring ordering is correct, the passthrough partition is exhaustive and accurate, the taproot wall holds, and the single `threshold_user_supplied` reader makes guard #2 watertight. But the spec cannot proceed to code: **C1** (inverted pinned-refusal tests, ~40+ cells) is a guaranteed GREEN-phase failure the plan does not budget for, and **I1–I5** must be folded. Fold these, re-grep all citations against `36e6bfa`, re-dispatch for R1.
