# Release history {#release-history}

Version history of the `mnemonic-gui` releases this manual is
pinned against. The manual itself ships under
`mnemonic-toolkit-v0.70.0+`; the GUI under `mnemonic-gui-v0.49.0+`.
Tags advance in lockstep when GUI surface (schema, conditional,
help-icon URL) changes in a way that affects the manual.

## `manual-gui-v1.1.0` — GUI pin `mnemonic-gui-v0.49.0` (this manual)

**Release date**: 2026-06-23.

**Highlights**:

- GUI pin advanced `mnemonic-gui-v0.3.0` → `mnemonic-gui-v0.49.0`
  (46 minor versions of accumulated surface). Purely additive
  documentation coverage: **+28 subcommand chapters/sections**,
  **+506 schema anchors** (459 → 965), **+69 outline targets**
  (59 → 128), 0 removed.
- New `mnemonic`-tab subcommand surfaces: `restore`,
  `build-descriptor`, `import-wallet`, four `xpub-search-*` variants,
  `addresses`, `ms-shares-split` / `ms-shares-combine`, `seedqr-encode`
  / `seedqr-decode`, `nostr`, `silent-payment`, `verify-message`,
  `decode-address`, `repair`, `inspect`, `electrum-decrypt`,
  `compare-cost`.
- New sibling-CLI subcommand surfaces: `ms split` / `combine` /
  `derive` / `repair`; `mk address` / `derive` / `repair`;
  `md repair`.
- Secret-redaction prose synced to the GUI's shipped `••••` sentinel
  (live since `mnemonic-gui-v0.39.0`): the run-confirm modal and the
  output-panel `argv:` echo mask secret VALUES; multi-row / slot
  secret rows mask per-row.
- The four implied CLI pin tags advanced in lockstep:
  `mnemonic-toolkit-v0.70.0`, `descriptor-mnemonic-md-cli-v0.7.0`,
  `ms-cli-v0.8.0`, `mk-cli-v0.9.0`.

**Schema changes since v0.3.0**: the display-grouping
`--group-size` / `--separator` pair across the `encode` subcommands;
the `bundle` / `verify-bundle` / `export-wallet` flag growth (template,
import-json, BSMS-form, own-account-search); and the full set of new
subcommands listed above.

## v0.3.0 — `manual-gui-v1.0`

**Release date**: 2026-05-15 (toolkit `manual-gui-v1` HEAD ships
with the v0.3.0 GUI tag pinned).

**Highlights**:

- 5 new `mnemonic` subcommand surfaces wired into the GUI:
  `slip39-split` / `slip39-combine`, `seed-xor-split` /
  `seed-xor-combine`, `final-word`. Each ships its own
  per-tab chapter under [§40](#mnemonic-per-tab-reference).
- 4 `*-stdin` flags added to passphrase-bearing subcommands
  (`bundle`, `verify-bundle`, `convert`, `derive-child`).
- 2 latent v0.2 bug fixes: the repeating-secret argv routing,
  and the gui-schema-JSON-preferred schema_mirror for flattened
  names.
- Companion `manual-gui-v1.0` cycle ships this manual:
  ~34 chapters, ~155 PDF pages, 459 schema anchors covered by
  bidirectional lint.

**Schema changes since v0.2**:

- Added the `TaggedOrIndexed` `FlagKind` variant
  (`mnemonic export-wallet --taproot-internal-key`).
- Added the `NodeValueComposite` widget shapes for
  share-splitting subcommands.

## v0.2 — pre-`manual-gui-v1.0`

Pre-manual cycle. The GUI shipped with help-icon scaffolding but
no manual to deep-link into. Per-subcommand schema modules at
`schema/{mnemonic,md,ms,mk}.rs` reached source-of-truth status
for the four sibling CLIs.

The Phase D.1 help-audit report at
`design/agent-reports/v0_2-phase-D1-help-audit-r1.md` enumerated
every flag's provenance against the CLI's `--help` output. This
audit is the input to the manual's per-flag accuracy discipline.

## v0.1 — initial GUI release

The first end-to-end GUI: tab-switched form scaffolding, the
secret-handling widget set (SecretLineEdit, run-confirm modal,
paste-warn), and an initial schema covering the `mnemonic` and
`md` tabs. `ms` and `mk` joined in v0.2.

## Pinned-upstream tag set (this cycle)

The four CLI versions this v1.0 cycle of the manual is pinned to
are recorded at `docs/manual-gui/pinned-upstream.toml`:

- `mnemonic-toolkit-v0.13.0` — `mnemonic` CLI (the integration
  toolkit, the m-format constellation's umbrella crate).
- `descriptor-mnemonic-md-cli-v0.5.0` — `md` CLI (wallet-policy
  templates).
- `ms-cli-v0.2.1` — `ms` CLI (BIP-39 entropy backups).
- `mk-cli-v0.3.1` — `mk` CLI (xpub backups).

CI clones the pinned `mnemonic-gui` tag (currently
`mnemonic-gui-v0.3.0`) into a sibling checkout before running
the bidirectional schema-coverage lint, so the manual's HTML
build and the GUI's schema modules are tested for byte-identical
anchor parity at every PR.

## Future cycles (v1.1+)

The v1.1 cycle is bookmarked at the FOLLOWUPS entries:

- `gui-manual-cross-refs-to-cli-manual` — bidirectional
  cross-references between this manual and the
  `docs/manual/` CLI manual where concepts overlap (currently
  one-way: this manual references the CLI manual but not vice
  versa).
- `gui-manual-localization` — non-English translations of the
  GUI manual content.
- `cli-manual-html-target` — HTML render of the CLI manual to
  match this manual's pipeline; only PDF + Markdown today.
- `gui-help-icon-per-flag-affordance` — extend the
  per-subcommand / per-Dropdown / per-NodeValueComposite help
  icons to per-flag granularity if user feedback surfaces gaps.
- `mk-vectors-pretty-out-help-mismatch` — fix the upstream
  `mk-cli` help-text claim that `--pretty` is ignored under
  `--out` (source actually honors it). Lockstep with
  mnemonic-gui schema/help text.

## How to read the per-version `Pinned:` banners

Every per-tab chapter opens with a `Pinned: <name> <version>`
label that mirrors the GUI's top-of-form banner. The pinned
version is the upstream CLI tag this v1.0 cycle of the manual
was authored against. If you are running a different tag, see
[§82 binary and version mismatch](#binary-and-launch) for
diagnostic recipes.
