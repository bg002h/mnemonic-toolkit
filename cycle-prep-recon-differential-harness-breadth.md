# cycle-prep recon — 2026-06-12 — differential harness breadth (GAP 4)

Origin SHAs: mnemonic-toolkit `origin/master` = `ca7d7bc` (confirmed `git status -sb` clean-tracking); descriptor-mnemonic `origin/main` = `422b049` (confirmed). All citations below are against these SHAs.

GAP 4 of the m-format miniscript-coverage audit: the three differential/verification harnesses are narrow (cross-tool corpus = 8 easy shapes; STRESS-A = wsh-only; md-codec bitcoind = 10 shapes) and the toolkit has NO bitcoind oracle at all. Recon verifies each harness's actual scope, why it is narrow, and what widening costs.

## Verification

### 1. Cross-tool md-cli differential — re-grounded (toolkit)

File: `crates/mnemonic-toolkit/tests/cli_cross_tool_differential.rs` (435 lines; NOTE the gap statement's path `tests/…` is missing the crate prefix). Workflow: `.github/workflows/cross-tool-differential.yml` — EXISTS.

**The corpus (8 entries, `corpus()` :253-340), ALL `Verdict::Match`:**

| # | label | shape |
|---|---|---|
| 1 | `wpkh` | `wpkh([fp/84'/0'/0']xpub3/<0;1>/*)` |
| 2 | `pkh` | `pkh([fp/44'/0'/0']xpub3/<0;1>/*)` |
| 3 | `wsh-multi-2of2` | `wsh(multi(2,@0,@1))` |
| 4 | `tr-pk-leaf` | `tr(@0,pk(@1))` (cosigner internal key, single pk leaf) |
| 5 | `wsh-pk` | `wsh(pk(@0))` |
| 6 | `wsh-pkh` | `wsh(pkh(@0))` |
| 7 | `wsh-and_v` | `wsh(and_v(v:pk(@0),pk(@1)))` |
| 8 | `wsh-or_d` | `wsh(or_d(pk(@0),pk(@1)))` |

Entries 5-8 were the `Diverge` pins the harness was BORN with (Check-PkK-in-non-tap); v0.55.0 (`bf7bf8b`, shipped since the GAP-4 framing was drafted) dropped the toolkit's `tap_context` gate, so all 8 are now `Match` — the harness is a pure cross-tool MATCH regression gate (FOLLOWUP `toolkit-check-pkk-non-tap-non-canonical` ✓ RESOLVED, `design/FOLLOWUPS.md:31`).

- **Verdict enum** (:62-72): `enum Verdict { Match, Diverge, BothError, ToolError(Tool) }` — four-arm; an entry is Match/Diverge ONLY when both tools exit 0 AND `md inspect --json` reads both `wallet_policy_id.hex` + `wallet_descriptor_template_id.hex`. (The `Entry` field is `expect: Verdict` — there is no separate `Expect` enum.)
- **Anti-vacuity guards** (:359-373, :419-434): declared `n_match >= 1`; run-time `saw_match` (≥1 entry actually reached a real Match); `n_both_error == 0 && n_tool_error == 0` (a tool silently failing would mask a walker regression behind a non-comparison). The old hard ≥1-Diverge requirement was dropped at v0.55.0 (would now panic).
- **Gating**: `#[ignore = "needs both compiled binaries…"]` (:343); `MNEMONIC_BIN` defaults to `CARGO_BIN_EXE_mnemonic`, `MD_BIN` required else skip-with-message (:346-355). CI workflow fires on push/PR touching `parse_descriptor.rs` / the test / the workflow + `workflow_dispatch`; installs md-cli **tag-pinned `descriptor-mnemonic-md-cli-v0.6.2`** with `--features cli-compiler` (yml:46 — a deliberate pinned baseline), builds debug `mnemonic`, runs `-- --ignored --nocapture` with both env vars (yml:51-55).
- **Frozen key pool** (:51-57): mfp `73c5da0a`, two depth-3 xpubs (84'/44') + exactly TWO depth-4 xpubs (m/48'/0'/{0,1}'/2'). md-cli enforces BIP-32 depth per script context, and `--path` is one shared origin per entry (multi-key entries already give both cosigners the same origin, [I2] :249-252) — a third cosigner needs ONE new frozen depth-4 literal (m/48'/0'/2'/2', derivable from the abandon phrase; the md-codec bitcoind corpus already uses exactly this account).

**Absent fragments (confirmed)**: `sortedmulti` (even sole-child), `thresh`, `older`, `after`, all four hashlocks (`sha256`/`hash256`/`ripemd160`/`hash160`), ≥3 cosigners, `or_i`, `andor`, `sh(wsh(…))` nesting, taproot multisig (`multi_a`/`sortedmulti_a`), NUMS internal key. These are precisely the fragments where two near-identical hand-written walkers can re-diverge (wrapper handling, digest byte order, timelock encoding, k-of-n bodies, NUMS flag).

**Both walkers already support all of them** — widening is pure corpus-row addition:
- md-cli walker (`descriptor-mnemonic/crates/md-cli/src/parse/template.rs`): `Terminal::Multi`/`MultiA` (:590/:596), `Older` (:646), `After` (:653), `Sha256`/`Hash256`/`Hash160` (:716-728, + ripemd by symmetry), `Thresh` (:772), `ShInner::SortedMulti`/`WshInner::SortedMulti` (:545/:562), NUMS H-point literal internal key → `is_nums:true` (:813-822; test :1153-1159 pins exactly `tr(<H-hex>,multi_a(2,@0,@1,@2))`). k range 1..=32 (:504).
- Toolkit walker (`parse_descriptor.rs`): `NUMS` sentinel substitution + `walk_tr` NUMS detection (:258-270); the wsh fragment family is proven by STRESS-A, which bundles all of them today.

**Caveat — tr-sortedmulti_a asymmetric**: md-cli/md-codec pin crates.io rust-miniscript 13.0.0, which LACKS `Terminal::SortedMultiA` parse (the v0.49.1 recon finding; FOLLOWUP `md-codec-sortedmulti-a-to-miniscript-rendering-gap`, toolkit `design/FOLLOWUPS.md:282`), while the toolkit workspace patches miniscript to fork rev `95fdd1c` (`Cargo.toml:16-17`) which HAS it. A `tr(NUMS,sortedmulti_a(…))` entry would likely land `ToolError(MdCli)` — scope the tr additions to `multi_a` (or pin the asymmetry deliberately as an expected `ToolError` entry; brainstorm decision). Verify md-cli's accepted depth for tap-multisig keys with one probe run (the bitcoind corpus uses depth-3 m/86'/0'/N').

