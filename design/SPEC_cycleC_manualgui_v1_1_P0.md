# SPEC_manual_gui_v1_1_modernization — Cycle-C P0

**Doc class:** P0 SPEC (single-author, R0-gated; NO authoring in this phase).
**Source SHAs (re-grep at every later phase — citations decay):**
- mnemonic-toolkit: `git rev-parse origin/master` at spec-write = the recon's `572a15d1` lineage (verify live at fold time).
- mnemonic-gui: pin target `mnemonic-gui-v0.49.0` = SHA `eecdb275e186defb03fc25f98662fcc4b2ebbf3f` (verified `git -C ../mnemonic-gui rev-list -n1 mnemonic-gui-v0.49.0`). GUI HEAD is `v0.49.0 + 2` (`f8edc72`), `Cargo.toml` version still `0.49.0` — so `v0.49.0` is the canonical current release tag to pin.

**Slug closed by this cycle:** `gui-run-confirm-modal-secret-redaction-manual-companion` (toolkit `design/FOLLOWUPS.md:1006`). Companion `gui-run-confirm-modal-secret-redaction` in `mnemonic-gui/FOLLOWUPS.md:716` is already `resolved` (v0.39.0; flipped 2026-06-22) — NO GUI source PR needed; the only cross-repo touch is *reading* the GUI at the pinned tag for schema extraction.

**Release:** `manual-gui-v1.1.0` — MINOR (purely additive coverage; 0 anchors removed from the schema side; 0 chapters deleted). Independent of the toolkit crate version (no toolkit re-tag).

---

## §0. Why this is a cycle, not a prose patch

The 2026-06-22 Wave-2 G1-B attempt (`758a44cc`, reverted at `a3ff1c3f`) proved the prose fix is **inseparable** from a GUI-pin bump: the honest-broken v0.3.0 prose is *correct* for the v0.3.0 GUI (which genuinely had no redaction), so the prose can only be corrected by re-pinning past v0.39.0. But `pinned-upstream.toml [mnemonic-gui]` is also the source the `gui-schema-coverage` lint clones — bumping the pin from v0.3.0 to v0.49.0 expands the documented-surface requirement by **+506 schema anchors + 69 outlines**, turning the manual-gui CI RED until every new anchor is documented. So this is the **manual-gui v1.1 modernization**: one cycle that (a) bumps the pin, (b) lands the prose fix, (c) documents the grown surface, gated GREEN only at PE.

Local-vs-CI footgun (carried from G1-B): `cspell` + `markdownlint` pass locally without a GUI checkout; only `gui-schema-coverage` (which needs the GUI clone) catches the anchor gap. Every content phase MUST run `make -C docs/manual-gui html && make -C docs/manual-gui lint MANUAL_GUI_UPSTREAM_ROOT=<gui-checkout-at-v0.49.0>` locally before commit.

---

## §1. THE EXACT PIN BUMPS (verified live)

`docs/manual-gui/pinned-upstream.toml` — current values (lines 18-25) and the v1.1 targets. **Source of the 4 implied CLI tags:** the *pinned GUI tag's own pin map*. At v0.49.0 the GUI's `pinned-upstream.toml` (schema-mirror map, `[mnemonic]/[md]/[ms]/[mk]` tables) is the authoritative record of which CLI versions its `src/schema/*.rs` was generated against. Verified via `git -C ../mnemonic-gui show mnemonic-gui-v0.49.0:pinned-upstream.toml`:

| pin field | OLD (v0.3.0 era) | NEW (v0.49.0 era) | authority for NEW |
|---|---|---|---|
| `[mnemonic-gui] tag` | `mnemonic-gui-v0.3.0` | `mnemonic-gui-v0.49.0` | canonical current GUI release tag |
| `[manual-gui] toolkit-tag-implied` | `mnemonic-toolkit-v0.13.0` | `mnemonic-toolkit-v0.70.0` | GUI v0.49.0 `pinned-upstream.toml [mnemonic] tag` |
| `[manual-gui] md-cli-tag-implied` | `descriptor-mnemonic-md-cli-v0.5.0` | `descriptor-mnemonic-md-cli-v0.7.0` | GUI v0.49.0 `[md] tag` |
| `[manual-gui] ms-cli-tag-implied` | `ms-cli-v0.2.1` | `ms-cli-v0.8.0` | GUI v0.49.0 `[ms] tag` |
| `[manual-gui] mk-cli-tag-implied` | `mk-cli-v0.3.1` | `mk-cli-v0.9.0` | GUI v0.49.0 `[mk] tag` |

