# cycle-1 — WHOLE-DIFF adversarial review (final pre-ship gate)

**Reviewer:** opus (mandatory non-deferrable post-implementation review, CLAUDE.md §post-implementation)
**Scope:** the MERGED shipping state of cycle-1's 3 CRITICAL funds-safety fixes (H12 / H1 / H13) across BOTH repos, as it will ship.
**Date:** 2026-06-20
**This review:** R0 covered plan-correctness; per-phase reviews covered each phase in isolation. THIS review covers the holistic combined change — cross-phase interactions, the merged build, residual funds-safety gap, ship-readiness surface.

## Artifacts reviewed (exact state)

- **toolkit (merged):** `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/cycle1-integration`, branch `cycle1-critical-integration`, HEAD `12344847`, off `origin/master` `4d5872ed`. Merge commits `36255edf` (H13) + `12344847` (H12+H1), 0 conflicts. Combined diff = `git diff origin/master...HEAD`.
- **md-cli:** `/scratch/code/shibboleth/descriptor-mnemonic/.claude/worktrees/cycle1-h13-mdcli`, branch `fix/cycle1-h13-md-cli-reject`, HEAD `ddddeff`, off `origin/main` `54dd765`.
- Context: GREEN spec/plan + 6 prior reviews under `design/agent-reports/cycle1-*.md`.

---

# VERDICT: GREEN — 0 Critical / 0 Important — CLEAR TO SHIP

The merged tree compiles and passes the full suite on both repos; the 3 CRITICALs are fully and non-overlappingly closed by the combined change; the one cross-phase interaction with real funds-safety stakes (H12 origin change ↔ H1 origin-exclusion) is provably BENIGN; no implementation-introduced regression; scope is clean (no version/error-variant/wire-shape churn). All per-phase MINORs are non-blocking and deferrable to FOLLOWUPs.

---

## Merged build / test result

| Target | Result |
|---|---|
| `cargo test -p mnemonic-toolkit` (merged, full suite, bin+lib+integration) | **exit 0 — all pass, 0 failed** |
| `cargo clippy -p mnemonic-toolkit --tests` (merged) | **exit 0 — clean, no warnings** |
| `cargo test -p md-cli` (h13 worktree, full suite) | **16 passed, 0 failed, 0 ignored** |
| toolkit lib unit subset | 158 passed, 0 failed, 3 ignored |
| H1 verify_bundle units (`h1_` filter, bin target) | **7 passed, 0 failed** |
| H13 parse_descriptor units (`hardened_multipath` filter, bin) | 6 passed, 0 failed |
| H13 malformed-double-marker units (bin) | 2 passed, 0 failed |
| H12 `compute_default_origin_path` + `literal_xpub` units (bin) | 2+2 passed, 0 failed |
| H12 CLI `cli_bundle_h12_taproot_origin.rs` | 4 passed (multi_a→3', sortedmulti_a→3', wsh/sh-wsh clean-neg), 0 failed |
| Differential anti-vacuity (DEFAULT-CI) `h12_taproot_default_origin_anti_vacuity_leg` | **ok** (default==3', default!=2', 2'!=3' sanity) |
| Differential H1 `h1_verify_bundle_rejects_divergent_policy_md1` | **ok** (4 divergence classes reject + genuine passes) |
| Heavy bitcoind legs (`bitcoind_h12_*`, `bitcoind_*`) | correctly `#[ignore]` (env-gated; 3 ignored) |
| md-cli `h13_hardened_multipath_reject.rs` | 5 passed, 0 failed |

Clean text-merge proven semantically sound: the two merged branches compile and pass TOGETHER. The toolkit binary builds at the pre-existing `0.60.0` (NOT bumped — version-site edits correctly deferred to the next step).

---

## Cross-phase interaction analysis (core of this review)

### (1) H12 origin change ↔ H1 origin-exclusion — BENIGN, no gap [PRIMARY]

**The interaction:** H12 changes the emitted BIP-48 origin's script-type leaf (`path_decl` last component `2'→3'`) for taproot multisig. H1's `md1_xpub_match` structural compare EXCLUDES origins (`path_decl` + origin/fingerprint TLV columns), comparing only `tree`, `use_site_path`, and `tlv.use_site_path_overrides`.

**Verdict: BENIGN — confirmed both directions, with an authoritative md-codec read-set proof.**

