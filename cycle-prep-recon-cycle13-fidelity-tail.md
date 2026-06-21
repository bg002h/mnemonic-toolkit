# Cycle-13 pre-cycle recon — the "fidelity tail" (L8, L9, H11, H14, M1, M7, L18)

**RECON ONLY — no implementation.** Verifies the 7 remaining open bug-hunt findings against
CURRENT origin default-branch source, determines which still reproduce, classifies by repo /
severity / effort, and recommends the final cycle structure.

Canonical source: `design/agent-reports/constellation-bughunt-2026-06-20.md` (this worktree).

## Origin SHAs verified against (all re-fetched this session)

| repo | default branch | HEAD SHA | version |
|---|---|---|---|
| mnemonic-toolkit | `master` | `d55bf4c3` | toolkit v0.65.2 |
| descriptor-mnemonic | `main` | `8c73b4d` | md-codec 0.39.0 / md-cli 0.9.1 |
| mnemonic-secret | `master` | `e80ea3b` | ms-cli v0.9.0 |
| mnemonic-key | `main` | `df7c2eb` | mk-cli v0.10.1 |
| mnemonic-gui | `master` | `1999323` | v0.46.0 |

**Headline: all 7 reproduce on current source. None was incidentally fixed by cycles 1-12.**
6 of 7 live entirely in the toolkit; 1 (L8) is toolkit-fixable but touches an md-codec citation
that is *contextual, not a fix site*. None changes a clap flag → **no schema_mirror impact**.
M7 changes a `--json` wire-shape (GUI paired-PR / manual prose, not gate-enforced).

---

## Per-finding verification

### M1 · `export-wallet --from-import-json` drops the BIP-32 account for single-sig
- **WHAT:** `import-wallet --json` hardcodes `bundle.account: 0`; the single-sig template emitters
  (electrum/coldcard/sparrow) rebuild the origin from `template.origin_path_str(network, account=0)`
  and ignore the real path → a wallet imported at account 5 re-emits `m/84'/0'/0'`. Xpub still
  correct (addresses correct); declared origin no longer matches the key. Multisig unaffected.
- **Repo/files:** toolkit · `cmd/import_wallet.rs`; `wallet_export/{electrum,coldcard,sparrow}.rs`;
  `cmd/export_wallet.rs`.
- **Citation check:**
  - `import_wallet.rs:1547` (`account: 0` literal) → **DRIFTED-by-10**, now `:1557` (the `account: 0`
    field of `BundleJson`; the real path is stored separately in `origin_path`/`origin_paths`). ACCURATE mechanism.
  - `electrum.rs:111` → **ACCURATE** (`template.origin_path_str(inputs.network, inputs.account)`).
  - `coldcard.rs:201` → **ACCURATE** (`origin_path_str(...)` for `deriv`).
  - `sparrow.rs:256` → **ACCURATE** (single-sig `origin_path_str(...)`).
  - The *multisig* emit paths use `s.origin_path_bare()` (per-slot) → correctly unaffected, as the
    report claims.
- **STILL REPRODUCES: YES.** `account: 0` is unconditional at import; `export-wallet
  --from-import-json` reads `envelope.bundle.account` (=0) into `EmitInputs.account`.
- **Severity reality:** **metadata-only (DEMOTED, confirmed).** Oracle: addresses correct; declared
  derivation wrong → PSBT key-origin matching / account discovery may fail. Fidelity/availability,
  not wrong-address.
- **Treatment:** **reviewed-patch lane** (behavioral, but fail-soft + narrow). Fix = decode the real
  account from the origin into `bundle.account` at import (`:1557`), OR make single-sig emitters
  honor `resolved_slots[0].origin_path_bare()` when present. Prefer the emitter-side fix (no
  envelope-schema change). Est. ~25–50 LOC + a non-zero-account round-trip test. Toolkit PATCH.
  No flag/schema/manual change.

### M7 · `bundle … --json` reports `multisig.threshold = N` (cosigner count) instead of K
- **WHAT:** In descriptor / `--import-json` / concrete-descriptor mode `args.threshold` is `None`, so
  the `--json` emitter falls back to the cosigner **count** `n`. The engraving **card** path does it
  right via `extract_multisig_threshold`; the JSON path never calls it. md1 wire correct; only JSON
  metadata wrong.
