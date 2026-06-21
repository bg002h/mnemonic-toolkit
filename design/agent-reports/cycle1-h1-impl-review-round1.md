# IMPL REVIEW — cycle-1 H1 (`verify-bundle` `md1_xpub_match` structural widening) — Round 1

**Reviewer role:** opus reviewer, mandatory per-phase H1 implementation review (CLAUDE.md: not "done"
until 0 Critical / 0 Important). Adversarial; review-only — no production code written.
**Under review:** H1-only diff `git -C <wt> diff c4b46624…HEAD` (HEAD `4ed3bbe9` on branch
`fix/cycle1-h12-h1`, H1 atop H12 `c4b46624`).
**Design basis:** `design/IMPLEMENTATION_PLAN_cycle1_critical_fixes.md` §2 P4 / §6.3 (C-PLAN-1 gate
`tree == && use_site_path ==` + per-`@N` overrides; origins excluded) and the plan-doc R0 round-2
GREEN (`design/agent-reports/cycle1-critical-fixes-plan-R0-round2.md`, incl. m-NEW-1 decode-boundary
basis).
**Canonical sources verified first-hand (NOT the draft):** the RESOLVED dependency
`md-codec 0.37.0` from crates.io — `Cargo.lock` pins
`checksum = fec7cad2…`, source `registry+…crates.io`; the extracted registry source at
`~/.cargo/registry/src/index.crates.io-…/md-codec-0.37.0/src/` was confirmed byte-identical
(`diff -q`) to the `descriptor-mnemonic origin/main @ 54dd765` checkout the plan-doc cites. All struct
defs and the derive path were read from that registry source.
**Date:** 2026-06-20.

---

## VERDICT: **GREEN — 0 Critical / 0 Important**

The central adversarial check — completeness of the compare against EVERY field of
`md_codec::Descriptor` — passes: **no address-driving field is missed.** Every field that feeds the
derived `miniscript::Descriptor` (hence the scriptPubKey / watched-address set) is bound by the H1 gate,
either directly (`tree`, `use_site_path`, `tlv.use_site_path_overrides`) or via the retained subordinate
`pubkeys_match` (`tlv.pubkeys`). The two genuinely-excluded categories (`path_decl` / origin-`tlv`
columns, and `tlv.unknown`) are origin/identity-metadata or forward-compat-only — correctly excluded to
preserve the L14 origin-elision tolerance the B.3 multiset change exists for, and proven non-vacuously
excluded by a live probe. No false-fail path, no Q-WIRE change, scope clean, full suite + clippy green.
Three MINORs, none blocking.

---

## THE CENTRAL CHECK — FIELD-BY-FIELD COMPLETENESS OF THE COMPARE

`md_codec::Descriptor` (registry `encode.rs:16-28`) has **5 top-level fields**. Field `tlv`
(`TlvSection`, `tlv.rs:23-39`) has **5 sub-fields**. I classify each as **address-driving** (feeds the
derived scriptPubKey → MUST be in the gate) vs **origin/identity metadata** (BIP-32 provenance / not the
scriptPubKey → rightly EXCLUDED), grounded in what `to_miniscript::to_miniscript_descriptor`
(`to_miniscript.rs:54-67`) actually reads to build the address.

**Derivation-path ground truth (read first-hand):** the address-producing `miniscript::Descriptor` is
built from exactly (a) `d.tree` → `node_to_descriptor(&d.tree, &keys)` (`:67`); (b) per-key `e.xpub`
(from `tlv.pubkeys`) + `e.use_site_path` (= baseline `d.use_site_path`, OR the per-`@N` entry in
`d.tlv.use_site_path_overrides`) → `build_descriptor_public_key(e, &e.use_site_path, chain)` (`:65`);
(c) the key-ORIGIN annotation `(fingerprint, origin_path)` from `tlv.fingerprints` + `path_decl` /
`tlv.origin_path_overrides` → `assemble_origin_and_xkey` (`:119-130`). Critically (c) becomes the
`DescriptorXKey.origin` `[fp/path]` provenance tag — it does **NOT** alter the derived scriptPubKey
(the address is a function of xpub + use-site path + tree only). `derive_address` (`derive.rs:110`) reads
only `self.use_site_path.multipath` for the chain bound.

