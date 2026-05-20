# v0.28.0 cycle in-progress FOLLOWUPS tracker

**Purpose:** scratchpad for out-of-scope items, deferred decisions, and surface-discovered work that emerges DURING v0.28.0 cycle execution. Per the plan-doc's scope-creep defense, new work is logged HERE (not folded mid-cycle), then triaged into `design/FOLLOWUPS.md` at Phase P14A (cycle close).

**Authoritative scope:** `/home/bcg/.claude/plans/unified-meandering-sundae.md` (R6 GREEN). Any work item NOT in the plan-doc's sub-phase rows is OOS by default.

**Cycle status:** Wave 1 in progress (P1B-v2 file added entry 2026-05-19; P9B prior entry; Wave 0 closed at `71592bc`).

---

## Format

Each entry:

```markdown
### `<short-slug>` — <one-line title>

- **Surfaced:** YYYY-MM-DD during Phase P{N}{X} execution; brief context.
- **Where:** file:line citations (re-grep at write-time per plan-doc verification discipline).
- **What:** what the work would be.
- **Why deferred:** explicit scope-creep-defense reasoning.
- **Triage decision (post-P14A):** open in design/FOLLOWUPS.md / wontfix / fold-into-existing-FOLLOWUP / fold-into-v0.28.0 (rare; requires user lift).
- **Tier:** `v0.28+` / `v0.29+` / etc.
```

---

## Open items (cycle-internal)

### `wallet-import-format-mismatch-matrix-completion` — cross-format mismatch symmetry