**Comment-block rewrite (load-bearing):** the current `pinned-upstream.toml` comment (lines 9-12) says the 4 CLI tags mirror "lines 41/48/55 of [mnemonic-gui-v0.3.0]'s pinned-upstream.toml." That line-number citation is STALE — the GUI's `pinned-upstream.toml` format changed (it is now a `[mnemonic]/[md]/[ms]/[mk]` schema-mirror map with `tag = ...` fields, not lines 41/48/55). Rewrite the comment to: "The four CLI-tag fields below mirror exactly what the pinned `mnemonic-gui` tag (above) pins in its own `pinned-upstream.toml` `[mnemonic]/[md]/[ms]/[mk]` tables. A cycle bumping the GUI tag must re-pin all four in lockstep." (G1-B's `758a44cc` already drafted this comment for v0.48.1 — reuse it, swapping v0.48.1→v0.49.0; the 4 CLI tags are identical between v0.48.1 and v0.49.0.)

**Note:** G1-B pinned v0.48.1; this cycle pins v0.49.0. The 4 implied CLI tags are unchanged between v0.48.1 and v0.49.0 (toolkit-v0.70.0 / md-cli-v0.7.0 / ms-cli-v0.8.0 / mk-cli-v0.9.0), so only the `[mnemonic-gui] tag` differs from the G1-B draft.

**CHANGELOG version-site (gated):** `docs/manual-gui/CHANGELOG.md` MUST gain a `## [1.1.0] - <date>` section — the manual-gui release job (`manual-gui.yml` Job 3) feeds `CHANGELOG.md` as release notes. (Per `project_toolkit_release_ritual_version_sites` — CHANGELOG is an easily-missed version site.) Also update the `release-history` appendix `src/90-appendices/94-release-history.md`.

---

## §2. THE LIVE ANCHOR INVENTORY (re-derived, not copied)

Re-derived by running the lint's own extractor (`tests/extract_gui_schema.py::extract`) + `check_gui_schema_coverage.py::build_expected` + `check_outline_coverage.py::expected_outlines` against the v0.3.0 schema (`git show mnemonic-gui-v0.3.0:src/schema/*.rs`) and v0.49.0. **Reconciliation is EXACT** (sum of per-tab deltas = totals = recon figures).

| metric | v0.3.0 (manual baseline) | v0.49.0 (target) | delta |
|---|---:|---:|---:|
| subcommands | 28 | 56 | **+28** |
| flags | 161 | 407 | +246 |
| variants | 270 | 502 | +232 |
| **total schema anchors** | **459** | **965** | **+506** |
| outline targets | 59 | 128 | **+69** |

**Subset invariant holds:** v0.3.0 anchors ⊂ v0.49.0 anchors (0 anchors/outlines removed from the schema side) — the work is purely additive coverage. (BUT see §2.4 — the *authored* side has 7 orphan anchors that the re-pin exposes.)

### §2.1 Per-tab delta (the authoring concentration)

| tab | subs | flags | variants | anchors | outlines | new subcommands |
|---|---|---|---|---|---|---|
| **mnemonic** | 10→30 (+20) | 99→307 (+208) | 224→421 | 333→**758** (+425) | 41→97 (+56) | 20 (see §2.2) |
| **ms** | 5→9 (+4) | 12→35 (+23) | 30→59 | 47→**103** (+56) | 6→15 (+9) | split, combine, derive, repair |
| **mk** | 5→8 (+3) | 19→31 (+12) | 0→3 | 24→**42** (+18) | 3→6 (+3) | address, derive, repair |
| **md** | 8→9 (+1) | 31→34 (+3) | 16→19 | 55→**62** (+7) | 9→10 (+1) | repair |

`mnemonic` = **84% of the new anchors** (425/506) → it is split across 3 phases.

### §2.2 NEW-subcommand work-list (per-subcommand anchor + outline budget)

