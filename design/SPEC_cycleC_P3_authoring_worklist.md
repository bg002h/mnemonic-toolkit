# SPEC_cycleC_P3 — LIVE per-tab anchor authoring work-list (manual-gui v1.1)

**Doc class:** P3-P0 deliverable — the authoring backlog the per-tab phases
(P1–P6) burn down. Derived **live** by running the lint's own
`tests/extract_gui_schema.py` + `check_gui_schema_coverage.py::build_expected`
+ `check_outline_coverage.py::expected_outlines` against two clones:

- `mnemonic-gui-v0.3.0` (SHA verified) — the v1.0 manual baseline.
- `mnemonic-gui-v0.49.0` = `eecdb275e186defb03fc25f98662fcc4b2ebbf3f` — the v1.1 pin target.

**Re-derive at every later phase** — these counts decay if the pin moves.
Re-run: `python3 tests/extract_gui_schema.py --upstream-root <v0.49.0-clone> --out /tmp/inv.json`.

---

## §0. Grand-total reconciliation (EXACT, verified live)

| metric | v0.3.0 | v0.49.0 | delta |
|---|---:|---:|---:|
| total schema anchors | 459 | 965 | **+506** |
| outline targets | 59 | 128 | **+69** |

Per-tab anchor delta (sum = +506) and outline delta (sum = +69):

| tab | anchors v0.3.0→v0.49.0 | Δanchors | outlines | Δoutlines | new subs |
|---|---|---:|---|---:|---:|
| **mnemonic** | 333→758 | **+425** | 41→97 | **+56** | 20 |
| **ms** | 47→103 | **+56** | 6→15 | **+9** | 4 |
| **mk** | 24→42 | **+18** | 3→6 | **+3** | 3 |
| **md** | 55→62 | **+7** | 9→10 | **+1** | 1 |

All four per-tab figures match the SPEC §2.1 exactly. (Reconciled by
`build_expected` + `expected_outlines`, not copied.)

**Live lint state after P0** (against the v0.49.0 clone, P0 having authored
**0** new anchors — only the pin bump + prose + orphan reconcile):

- `gui-schema-coverage`: **497 missing anchors, 0 orphans.** (Direction-B
  orphan check is GREEN — the P0 import-wallet reconcile closed it.) The
  missing count is 497 (not 506) because the **pre-existing**
  `4c-import-wallet.md` chapter already authored ~9 of the now-schema'd
  import-wallet anchors correctly (`--blob`, `--ms1`, `--slot`, `--json`,
  `--format` + its 2 documented variants, `--select-descriptor`, the sub
  anchor + outline). So the AUTHORING backlog is **497 schema anchors**.
- `outline-coverage`: **67 missing outlines + 11 bullet-count mismatches =
  78 outline fixes** (the in-place growth chapters' `## Outline` bullet
  counts are stale — see §3).

This is the expected mid-modernization RED. The branch stays RED until PE
closes all 497 + 78.

---

## §1. ANCHOR-ID + OUTLINE derivation rules (for mechanical authoring)

```
anchor(sub)     = "<tab>-" + kebab(sub-name)
anchor(flag)    = anchor(sub) + "-" + flag-name.lstrip("-")
anchor(variant) = anchor(flag) + "-" + kebab(variant)
kebab(v)        = lower; [^a-z0-9]+ → "-"; collapse "-+"; strip leading/trailing "-"
```

Outline rules (STRICT bullet count, column-0 `-`/`*` only):

- sub with **≥2 flags** → `### Outline {#<tab>-<sub>-outline}` with **exactly
  F top-level bullets** (one per flag, schema order).
- flag of kind `Dropdown`/`NodeValueComposite`/`TaggedOrIndexed` with **≥2
  variants** → `#### Outline {#<flag>-outline}` with **exactly V bullets**.

Generate the outline bullet list MECHANICALLY from the schema flag list —
the wrong-count failure (got N±1) is the #1 failure mode.

---

## §2. NEW-subcommand work-list (per-sub flag / anchor / outline budget — LIVE)