| `Descriptor` field | sub-field | address-driving? | in compare? | verdict |
|---|---|---|---|---|
| `n: u8` | — | **No** (derived: key-table size; tree references indices `0..n`; a divergent `n` cannot leave `tree`+`pubkeys` equal) | indirectly (via `tree`+`pubkeys`) | ✓ OK — no independent binding needed; `tree`'s `MultiKeys.indices`/`KeyArg.index` + the pubkey multiset already pin the key universe |
| `path_decl: PathDecl` (`{n, paths: Shared/Divergent(OriginPath)}`) | — | **No** — origin/identity metadata. Feeds only the `DescriptorXKey.origin` `[fp/path]` tag, not the scriptPubKey | **EXCLUDED** | ✓ CORRECT — same L14 elision/canonicalization brittleness as origins; binding it would false-FAIL legit origin-elided backups (the v0.5.0 B.3 class). Proven non-vacuous below |
| `use_site_path: UseSitePath` (`{multipath: Option<Vec<Alternative>>, wildcard_hardened}`) | — | **YES** — `multipath` selects the change-chain step in `derive_address` → fixes the WATCHED ADDRESS SET (`<0;1>` vs `<2;3>`, presence/count) | **IN** (`==`) | ✓ CORRECT — this is the C-PLAN-1 gap a `.tree`-only gate missed |
| `tree: Node` (`{tag: Tag, body: Body}`) | — | **YES** — carries Tag (Multi/SortedMulti/Tr/wrappers), threshold `k`, script-type/nesting (`sh(multi)` vs `sh(wsh(multi))`), order-sensitive `MultiKeys.indices`, AND the taproot internal-key (`Body::Tr{is_nums, key_index, tree}`), timelocks (`Body::Timelock(u32)` for After/Older), hashlocks (`Hash256Body`/`Hash160Body`) | **IN** (`==`) | ✓ CORRECT — see "taproot/timelock/hashlock are inside `tree`" below |
| `tlv: TlvSection` | `use_site_path_overrides: Option<Vec<(u8, UseSitePath)>>` | **YES** — per-`@N` use-site override → resolved into `e.use_site_path` at `:65`/`:91`; the #25/#26 silent-wrong-address class | **IN** (`==`) | ✓ CORRECT |
| `tlv` | `fingerprints: Option<Vec<(u8,[u8;4])>>` | **No** — origin/identity; feeds only the `origin` `[fp/…]` tag (`:124`) | **EXCLUDED** | ✓ CORRECT — origin-metadata, L14 class |
| `tlv` | `pubkeys: Option<Vec<(u8,[u8;65])>>` | **YES** — the cosigner xpub SET; `e.xpub` is the key material at `:122` | **IN** — bound by the retained subordinate sorted-multiset `pubkeys_match` (`verify_bundle.rs:2773-2789`) | ✓ CORRECT — key SUBSTITUTION caught by the set; key PERMUTATION in unsorted shapes caught by `tree`'s `MultiKeys.indices` order |
| `tlv` | `origin_path_overrides: Option<Vec<(u8, OriginPath)>>` | **No** — origin/identity; per-`@N` origin override, feeds the `origin` tag only | **EXCLUDED** | ✓ CORRECT — same L14 class as `path_decl` |
| `tlv` | `unknown: Vec<(u8, Vec<u8>, usize)>` | **No** — forward-compat raw-passthrough; by definition uninterpreted by `to_miniscript`/`derive` in 0.37 (no derivation reads it) | **EXCLUDED** | ✓ ACCEPTABLE — see MINOR n-H1-2 (a wire whose only delta is an `unknown` TLV cannot change the derived address in 0.37; excluding it is correct for the address-set verdict) |