Format: `subcommand: F flags, A anchors, O outlines (1 sub-outline if F≥2; flag-outlines = #enum-flags-with-≥2-variants)`. Anchors/outlines are the EXACT schema-derived counts each new chapter must emit (verified live).

**mnemonic tab (20 new subcommands; +425 anchors incl. existing-chapter growth):**
- `restore`: 25 flags, **64 anchors**, 6 outlines (flag-outlines on `--format`/11v, `--template`/10v, `--network`/4v, `--language`/10v, `--search-chain`/3v). [largest new chapter]
- `build-descriptor`: 17 flags, 35 anchors, 5 outlines (`--archetype`/6v, `--allow`/5v, `--format`/2v, `--network`/4v).
- `xpub-search-account-of-descriptor`: 15 flags, 30 anchors, 3 outlines (`--language`/10v, `--network`/4v).
- `xpub-search-passphrase-of-xpub`: 15 flags, 30 anchors, 3 outlines (`--language`, `--network`).
- `xpub-search-path-of-xpub`: 14 flags, 29 anchors, 3 outlines (`--language`, `--network`).
- `addresses`: 11 flags, 26 anchors, 3 outlines (`--network`/4v, `--language`/10v). NOTE `--chain` is Dropdown 0v (no flag-outline).
- `import-wallet`: 13 flags, 27 anchors, 3 outlines (`--format`/9v, `--network`/4v). **SEE §2.4 — the `4c-import-wallet.md` chapter ALREADY EXISTS but must be RECONCILED, not greenfield-authored.**
- `xpub-search-address-of-xpub`: 8 flags, 17 anchors, 3 outlines (`--address-type`/4v, `--network`/4v).
- `ms-shares-split`: 7 flags, 23 anchors, 4 outlines (`--separator`/3v, `--from`/2v, `--language`/10v).
- `ms-shares-combine`: 6 flags, 23 anchors, 4 outlines (`--separator`/3v, `--to`/3v, `--language`/10v).
- `verify-message`: 7 flags, 11 anchors, 2 outlines (`--format`/3v).
- `nostr`: 10 flags, 15 anchors, 2 outlines (`--network`/4v).
- `silent-payment`: 10 flags, 15 anchors, 2 outlines (`--network`/4v).
- `seedqr-decode`: 4 flags, 8 anchors, 2 outlines (`--variant`/2v). NOTE `--from`=NodeValueComposite 1v (no flag-outline; <2 variants).
- `seedqr-encode`: 3 flags, 7 anchors, 2 outlines (`--variant`/2v).
- `repair`: 6 flags, 7 anchors, 1 outline (sub-outline only — all flags non-enum).
- `inspect`: 5 flags, 6 anchors, 1 outline (sub-outline only).
- `electrum-decrypt`: 5 flags, 6 anchors, 1 outline (sub-outline only).
- `compare-cost`: 5 flags, 6 anchors, 1 outline (sub-outline only).
- `decode-address`: 1 flag (`--json`), 2 anchors, **0 outlines** (1 flag → no sub-outline).

**ms tab (4 new):**
- `split`: 8 flags, 22 anchors, 3 outlines (`--separator`/3v, `--language`/10v).
- `derive`: 9 flags, 20 anchors, 2 outlines (`--language`/10v). NOTE `--template`/`--network` are Dropdown 0v.
- `combine`: 2 flags, 6 anchors, 2 outlines (`--to`/3v).
- `repair`: 2 flags, 3 anchors, 1 outline (sub-outline only).

**mk tab (3 new):**
- `address`: 6 flags, 7 anchors, 1 outline (sub-outline only — `--address-type`/`--chain`/`--network` all Dropdown 0v).
- `derive`: 3 flags, 4 anchors, 1 outline (sub-outline only).
- `repair`: 1 flag, 2 anchors, 0 outlines.

**md tab (1 new):**
- `repair`: 1 flag (`--json`), 2 anchors, 0 outlines.

### §2.3 EXISTING-chapter GROWTH work-list (backfill — do NOT overlook)

Pre-existing chapters whose subcommands gained flags between v0.3.0 and v0.49.0 — these need IN-PLACE anchor backfill (new `## --flag {#...}` sections + outline-bullet count bumps), NOT new files. The display-grouping `--group-size`/`--separator` pair (toolkit mstring-grouping cycle) landed across multiple `encode` subcommands:

| tab/chapter | sub | +anchors | +flags |
|---|---|---|---|
| mnemonic/42-bundle.md | bundle | +10 | `--group-size`, `--import-json`, `--import-json-index`, `--md1-form`, `--separator` |
| mnemonic/44-convert.md | convert | +6 | `--group-size`, `--separator` |
| mnemonic/45-export-wallet.md | export-wallet | +8 | `--bsms-form`, `--from-import-json`, `--from-import-json-index` |
| mnemonic/43-verify-bundle.md | verify-bundle | +14 | `--accept-search-time`, `--cosigner`, `--expect-wallet-id`, `--from`, `--origin`, `--own-account-max`, `--search-addr-max`, `--search-addr-min`, `--search-address`, `--search-chain`, `--search-cosigner-subset` |
| md/53-encode.md | encode | +5 | `--group-size`, `--separator` |
| ms/63-encode.md | encode | +5 | `--group-size`, `--separator` |
| mk/73-encode.md | encode | +5 | `--group-size`, `--separator` |

Plus each affected sub's `## Outline` bullet count and flag-outline count must increase (the `--separator` Dropdown/3v adds a `#### Outline`). The §2.1 per-tab anchor totals INCLUDE this growth (e.g. mnemonic +425 = new chapters + these in-place additions; reconciled exactly to 506 grand total).

### §2.4 ORPHAN reconciliation (the re-pin trap — critical, recon did not surface this)

`is_schema_shaped()` (check_gui_schema_coverage.py:101) treats any HTML anchor matching `<tab>-<sub>` or `<tab>-<sub>-…` (for a known schema `(tab,sub)` pair, excluding `-outline`) as schema-shaped and orphan-checkable. At v0.3.0, `import-wallet` is NOT in the schema → its anchors are not schema-shaped → the existing `4c-import-wallet.md` chapter's prose-section anchors pass silently. **At v0.49.0, `import-wallet` enters the schema** → 7 authored anchors in `4c-import-wallet.md` become orphans and FAIL the orphan-direction check:

```
mnemonic-import-wallet-env-var-channel              (authored prose section; no schema flag)
mnemonic-import-wallet-no-auto-repair               (authored prose section; no schema flag)
mnemonic-import-wallet-select-descriptor-all        (--select-descriptor is Text/0-variant at v0.49.0)
mnemonic-import-wallet-select-descriptor-active-receive
mnemonic-import-wallet-select-descriptor-active-change
mnemonic-import-wallet-walkthrough-bsms             (authored walkthrough section)
mnemonic-import-wallet-walkthrough-core             (authored walkthrough section)
```

**Reconciliation (in the mnemonic-tab phase that owns import-wallet):**
- The 5 prose/walkthrough/env-var/no-auto-repair anchors: RENAME to a non-schema-shaped prefix so they fall outside the orphan namespace (e.g. `{#iw-env-var-channel}`, `{#iw-walkthrough-bsms}`, `{#iw-walkthrough-core}`, `{#iw-no-auto-repair}`) and update any intra-doc links. (Anchors NOT starting with `mnemonic-import-wallet-` are exempt from the orphan check.)
- The 3 `select-descriptor-*` variant anchors + the `### Outline {#mnemonic-import-wallet-select-descriptor-outline}`: DELETE — `--select-descriptor` is `FlagKind::Text` (0 variants) at v0.49.0 (`mnemonic.rs:2560-2561`), so it gets ONE flag anchor (`mnemonic-import-wallet-select-descriptor`) and NO outline/variants. The chapter currently documents it as an enumerated dropdown — wrong against v0.49.0.
- BACKFILL the import-wallet flags the existing chapter is MISSING against v0.49.0: `--format` has 9 variants (chapter documents only `bsms`+`bitcoin-core`; add `coldcard`, `coldcard-multisig`, `descriptor`, `electrum`, `jade`, `sparrow`, `specter`), plus `--network`/4v, `--bsms-encryption-token`, `--bsms-round1`, `--bsms-verify-strict`, `--decrypt-password{,-file,-stdin}`, `--slot`, `--json`.