- **Repo/files:** toolkit · `cmd/bundle.rs`.
- **Citation check:**
  - `:915` (`threshold = args.threshold.unwrap_or(n as u8)` in the JSON branch) → **ACCURATE**
    (current `:915`, same expression).
  - `:922` `MultisigInfo { threshold, … }` → **ACCURATE** (current `:921`).
  - `:924` `path_family` "hardcoded `bip87`" → **PARTIALLY FIXED / STRUCTURALLY-CHANGED.** Current
    `:924` reads `args.multisig_path_family.unwrap_or_default().human_name()` (an "r1 review I-1 fix").
    The `path_family` sub-claim is **no longer reproduced** — it now reflects `--multisig-path-family`
    (defaults bip87 only when unset). The *threshold* sub-claim is untouched.
  - `:1263` (card path uses `extract_multisig_threshold` correctly) → **ACCURATE** (current
    `:1263` computes `descriptor_threshold`; the card `BundleInputForCard.threshold` at `:1312` is
    `args.threshold.or(descriptor_threshold).or(...)`).
- **STILL REPRODUCES: YES, threshold only.** The JSON branch still emits `n` for a descriptor-mode
  2-of-3. `descriptor_threshold` is computed (`:1263`) only for the card block, ~350 lines later —
  the JSON branch never reuses it.
- **Severity reality:** **metadata-only (DEMOTED, confirmed).** Oracle: embedded descriptor + md1
  correct; only `.multisig.threshold` JSON field wrong. Most cosmetic of the 7.
- **Treatment:** **reviewed-patch lane.** Fix = derive threshold in the JSON branch via
  `extract_multisig_threshold(&tree)` (the card already does). The `path_family` half of the original
  finding is already fixed → close that sub-claim. Est. ~10–20 LOC + JSON-shape test. Toolkit PATCH.
  **SemVer/lockstep note:** changes a `--json` wire-VALUE (not flag name) → **no schema_mirror gate**,
  but a GUI `--json` consumer concern (paired-PR discipline) + a `docs/manual` prose touch
  (`45-foreign-formats.md` / SPEC §5.3) — neither gate-enforced.

### L8 · Multisig-template completion hardcodes mainnet coin-type 0' → non-mainnet all-own unrestorable
- **WHAT:** All-own multisig-template completion (no `--cosigner`/`--origin`) builds the own origin
  from `canonical_origin(&d.tree)`, which hardcodes coin `0'`; only the *account* component is
  substituted. Bundles emit cosigner origins via `network.coin_type()` (=1 non-mainnet). So `restore
  --network testnet` derives every own key at `m/48'/0'/…` → never matches → silent NO-MATCH.