**Caveat — sortedmulti-in-combinator**: bundle accepts it, so a cross-tool entry works (restore is not in this harness), but it is a known round-trip asymmetry (FOLLOWUP `bundle-accepts-sortedmulti-in-combinator-restore-cannot`, `design/FOLLOWUPS.md:53`, found by STRESS-A run #1). Prefer sole-child `wsh(sortedmulti(…))` entries unless the cycle wants to pin the combinator case too.

### 2. STRESS-A proptest — wsh-only confirmed, and WHY

File: `crates/mnemonic-toolkit/tests/prop_backup_restore_roundtrip.rs`. The 10 schemas (:143-249) all emit `{"schema_version":1, "wrapper":"wsh", "root":…}` (:246). **No taproot anywhere** — confirmed.

**Why wsh-only: STRUCTURAL, not unfinished.** The generator pipeline starts at `build-descriptor --spec`, and build-descriptor v1 only renders wsh: `descriptor_builder/ir.rs:91-95` — `enum WrapperKind { Wsh }` with the doc comment "v1 ships `wsh` only; `tr` is the deferred wrapper-strategy seam (SPEC §5.2)"; `descriptor_builder/schema.rs:154` — `"wrapper": { "values": ["wsh"] }`. So STRESS-A inherits build-descriptor's deliberate v1 scoping. (The STRESS-A SPEC's own line 10 even mentions "`wsh` vs `sh(wsh)` wrapper" parameterization, but the shipped `WrapperKind` lacks `ShWsh` too — the generator is narrower than its SPEC prose.)

