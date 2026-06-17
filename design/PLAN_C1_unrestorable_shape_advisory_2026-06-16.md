# PLAN — C1: bundle-time unrestorable-shape advisory (2026-06-16)

> Tier-2 item C1 from `design/PLAN_remaining_open_items_tiered_2026-06-16.md`.
> Resolves FOLLOWUP `bundle-unrestorable-shape-advisory` (FOLLOWUPS.md:101-106 umbrella;
> shapes at :68-74 + :81-86). **Source SHA: toolkit `342b5c1`** (HEAD == origin/master, post-C2;
> all citations grep-verified at write time). **PATCH → v0.57.0 → v0.57.1** — advisory-only, zero
> clap delta (mirrors the v0.55.2 older() advisory exactly). Toolkit git-tag only, no publish.
> R0 gate: **no code until R0 → GREEN (0C/0I).**

---

## Gap

`bundle --descriptor` accepts + engraves a wire-faithful md1 card for descriptor shapes that
`restore --md1` then REFUSES to mechanically reconstruct. The refusals are LOUD (not silent
funds-loss — the GAP-3 contract `prop_backup_restore_roundtrip.rs:684` pins this), but the user
gets no warning at engrave time that their backup won't watch-only-restore. The three currently-
unrestorable shapes (the OPEN set — pk-keyed was RESOLVED by PART 2, taproot-structural is a
separate deferred concern):