- **Repo/files:** toolkit · `cmd/restore.rs` (fix site); md-codec · `canonical_origin.rs` (the
  hardcoded-0' source, but a yes/no canonicity discriminator, not the fix site).
- **Citation check:**
  - `restore.rs:1342-1346` / `:1377-1382` / `:1411-1416` → **DRIFTED / STRUCTURALLY-MOVED.** The
    live mechanism is the `canonical_fallback` closure at `restore.rs:1541` calling
    `canonical_origin(&d.tree)`, consumed at `:1585-1590` where `own_origin_from_family(&fb, acct)`
    substitutes only the **account** component (coin stays `0'`). The `m/48'/coin'` comment is now at
    `:1588`. Mechanism intact, line numbers stale.
  - `md-codec/canonical_origin.rs:61,70` (hardcoded coin `0'`) → **ACCURATE** (current `:61`/`:70`,
    `mk_origin(&[(true,48),(true,0),…])`).
  - `synthesize.rs:1195-1197` → **CONTEXTUAL, not a bug site.** That line is the canonicity
    `is_some()` discriminator (origin elision), unrelated to coin-type. Drop from the fix scope.
- **STILL REPRODUCES: YES.** Confirmed end-to-end at the code level: `canonical_fallback()` →
  coin `0'`; only account substituted → non-mainnet all-own never matches.
- **Severity reality:** **A-wrong-address class but FAIL-SAFE** — produces NO-MATCH, never a wrong
  address. Real availability/restorability bug for testnet/signet/regtest all-own wallets.
- **Treatment:** **reviewed-patch lane (toolkit-only).** Fix = after `canonical_fallback()`,
  substitute `network.coin_type()` into the coin component (index 1) of the returned path — mirror the
  bundle emitter. **DESIGN FORK to flag:** do NOT add a network param to md-codec
  `canonical_origin` — it is a public API used pervasively by the toolkit purely as
  `.is_some()`/`.is_none()` (synthesize.rs ×4, restore.rs ×2, slot_input.rs); a signature change =
  md-codec MINOR + a wide ripple for no benefit. Patch coin-type toolkit-side. **md-codec NO-BUMP.**
  Est. ~15–30 LOC + a testnet all-own round-trip test. Toolkit PATCH.

### L9 · `run_multisig_template_completion` omits the hardened-use-site / taproot-override refusals
- **WHAT:** `run_multisig` refuses `has_hardened_use_site` and an unrestorable `taproot_override_card`
  before reconstruction; the parallel keyless-template completion path carries neither. Low
  reachability on legit cards (named templates are non-hardened; taproot never reaches this path) and
  downstream derive fail-safes to NO-MATCH — so today it is a missing **early actionable refusal**
  (opaque error instead of a precise message) + path inconsistency, latent if a future bundle form
  emits a hardened canonical multisig template.
- **Repo/files:** toolkit · `cmd/restore.rs`.
- **Citation check:**
  - `:1159-1585` "completion path, no guards" → **STRUCTURALLY ACCURATE, line-shifted.** Live
    function `run_multisig_template_completion` at `:1321`; an `awk` scan of `:1321–:2719` finds NO
    `has_hardened_use_site` / `taproot_override_card` guard (only doc comments + the unrelated
    `refuse_at_in_both` taproot @-in-both checks).
  - `:2581-2594` / `:2639-2646` (`run_multisig` guards) → **DRIFTED**, now `:2779`
    (`has_hardened_use_site`) and `:2786` (`taproot_override_card && !restorable_taproot_override_card`).
    Mechanism present.
  - `synthesize.rs:317,320,323` (use_site preserved verbatim) → not re-verified line-exact; the
    preservation claim is consistent with the completion path reaching reconstruction unguarded.
- **STILL REPRODUCES: YES.** The completion path is unguarded; the refusals live only in
  `run_multisig`.
- **Severity reality:** **B-policy-collapse, but LATENT / hardening.** Not a live wrong-address or
  funds-loss today (named templates can't carry hardened use-sites; downstream fail-safes to
  NO-MATCH). It is a defense-in-depth gap → earlier, clearer refusal.
- **Treatment:** **reviewed-patch lane** (mechanical: hoist the same two guards to the top of
  `run_multisig_template_completion`; optionally refuse templates carrying `origin_path_overrides`).
  Est. ~15–25 LOC + RED-then-GREEN guard tests. Toolkit PATCH. **Pairs naturally with L8** (same
  file, same completion path, file-adjacent).

### H11 · Coldcard/Jade multisig export collapses divergent cosigner paths to a wrong global `m/0'/0'`
- **WHAT:** `emit_coldcard_multisig_text` writes a single global `Derivation:` line only when ALL
  cosigner origins are identical; on divergence it silently falls back to the literal placeholder
  `m/0'/0'` and emits NO per-cosigner `Derivation:` lines — though the format (and the toolkit's own
  import parser) supports per-cosigner `Derivation:` overrides. Divergent paths are legitimate
  (collaborative custody). Jade delegates byte-identical → inherits the bug.
- **Repo/files:** toolkit · `wallet_export/coldcard.rs`; `wallet_export/jade.rs`.
- **Citation check:**
  - `coldcard.rs:324-336` → **ACCURATE.** Live code at `:330-337`:
    `derivation = if all-equal { derivations[0] } else { "m/0'/0'".to_string() }`; only ONE
    `Derivation:` line is pushed (region `:361`); the cosigner loop emits `<XFP>: <xpub>` with no
    per-line `Derivation:`.
  - `jade.rs:46` → **ACCURATE** (the multisig arms `=> emit_coldcard_multisig_text(inputs)`).
  - `wallet_import/coldcard_multisig.rs:38-49` (per-cosigner shape proves a faithful form exists) →
    consistent with the import parser supporting per-line `<XFP>:`/`Derivation:`.
- **STILL REPRODUCES: YES.** Divergent paths → `m/0'/0'`, no per-cosigner lines.
- **Severity reality:** **HIGH but metadata-only (DEMOTED, confirmed).** Oracle: watch-only
  addresses unchanged; origins corrupted. Re-import corrupts every divergent cosigner's declared
  path → a Coldcard derives a different xpub at `m/0'/0'` and refuses to register / registers with
  wrong origins → breaks co-sign / fund recognition. **Real device-interop breakage, not
  wrong-address.** Highest-impact of the demoted set (round-trip corruption, not just a JSON field).
- **Treatment:** **full R0 cycle (or a heavily-reviewed patch).** This is a real
  export-fidelity/round-trip funds-availability bug touching a multi-step emitter contract + a
  legitimate collaborative-custody scenario. Fix = when paths diverge, emit a per-cosigner
  `Derivation:` line before each `<XFP>: <xpub>`; shared line only when all agree; **refuse** rather
  than emit `m/0'/0'`. Verify against the Coldcard multisig format spec (per-cosigner `Derivation:`
  is genuinely supported) AND that the toolkit's own import parser round-trips the per-cosigner form.
  Est. ~40–80 LOC + divergent-path export→import round-trip test (Jade covered via delegation).
  Toolkit MINOR (changes export wire-shape for a previously-malformed case). **Manual prose:**
  `45-foreign-formats.md` / `37-wallet-export.md` (not gate-enforced).

### H14 · `coldcard-multisig` import uses the account xpub's own fingerprint as the master fp
- **WHAT:** A BIP-380 key-origin needs the MASTER fingerprint (depth-0), but `xpub.fingerprint()` is
  the account key's own identifier at `m/48'/0'/0'/2'`. With no depth guard: Row 4 (no XFP, accepted
  older-firmware shape) SILENTLY substitutes the account fp as the master fp; Row 2 (real master XFP
  supplied) can essentially never equal `xpub.fingerprint()` at depth>0 → a "disagrees with computed
  fingerprint" WARNING fires on every cosigner of every authentic export. (`json_envelope.rs` does
  the same substitution but with a loud NOTICE; coldcard-multisig Row 4 is silent.)