Format: `sub: F flags, A anchors, O outlines; flag-outlines=[(flag, V), …]`.
Anchors/outlines are the EXACT schema-derived counts the new chapter must emit.

### §2.1 mnemonic tab — 20 NEW subcommands (+425 anchors total incl. §3 growth)

`mnemonic` = 84% of the new anchors → split across P4/P5/P6.

| subcommand | flags | anchors | outlines | flag-outlines (≥2-variant enum flags) |
|---|---:|---:|---:|---|
| `restore` | 25 | **64** | 6 | `--format`/11v, `--template`/10v, `--network`/4v, `--language`/10v, `--search-chain`/3v |
| `build-descriptor` | 17 | 35 | 5 | `--archetype`/6v, `--allow`/5v, `--format`/2v, `--network`/4v |
| `xpub-search-account-of-descriptor` | 15 | 30 | 3 | `--language`/10v, `--network`/4v |
| `xpub-search-passphrase-of-xpub` | 15 | 30 | 3 | `--language`/10v, `--network`/4v |
| `xpub-search-path-of-xpub` | 14 | 29 | 3 | `--language`/10v, `--network`/4v |
| `import-wallet` (RECONCILE — see §4) | 13 | 27 | 3 | `--format`/9v, `--network`/4v |
| `addresses` | 11 | 26 | 3 | `--network`/4v, `--language`/10v |
| `ms-shares-split` | 7 | 23 | 4 | `--separator`/3v, `--from`/2v, `--language`/10v |
| `ms-shares-combine` | 6 | 23 | 4 | `--separator`/3v, `--to`/3v, `--language`/10v |
| `xpub-search-address-of-xpub` | 8 | 17 | 3 | `--address-type`/4v, `--network`/4v |
| `nostr` | 10 | 15 | 2 | `--network`/4v |
| `silent-payment` | 10 | 15 | 2 | `--network`/4v |
| `verify-message` | 7 | 11 | 2 | `--format`/3v |
| `seedqr-decode` | 4 | 8 | 2 | `--variant`/2v |
| `seedqr-encode` | 3 | 7 | 2 | `--variant`/2v |
| `repair` | 6 | 7 | 1 | none (sub-outline only) |
| `inspect` | 5 | 6 | 1 | none (sub-outline only) |
| `electrum-decrypt` | 5 | 6 | 1 | none (sub-outline only) |
| `compare-cost` | 5 | 6 | 1 | none (sub-outline only) |
| `decode-address` | 1 | 2 | 0 | none (1 flag → NO sub-outline) |

### §2.2 ms tab — 4 NEW subcommands (+56 anchors total incl. §3 growth)

| subcommand | flags | anchors | outlines | flag-outlines |
|---|---:|---:|---:|---|
| `split` | 8 | 22 | 3 | `--separator`/3v, `--language`/10v |
| `derive` | 9 | 20 | 2 | `--language`/10v (note `--template`/`--network` are Dropdown 0v) |
| `combine` | 2 | 6 | 2 | `--to`/3v |
| `repair` | 2 | 3 | 1 | none (sub-outline only) |

### §2.3 mk tab — 3 NEW subcommands (+18 anchors total incl. §3 growth)

| subcommand | flags | anchors | outlines | flag-outlines |
|---|---:|---:|---:|---|
| `address` | 6 | 7 | 1 | none (`--address-type`/`--chain`/`--network` all Dropdown 0v) |
| `derive` | 3 | 4 | 1 | none (sub-outline only) |
| `repair` | 1 | 2 | 0 | none (1 flag → NO sub-outline) |

### §2.4 md tab — 1 NEW subcommand (+7 anchors total incl. §3 growth)

| subcommand | flags | anchors | outlines | flag-outlines |
|---|---:|---:|---:|---|
| `repair` | 1 | 2 | 0 | none (1 flag → NO sub-outline) |

---

## §3. EXISTING-chapter GROWTH work-list (in-place backfill — NOT new files)

