# PLAN — README subcommand-inventory refresh (de-version, de-count, complete)

**Status:** R0 GREEN (round 2, 0C/0I; 3 polish minors folded) — implementation may begin
**Source grounding verified at:** toolkit `origin/master` = `bc373ed` (2026-06-10)
**Resolves:** `design/FOLLOWUPS.md::readme-subcommand-inventory-stale-4-sites`
**Recon:** `cycle-prep-recon-readme-subcommand-inventory.md` (untracked; zero drift, ground truth = 24 subcommands at v0.52.0)
**Shape:** docs-only, no bump, no tag, one commit (2 README files + FOLLOWUP flip).

## 0. Posture (the load-bearing decision)

**No hardcoded subcommand count and no prose version string survives anywhere in either README.** The `<!-- toolkit-version -->` markers (gated by the release ritual + install-pin-check arc) are the ONLY version surface; the manual (`docs/manual/`, lint-gated against live `--help`) and `--help` itself are the ONLY authoritative inventories. README inventories are a courtesy map — grouped, complete TODAY, and allowed to lag content-wise but never lying about a number. (A count gate over README prose would be disproportionate; deletion is the staleness-proofing.)

## 1. `README.md:14` (root "Status:" opener — long variant)

Replace the opener clause `Status: **v0.43.x** — twenty-three \`mnemonic\` subcommands (see [Subcommands](#subcommands)).` with (R0-r1 C1 — the version markers are INVISIBLE HTML comments; point at a visible surface):

> `Status: the \`mnemonic\` CLI (see [CHANGELOG.md](CHANGELOG.md) for the current release; subcommands grouped under [Subcommands](#subcommands)) spans …` — merging into the existing sentence so it stays one sentence.

…and extend the capability list in the SAME sentence (it currently stops at the v0.43-era feature set): after "single-sig (BIP-44/49/84/86) + multisig + BIP-388 descriptors + multi-leaf taproot + multi-source full multisig;" insert (and add "via ms-shares" to the root splitting clause — R0-r2 M-r2-2):

> `guided descriptor CONSTRUCTION (\`build-descriptor\`: a validated policy-tree → wsh descriptor engine with 5 archetype presets and a reviewed \`--allow\` sanity opt-out);`

(All claims verifiable against the shipped v0.50.0–v0.52.0 surface. **R0-r1 I2: the restore clause is STALE in both openers** — "watch-only single-sig restore documents…" predates v0.44.0 multisig + v0.49.1 taproot; replace in BOTH with: `watch-only restore documents (single-sig from a seed + passphrase, fingerprint-gated; multisig from the shared md1 card alone, incl. taproot NUMS)`. The remaining clauses verified accurate: conversions, splitting, BIP-85/352, nostr, verify, decode, BCH; the 7-format import/export list is a non-exhaustive subset of 8 — acceptable understatement.)

## 2. `README.md:44` ("Twenty-one … grouped below") + the grouped inventory `:48-54`

- `:44`: `Twenty-one \`mnemonic\` subcommands, grouped below.` → `The \`mnemonic\` subcommands, grouped below.`
- Inventory completion (3 missing; placements per R0-r1 I1/I3 — the inline list at crates README:32 is a 1:1 group mirror and MUST agree):
  - **Backup splitting** group gains: `` `ms-shares` (BIP-93 codex32 ms1 K-of-N share split/combine)``
  - **Wallet import / export** group gains (R0-r1 I3 — manual precedent: restore sits between export-wallet and import-wallet; `restore --format` IS an export-wallet emitter): `` `restore` (watch-only restore documents — single-sig from a seed + passphrase, fingerprint-gated; multisig from the shared md1 card alone, incl. taproot NUMS) `` (R0-r1 M1: two parallel sources, no "from a seed" umbrella)
  - NEW group line placed AFTER the Decrypt/repair/inspect group (R0-r1 I1 — manual orders build-descriptor last): `- **Descriptor construction** — \`build-descriptor\` (versioned JSON policy-tree or archetype preset → funds-safety-gated wsh descriptor + BIP-388 + cost preview; \`--allow\` reviewed sanity opt-out — cost preview unavailable on an overridden emit).`

## 3. `crates/mnemonic-toolkit/README.md:10` ("Status:" opener — short variant)