- **Repo/files:** toolkit · `wallet_import/coldcard_multisig.rs` (+ SPEC §11.4.1 + test fixtures).
- **Citation check:**
  - `:358-360` (`computed_fp = xpub.fingerprint()`) → **ACCURATE** (current `:359-360`:
    `xpub_parse_result.as_ref().ok().map(|x| x.fingerprint())`).
  - `:363-399` (5-row truth table; Row 4 silent substitute, Row 2 spurious warning) → **ACCURATE**
    (current `:363-400`; Row 2 sets `xfp_header_disagreed=true` + warns; Row 4 `(None,Some) =>
    computed` silent). Confirmed **no `.depth()` guard anywhere** in the file.
  - `:936-957` (test fixtures pin FP to `xpub.fingerprint()`) → consistent (tests assert
    `xpub.fingerprint()` equality, masking the bug).
- **STILL REPRODUCES: YES.** No depth guard; account fp substituted as master fp.
- **Severity reality:** **HIGH but metadata-only (DEMOTED, confirmed).** Oracle: addresses identical
  (xpub correct). Breaks PSBT device-matching ("not my key" checks fail on every cosigner) + erodes
  warning signal (false "internally inconsistent" warning on authentic exports). Real, not
  wrong-address.
- **Treatment:** **full R0 cycle (funds-safety-adjacent refusal-semantics change).** Fix changes
  observable behavior in a delicate way: at depth>0 with no XFP → **REFUSE** (master fp unrecoverable
  from an account xpub — child→parent is one-way) rather than substitute; at depth>0 with a supplied
  XFP → accept it as authoritative WITHOUT the disagreement warning; treat `xpub.fingerprint()` as a
  master fp only when the cosigner xpub is itself depth 0. Also fix SPEC §11.4.1 + test fixtures (use
  realistic master fps ≠ `xpub.fingerprint()`). Higher blast-radius: a previously-accepted shape
  (Row 4) now refuses → could surprise users on older-firmware exports → needs the R0 gate to weigh
  refuse-vs-NOTICE (json_envelope substitutes-with-NOTICE; consider parity). Est. ~50–90 LOC +
  fixture rewrite + truth-table tests. Toolkit MINOR (intake refusal change). **Pairs with H11**
  (same coldcard-multisig format, opposite direction: H11 export / H14 import).