Pre-existing chapters whose subcommands gained flags between v0.3.0 and
v0.49.0. Backfill new `## --flag {#…}` sections + bump the `## Outline`
bullet counts. The §0 per-tab anchor totals INCLUDE this growth.

| tab/chapter | sub | +anchors | +outlines | +flags |
|---|---|---:|---:|---|
| mnemonic/42-bundle.md | bundle | +10 | +2 | `--group-size`, `--import-json`, `--import-json-index`, `--md1-form`, `--separator` |
| mnemonic/43-verify-bundle.md | verify-bundle | +14 | +1 | `--accept-search-time`, `--cosigner`, `--expect-wallet-id`, `--from`, `--origin`, `--own-account-max`, `--search-addr-max`, `--search-addr-min`, `--search-address`, `--search-chain`, `--search-cosigner-subset` |
| mnemonic/45-export-wallet.md | export-wallet | +8 | +1 | `--bsms-form`, `--from-import-json`, `--from-import-json-index` |
| mnemonic/44-convert.md | convert | +6 | +1 | `--group-size`, `--separator` |
| md/53-encode.md | encode | +5 | +1 | `--group-size`, `--separator` |
| ms/63-encode.md | encode | +5 | +1 | `--group-size`, `--separator` |
| mk/73-encode.md | encode | +5 | +1 | `--group-size`, `--separator` |

**Live outline-mismatch backlog (11) — the existing `## Outline` bullet
counts that go stale at re-pin (must bump to the schema flag count):**

| outline | expected bullets | got |
|---|---:|---:|
| `#mnemonic-verify-bundle-outline` | 28 | 17 |
| `#mnemonic-bundle-outline` | 20 | 15 |
| `#mnemonic-convert-outline` | 19 | 17 |
| `#mnemonic-export-wallet-outline` | 18 | 15 |
| `#md-encode-outline` | 13 | 11 |
| `#mk-encode-outline` | 11 | 9 |
| `#mnemonic-export-wallet-format-outline` | 11 | 8 |
| `#mnemonic-convert-from-outline` | 14 | 13 |
| `#mnemonic-import-wallet-outline` | 13 | 7 |
| `#mnemonic-import-wallet-format-outline` | 9 | 2 |
| `#ms-encode-outline` | 7 | 5 |

(The two `import-wallet` mismatches are the §4 backfill leg — they resolve
when the missing import-wallet flags are authored.)

---

## §4. import-wallet RECONCILE — what P0 did vs what P4 must finish

**P0 (DONE, this phase):** closed the orphan-direction.

- RENAMED 5 prose/walkthrough anchors out of the `mnemonic-import-wallet-`
  schema-shaped namespace (so they are orphan-exempt):
  - `mnemonic-import-wallet-env-var-channel` → `iw-env-var-channel`
  - `mnemonic-import-wallet-no-auto-repair` → `iw-no-auto-repair`
    (`--no-auto-repair` is **NOT** an import-wallet flag at v0.49.0 — verified
    absent from the schema; it is a documentary prose section here.)
  - `mnemonic-import-wallet-walkthrough-bsms` → `iw-walkthrough-bsms`
  - `mnemonic-import-wallet-walkthrough-core` → `iw-walkthrough-core`
- DELETED the `--select-descriptor` enumerated-dropdown anchors + outline.
  At v0.49.0 `--select-descriptor` is a **0-variant** flag (the schema source
  comment calls it `Text`; the extractor resolves it to `Dropdown` with 0
  variants — either way it yields ONE flag anchor and NO outline/variants).
  The three variant anchors (`-all`, `-active-receive`, `-active-change`) +
  `-select-descriptor-outline` are gone; the flag is now plain prose with a
  bullet list.

**P4 (TODO) — BACKFILL the 7 import-wallet flags the chapter is MISSING:**
the live schema attaches 13 flags (sub-outline expects 13; chapter has 7). Add:

- `--format` — **9 variants** at v0.49.0 (chapter documents only `bsms` +
  `bitcoin-core`; ADD `coldcard`, `coldcard-multisig`, `descriptor`,
  `electrum`, `jade`, `sparrow`, `specter`). Its `#mnemonic-import-wallet-format-outline`
  must list **9** bullets (currently 2).