**Net authored-anchor accounting:** ADD 506 schema anchors + 69 outlines; REMOVE/RENAME 7 orphan anchors (all in 4c-import-wallet.md). Final state: 965 schema anchors all present, 0 orphans, 128 outlines correct.

### §2.5 Overview-chapter stale counts (4 files)

The per-tab overview chapters carry stale subcommand counts that must update (these are prose, not gated anchors, but they are wrong):
- `40-mnemonic/41-overview.md:4` "ten subcommands" → **30 subcommands** (and rewrite the 5-family grouping to cover the 20 new subs).
- `50-md/51-overview.md:3` "eight subcommands" → **9** (add `repair`).
- `60-ms/61-overview.md:3` "five subcommands" → **9** (add split/combine/derive/repair).
- `70-mk/71-overview.md:3` "five subcommands" → **8** (add address/derive/repair).

---

## §3. THE COVERAGE MECHANISM (exact, for authoring agents)

**Gate path:** `.github/workflows/manual-gui.yml` Job 1 `lint` → clones `mnemonic-gui` at `pinned-upstream.toml`'s `[mnemonic-gui] tag` (`git clone --depth 1 --branch "$PINNED_TAG"`) → `make html` → `make lint MANUAL_GUI_UPSTREAM_ROOT=<clone>`. `make lint` = `tests/lint.sh`, 7 phases; the two schema gates are **phase 4/7 gui-schema-coverage** and **phase 5/7 outline-coverage**.

**Anchor-id derivation (check_gui_schema_coverage.py:66-88, `build_expected`):**
```
anchor(subcommand) = "<tab>-" + kebab(sub-name)
anchor(flag)       = anchor(subcommand) + "-" + flag-name.lstrip("-")
anchor(variant)    = anchor(flag) + "-" + kebab(variant)
kebab(v)           = v.lower(); [^a-z0-9]+ → "-"; collapse "-+"; strip leading/trailing "-"
```
The schema flag/sub names are already lowercase-ascii-kebab, so the rule is identity for them; kebab matters only for variant strings with `/` or punctuation (e.g. `p2sh-p2wpkh` stays, `bitcoin-core` stays). `--flag` → `flag` (leading dashes stripped). The check collects EVERY `id="..."` from the rendered HTML (`build/m-format-gui-manual.html`, regex `id="([^"]+)"` — any element, not just headings) and:
- `missing = expected − found_ids` (direction A — every schema anchor needs an HTML id).
- `orphans = {a ∈ found − expected : is_schema_shaped(a)}` (direction B — every schema-shaped HTML anchor needs a schema entry; `-outline` anchors and non-`<tab>-<sub>`-prefixed anchors are EXEMPT).
- **Exit 0 iff `missing == ∅ AND orphans == ∅`.**

**Outline derivation (check_outline_coverage.py:66-93, `expected_outlines`):** reads MARKDOWN source (not HTML — fires pre-pandoc):
- Every subcommand with **≥2 flags** requires `### Outline {#<sub>-outline}` with **exactly N top-level bullets** (N = flag count).
- Every flag of kind `Dropdown`/`NodeValueComposite`/`TaggedOrIndexed` with **≥2 variants** requires `#### Outline {#<flag>-outline}` with **exactly V top-level bullets** (V = variant count).
- Bullet counting is STRICT: only column-0 `-`/`*` lines between the outline heading and the next heading; indented/nested bullets and fenced-code lines are excluded. Wrong count → `mismatch` failure.

**"Orphan" precisely:** an HTML `id=` that (a) is not `-outline`-suffixed, (b) matches a known `(tab,sub)` shape prefix, and (c) has no corresponding schema entry. This is what the §2.4 import-wallet prose anchors trip on at re-pin.

**Authored anchor convention (learned from `44-convert.md` + AUTHORING.md):** pandoc auto-slugs from heading TEXT, so explicit `{#anchor-id}` is MANDATORY after every heading (`## --from` would slug to `from`, not `mnemonic-convert-from`). Heading levels: `# {#<tab>-<sub>}` (H1, one per chapter file), `## --flag {#<tab>-<sub>-<flag>}` (H2), `### Outline {#...-outline}` for the sub, `### variant {#<flag>-<variant>}` (H3) under enum flags with `#### Outline {#<flag>-outline}` between flag and its variants. (The 44-convert chapter uses `##`/`###`/`####` — H2 sub-section / H3 flag-outline+variant; verify the exact level mapping against an existing chapter at author time since the `#`-depth and the `{#...}` anchor are decoupled.)