### L18 · Electrum import hard-refuses valid wallets with null `root_fingerprint`/`derivation`
- **WHAT:** Both singlesig + multisig Electrum paths require `root_fingerprint`/`derivation` to be
  non-null strings (`.as_str().ok_or_else`). Electrum emits these as JSON `null` for watch-only "use
  a master key" (xpub-import) wallets, older wallets, and (per the tracked round-trip quirk)
  toolkit-emitted files once re-saved by Electrum. The blob sniffs positive then bails.
- **Repo/files:** toolkit · `wallet_import/electrum.rs`.
- **Citation check:**
  - `:513-531` (singlesig) → **DRIFTED-by-~12**, now `:525-544`: `derivation` and `root_fingerprint`
    both `.and_then(|v| v.as_str()).ok_or_else(...)` (null/missing → hard `ImportWalletParse`).
  - `:796-813` (multisig) → **ACCURATE** (current `parse_multisig_cosigner` `:796+`: cosigner `xpub`,
    `derivation` each `.as_str().ok_or_else(...)`).
- **STILL REPRODUCES: YES** in both paths.
- **Severity reality:** **E-panic-dos false-reject (availability), FAIL-SAFE.** Refuses rather than
  corrupting — no funds risk. Real usability bug for watch-only xpub-import Electrum wallets +
  toolkit→Electrum→re-save round-trips.
- **EXTERNAL-PROTOCOL FACT TO VERIFY (do in-cycle):** the claim that Electrum's `keystore.dump()`
  emits `root_fingerprint: null` / `derivation: null` for "use a master key" watch-only wallets is
  plausible and matches Electrum behavior, but is an authoritative-source check per the constellation
  research-phase rule (BIP/SDK facts verified against source text, not the draft). Confirm before
  designing the fallback.
- **Treatment:** **reviewed-patch lane, with the protocol verification gate.** Fix = treat null as
  unknown-origin (`00000000` fp; purpose inferred from the SLIP-132 xpub prefix when `derivation` is
  also null) + emit a NOTICE. **Non-trivial nuance:** the singlesig wrapper/purpose is currently
  inferred from `derivation`; with `derivation` null the purpose must fall back to the SLIP-132
  prefix family (Zpub→wsh, etc.) — design the fallback carefully. Est. ~40–70 LOC + null-fixture
  round-trip tests (singlesig + multisig). Toolkit PATCH. No flag/schema change; optional manual
  prose in `45-foreign-formats.md`.

---

## Cross-cutting observations

- **All 7 reproduce; none was incidentally fixed by cycles 1-12.** One sub-claim died: M7's
  `path_family`-hardcoded-`bip87` half was fixed by the bundle r1-review I-1 fix (now reads
  `--multisig-path-family`); only M7's *threshold* half survives. Close the path_family sub-claim as
  ALREADY-FIXED.
- **6 of 7 are toolkit-only.** L8's only non-toolkit citation (md-codec `canonical_origin.rs`) is the
  *source* of the hardcoded `0'`, but the correct fix is toolkit-side coin-type substitution →
  **md-codec NO-BUMP** (avoid a public-API signature change rippling across ~8 `.is_some()` callers).
- **No CLI flag is added/removed/renamed by any of the 7 → schema_mirror (flag-NAME parity) is
  unaffected; no GUI schema PR required.** M7 alone changes a `--json` wire-VALUE — a GUI `--json`
  consumer concern under the paired-PR discipline, NOT gate-enforced.
