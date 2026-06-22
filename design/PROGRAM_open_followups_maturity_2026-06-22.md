# PROGRAM PLAN — Open Follow-ups Maturity Program (m-format constellation)

**Date:** 2026-06-22  **Author:** opus architect (planning/cataloging pass — read-only)
**Scope:** all 5 constellation repos — `mnemonic-toolkit`, `descriptor-mnemonic` (md), `mnemonic-secret` (ms), `mnemonic-key` (mk), `mnemonic-gui`
**Ground-truth source:** the five `FOLLOWUPS.md` files at `origin/master` (all in-sync 2026-06-22 reconcile) + `mnemonic-toolkit/CLAUDE.md`. Open-set extracted programmatically and spot-verified entry-by-entry. The live fmt-gate state was reproduced (`cargo +1.95.0 fmt --all -- --check` → 79 diff hunks across ~30 non-exempt source files; gate is RED on master right now).
**Status:** producer pass complete; adversarial review pass appended at the end. This is a roadmap/catalog, **not** an implementation plan for any feature.

---

## 1. Executive summary

The constellation's **critical funds-correctness and secret-leak work is closed.** The recent bug-hunt resolved every funds/correctness finding except one WONTFIX (the "57/58" tally is the bug-hunt report's own count, not in the FOLLOWUPS registries), and the cycle-14/cycle-15 secret-key-material sweeps wrapped the owner-side secrets (clap-arg, handler-local, persistent-field, and the highest-value derived-output sites). What remains is a **maturity-and-consistency program riding on top of a large, mostly-parked feature catalog.** Of ~144 open entries constellation-wide, only roughly **a dozen are genuinely actionable maturity work**; the rest are blocked-on-upstream, standing-discipline tripwires (open *by design*), or deferred features the user explicitly wants cataloged but **not** scheduled. The single most urgent item is **not a feature at all** — the pinned `+1.95.0` rustfmt CI gate is **live-RED on master** (verified: 79 diff hunks across ~29 files — ~28 non-exempt + the g6-exempt `mlock.rs`), which silently masks real format drift and weakens every downstream gate. After that, the work clusters into: secret-hygiene *residue* (defense-in-depth tails on already-scrubbed owners), a handful of low-severity funds-adjacent diagnostic/labeling fixes (the real funds holes are already closed — what's left is mislabeled-surface and refusal-message-quality), and consistency debt (ungated `--json` wire-shapes, cross-CLI cosmetics, manual/schema mirror lockstep). The funds-safety axis is treated as highest-severity by project ethos, but in practice it contributes only a few small SCHEDULABLE items plus one large BLOCKED umbrella (taproot coverage, waiting on a rust-miniscript release `> 13.1.0`). This plan sequences the actionable tail into **four waves**, separates the two non-burndown buckets (BLOCKED-on-upstream; STANDING-discipline) so they are never mistaken for work, and catalogs the deferred-feature backlog by theme without designing any of it.

---

## 2. Prioritization framework

### 2.1 The four axes, weighted

The user named four priorities. We weight them as a **strict severity ladder for ordering within and across waves**, with one override:

> **funds-safety > secret-hygiene > broken-gate > consistency > cosmetic > feature-catalog**

- **Funds safety (override axis).** Project ethos: never emit a wrong address / never silently mis-build a wallet. *Any* open item with a funds implication is ordered **first within its track**, regardless of size — even though, post-bug-hunt, every such open item is either already loud-and-safe (a refusal/labeling-quality residual) or BLOCKED on upstream. Funds severity is about *consequence-if-wrong*, not *likelihood*; we keep it at the top of the ladder on principle.
- **Secret-hygiene (first-class bar).** Memory-hygiene of key material. Post-sweep, the open items are **residue** (defense-in-depth tails on owners that are already scrubbed) — real but not leak-to-disk/argv/log. Ranked above broken-gate only when a residual touches a live spending-authority value (`Xpriv`/seed-phrase) versus a transient husk.
- **Broken-gate (maturity).** A CI gate that is RED or vacuous *actively erodes* every other guarantee, so a live-broken gate outranks consistency and cosmetics. The fmt gate is the headline; vacuous `MD_BIN=true`/`MNEMONIC_BIN=true` lint substitutions and the `ms-codec` no-CI gap are the same class.
- **Consistency (parity/lockstep).** Cross-CLI/codec parity, mirror/lockstep discipline, ungated wire-shape drift surfaces. Important for long-term maintainability; not safety-bearing on its own.
- **Cosmetic.** Message-quality, wording, doc-prose staleness, dead-code, comment-rot.
- **Feature-catalog.** New surfaces. **CATALOG ONLY — not scheduled** per the hard constraint.

### 2.2 Two buckets that are NOT schedulable burndown

These must never be counted as "remaining work to close":

- **BLOCKED-on-upstream (monitor-only).** Items whose fix is gated on a third-party release/merge the constellation does not control (rust-miniscript `>13.1.0`, rust-bitcoin/secp256k1/bip39/sha3/bip38/codex32 zeroize). Nothing is actionable here until the trigger fires. See §7.
- **STANDING-discipline tripwires (open by design).** Mirror/lockstep trackers that are *meant* to stay open for the lifetime of the artifact they guard. Closing them would mean *deleting the discipline*. They are surfaced in §5.3, never in a wave.

### 2.3 Gate discipline that governs every wave (from CLAUDE.md)

Every scheduled wave below inherits the project's hard gates, restated as reminders, not re-derived:

- **R0 gate:** every brainstorm spec and plan-doc passes an opus R0 review to **0 Critical / 0 Important before any code.** Reviewer-loop continues after every fold; persist reviews verbatim to `design/agent-reports/` before folding.
- **Per-phase TDD + post-impl adversarial review** (the 5-step ultracode per-phase pattern). Mandatory, non-deferrable.
- **Citation re-grep at write-time** — FOLLOWUPS line numbers decay every merge; re-grep against `origin/master` and pin the source SHA in the plan-doc.
- **Alphabetical `ToolkitError` ordering** for any new variant + match block.
- **Mirror invariants** fire on every CLI-surface change: manual (`docs/manual/src/40-cli-reference/`) and GUI `schema_mirror` (`mnemonic-gui/src/schema/mnemonic.rs`) in the **same PR or a paired sibling PR** (the leading discipline; the gate is a lagging indicator).
- **SemVer:** per-crate; new accepted input format / new flag = MINOR; pure internal hygiene/cosmetic = NO-BUMP (PATCH only if it changes observable behavior). `Cargo.lock` regenerated + committed on any version bump.

---

## 3. Sequenced maturity program (the core)

Four waves, ordered by the §2.1 ladder + dependencies. Sizes are **rough** (S ≤ ~1 focused cycle / single file-family; M = multi-file or cross-repo lockstep; L = multi-phase or cross-repo publish). Every wave runs under the §2.3 gates.

> **Cross-cutting dependency note:** several cross-repo hygiene/consistency closes are *cheapest* when folded into the **next ms-cli g6-pin bump that is also 1.95.0-formatted** — that single event would discharge `mlock-g4-a-page-count-assert-flake`, `mlock-rs-fmt-exempt`, and re-baseline the synced `mlock.rs`. Wave 1 should *decide* the fmt-canonical-formatter question because it determines whether that combined ms-cli tag is even possible. This is the program's one true sequencing keystone.

---

### Wave 1 — Live-broken gate + funds-adjacent safety residuals  *(highest severity; unblocks everything)*

**Theme:** stop the bleeding. A RED gate masks all future drift; the funds-adjacent residuals are loud-and-safe today but are the only open items on the override axis.

| Slug | Repo(s) | Size | SemVer | Notes |
|---|---|---|---|---|
| `toolkit-rustfmt-1-95-0-rebaseline-divergence` | toolkit (+ ms via mlock coupling) | **M** | NO-BUMP (chore) | **THE headline. Live-RED (verified ~30 files).** Decide canonical formatter: (a) re-pin gate to the rustfmt build that produced committed code, or (b) re-baseline whole workspace with new `+1.95.0` in ONE chore commit, then `git checkout -- mlock.rs` (g6 exemption) + bump the pinned-version comment. **Re-grep the file list at fix time.** Coupled to `mlock-rs-fmt-exempt`. |
| `mlock-rs-fmt-exempt` | toolkit + ms | **S** | NO-BUMP | The g6 fmt carve-out; resolved *together* with the re-baseline if option (b) chosen and a new 1.95.0-formatted ms-cli tag is cut. Otherwise stays exempt. Decide jointly with the item above. |
| `lint-md-flag-coverage-vacuous-with-md_bin-true` + the `manual-*-bin-real-binary` family | toolkit | **M** | NO-BUMP | **Vacuous-gate class.** CI manual flag-coverage runs with `MD_BIN=true`/`MNEMONIC_BIN=true` placeholders → the bidirectional flag check is a no-op for those binaries. Promote to real binaries. (Several tracked as separate slugs but are one coordinated CI fix.) |
| `export-wallet-green-tr-policy-singlesig-emission` | toolkit | **S** | PATCH (behavior) | **Funds-adjacent (first on override axis).** `export-wallet --format green` mislabels a general-tr policy as "singlesig". Descriptor *inside* is faithful → mislabeled-surface, not wrong-descriptor. restore-side already refuses (v0.55.1). Mirror that refusal in `green.rs::emit` (option a). |
| `export-wallet-direct-descriptor-unsorted-multi-generic-refusal` | toolkit | **S** | NO-BUMP | **Funds-adjacent (already safe).** Direct `--descriptor 'wsh(multi(…))'` to field-less vendor formats is *already refused* (generic `BadInput`); only the message is less specific than the typed H10 path. Cosmetic message-quality — surface the typed unsorted-multisig error. No behavioral change. |
| `xpub-search-descriptor-md1-detection-bech32-validate` | toolkit | **S** | NO-BUMP | **Funds-adjacent (tightening).** md1 tie-break uses `starts_with("md1")` not full bech32 validation. Today false-positives surface as clean typed errors (not silent misroutes), so it's defensible — but tightening to real bech32 syntactic validation removes a class of mis-route ambiguity. |

**Ordering rationale:** the fmt gate is first because while it is RED, no other gate's GREEN can be trusted, and any wave that touches the listed source files risks compounding the drift. The vacuous-lint promotions come next (same broken-gate class). The four funds-adjacent items are ordered ahead of all of Wave 2+ by the override axis even though they are S/cosmetic — they are the *only* open items on the funds track, and `green-tr` is the one with an actual behavioral (labeling) change.

**Lockstep:** the fmt decision is coupled cross-repo to ms via `mlock.rs`/g6. If option (b) is taken and a fresh 1.95.0-formatted ms-cli tag is cut, **fold `mlock-g4-a-page-count-assert-flake`'s one-line `assert_eq!` deletion into the same ms-cli tag** (its architect verdict is "leave tracked until the next g6 pin moves anyway") and re-pin `install.sh`/`manual.yml`/`rust.yml`.

---

### Wave 2 — Secret-hygiene residue  *(first-class bar; defense-in-depth tails)*

**Theme:** finish the sweep's tail. Owners are already scrubbed; these are the lingering bare-copy / un-confined-field residuals. None is a leak-to-disk/argv/log. Order within the wave by *value-of-the-secret-held* (full spending authority > transient husk).

| Slug | Repo(s) | Size | SemVer | Notes |
|---|---|---|---|---|
| `ms-cli-derive-xpriv-master-not-zeroized` (in-repo leg) | ms | **S** | NO-BUMP | **Highest-value residual: the master/root `Xpriv`** — `ms derive` is the only site that materializes an actual xpriv (root = every account, strictly above an account Xpriv). PARTIAL (cycle-15 Lane M lifetime-min landed). The in-repo leg (minimize lifetime + scrub reachable bytes) is schedulable; the FULL close is upstream-BLOCKED via `rust-bitcoin-xpriv-zeroize-upstream` (§7.2). |
| `derive-slot-account-xpriv-scrub-confinement` (the **7-site lift remainder**) | toolkit | **M** | NO-BUMP (watch SemVer caveat) | High-value residual: account `Xpriv` (full *account* spending authority). The minimal `ScrubbedXpriv` helper + `derive_account_xpub_only` already SHIPPED in v0.70.0; **remaining = replace `DerivedAccount.account_xpriv: Xpriv` with `ScrubbedXpriv` across ~7 `derive_slot.rs` consumers + `into_parts`.** Watch the pub-struct-Drop SemVer risk — `impl Drop` on a pub struct breaks move-out destructure for external lib users (toolkit isn't published yet, so contained). |
| `self-check-ms1-decode-not-zeroizing` | toolkit | **S** | NO-BUMP | Two sites (`bundle.rs::self_check_bundle`, `cmd/inspect.rs`) drop a decoded ms1 `Payload` (master-seed entropy) un-scrubbed. Fix = mirror the in-file sibling idiom (move entropy into `Zeroizing`). Add a `lint_zeroize_discipline` row. |
| `phrase-overlay-secretstring` | toolkit | **S/M** | NO-BUMP | `import-wallet` phrase-overlay copies the seed phrase into bare `Vec<(u8, String)>`. **No NEW residue vs pre-cycle-14** (status-quo). Fix = flip `apply_seed_overlay` signature + `Source::Phrase` to `SecretString` (non-trivial fan-out — why cycle-14 deferred it). |
| `stdin-reader-transient-buf-zeroizing` | toolkit | **S** | NO-BUMP | `read_stdin_passphrase`/`read_stdin_to_string` transient `buf` is bare. Owners already scrubbed; this is the last transient stack-rooted copy. In-fn `Zeroizing<String>`, return type unchanged (avoid `Zeroizing<Zeroizing<String>>` at the 14 call sites). |
| `ms-codec-share-strings-not-zeroized-encode-and-combine` (residual `String` legs) | ms | **M** | NO-BUMP / blocked-tail | **PARTIAL (cycle-15 Lane M):** reachable `Vec<u8>` intermediates wrapped; the `Codex32String`/`String` legs are root-caused to the codex32 vendor/fork decision → that portion is **BLOCKED** (see §7). |

**Deliberately NOT scheduled here (defense-in-depth at the libc/allocator floor — `v1+` tier):** `clap-argv-pre-parse-residue`, `allocator-pool-residue`, `dedicated-secret-arena`. Correctly parked as `v1+` design-class; flagged here only so the reader knows they were considered and consciously excluded.

**GUI hygiene residue (paired sub-wave — runs on the GUI's own cadence):**

| Slug | Repo | Size | SemVer | Notes |
|---|---|---|---|---|
| `gui-run-confirm-modal-secret-redaction` (+ toolkit companion manual prose) | gui (+ toolkit manual) | **M** | GUI MINOR | **Highest GUI severity.** Run-confirm modal renders secret-bearing argv tokens in **plaintext**; the manual currently ships "honestly-broken" prose. Fix = `redact_argv_for_display` mirroring `persistence::redact_for_persistence`. **Lockstep:** GUI source patch + toolkit manual-gui prose patch + pin bump — must land together. **Co-lander:** `gui-import-wallet-env-var-secret-channel` (the argv-rewrite-to-`@env:` direction) is specified to land WITH this display-redaction fix — schedule the pair, not half. |
| `tree-xprv-heuristic-only-covers-key-fields` | gui | **S** | GUI PATCH | Extend `is_xprv_like` sweep to `hex`/`w` free-text fields (belt-and-suspenders). |
| `gui-tree-key-egui-undo-ring-residue` | gui | **S** | GUI PATCH/deferred | egui undo ring may retain tree-key keystrokes post-zeroize. May stay a documented caveat. |
| `tree-mode-posix-pipeline-spec-json-unmasked` | gui | **S** | deferred/conditional | **No live leak** (build-descriptor tree is over xpubs/watch-only). Conditional: add JSON-redaction only if a future node carries secret-class value. |

**Ordering rationale:** within Wave 2 the master/root-Xpriv in-repo leg (`ms-cli-derive-xpriv-master…`) ranks first (root = every account), then `derive-slot…account-Xpriv` and `self-check-ms1…` (spending-authority-equivalent); the stdin/overlay/transient items follow as lower-value lingering copies. The GUI sub-wave is severity-led by the modal-redaction gap (on-screen plaintext exposure the manual currently documents as broken).

---

### Wave 3 — Consistency / wire-shape & cross-surface parity  *(maintainability)*

**Theme:** close the *schedulable* consistency debt — the ungated `--json` wire-shape surfaces and cross-CLI cosmetics. (Standing-discipline trackers excluded — see §5.3.)

| Slug | Repo(s) | Size | SemVer | Notes |
|---|---|---|---|---|
| `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification` (residual **option (b)**) | toolkit + gui | **M** | toolkit NO-BUMP; gui test-only | Option (c) [document the gap] already shipped (v0.34.3). **Residual = option (b):** per-consumer `--json` wire-shape regression tests on the GUI side for high-traffic subcommands (`xpub-search`/`import-wallet`/`export-wallet`). **The keystone consistency item** — converts the ungated wire-shape surface from "paired-PR discipline" to "automated gate", dissolving the drift-risk that `ms-kofn-json-wire-shape-ungated` + `toolkit-mnem-ms1-wire-shape-downstream-consumers` merely *track*. |
| `friendly-mk-codec-mixedcase-wording` | toolkit (+ mk SPEC §6.4.4) | **S** | NO-BUMP | Cross-CLI cosmetic: `friendly_mk_codec` MixedCase word-order differs from SPEC. |
| `mk-vectors-pretty-out-help-mismatch` | toolkit + mk | **S** | NO-BUMP | `mk vectors --pretty` help-text vs source behavior drift. |
| `walker-backport-to-md-cli` (+ md `terminal-rawpkh-walker-arm-missing`) | toolkit → md | **M** | md MINOR | Backport toolkit's expanded miniscript walker to md-cli (md-cli's walker is narrower). |
| `output-type-stderr-advisory` + `-sibling-sweep-mk-md` + `output-class-advisory-byte-parity-test-tautological` | toolkit + mk + md | **M** | NO-BUMP | Cross-CLI output-class stderr advisory parity (Cycle-B Phase 2). The byte-parity test is currently a within-repo tautology, not a cross-repo drift gate. |
| `install-sh-sibling-pins-stale-vs-flag-bearing-clis` + `install-sh-gui-sibling-pin-staleness-ungated` + `manual-yml-sibling-pin-vs-install-sh-drift-gate` + `sibling-pin-check-skips-manual-prose-install-commands` | toolkit | **M** | NO-BUMP | **Pin-staleness consistency cluster.** install.sh sibling pins stale vs flag-bearing releases; GUI pin drift *ungated* (was 8 versions stale, silently shipping a GUI missing security fixes); manual-prose install commands unscanned. Add cross-repo `gh api` drift-check + the manual.yml↔install.sh static gate. **Note the MSRV tension** (GUI v0.40.0+ needs rustc ≥1.88 vs toolkit MSRV 1.85). |
| `ms-codec-no-ci-workflow` | ms | **M** | NO-BUMP | **Absent-gate consistency.** `ms-codec` has **no CI at all**; ms-cli's `rust.yml` has no fmt step. Add test+clippy+fmt for both crates; requires a one-time `chore(fmt)` normalization commit FIRST (~16 files), landed standalone. |

**Ordering rationale:** the wire-shape gate option (b) leads because it *structurally* reduces future drift across the most-trafficked surfaces. Cross-CLI cosmetics and pin-staleness follow. `ms-codec-no-ci` grouped here as the ms-side absent-gate (blast-radius lower than the funds-bearing toolkit fmt gate).

---

### Wave 4 — Maturity tail: error-handling polish, dead-code, doc/prose staleness  *(robustness + cosmetic)*

**Theme:** the long, low-severity tail. Batch by file-family to keep diffs scoped. NO-BUMP unless noted. Representative (non-exhaustive) members:

- **Refactor/dedup (robustness):** `verify-bundle-bundle-rs-descriptor-mode-dedup` (**M** — dedup the bundle.rs ↔ verify_bundle.rs descriptor-mode binding so guard-drift can't recur; genuine correctness motivation — prioritize within the wave), `restore-emit-dispatch-3way-dedup`, `cmd-repair-inspect-helper-duplication`, `descriptor-origin-extraction-dedup`, `synthesize-descriptor-deduplicate-with-unified`.
- **Error-handling/diagnostics polish:** `export-wallet-bundle-descriptor-md1-clearer-error`, `verify-message-format-requested-debug-string`, `bip388-template-path-wallet-name`, the `pr-26-*` warning/default family.
- **Doc/prose/comment staleness (cosmetic):** `manual-v0.18-stale-md1-scenario-phrases`, `changelog-md-release-ritual-*`, `readme-subcommand-inventory-*`, `cli-help-golden-broad-staleness-not-gated`, the `pr-26-comment-rot-fold` family.
- **Test-hardening:** `cross-start-convergence-remaining-cells`, the `verify-bundle-multisig-*` unit-coverage residuals, `toolkit-descriptor-fuzz-target`.
- **Type-design sweeps (SemVer-aware):** `xpub-search-result-type-level-invariant-blocked-on-wire-shape-evolution` (**MINOR** wire-shape, v0.28+ — pair with the Wave-3 wire-shape gate), `pr-26-type-design-anti-pattern-sweep`.
- **Funds-adjacent advisories (from §4):** `bundle-accepts-sortedmulti-in-combinator-restore-cannot` (reconstruction asymmetry — the engrave-time advisory already fires), `verify-bundle-watch-only-xpub-path-internal-consistency` (xpub-path cross-check, defense-in-depth). NOTE: the emit-time unrestorable advisory itself already shipped (`bundle-unrestorable-shape-advisory`, v0.57.1) — do NOT re-schedule it.

> **Note:** `error-rs-retroactive-alphabetical-sort` is **RESOLVED** (toolkit v0.29.0). Do not schedule. Its successor `error-rs-exit-code-arm-fragmentation-post-sort` is an open cosmetic decision at most.

---

## 4. Funds-safety register

Every open item with a funds implication, ordered by the override axis. **The critical funds holes are already closed** — what remains is loud-and-safe refusals, labeling-quality, and tightening, plus one large blocked umbrella.

| # | Slug | Repo | Tag | Risk (one line) |
|---|---|---|---|---|
| 1 | `export-wallet-green-tr-policy-singlesig-emission` | toolkit | **SCHEDULABLE** (Wave 1) | Mislabels a general-tr policy as "singlesig" to Green's import dialog; descriptor inside faithful → wrong-*label* not wrong-*address*. restore-side already refuses. |
| 2 | `xpub-search-descriptor-md1-detection-bech32-validate` | toolkit | **SCHEDULABLE** (Wave 1) | md1 tie-break `starts_with` not full bech32; misroutes surface as clean typed errors today — tightening removes the ambiguity class. |
| 3 | `export-wallet-direct-descriptor-unsorted-multi-generic-refusal` | toolkit | **SCHEDULABLE** (Wave 1) | Already funds-safe (refused); only the refusal message is generic, not the typed H10. Cosmetic. |
| 4 | `restore-general-and-multi-leaf-taproot-roundtrip` (remainder) | toolkit | **BLOCKED-ON-UPSTREAM** | bundle emits general/multi-leaf tr md1 cards restore can't reconstruct; **refusal is loud + funds-safe today.** Single-leaf/depth-1 shipped (v0.55.1). Blocked on rust-miniscript `>13.1.0` + md-codec SortedMultiA. |
| 5 | `taproot-coverage-cycle-on-miniscript-gt-13-1-0` (UMBRELLA) | toolkit + md | **BLOCKED-ON-UPSTREAM** | Depth-≥2 taptrees + `sortedmulti_a`-as-non-root-leaf; refused loudly until the first crates.io rust-miniscript `>13.1.0` carrying PR #953 + #910. See §7. |
| 6 | `bundle-accepts-sortedmulti-in-combinator-restore-cannot` | toolkit | **SCHEDULABLE** (Wave 4) | Round-trip asymmetry; refusal is loud → no wrong-output, just an engrave-but-can't-mechanically-restore gap. |
| 7 | `bundle-engraves-unrestorable-pk-keyed-cards` (reconstruction remainder) | toolkit | **mostly RESOLVED; remainder BLOCKED** | The pk-keyed concern is RESOLVED (v0.54.1) and the **emit-time advisory ALREADY SHIPPED** — `bundle-unrestorable-shape-advisory` (`src/unrestorable_advisory.rs`, v0.57.1) fires at engrave time on `bundle` + `import-wallet` for all 3 unrestorable shapes, IFF restore would refuse. The only open remainder is *reconstruction* of sortedmulti-in-combinator (= row 6), partly upstream-gated. NOT new advisory work. |
| 8 | `coldcard-bip86…` / `…-tr-multi-a-pending-firmware` / `jade-…` / `electrum-…` / `green-native-multisig-…` | toolkit | **BLOCKED-ON-UPSTREAM** | Each *refuses* the unsupported export today (funds-safe); unblock = vendor firmware/server. |
| 9 | `restore-non-nums-tr-internal-key-also-in-leaf` | toolkit | **WONTFIX (permanent loud refusal)** | Degenerate key-reuse; the permanent loud refusal IS the funds-safe answer. Not work. |
| 10 | `verify-bundle-watch-only-xpub-path-internal-consistency` | toolkit | **SCHEDULABLE** (Wave 4) | watch-only verify-bundle doesn't cross-check mk1 xpub fields against md1's claimed OriginPath → weaker (not wrong) verification; defense-in-depth. |

**Headline:** of 10 funds-track entries, **3 SCHEDULABLE small Wave-1 items, 3 SCHEDULABLE Wave-4 advisories, the rest BLOCKED or WONTFIX.** No open item can currently emit a wrong address or silently mis-build a wallet; every gap is a loud refusal, a missing advisory, or a label-quality issue.

---

## 5. Consistency register

### 5.1 Ungated `--json` wire-shape surfaces (drift risk — schedulable mitigation exists)
`schema_mirror` enforces clap flag-NAME parity only, NOT wire-shape. A wire-shape change trips no gate (the documented *lagging-indicator* class).
- `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification` — meta gap; option (c) documented (v0.34.3); **option (b) per-consumer wire-shape regression tests is the open, schedulable mitigation (Wave 3).**
- `ms-kofn-json-wire-shape-ungated` — standing-posture tracker (also §5.3); structural close = option (b).
- `toolkit-mnem-ms1-wire-shape-downstream-consumers` — consumers self-update to ms-codec ≥0.3.0; paired-PR discipline.
- `gui-schema-*` projection backlog (Wave 3/4 by appetite).

### 5.2 Cross-CLI / codec parity (schedulable)
`friendly-mk-codec-mixedcase-wording`, `mk-vectors-pretty-out-help-mismatch`, `walker-backport-to-md-cli` (+ md `terminal-rawpkh-walker-arm-missing`), the output-class advisory family, the install.sh/manual.yml pin-staleness cluster, `md-codec-decode-with-correction-supports-non-chunked-md1`. See Wave 3. **`md-codec-sortedmulti-a-to-miniscript-rendering-gap` + `upstream-miniscript-taptree-depth2-display-asymmetry` are consistency surfaces but BLOCKED** (§7).

### 5.3 PERMANENT standing-discipline bucket — *open by design, NOT burndown*
**Never appears in a wave or a "remaining work" count.** Each is a tripwire that stays open for the lifetime of the artifact it guards.

| Slug | Guards | Why permanent |
|---|---|---|
| `manual-cli-surface-mirror` (+ md/ms/mk companions) | four-CLI surface ↔ `docs/manual/src/40-cli-reference/` | mirror invariant for the lifetime of `docs/manual/`. |
| `mnemonic-gui-schema-mirror` (+ toolkit companion) | toolkit clap surface ↔ `mnemonic-gui/src/schema/mnemonic.rs` | lagging drift gate; leading discipline = paired-PR rule. |
| `md-mk-private-key-surface-watch` (md/mk/ms/toolkit) | reopens Cycle-A hygiene IF md/mk grow a private-key surface | monitoring tripwire; fires only on a future event. |
| `ms-kofn-json-wire-shape-ungated` | K-of-N `--json` wire-shapes | standing-posture; structural close = §5.1 option (b), tracker itself permanent. |

*(Note: `gui-schema-mirror-lockstep-discipline` is NOT in this bucket — it was a one-time docs-codification task and is RESOLVED. The genuinely-permanent GUI-schema tripwire is `mnemonic-gui-schema-mirror`, row 2.)*

> Framing: these four are the constellation's *immune system*, not its *backlog*. The program keeps them green, it does not close them.

---

## 6. Gaps / new-features CATALOG — **CATALOG ONLY — NOT SCHEDULED FOR IMPLEMENTATION**

Per the hard constraint: identified and grouped, with a one-line description, rough strategic value, and a blocked? flag. **No design, no sequencing, no API.** "Strategic value" is a coarse signal for a *future* prioritization conversation, not a commitment.

### 6.1 Signing surface (the single biggest capability gap)
- `bip174-psbt-signing` — Partially Signed Bitcoin Transactions. *Value HIGH* (turns a backup tool into a signer; large surface + risk). *Blocked: no.*
- `bip340-schnorr-signing-surface-evaluation` — Schnorr signing surface; gates the deferred BIP-340/341 test corpora (md `bip341-keypath-signing-vector-coverage`). *Value HIGH (prerequisite). Blocked: no.*
- `bip327-musig2-collective-keys` — MuSig2 collective-key policies. *Value MED. Blocked: no (downstream of a signing surface).*
- `frost-threshold-keys` — FROST threshold signatures. *Value MED/exploratory. Blocked: no.*

### 6.2 Assets / protocol extensions
- `liquid-confidential-extended-keys` — Liquid sidechain extended-key formats. *Value LOW/niche.*
- `vault-construction-covenant-based` — CTV/OP_CAT/OP_VAULT vault descriptors. *Value LOW today (consensus-pending). Blocked: effectively.*
- `bip38-ec-multiplied-encrypt-mode-support` — BIP-38 EC-multiplied form. *Value LOW/niche* (the "refused" claim was already corrected by `bip38-spec-section-12-ec-multiplied-erratum`).
- `bip39-japanese-wordlist-support` — JP wordlist; gates JP BIP-39 vectors + silent-payment JP cross-check. *Value LOW.*
- `electrum-native-seed-address-derivation` / `electrum-non-latin-wordlists`. *Value LOW.*

### 6.3 BIP-85 application tail
- `bip85-rsa-rsa-gpg-applications` (+ `bip85-dice-application`), `bip85-passphrase-protected-master`, `bip85-non-english-bip39-language-codes`, `bip85-testnet-emission`, `bip85-stdin-master-xprv` — small BIP-85 surface-completeness gaps (flags inert). *Value LOW/niche.*

### 6.4 Batch / UX surfaces
- `single-sig-multi-script-type-batch-emit-not-surfaced` — `addresses --all-script-types` / `export-wallet --all-single-sig`. *Value MED (real ergonomics win).*
- `slip39-cli-extendable-flag` — `--extendable` toggle. *Value LOW.*
- `bip388-wallet-policy-to-descriptor-expansion-not-surfaced` — expose the policy→descriptor expander. *Value MED.*
- `miniscript-compiler-optimize-policy` / `descriptor-builder-engine` — guided custom-vault construction (md's policy compiler canonical). *Value MED/exploratory.*
- The xpub-search GUI UX cluster (`xpub-search-gui-bespoke-hub-pane`, `-bespoke-widgets`, `-positional-intake`, `-flag-mutex-visibility`, `xpub-search-manual-gui-chapters`) — discoverability/UX maturation of an existing feature. *Value MED.*

### 6.5 BSMS / multisig coordination
- `bsms-bip129-full-cutover` (+ encryption-envelope family) — complete BIP-129 conformance, deprecate the lenient 6-line parser. *Value MED. Blocked: partial (cross-impl smoke needs Coinkite ref).*
- `bsms-taproot-emit` — BIP-129 emit for tr(). *Value MED. Blocked: downstream of BIP-386/taproot tooling.*
- `wallet-import-{sparrow,specter,electrum,coldcard,jade,bsms}` ingest breadth. *Value MED.*

### 6.6 Distribution
- `mnemonic-toolkit-cratesio-publish` (+ gui publish) — swap codec git deps for crates.io, publish. *Value MED (unblocks `cargo install`; also flips the pub-struct-Drop SemVer risk from "contained" to "live" — hygiene-coupled). Blocked: gui downstream of toolkit.*

> §6 is a catalog. Nothing here is scheduled, sized, or designed. Surfaced so the user can *choose* what (if anything) to promote into a future feature cycle — which would run its own brainstorm → R0 → TDD pipeline, separate from this maturity program.

---

## 7. Blocked / monitor-only appendix

### 7.1 rust-miniscript `> 13.1.0` (the taproot keystone)
- **Umbrella:** `taproot-coverage-cycle-on-miniscript-gt-13-1-0`. **Components:** `upstream-miniscript-taptree-depth2-display-asymmetry`, `md-codec-sortedmulti-a-to-miniscript-rendering-gap`, `restore-general-and-multi-leaf-taproot-roundtrip` (remainder), md `rust-miniscript-multi-a-in-curly-braces-parser-quirk`.
- **Waits on:** the first crates.io release `> 13.1.0` carrying **PR #953** (taptree Display fix) **and** **PR #910** (`Terminal::SortedMultiA`). Latest crates.io is 13.1.0 (contains neither; they target the next major).
- **Trigger + 3-step unblock (cross-repo, ordered):** (1) release lands → (2) **md-codec cycle:** bump dep + render `SortedMultiA` as a real tap-leaf fragment, publish → (3) **toolkit cycle:** bump dep, **DROP the `[patch.crates-io]` git-rev** in both `Cargo.toml` AND `fuzz/Cargo.toml` (realign both lockfiles), LIFT the two restore gates (`ensure_taptree_depth_le_one`, `subtree_contains_sortedmulti_a`), flip refusal cells to reconstruction. Mirror in md.

### 7.2 zeroize-upstream cluster (defense-in-depth, third-party types)
`rust-bitcoin-xpriv-zeroize-upstream`, `rust-secp256k1-secretkey-zeroize-upstream`, `rust-bip39-mnemonic-zeroize-upstream`, `sha3-shake256-zeroize-upstream`, `bip38-crate-internal-zeroize-upstream`. **Waits on** upstream crates adding `Zeroize`/`Drop`. **Trigger:** an upstream release exposing the trait → wrap at use-site, drop the best-effort `non_secure_erase` caveat. Lifetimes already minimized where cheap.

### 7.3 codex32 (abandoned upstream — needs a *decision*, then unblocks)
`rust-codex32-zeroize-upstream` + `codex32-upstream-dormant-vendor-vs-accept-decision` (ms). The `Codex32String`/`String` legs of the share-string hygiene tail root here. **Waits on** a constellation **vendor/fork vs accept** decision (crate dormant; no upstream release coming). **Trigger:** internal decision — see §8 Q2.

### 7.4 Vendor firmware / server (export refusals)
`coldcard-bip86-generic-export-pending-firmware`, `coldcard-tr-multi-a-pending-firmware`, `jade-tr-multi-a-pending-firmware`, `electrum-tr-multi-a-pending-libsecp-taproot`, `green-native-multisig-pending-server-support`, `electrum-final-seed-version-drift`, `electrum-root-fingerprint-roundtrip-quirk`. Each refuses loudly today (funds-safe). **Trigger:** vendor ships the capability → flip refusal to emission + add round-trip cell.

### 7.5 Cross-repo coupled CI anchors (not external, but event-gated)
- `mlock-g4-a-page-count-assert-flake` — "leave tracked"; **trigger = the next ms-cli g6-pin bump** (ideally the 1.95.0-formatted tag that also discharges `mlock-rs-fmt-exempt`). Fold the one-line deletion then; cost ≈ zero.
- md/`rust-miniscript-fork` external-PR items — closed by upstream merge.

---

## 8. Assumptions, exclusions, and open questions for the user

### 8.1 Assumptions
1. Open-set current as of the in-sync `origin/master` reconcile (2026-06-22); extracted programmatically (heading + Status markers) and spot-verified against bodies/live source; the live fmt-gate red state was reproduced directly.
2. `audit-2026-06-10-backlog` treated as a closed backlog *index* (all sub-findings RESOLVED or WONTFIX — one benign WONTFIX, `addresses-env-sentinel-overapplied`; "open" only as container).
3. `error-rs-retroactive-alphabetical-sort` is **RESOLVED** (v0.29.0) — excluded from scheduling.
4. Sizes (S/M/L) and SemVer tags are planning estimates assuming citation re-grep at implementation time.
5. The GUI runs on its own release cadence; GUI waves require a toolkit pin bump only where a slug says so.

### 8.2 Deliberate exclusions
- No feature designed, sized for implementation, or sequenced (§6 catalog-only).
- The `v1+` defense-in-depth floor (`clap-argv-pre-parse-residue`, `allocator-pool-residue`, `dedicated-secret-arena`) excluded from near-term waves.
- The deep `verify-bundle-*` / `unified-slot-*` / `convert-*` deferred-feature tail folded into Wave-4 framing or §6 by theme rather than enumerated line-by-line (the decision-bearing ones are called out).
- The standing-discipline bucket (§5.3) excluded from all burndown math by design.

### 8.3 Open questions — decide BEFORE Wave 1
1. **fmt-gate canonicalization: re-pin (a) or re-baseline (b)?** Program keystone. (b) is cleaner *if* you also cut a fresh 1.95.0-formatted ms-cli tag to simultaneously discharge `mlock-rs-fmt-exempt` + `mlock-g4-a-page-count-assert-flake`. (a) is lower-blast-radius but leaves the mlock exemption + synced-file coupling. **Recommendation: (b) + a coordinated ms-cli tag** (closes three tracked items in one event) — but it costs an outward-facing ms-cli publish and moving the frozen g6 anchor, so needs sign-off.
2. **codex32 vendor/fork vs accept.** The crate is abandoned; the ms share-string hygiene tail can't fully close until decided. Vendor+add-Drop closes it; accept-and-document parks it permanently. Which?
3. **Wire-shape gate option (b) — commit now, or keep paired-PR discipline only?** The Wave-3 keystone; the only thing that converts ungated `--json` surfaces from "tracked by discipline" to "caught by CI." Spans both repos.
4. **GUI MSRV / pin policy.** GUI v0.40.0+ needs rustc ≥1.88 vs toolkit MSRV 1.85, so a 1.85–1.87 install gets toolkit+codecs but fails the GUI step. Should the drift-check also gate/surface the GUI MSRV, and is raising the documented install prerequisite acceptable?

---

## 9. Adversarial review pass (verdict + folds)

An independent opus architect reviewed this plan against the 5 FOLLOWUPS registries + live source. **Verdict: 0 Critical / 3 Important / 5 Minor.** The load-bearing conclusions were *verified*: the funds-safety headline ("no open item can currently emit a wrong address or silently mis-build a wallet") is **TRUE** (every funds entry is RESOLVED, loud-refusal-BLOCKED, WONTFIX, or a label/diagnostic residual — no silent-wrong-output path in any registry); the fmt-gate red state reproduces exactly; every §7 BLOCKED classification (miniscript >13.1.0 w/ PR #953+#910, the zeroize cluster, codex32 dormant, vendor firmware) is genuine; the §6 catalog-only constraint is honored; and the fmt-gate-first wave sequencing has no dependency inversions. The three Important findings were factual corrections (folded into the body above):

- **I-1 (folded, §4 row 7 / Wave-4):** the plan scheduled an emit-time unrestorable advisory that **already shipped** — `bundle-unrestorable-shape-advisory` (v0.57.1) fires at engrave time on bundle + import-wallet for all 3 unrestorable shapes. Reframed to "mostly RESOLVED; remainder = reconstruction (row 6, partly upstream-gated)"; dropped the dead advisory work.
- **I-2 (folded, Wave 2):** the secret-hygiene severity ladder was mis-ranked — `ms-cli-derive-xpriv-master-not-zeroized` holds the **master/root Xpriv** (strictly above an account Xpriv) and was absent. Added its in-repo leg as the new highest-value Wave-2 residual (full close upstream-BLOCKED, §7.2); downgraded the account-Xpriv row to "high-value"; corrected the ordering rationale.
- **I-3 (folded, §5.3):** `gui-schema-mirror-lockstep-discipline` was listed as a permanent tripwire but is **RESOLVED** (one-time docs-codification). Removed; the bucket has 4 genuine tripwires (the real permanent GUI-schema tripwire `mnemonic-gui-schema-mirror` was already row 2).

Minors folded: **M-1** softened the unverifiable "57/58" figure; **M-2** noted the audit backlog's one WONTFIX; **M-3** tightened the fmt file count to "79 hunks / ~29 files (~28 non-exempt + mlock)"; **M-4** added the `gui-import-wallet-env-var-secret-channel` co-lander to the Wave-2 modal-redaction row. **M-5 (noted, no body change needed):** md's `…stable-rust-1-95-toolchain-fmt-clippy-drift` is RESOLVED and mk's `…rustfmt-drift…` is RESOLVED — neither is open sibling fmt work; the plan asserts all 5 repos in-sync at the 2026-06-22 reconcile, which implies the md test-hardening branch that cleared md's latent-red state has merged (confirmed: descriptor-mnemonic origin/main = `f18a027`, the reconciled tip).

**Post-fold status: the plan converges (0 Critical / 0 Important remaining after folds).** The review verbatim is preserved in `design/agent-reports/` per project discipline.