- **H1 cannot false-FAIL on H12's `3'`:** the origin is the precise field H1 excludes. Anchored by a live probe: `h1_origin_divergent_but_policy_equal_passes` mutates ONLY the origin (`path_decl` `0'→5'` + clears origin TLV) and asserts PASS. So an origin delta — H12's `2'→3'` included — does not move H1's verdict.
- **H12's `3'` is still VERIFIED — transitively, not via field equality:** in descriptor-mode verify-bundle the H12-set origin is written into `descriptor_resolved.path_decl.paths` (`verify_bundle.rs:1373-1453`) and is then read at `:1476` as `anno_path`, which is the BIP-32 path the phrase-slot cosigner xpub is DERIVED at (`master.derive_priv(&secp, &anno_path)`, `:1530`). A wrong subtree (`2'`) derives a DIFFERENT account xpub → a different `tlv.pubkeys` entry → H1's subordinate pubkey-multiset check FAILS. Bundle emits at `3'` (`bundle.rs:1448`) and verify re-derives at `3'` (`verify_bundle.rs:1378`) via the SAME helper on the SAME `canonicity_probe.tree.tag` — symmetric. So `3'` is enforced through the derived-key binding, exactly the layer H1 leaves to the pubkey-multiset.
- **No gap where the two TOGETHER pass a wrong wallet:** the two checks are complementary and non-overlapping. Address-driving correctness is partitioned: keys → pubkey-multiset (catches H12 wrong-subtree xpub); script structure + change-chain → H1 `tree`/`use_site_path`/overrides; origin metadata → excluded by both *because md-codec does not derive addresses from it* (see read-set proof below). There is no field that is both address-driving and unchecked.

**md-codec 0.37.0 address-driving read-set (independently verified against pinned source):** `Descriptor::derive_address` → `to_miniscript_descriptor` → `node_to_descriptor(&d.tree, &keys)` where `keys = expand_per_at_n(d)`. `expand_per_at_n` (`canonicalize.rs:420+`) reads per-`@N`: `d.tree`, `d.use_site_path` + `d.tlv.use_site_path_overrides` (ADDRESS-DRIVING ✓ in H1 gate), `d.tlv.pubkeys` (ADDRESS-DRIVING ✓ pubkey-multiset), and `d.path_decl` + `d.tlv.fingerprints` + `d.tlv.origin_path_overrides` (origin = PSBT key-source metadata, NOT address-driving — `derive.rs:17` documents this verbatim). `d.tlv.unknown` is NOT read. `d.n` is structurally implied by `d.tree`. **Conclusion: H1's compared set (`tree` + `use_site_path` + `use_site_path_overrides` + pubkey-multiset) is the COMPLETE address-driving field set; the excluded fields are exactly the non-address-driving origin/identity metadata + forward-compat passthrough.** This is the funds-safety closure: nothing that moves an address is unchecked, and H12's origin change touches only a non-address-driving field (its address effect flows through the derived xpub, which IS checked).

### (2) H13 parse-time reject ↔ H12/H1 (operate on valid descriptors) — no interaction