---

## §4. THE PROSE FIXES (vendor G1-B `758a44cc` as prior art)

The GUI redaction CODE shipped at **v0.39.0** and is live at v0.49.0 (verified): `SECRET_MASK = "••••"` (`src/form/invocation.rs:137`), `assemble_argv_with_secret_mask` (`:152`, parallel `(Vec<String>, Vec<bool>)`), `render_copy_command_masked` (`:524`); the run-confirm modal substitutes the mask per token (`src/main.rs:960`); AND the output-panel `argv:` echo is masked too (`src/main.rs:480`, "the last-run command-line is masked too — v0.39.0 Item 1 D3"). Only the manual prose lags.

**Prior art:** `git show 758a44cc` (the reverted G1-B attempt) already worked out the correct `••••` reword for chapters 14 + 11 + the pin bump. VENDOR it (it was reverted only because it lacked the +506-anchor coverage, not because the prose was wrong). G1-B targeted v0.48.1 — adapt to v0.49.0.

**Five stale-prose sites (recon named 2; live grep found 5 — this is a SPEC correction):**

1. **`10-foundations/14-secret-handling.md` Defense-2 (lines 79-114):** replace the `:::danger` "renders secret-bearing argv tokens in plaintext, NOT as `***` redactions" block + cold-node operational mitigation. Vendor G1-B's Defense-2 rewrite verbatim: (a) the modal redacts each secret value as `••••`; (b) "Residual exposure — flag names visible, only secret VALUE masked" caveat (the still-true residual — for composite `--from <node>=<value>` the whole token incl. prefix is masked); (c) DEMOTE cold/airgapped to "General hygiene (no longer load-bearing)" plain bold prose. **Use plain bold prose, NOT `:::note`/`:::warning`** — the `primer-box.lua` filter only styles `:::primer`/`:::danger`.
2. **`10-foundations/11-what-is-mnemonic-gui.md` feature-2 (lines 45-49):** vendor G1-B's revert — "The modal redacts secret values: each secret-bearing argv token renders as a fixed `••••` sentinel … flag NAME stays visible, only its secret value is masked."
3. **`30-tour/32-run-and-output.md` (lines 96-97, 134-145, 174-176):** NOT in G1-B's diff — author fresh. (a) line 96 "the unredacted-modal operational warning" → reword to redaction-present framing; (b) lines 134-145 "At v0.3.0 the modal renders the secret-bearing argv tokens in plaintext … redaction gap is tracked at FOLLOWUP" → replace with the `••••` masked-modal description; the ASCII-art modal at lines 117-130 should show `phrase=••••` (or `••••`) instead of `phrase=abandon abandon …`; (c) **lines 174-176 "It does not redact secrets in the `argv:` echo line"** is ALSO STALE (the output panel argv-echo is masked at `main.rs:480`) → correct to "the `argv:` echo line IS masked (same `••••` sentinel)."
4. **`80-troubleshooting/84-secrets-and-os.md` (lines 14-18, "Known limitations (v0.3.0)" table):** the "Run-confirm modal renders argv tokens in plaintext" limitation row is RESOLVED → remove it (or move to a "Resolved since v0.39.0" note). Re-title the section away from "(v0.3.0)". Keep the "Multi-row text widgets … do not auto-mask repeating secret-bearing rows" row ONLY if still true at v0.49.0 — verify against the GUI (the slot/repeating-secret mask shipped in cycle-15 Lane G per the GUI FOLLOWUP; likely also stale — author phase MUST re-verify `src/runner.rs` `PendingConfirm.mask` + slot masking at v0.49.0 and demote/remove accordingly).
5. **`40-mnemonic/42-bundle.md` (lines 17-21, `:::danger` block):** "the GUI's v0.3.0 run-confirm modal renders secret-bearing argv tokens in plaintext, including pasted BIP-39 phrases on the slot editor's secret-bearing rows" → reword to the redaction-present framing (keep the canonical-seed never-engrave/fund danger; only fix the plaintext-modal claim).

