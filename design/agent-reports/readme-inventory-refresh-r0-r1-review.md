# R0 review — PLAN_readme_inventory_refresh — round 1

**Verdict: YELLOW**

Docs-only, well-scoped, recon-grounded, and the codex32 trap is correctly defused — but the headline replacement text at both Status sites points at an invisible HTML comment, the §2/§4 inventories disagree about where `build-descriptor` lives, and §1/§3 certify as "already accurate" a restore clause that has been stale since v0.44.0. All fixable in a plan revision; no re-recon needed.

## Critical

**C1 — "Status: see the version marker above" points at nothing a reader can see.** The `<!-- toolkit-version -->` markers are HTML comments — invisible in rendered markdown on GitHub, stripped by crates.io. What renders "above" is the card table (root) and "Installs as binary `mnemonic`." (crate). Fix: drop the version pointer, or point at a VISIBLE surface — root: `Status: the \`mnemonic\` CLI (see [CHANGELOG.md](CHANGELOG.md) for the current release) spans …`; crate variant can open `The \`mnemonic\` subcommands span …` (its paragraph already ends with a CHANGELOG pointer).

## Important

**I1 — §2 and §4 contradict each other on `build-descriptor` placement.** The inline list is an exact 1:1 mirror of the root grouped inventory (verified group-by-group). §4 inserts after `compare-cost` (inside Decrypt/repair/inspect); §2 creates a new group after Convert/derive. The manual orders `build-descriptor` LAST, after `compare-cost` — §4's instinct has manual precedent; pick one, make both inventories agree.

**I2 — §1/§3 certify a stale restore clause as "already accurate."** Both openers say "watch-only **single-sig** restore documents" — `restore --md1` multisig shipped v0.44.0, taproot NUMS at v0.49.1. The same commit adding multisig-restore to the inventory would leave the openers asserting single-sig-only. Extend both: "watch-only restore documents (single-sig from a seed + passphrase, fingerprint-gated; multisig from the shared md1 card, incl. taproot NUMS)".

**I3 — `restore` → Bundle group is the weakest placement and the plan doesn't argue it.** The manual orders restore between export-wallet and import-wallet; `restore --format` invokes "an export-wallet emitter" with the same value list. Either move restore to Wallet import/export, or retitle Bundle. Decide explicitly; §4 follows per I1's mirror rule.

**I4 — the §0 "manual is the ONLY authoritative inventory" claim is undermined by a 5th stale inventory.** The manual chapter intro (`41-mnemonic.md:3-19`) says "**Twenty** subcommands:" then lists **21** names — omitting `addresses`, `restore`, `build-descriptor` (all three have full sections in the same file). The lint gate does NOT cover that paragraph. Fix it in the same commit (same de-count posture) or file a FOLLOWUP; silence is the only wrong option.

## Minor

**M1 —** restore §2 wording: "from a seed" umbrella is wrong for the multisig branch (seed OPTIONAL in --md1 mode; the md1 ALONE reconstructs). Two parallel sources per I2.
**M2 —** §0 absolutism vs `README.md:20` "all 5 m-format constellation components" — a COMPONENT count, fine; add a carve-out so nobody "fixes" it.
**M3 —** §3 fragment grammar; resolves under the C1 rewrite.
**M4 —** add `grep -n "single-sig restore"` → 0 hits to §5.

## Claim audit (abbrev.)
- "5 archetype presets" OK; "--allow reviewed opt-out" OK; "validated policy-tree → wsh" OK; cost-preview nuance (unavailable under --allow) noted; ms-shares split/combine OK; **codex32 trap DEFUSED** (`git log -S` proves the README clause was BORN with the v0.40.0 ms-shares release — it never referred to another subcommand; slip39=Shamir, seed-xor=XOR, no other codex32 surface); 24 names OK; restore clause CORRECTED (I2/M1); import/export 7-format list = non-exhaustive subset of the binary's 8 (understatement, OK).

## Empirical probes run
Binary 0.52.0 --help census (24); build-descriptor/ms-shares/slip39/restore --helps; git log -S over both READMEs; full reads of both README regions (markers confirmed invisible-comment-only); manual chapter order + intro count ("Twenty" listing 21, 3 omissions); CHANGELOG greps (ms-shares v0.40.0; taproot restore v0.49.1); FOLLOWUP + recon reads; repo at bc373ed clean.
