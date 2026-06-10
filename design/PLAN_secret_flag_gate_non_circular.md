# PLAN — non-circular secret-flag completeness gate + classify `--phrase`/`--phrase-stdin`/`--ms1-stdin` (audit I3)

**Cycle:** v0.53.1 (PATCH — see SemVer call below) · **Source SHA:** `e2a09ba` (= v0.53.0) · **Recon:** `cycle-prep-recon-vacuous-secret-flag-gate.md`
**Resolves:** `design/FOLLOWUPS.md` backlog lines `vacuous-secret-flag-gate` **[IMPORTANT]** (audit I3) + `flag-is-secret-completeness-unguarded-by-design` **[obs]** (the audit report's proposed name `secret-flag-completeness-gate-non-circular` was never filed; the registry ids above are canonical).
**Unblocks:** the `schema/mnemonic.rs` half of audit I4 (GUI cleartext master-phrase render — GUI mirrors the toolkit `secret` bit); I4's `schema/ms.rs` half is ms-cli-surface, out of this cycle's reach.

## Problem (verified at e2a09ba)

1. **The completeness gate is a tautology.** `tests/cli_gui_schema_v5_extensions.rs:284` (`secret_flag_enumeration_matches_authoritative_predicate`, assert :299-305) compares the schema's `secret` bit against `secrets::flag_is_secret(name)` — but `cmd/gui_schema.rs:1196` *derives* the bit from exactly that predicate. Both sides move together; the cell can only catch emitter-plumbing regressions, never an allowlist omission. `lint_argv_secret_flags` axis-1 is transitive on the same predicate and says so (`tests/lint_argv_secret_flags.rs:31-36`: "There is no separate gate over `flag_is_secret` completeness itself").
2. **The gap is live, not hypothetical.** Heuristic sweep of the v0.53.0 `gui-schema` surface (recon §LIVE GAP) finds exactly 3 misclassified names:
   - `--phrase` (kind=text) ×3 xpub-search modes — **raw master BIP-39 phrase**, argv-inline (`src/cmd/xpub_search/{path_of_xpub.rs:38, passphrase_of_xpub.rs:56, account_of_descriptor.rs:39}`). `secret:false` → GUI renders cleartext, no paste-warn/run-confirm/zeroize (= audit I4's toolkit-side root).
   - `--phrase-stdin` (boolean) ×3 — violates the `*-stdin`-toggle-is-secret convention (cf. `--passphrase-stdin` et al., all `secret:true`; rationale `src/secrets.rs:25-29`).
   - `--ms1-stdin` (boolean) ×3 — `--ms1` is secret but its stdin toggle is not.
   No existing test pins any of the 3 as non-secret (swept `tests/` + `src/secrets.rs` unit tests — I10 lesson). Runtime already treats `--phrase` as secret-equivalent (`@env:` sentinel + stdin route, `tests/cli_xpub_search_env_var_sentinel.rs`).
3. `flag_is_secret` has exactly ONE non-test consumer (`gui_schema.rs:1196`), so the flip's blast radius = gui-schema JSON bits + lint axis-1 derived set + GUI mirrors at next pin bump. No runtime stderr/exit behavior changes.

## Design

### D1 — classification fix (src/secrets.rs)

Add `--ms1-stdin`, `--phrase`, `--phrase-stdin` to the `matches!` (keep the list's near-alphabetical placement: `--ms1-stdin` after `--ms1`; `--phrase`/`--phrase-stdin` between them and `--secret`). Add "Membership rationale" doc entries (`--phrase` = raw BIP-39 master phrase on xpub-search; the two `-stdin` toggles per the existing sentinel-toggle rationale). Extend the in-module unit tests (`known_secret_flags_classify_as_secret` gains the 3 names; the existing no-leading-dashes row `secrets.rs:120` is kept as-is — R0-r1 M-4).

### D2 — three new NON-CIRCULAR gate cells (tests/cli_gui_schema_v5_extensions.rs, new §7b)

All walk the **live** `gui-schema` JSON (existing `run_gui_schema()` helper) and share test-local constants — **never calling `flag_is_secret`** (that's what makes them non-circular):

- **Cell 1 `heuristic_secret_name_net`:** for every flag whose `kind ∉ {path, number, boolean}` (i.e. text/dropdown/future kinds), if the name matches
  `phrase|secret|password|share|seed|mnemonic|wif|xprv|entropy|digits|ms1|priv`
  then `secret` must be `true` — unless `(subcommand, flag)` is in a literal `EXEMPT` table (empty today; each future entry must carry a rationale comment). `phrase` subsumes `passphrase`; `priv` deliberately included but currently speculative vocabulary — it matches nothing in the included kinds today (`--privacy-preserving` is boolean-excluded); note this in a cell comment (R0-r1 M-3). `key` deliberately excluded (`--pubkey` noise). Kinds are excluded **by kind, not by name**: path = file reference, number = count (`--shares`), booleans handled by Cell 2.
- **Cell 2 `stdin_toggle_secrecy_matches_base_flag`:** for every boolean flag named `--X-stdin`, its emitted `secret` must equal the emitted `secret` of `--X` **in the same subcommand**; assert the base flag exists. Live census (R0-r1 I-2): **8 distinct `--X-stdin` names / 25 instances** (R0-r2 M-5) — `--passphrase-stdin`, `--bip38-passphrase-stdin`, `--decrypt-password-stdin`, `--secret-stdin`, `--ms1-stdin`, `--phrase-stdin`, **plus the non-secret pairs `--message-stdin` (verify-message, base `--message`) and `--xpub-stdin` (xpub-search-address-of-xpub, base `--xpub`)** — all 8 have a same-subcommand base, so the existence assert holds today; the two non-secret pairs are in the cell's domain by design (false==false passes). A future orphan toggle fails loudly and is triaged explicitly. The failure message must print the `(subcommand, toggle, base)` triple.
- **Cell 3 `secret_flag_name_set_matches_frozen_literal`:** the distinct set of flag names carrying `secret:true` anywhere in the schema must SET-EQUAL a frozen test-local literal (after D1: the 11 current + 3 new = **14** names). Catches silent removals AND additions — any change to the predicate forces a conscious test edit (the audit's option (b), as belt-and-braces over the heuristic).

Keep the existing :284 cell **as-is functionally** but rewrite its doc-comment to its honest scope ("emitter-plumbing consistency: schema bit == predicate; completeness is gated by §7b") and rename to `secret_bit_plumbing_matches_predicate`.

**TDD order (RED matrix corrected per R0-r1 I-1):** Cells land first; expected pre-D1 RED is — **Cell 1: RED on `--phrase` ×3** (xpub-search modes); **Cell 2: RED on `--ms1-stdin` ×3 ONLY** (`--phrase-stdin` is GREEN pre-D1: toggle false == base `--phrase` false — equal, passes); **Cell 3: RED on the 3-name set difference** (live 11 vs frozen 14). `--phrase-stdin`'s pre-D1 coverage comes ONLY from Cell 3; Cell 2 protects it transitively once D1 flips `--phrase`. Do NOT distort Cell 2 to force it red on `--phrase-stdin`. D1 turns all three GREEN.

### D3 — lint_argv_secret_flags lockstep (tests/lint_argv_secret_flags.rs)

`--phrase` becoming `secret && kind==text` grows the axis-1 derived set → set-equality cell REDs until 3 new `Route` rows are declared:

```
Route { subcommand: "xpub-search-path-of-xpub",        flag: "--phrase", source_file: "src/cmd/xpub_search/path_of_xpub.rs",        evidence: &["pub phrase_stdin", "fn phrase_stdin"] },
Route { subcommand: "xpub-search-passphrase-of-xpub",  flag: "--phrase", source_file: "src/cmd/xpub_search/passphrase_of_xpub.rs",  evidence: &["pub phrase_stdin", "fn phrase_stdin"] },
Route { subcommand: "xpub-search-account-of-descriptor",flag: "--phrase", source_file: "src/cmd/xpub_search/account_of_descriptor.rs",evidence: &["pub phrase_stdin", "fn phrase_stdin"] },
```

**Evidence needles must be DISCRIMINATING (R0-r1 I-3):** the lint check is plain `source.contains` (`lint_argv_secret_flags.rs:240`) and bare `"phrase_stdin"`/`"phrase-stdin"` are suffixes of `"passphrase_stdin"`/`"passphrase-stdin"`, which already occur in all 3 files for the existing `--passphrase` routes — bare needles would be satisfied even with the `--phrase-stdin` wiring deleted. `"pub phrase_stdin"` / `"fn phrase_stdin"` are not substrings of any `passphrase` token and are verified present (`path_of_xpub.rs:45,124`, `passphrase_of_xpub.rs:63,155`, `account_of_descriptor.rs:46,136`). `--phrase-stdin`/`--ms1-stdin` become `secret && boolean` = evidence toggles, not routes — no rows. Update the **boundary prose** `:31-36`: the "no separate gate over `flag_is_secret` completeness" sentence becomes false — rewrite to cite the §7b gate and state the residual boundary honestly (a future secret flag with *novel vocabulary* outside the name-net AND not `--from`/`--slot`-routed still escapes; the net narrows the design-accepted hole, doesn't eliminate it).

### D4 — docs + ritual

- **FOLLOWUPS.md:** promote `vacuous-secret-flag-gate` to its own `###` entry (RESOLVED v0.53.1, fold the [obs] line into it; alias note for the report's `secret-flag-completeness-gate-non-circular`), strike/annotate both backlog index lines, update the index `Status:` line.
- **Cross-repo companion (R0-r1 M-1/M-2):** new toolkit entry `gui-secret-mirror-phrase-ms1-stdin` (open) — GUI must mirror the 3 names in `mnemonic-gui/src/secrets.rs` (token-for-token drift gate) + flip **9 hand-coded `secret:` sites** in `src/schema/mnemonic.rs` (3 flags × 3 xpub-search subcommands; audit I4's :2286/2448/2718 are the `--phrase` entries only) at its next toolkit pin bump. GUI-side: do NOT file a fresh third entry — add `Companion:` lines to the two EXISTING open GUI entries `xpub-search-inline-phrase-not-secret-classified` (`mnemonic-gui/FOLLOWUPS.md:73`) and `ms-repair-ms1-not-secret-classified` (`:81`); registry is at GUI repo ROOT, not design/ (known trap). Scope honesty: this cycle unblocks only the `schema/mnemonic.rs` half of I4 — the `schema/ms.rs:321` half mirrors the **ms-cli** surface (the toolkit's own `repair --ms1` is already secret:true) and is untouched by this cycle.
- **CHANGELOG.md:** `[0.53.1]` entry (changelog-check.yml gates tags).
- **Version:** Cargo.toml `0.53.0 → 0.53.1` + Cargo.lock + README version markers ×2 (`both_readmes_carry_current_version_marker`).
- **Manual:** NO edit — no flag-name/help-text change; flag-coverage lint unaffected.
- **GUI schema_mirror:** unaffected (flag-NAME parity only; no names change). The GUI **secret-drift** gate will RED at the next GUI pin bump until the companion lands — that is the designed lagging indicator; the companion FOLLOWUP is the leading record.

### SemVer call — PATCH (advisor-checkpoint at tag time)

No flag/subcommand/value surface change; card bytes unchanged; the only wire delta is gui-schema JSON `secret` metadata flipping `false→true` for 3 names (same shape). This is a *misclassification bugfix* (precedent: v0.47.3 `--timestamp` default-value drift = PATCH), not a new capability (contrast v0.49.0 new input format = MINOR, v0.53.0 card-byte change = MINOR). Confirm with a quick advisor sanity-check before tagging.

## Phases

1. **Phase 1 (TDD RED):** write §7b cells + rename/re-comment :284 → run `cargo test --test cli_gui_schema_v5_extensions` → expect EXACTLY the corrected RED matrix above (Cell 1: `--phrase` ×3; Cell 2: `--ms1-stdin` ×3 only; Cell 3: set diff) — these are integration tests in `tests/`, no `--bin` trap here.
2. **Phase 2 (GREEN):** D1 classification + D3 lint rows + boundary prose → full `cargo test --workspace` green + clippy + fmt.
3. **Phase 3 (ritual):** D4 docs/version/CHANGELOG → full suite again → commit (staged paths explicitly) → push → CI green (rust + manual + changelog-check + sibling-pin-check) → advisor SemVer confirm → tag `mnemonic-toolkit-v0.53.1` → push tag → CI green → resolve FOLLOWUPS + GUI companion entry.

## Risks / non-goals

- **Non-goal:** positional secrets (GUI I5), tree WIF/hex redaction (GUI I6), the GUI-side I4 flips themselves (companion-staged), `secret_taxonomy` node/slot-level classification (separate axis, already gated).
- **Risk — heuristic false-positives on future flags:** mitigated by the `EXEMPT` literal escape hatch + kind exclusions.
- **Risk — GUI behavior change on pin bump:** masking/run-confirm turns ON for `--phrase`/`--ms1-stdin` forms — that is the *point* (I4); GUI cycle validates UX.
- **Risk — `--digits` heuristic collision:** `--digits` already secret; net matches it; no-op.