**cspell additions:** add `exfiltration`, `unredacted`, and `sentinel` (if not present) to `docs/manual-gui/.cspell.json` `words[]` (194 entries; G1-B added the first two). Verify the `••••` glyph (U+2022 BULLET) renders in the PDF font (DejaVu Serif) at the `make pdf` phase — if it fails like the v1.0.1 U+2715 case, fall back per the PDF-glyph rule.

**Defense-2 list (lines 4-22) is correct** (the schema-`secret: true` class list) — leave it; only the modal-behavior claim is stale.

---

## §5. THE PHASING + BRANCH/MERGE GATING

**Branch:** `manual-gui-v1.1-modernization` (off `master`). **The `gui-schema-coverage` gate is RED from the P0 pin-bump until PE closes all 506 anchors** — therefore ALL phases run on this dedicated branch; `master` is NEVER left with a RED manual-gui CI. Merge to `master` (fast-forward / no-squash) ONLY when PE's full `make lint` is 7/7 GREEN. Do not push intermediate phase commits to `master`.

Each phase: mandatory R0 SPEC/plan gate BEFORE authoring (0C/0I convergent loop), single-subagent TDD-style authoring in a worktree, per-phase `make -C docs/manual-gui html && make lint MANUAL_GUI_UPSTREAM_ROOT=<v0.49.0-clone>` checking **its tab's anchor subset closes** (the lint reports per-anchor, so a tab is "done" when its `<tab>-*` missing-set is empty even while other tabs remain RED), per-phase review persisted to `docs/manual-gui/agent-reports/`, then commit.

- **P0 (this SPEC + pin + prose + scaffolding):** land §1 pin bump + comment rewrite, §4 prose fixes (5 files + cspell), §2.5 overview count fixes, §2.4 import-wallet orphan RENAME/DELETE (close the orphan-direction failures up front so later phases only fight the missing-direction). After P0 the lint is RED on missing anchors (expected) but has 0 orphans. CHANGELOG `[1.1.0]` stub + release-history appendix.
- **P1 — md tab** (+7 anchors / +1 outline): `md repair` new chapter (`5a-repair.md`) + `53-encode.md` `--group-size`/`--separator` backfill + overview. Smallest — warm-up to validate the author→render→lint loop.
- **P2 — mk tab** (+18 / +3): new chapters `address`, `derive`, `repair` (`77-`/`78-`/`79-`) + `73-encode.md` backfill + overview.
- **P3 — ms tab** (+56 / +9): new chapters `split`, `combine`, `derive`, `repair` (`67-`/`68-`/`69-`/`6a-`) + `63-encode.md` backfill + overview.
- **P4–P6 — mnemonic tab** (+425 / +56), split by subcommand family:
  - **P4 — restore/import-export family:** `restore` (64 anchors, biggest), `import-wallet` reconcile+backfill (§2.4), `build-descriptor`; + `42-bundle.md`, `43-verify-bundle.md`, `45-export-wallet.md`, `44-convert.md` in-place growth (§2.3).
  - **P5 — xpub-search/addresses/silent-payment family:** the 4 `xpub-search-*` subs, `addresses`, `silent-payment`, `nostr`, `verify-message`, `decode-address`, `compare-cost`.
  - **P6 — seedqr/ms-shares/utility family:** `seedqr-encode`, `seedqr-decode`, `ms-shares-split`, `ms-shares-combine`, `inspect`, `repair`, `electrum-decrypt`; + `40-mnemonic/41-overview.md` final.
- **PE — close-out:** full `make -C docs/manual-gui lint` 7/7 GREEN (0 missing / 0 orphan / 0 outline-mismatch); `make html` + `make pdf` build clean (glyph check); glossary (phase 6) + index bidirectional (phase 7) pass for any new terms; whole-diff adversarial review; tag `manual-gui-v1.1.0`; flip `gui-run-confirm-modal-secret-redaction-manual-companion` → `resolved` in the shipping commit (the GUI companion is already resolved). The `manual-gui-v*` tag push triggers `manual-gui.yml` Job 3 (release + gh-pages publish).

---

## §6. SemVer + lockstep confirmation