- `--network`/4v (new flag-outline).
- `--bsms-encryption-token`, `--bsms-round1`, `--bsms-verify-strict`.
- `--decrypt-password`, `--decrypt-password-file`, `--decrypt-password-stdin`.

Also (P4 carry-forward, NOT a gated anchor): the `4c-import-wallet.md` prose
still carries the v0.11.0-era "run-confirm modal renders argv verbatim"
secret-redaction claim (`--ms1` section + walkthrough). It is now stale (the
`••••` redaction shipped at GUI v0.39.0); reword it in P4 when the chapter is
fully reconciled, consistent with the §4-prose fix already applied to chapters
11/14/32/42/84 in P0. (Left untouched in P0 to avoid editing a chapter
mid-reconcile.)

---

## §5. AUTHORING RECIPE (guarantees the per-tab subset closes)

Per the SPEC §7. Each chapter mirrors GUI form-fields ↔ the CLI-manual flag
surface (`docs/manual/src/40-cli-reference/{41-mnemonic,42-md,43-ms,44-mk-cli}.md`).

1. Emit the anchor skeleton from the schema entry: `# <sub> {#<tab>-<sub>}`
   → `## Outline {#<tab>-<sub>-outline}` (exactly F bullets if F≥2) → one
   `## --flag {#…}` per flag → per enum flag with V≥2, a
   `### Outline {#<flag>-outline}` (exactly V bullets) + one
   `### <variant> {#<flag>-<variant>}` per variant.
2. Fill each flag body from CLI-manual prose (type/required/default,
   XOR-stdin partners, secret advisory now in the `••••` redaction framing).
3. Cross-link shared enums (`--network`/`--language`/`--template`) to a
   canonical chapter rather than re-prosing.
4. ONE worked example per chapter using only the canonical all-`abandon` seed,
   opened with the `:::danger` never-fund admonition.
5. `\index{<tab> <sub>}` first use + a `99-index-table.md` row.
6. `make html && make lint MANUAL_GUI_UPSTREAM_ROOT=<v0.49.0-clone>`; confirm
   the chapter's `<tab>-<sub>-*` anchors show 0 missing + 0 orphan + 0
   outline-mismatch before commit.

---

## §6. Phase → subcommand mapping (from SPEC §5)

- **P1 — md** (+7 / +1): new `md repair` chapter + `53-encode.md`
  `--group-size`/`--separator` backfill + `md-encode-outline` 11→13 + overview.
- **P2 — mk** (+18 / +3): new `address`, `derive`, `repair` chapters +
  `73-encode.md` backfill + `mk-encode-outline` 9→11 + overview.
- **P3 — ms** (+56 / +9): new `split`, `combine`, `derive`, `repair` chapters
  + `63-encode.md` backfill + `ms-encode-outline` 5→7 + overview.
- **P4 — mnemonic restore/import-export family:** `restore` (64, biggest),
  `import-wallet` reconcile+backfill (§4), `build-descriptor`; + `42-bundle.md`,
  `43-verify-bundle.md`, `45-export-wallet.md`, `44-convert.md` §3 growth.
- **P5 — mnemonic xpub-search/addresses/SP family:** 4 `xpub-search-*`,
  `addresses`, `silent-payment`, `nostr`, `verify-message`, `decode-address`,
  `compare-cost`.
- **P6 — mnemonic seedqr/ms-shares/utility family:** `seedqr-encode`,
  `seedqr-decode`, `ms-shares-split`, `ms-shares-combine`, `inspect`, `repair`,
  `electrum-decrypt`; + `41-overview.md` final family-grouping rewrite.
- **PE — close-out:** full `make lint` 7/7 GREEN; `make pdf` glyph check
  (`••••` U+2022 in DejaVu Serif); whole-diff review; tag `manual-gui-v1.1.0`;
  flip `gui-run-confirm-modal-secret-redaction-manual-companion` → resolved.