- **Manual:** H11/H14/L18/M7 each touch foreign-format prose (`45-foreign-formats.md`,
  `37-wallet-export.md`) but no CLI-reference flag table → manual-mirror lint (flag-name based) won't
  fire; prose updates are voluntary, ship same-PR as good hygiene.
- **No CRITICAL remains** in this tail. Demotion-confirmed metadata-only: H11, H14, M1, M7. Fail-safe
  availability: L8 (NO-MATCH), L18 (false-reject), L9 (latent/hardening).
- **File adjacency / parallelism:** the toolkit splits into 3 natural file-disjoint clusters:
  - **restore.rs cluster:** L8 + L9 (same file, same completion path — must be ONE lane to avoid
    self-conflict).
  - **coldcard-multisig cluster:** H11 (export) + H14 (import) — different files
    (`wallet_export/coldcard.rs`+`jade.rs` vs `wallet_import/coldcard_multisig.rs`) but the same
    format spec / fixtures; co-design as one lane for spec coherence.
  - **import-fidelity cluster:** M1 (`export_wallet`/emitters + `import_wallet`) + M7 (`bundle.rs`) +
    L18 (`wallet_import/electrum.rs`) — file-disjoint from each other and from the above.
  All three clusters are file-disjoint from each other → safely parallelizable as lanes off
  v0.65.2, but they all bump the SAME crate (toolkit) → serialize the version bumps / ship order.

## Recommended scope — minimum sensible cycle structure

Two tiers by risk, three authoring lanes:

**LANE A — full R0 cycle: the coldcard-multisig fidelity pair (H11 + H14).** Both are HIGH (filed),
real device-interop / round-trip funds-availability bugs, with refuse-vs-substitute semantics that
need the R0 gate (H14 turns a previously-accepted Row-4 shape into a refusal; H11 changes export
wire-shape). Co-design (export+import of the same format) → one brainstorm/SPEC/plan, single-subagent
TDD, per-phase + whole-diff review. Toolkit **MINOR** (0.66.0). ~90–170 LOC + round-trip fixtures.

**LANE B — reviewed-patch lane: the restore completion guards + coin-type (L8 + L9).** Same file
(`restore.rs`), both fail-safe, mechanical (coin-type substitution + hoisting two existing guards).
md-codec NO-BUMP. One patch, one review pass. Toolkit PATCH if shipped alone (folds into the MINOR if
batched). ~30–55 LOC.

**LANE C — reviewed-patch lane: the import/JSON metadata trio (M1 + M7 + L18).** All metadata-only or
fail-safe, file-disjoint (`emitters/import_wallet` · `bundle.rs` · `electrum.rs`). M7 trivial
(~15 LOC); M1 narrow (~40 LOC); L18 needs the Electrum-`null` protocol-fact verification before the
fix lands. Bundle into one reviewed patch with M7's path_family sub-claim closed as already-fixed.
~65–140 LOC.

**Lanes A/B/C are file-disjoint and can be authored in parallel.** Because all three bump the same
toolkit crate, **ship order: A → B → C** (A is the MINOR; B+C renumber as PATCH-on-A or fold in).
Simplest delivery: **one toolkit MINOR (v0.66.0) carrying all 7**, structured as Lane-A under the
full R0 gate and Lanes B/C as reviewed-patch phases within the same cycle — this matches the
"fidelity tail" framing and ships the constellation bug-hunt to completion in a single tag.

**Already-fixed / not-a-bug:**
- M7 `path_family` sub-claim — **ALREADY-FIXED** (bundle.rs:924 now derives from
  `--multisig-path-family`). Close; keep only the threshold half.
- `synthesize.rs:1195-1197` (L8 citation) — **NOT A FIX SITE** (canonicity discriminator); drop from
  scope.
- L9 — **not a live funds bug** (latent/hardening); still worth the cheap guard-hoist for an earlier,
  clearer refusal + completion/run_multisig parity.

**No cross-repo companion required** (toolkit-only; md-codec NO-BUMP; no sibling-CLI change). No
FOLLOWUPS sibling-mirror needed unless the L18 Electrum-null fallback surfaces an md/ms/mk parallel
(none expected).