- **`manual-gui-v1.1.0`** — MINOR (additive: +28 subcommand chapters/sections, +506 anchors, 0 schema anchors removed, 0 chapters deleted). Tag triggers the manual-gui release/gh-pages workflow.
- **No `schema_mirror` fire:** that gate (`mnemonic-gui/src/schema/...` ↔ toolkit clap surface) is unrelated to `docs/manual-gui/`; this cycle touches NO toolkit clap surface → no schema_mirror, no `docs/manual/src/40-cli-reference/` CLI-manual mirror, no toolkit re-tag.
- **No GUI source PR:** companion `gui-run-confirm-modal-secret-redaction` already `resolved` (v0.39.0). Only cross-repo touch = reading GUI v0.49.0 for schema extraction.
- **No codec/CLI publish:** the 4 implied CLI tags are documentary pin records, not dependency edges to bump in `Cargo.toml`.

---

## §7. AUTHORING APPROACH for per-tab chapters

Each GUI chapter mirrors **the GUI form-fields ↔ the corresponding CLI flag surface** — so an authoring agent produces a chapter from TWO live sources: (1) the v0.49.0 schema (the authoritative anchor/outline shape — run `python3 tests/extract_gui_schema.py --upstream-root <v0.49.0-clone> --out /tmp/inv.json` and read the sub's entry), and (2) the matching CLI-manual chapter under `docs/manual/src/40-cli-reference/` (`41-mnemonic.md` / `42-md.md` / `43-ms.md` / `44-mk-cli.md`) which already documents the same flags' semantics in prose. The chapter REFRAMES the CLI prose as GUI form-fields (widget per flag: text field / Dropdown / checkbox / NodeValueComposite / slot editor), not re-deriving facts.

**Per-chapter recipe (guarantees PE coverage passes):**
1. From the schema entry, emit the exact anchor skeleton: `# <sub> {#<tab>-<sub>}` → `## Outline {#<tab>-<sub>-outline}` with **exactly F bullets** (one `[`--flag`](#<tab>-<sub>-<flag>)` per flag, in schema order) when F≥2 → one `## `--flag` {#<tab>-<sub>-<flag>}` per flag → for each enum flag with V≥2 variants, a `### Outline {#<flag>-outline}` with **exactly V bullets** + one `### <variant> {#<flag>-<variant>}` per variant.
2. Fill each flag body from the CLI-manual prose: type/required/default, conditional partner (XOR stdin pairs), secret advisory (schema `secret: true` → `SecretLineEdit` + run-confirm note, now with the `••••` redaction framing from §4), and a GUI form-field framing sentence.
3. Cross-link variants/networks/templates to the canonical chapter (e.g. `--network` variants link to `mnemonic bundle --network <v>` like `44-convert.md` does) to avoid re-prosing shared enums.
4. One worked example per chapter using ONLY the canonical all-`abandon` seed, opened with the `:::danger` never-fund admonition (AUTHORING.md §"Worked-example seed").
5. Add `\index{<tab> <sub>}` on first use + a matching row in `99-index-table.md` (phase 7 bidirectional gate).
6. Run `make html && make lint MANUAL_GUI_UPSTREAM_ROOT=<v0.49.0-clone>`; confirm the chapter's `<tab>-<sub>-*` anchors show 0 missing + 0 orphan + 0 outline-mismatch before commit.

The bullet-count exactness (§3 outline rule) is the single most common failure mode — an agent that lists F-1 or F+1 bullets in `## Outline` fails phase 5. Generate the outline bullet list mechanically FROM the schema flag list, never hand-typed.

---

## §8. Open risks / verify-at-phase-time

- The `••••` (U+2022) glyph must render in the PDF font; verify at PE `make pdf` (v1.0.1 precedent: U+2715 rendered as `?`).
- `84-secrets-and-os.md` "multi-row widgets do not auto-mask" row: re-verify against GUI v0.49.0 `src/runner.rs`/slot-editor masking (cycle-15 Lane G shipped slot/repeating masking) — likely stale; demote/remove if so. Do NOT assume; grep the GUI at author time.
- All schema citations decay: every phase re-runs `extract_gui_schema.py` against the v0.49.0 clone rather than trusting this SPEC's snapshot counts.