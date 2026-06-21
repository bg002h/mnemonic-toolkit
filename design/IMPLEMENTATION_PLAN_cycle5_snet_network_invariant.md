# IMPLEMENTATION PLAN — cycle-5 — S-NET network-provenance invariant

Phased TDD execution plan for the R0-GREEN brainstorm spec
(`design/BRAINSTORM_cycle5_snet_network_invariant.md`, **0C/0I** at spec-R0 round 2 —
`design/agent-reports/cycle5-spec-r0-round{1,2}-review.md`). DESIGN ONLY — feeds the
mandatory opus-architect **plan-doc R0 loop to 0C/0I BEFORE any code**. Toolkit-only,
MINOR; no registry publish; **no clap/`--json`-wire/dropdown change → no GUI
schema_mirror, no manual leg, no sibling-codec companion**.

## Source-of-truth SHA
`/scratch/code/shibboleth/mnemonic-toolkit`, `origin/master` **`ac4eead0`**, toolkit
**0.62.1 → 0.63.0** (MINOR). 0.63.0 collides with the paused
`feature/own-account-subset-search` cycle — **first-to-ship claims it; do NOT touch
that branch** (it renumbers).

**Execution model:** a **single implementer in a toolkit worktree off `origin/master`**,
strict TDD (RED before GREEN), **FULL `cargo test -p mnemonic-toolkit`** (whole
package, per `feedback_r0_review_run_full_package_suite`) + `cargo clippy --all-targets
-D warnings` at each phase gate. Re-grep every line the spec cites (line numbers
DRIFT). **NEVER `cargo fmt`** (toolkit `mlock.rs` fmt-exempt + rustfmt skew). The
**two distinct axes** must stay separate throughout (see Phase 2).

---

## Phase 1 — the shared helper + the `NetworkMismatch` variant (foundation)

**The invariant (spec §2.1/§2.2):** `network::assert_network_agrees(decoded: NetworkKind,
asserted: NetworkKind, context: &'static str) -> Result<(), ToolkitError>` — takes
already-extracted `NetworkKind`s (so it serves WIF too), ports the
`synthesize.rs:776-790 CosignerSpec` predicate (`decoded != asserted → reject`),
NetworkKind-granular (Main vs Test, 2-way — matches xpub version bytes). **No-op
precondition:** callers MUST skip the check when no network is asserted (originless /
no-coin-type input — `coin_type_from_path` needs ≥2 path components, `descriptor.rs:199`).

**`NetworkMismatch` variant (spec §2.3, M-4):** keep `&'static str` fields; rename to
`decoded_network`/`expected_network`, add `context: &'static str`; feed names via a new
`network_kind_name(NetworkKind) -> &'static str` const-fn. Already `#[allow(dead_code)]`
at `error.rs:273`, exit **2** at `:587` (verify), Display at `:830`/`:913`. In-place edit
keeps it alphabetical (no re-sort).

**RED tests (write first):** `assert_network_agrees` unit tests — (a) Main vs Test →
`Err(NetworkMismatch{..})`; (b) Main vs Main / Test vs Test → `Ok`; (c) the no-op
contract is exercised by callers (Phase 2-4). **GREEN:** add the module + const-fn +
variant edit. **Gate:** package test + clippy green (the variant rename will touch the
existing Display/exit/kind arms — compile-forced to update them consistently).

---

## Phase 2 — import sites: H15 (7 parsers, axis-2) + H9 (per-entry, axis-1)

**KEEP THE TWO AXES SEPARATE — this is the funds-safety crux (spec §2.3.1):**
- **Axis 1 (H9) — `--network`/class:** extend the existing `first()`-only check
  (`import_wallet.rs:1192` guard → `ImportWalletNetworkClassMismatch` at `:1199`, exit
  **1**) to **ALL** parsed entries (the `iter_mut()` rebind-all is the bug). **Reuse
  `ImportWalletNetworkClassMismatch` (exit 1)** — NOT NetworkMismatch. RED: mixed
  `[Bitcoin, Testnet] + --network bitcoin` → exit 1 (first passes old check, Testnet
  entry now caught). Positive control: same-class `[Bitcoin, Bitcoin] + --network
  bitcoin` → exit 0. **plan-R0 M2 (non-vacuity):** the H9 RED MUST use **`--format
  bitcoin-core`** — it is the ONLY import parser emitting a multi-element `ParsedImport`
  Vec; the other 8 return a single-element `vec![…]`, so a mixed-entry RED is only
  constructible through bitcoin-core.
- **Axis 2 (H15) — xpub-version vs coin-type:** in each of the **7 import parsers**
  (descriptor / specter / sparrow / bitcoin-core / bsms / coldcard_multisig / electrum —
  re-grep exact fns), after the coin-type network resolves, call `assert_network_agrees(
  xpub_networkkind, coin_type_networkkind, "<parser>")` → `NetworkMismatch` (exit 2) on a
  tpub-at-mainnet-coin-type (and vice-versa). **No-op when originless.** RED per parser:
  a coin-type-0 origin carrying a `tpub` → exit 2; positive control: consistent
  tpub-on-coin-type-1 still imports (the existing fixtures must stay green).

**Gate:** FULL package test (the H9 + H15 RED tests RED-first, then green) + clippy.

---