**Extending to tr-multi_a is feasible WITHOUT touching build-descriptor** (keeping the cycle test-only/NO-BUMP):
- restore DOES reconstruct taproot NUMS multisig since v0.49.1: `cmd/restore.rs:66-67` ("wsh / sh(wsh) and taproot NUMS multisig (tr-multi-a / tr-sortedmulti-a); a non-NUMS … is refused"), `:704-705` (`Tag::MultiA → CliTemplate::TrMultiA`, `Tag::SortedMultiA → TrSortedMultiA`), `:685-689` (non-NUMS → loud refusal), `:710` (non-multisig tap leaf → refusal). Reconstructable taproot domain = `tr(NUMS, {multi_a|sortedmulti_a}(k, …))`, single leaf.
- A tr leg generates concrete descriptor STRINGS directly (`tr(NUMS,multi_a(k,…))` — the toolkit substitutes the `NUMS` sentinel, `parse_descriptor.rs:258-270`) and enters the pipeline at `bundle --descriptor` (skipping the build-descriptor gate; bundle acceptance becomes the step-1 gate for that leg). O1 `normalize`, O2 fixed-point, and O3 `derive_receive` all reuse unchanged — the test's `miniscript` is the workspace-patched fork `95fdd1c` (`[patch.crates-io]`, `Cargo.toml:16-17`), which parses AND derives `multi_a`/`sortedmulti_a` (the reason v0.49.1's `derive_receive_addresses` works on the descriptor string).
- Bonus gap to close in the same leg: the SPEC's negative property (SPEC :35) says "General-taproot: assert the loud refusal at restore's taproot arm" — the SHIPPED `negative_property_unreconstructable_shapes_refuse_loudly` (:551-588) covers only the two wsh `@N` use-site shapes; the non-NUMS-tr refusal cell never shipped. Add it.

### 3. md-codec bitcoind differential — re-grounded (descriptor-mnemonic)

