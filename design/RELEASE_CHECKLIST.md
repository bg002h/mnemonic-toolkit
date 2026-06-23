# Release checklist (m-format constellation)

Manual cross-repo discipline (plus the toolkit per-release ritual,
below) that the automated CI gates can't fully reach. Run through this
BEFORE pushing the release tag for any constellation repo.

The CI gate at `.github/workflows/install-pin-check.yml` handles the
**toolkit self-pin** automatically (tag-event check: fails the tag
push if `scripts/install.sh` lags). This checklist covers the
**cross-repo pins** that no single repo's CI can verify without
network calls into sibling repos.

## Toolkit per-release ritual (every `mnemonic-toolkit-v*` tag)

1. **CHANGELOG.md** — add the `## mnemonic-toolkit [X.Y.Z] — <date>` section
   in the release commit (CI-gated: `changelog-check.yml` fails the tag push
   without it; lapsed silently for v0.48.0–v0.51.0 — don't trust habit).
2. Version bump sites in ONE commit: `Cargo.toml` + `Cargo.lock` + both
   README `<!-- toolkit-version -->` markers + `scripts/install.sh` self-pin
   (CI-gated: `install-pin-check.yml`).
3. Full suite AFTER the bump; push; ALL master CI green; THEN tag.

## install.sh component table (`scripts/install.sh:29-49`)

Five pins live in the toolkit's `component_info` shell table. The CI
gate guards row 1 (mnemonic-toolkit). Rows 2-5 need manual discipline:

| Component | Pin updated by | When |
|---|---|---|
| mnemonic-toolkit (self) | **CI gate** (auto) | every tag push |
| md-cli | this checklist | when sibling repo `descriptor-mnemonic` cuts a new `descriptor-mnemonic-md-cli-v*` tag |
| ms-cli | this checklist | when sibling repo `mnemonic-secret` cuts a new `ms-cli-v*` tag |
| mk-cli | this checklist | when sibling repo `mnemonic-key` cuts a new `mk-cli-v*` tag |
| mnemonic-gui | this checklist | when sibling repo `mnemonic-gui` cuts a new `mnemonic-gui-v*` tag |

## Pre-release ritual (any repo)

Before pushing a release tag from ANY of the 5 constellation repos,
audit cross-repo pin lag:

```sh
# Latest released tag per repo (run from anywhere with gh CLI):
gh release list -R bg002h/mnemonic-toolkit  --limit 1
gh release list -R bg002h/descriptor-mnemonic --limit 1
gh release list -R bg002h/mnemonic-secret   --limit 1
gh release list -R bg002h/mnemonic-key      --limit 1
gh release list -R bg002h/mnemonic-gui      --limit 1

# Current install.sh pins:
grep -oE '(mnemonic-toolkit|descriptor-mnemonic-md-cli|ms-cli|mk-cli|mnemonic-gui)-v[0-9.]+' \
  /scratch/code/shibboleth/mnemonic-toolkit/scripts/install.sh
```

> The `mnemonic-gui` pin specifically is also watched continuously by
> `.github/workflows/gui-pin-drift-check.yml` (warn-only, daily cron) —
> it ::warning::s when `scripts/install.sh:44` lags the latest released
> `mnemonic-gui-v*` tag. (md/ms/mk pins remain manual-audit-only.)

If any pin LAGS the latest release tag for that component:

1. Bump `scripts/install.sh` line 32 / 35 / 38 / 41 / 44 to the latest tag.
2. Commit with message: `chore(install): bump <component> pin v<old> -> v<new>`
3. Push BEFORE creating the next release tag.

## Per-repo release ritual

When cutting a new release of YOUR repo, also check whether
install.sh in the TOOLKIT repo needs an update for your repo's pin:

- **Toolkit release**: CI gate handles self-pin. Nothing else manual on the install.sh side.
- **GUI release**: Update toolkit `scripts/install.sh:44` `mnemonic-gui` pin in lockstep.
- **md/ms/mk-cli release**: Update the corresponding toolkit `scripts/install.sh:35/38/41` pin in lockstep.

Cross-repo PRs aren't required (install.sh is a 1-line change); a
follow-up commit to toolkit master is sufficient.

## Historical drift

- 2026-05-16: install.sh was 4-5 releases stale for mnemonic-toolkit
  (v0.14.2 vs v0.18.0) AND mnemonic-gui (v0.4.2 vs v0.7.1). Fixed at
  toolkit `7e0b846`. The CI gate + this checklist filed in the same
  cycle to prevent recurrence.