**Taproot internal-key / NUMS, timelocks, hashlocks are ALL inside `tree` (so already covered):**
- Taproot: `Body::Tr { is_nums: bool, key_index: u8, tree: Option<Box<Node>> }` (`tree.rs:49-57`) — the
  internal-key choice (NUMS vs a real `@key_index`) and the tap-script subtree are part of the `Node`
  AST. A real-key-at-trunk vs NUMS divergence, or a divergent tap-tree, changes `tree` → caught by
  `tree ==`. (`derive_address`/`to_miniscript` read NO taproot internal-key field outside `tree`.)
- Timelocks: `Body::Timelock(u32)` (`tree.rs:70`, `After`/`Older`). Hashlocks: `Body::Hash256Body([u8;32])`
  / `Body::Hash160Body([u8;20])` (`tree.rs:66-68`). All inside `tree` → `tree ==` binds them. (This is
  the exact general-policy-collapse class from the v0.54.0 restore fix; the H1 gate does not regress it.)
- Network: NOT a `Descriptor` field at all (network is a toolkit-side CLI arg, applied at render). Not in
  scope for this struct-equality gate; the `bundle`/`verify-bundle` `--network` arg governs it and the
  ms1/mk1 checks bind the seed/xpub material. No gap.

**Conclusion:** the gate `passed = (tree == && use_site_path == && tlv.use_site_path_overrides ==) &&
pubkeys_match` binds **every** address-driving field of `Descriptor`. The only excluded fields are
origin/identity metadata (`path_decl`, `tlv.fingerprints`, `tlv.origin_path_overrides`) and
forward-compat passthrough (`tlv.unknown`), none of which alter the derived scriptPubKey in md-codec
0.37. **No address-driving field is missed — there is no C-PLAN-1-class residual hole.**

---

## NO-FALSE-FAIL (decode-boundary basis) — VERIFIED

- **Both operands are DECODED md1.** `expected_md_decoded = md_codec::chunk::reassemble(&expected_md1_strs)`
  (`verify_bundle.rs:2709-2710`); `desc` is the `Ok(desc)` arm of `supplied_md_decoded`
  (`:2712-2713`). The compare runs only inside the `if wp { … }` wallet-policy branch — both are
  full `Descriptor`s off the decoder.
- **The decoder enforces canonical form**, so semantically-identical wallets decode byte-identical on
  every gate field: `tree`/`use_site_path` are faithful order-significant 1:1 reps (registry
  `use_site_path.rs` has no sort/canonicalize/normalize); the `use_site_path_overrides` `@N` keys are the
  canonicalization-touched column, but the decoder rejects `@0`/baseline-redundant overrides and keeps
  the TLV idx column strictly ascending, so two equal wallets decode to identical `@N` keys. The in-code
  comment cites this decode-boundary basis precisely (m-NEW-1 folded — it does NOT mis-cite
  `validate_multipath_consistency`). No false-fail.
- **Origins are EXCLUDED → no elided-vs-explicit false-fail.** `path_decl` + origin/fp TLV columns are
  not in the gate. Verified the `h1_origin_divergent_but_policy_equal_passes` test is a **real,
  non-vacuous assertion**: an adversarial probe that ADDED `&& expected_md_decoded.path_decl ==
  desc.path_decl` to the production gate flipped that test RED — proving its re-encoded md1 carries a
  genuinely divergent origin (`m/48'/0'/0'/2'` → `…/5'/2'`) AND that excluding origins is load-bearing,
  not a no-op. (The test also clears `tlv.fingerprints`/`tlv.origin_path_overrides`, exercising all
  origin-category exclusions.)

---

## ANTI-VACUITY OF THE 7 TESTS — VERIFIED BY LIVE PROBE (RED→GREEN discriminators)