File: `crates/md-codec/tests/bitcoind_differential.rs` (601 lines). **The corpus is 10 shapes, not 11** (the GAP-4 framing's "11 shapes" is a counting drift — `corpus()` :112-404 and the workflow header both say 10):

1. `pkh` (BIP-44) · 2. `sh(wpkh)` (BIP-49) · 3. `wpkh` (BIP-84) · 4. `tr` keypath (BIP-86) · 5. `wsh(sortedmulti 2-of-3)` (BIP-48/2) · 6. `sh(wsh(sortedmulti 2-of-3))` (BIP-48/1) · 7. `tr(NUMS, multi_a 2-of-3)` · 8. `tr(key, multi_a 2-of-3)` · 9. `wsh(and_v(v:pk, older(144)))` · 10. `wsh(thresh(2, pk, s:pk, s:pk))`.

Confirmed absent: plain (unsorted) `multi`, all hashlocks, `after`, multi-leaf taptree, wrappers other than `v:`/`s:` (`Tag::Verify` :348, `Tag::Swap` :388), and every or_/andor combinator (only `and_v`).

- **Env gate** (:419-437): connect-only (the test never spawns bitcoind); `BITCOINCLI_BIN` + `BITCOIND_DATADIR` + `BITCOIND_RPCPORT`. All unset → skip (the `#[ignore]` local default); partially set → panic; set-but-node-dead → `getblockchaininfo` panic (RED, never green-by-skip). Anti-vacuity golden: wpkh chain0 idx0 must equal the published BIP-84 address `bc1qcr8te4…` (:52, :570-576) before any bitcoind compare counts.
- **CI**: `.github/workflows/bitcoind-differential.yml` EXISTS — push/PR on `derive.rs`/`to_miniscript.rs`/`canonicalize.rs`/`encode.rs`/the test/the workflow, PLUS daily cron `17 5 * * *` + dispatch. Pinned Bitcoin Core **v27.0** tarball, sha256-verified (`2a6974c5…`, yml:51-53), cached by content hash; **offline `-chain=main`** (`-connect=0 -listen=0 -blocksonly=1`, rpcport 18999, cookie auth; regtest is dead here — it rejects mainnet xpubs and md-codec's TLV→xpub path always renders `xpub…`). 10 shapes × 2 chains × idx 0..=4 = 100 address checks + 20 `getdescriptorinfo` checksum round-trips.
- **Adding shapes is cheap and uniform**: one `Shape { label, desc }` struct per row (~30-45 lines of mechanical md-codec `Descriptor`/TLV construction mirroring `address_derivation.rs`). No per-shape multipath special-casing — the test renders the per-chain single descriptor itself via `to_miniscript_descriptor(&desc, chain)` (:500), so bitcoind never sees `<0;1>`. Constraints per new shape: it must sit in the "md-codec-derivable ∩ bitcoind-sane" intersection — i.e. `to_miniscript` must render it (so **no `SortedMultiA`**: md-codec's crates.io miniscript 13.0.0 gap, FOLLOWUP `md-codec-sortedmulti-a-to-miniscript-rendering-gap`), and Core v27 must accept it. Plain `multi`, the four hashlocks, `after`, and `or_d`/`or_i`/`andor` combinator rows are all straightforward Tag/Body rows; a multi-leaf taptree row needs a check that md-codec's `Body::Tr { tree }` + to_miniscript can render a branch node (uncertain — one probe; flag for the brainstorm).

### 4. Toolkit bitcoind oracle — ABSENT, confirmed

Grep of the whole toolkit repo (`bitcoind|bitcoin-cli|getdescriptorinfo|deriveaddresses`, `*.rs|*.yml|*.toml|*.sh`) hits exactly two non-oracle sites:
- `src/error.rs:769` — a help string telling the user to re-run `bitcoin-cli listdescriptors` without `true` (import-wallet diagnostics);
- `tests/cli_import_wallet_bitcoin_core.rs:394` — a comment; the test consumes a **static** `listdescriptors` JSON fixture, no live node.

No workflow, no RPC client, no `deriveaddresses` anywhere. **The toolkit has no bitcoind oracle.**

**Would a toolkit one duplicate md-codec's? No — different surface.** md-codec's differential tests the CODEC-internal `Descriptor::derive_address` against Core. A toolkit differential would test the END-TO-END user pipeline against Core: generate policy → `bundle --descriptor` (engrave) → `restore --md1` → assert restore's reported descriptor + `first_addresses` against Core's `deriveaddresses` on that descriptor (and the original). That validates the toolkit's own address derivation (`derive_address.rs`/`address_render.rs`, which v0.49.1 routes AROUND md-codec for taproot) and the reconstruction path with EXTERNAL C++ ground truth. STRESS-A's O3 is a rust-miniscript differential — a same-ecosystem oracle (the toolkit delegates to the same patched rust-miniscript); bitcoind is the only oracle that catches the class both share. All bitcoind infra (pinned-tarball download/verify/cache, offline-mainnet lifecycle, connect-only env-var contract) is directly liftable from md-codec's 140-line workflow + ~60-line client.

### FOLLOWUP sweep (both repos)

No existing slug covers harness breadth itself. Related, all verified live at the SHAs above:
- toolkit `bundle-accepts-sortedmulti-in-combinator-restore-cannot` (:53, deferred) — found by STRESS-A run #1; constrains corpus choices (prefer sole-child sortedmulti).
- toolkit `md-codec-sortedmulti-a-to-miniscript-rendering-gap` (:282) — blocks tr-sortedmulti_a on the md-cli AND md-codec-bitcoind sides.
- toolkit `toolkit-check-pkk-non-tap-non-canonical` — ✓ RESOLVED v0.55.0 (:31); the proof the cross-tool harness finds real bugs.
- toolkit `orphaned-v0_2-md1-vectors-no-harness` (:290) — adjacent dead-golden cleanup, not this cycle.
- A NEW FOLLOWUP (or three) should be filed for whatever this cycle defers.

## Assessment

All three sub-improvements are TEST-BREADTH (NO-BUMP, test-only) — unless widening RE-FINDS a divergence, in which case the cycle spawns a real fix (possibly a v0.55.0-style MINOR wire change). That is the point, not a risk: the cross-tool harness exists because the walkers diverged once already, and the absent fragments (thresh wrapper handling, hashlock digest byte order, timelock bodies, k-of-n, NUMS/multi_a) are exactly hand-written-walker territory.

**(a) Widen the cross-tool corpus — CHEAPEST, HIGHEST VALUE.**
- Scope: ~8-12 new `Entry` rows in ONE file (`cli_cross_tool_differential.rs`), + 1 new frozen depth-4 xpub literal (m/48'/0'/2'/2') and possibly 2-3 depth-3 m/86' literals for tr. Candidate rows: `wsh(sortedmulti(2,@0,@1))` sole-child; `wsh(multi(2,@0,@1,@2))` 3-cosigner; `wsh(thresh(2,pk,s:pk,s:pk))`; `wsh(and_v(v:pk,older(144)))`; `wsh(and_v(v:pk,after(800000)))`; one row per hashlock family (or one combined andor-hashlock archetype); `wsh(or_i(…))`; `sh(wsh(sortedmulti(…)))`; `tr(NUMS,multi_a(2,@0,@1,@2))`. No workflow change needed (it already triggers on the test path; the md-cli pin v0.6.2 already supports every fragment — verified against its walker arms).
- Blockers: none hard. Two probe items for the brainstorm: md-cli's enforced key depth in tap-multisig context, and whether to pin tr-sortedmulti_a as an expected `ToolError(MdCli)` (the 13.0.0 gap) or omit it.
- Divergence-refind likelihood: REAL — this is the only sub-improvement that directly re-guards the walker parity that already broke once (Check-PkK). Budget the cycle for a possible found-bug detour.

**(b) Extend STRESS-A to a taproot leg — MEDIUM effort, medium-high value.**
- Scope: ~120-180 lines in `prop_backup_restore_roundtrip.rs`: a tr schema family generating concrete `tr(NUMS,{multi_a|sortedmulti_a}(k,…))` strings (bypassing build-descriptor, whose `WrapperKind` is wsh-only BY DESIGN — extending build-descriptor itself would be a feature/MINOR, out of scope for a test cycle), entering at `bundle`; O1/O2/O3 oracles reuse unchanged (patched-miniscript parses + derives both tap multisig forms); + the never-shipped non-NUMS-tr loud-refusal negative cell (SPEC :35 promised it).
- Blockers: none — restore's taproot arm shipped v0.49.1; the domain is narrow (NUMS + single multi_a/sortedmulti_a leaf) but it is the entire reconstructable taproot surface, currently covered only by fixed goldens.
- Could it find a bug? Plausible (k/n randomization + sortedmulti_a ordering + O2 fixed-point through the v0.48.0 NUMS emit path were never property-tested), but lower odds than (a).

**(c) bitcoind breadth — two distinct items, highest infra cost.**
- (c1) md-codec corpus extension (sibling repo): +4-6 `Shape` rows (plain `multi`, a hashlock shape, `after`, `or_d`/`andor`, maybe multi-leaf taptree pending a render probe). Cheap per-row (~30-45 lines, uniform handling), CI already runs daily; each row needs one local derivability proof run. No tr-sortedmulti_a (13.0.0 gap).
- (c2) Toolkit bitcoind oracle: NEW — a deterministic end-to-end test (bundle→restore→addresses vs Core `deriveaddresses`, reusing STRESS-A's seed-42 smoke shapes + the tr leg rather than proptest-under-CI) + a workflow lifted from md-codec's (pinned v27.0, offline `-chain=main`, connect-only env contract). ~250-line test + ~140-line yml. Tests a surface nothing else covers with an external oracle (the toolkit's own derivation + reconstruction), but it is the most machinery for the least immediate divergence-finding power — Core agrees with rust-miniscript on these shapes far more often than two hand-written md1 walkers agree with each other.

## Recommended scope

**Verdict: GO — split into two cycles; widen the cross-tool corpus first.**

- **Cycle 1 (this cycle): (a) cross-tool corpus widening.** One file, ~8-12 entries + 1-4 frozen key literals, zero workflow/infra change, NO-BUMP test-only — and it directly re-guards the proven re-divergence surface. R0 the entry list (which fragments, the tr-sortedmulti_a ToolError-vs-omit call, the md-cli tap key-depth probe) before writing rows. If a row Diverges, that is a FOUND BUG: stop, recon the divergence (which side is SPEC-conformant, per the v0.55.0 playbook), and let the fix spawn its own (possibly MINOR) cycle with the new row pinned `Diverge` until it ships.
- **Cycle 2: (b) STRESS-A tr leg** + the missing non-NUMS refusal negative cell. Test-only, NO-BUMP, no build-descriptor changes (its wsh-only WrapperKind is a deliberate v1 seam — do not feature-creep it from a test cycle).
- **Defer (c)** behind a fresh FOLLOWUP pair: `bitcoind-differential-corpus-breadth` (descriptor-mnemonic, the +4-6 rows — cheap, can ride any md-codec cycle) and `toolkit-bitcoind-end-to-end-oracle` (toolkit, the new harness — do it after (a)+(b) so the deterministic shape set it pins is the WIDENED one).
- Tier: test-breadth / NO-BUMP for (a) and (b) as written; (a) carries a deliberate found-bug contingency.