1. **sortedmulti-in-combinator** — `Tag::SortedMulti` anywhere other than the sole child of
   `wsh` / sole grandchild of `sh`→`wsh`. md-codec's miniscript 13.0.0 has no `Terminal::SortedMulti`
   leaf, so restore refuses ("`Tag::SortedMulti` must be the sole child of wsh/sh…The engraved card
   remains a faithful backup"). E.g. `wsh(or_d(pk(@1),sortedmulti(2,@0,@1)))`.
2. **per-key use-site overrides** — `desc.tlv.use_site_path_overrides.is_some()` (cosigners don't
   share one multipath suffix, e.g. `wsh(multi(2,@0/<0;1>/*,@1/*))`). Restore refuses (would
   silently render one shared suffix).
3. **hardened wildcard** — `desc.use_site_path.wildcard_hardened` (`/*h`). Restore refuses (would
   silently render `/*`).

**The advisory's governing principle: fire IFF restore would refuse.** No false positives
(don't warn on a restorable shape), no false negatives (warn on every shape restore refuses).

## Architectural precedent — mirror the v0.55.2 older() advisory EXACTLY

`src/timelock_advisory.rs` is a shipped bundle-time NON-BLOCKING stderr advisory. C1 clones its
shape:

| older() advisory | C1 mirror |
|---|---|
| `pub mod timelock_advisory;` lib.rs:170 + `mod timelock_advisory;` main.rs:34 (dual-homed) | new `src/unrestorable_advisory.rs`, dual-homed the same way |
| `older_advisories_tree(desc: &md_codec::Descriptor) -> Vec<TimelockAdvisory>` (`:187`) walks `desc.tree` | `unrestorable_advisories(desc: &md_codec::Descriptor) -> Vec<UnrestorableAdvisory>` — reads `desc.tree`, `desc.tlv.use_site_path_overrides`, `desc.use_site_path.wildcard_hardened` |
| `TimelockAdvisory::message()` builds the `"advisory: older(…"` string | `UnrestorableAdvisory::message()` per shape, ending "…The engraved card remains a faithful backup; keep the full descriptor to restore. Tracked: <slug>" |
| `emit_advisories(&[..], stderr)` best-effort `writeln!` to stderr (`:102`) | identical signature/body |
| hooked at bundle.rs:1665, :1707, :1969 (the 3 engrave sites, each on the bound `descriptor`) | hook the new walk at the SAME 3 sites, right after the `older_advisories_tree` call |
| NOT suppressed under `--json` (stderr ≠ stdout JSON wire; comment bundle.rs:812) | same — never suppressed |
| no new `ToolkitError` variant; exit code stays 0 | same |

## Citations (grep-verified @ `342b5c1`)

| Surface | Location |
|---|---|
| advisory precedent module | `src/timelock_advisory.rs` — `emit_advisories` `:102`, `older_advisories_tree` `:187`, `older_advisories_node` `:193` |
| module dual-home | `src/lib.rs:170` (`pub mod timelock_advisory;`), `src/main.rs:34` (`mod timelock_advisory;`); `cmd` is bin-only (`main.rs:6 mod cmd;`) |
| bundle hook sites (3) | `cmd/bundle.rs:1665`, `:1707`, `:1969` (`older_advisories_tree(&descriptor)`); descriptor bound at `:1601` (unified), `:1701` (concrete), `:1946` (import-json) |
| import-wallet hook site (4th — R0-r1 I2) | `cmd/import_wallet.rs:1291` (`older_advisories_tree(&p.descriptor)`); md1 synth `:1439`, emit `:1532`; `p.descriptor: md_codec::Descriptor` `:1289` |
| shape-1 third acceptance arm (R0-r1 I1) | md-codec `to_miniscript.rs:248` `new_sh_sortedmulti` (bare `sh(sortedmulti)`); toolkit emit shape `parse_descriptor.rs:1511 walk_sh_sortedmulti_root` |
| shape-2 guard (restore) | `cmd/restore.rs:1247` `if d.tlv.use_site_path_overrides.is_some()` → `ModeViolation` "per-cosigner use-site path overrides…faithful backup…Tracked: restore-md1-per-key-use-site-and-hardened-wildcard" |
| shape-3 guard (restore) | `cmd/restore.rs:1254` `if d.use_site_path.wildcard_hardened` → `ModeViolation` "hardened wildcard (`/*h`)…faithful backup…" |
| shape-1 refusal (md-codec renderer) | `md-codec 0.36.0 src/to_miniscript.rs:417` (`Tag::SortedMulti` leaf arm errs); restore reaches it via `restore.rs:1344 faithful_multisig_descriptor` → `:1109 to_miniscript_descriptor` → wrap `:1119` "…faithful backup." |
| shape-1 sole-child acceptance (the NON-firing set to mirror) | md-codec `to_miniscript.rs:198-207` `wsh_inner_to_descriptor` (SortedMulti sole wsh child) + `:222-248` `sh_inner_to_descriptor` (sole sh→wsh grandchild); toolkit recognizer `restore.rs::plain_template_from_tree` `:1156-1182` |
| GAP-3 loud-refuse contract (shape 1) | `tests/prop_backup_restore_roundtrip.rs:684` `sortedmulti_in_combinator_bundles_but_restore_refuses_loudly` (bundle `.success()` + restore `.failure()` stderr `contains("sole child")&&contains("faithful backup")`) |
| md_codec Descriptor fields | `Descriptor.tree: Node`, `.use_site_path: UseSitePath`, `.tlv: TlvSection`; `UseSitePath.wildcard_hardened: bool`; `TlvSection.use_site_path_overrides: Option<Vec<(u8, UseSitePath)>>` |
| older() cross-surface test (mirror structure) | `tests/cli_older_advisory.rs` (4 invariants: fires+non-blocking / clean-negative / dedup / `--json` stdout-clean) |
| older() per-bundle tests | `tests/cli_bundle_full.rs:191` `bundle_descriptor_masked_older_emits_advisory` + `:228` clean negative |
| manual anchor to mirror | `docs/manual/src/40-cli-reference/41-mnemonic.md:50` `### Consensus-masked relative timelocks {#…}` (in `## mnemonic bundle`); restore's documented refusals at `:954-958` (use-site/hardened) + `:968-969` (sortedmulti_a-in-leaf) |
| CHANGELOG PATCH precedent | `CHANGELOG.md:51-57` (v0.55.2 older()-advisory block: "PATCH — advisory-only, zero clap delta → no GUI `schema_mirror` impact") |
| FOLLOWUP umbrella | `design/FOLLOWUPS.md:101-106` (`bundle-unrestorable-shape-advisory` tracker) + `:68-74` (shape 1) + `:81-86` (shapes 2/3) |

## Scope — the md1-ENGRAVING surfaces: `bundle` + `import-wallet` (R0-r1 I2 CORRECTION)

The advisory fires on the surfaces that **synthesize a NEW md1 card from an `md_codec::Descriptor`**
— i.e. the surfaces whose engraved card a user would later try to `restore --md1`. There are TWO:

1. **`bundle`** — 3 engrave sites (`bundle.rs:1665/1707/1969`), descriptor bound via `parse_descriptor`.
2. **`import-wallet`** — `import_wallet.rs:1439 synthesize_descriptor(&p.descriptor,…)` → `:1532 md1:
   bundle.md1`, with `p.descriptor: md_codec::Descriptor` (`:1289`) populated by the same
   `parse_descriptor` (foreign parsers `wallet_import/bitcoin_core.rs:278`, `electrum.rs:377`, …).
   The older() advisory ALREADY fires here (`import_wallet.rs:1291`). **R0-r1 verified empirically:**
   importing a `bitcoin-core` wallet with `wsh(sortedmulti(2,…/*h,…/*h))` emits an md1 that
   `restore --md1` refuses (`restore.rs:1254`) — a real zero-warning gap. So import-wallet MUST
   warn too (option (a)), matching the older() precedent + the "engraves a restore-refusable md1"
   criterion.

**Excluded (correctly):** `export-wallet` never builds an `md_codec::Descriptor` (only
`miniscript::Descriptor` via `from_str` → shapes 2/3's md-codec-only fields are structurally
undetectable) AND emits no md1 card. `inspect`/`repair` CONSUME existing md1 chunks (decode /
BCH-correct) — they don't synthesize a new md1. No `convert` path emits an md1 from a descriptor.
So the advisory's surface set is exactly {`bundle`, `import-wallet`} — the older()-advisory
`_tree` (Adapter-A) sites that hold an `md_codec::Descriptor` AND engrave/emit an md1.

## Implementation

**New `src/unrestorable_advisory.rs`** (dual-homed: `pub mod unrestorable_advisory;` in lib.rs
after `:170`; `mod unrestorable_advisory;` in main.rs after `:34`):

```rust
pub enum UnrestorableShape { SortedMultiInCombinator, PerKeyUseSiteOverrides, HardenedWildcard }
pub struct UnrestorableAdvisory { shape: UnrestorableShape }
impl UnrestorableAdvisory { pub fn message(&self) -> String { /* per-shape, mirrors restore wording + slug */ } }

/// Fire iff restore --md1 would refuse this descriptor.
pub fn unrestorable_advisories(desc: &md_codec::Descriptor) -> Vec<UnrestorableAdvisory> {
    let mut out = vec![];
    if tree_has_sortedmulti_in_combinator(&desc.tree) { out.push(SortedMultiInCombinator); }
    if desc.tlv.use_site_path_overrides.is_some()     { out.push(PerKeyUseSiteOverrides); }
    if desc.use_site_path.wildcard_hardened           { out.push(HardenedWildcard); }
    out
}
pub fn emit_advisories<E: Write>(advs: &[UnrestorableAdvisory], stderr: &mut E) { /* mirror :102 */ }
```

**Shape-1 predicate `pub(crate) fn tree_has_sortedmulti_in_combinator(root: &Node) -> bool`**
(`pub(crate)` per M1, so module unit tests reach it without an `md_codec::Descriptor` literal) — the
only NEW logic (shapes 2/3 are field reads). Mirror md-codec's acceptance set exactly. **R0-r1 I1:
md-codec accepts `Tag::SortedMulti` in THREE restorable positions** (`to_miniscript.rs` `:205`/`:231`/
`:248`) — the predicate must NOT fire on any of them:
- (a) the sole `Body::Children` child of a top-level `Tag::Wsh` — `wsh(sortedmulti)` (`new_wsh_sortedmulti :205`).
- (b) the sole grandchild via top-level `Tag::Sh`→sole-child `Tag::Wsh`→sole-child `SortedMulti` — `sh(wsh(sortedmulti))` (`new_sh_wsh_sortedmulti :231`).
- (c) the sole **direct** `Body::Children` child of a top-level `Tag::Sh` (NO intervening `Wsh`) — bare legacy P2SH `sh(sortedmulti)` (`new_sh_sortedmulti :248`; toolkit emits root `Tag::Sh` with `children[0].tag == Tag::SortedMulti`, cf. `parse_descriptor.rs:1511 walk_sh_sortedmulti_root`). **The R0-r1 omission — `sh(sortedmulti)` bundles + restores faithfully (exit 0), so firing on it is a false positive.**

Algorithm: if `root` matches sole-child position (a), (b), or (c), descend PAST the recognized
`SortedMulti` (it's accepted) and walk only the rest (there is no rest for a sole-child template, so
those return false); otherwise recursively walk the whole tree and return true if ANY
`Tag::SortedMulti` node is encountered (every such node is a combinator-leaf → restore-refuse).
`Tag::Multi` (a real miniscript Terminal) and `Tag::SortedMultiA`/taproot are OUT of scope
(multi-in-combinator restores fine; SortedMultiA-in-leaf is a separate taproot concern — `to_miniscript.rs:423`).

**Hook (4 sites):** at `bundle.rs:1665`, `:1707`, `:1969` AND `import_wallet.rs:1291`, immediately
after each existing `older_advisories_tree(&descriptor)` (or `&p.descriptor`) + its
`emit_advisories`, add:
```rust
let unrest = crate::unrestorable_advisory::unrestorable_advisories(&descriptor); // or &p.descriptor
crate::unrestorable_advisory::emit_advisories(&unrest, &mut stderr); // mirror the adjacent stderr handle
```
Match each site's existing stderr handle + emit style exactly (mirror the adjacent older() call).
`unrestorable_advisories` returns AT MOST ONE entry per shape (each is a single bool/`is_some`; two
`/*h` keys = one `wildcard_hardened` fire — M3), so emit is at most 3 stderr lines.

## TDD — tests are the deliverable; PARITY is the non-vacuity oracle

**Predicate-parity is the core correctness property: the advisory fires IFF restore refuses.** Each
positive test feeds ONE descriptor to BOTH `bundle` (assert advisory fires, exit 0) and
`restore --md1` of the emitted card (assert refusal) — proving parity on real shapes.

New `tests/cli_unrestorable_shape_advisory.rs` (mirror `cli_older_advisory.rs` structure):
1. **Shape-1 fires + non-blocking + parity.** `bundle --descriptor "wsh(or_d(pk(@1),sortedmulti(2,@0,@1)))" … --json` → `.success()` (the GAP-3 `.success()` guard MUST hold) with stderr advisory; then `restore --md1 <emitted>` → `.failure()` ("sole child"…"faithful backup"). Reuses the GAP-3 descriptor (`prop_backup_restore_roundtrip.rs:684`).
2. **Shapes 2 & 3 fire + parity (R0-r1 verified bundle-constructible).** `bundle --descriptor "wsh(multi(2,@0/<0;1>/*,@1/*))" …` → exit 0 + shape-2 advisory; `restore --md1 <card>` → refusal (`restore.rs:1247`). `bundle --descriptor "wsh(multi(2,@0/*h,@1/*h))" …` → exit 0 + shape-3 advisory; `restore --md1 <card>` → refusal (`restore.rs:1254`). (R0-r1 ran both end-to-end — no md_codec-direct construction needed; bundle's `parse_descriptor` sets both fields.)
3. **import-wallet parity (R0-r1 I2).** `import-wallet --format bitcoin-core --blob -` with a `listdescriptors` envelope carrying `wsh(sortedmulti(2,[…]xpub…/*h,[…]xpub…/*h))` → exit 0 + shape-3 advisory on stderr; `restore --md1 <emitted card>` → refusal. The R0 reviewer built this exact fixture — reuse it. Mirrors the older() import-wallet test structure.
4. **Clean negatives (no false positives).** ALL THREE sole-child sortedmulti shapes must NOT fire (R0-r1 I1): `wsh(sortedmulti(2,@0,@1))`, `sh(wsh(sortedmulti(2,@0,@1)))`, AND **`sh(sortedmulti(2,@0,@1))`** (the bare-P2SH case the predicate must exempt) — each: bundle exit 0, NO advisory, `restore --md1` exit 0 (faithful). Plus a shared-suffix multisig (no overrides), an unhardened-wildcard descriptor, and a `wsh(multi(2,…))`-in-combinator (`multi` not `sortedmulti`, restores fine) → NO advisory — guard the predicates against over-firing.
5. **`--json` stdout cleanliness.** The advisory is on stderr; the `--json` stdout payload never contains the advisory text (mirror `cli_older_advisory.rs:20-21`).

**Module unit tests** in `unrestorable_advisory.rs` (mirror `timelock_advisory.rs:222`): focus on the
shape-1 `pub(crate)` walk over hand-built `Node` trees — combinator-sortedmulti → true; ALL THREE
sole-child shapes (`wsh(sortedmulti)`, `sh(wsh(sortedmulti))`, `sh(sortedmulti)`) → false;
multi-in-combinator → false; no-sortedmulti → false — plus the `message()` forms. **M2: shapes 2/3
are `md_codec::Descriptor` FIELD reads, not `Node` walks** — cover them at the CLI cross-surface
layer (tests 2-3 above), NOT as module unit tests (avoid fragile `Descriptor` literals).

**Non-vacuity:** revert the 4 hooks → tests 1-3 lose the stderr advisory → RED. Revert the shape-1
walk's sole-child exemptions → the clean-negatives (esp. `sh(sortedmulti)`) start firing → RED.

## Lockstep / SemVer (the FULL release-ritual checklist — corrected by C2's impl-review)

- **PATCH → v0.57.0 → v0.57.1.** Advisory-only, zero clap delta → **no schema_mirror surface**, no
  GUI paired PR (mirrors v0.55.2). **No new `ToolkitError` variant** (stderr warning, exit 0).
- **Version sites — ALL of (C2-impl-review C1/I1 lesson):** `crates/mnemonic-toolkit/Cargo.toml`
  version; **BOTH** `README.md:13` AND `crates/mnemonic-toolkit/README.md:9` toolkit-version markers
  (the `readme_version_current` test enforces both); `scripts/install.sh:32` self-pin;
  `fuzz/Cargo.lock` mnemonic-toolkit package (separate workspace — silent drift; `cargo update -p
  mnemonic-toolkit --precise 0.57.1` in `fuzz/`); main `Cargo.lock`. CHANGELOG `[0.57.1]` mirroring
  the `:51-57` block. NO sibling pin changes → manual.yml/quickstart.yml/cross-tool FROZEN.
- **Manual mirror (mandatory) — BOTH surfaces (R0-r1 I2):** add a subsection mirroring the older()
  anchor (`41-mnemonic.md:50`) — a `### Unrestorable descriptor shapes {#unrestorable-shapes}` under
  `## mnemonic bundle`, cross-ref'd to restore's already-documented refusals (`:954-958`, `:968-969`),
  AND a "See …" cross-ref in the `## mnemonic import-wallet` section (mirroring how export-wallet
  `:673-674` cross-refs the older() anchor). Run `make -C docs/manual lint` (4 binaries) —
  flag-coverage unaffected (no new flag); confirm markdownlint/cspell/index/glossary pass.
- **fmt gate:** `cargo +1.95.0 fmt --all` then REVERT `mlock.rs` (g6 exemption).
- **FOLLOWUP flip:** `bundle-unrestorable-shape-advisory` umbrella → `resolved` (FOLLOWUPS.md:106),
  citing the new module + the parity tests + BOTH surfaces (bundle + import-wallet). (The shape-1
  `bundle-accepts-sortedmulti-in-combinator-restore-cannot` and shapes-2/3
  `restore-md1-per-key-use-site-and-hardened-wildcard` entries note the advisory now warns at
  engrave time on both surfaces, but their RECONSTRUCTION halves stay open.)

## Execution

1. R0 architect review of THIS plan → loop to GREEN (0C/0I), persist verbatim to
   `design/agent-reports/c1-unrestorable-advisory-plan-r0-round{N}-review.md`. **R0 must vet the
   shapes-2/3 construction story** (are they bundle-constructible? — the one open design risk).
2. TDD: write cross-surface + module tests; confirm RED.
3. Implement the module + 4 hooks (bundle ×3 + import-wallet ×1). Confirm GREEN + full suite + clippy + manual lint + fuzz build.
4. Per-phase impl review → 0C/0I, persist.
5. Version bump v0.57.1 + ALL lockstep sites + manual + CHANGELOG + FOLLOWUP flip. fmt gate.
6. Commit, tag `mnemonic-toolkit-v0.57.1`, push master, verify CI green (per standing authorization).