Adversarial probe: I reverted the production gate to the pre-H1 baseline (forced `policy_match = true`,
i.e. multiset-only) and re-ran. Result — **exactly the 5 `*_fails` discriminators flipped RED, both
clean-negatives stayed GREEN, and the differential row flipped RED**:

| test | baseline (multiset-only) | H1 gate | discriminates |
|---|---|---|---|
| `h1_wrong_threshold_fails` | FAILED (false-GREEN) | ok | `tree` (k) |
| `h1_sorted_vs_unsorted_fails` | FAILED | ok | `tree` (Tag) |
| `h1_script_type_wrapper_fails` | FAILED | ok | `tree` (wsh vs sh(wsh) nesting) |
| `h1_multipath_divergence_fails` (`<0;1>` vs `<2;3>`) | FAILED | ok | `use_site_path` (C-PLAN-1) |
| `h1_multipath_presence_divergence_fails` (`<0;1>/*` vs bare `/*`) | FAILED | ok | `use_site_path` presence |
| `h1_genuine_match_passes` | ok | ok | clean-negative (no over-reject) |
| `h1_origin_divergent_but_policy_equal_passes` | ok | ok | clean-negative (origins excluded) |
| `h1_verify_bundle_rejects_divergent_policy_md1` (differential) | FAILED | ok | end-to-end exit-code verdict |

Each `*_fails` is therefore a genuine RED→GREEN discriminator that the H1 widening flips, NOT vacuous.
The cosigner SET is held identical across cases (same 3 mnemonics → same `tlv.pubkeys` multiset), so the
multiset gate alone passes — isolating the structural gate exactly as the prompt requires. The two
clean-negatives confirm no over-rejection. (Production file restored byte-identical to HEAD after each
probe; confirmed `git diff --stat` empty.) Differential row covers wrong-k / sorted-vs-unsorted /
script-type / `<0;1>`-vs-`<2;3>` multipath via the real `bundle`/`verify-bundle` CLI; the
"different-addresses" premise for `<0;1>`-vs-`<2;3>` rides the harness's existing divergent-multipath
`derive_receive` anchoring; the verdict assertion is exit-code-behavioral (no bitcoind). Sound.

---

## pubkeys_match RETAINED — combined predicate catches the full class

`pubkeys_match` (sorted-multiset over `tlv.pubkeys`) is retained subordinate (`passed = policy_match &&
pubkeys_match`). Combined coverage confirmed:
- **wrong-k / wrapper / script-type / nesting** → `tree` (`MultiKeys.k`, Tag, `sh(multi)` vs
  `sh(wsh(multi))` body) ✓
- **sorted-vs-unsorted** → `tree` Tag (`SortedMulti` vs `Multi`) ✓
- **multipath / change-chain divergence** → `use_site_path` + `use_site_path_overrides` ✓
- **key SUBSTITUTION** → `pubkeys_match` set inequality ✓
- **key PERMUTATION (unsorted)** → `tree`'s order-sensitive `Body::MultiKeys.indices` ✓ (note: for a
  `SortedMulti` shape a pure key permutation is consensus-equivalent and md-codec canonicalizes order, so
  it is not a distinct wallet — correctly not flagged; for `Multi` the `indices` order is load-bearing
  and bound)

---

## Q-WIRE — VERIFIED

- Check NAME stays `md1_xpub_match` on all three arms (`:2794`, `:2821`, `:2845`). Only the `passed`
  predicate widened (`policy_match && pubkeys_match`) plus a new structural-mismatch arm reusing the SAME
  name. No new `--json` `checks[]` id, no field add, no rename → no wire-shape change.
- `cli_json_envelopes` test: **2 passed** (re-ran — green). The mismatch `detail` text change and the
  `expected`/`actual` populated on the new arm are within the existing `VerifyCheck` free-form
  shape (`format.rs:132`, `name: String` + `Option` fields) — wire-safe. No GUI paired-PR / no
  `schema_mirror` (flag-NAMES only) / no manual leg owed.