- **Surfaced:** 2026-05-19 during Phase P1C-v2 execution (instance A, `v0.28.0/p1-sparrow-v2`); Site 2 wiring discovery. Extended at Phase P2C-v2 (`v0.28.0/p2-specter-v2-bg`) — same gap pattern.
- **Where:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` (each `Some("X")` arm at Site 2: BSMS arm checks only BitcoinCore sniff; BitcoinCore arm checks only BSMS sniff; ColdcardMultisig arm checks BSMS + BitcoinCore; Sparrow arm checks BSMS + BitcoinCore + ColdcardMultisig; Specter arm checks BSMS + BitcoinCore + ColdcardMultisig + Sparrow).
- **What:** v0.26.0 wired the BSMS ↔ BitcoinCore mutual-mismatch pair. v0.28.0 P1C extended Sparrow's mismatch coverage to BSMS + BitcoinCore + ColdcardMultisig; P2C extended Specter's to BSMS + BitcoinCore + ColdcardMultisig + Sparrow. The inverse wires — `--format bsms` mismatching a Sparrow / Specter sniff, `--format bitcoin-core` mismatching a Sparrow / Specter sniff, etc. — are NOT wired (existing arms only check pre-existing sniff axes). Same N×N matrix gap will repeat for each per-parser flip (P3C-P6C, P7C). Recommend: each per-parser P{N}C extends the mismatch matrix symmetrically so EVERY `--format X` arm refuses EVERY other parser's positive sniff.
- **Why deferred:** the inverse mismatch lands in a benign fallthrough (the parser fails the alien blob shape with `ImportWalletParse` exit 2 rather than `ImportWalletFormatMismatch` exit 1) — same user-visible "this doesn't work" message, different exit code + stderr template. Cosmetic + not load-bearing for v0.28.0 cycle correctness; full matrix completion is end-of-cycle FOLLOWUP triage.
- **Triage decision (post-P14A):** open in design/FOLLOWUPS.md.
- **Tier:** v0.28+ (fold incrementally as per-parser P{N}C lands).

### `sparrow-taproot-descriptor-passthrough-import-support` — Sparrow taproot import support

- **Surfaced:** 2026-05-19 during Phase P1B-v2 execution (instance A, `v0.28.0/p1-sparrow-v2`); SPEC §11.1 implementation discovery.
- **Where:** `crates/mnemonic-toolkit/src/wallet_import/sparrow.rs` (parse-step-6 taproot refusal — `script_template.contains("tr(")` short-circuit returning `ImportWalletParse("taproot scripts are not yet supported ...")`); `crates/mnemonic-toolkit/src/wallet_export/sparrow.rs:215-219` (emit-side taproot descriptor-passthrough).
- **What:** Sparrow's emit ships taproot wallets as DESCRIPTOR-PASSTHROUGH (concrete `[fp/path]xpub` keys embedded in `defaultPolicy.miniscript.script` instead of `@N/**` placeholders). The P1B parse path substitutes `@N/**` placeholders and refuses taproot scripts; full taproot import requires a parallel parse path that detects descriptor-passthrough shape via heuristic (e.g., `[fp/path]xpub` substring vs `@N/**`) and consumes the embedded concrete-keys descriptor verbatim via `concrete_keys_to_placeholders`.
- **Why deferred:** P1B is the first per-parser cycle; taproot import is a non-trivial second parse path with its own sniff/refusal matrix. Better to ship singlesig + sortedmulti coverage first and dedicate a follow-on cycle to taproot multisig + descriptor-passthrough support symmetric across all 6 new parsers (Sparrow/Specter/Coldcard/etc.).
- **Triage decision (post-P14A):** open in design/FOLLOWUPS.md as `sparrow-taproot-descriptor-passthrough-import-support`.
- **Tier:** v0.29+.

### `bsms-import-taproot-refusal-parity` — BSMS parser should refuse tr() blobs at parse time

- **Surfaced:** 2026-05-19 during Phase P9B execution (instance G3, `v0.28.0/g3-bsms-fixtures`).
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:217-224` — current parser ACCEPTS
    taproot at parse time, only skipping the first-address-verify WARNING.
  - `crates/mnemonic-toolkit/src/wallet_export/bsms.rs:69-76` — EMIT side refuses taproot
    with `BadInput("--format bsms does not support taproot descriptors; BIP-129 §1
    prerequisites pre-date BIP-386. ...")`. Asymmetric: emit refuses, import accepts.
  - `crates/mnemonic-toolkit/tests/cli_import_wallet_bsms.rs::bsms_2line_tr_nums_current_behavior_no_refusal`
    pins the CURRENT behavior; the cell-name preserves the plan-doc's
    forward-looking intent via the suffix `_current_behavior_no_refusal`.
- **What:** add a `Tr(_)` short-circuit at the top of `BsmsParser::parse` mirroring
  `wallet_export/bsms.rs:69-76`'s emit-side refusal. Refusal text would re-use the same
  substring ("does not support taproot descriptors; BIP-129 §1 prerequisites pre-date BIP-386")
  for parity. Cell would then be renamed `bsms_tr_nums_refused` per plan-doc R1-M2 wording
  and assert exit-2 with `ImportWalletParse` containing the substring.
- **Side-channel finding:** `extract_threshold`'s regex at `bsms.rs:419-421` does NOT match
  `sortedmulti_a(` (the `_a` taproot variant). For `tr(NUMS, sortedmulti_a(2, ...))`, the
  regex returns `Ok(None)` and the CLI summary emits `threshold=none`. A parser that
  refuses tr() at the top eliminates this stay-behind hazard entirely.
- **Why deferred:** P9B's plan-doc scope (`/home/bcg/.claude/plans/unified-meandering-sundae.md:555`)
  is `~0 src + ~250 tests + 4 fixture files`. Modifying the parser to refuse tr() is a
  source-code change with normative-SPEC implications (would require a §10 amendment
  declaring tr() refusal alongside the 4-line shape lock). Out of P9B's authored scope.
  G2 (Phase P8 — `bsms-taproot-emit` refusal scaffold) is the natural cycle-resident
  fold target if user lifts mid-cycle; otherwise file as v0.28+ FOLLOWUP at P14A.
- **Triage decision (post-P14A):** open in design/FOLLOWUPS.md as `bsms-import-taproot-refusal-parity`;
  cross-cite the SPEC §10 amendment + the related v0.27.0 first-address-skip discipline at
  `bsms.rs:217-224`.
- **Tier:** `v0.28+` (low-priority; the emit-side refusal already prevents users from
  generating tr() blobs via the toolkit, so the import-side hole is only triggered by
  externally-coordinated tr() BSMS blobs — currently rare in the wild).

### `green-emitter-multisig-refusal-template-only` — Green's multisig refusal misses descriptor-mode

- **Surfaced:** 2026-05-19 during Phase P11C execution (instance Wave-2,
  `v0.28.0/p11-cross-format-matrix`); cross-format refusal matrix probe.
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/green.rs:30-44` (the
  `if let Some(t) = inputs.template` guard skips refusal entirely when
  `template == None`, which is the case on every `--from-import-json`
  invocation per `cmd/export_wallet.rs:603`).
- **What:** `GreenEmitter::emit` refuses multisig templates (P{2sh,2wsh,..}Multi),
  but the refusal is gated on `inputs.template.is_some()`. In descriptor-mode
  invocations (`--descriptor` or `--from-import-json`), `template` is `None`,
  so the multisig guard never fires — Green emits a multisig wsh()-descriptor
  text comment-block even though Green's actual file-import surface refuses
  multisig wallets at runtime. Refusal should be derived from the canonical
  descriptor's script-type (`script_type_from_descriptor`) when `template` is
  absent — `inputs.script_type: WalletScriptType` already encodes the
  multisig variants and is populated on both paths. Refactor: refuse when
  `inputs.script_type.is_multisig()` regardless of template presence.
- **Why deferred:** the matrix-test fix (filter green out of the
  multisig-refusal matrix and pin the current behavior with a regression
  cell) is scoped to P11C. Patching green's emitter is OOS for P11C
  (Phase 11 is matrix-coverage, not refusal-contract reshuffle); changing
  `GreenEmitter::emit` would affect `cli_export_wallet_green.rs`
  multisig-refusal cells that currently use templated input.
- **Triage decision (post-P14A):** open in design/FOLLOWUPS.md as
  `green-emitter-multisig-refusal-template-only`.
- **Tier:** v0.28+ (refusal-contract correctness; not blocking matrix).
- **Companion:** body update — `wallet-import-format-mismatch-matrix-completion`
  notes the matrix-symmetry gap on the import side; this is the symmetric
  emit-side gap.

### `import-wallet-envelope-schema-version-narrative-drift` — schema_version SPEC vs source drift

- **Surfaced:** 2026-05-19 during Phase P11A execution; helper test
  `p11a_helper_envelope_carries_schema_version_and_source_format` asserted
  schema_version="4" per a misread of `cmd/import_wallet.rs:975` (inner
  BundleJson's own `schema_version: "4"`). The OUTER envelope's
  schema_version is `"1"` per `IMPORT_WALLET_ENVELOPE_SCHEMA_VERSION` at
  `cmd/import_wallet.rs:87`.
- **Where:** `cmd/import_wallet.rs:87` (outer envelope const `"1"`) vs
  `cmd/import_wallet.rs:975` (inner BundleJson literal `"4"`).
- **What:** the dual `schema_version` fields share a name but have
  independent rev numbers (envelope wire-shape vs BundleJson wire-shape).
  Per CLAUDE.md plan-doc verification discipline, this duality is a
  silent footgun for future readers / parser authors. Recommend renaming
  one to disambiguate (`envelope_schema_version` vs
  `bundle_schema_version`) OR adding a doc-comment at both sites
  cross-referencing the other.
- **Why deferred:** rename is wire-shape-breaking; affects
  GUI schema mirror + every downstream JSON consumer. Documentation
  fix is low-risk but OOS for P11 (matrix coverage, not envelope
  redesign).
- **Triage decision (post-P14A):** open in design/FOLLOWUPS.md.
- **Tier:** v0.28+.

---

## Triage queue for Phase P14A

(none yet — populated at cycle close from the open-items list)