`Status: **v0.43.x** — twenty-three \`mnemonic\` subcommands spanning` → `The \`mnemonic\` subcommands span` (R0-r1 C1/M3 — the paragraph already ends with a visible CHANGELOG pointer; no version deixis, full sentence); and in its capability run-list insert `guided descriptor construction (build-descriptor: policy-tree/archetype presets → gated wsh descriptors),` after the wallet import/export clause. Add `ms1 K-of-N shares (ms-shares),` to the backup-splitting clause if absent (verify at edit time — the clause currently reads "backup splitting (seed-XOR, SLIP-39, BIP-93 codex32 K-of-N shares, SeedQR)"; note BIP-93 codex32 IS ms-shares' mechanism — reword to "(seed-XOR, SLIP-39, BIP-93 codex32 K-of-N shares via ms-shares, SeedQR)" rather than double-listing).

## 4. `crates/mnemonic-toolkit/README.md:32` (inline "Twenty-one" list)

`Twenty-one \`mnemonic\` subcommands — \`bundle\` / … / \`gui-schema\`.` → `The \`mnemonic\` subcommands — ` + the SAME inline list with the 3 additions mirroring the grouped inventory (R0-r1 I1): `restore` joins the wallet slash-group (after `export-wallet`); `ms-shares` after `slip39`; `build-descriptor` as its own comma-group after the `xpub-search` group, before the final `and \`gui-schema\``. (24 names total — verified against the binary's Commands block at `bc373ed`.)

## 4b. Manual intro inventory (R0-r1 I4 — fixed in the SAME commit)

The manual chapter intro (`docs/manual/src/40-cli-reference/41-mnemonic.md:3-19`) says "**Twenty** subcommands:" and lists 21, omitting `addresses`, `restore`, `build-descriptor` — the lint gate does not cover that paragraph, and the new README text calls the manual authoritative. Apply the same posture there: de-count ("The `mnemonic` subcommands:") and add the 3 missing names at pinned positions (R0-r2 M-r2-1): `restore` between `export-wallet` and `import-wallet`; `addresses` before `decode-address`; `build-descriptor` before the terminal `gui-schema`. **This makes the cycle touch `docs/manual/` → run the FULL manual lint (incl. cspell) with all four binaries before pushing.**

## 5. FOLLOWUP flip + verification

- Flip `readme-subcommand-inventory-stale-4-sites` → resolved (posture: counts never return; manual/`--help` authoritative).
- Verify: `grep -rn "wenty" README.md crates/mnemonic-toolkit/README.md docs/manual/src/40-cli-reference/41-mnemonic.md` → 0 hits (the manual intro included — I4); `grep -rn "v0\.43" …` → 0 hits; `grep -n "single-sig restore" README.md crates/mnemonic-toolkit/README.md` → 0 hits (R0-r1 M4); every one of the 24 binary subcommand names appears in BOTH grouped/inline inventories (a one-shot shell check in the commit message, not a CI gate); full test suite unaffected (docs-only) but run per ritual; **FULL manual lint required** (the I4 manual-intro edit; READMEs alone wouldn't need it). Carve-out (R0-r1 M2): `README.md:20`'s "all 5 m-format constellation components" is a COMPONENT count (installer-pinned, different churn) — it stays; the no-counts posture is scoped to SUBCOMMAND counts.

---

## Fold log

- **R0 round 1 (YELLOW → folded, 2026-06-10; persisted at `design/agent-reports/readme-inventory-refresh-r0-r1-review.md`):** C1 the "see the version marker above" deixis pointed at an invisible HTML comment → visible CHANGELOG/Subcommands pointers. I1 build-descriptor placement unified (own group AFTER Decrypt/repair/inspect — manual orders it last; inline mirror agrees). I2 the stale "single-sig restore" clause in BOTH openers extended to the v0.44.0/v0.49.1 surface (the plan had certified it accurate). I3 restore → Wallet import/export group (manual precedent + restore --format IS an export-wallet emitter). I4 the MANUAL intro's own stale inventory ("Twenty" listing 21, 3 omissions) fixed in the same commit → full manual lint now required. M1 two-parallel-sources wording. M2 component-count carve-out. M3 crate opener as a full sentence. M4 single-sig-restore zero-hits grep added.
- **R0 round 2 (GREEN 0C/0I, 2026-06-10; persisted at `design/agent-reports/readme-inventory-refresh-r0-r2-review.md`):** all 9 folds verified; placements agree; M4 grep correctly README-scoped (the manual's 2 legitimate mode-contrast uses survive). Polish folded: intro insertion positions pinned; via-ms-shares mirrored to the root clause; the duplicate-CHANGELOG-pointer note recorded. **Gate satisfied.**