---

## SCOPE — VERIFIED

- `git diff --stat c4b46624…HEAD` touches exactly TWO files:
  `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (the `emit_multisig_checks` md1 block, hunks at
  `@@ 2725` and `@@ 2743`, + tests at `@@ 3903`) and
  `crates/mnemonic-toolkit/tests/bitcoind_differential.rs` (the H1 differential row). Nothing else.
- The H12 `:1373` mirror (`compute_default_origin_path` caller) is **NOT** touched by the H1 diff
  (the verify_bundle hunks are at 2725/2743/3903 only) — correct, that is H12's surface.
- **No new error variant** (the mismatch is a `VerifyCheck { passed: false }`, not a `ToolkitError`),
  **no new check-id**, **no version/README/install.sh/Cargo.toml churn** (`git diff` over those paths is
  empty). No `cargo fmt --all` / mlock churn.
- Full `cargo test -p mnemonic-toolkit`: **183 `test result: ok` blocks, 0 FAILED / 0 panicked / 0
  `error[E…]`**. `cargo clippy -p mnemonic-toolkit --tests`: clean (`Finished`, no warnings/errors under
  the workspace `-D warnings`).

---

## CRITICAL FINDINGS

**None.**

## IMPORTANT FINDINGS

**None.**

## MINOR / NITS (none block — GREEN stands)

- **n-H1-1 (cosmetic — mismatch `detail` granularity):** the structural-mismatch arm's
  `expected`/`actual` render `format!("{:?}", …tree)` only (`:2827-2828`), so a divergence whose ONLY
  delta is `use_site_path` or `use_site_path_overrides` (the `classes` string names it, but the
  `expected`/`actual` fields show identical `tree` Debug). Purely diagnostic; the `passed:false` verdict
  and the `classes` `detail` string are correct. Optionally include the differing use-site field in
  `expected`/`actual` for operator clarity. Non-load-bearing.
- **n-H1-2 (`tlv.unknown` exclusion — document, don't bind):** `tlv.unknown` (forward-compat raw TLV
  passthrough) is not in the gate. In md-codec 0.37 no derivation path reads it, so a wire differing ONLY
  in an `unknown` TLV derives the identical address set — excluding it is correct for the address-set
  verdict and binding it would risk a forward-compat false-fail. Worth a one-line code/FOLLOWUP note that
  IF a future md-codec promotes an `unknown` tag to address-driving, the gate must be revisited
  (a generic completeness tripwire, not an H1 defect).
- **n-H1-3 (carried from plan m-NEW-1 — already folded):** the in-code comment correctly attributes
  `==`-safety to the DECODE BOUNDARY (not `validate_multipath_consistency`) and notes
  `use_site_path_overrides` is canonicalization-touched but decode-boundary-safe. No action — recorded as
  confirmation the m-NEW-1 polish landed.

---

## WHAT THIS CLEARS

H1 is **GREEN (0 Critical / 0 Important)**. The compare is complete over every address-driving field of
`md_codec::Descriptor` (verified field-by-field against canonical md-codec 0.37 registry source + the
`to_miniscript`/`derive` read-set); the excluded fields are origin/identity-metadata or forward-compat
passthrough, correctly excluded to preserve the L14 origin-elision tolerance and proven non-vacuously
excluded by a live origin-binding probe. No false-fail (decode-boundary basis confirmed), Q-WIRE
unchanged (`md1_xpub_match` name preserved, `cli_json_envelopes` green), scope clean (two files, no
variant/check-id/version churn, H12 `:1373` mirror untouched), full suite + clippy green, and the 7 tests
+ differential row are proven RED→GREEN discriminators isolating the structural gate. **No
address-driving field is missed.** Per-phase H1 gate is satisfied. The three MINORs may be folded
opportunistically; none block.