H13 rejects malformed inputs (hardened / malformed-double-marker multipath) at the lexer, BEFORE any descriptor is built. H12/H1 operate only on already-valid descriptors. There is no path where H13's reject alters H12/H1 behavior: a rejected input never reaches the H12 origin-inference or the H1 compare. Conversely, H13's stricter lexer does NOT reject valid H12 inputs — proven by the H12 CLI tests, which feed `tr(NUMS,multi_a(2,@0/<0;1>/*,...))` (non-hardened `<0;1>`) end-to-end through `parse_descriptor` (which now contains H13's lexer) and PASS. The H13 clean-negatives (`lex_nonhardened_multipath_still_parses`, `parse_descriptor_nonhardened_multipath_ok`) independently confirm valid `<0;1>` is accepted.

### (3) Shared symbols touched by two phases — no collision / no drift

- **`template::bip48_script_type_for_root_tag` (H12):** single definition (`template.rs:295`), called identically from `bundle.rs:1448` AND `verify_bundle.rs:1378` with the same `canonicity_probe.tree.tag`. Fully symmetric (bundle emit ↔ verify re-derive agree by construction). Delegates to the unchanged single 1/2/3 authority `CliTemplate::bip48_script_type`. No second definition.
- **`parse_descriptor` lexer (H13) ↔ `parse_descriptor` canonicity-probe (H12 in bundle/verify):** H12's bundle/verify both call `parse_descriptor` as the canonicity probe; that probe now routes through H13's stricter lexer. Confirmed benign (per (2)): valid taproot multipath descriptors pass; only malformed bodies reject (which should never have produced a bundle anyway). The H12 xpub-search arm (`descriptor_intake.rs`) uses rust-miniscript `MsDescriptor::from_str` — NOT the toolkit lexer — so it is untouched by H13.
- **`make_use_site_path` signature (H13 toolkit):** changed non-`Result`→`Result` with `?` threaded at the two `resolve_placeholders` call sites; the full suite passing confirms no dangling non-`Result` caller. md-cli's was already `Result` on origin/main (pre-existing from the 0.7.1 hardened-anywhere guard), so the toolkit change converges the two to the intended structural twin; no md-cli signature break.

---

## Holistic funds-safety — all 3 CRITICALs closed, no residual gap, no new bug

- **H12 (taproot `3'` origin):** CLOSED both sides (bundle emit + verify re-derive) and on the xpub-search literal arm. Empirical anchor holds in the merged tree: default taproot xpubs == independent `3'` derivation and != `2'` (anti-vacuity leg + CLI test + re-captured `cli_restore_multisig_general` golden, whose `bc1p…` value legitimately CHANGED — itself corroboration that `3'≠2'`). wsh/sh-wsh clean-negatives confirm taproot-specificity.
- **H1 (verify-bundle structural compare):** CLOSED. Widened `md1_xpub_match` from pubkey-multiset-only to ALSO require structural `tree == && use_site_path == && tlv.use_site_path_overrides ==`. Discriminators (wrong-k, sorted-vs-unsorted, script-type-wrapper, multipath `<0;1>`-vs-`<2;3>`, multipath presence/count) all RED→GREEN at both the unit and the CLI exit-code level. Genuine-match + origin-divergent-but-policy-equal clean-negatives confirm no over-rejection. Field-completeness proven against md-codec 0.37 read-set (above) — no address-driving field missed.
- **H13 (reject hardened/malformed multipath, both repos):** CLOSED in toolkit + md-cli with structurally-twin implementations (permissive `<([^>]*)>` capture → strict per-alt integer validation; `substitute_synthetic` strip-class reverted to `[0-9;]`). Rejects hardened (`'`/`h`) and malformed-double-marker (`<0'';1>`,`<0'h;1>`,`<0h';1>`) as typed parse errors (exit 2), never the prior silent bare-`/*` collapse. The C1 round-1 finding (malformed-double-marker silent collapse) is folded and regression-guarded end-to-end in both repos.

**No new bug introduced by the combination.** Clippy clean, full suite green, scope minimal.

---

## Critical

None.

## Important

None.

## Minor (all non-blocking — defer to FOLLOWUP)

| ID | Summary | Block ship? | Disposition |
|---|---|---|---|
| **H12 M-1** | Legacy BIP-45 `sh(sortedmulti)` (not sh-wsh) now defaults to `1'` instead of `2'` (`Tag::Sh → ShWshSortedMulti → 1'`). Neither leaf is BIP-45-correct (BIP-45 has no script-type component); both are best-effort heuristics for an under-specified descriptor, user-overridable. Not a correctness regression. | **No** | DEFER — optional one-line FOLLOWUP noting the `sh`-default heuristic is BIP-48-biased, if a future cycle wants BIP-45-aware handling. |
| **H1 n-H1-1** | Mismatch `detail` granularity: `expected`/`actual` render `tree` Debug only, so a use-site-only divergence shows identical `tree` Debug in those fields (the `classes` string still names it; `passed:false` verdict correct). Purely diagnostic, non-load-bearing. | **No** | DEFER — optional polish to surface the differing use-site field. |
| **H1 n-H1-2** | `tlv.unknown` exclusion is correct for md-codec 0.37 (independently re-verified: no derivation reads it). Worth a generic completeness tripwire: IF a future md-codec promotes an `unknown` tag to address-driving, the gate must be revisited. Not an H1 defect. | **No** | DEFER — FOLLOWUP tripwire note (forward-compat hygiene, not cycle-1 funds-safety). |
| **H1 n-H1-3 (m-NEW-1)** | Confirmation the decode-boundary `==`-safety comment landed correctly. | n/a | Already folded — no action. |

None of the four block ship. M-1 and n-H1-2 are the two worth a one-line FOLLOWUP each (cross-cutting heuristic / forward-compat tripwire); n-H1-1 is cosmetic.

---

## Ship-readiness / deferred-minors

- **Only intended files changed.** toolkit: 9 files (5 production — `bundle.rs`, `verify_bundle.rs`, `descriptor_intake.rs`, `parse_descriptor.rs`, `template.rs`; 4 test — `bitcoind_differential.rs` [modified], `cli_bundle_h12_taproot_origin.rs` [new], `cli_non_canonical_descriptor.rs` [3'-notice update], `cli_restore_multisig_general.rs` [golden re-capture]). md-cli: 2 files (`parse/template.rs`, `h13_hardened_multipath_reject.rs` [new]).
- **No `cargo fmt --all` / mlock churn** (no `mlock` file touched; diff scope is surgical).
- **No version-site edits** (`Cargo.toml`/`Cargo.lock`/READMEs/`install.sh`/fuzz all untouched on both branches) — CORRECT; that is the next step.
- **No new error variants** (`error.rs` untouched both repos; H1 reuses `VerifyCheck`, H13 reuses `DescriptorParse`/`TemplateParse`, both pre-existing). Alphabetical-ordering convention N/A.
- **`--json` wire-shape unchanged (Q-WIRE):** `format.rs` (home of `VerifyCheck`) is UNCHANGED. H1 keeps the check NAME `md1_xpub_match`; its reject path populates the pre-existing `#[serde(skip_serializing_if="Option::is_none")]` forensic fields (`expected`/`actual`) — exactly their documented v0.4.1 purpose on `passed:false`. No field add/rename/removal. bundle `--json` shape unchanged (H12 only changes a path VALUE rendered into the existing `origin_path`/notice strings).
- **Manual / GUI schema-mirror:** no clap flag/subcommand/dropdown-value add/remove/rename in this diff (purely behavioral fixes to existing surfaces), so neither the `docs/manual` CLI-reference mirror nor the `mnemonic-gui` schema-mirror is triggered by cycle-1. (The next-step release ritual still owns version-site + README bumps.)

**CLEAR TO SHIP.** Proceed to version-bump / tag / merge-to-mainline.