## Phase 3 — convert + export sites (axis-2 `NetworkMismatch`)

- **M14 convert `--xpub-prefix`** (`convert.rs:1100`, presence-only guard `:922`): call
  the helper — re-emitting an xpub into a `--network` family that disagrees with the
  xpub's own version → exit 2. RED + consistent-positive-control.
- **L11 convert `--from wif --to xpub`** (`convert.rs:1480`): extract the **WIF's own
  NetworkKind** (WIF carries a network byte, not a BIP-32 version — spec §5) and pass as
  `decoded`; assert vs `--network`. RED (mainnet WIF + `--network testnet` → exit 2) +
  positive control.
- **M13 export `--from-import-json`** (`export_wallet.rs:742`, `cli_network_from_str(
  envelope.bundle.network)`): cross-check the envelope's declared network against the
  decoded xpubs → exit 2 on disagreement. RED + positive control.

**Gate:** FULL package test + clippy.

---

## Phase 4 — build-descriptor WARN (L1) + L3 ride-along + remaining (L2/L10)

- **L1 build-descriptor (WARN, NOT reject)** (`build_descriptor.rs:476`): the deliverable
  is network-agnostic (display preview only) — **diagnose/warn**, do not hard-fail.
  Infer the display network from keys; per-`DescriptorPublicKey` variant (spec M-5): all
  `Single` (raw pubkey) → unknown → **no warn**; XPub → its NetworkKind. RED: a
  mismatched preview emits a warning (assert on stderr, NOT a non-zero exit); positive
  control: consistent → no warning.
- **L2 electrum-multi** (`:660`) + **L10 bsms**: the coin-type-only network now also
  cross-checks the xpub version via the helper → exit 2. RED + positive controls.
- **L3 ride-along (SPLIT concern — spec §4, decision-firewalled from the network
  helper):** the `u64→u32` account-index truncation at the legacy top-level-xpub fallback
  branch (the `deriv_path_str_opt == None` arm; the live `format!` is **`coldcard.rs:266`**
  — plan-R0 M4, re-grep). **REJECT on `account > u32::MAX`** (never saturate / silently
  rewrite an index) with an appropriate typed error. **plan-R0 M3 (non-vacuity):** the L3
  RED MUST drive a **legacy top-level-xpub fixture** that hits the `deriv_path_str_opt ==
  None` arm (a per-`bipN` fixture takes the other branch and is vacuous); positive control:
  in-range account still bakes the correct origin.

**Gate:** FULL package test + clippy.

---

## Phase 5 — zero-false-reject proof + oracle gate + ship (0.63.0)

**Zero-false-reject proof (spec I3):** the FULL `cargo test -p mnemonic-toolkit` package
sweep must be green — every pre-existing network-consistent fixture (incl. originless
tpub `cli_descriptor_concrete.rs:174`) still passes. This is the over-rejection guard.

**Class-A differential-oracle gate (program plan):** S-NET only ADDS rejections — no
derived address changes for valid input. Run `tests/bitcoind_differential.rs` if a local
bitcoind is available (`#[ignore]`/env-gated): the AGREE rows must stay byte-identical;
the new DISAGREE/reject asserts live in the CLI/unit suites (the oracle can't derive a
"correct" address for corrupt input). If bitcoind is unavailable, NOTE it and rely on the
package suite + the unchanged-AGREE argument (do not block ship on an unrunnable env-gate;
flag it).

**Version sites (release ritual — `project_toolkit_release_ritual_version_sites`):**
toolkit `0.62.1 → 0.63.0` in `Cargo.toml` + **BOTH READMEs** (self-tag/install) +
`scripts/install.sh` self-pin + `fuzz/Cargo.lock` + CHANGELOG (`mnemonic-toolkit
[0.63.0]` — the S-NET invariant, 9 findings, the dead-variant wiring, fail-closed
rejections). **`--json` note:** the `NetworkMismatch` error's `detail_json` adds error
wire-shape keys — honest CHANGELOG note, but NOT a schema_mirror/manual trigger (those
gate flag-NAMES + dropdown VALUES, not `--json` error shape).

**FOLLOWUP / report:** tick **H15, M13, M14, H9, L1, L2, L3, L10, L11** `[ ]`→`[x]` in
`design/agent-reports/constellation-bughunt-2026-06-20.md` with the fixing commit; file
the spec's FOLLOWUP slugs.

---

## Mandatory post-implementation gate
After Phase 4 GREEN, before the version bump/ship: a **mandatory independent adversarial
whole-diff execution review** over the whole toolkit diff — primary targets: (1) the
two-axis separation held (no H9-vs-H15 exit-code conflation), (2) NO over-rejection of any
legitimate wallet (the no-op-on-originless contract + every positive control), (3) the L3
reject-not-saturate, (4) build-descriptor WARN-not-reject. Persist to
`design/agent-reports/`. Ship only after it is GREEN.

## Phase order & disjointness
P1 (foundation) → P2/P3/P4 (sites, can be done sequentially by the single implementer;
each touches distinct cmd files) → P5 (ship). Plan-R0 must converge 0C/0I before any P1
code. Multi-instance: all work in a worktree off `origin/master`; design-trail + report
ticks via a master worktree (as cycles 3/4); do NOT commit on the paused branch.
