# Changelog

All notable changes to `mnemonic-toolkit` are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project follows [SemVer](https://semver.org/spec/v2.0.0.html) with the pre-1.0 convention that the second component (`0.X`) is the breaking-change axis.

Releases under the `tech-manual-vX.Y.Z` tag namespace are documented inline below; the rendered PDF artifact (`m-format-technical-manual.pdf`) ships as a GitHub release asset.

## mnemonic-toolkit [0.83.0] — 2026-07-10

**SemVer-MINOR — funds-safety fix: network fail-open closed on five edges (constellation-eval F3). An asserted `--network` (explicit OR the clap default) that disagrees with a key's version bytes now REFUSES fail-closed (exit 2) instead of silently minting a wrong-network address/derivation/re-encoding from the key.** Previously `convert` (xpub→address), `xpub-search address-of-xpub`, `silent-payment` (xprv/tprv master), `export-wallet` (`--template`/`--slot` electrum/coldcard SLIP-0132 rewrite), and `export-wallet --descriptor --format bsms` (4-line first-address) used the asserted network directly with no agree-check — so e.g. a `tpub` + `--network mainnet` yielded a mainnet address whose key lives under the testnet subtree (invisible to a standard mainnet restore), and `export-wallet --template bip84 --slot @0.xpub=<tpub> --format electrum` minted a mainnet `zpub` **on the default network with no flag at all**. This release brings all five edges to parity with the already-guarded `addresses` / `convert` WIF→xpub / `export --from-import-json` prior art via `assert_network_agrees` (reusing the existing `ToolkitError::NetworkMismatch`, exit 2). Only the asserted-network arm is guarded — network inference (no `--network`), network-agnostic seed/phrase/ms1 inputs, `Single` raw-hex descriptor keys, and verbatim descriptor passthrough (`--format descriptor`/`bitcoin-core`/`bip388`) are untouched (no over-rejection). The BSMS guard covers both `XPub` and BIP-389 `MultiXPub` (`<0;1>/*`) via miniscript's `xkey_network()`. Toolkit-only; md/mk/ms NO-BUMP; no clap flag → no GUI/`schema_mirror` change; no crates.io publish; no re-vendor. The GUI `--network` dropdown-default amplification is a deferred companion (the toolkit fail-closed fix closes the funds risk regardless; the GUI consumes a pinned binary). Full R0 pipeline GREEN (Fable SPEC ×4, plan ×1, post-impl whole-diff — the loop caught three live-reproduced mints, E4/E5/the E5 multipath variant, beyond the original three edges); full `cargo test -p mnemonic-toolkit` 3693 passed / 0 failed; clippy `-D warnings` clean.

### Fixed

- **[funds] network fail-open on five edges** (`convert.rs` xpub→address, `xpub_search/address_of_xpub.rs`, `silent_payment.rs` xprv/tprv master, `export_wallet.rs` `--template`/`--slot` slot loop, `wallet_export/bsms.rs` 4-line first-address). Each asserts the key's own embedded network (`xpub.network`/`xpriv.network`/`xkey_network()`) against the asserted `--network` before rendering/deriving/re-encoding, refusing on disagreement (`NetworkMismatch`, exit 2). Closes constellation-eval F3 (incl. the M-severity `xpub-search`/`silent-payment` sub-findings). A latent test fixture that paired mainnet cosigner xpubs with `--network testnet` (the exact hazard) was corrected to testnet-version-byte-swapped copies of the same key material.

### Changed

- **`export-wallet --format bsms` 4-line now also fail-closes a hand-crafted `--from-import-json` envelope whose descriptor keys contradict its declared network** (whole-diff review M-1). Strictly fail-closed widening consistent with F3; toolkit-produced envelopes are network-consistent by construction and unaffected.

## mnemonic-toolkit [0.82.0] — 2026-07-09

**SemVer-MINOR — two independent toolkit hardening/capability follow-ups (Cycle G): (1) the repair engine's corrected-secret buffers now zeroize-on-drop, and (2) `compare-cost --descriptor` now accepts `wsh`/`tr` multipath (`/<0;1>/*`, `/**`) instead of rejecting all multipath.** Both are toolkit-only; md/mk/ms NO-BUMP; no GUI/`schema_mirror`/manual-CLI-surface change; no crates.io publish; no re-vendor. Full R0 pipeline GREEN (Fable spec ×3, plan ×1, post-impl whole-diff); full `cargo test -p mnemonic-toolkit` 3671 passed / 0 failed; clippy `-D warnings` clean.

### Security

- **[secret-hygiene] repair-engine corrected-secret buffers zeroize-on-drop** (`crates/mnemonic-toolkit/src/repair.rs`; `src/secret_string.rs`; `src/cmd/repair.rs`; `src/cmd/verify_bundle.rs`). The repair engine's owned secret-bearing buffers migrate from plain `String`/`Vec<String>` to the existing redacting `SecretString` (`Zeroizing<String>` inner → zeroize-on-drop; length-only `Debug`; transparent `Serialize`/`Display`): `RepairOutcome.corrected_chunks` (`Vec<String>` → `Vec<SecretString>`), `RepairDetail.original_chunk`/`corrected_chunk` (`String` → `SecretString`). **NO WIRE CHANGE** — the transparent `Serialize`/`Display` keep `mnemonic repair` `--json` and text output byte-identical (the corrected chunk is deliberately emitted for the repair UX); no emitter uses `{:?}` (which now redacts). Verified live byte-for-byte against a from-source v0.81.0 build across `--ms1`/`--mk1`/`--md1` repair. Closes FOLLOWUP `repair-engine-outcome-zeroization`. (A sibling indel-recovery buffer `IndelCandidate.recovered` remains a plain `String`, newly filed as `indel-candidate-zeroization`.)

### Changed

- **`compare-cost --descriptor` now accepts `wsh`/`tr` multipath descriptors** (`crates/mnemonic-toolkit/src/cost/strip.rs::translate_descriptor`). Previously ALL multipath (`/<0;1>/*`, and the Cycle-C-expanded `/**`) rejected with "multipath key cannot be a DerivedDescriptorKey". `translate_descriptor` now splits the multipath FIRST (`into_single_descriptors()` + is_empty guard + `remove(0)`, mirroring `derive_address.rs`) and costs the receive branch (cost is chain-index-independent), so `/**` == `/<0;1>/*` == `/0/*` cost identically. `wpkh`/`pkh`/`bare` multipath still correctly reject as `UnsupportedWrapper`; malformed inconsistent-branch-count multipath errors cleanly at parse (exit 2). Closes FOLLOWUP `compare-cost-multipath-descriptor-unsupported`.

## mnemonic-toolkit [0.81.0] — 2026-07-09

**SemVer-MINOR — CRITICAL funds-safety fix: an ms1 seed-card BCH repair is now a VERIFY-ME candidate, never a silent "recovered".** An `ms1` card is a single-string BIP-39-entropy bearer secret with NO cross-chunk hash and no internal redundancy, so a >4-error BCH substitution "correction" can alias onto a DIFFERENT valid seed with zero downstream signal that it is wrong — and auto-repair fired by default on TTY `convert`/`inspect`/`verify-bundle`/`xpub-search` intake, so a corroded ms1 could silently yield a WRONG SEED. This release DEMOTES every ms1 substitution-correction to an exit-4 VERIFY-ME **candidate** (never a silent exit-5 "recovered") with an "UNVERIFIED — confirm independently before use; BIP-93 recommends confirming a corrected codex32 string" advisory; auto-repair no longer silently applies a corrected ms1 (it falls through with the advisory). Inside `verify-bundle`, where the user's TYPED seed is the ground truth, the corrected ms1 is compared byte-for-byte to the expected card (`expected.ms1[i]`): a MATCH lets the ms1 checks pass ("recovered via auto-repair, confirmed against expected seed"); a MISMATCH fails `ms1_entropy_match` with a redacted detail (no seed bytes) → exit 4 with the full check table. A unique `mnemonic repair --max-indel` recovery still exits 5 (indel recovery re-validates the FULL checksum, a genuine self-verification unlike substitution). This closes the last, highest-consequence leg of constellation-eval F4 (mk1/md1 shipped in v0.80.0). Cross-repo lockstep: `ms-cli v0.14.0` (`ms repair` demoted identically; sibling pin advanced `ms-cli-v0.13.2` → `ms-cli-v0.14.0`). ms-codec / mk-codec / md-codec NO-BUMP. No GUI / `schema_mirror` change (no clap-surface change). Full R0 pipeline GREEN (Fable spec ×3, plan ×3, per-phase P0/P1/P2, post-impl whole-diff).

### Fixed

- **[funds] ms1 repair demote-to-candidate** (`crates/mnemonic-toolkit/src/repair.rs` Ms1 arm; `src/cmd/verify_bundle.rs::ms1_ground_truth_compare`; sibling `ms-cli` `ms repair`). Any ms1 substitution-correction (`repairs` non-empty) now yields `SetVerify::Unverified` → exit 4 (`mnemonic repair --ms1` / `ms repair`) or a non-short-circuiting fall-through advisory (auto-repair) — never a silent exit-5 blessing. In `verify-bundle`, the corrected ms1 is confirmed against the typed-seed ground truth (`expected.ms1[i]`); a wrong-fit correction (e.g. a corroded card for a different wallet) fails the entropy-match check and exits 4 rather than being reported as recovered. The `--json` repair envelope gains a `verdict` field (`"blessed"`/`"candidate"`, after `kind`).

### Changed

- **`RepairJson` `--json` envelope is now a superset** (`mnemonic repair` + `ms repair`): a `verdict` field is inserted after `kind`. `md repair` / `mk repair` are NO-BUMP and retain the prior field set; a shared-field parser still reads all four, and byte-exact parity holds between `mnemonic repair --ms1 --json` and `ms repair --json`. Downstream `--json` consumers self-update (not `schema_mirror`-gated).

## mnemonic-toolkit [0.80.0] — 2026-07-07

**SemVer-MINOR — CRITICAL funds-safety fix: mk1 repair now RE-VERIFIES a BCH correction by full-set reassembly before blessing it.** The codex32 BCH code guarantees correction only for ≤4 character substitutions; beyond that, bounded-distance decoding can ALIAS a corrupted mk1 chunk onto a *different valid codeword* (a "miscorrection"). Previously, `mnemonic repair`/auto-repair (`convert`/`inspect`/`verify-bundle`/`xpub-search` intake) — and the sibling `mk repair` (mk-cli) — corrected each chunk with BCH and returned success (exit 5 / auto-applied) **without re-checking that the corrected chunks reassemble into a self-consistent card**, so a >4-error wrong-fit repair was blessed as recovery of a *different wallet's* key card. This release closes that: after per-chunk correction, the full card set is reassembled through `mk_codec::decode` (which enforces the cross-chunk SHA-256 hash) and the outcome is a **tri-state**: a full-set correction that fails reassembly is **REJECTED** (exit 2, funds message naming the `chunk_set_id`) instead of blessed; a **single-plate** (partial-set) correction — which cannot be set-verified and is a documented per-plate workflow — is preserved as an **UNVERIFIED candidate** (`mnemonic repair` exit 4 / `mk repair` exit 5, with a loud "reassemble the full card to confirm; BIP-93 recommends confirming a corrected codex32 string" advisory); a genuine ≤4-substitution full-set correction still blesses (exit 5). The just-past-threshold (5-substitution) miscorrection rate was empirically measured in-repo at a 95% Clopper-Pearson upper bound of ≈7.2×10⁻⁵ (a pinned, seeded, reproducible harness). md1 was already protected (its content-derived cross-chunk id check runs unconditionally) and is regression-locked; ms1 (single-string raw entropy, no set-level oracle) is the higher-consequence remaining leg, tracked as F4 Cycle 2 (a separate advisory-semantics cycle). Cross-repo lockstep: `mk-cli v0.12.0` (sibling pin advanced `mk-cli-v0.11.2` → `mk-cli-v0.12.0`); mk-codec/md-codec/ms-codec NO-BUMP (existing public API reused). Full R0 pipeline GREEN (SPEC ×3, plan ×2, per-phase P0/P1, post-impl whole-diff). No GUI/`schema_mirror` change (no clap-surface change).

### Fixed

- **[funds] mk1 repair set-level re-verify** (`crates/mnemonic-toolkit/src/repair.rs` `verify_mk1_set` + `repair_card` Mk1 arm + `try_repair_and_short_circuit`; sibling `mk-cli` `classify_mk1_set`). A corrected mk1 card set is presented as *recovered* (exit 5 / auto-repair short-circuit) IFF `mk_codec::decode` on the exact supplied corrected group returns `Ok`; any decode failure on a complete-and-consistent group is a REJECT (exit 2), and an incomplete (single-plate) group is an UNVERIFIED candidate — never a silent bless. Multi-group/batch invocations fold to the dominant outcome (reject > candidate > bless), and a rejected group's chunks are never emitted as recovered. Closes the wrong-fit-miscorrection-blessed-as-recovery funds bug (constellation-eval F4, mk1 leg).

## mnemonic-toolkit [0.79.0] — 2026-07-07

**SemVer-MINOR — CRITICAL funds-safety fix: reject a concrete non-ranged xpub instead of silently ranging it.** A concrete descriptor `wpkh([fp/84'/0'/0']xpub)` with NO derivation suffix after the xpub (no `/*`, no `/<a;b>/*`) was SILENTLY ranged to `wpkh(@0/*)` on encode — because md1's `UseSitePath` always encodes a wildcard use-site — engraving a **materially different (ranged, multi-address) wallet** than the single fixed key the descriptor named. Worse, `verify-bundle --descriptor` re-parsed the same no-wildcard descriptor through the same collapsing path and **FALSE-PASSED (exit 0)** against that wrong card. This release makes it FAIL-CLOSED: a concrete `[fp/path]xpub` with no derivation suffix is REJECTED at the substitution layer (`DescriptorParse`, exit 2) rather than silently mis-encoded, closing both the encode silent-accept and the `verify-bundle` false-pass at one choke point (the reject fires at re-parse, BEFORE card comparison). This is the last of the descriptor use-site-collapse residuals (Cycle A design decision D1). Full R0 pipeline GREEN (SPEC R0, plan R0, per-phase P0, post-impl whole-diff). md-codec / mk-codec / ms-codec NO-BUMP (toolkit-only; md1's multipath-centric `UseSitePath` is the correct invariant, not changed here). No GUI / `schema_mirror` / manual lockstep (no CLI-surface change).

### Fixed

- **[funds] Concrete non-ranged xpub reject** (`crates/mnemonic-toolkit/src/wallet_import/pipeline.rs::concrete_keys_to_placeholders`). A `[fp/path]xpub` key match not immediately followed by a `/` derivation (i.e. followed by a use-site terminator — `)`, `,`, `}`, `#`, or end-of-string) is now refused with a funds-framed error naming the offending `@N` and pointing at the remedy (append `/*` for a ranged wallet or `/<0;1>/*` for a receive/change multipath). A ranged key (`/*`, `/<0;1>/*`) is unaffected; a fixed step (`/0/*`) still hits the v0.76.0 residue floor; a hand-typed bare `@N` keyless template is unaffected (the check lives in the concrete-key match loop and never sees `@N`). Applies at every surface that builds an md1 card or verifies one from a concrete descriptor — `bundle`/`verify-bundle --descriptor`, `import-wallet --format descriptor|bsms`, `export-wallet --descriptor`, `xpub-search account-of-descriptor` — via the shared substitution choke point.
- **[funds] `verify-bundle --descriptor` concrete false-pass closed.** A concrete non-ranged descriptor now rejects at re-parse (exit 2) BEFORE card comparison, so a no-wildcard descriptor can no longer verify successfully against a ranged card encoding a different wallet.

## mnemonic-toolkit [0.78.0] — 2026-07-07

**SemVer-MINOR — accept the BIP-388 `/**` combined-wildcard shorthand on descriptor intake.** `/**` is the [BIP-388](https://github.com/bitcoin/bips/blob/master/bip-0388.mediawiki) ("Wallet Policies") shorthand, defined as an exact synonym for `/<0;1>/*` (receive = chain 0, change = chain 1). Since the v0.76.0 fixed-step floor, a bare-literal `/**` (concrete `xpub/**` or `@N/**` template) hard-failed. This release adds a shared `expand_literal_double_star` string pre-expander that rewrites a final-use-site `/**` → `/<0;1>/*` BEFORE the descriptor reaches the parser, on **every literal-descriptor user surface**, so `/**` behaves byte-for-byte identically to the explicit `/<0;1>/*` spelling. Full R0 pipeline GREEN (SPEC ×2, plan ×3, per-phase P0, post-impl whole-diff). md-codec / mk-codec / ms-codec NO-BUMP; no GUI / `schema_mirror` impact (no clap surface change).

### Added / Changed

- **BIP-388 `/**` shorthand accepted on descriptor intake** (`crates/mnemonic-toolkit/src/parse_descriptor.rs::expand_literal_double_star`). A final-use-site `/**` (immediately followed by `)`, `,`, `}`, whitespace, `#`, or end-of-string) is expanded to `/<0;1>/*` at every literal-descriptor entry point: `import-wallet --format descriptor|bsms` + its `--json` canonicalize, `bundle --descriptor` (concrete + `@N` template), `verify-bundle --descriptor`, `export-wallet --descriptor`, `xpub-search account-of-descriptor --descriptor`, and `gui-schema --classify-descriptor`. Precision: only the exact `/**` is expanded — `/***` and `/**'` are left untouched (and keep their existing reject), a `/**` inside an origin/key is never rewritten, and each `/**` in a multisig expands independently. A fixed step combined with the shorthand (e.g. `/0/**` → `/0/<0;1>/*`) still rejects on the leading fixed step (the v0.76.0 floor is not weakened).
- **`compare-cost --descriptor`** treats `/**` identically to `/<0;1>/*` (equivalence): compare-cost has a pre-existing limitation — it rejects ALL multipath `/<0;1>/*` descriptors (no `into_single_descriptors` split) — so `/**` now rejects with the same "multipath key cannot be a DerivedDescriptorKey" error rather than a cryptic "invalid child number format". Tracked as follow-up `compare-cost-multipath-descriptor-unsupported`.

### Fixed / Docs

- **BIP attribution corrected `/**` = BIP-388, not BIP-389** across source comments, the reject message, tests, and the manual (`docs/manual/src/40-cli-reference/41-mnemonic.md`, `45-foreign-formats.md`). BIP-389 ("Multipath Descriptors") defines only the explicit `/<0;1>/*` form; those references are unchanged. The residue-reject message no longer names `/**` (it is now accepted); the manual's "Non-representable use-site steps" section documents the expansion.

## mnemonic-toolkit [0.77.0] — 2026-07-06

**SemVer-MINOR — restores standard Bitcoin Core `listdescriptors` import, which was INTERIM-BROKEN since v0.76.0.** Bitcoin Core exports a wallet's receive (`/0/*`, `internal:false`) and change (`/1/*`, `internal:true`) chains as two SEPARATE single-path descriptors (BIP-389 multipath `<0;1>` is import-only, never emitted — verified vs bitcoin/bitcoin PR #22838). Since the v0.76.0 fixed-step floor, a standard Core export hard-failed. This release adds a parse-time pre-pass (`wallet_import/bitcoin_core.rs`) that recombines a **same-key** receive/change pair into one `<0;1>/*` multipath entry, restoring the mainstream import path — WITHOUT weakening the floor for anything that is not a provable same-key adjacent pair. Full R0 pipeline GREEN (SPEC ×3, plan ×2, per-phase P0+P1, post-impl whole-diff). md-codec / mk-codec / ms-codec NO-BUMP (pure toolkit parse-layer).

### Added / Changed

- **[funds] Bitcoin Core receive/change pair-merge on import** (`crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs`). A pre-pass before the per-entry parse loop recombines a same-key `.../0/*` + `.../1/*` pair into one `.../<0;1>/*` multipath descriptor. **Maximally-strict guard matrix** — merges IFF ALL hold: identical grouping key (script template + threshold + ORDERED per-key `(fingerprint, full origin path, xpub)`, EXCLUDING the final step) · each side a fixed single unhardened wildcard final step with matching wildcard-hardness · steps differ · `internal` flags disagree · exactly two share the grouping key · multi-key uniformity (all keys go `/recv`→`/chg` together). ANY deviation → do not merge. Distinct keys are DIFFERENT wallets and NEVER merge; a distinct-key receive/change-shaped pair is refused with a differentiated exit-2 message. Covers `wpkh`, `wsh(multi/sortedmulti)`, `sh(wsh(...))`, and single-key BIP-86 `tr`; a script-path `tr` (tapscript leaves) is out of scope and falls to the floor reject. The merged descriptor's BIP-380 checksum is recomputed; each input entry's own checksum is validated before consuming (a corrupt entry is never silently "repaired"). Extraction is via rust-miniscript (`Descriptor::<DescriptorPublicKey>::from_str`), never the fixed-step-rejecting lexer. Guarded by an original-anchored address oracle (merged chain-0/chain-1 derived from the merged descriptor must equal addresses independently derived from the ORIGINAL split `/0/*` and `/1/*`) so a misfired merge cannot false-pass `verify-bundle`.
- **[wire-shape] `import-wallet` `source_metadata.internal` is now nullable** (`Option<bool>`). A merged multipath entry carries `internal: null` in `--json` (and `bundles[i].internal=both` in the text summary) and satisfies BOTH `--select-descriptor active-receive` and `active-change`. A passthrough single-path entry is unchanged (`true`/`false`). This is an un-gated wire-shape change; the `mnemonic-gui` consumer is updated in a paired PR. No clap flag/subcommand/dropdown change (the manual flag-mirror lint and GUI `schema_mirror` gate are UNTOUCHED).

### Fixed / Docs

- **Manual prose corrected** (`docs/manual/src/40-cli-reference/41-mnemonic.md`, `45-foreign-formats.md`, `30-workflows/39-cross-format-conversion.md`). Removed the factually-wrong "Bitcoin Core 25+ emits `listdescriptors` output with the `<0;1>/*` multipath shape" claim (Core emits the SPLIT pair; multipath is import-only) and retired the hand-combine-to-`<0;1>/*` + `--format descriptor` workaround, replacing it with the native auto-recombine. Recipe 2 (`recipe-2-bitcoin-core-to-bundle`) rewritten to the native `--format bitcoin-core` flow (goldens byte-identical — native merge produces the same cards as the retired hand-combine).

## mnemonic-toolkit [0.76.0] — 2026-07-06

**SemVer-MINOR — CRITICAL funds-safety fix: descriptor use-site collapse (constellation-eval C1). The descriptor lexer (`lex_placeholders`) silently DROPPED a fixed use-site derivation step (`@0/0/*`, `@0/0h/*`) or the BIP-389 combined-wildcard shorthand (`@0/**`), collapsing `…/0/*` to a bare `…/*` — which derives a DIFFERENT address set (`xpub/i` instead of `xpub/0/i`) and encoded the WRONG wallet into the md1 card. `verify-bundle` re-parsed through the same collapsing lexer and FALSE-PASSED (exit 0) on the wrong card. This release makes the lexer FAIL-CLOSED: a fixed step or `/**` shorthand after a placeholder is now REJECTED (`DescriptorParse`, exit 2 on the encode/import and concrete-`verify-bundle` paths; `DescriptorReparseFailed`, exit 4 on the `@N`-template verify path) rather than silently mis-encoded, because md1's `UseSitePath` can only represent the multipath `/<a;b>/*` form (or a bare `/*`). The verify-bundle concrete false-pass is closed: the reject now fires at re-parse, BEFORE any card comparison. This is a BREAKING behavior change (previously-accepted `/0/*` and `/**` imports now hard-fail), hence the MINOR bump. Additive to the CLI SURFACE only in behavior — no new flag / subcommand / dropdown value, so the manual flag-mirror lint and the GUI `schema_mirror` gate are UNTOUCHED. md-codec / mk-codec / ms-codec NO-BUMP (the toolkit catches up to md-cli's already-shipped M5 reject; `UseSitePath`'s inability to hold a fixed step is the CORRECT invariant enforced here, not a codec change).**

### Fixed

- **[funds] Use-site residue reject in `lex_placeholders`** (`crates/mnemonic-toolkit/src/parse_descriptor.rs`). After each `@N[..]` placeholder match, a derivation step that is not a legal terminator (`)` `,` `}`, whitespace, or end-of-string) is refused rather than left as silently-dropped residue. Catches `@0/0/*`, `@0/0h/*`, bracketed-origin `@0[fp/path]/0/*`, post-multipath `@0/<0;1>/0/*`, pre-multipath `@0/0/<0;1>/*`, bare-unbracketed-origin `@0/48h/0h/0h/<0;1>/*`, and the `/**` shorthand. Mirrors `md-cli`'s already-shipped M5 reject (`descriptor-mnemonic/crates/md-cli/src/parse/template.rs`). Placed after the multipath-body validator so a hardened/malformed `<…>` body keeps its byte-exact H13 reject. Reuses `ToolkitError::DescriptorParse` (exit 2) — no new error variant. Applies uniformly to every intake surface (`bundle --descriptor`, `bundle --import-json` replay, `import-wallet --format descriptor|bitcoin-core|specter|bsms`, `verify-bundle`).
- **[funds] `verify-bundle` concrete-descriptor false-pass closed.** A concrete descriptor carrying a fixed use-site step now rejects at re-parse (exit 2, `DescriptorParse`) BEFORE comparison against the supplied cards, so a `/0/*`-collapsed descriptor can no longer verify successfully against the wrong card. The `@N`-template verify path rejects at exit 4 (`DescriptorReparseFailed`).

### Notes / interim limitations

- **Bitcoin Core receive/change split imports hard-fail (interim).** `bitcoin-cli listdescriptors` exports the receive (`/0/*`, `internal:false`) and change (`/1/*`, `internal:true`) branches as two SEPARATE non-multipath descriptors — Core has never exported the combined `/<0;1>/*` multipath form. Each entry now hits the residue reject, so a standard Core export hard-fails. **Workaround:** combine the pair into one `…/<0;1>/*` descriptor (index 0 = receive, 1 = change) and import via `--format descriptor`. Automatic receive/change recombination is a planned follow-up cycle (`bitcoin-core-receive-change-pair-merge`).
- **`/**` combined shorthand hard-fails.** The BIP-389 `xpub/**` shorthand is refused; use the explicit `…/<0;1>/*` form. First-class `/**` → `<0;1>` expansion is a filed follow-up (`bip389-double-star-shorthand-support`).
- **Deferred residual:** a CONCRETE non-ranged xpub with no wildcard (e.g. `wpkh([fp/84'/0'/0']xpub)`) still silently gains a `/*` on restore. This is a distinct mechanism (indistinguishable from a legitimate keyless template at the lexer) tracked as a separate upstream follow-up (`concrete-nonranged-xpub-implied-wildcard`); it is NOT introduced by this release.
- **Documentation:** `docs/manual/src/40-cli-reference/41-mnemonic.md` (authoritative "Non-representable use-site steps" section + verify-bundle exit-code split) and `docs/manual/src/45-foreign-formats.md` (Bitcoin Core interim hard-fail + combine workaround; Specter/BSMS notes) document the reject and workarounds. Sparrow/Coldcard/Electrum are unaffected (they synthesize the multipath form before lexing).

## mnemonic-toolkit [0.75.0] — 2026-06-28

**SemVer-MINOR — `mnemonic inspect <md1>` now emits a default-on keyless BIP-388 `template:` line. Inspecting an `md1` descriptor card now renders the wallet policy with its bound xpubs replaced by their keyless `@N` template placeholders (BIP-388 form) on a new `template:` output line — visible by default in both the text and `--json` surfaces — and the rendered string is BYTE-IDENTICAL to `md decode`'s template output. The descriptor → template renderer was relocated INTO `md-codec` 0.40.0 as the single source of truth (`md-cli` 0.11.3 now delegates to it rather than carrying its own copy), so the toolkit, `md`, and any future consumer cannot drift. `INSPECT_SCHEMA_VERSION` is bumped `1` → `2` (additive: the new `template` field appears in the `md1` `--json` body; the `ms1` / `mk1` `--json` envelopes now report `"schema_version": "2"` but their bodies are UNCHANGED). A new `ToolkitError::Render` variant surfaces a structural render failure (exit code 2). Additive output only — no flag / subcommand / dropdown surface change, so the manual flag-mirror lint and the GUI `schema_mirror` gate are UNTOUCHED. Consumes the relocated renderer published in `md-codec` 0.40.0 (`mk-codec` NO-BUMP, stays 0.4.1).**

### Added

- **Default-on `template:` line in `mnemonic inspect <md1>`** (`crates/mnemonic-toolkit/src/cmd/inspect.rs`). For an `md1` descriptor card, `inspect` now renders the BIP-388 keyless template — each bound xpub replaced by its `@N` placeholder — on a `template:` line in the text output and as an additive `template` field in the `--json` body. The rendered template is byte-identical to `md decode`'s output (shared renderer, see below). The `ms1` / `mk1` inspect surfaces are unchanged apart from the envelope's `schema_version` field.
- **`ToolkitError::Render(md_codec::RenderError)`** (`crates/mnemonic-toolkit/src/error.rs`). Surfaces a structural template-render failure from the relocated `md-codec` renderer as `inspect: template render failed: …` (exit code 2). Alphabetically ordered among the post-v0.27.2 variants.

### Changed

- **`md-codec` `"0.39.1"` → `"0.40.0"`** — the descriptor → keyless-template renderer is relocated INTO `md-codec` 0.40.0 as the single source of truth; `md-cli` 0.11.3 delegates to it (no longer carries its own copy), and the toolkit's `inspect` consumes the same code path, guaranteeing the `inspect` `template:` line and `md decode` cannot drift. `mk-codec` stays `"0.4.1"` (NO-BUMP).
- **`INSPECT_SCHEMA_VERSION` `"1"` → `"2"`** (`crates/mnemonic-toolkit/src/cmd/inspect.rs`). Additive bump for the new `template` field on the `md1` `--json` body. The `ms1` / `mk1` `--json` envelopes report the new `"2"` but their bodies are byte-for-byte unchanged.

## mnemonic-toolkit [0.74.0] — 2026-06-26

**SemVer-MINOR — `mnemonic word-card`: re-encode an `mk1` / `md1` card as an engravable BIP-39 Word Card. A new visible `word-card` subcommand re-renders a PUBLIC `mk1` xpub card or `md1` descriptor card (NOT the secret `ms1` entropy card) as a list of BIP-39 words for steel engraving, with progressive Reed–Solomon error-correction (`--parity-words` / `--parity-pct`), a non-linear integrity tag (`--integrity-bits`) that catches RS miscorrection, sync / checkpoint framing, and an optional cross-plate RAID layer (`--raid 1` = RAID-5 one recovery plate, `--raid 2` = RAID-6 two recovery plates) over an `mk1` xpub array. `--decode` (solo) and repeated `--decode-plate` (RAID array) recover the payload back to the original `m*1` / xpub / descriptor with a repair + truncation report. `--json` emits a versioned envelope. The value engine ships as a new in-workspace `wc-codec` crate (0.1.0). Additive CLI surface — every existing subcommand's output is BYTE-IDENTICAL; the new `word-card` flag set is mirrored into the manual (`docs/manual/`) but the `word-card` `--json` wire-shape is NOT `schema_mirror`-gated (clap-flag-NAME parity only; GUI consumers self-update via the paired-PR rule). Consumes the additive `canonical_payload_bytes` accessors published in `mk-codec` 0.4.1 / `md-codec` 0.39.1.**

### Added

- **`mnemonic word-card`** (`crates/mnemonic-toolkit/src/cmd/word_card.rs`). Encode mode (default): `--from <mk1|md1>` (repeatable; `-` reads one card per line from stdin) re-encodes each card into a solo Word Card; `--raid 1|2` with `≥ 2` `mk1` `--from` cards emits `n` data plates + `r` recovery plates (RAID-5 / RAID-6). Decode mode: `--decode` with positional `<WORD>...` (or `-`/stdin) recovers a solo card; repeated `--decode-plate <WORDS>` RAID-reconstructs a lost data plate from the surviving `≥ n` plates. Error-correction budget via `--parity-words <N>` (RS parity-word count; corrects `⌊N/2⌋` substitutions / fills `N` erasures) OR `--parity-pct <PCT>` (`N = ceil(K·PCT/100)` of the data-symbol count `K`); `--integrity-bits <BITS>` (default 44, min 33) sizes the non-linear SHA-256 truncation tag that bounds an RS miscorrection at `≤ 2⁻ᵗ`. `--json` emits a versioned envelope (`schema_version: "1"`). The global `--no-auto-repair` flag is accepted. The source `mk1`/`md1` cards are PUBLIC watch-only material (xpub / descriptor) — not secret-classified; the `ms1` entropy card is intentionally NOT word-card-able.
- **`wc-codec` crate (0.1.0)** — a new in-workspace member (`crates/wc-codec`) holding the Word-Card value engine: `GF(2¹¹)` Reed–Solomon over the BIP-39 wordlist, progressive parity, sync / checkpoint framing, the integrity tag, the GEOM header / ledger / stop-sign, and the cross-plate RAID (`r = 1` XOR / `r = 2` MDS) layer. The toolkit consumes it via the canonical-payload adapter (`crate::word_card_adapter`), which bridges the sibling codecs' `canonical_payload_bytes` accessors to the engine.

### Changed

- **`mk-codec` `"0.4.0"` → `"0.4.1"` and `md-codec` `"0.39"` → `"0.39.1"`** — consume the additive `KeyCard`/`Descriptor` `canonical_payload_bytes` / `from_canonical_payload_bytes` accessors the Word-Card adapter is built on (non-breaking; the prior published surface is unchanged).
- **Manual mirror** (`docs/manual/`): a `word-card` CLI-reference section added to `src/40-cli-reference/41-mnemonic.md` (every flag documented for the `flag-coverage` lint) and a `mnemonic word-card` line added to `tests/cli-subcommands.list`.

## mnemonic-toolkit [0.73.3] — 2026-06-24

**SemVer-PATCH — LOUD funds-safety advisory for a CUSTOM use-site on a NUMS-taproot card. A `tr(NUMS, multi_a)` multisig card can carry per-cosigner use-site path overrides (divergent derivation suffixes per `@N`). Since #26 (v0.59.1) the toolkit RECONSTRUCTS this shape faithfully — but no known wallet produces it (every standard wallet uses one uniform `<0;1>/*` suffix across all cosigners), so a misconfigured user would silently get non-matching addresses and risk permanent loss of funds. This release KEEPS the faithful reconstruction (proceed-and-warn, not refuse — refusing would strand the rare legitimate user) BUT emits a LOUD `WARNING (funds-safety): …` line at BOTH engrave (`bundle` ×3 paths + `import-wallet`) AND restore (`restore --md1`). BASELINE (uniform-suffix) `tr(NUMS, multi_a)` does NOT warn; un-restorable / refused shapes keep their existing calm `advisory:` notes. Behavior-only stderr advisory; no API / `--json` wire-shape / CLI-flag / subcommand change → the manual flag-mirror lint and the GUI `schema_mirror` gate are UNTOUCHED. md-codec / mk-codec NO-BUMP. The FOLLOWUP `restore-md1-taproot-use-site-override-arm` stays PARTIALLY RESOLVED (the `tr(sortedmulti_a)`, non-NUMS-internal, and hardened taproot-override legs remain open/refused).**

### Added

- **LOUD funds-safety advisory for a CUSTOM use-site on a NUMS-taproot card.** A new `FundsSafetyAdvisory` register in `crates/mnemonic-toolkit/src/unrestorable_advisory.rs` (a SEPARATE enum + struct + collector + `WARNING (funds-safety):` prefix, deliberately textually distinct from the calm `advisory:` un-restorable siblings) fires for exactly the RESTORABLE-but-no-precedent `tr(NUMS, multi_a)` divergent-per-cosigner override case, single-sourced via the new `custom_use_site_nums_taproot_card` predicate in `taproot_override_classify.rs` (`taproot_override_card(d) && restorable_taproot_override_card(d)`) so the engrave and restore surfaces cannot drift. The reconstruction is unchanged (restore still exits 0 with the addresses on stdout); the warning is a best-effort stderr line. The advisory and the existing `TaprootUseSiteOverride` calm advisory are MUTUALLY EXCLUSIVE for any taproot override card, and BASELINE (uniform `<0;1>/*`) fires neither.

## mnemonic-toolkit [0.73.2] — 2026-06-23

**SemVer-PATCH — musl static-binary release asset + musl build/test CI coverage. Ships the first fully-static, dependency-free `mnemonic` Linux binaries (`x86_64-unknown-linux-musl` + `aarch64-unknown-linux-musl`) as GitHub-release tarballs on the `mnemonic-toolkit-v*` tag, each accompanied by a per-arch `SHA256SUMS.<arch>` for offline / air-gapped out-of-band verification — exactly the deployment surface a self-custody steel-backup tool runs on (an offline, minimal, possibly Alpine/musl box). Also adds musl build/test CI coverage to `rust.yml` (x86_64-musl FULL `cargo test` natively, aarch64-musl `cross build`). No crate source, API, `--json` wire-shape, CLI-flag, or subcommand change → the manual flag-mirror lint and the GUI `schema_mirror` gate are UNTOUCHED. NOT published to crates.io (binary-asset-only PATCH; the tag ships the binary, `cargo install` never delivers a prebuilt blob). The shipped guarantee is *static + checksummed*, not bit-for-bit *reproducible* (the binaries inherit the existing release profile; reproducible builds are a possible later followup).**

### Added

- **musl static-binary release-asset job** (`.github/workflows/man-pages.yml`, new `musl-binaries` job). On each `mnemonic-toolkit-v*` tag it builds `mnemonic` for `x86_64-unknown-linux-musl` (natively on `ubuntu-latest` with `musl-tools` + `CC_x86_64_unknown_linux_musl=musl-gcc`) and `aarch64-unknown-linux-musl` (via `cross`), tarballs each as `mnemonic-<version>-<arch>-linux-musl.tar.gz`, emits a per-arch `SHA256SUMS.<arch>`, and attaches all of them to the same release via `gh release upload --clobber` (alongside the existing `mnemonic-man.tar.gz`). `crt-static` is left at its musl default (ON) — the binaries are fully static (`file` → "static-pie linked", `ldd` → "statically linked", zero `NEEDED` libs / zero GLIBC refs); `-Ctarget-feature=-crt-static` is never set (per `rust#135244` it would silently link host glibc and destroy the static deliverable). Toolchain pinned `dtolnay/rust-toolchain@1.85.0` (matches `rust-toolchain.toml`). The only C dependency is the vendored libsecp256k1 in `secp256k1-sys`, compiled against musl by `musl-gcc` / the cross container.
- **musl build/test CI coverage** (`.github/workflows/rust.yml`, new `musl-build-test` job). `x86_64-unknown-linux-musl` runs the FULL `cargo test -p mnemonic-toolkit` natively (the `mnemonic` bin is unix, so unlike `lib cross-platform check`'s `--lib`-only Windows row the whole package — argv/schema/version lints included — runs on musl), with `musl-tools` + `CC_x86_64_unknown_linux_musl=musl-gcc` and the `ulimit -l 65536` mlock-test dance. `aarch64-unknown-linux-musl` is a `cross build`-only row (build coverage; no emulated test execution). Pinned `@1.85.0`.

## mnemonic-toolkit [0.73.1] — 2026-06-23

**SemVer-PATCH — BSD secret-hygiene parity + FreeBSD compile-gate. `set_non_dumpable()` (in `crates/mnemonic-toolkit/src/process_hardening.rs`) was fenced `#[cfg(target_os = "linux")]` and a silent no-op on the BSDs, so a custody process on FreeBSD/OpenBSD/NetBSD could be ptrace/ktrace-introspected and could drop a core file the BIP-39 seed / xpriv spills into. A second cfg arm restores parity. No CLI flag / `--json` wire-shape / subcommand surface changed → the manual flag-mirror lint and the GUI `schema_mirror` gate are UNTOUCHED. Linux behavior is unchanged (the new arm is cfg-gated off everywhere but the BSDs). Shipped in lockstep with `md-cli` 0.11.1 / `ms-cli` 0.13.1 / `mk-cli` 0.11.1 (byte-identical executable arm in all four).**

### Changed

- **`set_non_dumpable()` gains a BSD parity arm** (`crates/mnemonic-toolkit/src/process_hardening.rs`). The existing Linux `prctl(PR_SET_DUMPABLE, 0)` arm is kept; a new `#[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]` arm does (i) on FreeBSD only, `procctl(P_PID, 0, PROC_TRACE_CTL, PROC_TRACE_CTL_DISABLE)` — disables ptrace/ktrace/debugging-sysctl/hwpmc/dtrace introspection AND core dumping for this process — and (ii) on all three BSDs, `setrlimit(RLIMIT_CORE, {0, 0})` — hard-zeros the core-dump size so a secret on heap/argv cannot land in a core file. Best-effort (return values ignored, matching the `prctl` contract). macOS/Windows remain a documented no-op. All required libc symbols are present in the locked `libc` (no version bump). Compile-gated BSD unit tests added (`#[cfg(target_os = "freebsd")]` trace-status + `#[cfg(any(... bsd ...))]` core-limit) — compile-checked but never executed by the chosen CI (documentation / future-native-VM scaffolding).
- **FreeBSD compile-gate added to CI** (`.github/workflows/rust.yml`). The `lib cross-platform check` matrix gains an `x86_64-unknown-freebsd` row (Tier 2 with Host Tools; bare `rustup target add` + `cargo check --lib`, validated locally). Catches the day a Linux-only syscall / cfg / crate breaks the `cargo install`-on-FreeBSD path and now compile-covers the BSD hardening arm. The stale `ring / sha2-asm` comment on the aarch64 cross-compiler step is corrected (the only C dep is the vendored libsecp256k1).

## mnemonic-toolkit [0.73.0] — 2026-06-23

**SemVer-MINOR — `mnemonic gen-man`: self-emit roff man pages. A new visible `gen-man --out <DIR>` subcommand generates a clap-faithful man page per (nested) subcommand directly from the compiled `clap::Command` tree (`clap_mangen::generate_to`, no pre-`build()`), so the pages cannot drift from the binary's flag surface. `scripts/install.sh` drops them into the XDG user manpath after `cargo install` (no sudo, no system files; `--no-man` / `--man-dir` overrides). Part of the constellation-wide man-pages cycle (sibling `md`/`ms`/`mk` ship `gen-man` in lockstep).**

### Added

- **`mnemonic gen-man --out <DIR>`** (`crates/mnemonic-toolkit/src/cmd/gen_man.rs`). Writes `mnemonic.1` + one `mnemonic-<sub>.1` page per (nested) subcommand (hyphen-joined parent→child, e.g. `mnemonic-seed-xor-split.1`) into `<DIR>` (created if absent). Visible subcommand; appears in `mnemonic --help` and `gui-schema` output. The output carries ZERO `*-help*.1` shadow pages (the bare `generate_to` call avoids the pre-`build()` help-shadow-tree poisoning). The global `--no-auto-repair` flag renders in every generated page's `OPTIONS` section (clap_mangen surfaces the root command's global args on each page); it is also discoverable via `--help`.
- **`scripts/install.sh` man step.** After each successful `cargo install`, the installed CLI self-emits its man pages into `${XDG_DATA_HOME:-$HOME/.local/share}/man/man1` via `<bin> gen-man --out`. New `--no-man` opt-out and `--man-dir <DIR>` override; `--dry-run` prints the invocation. The hook is `||`-guarded (non-fatal under `set -eu`) to tolerate the crates.io-latest-vs-pinned-tag rollout window (a sibling whose published latest lacks `gen-man`), read-only `MAN_DIR`, and disk-full. An unconditional tail hint prints the portable `man -M "$MAN_DIR" <cli>` fallback.

### Changed

- **`clap_mangen = "0.3"` dependency added** (requires clap ^4.0; the toolkit is clap 4.6.1 — no clap bump).
- **`tests/cli_gui_schema.rs::gui_schema_lists_all_subcommands` golden vector** gains `"gen-man"` (between `final-word` and `import-wallet`); the subcommand-name count narrative moves 30 → 31. `gen-man` enters the `gui-schema` JSON whether visible or hidden (the emitter filters by name, never `is_hide_set()`).
- **Manual mirror** (`docs/manual/`): `gen-man` sections added to all four CLI-reference chapters (`41`–`44`) and `<cli> gen-man` lines added to `tests/cli-subcommands.list`.
- **`scripts/install.sh` sibling pins bumped** to `descriptor-mnemonic-md-cli-v0.11.0` / `ms-cli-v0.13.0` / `mk-cli-v0.11.0` (the sibling builds that carry `gen-man`).

## mnemonic-toolkit [0.72.0] — 2026-06-23

**SemVer-MINOR — Cycle-B paired coordination: ms-codec vendored its codex32 dependency inline (shape A) and dropped the external `codex32` crate, so the toolkit drops its own direct `codex32 = "=0.1.0"` dep and names the codex32 share-error types via `ms_codec::codex32::`. End-user behavior UNCHANGED; no clap-flag / subcommand / dropdown / `--json` wire-shape change → no GUI schema-mirror trip and no manual flag-mirror trip. ms1 wire output is BYTE-IDENTICAL (the vendor is a byte-for-byte copy of the codex32 encoding paths).**

### Changed

- **Drop the direct `codex32 = "=0.1.0"` dependency.** It existed only so `friendly_ms_codec` could name the codex32 share-error variants (`ThresholdNotPassed` / `RepeatedIndex` / `Mismatched*` / `InvalidChecksum`) wrapped by `ms_codec::Error::Codex32(_)`. ms-codec 0.7.0 vendored codex32 inline, so the toolkit now reaches those types via `ms_codec::codex32::` — `friendly.rs`'s 16 `codex32::Error::*` / `codex32::Fe::*` path-references across 15 sites (the `RepeatedIndex` line carries two: `Error::RepeatedIndex` + `Fe::Q`) are rewritten accordingly. Friendly message text is byte-identical (the prose tests assert on substrings).
- **Re-pin `ms-codec` `"0.6"` → `"0.7"`** (consumes the breaking inner-type move of `ms_codec::Error::Codex32` from the extern `codex32::Error` to `ms_codec::codex32::Error`).

### Notes

- Companion to ms-codec 0.7.0 + ms-cli 0.12.0 (Cycle-B). Mirrors the ms-side `codex32-upstream-dormant-vendor-vs-accept-decision` resolution per the cross-repo follow-up rule.

## mnemonic-toolkit [0.71.0] — 2026-06-22

**SemVer-MINOR — Wave-2 secret-memory-hygiene (T1–T4). Four owned-secret sites that lingered un-scrubbed (or in a leak-prone bare type) are migrated to the shipped scrub-on-drop / redacting newtypes. Toolkit-internal only; no codec/GUI bump. No clap-flag / subcommand / dropdown / `--json` wire-shape change → no GUI schema-mirror trip and no manual flag-mirror trip. The `convert --to xprv` output and the `inspect` output are BYTE-IDENTICAL (golden-pinned).**

### Changed (secret-memory hygiene — no behavior / wire change)

- **T1 — `DerivedAccount.account_xpriv` migrated `Xpriv` → `ScrubbedXpriv` (MINOR).** The account spending key was a bare `Xpriv` (`Copy`, dropped WITHOUT erase). It is now the move-only scrub-on-drop `ScrubbedXpriv` newtype (shipped v0.70.0). `DerivedAccount::into_parts` DROPPED its `Xpriv` tuple element entirely — the field stays in `self` and scrubs in place; all 8 callers discarded it. The single deliberate reader (`convert --to xprv`) goes through a new narrow, string-only `ScrubbedXpriv::expose_xprv_string(&self) -> SecretString` (length-only redacting Debug, scrub-on-drop) — no bare `Xpriv` handle escapes. `ScrubbedXpriv` gained a hand-written redacting `Debug` (NOT `#[derive]`, which would leak the inner key). `--to xprv` output byte-identical (golden `entropy_to_xprv_bip84_mainnet`). Closes FOLLOWUP `derive-slot-account-xpriv-scrub-confinement`.
- **T2 — `self_check_bundle` + `inspect` no longer drop a decoded ms1 `Payload` (master-seed entropy) un-scrubbed.** Both sites now move the decoded entropy OUT of the bare `ms_codec::Payload` into a fn-local `Zeroizing<Vec<u8>>` (the `InspectPayload::Ms1` variant is reshaped to carry `Zeroizing<Vec<u8>>` + `PayloadKind` + `Option<u8>` language, read off the husk at decode time). Inspect text/JSON output byte-identical. Closes FOLLOWUP `self-check-ms1-decode-not-zeroizing` (both sites).
- **T3 — `import-wallet` seed-overlay no longer copies the phrase / ms1 secret into a bare `String`.** `phrase_overlays` and `ms1_args` flip to `SecretString`; `apply_seed_overlay`'s signature + its fn-local `Source` enum carry `SecretString` end-to-end to the bip39/ms decode consumers. Closes FOLLOWUP `phrase-overlay-secretstring` (both the Phrase AND the ms1 arms).
- **T4 — `read_stdin_to_string` / `read_stdin_passphrase` scrub their transient scratch buffer.** The internal `read_to_string` buffer is now `Zeroizing<String>`. The return type stays `String` (every secret-class caller already wraps the returned value; narrowing would double-wrap the reader call sites). Behavior byte-identical (same trimming / CRLF-strip). Closes FOLLOWUP `stdin-reader-transient-buf-zeroizing`.

### Notes

- **Zeroize-discipline lint:** `src/cmd/inspect.rs` is now a declared secret-bearing row; `SECRET_FILE_FLOOR` recomputed live and tightened 37 → 39 (the static 37 was already 1 below the live partition of 38 at authoring; +1 for inspect.rs).
- **No codec/GUI bump.** R0 GREEN (0C/0I, 2 Minors folded) + post-impl whole-diff review — see `design/SPEC_wave2_secret_hygiene_toolkit.md` + `design/agent-reports/wave2-toolkit-r0-review.md`.

## mnemonic-toolkit [0.70.1] — 2026-06-22

**SemVer-PATCH — open-followups maturity program, Wave 1: export-refusal hardening + reconcile. `export-wallet --format green` now refuses a taproot tap-script-tree POLICY (previously mislabeled it "(singlesig)" — a wrong-label, not wrong-address, defect); a direct `--descriptor 'wsh(multi(…))'` (unsorted) to field-less vendor formats now surfaces the typed unsorted-multisig refusal instead of a generic one. Toolkit-only; no codec/GUI bump. No clap-flag / dropdown / `--json` wire-shape change → no GUI schema-mirror; no manual change.**

### Changed (funds-adjacent labeling — refusal where it previously mislabeled)

- **`export-wallet --format green` refuses a taproot tap-script-tree policy.** A general taproot policy (`tr(internal,{…})` carrying a tapscript tree) is classified `P2tr` and previously fell through the multisig guard, emitting Green's static `# … (singlesig)` header for a script-tree policy (the descriptor inside was faithful → wrong-LABEL, not wrong-address). It now refuses with a pointer to `--format bitcoin-core` / `--format descriptor`, mirroring the restore-side refusal (v0.55.1). Discrimination is STRUCTURAL via miniscript `Tr::tap_tree()` — a single-leaf tree (`tr(NUMS,pk(A))`) renders without `,{`, so a substring probe would be unsound. Keypath-only single-sig taproot (BIP86) emission is unchanged. Closes FOLLOWUP `export-wallet-green-tr-policy-singlesig-emission`. (Whether Green's file import accepts a `tr(KEY)` keypath descriptor is unverified upstream → tracked by new FOLLOWUP `green-taproot-keypath-file-import-unverified`.)

### Changed (diagnostic message quality — no behavior change)

- **Direct `--descriptor 'wsh(multi(…))'` (unsorted) to field-less vendor formats surfaces the typed unsorted-multisig refusal.** The direct-descriptor path (`template == None`) previously refused unsorted-multi exports to electrum/coldcard/jade with a generic `BadInput` ("requires --template"); a second arm of the H10 guard now returns the typed `ExportWalletUnsortedMultisigUnsupported` (exit 2 unchanged) — already funds-safe, now clearer. Closes FOLLOWUP `export-wallet-direct-descriptor-unsorted-multi-generic-refusal`.

### Notes

- **xpub-search md1 detection — DEFERRED.** `xpub-search-descriptor-md1-detection-bech32-validate` is deferred: md1 uses a custom BCH(93,80,8) checksum (target residue `md_codec::bch::MD_REGULAR_CONST`), so the proposed `bitcoin::bech32::decode` tightening would reject every valid md1 string (a funds-feature regression); the current intake is already loud-and-safe. The correct future primitive is `md_codec::bch::bch_verify_regular` (per-chunk).
- **CI hygiene reconcile.** Flipped the already-resolved-in-CI vacuous flag-coverage lint FOLLOWUPs (`lint-md-flag-coverage-vacuous-with-md_bin-true`, `manual-yml-bind-real-mnemonic-bin`) → resolved.
- **No codec/GUI bump.** All R0-GREEN (2 rounds) + post-impl whole-diff review — see `design/agent-reports/w1-export-refusals-r0-*`.

## mnemonic-toolkit [0.70.0] — 2026-06-22

**SemVer-MINOR — own-account subset-search: `restore --own-account-max K` (re-enabled) over-supplies own-account candidates and resolves the unique multisig-template assignment via an own-anchored k-permutation search; opt-in `--search-cosigner-subset` extends it to over-supplied cosigner candidates (bounded).** Toolkit-only; md-codec / mk-codec / GUI NO-BUMP. New clap flags are documented in the manual in lockstep (`docs/manual/src/40-cli-reference/41-mnemonic.md`); the GUI schema-mirror is a paired-PR concern handled by the orchestrator.

### Added

- **`restore --search-cosigner-subset`** — opt-in flag that extends the own-anchored subset-search to over-supplied **cosigner** candidate cards (more `--cosigner` mk1 cards than the template's cosigner slots), so a multisig template can be re-assembled when the operator supplies a superset of cosigner cards. Bounded (see Notes). Default OFF — cosigners are matched exactly when the flag is absent.
- **`verify-bundle --own-account-max K`** — mirrors `restore --own-account-max`: over-supplies own-account candidates (the own seed derived at accounts `0..K-1`) and resolves the unique template assignment via the own-anchored k-permutation search, for verifying a bundle when the operator does not recall their account index.
- **`verify-bundle --search-cosigner-subset`** — the `verify-bundle` mirror of `restore --search-cosigner-subset` (opt-in, bounded over-supplied-cosigner search).

### Changed

- **`restore --own-account-max` flips from refuse → subset-search.** Previously (v0.60.0 reserved/refused — deferred) this flag exited with an error; it now derives the own seed at accounts `0..K-1`, over-supplies those own-account candidates, and resolves the unique multisig-template assignment via the own-anchored k-permutation search. Mutually exclusive with `--account` (a fixed index needs no search). This lifts the #28-phase-2 P3a deferral.

### Notes

- **Own-only by default; cosigners exact.** Without `--search-cosigner-subset`, only the **own** account is over-supplied and searched; supplied cosigner cards are matched exactly. `--search-cosigner-subset` is opt-in and **bounded**: own pool `K_own ≤ 256`, optional-cosigner search space `S_opt ≤ 1e15` hard ceiling plus the adaptive time-cap; over-budget inputs refuse (exit ≠ 0) rather than run unbounded. Large pools should prefer address-search.
- **Funds-safety.** `realized_s` (`= S_own` for own-only, `= S_opt` when cosigner-subset is engaged) sizes the strong prefix required over the (larger) search space, so an over-supplied search cannot accept a weak/ambiguous match. The own candidate pool is derived **public-only** (`derive_account_xpub_only` plus the move-only `ScrubbedXpriv` scrub, P0) — the own xpriv never lingers un-scrubbed. All refusals exit ≠ 0.
- **No codec/GUI bump.** md-codec / mk-codec unchanged (NO-BUMP). P0–P5 all R0-GREEN — see `design/agent-reports/own-account-subset-search-*`.

## mnemonic-toolkit [0.69.1] — 2026-06-22

**SemVer-PATCH — bip85 encode/dice internal-scratch `Zeroizing` (cycle-15 Group A derived-output-sweep FOLLOWUP slug `bip85-encode-helper-internal-scratch-zeroizing`). Toolkit-only; no codec/GUI bump. No signature / clap-flag / dropdown / `--json` wire-shape change → no GUI schema-mirror; no manual change. Outputs byte-identical (BIP-85 KATs unchanged). PATCH (not MINOR) — the three wraps are function-local, reach no signature or public type.**

### Changed (secret-memory hygiene — no observable behavior change)

- **bip85 — encode/dice internal scratch wrapped in `Zeroizing`.** The seven `format_*` functions already return `SecretString` (Lane T, v0.68.0), but three internal locals materialized the derived secret in a bare heap allocation before the wrapped return, lingering un-scrubbed until function exit: the `format_password_base64` / `format_password_base85` full `encoded` String (only `encoded[..length]` was wrapped into the `SecretString` return; the full encode lingered) and the `format_dice_rolls` per-roll `out: Vec<String>` aggregate (the dice secret). All three now wrap in `Zeroizing` (`Vec<String>: Zeroize` via the blanket impl scrubs each String on drop). Ripple-free — `encoded[..length].to_string()` indexes through `Deref<Target=String>` and `out.push(…)` / `out.join(",")` go through `Deref`/`DerefMut`; no signature, lint, wire, or caller change; BIP-85 KATs byte-identical. The per-roll `buf` (raw SHAKE bytes, overwritten each iteration) and the encode-helper-internal `out: String` (moved out into the caller's now-`Zeroizing` `encoded`) are dispositioned out-of-scope. Closes FOLLOWUP `bip85-encode-helper-internal-scratch-zeroizing`.

## mnemonic-toolkit [0.69.0] — 2026-06-22

**SemVer-MINOR — cycle-15 Group A: BSMS HMAC_KEY secret-memory-hygiene + zeroize-lint allowlist precision (2 derived-output-sweep FOLLOWUP slugs + 2 Lane T whole-diff-review nits). Toolkit-only; no codec/GUI bump. No clap-flag / dropdown / `--json` wire-shape change → no GUI schema-mirror; no manual change. Outputs byte-identical (BIP-129 TV-3 MAC/IV/round-trip unchanged). MINOR (not PATCH) per the v0.10.1 / v0.67.0 / 0.68.0 precedent for a `pub`-signature secret-type migration even when no public shape changes.**

### Changed (secret-memory hygiene — no observable behavior change)

- **BSMS / BIP-129 — `bsms_crypto::derive_hmac_key` returns `Zeroizing<[u8; 32]>`.** The HMAC_KEY (`= SHA256(ENCRYPTION_KEY)`) is secret-class (derived from the `Zeroizing` ENCRYPTION_KEY); its scrub obligation now lives in the return type so no present-or-future caller can leak it by forgetting to wrap. The signature change reaches no real consumer — the sole production caller (`import-wallet`), the in-module tests, and the one integration-test helper all compile and pass with ZERO edits (`Zeroizing`'s `Deref`/`AsRef` covers `hex::encode`, `compute_mac(&hmac_key, …)`, and the slice path). `compute_mac` stays a bare `[u8; 32]` (its output is a published BIP-129 authentication tag — first 16 bytes become the on-wire IV, compared against the untrusted `mac_recv` — not secret-class). Closes FOLLOWUP `bsms-derive-hmac-key-not-zeroizing`.

### Internal (test/lint precision — no shipped-binary change)

- **zeroize-completeness lint — `TEST_ONLY_SECRET_FILES` cfg(test)-confinement tier.** `src/bundle_unified.rs` (whose sole `SecretString::new` is the `#[cfg(test)] s()` SlotInput fixture) moves out of the whole-file `NON_ROW_SECRET_FILES` allowlist into a new `TEST_ONLY_SECRET_FILES` tier guarded by `test_only_secret_files_confine_secret_patterns_to_cfg_test`: every secret pattern in such a file must appear AFTER the file's first `#[cfg(test)]` line, so a future PRODUCTION secret allocation added above the marker is CAUGHT rather than silently masked. Both partition-scan consumers (the declared/allowlisted scan + the staleness tripwire) union the new tier; pure helpers (`first_cfg_test_line` / `production_secret_lines`) are unit-tested over synthetic strings. The `ZEROIZE_ROWS.len()` count-guard message was reworded to drop the decay-prone hardcoded counts (keeps the live `{n}` and the `18..=66` bound). Live partition stays 38; `SECRET_FILE_FLOOR` stays 37. Closes FOLLOWUP `bundle-unified-whole-file-allowlist-precision`.

## mnemonic-toolkit [0.68.0] — 2026-06-21

**SemVer-MINOR — cycle-15 Lane T: derived-output secret-memory-hygiene sweep (3 toolkit FOLLOWUP slugs). Toolkit-only; no codec/GUI bump. No clap-flag / dropdown / `--json` wire-shape change → no GUI schema-mirror; no manual change. Defense-in-depth (heap residue) — behavior / text / `--json` byte-identical. MINOR (not PATCH) per the v0.10.1 precedent for a `pub mod`-touching secret-type migration even when nothing public changes shape.**

### Changed (secret-memory hygiene — no observable behavior change)

- **bip85 / derive-child — rendered SPENDABLE BIP-85 child secrets carry as `SecretString`.** The seven `bip85::format_*` functions (BIP-39 phrase / HD-seed WIF / child xprv / hex / password-base64 / password-base85 / dice) now return `Result<SecretString, ToolkitError>` instead of bare `Result<String, _>`, and the single `derive-child` `output` emitter local is a `SecretString`. `SecretString` zeroizes on drop and has a length-only **redacting** `Debug` (a bare `Zeroizing<String>` would leak the secret through `{:?}` / panic / `assert_eq!` — `zeroize 1.8.2`). `Display`/`Deref` render verbatim, so the `writeln!("{output}")` text path is byte-identical; `derive-child` has no `--json`.
- **electrum native-seed — entropy returned BY MOVE, normalize intermediates wrapped.** `electrum::phrase_to_entropy` returns `Zeroizing<Vec<u8>>` (`Ok(acc)`) instead of cloning the secret entropy out of the `Zeroizing` wrapper into a bare un-scrubbed `Vec` — the funds-relevant fix. The PBKDF2 normalize intermediates (the electrum-local `normalize_text_electrum` / `normalize_phrase_for_hmac` returns, the per-word and per-candidate phrase scratch) wrap in `Zeroizing<String>` at the consumption boundary. `entropy_to_phrase` keeps its `Result<String, _>` return (its caller type-unifies against bare-`String` arms); `wordlists::normalize_electrum` stays `-> String` (cross-module helper).
- **seedqr — internal scratch wrapped in `Zeroizing`.** `decode` / `encode` / `encode_compact` / `decode_compact` wrap their internal secret scratch (raw-digit `stripped`, per-word `words` / `normalized` / `digits`, the `decode_compact` hex-decoded raw-entropy `bytes`) in `Zeroizing`. The four `pub fn` returns stay bare `String` (no SemVer break); the bare consumer-locals are a separate pre-existing residue class, out of scope.
- **ms-codec pin `0.5` → `0.6`** (recompile-only; `Payload`/`decode` byte-stable 0.5.0→0.6.0, diff-confirmed). Re-resolves both lockfiles to ms-codec 0.6.0.
- **zeroize-discipline lint** — `ZEROIZE_ROWS.len()` count-guard widened `(18..=60)` → `(18..=66)`; +6 canonical rows (bip85 `SecretString` returns, derive-child output, electrum move-out / norm-scratch / per-word wrap, and the new `src/seedqr.rs` `source_file` row). `SECRET_FILE_FLOOR` unchanged.

## mnemonic-toolkit [0.67.0] — 2026-06-21

**SemVer-MINOR — closes L22, the constellation bug-hunt's final secret-memory-hygiene finding (cycle-14). Toolkit-only; no codec/GUI bump. No clap-flag / dropdown / `--json` wire-shape change → no GUI schema-mirror; no manual change. Defense-in-depth (heap residue) — no observable CLI/wire behavior change.**

### Fixed

- **L22 (SECRET / defense-in-depth) — a stdin / `@env:` secret read into a `@N.<secret>=-` slot no longer lingers un-scrubbed in a bare `String`.** The persistent secret-bearing field `SlotInput.value` migrates `String` → `SecretString` (`Zeroizing<String>` inner, length-only **redacting** `Debug`), so the secret zeroizes on drop and never re-leaks through `{:?}` (panic/`assert_eq!`/log output). A raw `Zeroizing<String>` would compile but its derived `Debug` LEAKS the secret (`zeroize 1.8.2`), so `SecretString` is the correct carrier; it gains plain (non-constant-time) `PartialEq`/`Eq` (equality is test-only + the public `"-"` sentinel — no auth/timing boundary). The same wrap **also closes the `@env:` write-back residue path** (`bundle` / `import-wallet` / `verify-bundle` resolve `@env:VAR` to the actual secret phrase and store it back into the field). The `convert` / `restore` / `addresses` handler-scope `passphrase` / `--from` / BIP-38 locals (mlock-pinned but previously un-scrubbed — mlock prevents swap-out, it does not zeroize) are wrapped in `Zeroizing<String>`; `restore`'s `TemplateSeed.passphrase` field is wrapped so the resolved passphrase scrubs end-to-end. The stdin readers stay bare `String` (flipping them would make 14 already-wrapping callers illegal `Zeroizing<Zeroizing<String>>`). mlock pins preserved. The zeroize-discipline lint gains the `slot_input.rs` canonical row + a `bundle_unified.rs` test-fixture allowlist entry; `SECRET_FILE_FLOOR` 35 → 37. Closes the toolkit's Cycle-B canonical "Site 1" scrub leg (the mlock leg shipped at v0.10.x). The downstream `phrase_overlays` Vec deep-wrap is deferred (FOLLOWUP `phrase-overlay-secretstring`).

## mnemonic-toolkit [0.66.0] — 2026-06-21

**SemVer-MINOR — the "fidelity tail": closes the final 7 findings of the constellation bug-hunt (cycle-13: H11 · H14 · L8 · L9 · M1 · M7 · L18). Toolkit-only; no codec/GUI bump. No clap-flag / dropdown / schema change → no GUI schema-mirror. Foreign-format manual prose + SPEC §11.4.1 updated (prose only, no flag-table change).**

### Fixed

- **H11 (export fidelity)** — `export-wallet` coldcard-multisig / jade no longer collapses divergent cosigner origins to a wrong global `m/0'/0'`. On divergent (collaborative-custody) paths it now emits a per-cosigner `Derivation:` line read from each cosigner's OWN sorted slot (sorted is the only reachable divergent case given the v0.62.0 unsorted-export refusal), keeping the single shared line only when all origins agree, and refusing rather than emitting a placeholder when an origin is un-emittable.
- **H14 (import fidelity)** — `coldcard-multisig` / jade import applies a depth-gated master-fingerprint matrix: a depth>0 account xpub with no supplied XFP is now REFUSED (the master fingerprint is unrecoverable from an account key — previously the account's own fingerprint was silently substituted as the master fp); a supplied XFP at depth>0 is accepted as authoritative without a spurious "disagrees" warning; `xpub.fingerprint()` is treated as a master fp only at depth 0.
- **H11+H14 round-trip** — the `<XFP>:` import parser arm now consumes a pending per-cosigner `Derivation:` path, and the round-trip-verify canonicalizer preserves per-cosigner paths, so a divergent export re-imports faithfully.
- **L8 (restore availability)** — all-own multisig-template completion now substitutes the network coin-type into the synthesized own origin, so testnet/signet/regtest all-own wallets restore (previously hardcoded mainnet `0'` → silent NO-MATCH). Fail-safe (never a wrong address). md-codec unchanged.
- **L9 (restore hardening)** — the multisig-template completion core now applies the same hardened-use-site / unrestorable-taproot-override refusals as `run_multisig`, giving restore + verify-bundle a precise early refusal instead of an opaque downstream error.
- **M1 (import metadata)** — `import-wallet --json` decodes the real BIP-32 account from a single-sig origin into `bundle.account` (was hardcoded `0`), so `export-wallet --from-import-json` re-emits the correct `m/.../<account>'` instead of `m/.../0'`. Multisig (per-slot origins) unaffected.
- **M7 (bundle metadata)** — `bundle … --json` reports the real multisig threshold K via `extract_multisig_threshold` (was the cosigner count N) in descriptor / `--import-json` mode. `--json` wire-value change (GUI consumer paired-PR concern); md1 wire + embedded descriptor were already correct.
- **L18 (import robustness)** — Electrum import accepts watch-only "use a master key" wallets that emit null `root_fingerprint`/`derivation` (verified against live Electrum `keystore.py`): null fingerprint → `00000000` sentinel + NOTICE; null derivation → script-type inferred from the SLIP-132 xpub prefix and a canonical origin synthesized (key-origin metadata only — never affects address derivation) + NOTICE. Previously hard-refused.

## mnemonic-toolkit [0.65.2] — 2026-06-21

**SemVer-PATCH — transitive md-codec pin bump `0.38` → `0.39` (constellation bug-hunt cycle-10).** No toolkit source change. Picks up md-codec v0.39.0: M3 `derive_address` chain-gate widening (change addresses derivable for `None`-baseline + per-`@N`-override wallets — a funds-availability fix), L14/L15 WalletPolicyId/WalletDescriptorTemplateId stability across origin elision + placeholder ordering (in-memory identity values only — NOT on the md1 wire; a toolkit-built explicit-origin descriptor now correctly matches an md-cli-elided card's id), and the L6 typed `DivergentPathCountMismatch` guard. The toolkit only ever builds explicit-origin / canonical-ordering descriptors, so its recorded in-memory ids are unchanged in practice; full `cargo test -p mnemonic-toolkit` GREEN against md-codec 0.39.0. No new flag/wire/variant → no GUI schema-mirror.

## mnemonic-toolkit [0.65.1] — 2026-06-21

**SemVer-PATCH — toolkit secret-hygiene / robustness cluster (constellation bug-hunt cycle-11b: L21 · L24 · L25). No new `ToolkitError` variant, no CLI-flag / dropdown / `--json` wire-shape change → no GUI schema-mirror; no codec bump; manual PROSE only (no flag-table change).**

### Fixed

- **L21 (funds-safety / SECRET) — `convert` now REFUSES a composite `(seedqr|phrase|entropy)→bip38` edge when `--bip38-passphrase` is unset.** Previously `--passphrase` fed only the BIP-39 PBKDF2 leg, so an unset `--bip38-passphrase` silently encrypted the BIP-38 Scrypt layer with the EMPTY passphrase (and the "ignored passphrase" warning is suppressed on BIP-38 edges → silent) — an engravable card the user believed was BIP-38-protected was encrypted with `""`. A position-based refusal at the composite `Bip38 =>` sub-arm head (covering all three sources incl. seedqr) returns `ConvertRefusal` (**exit 2**) before the empty encrypt. The predicate tests `.is_none()` (not emptiness), so `--bip38-passphrase ""` still encrypts deliberately. Direct `(wif↔bip38)` edges keep their documented `--passphrase` fallback. Manual PROSE updated (`56-bip39-vs-bip38-pass.md` edge table + `41-mnemonic.md` `--bip38-passphrase` row).
- **L24 (robustness) — `verify-bundle` descriptor-mode `--slot @N.path` over-`n` OOB panic → typed `DescriptorParse`.** `validate_slot_set` enforces only contiguity (`0..=max_idx`), not range-vs-`n`, so a contiguous slot set whose max index exceeds the descriptor's placeholder count `n` reached the per-slot override loop and panicked on the unguarded `new_paths[idx]` write. Mirrored `bundle.rs`'s exact-coverage `max(idx+1) != n` gate into `verify_bundle.rs` → clean `DescriptorParse` (**exit 2**); also catches the under-`n` case (bundle.rs parity). Standalone gate carries an S-VERIFY fold note (FOLLOWUP `verify-bundle-bundle-rs-descriptor-mode-dedup`).
- **L25 (cosmetic) — `import-wallet` now classifies a raw 64-hex x-only taproot key in a key position as keyed.** `has_any_key_token` matched xpub-family + `02/03`-prefixed 66-hex compressed keys but not bare 64-hex x-only (BIP-340/341) keys, so an origin-less `tr(<xonly>, …)` routed to the misleading "keyless script (hashlock/timelock only)" message. An ADDITIVE position-aware anchor (`tr(`/`pk(`/`pk_k(`/`pk_h(` + 64-hex) re-routes it to the correct "must carry a key origin" message; `sha256`/`hash256`/`ripemd160`/`hash160` 64-hex hash literals stay keyless, and the 66-hex compressed-key match is unchanged. Both arms still `Err` (message-only change).

All three are toolkit-only: no `md-codec` / `ms-codec` / `mk-codec` bump, no new `ToolkitError` variant, no CLI-flag / dropdown / `--json` wire-shape change → no GUI schema-mirror; only manual PROSE.

## mnemonic-toolkit [0.65.0] — 2026-06-21

**SemVer-MINOR — `build-descriptor` extra-derivation-suffix fail-closed reject (constellation bug-hunt cycle-7). No new `DiagnosticKind` → no `--json` wire-shape change, no GUI schema-mirror; no codec/manual leg.**

### Fixed

- **M8 (funds) — `build-descriptor` now REJECTS a key whose xpub body carries an extra derivation suffix** (e.g. `[fp/84h/0h/0h]xpub.../5`). Previously such a key was silently accepted and the builder appended `/<0;1>/*` on top, deriving a DEEPER / WRONG subtree — a silent wrong-address bug (funds-safety). The reject routes through the existing `DiagnosticKind::SchemaField` (**exit 2**); the diagnostic path + message name the offending key — for quorum archetypes the `flag` provenance resolves to `--threshold`, single-key to `--key` (both exit 2). No new `DiagnosticKind` → no `--json` wire-shape change, no GUI schema-mirror.
- **L23 — `ecies_decrypt_message` now returns a typed `EciesDecryptError::InvalidScalar`** (was a panic via `.expect()`) on a zero private scalar. Latent — not CLI-reachable (the sole caller already rejects a zero scalar) — so this is a defensive close, not a behavior change at any surface. No codec / GUI / manual lockstep.

Both fixes are toolkit-only: no `md-codec` / `ms-codec` / `mk-codec` bump, no CLI-flag / dropdown / `--json` wire-shape change, so no GUI schema-mirror or manual leg.

## mnemonic-toolkit [0.64.0] — 2026-06-21

**SemVer-MINOR — decaying-multisig decay-ordering fail-closed validation (constellation bug-hunt cycle-6). No CLI-flag / `--json` wire-shape / dropdown change, no new `ToolkitError` variant → no GUI schema-mirror, no manual leg, no codec change.**

### Fixed

- **D-decay-rel — `build-descriptor`'s `validate_params` now refuses a mis-orderable `--older` / `--recovery-older` relative-timelock pair.** A CROSS-UNIT pair (one block-height delay, one 512-second delay) is not orderable offline — block delays and time delays cannot be compared without consensus state — so the decay tiers' relative spendability ordering is undefined; it is now rejected. A SAME-UNIT pair that is not strictly ordered (a recovery tier whose delay is ≤ the primary's) is also rejected — a recovery tier could otherwise unlock BEFORE the primary, silently mis-building the intended decaying spending policy. A new `timelock_advisory::older_unit_value` helper classifies the BIP-68 bit-22 (512-second) time-unit vs block-unit of an `older(n)` value to drive the cross-unit / same-unit determination.
- **D-decay-abs — `build-descriptor` now refuses an absolute `after(N)` whose value is a PAST block height or unix-time.** A last-resort `after()` tier set to an already-elapsed BIP-65 locktime would be immediately spendable, silently mis-building the decay policy; static BIP-65 past-floor constants now reject it.

Both classes previously silently mis-built a wrong spending policy (funds-safety). Routed through the existing `Diagnostic { kind: Param }` (**exit 2**) — no new flag, `--json` wire-shape, or `ToolkitError` variant.

The canon `decaying-multisig` fixtures migrated `after(500000)` → `after(4000000)` (past-floor compliant); the descriptor checksum regenerated `#llvl05j9` → `#9fqrjy7e`.

Closes D-decay-rel + D-decay-abs; re-scopes FOLLOWUP `archetype-older-blocks-flag-accepts-time-units` (funds facet resolved). Toolkit-only: no `md-codec`/`ms-codec`/`mk-codec` bump, so no GUI schema-mirror or manual leg.

## mnemonic-toolkit [0.63.0] — 2026-06-21

**SemVer-MINOR — S-NET network-provenance invariant (constellation bug-hunt cycle-5). A new fail-closed rule rejects a decoded xpub/WIF whose network disagrees with the asserted network / coin-type. No CLI-flag / `--json` wire-shape / dropdown change → no GUI schema-mirror, no manual leg, no codec change.**

### Fixed

- **H15 / M13 / M14 — network-provenance cross-check at the import / convert / export surfaces.** A decoded extended key (xpub) or WIF whose embedded network byte disagrees with the asserted `--network` / coin-type was previously accepted, silently producing a wallet on the wrong chain. A shared `network::assert_network_agrees` helper is now wired at the 7 import parsers, the two `convert` arms (`--xpub-prefix`, `--from wif --to xpub`), and `export-wallet --from-import-json`. This wires the formerly-dead `ToolkitError::NetworkMismatch` (**exit 2**). The `NetworkMismatch` error's `detail_json` adds error-shape keys — NOT a `schema_mirror` trigger (that gate covers clap flag-names / dropdown-values, not the `--json` error-shape).
- **H9 — `import-wallet --network` class-check is now PER-ENTRY** (was `first()`-only, so a class-mismatched non-first entry slipped through). A disagreement now fires the typed `ToolkitError::ImportWalletNetworkClassMismatch` (**exit 1**).
- **L3 — `build-descriptor` coldcard single-sig account `> u32::MAX` now REJECTS** instead of silently truncating.
- **L1 — `build-descriptor` now WARNs (not rejects) on a `--network` / keys preview disagreement.**

Closes H15 / M13 / M14 / H9 / L1 / L2 / L3 / L10 / L11.

Provenance: `design/agent-reports/constellation-bughunt-2026-06-20.md`, `design/PLAN_constellation_bughunt_fix_program.md`, the cycle-5 brainstorm / plan / R0 / impl reviews. Toolkit-only: no CLI-flag / `--json` wire-shape change, no `md-codec`/`ms-codec`/`mk-codec` bump, so no GUI schema-mirror or manual leg. (Version `0.63.0` taken first-to-ship; the paused own-account-subset-search cycle renumbers.)

## mnemonic-toolkit [0.62.1] — 2026-06-21

**SemVer-PATCH — cycle-4 codec-funds convergence: pins the two just-published codec funds-safety MINORs (`md-codec` 0.38.0 + `ms-codec` 0.5.0) and adds the downstream lockstep exit-code / prose arms. No toolkit CLI-flag / `--json` wire-shape change → no GUI schema-mirror or manual leg.**

### Changed

- **Pinned `md-codec` 0.37 → 0.38.0 and `ms-codec` 0.4.4 → 0.5.0** (caret pins hand-edited — `cargo update` cannot cross the `^0.37`<0.38 / `^0.4.4`<0.5.0 bounds), pulling the three codex32 regular-code length caps (H6 encode-side `PayloadTooLongForSingleString`; M4 correcting-decode `ChunkSymbolCountOutOfRange`; I1 non-correcting-decode `StringSymbolCountOutOfRange`) and the ms-codec cross-share polynomial-consistency check (M6 `InconsistentShareSet`).
- **`md_codec_exit_code` (exhaustive match — the pin bump was compile-forced) gains arms** for `PayloadTooLongForSingleString`, `ChunkSymbolCountOutOfRange`, and `StringSymbolCountOutOfRange` → **exit 2** (decode/format-reject class, alongside the `TooManyErrors` group). `friendly_md_codec` (also an exhaustive match) gains matching prose arms.
- **`ms_codec_exit_code` gains an explicit `InconsistentShareSet => 2` arm** (funds/format-violation class). Because `ms_codec::Error` is `#[non_exhaustive]` and the match ends `_ => 1`, this is a SILENT lockstep — the arm is explicit by design (a missing arm would mis-route a funds error to exit 1). `friendly_ms_codec` gains a clear prose arm ("inconsistent share set: one or more shares are not from the same split…").

### Funds-safety notes (toolkit-surface behavior)

- `mnemonic inspect <over-93-symbol md1>` → **exit 2** — the I1 non-correcting cap surfacing through `md_codec_exit_code`.
- `mnemonic ms-shares combine <inconsistent same-id share set>` → **exit 2** with prose; valid exactly-k and over-threshold all-consistent combines still recover the correct secret unchanged (no regression).
- `mnemonic repair --md1 <over-93-symbol md1>` → exit 2 via the toolkit's own `repair.rs` length-band classifier (independent of the codec M4 cap). `mnemonic restore --md1 <over-93-symbol md1>` → exit 1: the cap MESSAGE renders but the restore path's pre-existing `bad()` wrapper down-classifies every md1-reassemble failure to `BadInput` (unchanged from prior releases; re-routing it is out of scope for this PATCH).

Provenance: `design/BRAINSTORM_cycle4_codec_funds_fixes.md` + `design/IMPLEMENTATION_PLAN_cycle4_codec_funds_fixes.md` (both R0-GREEN), `design/agent-reports/cycle4-*`. Codec tags: `descriptor-mnemonic-md-codec-v0.38.0`, `mnemonic-secret-ms-codec-v0.5.0` (both on crates.io). Toolkit-only consumer step; `mk-codec` unchanged.

## mnemonic-toolkit [0.62.0] — 2026-06-21

**SemVer-MINOR — FUNDS-SAFETY: three more empirically-grounded fixes from the constellation differential-oracle bug hunt (H8 / H10 / H7), cycle-2. No CLI-flag / `--json` wire-shape / codec change.**

### Fixed

- **H8 — `bundle --md1-form=template` now carries the BIP-39 wordlist language into the per-slot ms1 emit.** The template ms1 emit loop hardcoded `bip39::Language::English`, so a NON-English seed whose wordlist language is supplied only by the run-level `--language` (slot `language == None`) was silently re-emitted as an English `Payload::Entr` card. A BIP-39 seed is PBKDF2 over the NFKD wordlist *string*, so English-defaulted recovery of that entropy derives a DIFFERENT 512-bit seed → wrong master seed → unrecoverable card. The run-level language is now threaded into `synthesize_template_descriptor` and forwarded at the `--md1-form=template` call site; the hardcoded default becomes `unwrap_or(run_language)` — byte-identical to the already-correct keyed path. The shared emit loop serves both single-sig and multisig template forms, so one change covers both. Private fn → no clap surface.
- **H10 — `export-wallet` to the field-less `electrum` / `coldcard` / `coldcard-multisig` / `jade` multisig formats now REFUSES an unsorted `multi(...)` descriptor instead of silently coercing it to BIP-67 `sortedmulti`.** Those vendor file formats are sortedmulti-only (no field expresses literal `multi(...)` key order), so exporting an UNSORTED `wsh(multi)` / `sh(wsh(multi))` silently re-sorted the keys, changing the witnessScript and every address (oracle-proven by the `wsh-multi-2of3-divergent` row in `tests/bitcoind_differential.rs`). A new typed `ToolkitError::ExportWalletUnsortedMultisigUnsupported { format }` (exit 2) now fires in the shared `emit_payload` chokepoint for those four formats when the resolved template is `Some(WshMulti)` / `Some(ShWshMulti)`. PURE REFUSAL, no flag. `sortedmulti` / single-sig / faithful (descriptor-preserving) formats are unaffected. The direct `--descriptor 'wsh(multi(…))'` path was already funds-safe (refused by the emitters' generic `BadInput`, never silently coerced); upgrading that path to the typed message is a deferred cosmetic FOLLOWUP.
- **H7 — the BIP-380-canonical PREFIX-form `[fingerprint/path]@N` key-origin annotation is now ACCEPTED.** It was silently IGNORED by the suffix-only placeholder lexer, so a user/tool following the standard had the origin path DROPPED (the slot xpub built at the default/master path → the backup watched a DIFFERENT address set) AND the per-`@N` master-fingerprint cross-check bypassed. `lex_placeholders` now admits an optional prefix-origin alternation before `@N`; the regex was converted to ALL-NAMED capture groups and every consumer rewritten from numeric indices to access-by-name — INCLUDING cycle-1's H13 hardened-multipath validator (`.get(4)` → `.name("mpath")`, body byte-identical) — so prepending the prefix can never shift the multipath group out from under the H13 reject (the C1 fold). Both bare `@0/<0';1'>/*` and prefix `[fp]@0/<0';1'>/*` still reject.

Provenance: `design/agent-reports/constellation-bughunt-2026-06-20.md`, `design/BRAINSTORM_cycle2_funds_loss_fixes.md` + `design/IMPLEMENTATION_PLAN_cycle2_funds_loss_fixes.md` + the cycle-2 R0/impl reviews (3 spec-R0, 2 plan-R0, 3 per-finding impl reviews, 1 whole-diff exec review). Toolkit-only: no CLI-flag / `--json` wire-shape change, no `md-codec`/`mk-codec`/`ms-codec` bump, so no GUI schema-mirror or manual leg. (Version `0.62.0` taken first-to-ship; the paused own-account-subset-search cycle renumbers to `0.63.0`.)

## mnemonic-toolkit [0.61.0] — 2026-06-20

**SemVer-MINOR — FUNDS-SAFETY: three empirically-proven CRITICAL fixes from the constellation differential-oracle bug hunt (H12 / H1 / H13). Bitcoin Core `deriveaddresses` confirmed both the wrong-address divergences and their closure. No CLI-flag / `--json` wire-shape change.**

### Fixed

- **H12 — descriptor-mode taproot multisig now defaults the BIP-48 cosigner origin to script-type `3'` (P2TR), not `2'` (P2WSH).** A `tr(NUMS, multi_a/sortedmulti_a)` descriptor supplied path-less in descriptor mode previously engraved `[fp/48'/coin'/account'/2']` AND derived the cosigner key at the `2'` subtree, so every receive/change address diverged from the `3'` wallet any BIP-48 coordinator (Sparrow/Coldcard/Jade) re-derives — a non-cosignable wallet. The default-origin helper is now taproot-aware (reuses `bip48_script_type()`) at all three sites (`bundle`, `verify-bundle`, `xpub-search` intake). `3'`=P2TR is a documented de-facto interop convention. Differential-oracle-proven.
- **H1 — `verify-bundle`'s `md1_xpub_match` now compares the full policy structure, not just the pubkey multiset.** It previously returned `result: ok` for a supplied md1 reconstructing a DIFFERENT wallet — wrong threshold (e.g. 1-of-3 anyone-spends vs 2-of-3), unsorted-vs-sorted, script-type/wrapper drift, or divergent multipath change-chains. The predicate now also requires `tree == && use_site_path == && use_site_path_overrides ==` (origins excluded — legitimate elision variance), retaining the subordinate pubkey-set check. Check NAME and `--json checks[]` shape unchanged (predicate-only widening).
- **H13 — hardened multipath (`<0';1'>` / `<0h;1h>`) in a descriptor template is now REJECTED with a typed `ToolkitError::DescriptorParse`, not silently collapsed to a bare single-path key** (a silent policy-collapse to a different, derivable wallet). Hardened derivation is impossible on a watch-only (xpub) card (BIP-32 / BIP-389), so error is the only safe outcome. Malformed double-marker bodies (`<0'';1>`) are also rejected. Lockstep with `md-cli 0.8.0`.

Provenance: `design/agent-reports/constellation-bughunt-2026-06-20.md`, `design/PLAN_constellation_bughunt_fix_program.md`, `design/BRAINSTORM_cycle1_critical_fixes.md` + plan-doc + 9 R0/impl reviews. (Version `0.61.0` taken first-to-ship; the paused own-account-subset-search cycle renumbers to `0.62.0`.)

## mnemonic-toolkit [0.60.0] — 2026-06-20

**SemVer-MINOR — `bundle --md1-form=template` extends to MULTISIG + general policies; `restore`/`verify-bundle` complete a concrete watch-only wallet from a keyless multisig template + externally-supplied cosigner keys via a parallel permutation-search engine. #28 phase 2.**

### Added

- **`bundle --md1-form=template` now admits multisig (`wsh(multi/sortedmulti)`, `sh(wsh)`) and general/thresh policies** (incl. `tr(NUMS, multi_a)`) — not just the v0.59.0 canonical single-sig shape. It emits a keyless template md1 + N keyless cosigner mk1 stub cards and prints the order-sensitive **`WalletPolicyId`** (the completion checksum) on stderr, plus a **loud stderr warning about key ordering** (N keys → N! ways to assign keys to a `multi()` descriptor, only one correct; softened for order-independent `sortedmulti`). Origins are carried CONDITIONALLY: canonical multisig elides to empty (re-derived on decode); general policies carry the source per-`@N` origins for decode-validity only. `tr(sortedmulti_a)` + hardened use-sites are refused.
- **`restore --md1 <keyless multisig/general template> --from <seed>`** completes the wallet: own keys from `--from` + `--account <list>`; cosigners from new **`--cosigner <mk1>`** cards (unassigned → the search places them; or explicit `--cosigner @N=`). Three completion modes: **id-search** (`--expect-wallet-id`, a strong prefix sized to the realized search space), **address-search** (new **`--search-address`** + **`--search-addr-min`**/**`--search-addr-max`** range + **`--search-chain <receive|change|both>`**), and explicit assignment. New **`--accept-search-time`** overrides the adaptive ~1-hour search-time ceiling (with forced acknowledgment). Funds-safety floors: distinct-keys, every-slot-supplied, strong-prefix, ambiguity/no-match all refuse loudly (never a silent wrong wallet); per-slot origins are BUILT FRESH from the supplied keys (the carried template origin is never loaded). **`--own-account-max`** is reserved/refused this cycle (the own-account-range subset search is deferred — FOLLOWUP `template-multisig-own-account-range-subset-search`).
- **`verify-bundle`** gains the same completion intake — new **`--from`**, **`--cosigner`**, **`--search-address`**, **`--search-addr-min`**, **`--search-addr-max`**, **`--search-chain`**, **`--accept-search-time`** — to verify + recompose a keyless multisig/general template bundle (card↔template-id binding + the completion search; shares the exact engine `restore` uses).

### Notes

`md-codec`/`mk-codec` are **NOT bumped** — the keyless multisig wire form, `WalletDescriptorTemplateId`, and the form-aware mk-cli stub all pre-existed. New clap flags on `restore` + `verify-bundle` → GUI schema-mirror + manual updated in lockstep this cycle. Funds-safety + plan-correctness are gated by the persisted R0 reviews under `design/agent-reports/template-multisig-*`.

## mnemonic-toolkit [0.59.1] — 2026-06-20

**SemVer-PATCH — `restore --md1` faithfully reconstructs a NUMS-keyed single-leaf taproot `tr(NUMS, multi_a)` wallet card carrying per-cosigner use-site path overrides (divergent derivation suffixes). FUNDS-SAFETY: extends the v0.58.2 non-taproot override reconstruction to the taproot `multi_a` leg.**

### Changed

- **`restore --md1` now restores `tr(NUMS, multi_a)` per-cosigner use-site override cards** (non-hardened). A new single predicate `restorable_taproot_override_card` (`Tag::Tr` root ∧ NUMS internal ∧ plain `multi_a` leaf ∧ no hardened use-site) re-routes such cards from the override-blind `Template` string-builder to the faithful arm → md-codec 0.37.0's per-`@N` `to_miniscript_descriptor_multipath`. The SAME predicate drives the restore guard, the classify re-route, and the engrave-surface `TaprootUseSiteOverride` advisory (refuse ⟺ advisory fires — exact parity). Address-equivalence is gated against an independent rust-miniscript golden (divergent ≠ baseline-collapse) + a bitcoind `deriveaddresses` differential row.
- Still loudly refused (+ advised): `tr(sortedmulti_a)` overrides (the `sortedmulti_a` renderer rides the next rust-miniscript release — FOLLOWUP `restore-md1-taproot-use-site-override-arm`), taproot overrides with a non-NUMS internal key, and any hardened use-site.

### Notes

`md-codec`/`mk-codec` are **NOT bumped** (the `tr(multi_a)` leg rides #25's published md-codec 0.37.0 per-`@N` builder, which is taproot-agnostic). No new flag / wire / `ToolkitError` variant → no GUI schema-mirror change; the manual `### Unrestorable descriptor shapes` prose is narrowed.

## mnemonic-toolkit [0.59.0] — 2026-06-19

**SemVer-MINOR — `bundle --md1-form=template` emits a KEYLESS, account-agnostic single-sig template md1 (one engraving reusable by thousands of users of the same wallet TYPE); `restore`/`verify-bundle` complete the template from a seed + `--account`/`--origin`. New flags: `bundle --md1-form`, `restore`/`verify-bundle --origin` + `--expect-wallet-id`. #28 phase 1.**

### Added

- **`bundle --md1-form <policy|template>`** (default `policy` = unchanged full keyed wallet-policy md1). `template` emits a keyless, fingerprint-stripped, canonical-origin-elided **single-sig** template md1 — byte-identical across all users of a given canonical type (bip44/84/86). The whole-bundle binding re-roots on `WalletDescriptorTemplateId` (mk1 stub reflects it; ms1 is byte-unchanged). Requires a canonical single-sig shape; multisig / non-canonical / bip49-nested-segwit are refused (`TemplateFormUnsupportedShape`). The key-specific `WalletPolicyId` is printed on **stderr** (full hex + 12-word phrase + 4-byte convenience prefix) to record out-of-band — stdout (the engraved cards) is unchanged.
- **`restore --md1 <keyless-template> --from <seed>`** completes a single-sig template: `--from` is REQUIRED (a no-seed template restore is refused, not mis-routed to watch-only), `--account <N>` (canonical) or new **`--origin <path>`** (arbitrary BIP-32 origin) supplies the account/origin, and new **`--expect-wallet-id <prefix>`** recomputes the `WalletPolicyId` from the completed fully-keyed wallet and refuses loudly on mismatch (exit 4; advisory under 4 bytes; skipped under `--origin`).
- **`verify-bundle`** gains `--origin` + `--expect-wallet-id` to verify + recompose a keyless template bundle (mirrors `restore`). `self_check_bundle` branches for the keyless template form (skips the wallet-policy/pubkeys-present gates; the stub-coherence + mk1 origin/fp + ms1 parity checks still run).

### Notes

`md-codec`/`mk-codec` are **NOT bumped** — the template id (`compute_wallet_descriptor_template_id`) and the keyless wire form already exist in `md-codec 0.37.0`. Funds-safety: completion address-equivalence is gated against an INDEPENDENT golden + a D7 same-preimage round-trip (`bundle`-printed `WalletPolicyId` == `restore`-recomputed). Lockstep follow-ons (separate PRs): `mnemonic-gui` schema-mirror (`--md1-form` flag + `policy|template` dropdown + `--origin`/`--expect-wallet-id`) and the `mk-cli` `derive_stub_from_md1` template-id branch.

## mnemonic-toolkit [0.58.2] — 2026-06-19

**SemVer-PATCH — `restore --md1` faithfully reconstructs non-taproot multisig wallet cards carrying per-cosigner use-site path overrides (divergent derivation suffixes). FUNDS-SAFETY: such cards were previously loud-refused; the underlying md-codec reconstruction silently collapsed divergent per-`@N` suffixes to one shared baseline (closed in md-codec 0.37.0). `restore` now reconstructs each cosigner's own suffix faithfully, or loudly refuses the shapes it still cannot derive.**

### Changed

- **`restore --md1` now restores non-taproot, non-hardened per-cosigner use-site override cards** (`wsh(multi)`, `wsh(sortedmulti)`, `sh(wsh(multi))`, bare `sh(multi)`, and `Some`/`None` multipath mixes). The faithful arm consumes md-codec 0.37.0's `to_miniscript_descriptor_multipath`, which carries each `@N` key's own resolved multipath suffix; the `ReconstructTranslator` is reduced to network-correction only (it no longer re-promotes a shared baseline suffix). Address-equivalence is gated against an independent rust-miniscript golden (anti-vacuity: divergent ≠ baseline-collapse), with a bitcoind `deriveaddresses` differential corpus shape.
- **Guard narrowed + advisory parity.** The blanket override refusal is replaced by a refuse-iff `md_codec::has_hardened_use_site(d)` OR `taproot_override_card(d)` — ONE predicate each, shared by the restore guard and the engrave-surface `unrestorable_descriptor` advisory (refuse ⟺ advisory fires). Still loudly refused: any hardened use-site (`/*h` baseline or override — underivable from an xpub) and taproot override cards (`tr(multi_a)`/`tr(sortedmulti_a)`; tracked by FOLLOWUP `restore-md1-taproot-use-site-override-arm`).

### Dependencies

- `md-codec` `0.36` → `0.37.0` (the faithful per-cosigner reconstruction + `has_hardened_use_site` live here). `scripts/install.sh` md-cli sibling pin bumped `descriptor-mnemonic-md-cli-v0.6.2` → `descriptor-mnemonic-md-cli-v0.7.1`.

## mnemonic-toolkit [0.58.1] — 2026-06-17

**SemVer-PATCH — `convert --from mk1 --to xpub` prints a path-implied SLIP-0132 hint on stderr. Advisory-only; stdout unchanged. Tier-3 item C6.**

### Added

- **Path-implied SLIP-0132 hint on mk1 read-back.** When `convert --from mk1 --to xpub` reads a card whose origin path conventionally implies a SLIP-0132 variant (`m/49'`→ypub, `m/84'`→zpub, `m/48'/…/1'`→Ypub, `m/48'/…/2'`→Zpub), it now prints a one-line stderr note naming the variant and pointing at `--xpub-prefix` (e.g. `note: this card's derivation path is conventionally SLIP-0132 zpub; re-emit with --xpub-prefix zpub …`). **Stdout is unchanged** — it stays the BIP-32-neutral `xpub` (so `xpub→mk1→xpub` byte-identity and Bitcoin Core interop are preserved). The note suppresses when the path is neutral (44'/45'/86') or when `--xpub-prefix` is already supplied. New `slip0132::path_implied_xpub_prefix`. FOLLOWUP `mk1-card-slip0132-variant-not-preserved-on-card`. Plan + R0: `design/PLAN_C6_mk1_slip0132_stderr_hint_2026-06-17.md`.

### Notes

The mk1 card stores only the neutral xpub (the SLIP-0132 variant is destroyed on intake and cannot be recovered exactly — on-card preservation would be a breaking wire-generation bump, the product-question outcome being NOT to pursue it). The hint reconstructs the *conventional* variant from the path the card already stores; `--xpub-prefix <variant>` produces the actual SLIP-0132 form on demand. PATCH — advisory-only, zero clap delta, zero stdout change → no GUI `schema_mirror` impact (older()-advisory v0.55.2 precedent).

## mnemonic-toolkit [0.58.0] — 2026-06-17

**SemVer-MINOR — `import-wallet --format descriptor`: a generic commented-descriptor intake. Tier-2 item C5 (re-scoped from "import `--format green`"). Paired GUI dropdown-value + pin bump (mnemonic-gui v0.42.0).**

### Added

- **`import-wallet --format descriptor`** — read a watch-only descriptor from a text file, tolerating leading `#`-comment lines + blank lines, then flow it through the existing concrete-keys import pipeline → md1 bundle (watch-only). This subsumes the toolkit's own `export-wallet --format green` 3-line output AND `export-wallet --format descriptor`'s bare line AND any hand-written / foreign commented descriptor — closing the round-trip gap (export → re-import) that `--format bitcoin-core` (JSON-only) could not. Supports **both singlesig and multisig** (a descriptor carries threshold + cosigners; unlike green-*export*, which is singlesig-only). **Explicit-only** (`--format descriptor` is REQUIRED — a bare descriptor is too generic to auto-sniff, mirroring encrypted-BSMS). BIP-380 checksum is **tolerant** (validated if present — a bad checksum is refused; tolerated if absent), matching `bundle --descriptor`. New parser `wallet_import/descriptor.rs`; `ImportProvenance::Descriptor` unit variant. FOLLOWUP `import-wallet-format-descriptor`. Plan + R0: `design/PLAN_C5_import_format_descriptor_2026-06-17.md`.

### Notes

Paired GUI lockstep (mnemonic-gui v0.42.0): `IMPORT_WALLET_FORMATS` gains `"descriptor"` + toolkit pin v0.56.0 → v0.58.0. The GUI `schema_mirror` gate checks flag-NAMES only (not dropdown values), so the value addition is discipline-enforced (paired-PR rule), not gate-caught.

## mnemonic-toolkit [0.57.1] — 2026-06-16

**SemVer-PATCH — non-blocking advisory at `bundle` / `import-wallet` engrave time for descriptor shapes `restore --md1` cannot reconstruct. Tier-2 item C1. Advisory-only, zero clap delta → no GUI `schema_mirror` impact.**

### Added

- **Unrestorable-shape advisory (stderr, non-blocking).** `bundle` and `import-wallet` engrave a wire-faithful md1 card for three descriptor shapes that `restore --md1` then refuses to mechanically reconstruct: (1) `sortedmulti()` inside a combinator (not the sole child of `wsh`/`sh`), (2) per-cosigner use-site path overrides, (3) a hardened wildcard (`/*h`). Each now surfaces a `advisory: restore --md1 cannot reconstruct this descriptor — …` line on stderr at engrave time, naming the shape and the tracking FOLLOWUP. The card is still emitted (a faithful backup); exit code is unchanged (the advisory cannot flip `bundle`/`import-wallet` to failure). The advisory fires **IFF** `restore --md1` would refuse — the shape-1 predicate mirrors md-codec's `to_miniscript` acceptance set exactly (the three restorable SortedMulti positions: `wsh(sortedmulti)`, `sh(wsh(sortedmulti))`, bare-P2SH `sh(sortedmulti)`). Never suppressed under `--json` (stderr is separate from the stdout JSON payload). New module `src/unrestorable_advisory.rs`, mirroring the v0.55.2 `older()` advisory. FOLLOWUP `bundle-unrestorable-shape-advisory`. Plan + R0: `design/PLAN_C1_unrestorable_shape_advisory_2026-06-16.md`.

### Notes

The reconstruction halves of the underlying gaps stay open (`bundle-accepts-sortedmulti-in-combinator-restore-cannot`, `restore-md1-per-key-use-site-and-hardened-wildcard`) — the advisory warns; it does not yet reconstruct. PATCH — advisory-only, zero clap delta → no GUI `schema_mirror` impact (mirrors v0.55.2).

## mnemonic-toolkit [0.57.0] — 2026-06-16

**SemVer-MINOR — `verify-bundle --descriptor` now accepts a BIP-388 wallet-policy JSON (intake parity with `bundle` / `export-wallet`). Tier-2 item C2.**

### Added

- **`verify-bundle --descriptor` / `--descriptor-file` accept a BIP-388 wallet-policy JSON** `{name, description_template, keys_info}` (auto-detected by a leading `{`), expanding it to the concrete descriptor before verifying — the same `is_bip388_policy_shape` → `expand_bip388_policy` probe `bundle` and `export-wallet --descriptor` have shipped since v0.49.0. Closes the intake asymmetry: a user who can `bundle` from a wallet-policy JSON can now `verify-bundle` against the same JSON. Read-only verify → no policy-name preservation (unlike `export-wallet`). No new flag (only the accepted input FORMAT widens) → no GUI `schema_mirror` surface; the manual `--descriptor` row is updated in lockstep. FOLLOWUP `verify-bundle-bip388-policy-intake`. Plan + R0: `design/PLAN_C2_verify_bundle_bip388_intake_2026-06-16.md`.

### Notes

The previously-pinned refusal cell (`verify_bundle_refuses_bip388_policy_json`) was inverted red→green into `verify_bundle_accepts_bip388_policy_json` (a 2-of-2 sortedmulti policy bundle→verify-bundle round-trip via `--bundle-json`); a single-sig `wpkh` policy round-trip and a malformed `@N`-beyond-`keys_info` refusal (asserting the expander's `"@N beyond keys_info"` message) round out the coverage.

## mnemonic-toolkit [0.56.0] — 2026-06-15

**SemVer-MINOR — standardized mstring display-grouping (`bundle`/`convert`/`ms-shares` gain `--group-size`/`--separator`; default card output is now space/5 print-once). The cross-constellation `display-grouping-render-strip-v1` cycle, P4 (the integration crate). Pins md-codec 0.36.0.**

### Added

- **`--group-size <u16>`** (default `5`, `0` = unbroken) + **`--separator <space|hyphen|comma>`** (keyword or literal `" "|-|,`, default `space`) on **`mnemonic bundle`** (the engraved `ms1`/`mk1`/`md1` cards), **`mnemonic convert`** (when emitting an `ms1`/`mk1` card — node-gated; `xpub`/`wif`/`descriptor` outputs stay raw), and **`mnemonic ms-shares split`** / **`ms-shares combine --to ms1`**. SPEC `design/SPEC_mstring_display_grouping.md`.
- **Default-output change:** `bundle` text output is now **print-once, space/5-grouped** (previously print-twice: an unbroken copy *and* a 5-char/hyphen grouped copy with wrap@10). `md1` cards flip the legacy hyphen grouping to the uniform space default. `--json` and `verify-bundle` forensic `expected`/`actual` strings ALWAYS stay **unbroken canonical**; `repair` output stays unbroken (no grouping flags). Hence MINOR (stdout text was never a declared-stable interface; precedent v0.48.0/v0.49.0).
- **Separator-stripping intake** on every md1/ms1 intake surface (`bundle --slot @N.ms1=`, `verify-bundle --ms1/--mk1/--md1`, `repair`, `convert --from ms1=/mk1=`, `ms-shares combine`): a grouped or unbroken card both re-ingest (strips ALL whitespace + `-` + `,`). `convert --from mk1=` strips only `-`/`,` (whitespace stays the multi-chunk delimiter per SPEC §5.a).
- Conformance vectors `design/display-grouping-vectors.tsv` (+ a `display_grouping_conformance` test) — the cross-repo byte-identity gate shared with `md-codec`/`ms-cli`/`mk-cli`.
- `docs/Examples.pdf`: a new degrade2 (11-key/4-branch degrading-multisig) `bundle → restore` round-trip example proving the reconstructed first receive address is byte-identical to the canonical descriptor's.

### Changed / Removed

- `format.rs::chunk_5char` / `chunk_mk1` / `chunk_md1` are **deleted**, collapsed into the single `display_grouping::render_grouped(s, group_size, separator)` (a pure lib module, P0). Technical-manual API table + index updated.
- `md-codec` pin `0.35` → `0.36` (the published `render_grouped`/wrapper additions; additive — the toolkit now renders via its own local `render_grouped`, no longer calling `md_codec::encode::render_codex32_grouped` for display).

### Notes

Sibling CLIs shipped earlier in the cycle: **md-codec 0.36.0 + md-cli 0.7.0** (P1), **ms-cli 0.8.0** (P2), **mk-cli 0.9.0** (P3). Cross-repo lockstep: `mnemonic-gui` `schema_mirror` (flags + separator keyword dropdown) is the remaining P5 paired PR. FOLLOWUP `display-grouping-render-strip-v1`.

## mnemonic-toolkit [0.55.3] — 2026-06-13

**SemVer-PATCH — `restore --md1` now faithfully reconstructs non-NUMS ("real key at the trunk") taproot wallet-policy cards (was: every `is_nums:false` taproot md1 refused).**

- **Non-NUMS key-path taproot reconstructs.** A taproot card whose internal ("trunk") key is a real cosigner key — a live key-path spend, not the provably-unspendable NUMS sentinel — now restores to a watch-only descriptor + receive address for two shapes: (a) a **general** single-leaf / depth-1 policy (e.g. `tr(D,and_v(v:pk(K),older(N)))`) via the GeneralFaithful route-around, and (b) a **distinct-trunk multisig** `tr(cosigner_i, multi_a/sortedmulti_a(k, {the OTHER cosigners}))` via the Template path. The trunk key is read off the wire (`Body::Tr.key_index`), never inferred. NUMS reconstruction is byte-identical (v0.49.1/v0.55.1 goldens unchanged).
- **`@-in-both` refused structurally (funds-safety).** The one shape that stays refused (exit 2) is `tr(@i, multi_a/sortedmulti_a(k, …@i…))` — the trunk key is *also* a leaf key. The Template path's "leaf = all-other-cosigners" shortcut would silently emit a *different* multisig at a *different* address, and the parse→print Display-fidelity guard cannot catch it (the output is its own re-print). A STRUCTURAL classify-time guard (trunk `key_index ∈ leaf indices`) refuses it rather than emit a silently-wrong wallet. Deferred: FOLLOWUP `restore-non-nums-tr-internal-key-also-in-leaf`. (Depth-≥2 tap trees and `sortedmulti_a`-under-a-taptree remain refused, unchanged.)
- **`--format` matrix.** A non-NUMS **general-tr** emits `descriptor` / `bitcoin-core` only — `bip388` and `green` are refused (a tap-script tree reconstructed via the route-around has no named-template form; the refusal is gated to the route-around arm). A non-NUMS **distinct-trunk multisig** also emits `bip388` faithfully (`tr(@idx/**,multi_a(k,…))`) via its Template path. The template-requiring formats (`bsms`/`coldcard`/`jade`/`electrum`/`sparrow`/`specter`) refuse the general-tr arm.
- **PATCH — watch-only, zero clap delta** → no GUI `schema_mirror` impact, no paired-PR. Resolves FOLLOWUP `restore-non-nums-taproot-internal-key` (the `is_nums:false` carve-out deferred by `restore-multisig-taproot-reconstruction` at v0.49.1).

## mnemonic-toolkit [0.55.2] — 2026-06-13

**SemVer-PATCH — non-blocking advisory when an intake/round-trip descriptor's `older()` relative timelock is BIP-68 consensus-masked (e.g. `older(65536)` → 0 effective blocks).**

- **Advise-on-intake (the other half of v0.53.9's authoring gate).** The seven intake/round-trip surfaces — `bundle`, `export-wallet --descriptor`, `import-wallet`, `xpub-search`, `verify-bundle --descriptor`, `restore --md1`, `compare-cost` — now print a non-blocking stderr advisory when a descriptor's `older(N)` carries garbage bits or a zero 16-bit value (consensus masks the operand to `0x0040FFFF`, silently weakening or nullifying the timelock). The command still **succeeds** (exit unchanged): blocking would strand recovery/backup of an already-deployed wallet. The authoring surface `build-descriptor` keeps its existing hard refuse — both halves now share one bit-math predicate (`timelock_advisory::older_consensus_masked`), so the `build-descriptor` diagnostic is byte-identical.
- **Two walk adapters, one predicate.** A new `timelock_advisory` module: the shared BIP-68 predicate plus an md_codec node-tree walker (md1-card / `parse_descriptor` surfaces; bit-31 reachable on raw-card decode) and a generic miniscript-AST walker (`from_str` surfaces; bit-31 fail-closed at parse). Advisories dedupe by operand; the bit-31 "no timelock at all" form is distinguished from the masked-to-`<effective>` form. All descriptor-emit paths covered (incl. `bundle`'s concrete-key and import-json paths, `verify-bundle`'s concrete fork, `sh(<miniscript>)`, taproot leaves); structurally timelock-free paths (`--template`, single-sig restore) correctly excluded.
- **PATCH — advisory-only, zero clap delta** → no GUI `schema_mirror` impact. `after()` needs no advisory (fail-closed at `from_str`). Resolves FOLLOWUP `intake-surfaces-accept-masked-older-no-advisory`; spawned `older-advisory-blindness-suppression` (WONTFIX).

## mnemonic-toolkit [0.55.1] — 2026-06-12

**SemVer-PATCH — `restore --md1` now faithfully reconstructs single-leaf and depth-1 two-leaf `tr(NUMS,<general miniscript>)` wallet-policy cards (was: refused "not a recognized multisig").**

- **General taproot restore (GAP-1 T3-partial).** A `tr(NUMS,…)` md1 whose tap tree is a single general miniscript leaf (e.g. `and_v(v:pk(K),after(N))`) or a depth-1 two-leaf taptree (e.g. `{pk(K0),pk(K1)}`, `{multi_a(2,K0,K1),pk(K2)}`) now routes the same faithful general-policy arm as `wsh` (`faithful_multisig_descriptor`), printing the full policy tree — timelocks, hashlocks, structure — instead of refusing. The reconstructed descriptor carries the BIP-341 NUMS H-point as hex (not the literal `NUMS` token), md-codec's depth-0 reconstructed xpubs, and the `<0;1>/*` multipath, exactly like the wsh general arm. Golden addresses independently cross-verified against Bitcoin Core.
- **Conservative structural gates (never Display-luck), `ModeViolation` exit 2, slug-citing.** Depth ≥2 taptrees (≥3 leaves) refuse — the pinned miniscript `95fdd1c` mis-prints a LEFT-child taptree, and the gate refuses right-spine shapes too (chirality-independent; lift when the miniscript #953 fix releases): `upstream-miniscript-taptree-depth2-display-asymmetry`. `sortedmulti_a` under a taptree refuses (`md-codec-sortedmulti-a-to-miniscript-rendering-gap`; single-leaf `sortedmulti_a` keeps the byte-identical template path). Non-NUMS (cosigner) internal keys stay refused.
- **`--format green` refuses the general-tr arm explicitly** (exit 1): `script_type_from_descriptor` classifies a general tr without `multi_a(` as `P2tr` (taproot singlesig), so green's `is_multisig` gate would have emitted a "singlesig" payload for a script-tree policy. `bip388` keeps its loud multipath refusal (the NUMS internal key carries no `/<0;1>/*` suffix); `descriptor`/`bitcoin-core` emit faithfully; everything else refuses as before.
- **Display-fidelity guard.** The reconstructed descriptor must survive its own parse→print round-trip before any address derivation or payload emission — the guard against a future parseable-but-wrong Display infidelity in the pinned miniscript.

## mnemonic-toolkit [0.55.0] — 2026-06-12

**SemVer-MINOR (wire-content change) — the descriptor walker emits a bare `PkK`/`PkH` leaf in `wsh`/`sh` (was `Check(PkK)`), conforming to descriptor-mnemonic SPEC v0.30 §5.1 and matching `md-cli`.**

- **Canonical Check-collapse.** The toolkit's `parse_descriptor` walker GATED the `Terminal::Check(PkK|PkH) → bare Tag::PkK|PkH` collapse on `tap_context`: it collapsed inside taproot leaves but KEPT `Tag::Check(Tag::PkK)` on the wire in `wsh`/`sh`. descriptor-mnemonic SPEC v0.30 §5.1 mandates bare `PkK`/`PkH` regardless of context (`Tag::Check` is never emitted wrapping a key leaf on the wire), and `md-cli` collapses unconditionally — so the toolkit engraved a NON-CONFORMANT `md1` for `wsh(pk)`-shaped descriptors and computed a DIFFERENT `wallet_policy_id` than `md-cli` for the same wallet (an interop hazard: a card made by `mnemonic` did not match one made by `md`). v0.55.0 drops the gate; the walker collapses `Check(PkK|PkH)→bare` unconditionally (the fall-through `Tag::Check` emit for `Check`-over-non-key is preserved). The now-dead `tap_context` parameter is removed.
- **Affected shapes (8).** Anything carrying a `Check(PkK|PkH)` in non-tap context changes its `md1` wire output and therefore its `wallet_policy_id`/`wallet_descriptor_template_id`: `wsh(pk)`, `wsh(pkh)`, `wsh(and_v(…pk…))`, `wsh(or_d(…pk…))`, `wsh(thresh(…pk…))`, `sh(pk)`, `sh(pkh)`, `sh(wsh(pk))`. Common shapes are UNAFFECTED: `wpkh`/`pkh` top-level (no miniscript walk), `sh(wpkh)`, `wsh(multi)`/`wsh(sortedmulti)` (use `Multi`/`SortedMulti`, not `Check`), and `tr`/tap leaves (already collapsed). This is WIRE-canonicity, NOT funds-loss — both forms decode to the identical descriptor.
- **No card-reading regression.** `restore --md1` reads BOTH the old `Check(PkK)` and the new bare-`PkK` `md1`: md-codec 0.35.1's `to_miniscript` re-wraps a bare `Tag::PkK` as `Check(pk_k)` at the miniscript layer and its `Check`-over-bare-key idempotence arm collapses `Check(PkK)` to the same `c:pk_k` — both wire forms produce the identical miniscript AST. Cards engraved by any prior toolkit version remain readable.
- **Scope honesty**: this changes the EMIT side only; SemVer-MINOR (wire-content change for a class of descriptors, no CLI/schema/manual change), per the [0.48.0] NUMS-wire-change precedent.
- **Wire-change test coverage (user mandate "if we change the wire format, make sure our tests cover it").** New always-on in-suite golden `tests/cli_check_pkk_canonical_golden.rs` pins the post-fix `wallet_policy_id`+`wallet_descriptor_template_id` for the four flagship shapes — `bundle --descriptor … --json` → `.md1` → decoded IN-CRATE via `md_codec::chunk::reassemble` → `compute_wallet_policy_id`/`compute_wallet_descriptor_template_id` (no external binary) — plus a `bundle → restore --md1` round-trip on `wsh(pk)`/`wsh(pkh)`. The four AST unit tests in `parse_descriptor.rs` were inverted to assert the bare leaf (`walk_check_kept_in_non_tap_context` renamed to `walk_check_collapsed_in_non_tap`), and the Cycle-D cross-tool differential (`tests/cli_cross_tool_differential.rs`) flips the four formerly-`Diverge` entries to `Match` (its anti-vacuity guards restructured to a verdict-agnostic check: ≥1 real `Match` + no silent `BothError`/`ToolError` — now a cross-tool re-divergence regression gate).
- **Tests.** Full toolkit suite green (the 4 AST tests pass inverted; the new golden passes; `prop_backup_restore_roundtrip` 9/9) + clippy `--all-targets -D warnings` (0); differential confirmed locally (`MNEMONIC_BIN`+`MD_BIN`, all 8 entries `Match`). Resolves `toolkit-check-pkk-non-tap-non-canonical` (surfaced by stress-Cycle-D; companion in descriptor-mnemonic). Audit trail: `design/BRAINSTORM_check_pkk_non_tap_canonical_fix.md` + `design/agent-reports/check-pkk-fix-r0-round{1,2}-review.md`.

## mnemonic-toolkit [0.54.4] — 2026-06-12

**SemVer-PATCH (SECURITY) — error rendering no longer echoes secret ms1 input (ms-codec 0.4.4 pin bump).**

ms1 is secret-bearing. Before ms-codec 0.4.4, `ms_codec::Error`'s `Display`/`Debug` embedded the raw input — `Codex32(InvalidChecksum{string})` carried the full secret on a checksum failure, and `WrongHrp{got}` could echo a long secret prefix. The toolkit inherited this: any `{:?}` of a `ToolkitError` wrapping `MsCodec(_)` (panics, `expect`, logs) and the `friendly_ms_codec` mapper transitively rendered it. ms-codec 0.4.4 closes the leak at the source — the `Codex32` arm is a structural-only variant match, `WrongHrp.got` is capped to 4 chars at construction, and a hand-rolled `Debug` replaces the leaky derive — so the toolkit inherits the protection with no code change. The `repair.rs` wrapper's surfaced `HrpMismatch.found` is now bounded for free.

Dependency-version bump only (Cargo.lock + pin `0.4.3` → `0.4.4`); no toolkit source-logic / CLI / wire change beyond the pin. 1 regression cell (`repair.rs::repair_via_ms_codec_wrong_hrp_found_is_bounded`). Resolves the toolkit companion of `ms-codec-error-display-echoes-input`.

## mnemonic-toolkit [0.54.3] — 2026-06-12

**SemVer-PATCH — `mnemonic repair` no longer panics on a non-ASCII chunk with no `'1'` separator (ms-codec 0.4.3 pin bump).**

The auto-repair path (`repair.rs::repair_via_ms_codec`) calls `ms_codec::decode_with_correction`, which before 0.4.3 sliced `lower[..len-1]` when reporting the observed HRP of a non-`ms1` string. With no `'1'` separator, `len-1` can land inside a multi-byte char (e.g. a trailing `é`) → char-boundary panic — so `mnemonic repair café` (and the indel-repair oracle) aborted instead of returning a clean error. ms-codec 0.4.3 slices at `rfind('1')` (`'1'` is ASCII, always a char boundary) and uses the whole string as the observed HRP when there is no separator; the toolkit's wrapper maps it to a clean `HrpMismatch`. Found by stress-Cycle-C fuzzing (mnemonic-secret `fuzz/`); resolves the ms-codec FOLLOWUP `decode-with-correction-panics-on-non-char-boundary-hrp-slice`.

Dependency-version bump only (Cargo.lock ms-codec 0.4.2 → 0.4.3, Cargo.toml pin `0.4.2` → `0.4.3`); no toolkit source-logic / CLI / wire change beyond the pin. 1 regression cell (`repair.rs::repair_via_ms_codec_no_separator_multibyte_is_clean_error`).

## mnemonic-toolkit [0.54.2] — 2026-06-11

**SemVer-PATCH — `export-wallet --from-import-json` refuses general policies for template-requiring formats instead of silently collapsing them (funds-safety C2).**

### Fixed

- **`export-wallet-from-import-json-template-collapse` (C2).** `export-wallet --from-import-json <env> --format <sparrow|coldcard|coldcard-multisig|jade|electrum>` for a GENERAL-policy descriptor (timelocks/hashlocks/andor/decay) silently collapsed it to plain multisig — the same top-level-wrapper-only `template_from_descriptor` (`Wsh(_) => WshMulti`) collapse fixed on the restore path in v0.54.0, on the export `--from-import-json` door. A template-requiring (k-of-n multisig) format genuinely cannot represent a general miniscript policy, so the export now REFUSES loudly (new `descriptor_is_general_policy` structural gate) instead of emitting a wrong/partial payload. Descriptor-passthrough formats (`bitcoin-core`/`descriptor`/`bip388`/`bsms`/`green`/`specter`) emit the faithful descriptor unchanged; **singlesig** (`pkh`/`wpkh`/`sh(wpkh)`) and **plain multisig** still map to their template and export as before (the gate refuses ONLY general policies — R0-caught regression guard).
- Tests: `wallet_export::descriptor_is_general_policy` unit cells (general vs plain-multisig vs singlesig) + `tests/cli_export_wallet_from_import_json.rs` (general→template-format refusal RED-proven; general→passthrough faithful keeps `older(1000)`; singlesig→template-format unchanged).

### Notes

No CLI flag/subcommand/value change → no `schema_mirror`/GUI/manual/sibling lockstep (a new refusal on an existing flag combination). SPEC + R0 ×2 GREEN: `design/SPEC_c2_from_import_json_general_policy_gate.md`, `design/agent-reports/c2-from-import-json-gate-r0-round{1,2}-review.md`. Completes the general-policy collapse cleanup begun with the restore C1 fix (v0.54.0/v0.54.1).

## mnemonic-toolkit [0.54.1] — 2026-06-11

**SemVer-PATCH — `restore --md1` now reconstructs `pk(@N)`/`pkh(@N)`-keyed wallet policies (md-codec 0.35.1 pin bump; PART 2 of the restore C1 fix).**

### Fixed

- **`to-miniscript-check-pkh-double-wrap` (PART 2).** v0.54.0 reconstructed multi-keyed general policies faithfully but LOUD-REFUSED policies whose keys appear as bare `pk(@N)`/`pkh(@N)` outside a `multi()` (e.g. the v0.19.0 flagship `wsh(andor(pkh(@0),after(N),or_i(and_v(v:pkh(@1),older(M)),and_v(v:pkh(@2),older(K)))))`) — md-codec's `to_miniscript` rendered a `Check(Check(PkH))` double-wrap that errored "cannot wrap a fragment of type B". md-codec **0.35.1** fixes the renderer (Check-idempotence collapse); this release bumps the pin (`Cargo.lock`: md-codec 0.35.0 → 0.35.1) so those policies reconstruct through the SAME v0.54.0 general arm — **zero toolkit code change**, just the dep update + tests. Every key-check fragment + all timelocks survive (verified end-to-end on the flagship).
- Tests (`tests/cli_restore_multisig_general.rs`): `flagship_pk_keyed_vault_reconstructs` (the v0.19.0 flagship) + `general_pkh_leaf_reconstructs_after_md_codec_fix` (flipped from the v0.54.0 refusal cell). md1 fixed-point oracle on both.

### Notes

No toolkit source/CLI/wire change — a dependency-version bump (md-codec 0.35.0 → 0.35.1, published to crates.io 2026-06-11) + test cells. No `schema_mirror`/GUI/manual delta beyond v0.54.0's. Companion: `descriptor-mnemonic` md-codec 0.35.1 (`design/SPEC_to_miniscript_check_pkh_double_wrap.md`, R0 GREEN). Completes the long-term restore C1 fix (PART 1 = v0.54.0 general arm; PART 2 = this). Remaining: C2 (`export-wallet --from-import-json` same collapse).

## mnemonic-toolkit [0.54.0] — 2026-06-11

**SemVer-MINOR — `restore --md1` reconstructs GENERAL wallet-policy descriptors faithfully (funds-safety: was silently collapsing them to plain multisig).**

### Fixed

- **C1 (funds-safety).** `restore --md1` SILENTLY reconstructed a *different* wallet for general `wsh`/`sh(wsh())` policies whose keys sit inside `multi()`/`sortedmulti()`: a `wsh(and_v(v:multi(2,…),older(4032)))` md1 restored as a plain `wsh(multi(2,…))` — the `older` timelock GONE — at exit 0, with a false "verified" banner and a wrong importable payload. Root cause: restore computed the faithful descriptor (`to_miniscript_descriptor`) then DISCARDED it via a top-level-wrapper-only template classifier (`Wsh(_) => WshMulti`) and rebuilt a plain `multi(k, all-keys)`. The general arm now KEEPS the faithful descriptor (with a `translate_pk` pass for canonical multipath + `--network`-correct keys), preserving the full policy tree (timelocks, hashlocks, andor, decay vaults). Addresses derive from the emitted descriptor (self-consistent). Discrimination is structural on the md1 tree (`plain_template_from_tree`), so plain `multi`/`sortedmulti` and taproot reconstruct **byte-for-byte unchanged** (all 13+12 existing goldens green).
- General policies are labeled `wallet_type: "miniscript-policy"` (header "miniscript policy restore (N cosigner[s])"); the top-level `--json` `threshold` is `null` (a general policy has no single k-of-n threshold). Descriptor-driven `--format`s (`bitcoin-core`/`descriptor`/`bsms`) emit the faithful descriptor (`bip388` too for a multipath `/<0;1>/*` card; it refuses a wildcard-only one); template-requiring k-of-n formats (`coldcard`/`jade`/`electrum`/`sparrow`/`green`/`specter`) refuse loudly.
- A card that cannot be reconstructed faithfully — per-cosigner use-site path overrides (cosigners not sharing one multipath suffix) or a hardened wildcard (`/*h`) — is REFUSED loudly (the md-codec reconstruction renders one shared, unhardened use-site for all keys, which would silently misrepresent the wallet). The engraved card remains a faithful backup. Tracked: `restore-md1-per-key-use-site-and-hardened-wildcard`.
- **Scope boundary:** `pk(@N)`/`pkh(@N)` key-leaf policies (toolkit-authored wire carries a `Check(Check(PkH))` double-wrap) currently hit a CLEAR loud refusal naming the md-codec follow-up (`to-miniscript-check-pkh-double-wrap`) — no longer a silent collapse or the cryptic k-gate. They reconstruct through the same general arm once that md-codec fix ships (PART 2). Multi-keyed general policies reconstruct **now**.
- Tests: new `tests/cli_restore_multisig_general.rs` (and_v+older RED-proven; decay vault; sha256 hashlock; `--format descriptor` faithful; `--format coldcard` refuses; pkh-leaf clean refusal; wildcard-only `multipath==None` path). md1 fixed-point oracle (re-bundling the reconstruction reproduces the card) since byte-equality with `export-wallet --descriptor` is impossible (md1 keys are depth-0).

### Notes

No CLI flag/subcommand/value change → no `schema_mirror`. The `restore --json` envelope gains `wallet_type: "miniscript-policy"` + `threshold: null` for general policies (wire-shape change, no drift gate → GUI paired-PR + manual restore-chapter update). SPEC + R0 ×3 GREEN: `design/SPEC_faithful_general_policy_restore.md`, `design/agent-reports/faithful-general-policy-restore-r0-round{1,2,3}-review.md`. Companion fixes filed: PART 2 (md-codec `Check` double-wrap, `to-miniscript-check-pkh-double-wrap`), C2 (`export-wallet --from-import-json` same collapse).

## mnemonic-toolkit [0.53.9] — 2026-06-11

**SemVer-PATCH — `build-descriptor` rejects BIP-68 `older()` timelocks that consensus would silently weaken or zero (funds-safety).**

### Fixed

- **`gate-after-older-upper-bound-deferred-to-step2`.** `build-descriptor`'s step-1 field gate accepted `older(N)` values that BIP-68 consensus silently masks to a weakened or zero relative timelock, building a checksummed, engraving-ready descriptor with no warning. BIP-68 uses only the low 16 bits as the value and bit 22 (`0x400000`) as the 512-second-unit flag; consensus masks the CSV operand to `0x0040FFFF`, so bits 16–21/23–30 are dropped and a zero 16-bit value is a no-op lock. So `older(105120)` ("2-year vault" in blocks) silently became ~275 days, `older(65536)` became **0 — spendable immediately**, and `older(0x400000)` became a zero-delay time lock. The gate now rejects any `older(N)` with a bit outside the value/type-flag field **or** a zero 16-bit value (`(n & !0x0040FFFF) != 0 || (n & 0xFFFF) == 0`), with a node-localized diagnostic stating the consensus-effective value — or, for a bit-31 (disable-flag) input, that consensus would treat it as a no-op (not a masked value). Valid block (`1..=65535`) and 512-second-unit (`0x400001..=0x40FFFF`) encodings are unaffected; all archetype presets and existing fixtures use safe values. `after(N)` additionally gets a step-1 upper-bound check (`> 0x7FFFFFFF`) for a node-localized field diagnostic — behavior-equivalent (step-2 `from_str` already rejected it).
- Tests: `descriptor_builder/gate.rs` (`rejects_masked_older_timelocks` incl. the bit-31 no-op-wording branch; `accepts_valid_older_block_and_time`; `rejects_after_above_max`; all RED-proven) + CLI cells in `tests/cli_build_descriptor.rs` (`--spec` masked `older(65536)` exit-2 with `effective value`; preset `--archetype kofn-recovery --older 105120` exit-2 with `--older` provenance). The pre-existing `mixed_timelock_spec()` fixture used `older(0x400000)` (a no-op the gate now correctly rejects); swapped to `older(0x400001)`, a valid 512-second time lock that preserves its height/time mix.
- **Carved out to FOLLOWUPs:** `archetype-older-blocks-flag-accepts-time-units` (the preset `--older` flag is documented "blocks" but still accepts the narrow valid 512-second-unit window — near-nil exposure; the `validate_params` "does-not-duplicate-gate-rules" boundary makes the bound a separate design call) and `intake-surfaces-accept-masked-older-no-advisory` (import/round-trip surfaces still accept masked `older()` from already-deployed wallets, as they must — future non-blocking advisory).

### Notes

No CLI flag/help/subcommand change and the `--spec` JSON grammar (`"older"`/`"after"` = `uint`) is unchanged → no `schema_mirror` / GUI / sibling-codec lockstep. Manual prose accuracy: the `older()` domain line in `docs/manual/src/40-cli-reference/41-mnemonic.md` was corrected from `1 ≤ N < 2³¹` to the masked domain. Discovered via deep recon of three deferred footguns (the other two — `addresses-env-sentinel-overapplied`, `two-miniscripts-patch-load-bearing-stale-error` — were verified WONTFIX). SPEC + R0 ×3 GREEN: `design/SPEC_older_timelock_mask_gate.md`, `design/agent-reports/older-timelock-mask-gate-r0-round{1,2,3}-review.md`.

## mnemonic-toolkit [0.53.8] — 2026-06-11

**SemVer-PATCH — `export-wallet --format bip388` preserves the wallet-policy `name` across a round-trip.**

### Fixed

- **`bip388-policy-name-lossy-roundtrip`.** `export-wallet --descriptor <bip388-policy-json> --format bip388` lost the policy `name`: the expand step dropped it and the emit hardcoded `"imported-descriptor"`. The policy `name` is now lifted (new `wallet_import::pipeline::bip388_policy_name`) into the existing `EmitInputs.wallet_name` channel — whose `--descriptor`-path default is ALREADY `"imported-descriptor"`, so unnamed inputs are unchanged — and the emit reads it. A named policy now round-trips its name; precedence is `--wallet-name` flag > policy name > `"imported-descriptor"`. The lift is GENERAL (consistent with the import-json `resolved_wallet_name` precedent), so a named policy also unblocks `--format specter` (which previously refused the silent default name): the policy name becomes the Specter `label`. `build-descriptor --emit-spec bip388` is unchanged (passes the `DEFAULT_BIP388_POLICY_NAME` default).
- Tests (`tests/cli_bip388_policy_intake.rs` + `wallet_import/pipeline.rs`): T1 (one-step round-trip preserves `"test-vault"`; RED-proven), T2 (unnamed → default), T3 (`--wallet-name` overrides), T4 (extractor unit: name / `None`-on-malformed), T5 (Specter unblock; RED-proven). The two-step `--format descriptor → bip388` path still legitimately drops the name (intermediate concrete step has no policy metadata) — comment corrected.
- **Carved out (separate gap, FOLLOWUP `bip388-template-path-wallet-name`):** `--format bip388` on the `--template` path still emits `template.human_name()` and ignores `--wallet-name` — NOT touched by this fix.

### Notes

No CLI flag/help/subcommand change; the `--format bip388` JSON `name` field already exists (no wire-schema add/remove) → no `schema_mirror` / manual / GUI / sibling-codec lockstep. SPEC + R0 ×2 GREEN: `design/SPEC_bip388_policy_name_roundtrip.md`, `design/agent-reports/bip388-policy-name-roundtrip-r0-round{1,2}-review.md`.

## mnemonic-toolkit [0.53.7] — 2026-06-11

**SemVer-PATCH — `silent-payment` auto-detects the BIP-39 phrase language (no longer English-only).**

### Fixed

- **`silentpayment-phrase-english-only`.** `silent-payment`'s seed resolver parsed a raw BIP-39 phrase as **English only** (`resolve_master_xpriv`), so a valid non-English phrase (Japanese, Spanish, French, …) was wrongly refused — even though the crate ships all wordlists. The raw-phrase branch is now **English-first, then auto-detect** (`parse_in(English).or_else(|_| parse(s))`): English phrases parse exactly as before (preserving behavior, incl. any word-ambiguous-across-wordlists case), and only English *failures* fall through to `bip39::Mnemonic::parse` (NFKD-normalize + `language_of` over the `all-languages` set). The seed derives from the actual phrase WORDS, so this is funds-relevant — a Japanese phrase now derives its own seed rather than being rejected. A genuinely ambiguous non-English phrase surfaces `AmbiguousLanguages` (there is no `--language` flag, so a clear refusal is correct). The ms1 branch already resolved per-card language; the entropy-hex branch correctly stays English (raw entropy has no wire language).
- Tests: `cmd/silent_payment.rs` phrase-language module — T1 (Japanese phrase resolves; RED-proven via the English-only path), T2 (same-entropy Japanese vs English derive DISTINCT xprivs — words-based), T3 (English no-regression pinned to the **published BIP-39 "abandon … about" root xprv** — a non-circular external oracle). The Japanese external-seed-vector cross-check (T4) is deferred to FOLLOWUP `silentpayment-japanese-bip39-seed-vector-cross-check`.

### Notes

No CLI flag/help/subcommand/wire change → no `schema_mirror` / manual / GUI / sibling-codec lockstep. SPEC + R0 ×2 GREEN: `design/SPEC_silentpayment_phrase_language_autodetect.md`, `design/agent-reports/silentpayment-phrase-autodetect-r0-round{1,2}-review.md`.

## mnemonic-toolkit [0.53.6] — 2026-06-11

**SemVer-PATCH — two audit-backlog hardening fixes: gate the import-json `schema_version`, and zeroize derived private-key strings in `silent-payment` / `nostr`.**

### Changed

- **`import-json` schema-version gate** (`import-json-schema-version-unchecked`). The import-json envelope carried `schema_version` (outer `"1"`, inner bundle `"4"`) into deserialized `String` fields that NO consumer read — so a future incompatible envelope would be silently mis-parsed (serde drops unknown fields). `parse_import_json_envelopes` (the shared chokepoint for both `export-wallet --from-import-json` and `bundle --import-json`) now validates the SELECTED envelope's outer + inner `schema_version` strict-equal against the supported `"1"`/`"4"`, rejecting anything else with `BadInput` ("unsupported import-json … schema_version …; upgrade the toolkit") — fail-closed, zero regression for valid current envelopes.
- **Zeroize derived private-key strings** (`silentpayment-nostr-priv-not-zeroizing`). `silent-payment`'s `scan_priv`/`spend_priv` (hex of the derived scan/spend secret bytes) and `nostr`'s `wif` (full spending key) — which propagate into the `--json` envelope, the `electrum` import string (`{prefix}{wif}`), and the text output — were plain `String`s that lingered un-scrubbed in the heap after the command returned. They now use a new serialize-transparent `SecretString(Zeroizing<String>)` (`crate::secret_string`) that zeroizes every copy on drop. The `--json` wire-shape + text output are BYTE-IDENTICAL (Serialize = `serialize_str`; Display/Deref transparent); a length-only `Debug` prevents accidental log/panic leaks. Best-effort caveat: the emitted bytes (stdout/pipe) and the secp256k1 source keys are out of scope. Added 4 `lint_zeroize_discipline` rows + widened the row-count range. NOT yet covered (filed FOLLOWUPs do not exist for these — they are inherent): the secret is intentionally on stdout.

### Notes

No CLI flag/help/subcommand/wire change → no `schema_mirror` / manual / GUI / sibling-codec lockstep. SPEC + R0 ×2 GREEN: `design/SPEC_import_json_schema_gate_and_secret_zeroize.md`, `design/agent-reports/import-json-gate-and-zeroize-r0-round{1,2}-review.md`.

## mnemonic-toolkit [0.53.5] — 2026-06-10

**SemVer-PATCH — bump ms-codec 0.4.0 → 0.4.2: all-uppercase ms1 cards now decode end-to-end (completes audit M11), and `ms-shares combine` inherits a secret-leak guard fix.**

### Changed

- **ms-codec pin 0.4.0 → 0.4.2** (now published to crates.io). v0.53.3 made the toolkit's HRP *probes* case-insensitive (uppercase mk1/md1 worked; uppercase ms1 routed but the codec still rejected it). With 0.4.2 the ms-codec envelope canonicalizes case, so **uppercase ms1 now decodes end-to-end** — `inspect`, `repair`, and `silent-payment` on an all-uppercase ms1 card now produce the same result as the lowercase twin (verified byte-identical). Closes the ms1 leg of `hrp-classifier-rejects-valid-uppercase-cards` (audit M11).
- **SECURITY (inherited):** `mnemonic ms-shares combine` delegates to ms-codec's `combine_shares`, which in 0.4.2 fixed a guard bypass — a uniform-uppercase secret-at-`S` card previously slipped past `SecretShareSuppliedToCombine` (a raw `b's'` comparison missed `b'S'`) and the interpolation short-circuit would have RETURNED the secret. `mnemonic ms-shares combine --share <uppercase secret-at-S>` now refuses cleanly (exit 2), pinned by a red-first toolkit cell. (Also inherits 0.4.1's combine non-standard-length validation.)

The 4 staged uppercase-ms1 characterization cells (which asserted the old `WrongHrp` attribution) are inverted to assert decode-success; mixed-case ms1 still rejects. No CLI flag/surface change. Plan + 2 R0 rounds + impl review: `design/PLAN_ms_codec_pin_bump_0_4_2.md`, `design/agent-reports/ms-codec-pin-bump-*.md`.

## mnemonic-toolkit [0.53.4] — 2026-06-10

**SemVer-PATCH — the friendly error mapper no longer echoes a corrupt ms1's full input on stderr (leak-hardening).**

### Fixed

- A bad-checksum lowercase ms1 (uncorrectable, or on the `--no-auto-repair`/piped path) rendered through the friendly mapper's `codex32::Error` catch-all, whose `{:?}` Debug-print of `InvalidChecksum { string }` dumped the FULL near-secret on stderr. An explicit arm now withholds it: `ms1 codex32: invalid <short|long> checksum (<N> chars; input withheld)` — the checksum kind + length stay (so a wrong-length card is spottable) but the bytes never appear. Withholding is FULL, not the v0.53.3 `UnknownHrp` head-truncation (ms1 chars 9+ are payload, so any head-echo would leak payload). Variant sweep confirmed `InvalidChecksum` is the only catch-all-reachable codex32 variant carrying the input.

Resolves `friendly-ms1-invalidchecksum-echoes-full-input` (spawned by v0.53.3). 1 integration cell (red-first — full input on stderr today) + 1 unit redaction-pin cell. No CLI/schema/manual-flag impact.

## mnemonic-toolkit [0.53.3] — 2026-06-10

**SemVer-PATCH — HRP probes are case-insensitive (audit M11): valid all-uppercase cards route to the right codec instead of a misattributed error; codecs stay the authority on case.**

### Fixed

- **All seven card-type probe sites lowercase before the HRP check and pass the ORIGINAL string to the codec** (`classify_hrp_prefix` — which also fixes the inspect/repair/verify-bundle positionals, the v0.53.2 positional secret-argv advisory, and seed intake transitively — plus `restore --cosigner`, `xpub-search --target-xpub`/`--descriptor`/address-of-xpub, and `silent-payment`'s secret-kind dispatch). Uppercase mk1 and md1 cards now work END-TO-END (mk-codec/md-codec self-normalize; BIP-173 uppercase is what QR alphanumeric mode produces, so engraved/QR'd cards legitimately come back uppercase). Never normalize-at-intake: mixed-case still reaches the codecs' own deliberate rejections (mk `MixedCase`, codex32 `InvalidCase`; md-codec accepts mixed — characterization-tested, its leniency, not ours).
- **The typed-flag case-mismatch rejection is RELAXED** (the v0.24.0 I5 "lowercase canonical" gate): `--mk1 MK1…` is now accepted (the codecs are the case authority); a true HRP mismatch (`--ms1 MK1…`) still rejects with the expected/got message. The surface was already inconsistent — `restore --md1` and `convert` accepted uppercase all along.
- **Uppercase ms1 is now correctly ATTRIBUTED but still cannot decode:** the audit's "codecs decode all-uppercase" claim is FALSE for ms-codec 0.4.0 (its envelope layer compares the raw HRP/share-index case-sensitively past codex32) — recon overturned it against the pinned source. Companion filed both repos (`ms1-envelope-uppercase-bip173`); the toolkit pin bump flips the leg when ms-codec ships it. Until then an uppercase ms1 gets the honest `ms1 wrong HRP: got "MS", expected "ms"` (and the secret-argv advisory now FIRES for uppercase positional ms1 — previously the case-sensitive probe skipped it).
- **Secret-echo rider:** `UnknownHrp`'s error display no longer echoes the full unrecognized positional — truncated to 12 chars + `…` (previously an uppercase ms1 positional put the FULL master secret on stderr via this path).

16 new/inverted test cells (red-first). No CLI flag/surface change (no GUI schema_mirror or manual flag-coverage impact). Resolves audit-2026-06-10 `hrp-classifier-rejects-valid-uppercase-cards` (M11). Plan + 4 R0 rounds + impl review: `design/PLAN_hrp_case_insensitive_probes.md`, `design/agent-reports/hrp-case-insensitive-*.md`; recon `cycle-prep-recon-hrp-case-insensitive.md`.

## mnemonic-toolkit [0.53.2] — 2026-06-10

**SemVer-PATCH — audit minors M1+M2+M3+M13: self-check binds mk1 xpubs slot-exactly, inspect/repair warn on inline ms1 argv, `localize` enforces its invariant, 25 stale goldens deleted.**

### Fixed

- **`bundle --self-check` now binds every decoded mk1 card's xpub to the descriptor SLOT-EXACTLY** (audit M1): `desc.tlv.pubkeys` must contain `(slot, xpub_to_65(card_xpub))` in both the Single and Multi branches — previously a wrong-xpub mk1 card (or two cosigner cards swapped between slots, which also passes the stub-equality check) sailed through self-check while `verify-bundle` would have caught it. Slot-exact is sound because pubkeys and cards are emitted from the same slots vector in the same order at every call site. 3 new synthesize-then-mutate cells (single / multi / cross-slot swap; the multisig fixture deliberately uses distinct per-slot seeds — a self-multisig fixture would make swaps undetectable by construction).
- **`inspect` and `repair` fire the secret-argv advisory for inline ms1** (audit M3): the shared intake (`resolve_groups`) warns per-occurrence on raw pre-stdin-expansion argv values — unconditionally for a non-`-` `--ms1` value (so an indel-corrupted prefix under `repair --max-indel` still warns) and HRP-probe-gated for positionals (`positional ms1`); recommended alternative `--ms1 -` in both. `--ms1 -` itself and mk1/md1 values never warn. Manual: advisory rows added to both command sections.
- **`build-descriptor`'s `localize()` enforces its NonTopLevel invariant** (audit M2): the per-subtree error collapse is narrowed to the invariant-sanctioned `NonTopLevel → None`; any other variant now `debug_assert!`s (naming it) before returning `None` — release behavior is byte-identical (the caller already root-falls-back); a future relaxation of the step-2 type-check trips dev/test builds immediately instead of silently mis-pointing diagnostics.
- **Deleted the 25 orphaned `tests/vectors/v0_2` multisig goldens** (audit M13; resolves `orphaned-v0_2-md1-vectors-no-harness`): no test read them, and the v0.48.0 NUMS + v0.53.0 csi wire changes left them stale — a harness wired today would fail against them or pin outdated bytes. The single read golden (bip84) stays; `cli_self_check.rs`'s module doc corrected (it named a now-deleted fixture).

No CLI flag/surface change (no GUI `schema_mirror` or manual flag-coverage impact); no card-byte change. Resolves audit-2026-06-10 index lines `self-check-no-mk1-xpub-binding`, `inspect-repair-no-argv-advisory`, `localize-broad-error-collapse`, `orphaned-v0_2-multisig-goldens`, `self-check-header-claims-unread-vector` [obs]. Plan + reviews: `design/PLAN_minors_m1_m2_m3_m13.md`, `design/agent-reports/minors-m1-m2-m3-m13-*.md`.

## mnemonic-toolkit [0.53.1] — 2026-06-10

**SemVer-PATCH — `--phrase`/`--phrase-stdin`/`--ms1-stdin` are now secret-classified, and the secret-flag completeness gate is no longer circular (audit I3).**

### Fixed

- `xpub-search {path-of-xpub, passphrase-of-xpub, account-of-descriptor} --phrase` — a **raw BIP-39 master phrase** supplied inline — emitted `secret: false` in the `gui-schema` envelope, so GUI consumers rendered it cleartext with no paste-warn / run-confirm / exit-zeroize. It is now `secret: true`, along with its `--phrase-stdin` toggle and the previously-missed `--ms1-stdin` toggle (the `*-stdin`-sentinel-of-a-secret-flag convention). No card bytes, no flag names, and no runtime behavior change — the only wire delta is the `gui-schema` JSON `secret` metadata for these three names (same shape). The GUI mirrors flip at its next pin bump (companion staged on the open `mnemonic-gui` FOLLOWUPS entries; this is the toolkit-side half of audit I4).
- The test presented as the secret-flag drift gate (`secret_flag_enumeration_matches_authoritative_predicate`) compared the schema's `secret` bit against the same `secrets::flag_is_secret` predicate the emitter derives it from — a tautology that could never catch an allowlist omission (audit I3, `vacuous-secret-flag-gate`). It is kept (renamed `secret_bit_plumbing_matches_predicate`) as the emitter-plumbing check it always was, and three **non-circular** §7b gate cells now judge the live schema against test-local knowledge only: a secret-vocabulary name-net over value-bearing flags (with an audited-`EXEMPT` escape hatch, empty today), a structural `--X-stdin`-toggle-secrecy-equals-base-flag rule, and a frozen 14-name literal that set-equals the live `secret:true` surface. Written TDD-first: all three were red on exactly the three names above before the classification fix.
- `lint_argv_secret_flags` axis-1 (transitive on the same predicate) gains the three `xpub-search --phrase` routes with **discriminating** evidence needles (`pub phrase_stdin` / `fn phrase_stdin` — bare `phrase_stdin` is a suffix of the files' existing `passphrase_stdin` anchors and would prove nothing), and its module-doc boundary statement is rewritten: the predicate's completeness is now gated; the honestly-stated residual is novel-vocabulary names outside the net.

Resolves audit-2026-06-10 items `vacuous-secret-flag-gate` (I3) + `flag-is-secret-completeness-unguarded-by-design`. Plan + R0 reviews: `design/PLAN_secret_flag_gate_non_circular.md`, `design/agent-reports/secret-flag-gate-plan-r0-round{1,2}-review.md`.

## mnemonic-toolkit [0.53.0] — 2026-06-10

**SemVer-MINOR — multisig mk1 `chunk_set_id` is now slot-unique (audit I10) + the engraving display matches it (n1-vs-nge2).**

### Fixed

- Multisig mk1 `chunk_set_id` (csi) was derived per cosigner from the xpub fingerprint. Two cosigners reusing **one xpub at different origin paths** (admitted by the BIP-388 distinctness gate — it rejects only when both xpub AND path match) shared a fingerprint → shared csi → `verify-bundle` merged their chunks into one reassembly group → `ChunkedHeaderMalformed` → spurious `result: mismatch` on two individually-valid cards. The csi is now `derive_mk1_chunk_set_id(policy_stub) ^ slot_index` — distinct per cosigner (XOR is injective) and immune to fingerprint collision. Cosigner count is ≤16, so the slot index only XORs into the low nibble (5th hex char): the **leading 16 bits — the bundle-binding prefix shared with md1 — are preserved**, so cosigner cards still bind to one bundle while grouping correctly. Single-sig (n=1) csi is `^0` ⇒ **byte-identical** to before (no wire change). The csi is never recomputed at verify time, so previously-emitted cards keep verifying.
- The engraving-card display showed the policy-stub-derived id for every slot, which did not match the per-cosigner csi actually emitted for multisig. The display (both `mk1_card_id` and `ms1_card_id`) now uses the same slot-aware derivation, so the id stamped on each cosigner's plate matches its real csi (resolves `n1-vs-nge2-csi-derivation-inconsistency`).

**Wire change:** emitted multisig mk1 bytes change (the csi); single-sig bundles are unchanged. No CLI flag/surface change. The `bundle --json` mk1 values change (same JSON shape). Resolves audit-2026-06-10 items `mk1-chunk-set-id-fingerprint-grouping-assumes-distinct-fps` (I10) + `n1-vs-nge2-csi-derivation-inconsistency`. (`anti-collision-16bit-invariant-false` — md1↔mk1 cross-card agreement — remains open.)

## mnemonic-toolkit [0.52.0] — 2026-06-09

**SemVer-MINOR — `build-descriptor --allow <rule>`: a per-variant, reviewed opt-out of the funds-safety sanity gate (never silent).**

- **`--allow {malleable, mixed-timelock, repeated-keys, resource-limit, sigless-branch}`** (repeatable; kebab names align 1:1 with the diagnostic kinds, so a refusal teaches the exact token: `…; rerun with --allow mixed-timelock after review`). miniscript's 6th opt-out, `raw_pkh`, is not exposed (unreachable from the builder's node grammar). Mechanism: the gate's `sanity_check()` becomes `ext_check(&ExtParams)` — behavior-identical with no `--allow`.
- **Never silent.** Every rule that actually FIRES is named in an unmissable stderr warning (all output modes, `--json` included); a requested-but-unused allowance gets a `did not fire` note; the `--json` success envelope gains `allowed_rules_fired` (only when non-empty — default output byte-unchanged).
- **Cost preview deterministically skipped on a sanity-overridden emit** (`--json` `"cost": null`; human view prints `cost preview unavailable for a sanity-overridden descriptor`) — its taproot comparison would re-run the waived rules. `compare-cost` itself stays strict.
- **Composes with `--spec` AND `--archetype`** (the same-key "degrading threshold" the presets deliberately refuse is now buildable after review: `--allow repeated-keys`). `--emit-spec` records NO allowance — replaying an emitted spec without `--allow` correctly refuses.
- **Tests.** 12 new integration cells + 5 gate unit cells + a CliAllow/DiagnosticKind drift self-test; full suite + clippy (0); full manual lint green. Resolves `descriptor-builder-allow-extparams-reviewed-optout`. Audit trail: `design/SPEC_descriptor_builder_allow.md` + `design/agent-reports/descriptor-builder-allow-{r0-r1,r0-r2,impl-r1}-review.md`.

## mnemonic-toolkit [0.51.0] — 2026-06-09

**SemVer-MINOR — descriptor-builder archetype presets (Release B): `mnemonic build-descriptor --archetype` builds 5 curated vault shapes without hand-authoring JSON.**

> The five sections from [0.48.0] through [0.51.0] were backfilled on 2026-06-09 (the per-release CHANGELOG ritual silently lapsed after [0.47.4]; resolved by FOLLOWUP `changelog-md-release-ritual-lapsed-since-v0-47-4` — these entries are sourced from the tag annotations, release commits, and the resolved `design/FOLLOWUPS.md` entries; a tag-time CI guard `changelog-check.yml` now prevents recurrence).

- **5 archetype presets** over the frozen v0.50.0 `PolicyNode` IR: `--archetype {decaying-multisig, hashlock-gated, kofn-recovery, simple-timelocked-inheritance, tiered-recovery}` + a generic parameter vocabulary (`--key`, `--threshold`, `--recovery-key`, `--recovery-threshold`, `--final-key`, `--older`, `--recovery-older`, `--after`, `--hash` — all `requires = archetype`) + `--emit-spec` (print the lowered, gate-validated node-tree spec JSON for review; feed it back via `--spec`). Every preset flows through the SAME validation gate as `--spec`; the producer layer checks only applicability/arity/decay-ordering.
- **Kind-aware diagnostic flag provenance.** In preset mode, gate diagnostics carry the responsible CLI flag (`flag` field in `--json`, `(from --key)` suffix in human output); producer param errors use the new `param` diagnostic kind with the `node_path: "params"` sentinel. Spec-mode `--json` output is byte-identical (the `flag` field is skip-serialized when absent; pinned by a literal golden).
- **`--spec-schema` gains an `archetypes` section** — per-preset parameter field-specs (`flag`, `kind`, `required`, `repeatable`, `min`), generated from the registry (the contract a GUI archetype-forms wizard consumes). Both schema versions stay 1 (additive).
- **Producers are byte-pinned to the Release-A fixture canon**: IR AST equality vs the fixture JSON, preset CLI goldens vs the same `.descriptor`/`.bip388` files, and per-archetype mutated-param non-vacuity cells.
- **GUI lockstep:** 11 new clap flag names await the GUI pin bump — FOLLOWUP `gui-build-descriptor-presets-pending-pin-bump` (filed in both repos).
- **Tests.** Full toolkit suite + clippy `--all-targets` (0) + full manual lint (incl. cspell) green pre-push. Resolves the Release-B scope of `descriptor-builder-engine`. Audit trail: `design/SPEC_descriptor_builder_presets.md` + `design/agent-reports/descriptor-builder-presets-{r0-r1,r0-r2,phase-1-r1,phase-1-r2,phase-2-r1}-review.md`.

## mnemonic-toolkit [0.50.0] — 2026-06-09

**SemVer-MINOR — descriptor-builder engine (Release A): new top-level `mnemonic build-descriptor` turns a versioned JSON policy-tree spec into a validated `wsh(M)` descriptor.**

- **New subcommand `build-descriptor`** (`--spec <FILE|->`): a versioned JSON `PolicyNode` fragment-tree IR (`schema_version: 1`, externally-tagged nodes, `deny_unknown_fields`), rendered to `wsh(M)` and run through a **4-step funds-safety validation gate** — schema field-validate → type-check → `sanity_check` (the funds-footgun rules: sigless branch / malleable / resource limits / repeated keys / mixed timelocks) → build-time complexity cap (the always-previewable envelope).
- **Emits a reviewable bundle**: canonical descriptor (+ BIP-380 checksum), BIP-388 wallet-policy JSON, embedded compare-cost preview, and **node-addressed diagnostics** (`node_path` into the authored tree) on refusal (exit 2). `--format {descriptor,bip388}` bare artifacts; `--json` structured envelope.
- **`--spec-schema`** dumps the machine-readable node-tree grammar (the versioned contract GUI/preset producers consume).
- **Watch-only-out**: a secret key (xprv/WIF) in any node is refused without ever echoing it.
- **The 5 archetype shapes ship as hand-authored acceptance fixtures** (the Release-B presets' canon), incl. `tiered-recovery` replacing the brainstorm's same-key `degrading-threshold` (which trips `RepeatedPubkeys` by design).
- **Tests.** Full toolkit suite + clippy (0); golden-pinned descriptor + bip388 per fixture; bip388 round-trips through `export-wallet --descriptor`. Resolves the Release-A scope of `descriptor-builder-engine`. Audit trail: `design/SPEC_descriptor_builder_engine.md` (R0 7 rounds) + `design/agent-reports/descriptor-builder-*`.

## mnemonic-toolkit [0.49.1] — 2026-06-09

**SemVer-PATCH — `restore --md1` reconstructs taproot NUMS multisig descriptors (`tr-multi-a` + `tr-sortedmulti-a`), closing a refusal open since v0.44.0.**

- **Taproot multisig restore.** `mnemonic restore --md1` now reconstructs watch-only restore documents for `tr-multi-a` and `tr-sortedmulti-a` policies (BIP-341 NUMS internal key + multisig leaf) instead of refusing. Golden receive-address tests pin the reconstruction.
- **Routes around md-codec** for both the descriptor build and address derivation (md-codec's pinned rust-miniscript lacks `Terminal::SortedMultiA`; the toolkit's own pinned miniscript rev has it) — new address derivation on the descriptor STRING rather than re-entering md-codec.
- **Tests.** Full suite + clippy (0); reconstruction goldens replace the former refusal cells. Resolves `restore-multisig-taproot-reconstruction`. Audit trail: `design/SPEC_restore_multisig_taproot.md` (v2) + `design/agent-reports/restore-multisig-taproot-*`.

## mnemonic-toolkit [0.49.0] — 2026-06-08

**SemVer-MINOR — `export-wallet --descriptor` / `bundle --descriptor` accept a full BIP-388 wallet-policy JSON (auto-detected, expanded to a concrete descriptor).**

- **BIP-388 policy intake.** A leading-`{` value on `--descriptor` is auto-detected as a BIP-388 wallet policy (`description_template` + `keys_info`) and expanded to the concrete descriptor via the shared `wallet_import::pipeline::expand_bip388_policy` (extracted from the xpub-search intake, which now delegates).
- **Closes the round-trip with `--format bip388`**: byte-stable for `description_template` + `keys_info` (`name` is lossy — the emitter hardcodes its own label; tracked by the open FOLLOWUP `bip388-policy-roundtrip-wallet-name-not-honored`).
- **Ordering invariant**: the policy-shape probe runs FIRST at both intake sites (a raw policy would otherwise trip the `@N`/key-regex probes).
- **Tests.** 14 new (6 unit + 8 integration); full suite + clippy (0). Resolves `bip388-wallet-policy-to-descriptor-expansion-not-surfaced`. Audit trail: `design/SPEC_bip388_policy_descriptor_expansion.md` + `design/agent-reports/bip388-policy-descriptor-expansion-*`.

## mnemonic-toolkit [0.48.0] — 2026-06-08

**SemVer-MINOR (wire-content change) — bundled taproot-multisig `md1` emits the BIP-341 NUMS internal key (`is_nums: true`) instead of cosigner `@0`.**

- **NUMS internal key.** The bundled `tr-multi-a` template's taproot internal key flips from cosigner `@0` to the BIP-341 NUMS H-point (`50929b74…803ac0`), removing the unintended `@0` key-path spend; the multisig leaf is unchanged.
- **Whole-bundle wire change**: the policy-id stub seeds BOTH cards, so `md1` AND `mk1` both shift for affected bundles (hence MINOR, not PATCH). The shipped manual already documented NUMS — the code was non-conformant with its own docs.
- **Scope honesty**: this changes the EMIT side only; taproot-multisig `restore` remained refused until [0.49.1].
- **Tests.** Characterization test pins the NUMS emission (8 stale orphaned golden vectors deleted); full suite + clippy (0). Resolves `toolkit-trmultia-nums-internal-key`. Audit trail: `design/SPEC_trmultia_nums_internal_key.md` + `design/agent-reports/trmultia-nums-r0-round{1,2}-review.md`.

## mnemonic-toolkit [0.47.4] — 2026-06-06

**SemVer-PATCH — `bundle --self-check` now validates the per-slot `ms1` emission (was `md1` + `mk1` only).**

- **Closed self-check gap.** `bundle --self-check` re-parses a freshly-emitted bundle to confirm internal consistency. It validated `md1` (wallet-policy reassemble) + `mk1` (per-cosigner decode + policy-id-stub linkage) but **never inspected `ms1`** — so a silent regression in the per-slot `ms1` emission rule (most dangerously the **@0-only reversion**: `ms1[0]` populated, `ms1[1+]` wrongly `""` for a full-mode multisig) passed `--self-check` undetected. `self_check_bundle` now also validates `ms1`: for each slot, (a) **emptiness parity** — `ms1[i]` is non-empty iff the slot is entropy-bearing (catches the @0-only reversion and watch-only false-populate); (b) for entropy-bearing slots, the emitted `ms1` must **decode** (`ms_codec::decode`) AND **round-trip** to the slot's source entropy.
- **Correctness detail.** The entropy-bearing oracle is `resolved_slots[i].entropy.is_some()` — the exact predicate that drives emission — **not** the supplied `--slot` subkey. This matters: an `import-wallet --ms1` envelope replayed via `bundle --import-json` carries a populated `ms1` with no `--slot` arg, and a `--slot @N.wif=` slot is secret-bearing yet emits an empty `ms1` (ms-codec ENTR needs BIP-39 entropy, not raw WIF bytes); both shapes self-check correctly under the source-entropy oracle. The entropy round-trip is free (the source entropy is already materialized during synthesis — no secret is re-read).
- **No CLI-surface change** — `--self-check` is unchanged; this strengthens what it validates internally (`self_check_bundle` gains an internal `entropy_bearing` parameter, threaded from all four call sites). No GUI `schema_mirror`, no manual mirror, no sibling-codec change. Regression guards: `self_check_detects_at0_only_ms1_regression`, `self_check_detects_wrong_entropy_ms1`, `self_check_passes_watch_only_all_empty_ms1` (bundle.rs), `bundle_wif_slot_self_check_passes`, `bundle_import_json_seeded_ms1_self_check_passes`.
- **Tests.** Full toolkit suite (`--no-fail-fast`, 0 failed; all pre-existing `--self-check` tests pass unchanged) + clippy `--all-targets` (0). Resolves `self-check-ms1-iteration-audit`. Audit trail: `design/SPEC_self_check_ms1_iteration.md` + `design/agent-reports/self-check-ms1-iteration-r0-round{1,2}-review.md` + `…-phase-2-review.md`.

## mnemonic-toolkit [0.47.3] — 2026-06-06

**SemVer-PATCH — `--timestamp` defaults to `0` (rescan from genesis) consistently across every Bitcoin Core `importdescriptors` emitter.**

- **Default change.** The Bitcoin Core `importdescriptors` `timestamp` rescan-anchor now defaults to `0` (rescan from genesis — so the imported wallet discovers an existing key's full transaction history), the correct default for a recovery/watch-only-import workflow. Previously `export-wallet` defaulted to `now` (watch-going-forward, skipping the historical rescan) while `nostr --import` already defaulted to `0` — an inconsistency. Now uniform across all three emitters:
  - `export-wallet --timestamp` default `now` → `0` (`export_wallet.rs`). Explicit `--timestamp now` still works for users who want watch-forward.
  - `restore --format <X>` (single-sig) and `restore --md1 --format <X>` (multisig) — the two hardcoded `TimestampArg::Now` sites → `Unix(0)`. `restore` has no `--timestamp` flag; `0` is the right fixed value for a recovery.
  - `nostr --import` was already `0` (unchanged).
- **Observable effect.** A default `export-wallet`/`restore` Bitcoin Core payload now emits `"timestamp": 0` (a JSON number) instead of `"timestamp": "now"` (a string). No flag/value/subcommand change — `--timestamp` keeps its name and free-string parser; this is a default-*value* change only, hence PATCH (pre-1.0: not breaking; explicit values unchanged).
- **No GUI `schema_mirror` change** (the default *value* of a free-string flag is not a flag-NAME or value-enum). **Cross-repo note (NOT gated by `schema_mirror`):** the GUI's hand-maintained schema declares `export-wallet --timestamp` `default_value: Some("now")`, and its D33 default-suppression would silently drop an explicit `--timestamp now` selection once the GUI bumps its toolkit pin to ≥v0.47.3. Tracked by FOLLOWUP `gui-timestamp-default-value-drift-v0.47.3` (a two-release arc — the GUI fix lands at its next pin-bump).
- **Tests.** Full toolkit suite (`--no-fail-fast`, 0 failed) + clippy `--all-targets` (0) + `make -C docs/manual audit` GREEN (`verify-examples` 20/20; recipe-1/recipe-5 transcripts regenerated to `timestamp: 0`). Resolves `export-wallet-timestamp-default-zero` + `timestamp-zero-default-docs-sweep`. Audit trail: `design/SPEC_timestamp_default_zero.md` + `design/agent-reports/timestamp-default-zero-r0-round{1,2}-review.md` + `…-phase-2-review.md`.

## mnemonic-toolkit [0.47.2] — 2026-06-06

**SemVer-PATCH — three independent footgun/accuracy fixes: repair/inspect mutex-doc correction, `import-wallet` argv-secret advisory, honest `convert electrum→address` redirect.**

- **Docs (slug 1 — `manual-repair-flag-mutex-inaccuracy`).** The manual's `repair` and `inspect` reference rows wrongly described `--ms1`/`--mk1`/`--md1` as "mutually exclusive". Ground truth: both subcommands define `ArgGroup::new("kind").args(["ms1","mk1","md1"]).required(false).multiple(true)` with **no `conflicts_with_all`** — mixed-HRP invocations (`mnemonic inspect ms1… mk1… md1…`) are valid (one HRP per card; per D35). Reworded the 4 affected rows to "may be combined with … (one HRP per card; per D35)" and rewrote the two synopses (`repair`, `inspect`) from the false brace-pipe `{--ms1 | --mk1 | --md1}` to the curated independently-optional form. No code change.
- **Secret-hygiene (slug 2 — `import-wallet-ms1-argv-advisory-gap`).** `mnemonic import-wallet` now emits the standard `secret_in_argv_warning` stderr advisory when a secret-bearing `--ms1 <inline>` **or** `--slot @N.phrase=<inline>` value is passed on argv (skipped for the `@env:VAR` indirection and the `""` watch-only sentinel). Matches the existing `bundle`/`verify-bundle` precedent; the advisory reads the RAW args before the env-resolution rebind so an `@env:` value never trips it. Additive stderr only — no exit-code or stdout change.
- **Honesty (slug 3 — `electrum-phrase-address-refusal-honest-wording`).** `mnemonic convert --from electrum-phrase --to address` previously fell through to the generic one-way refusal. It now returns a dedicated `ConvertRefusal` that explains Electrum uses its own PBKDF2 salt + non-BIP-44 derivation (not a `convert` edge) and **redirects to the real path**: `mnemonic addresses --from electrum-phrase=<seed> --address-type <p2pkh|p2wpkh>`. Same exit code (2); no other electrum edge changed.
- **No CLI-surface change** — no flag/value/subcommand added → no GUI `schema_mirror`, no sibling-codec change, no new error variant. Slug 1 fires the `manual` CI workflow (manual file changed); slugs 2/3 are code+test only.
- **Tests.** Full toolkit suite (`--no-fail-fast`, 0 failed) + clippy `--all-targets` (0) + `make -C docs/manual audit` GREEN. Resolves `manual-repair-flag-mutex-inaccuracy`, `import-wallet-ms1-argv-advisory-gap`, `electrum-phrase-address-refusal-honest-wording`. Audit trail: `design/SPEC_quick_wins_v0_47_2.md` + `design/agent-reports/quick-wins-v0_47_2-r0-round{1,2,3}-review.md` + `…-phase-2-review.md`.

## mnemonic-toolkit [0.47.1] — 2026-06-06

**SemVer-PATCH — internal dedup: `synthesize_unified` delegates its card-emission to `synthesize_descriptor`.**

- **Refactor.** `synthesize_unified` (template/multisig bundle path) and `synthesize_descriptor` (descriptor-mode path) shared a byte-identical back-half: from a `Descriptor` + cosigner list, compute the policy-id stub, emit per-slot `ms1` (Entr/Mnem by wire language), per-cosigner `mk1` (`Single`/`Multi`), and `md1`. Since `type CosignerKeyInfo = ResolvedSlot`, `synthesize_unified`'s `slots: &[ResolvedSlot]` IS `synthesize_descriptor`'s `cosigners: &[CosignerKeyInfo]` — so `synthesize_unified` now keeps only its front-half (validation + descriptor construction) and **delegates** the emission via `synthesize_descriptor(&descriptor, slots, …)`. Net **−69 LOC** in `synthesize.rs`; no user-visible behavior change.
- **Behavior-preserving (guarded).** A new byte-exact characterization cell `synthesize_unified_multisig_distinct_cosigners_byte_exact` (2-of-2 `wsh-sortedmulti`, two distinct cosigners → distinct per-cosigner `mk1` csi) was captured from the pre-refactor binary and pins the n>1 `Multi` branch's full Bundle byte-shape (the path that previously had no byte-exact golden — the multisig self-check golden was dropped in v0.4.2). It + the frozen 16-cell single-sig golden + `make verify-examples` (20 transcripts) all stay GREEN through the delegation.
- **No CLI-surface change** — no flag/value/subcommand → no GUI `schema_mirror`, no manual mirror, no sibling-codec change; no new error variant. The ~9 `synthesize_*` call sites are signature-stable.
- **Tests.** Full toolkit suite + clippy `--all-targets` + `make -C docs/manual verify-examples` GREEN. Resolves `synthesize-descriptor-deduplicate-with-unified`. Audit trail: `design/SPEC_synthesize_descriptor_dedup.md` + `design/agent-reports/synthesize-descriptor-dedup-r0-round{1,2}-review.md` + `…-phase-2-review.md`.

## mnemonic-toolkit [0.47.0] — 2026-06-06

**SemVer-MINOR — `mnemonic addresses --from electrum-phrase=`: derive an Electrum native seed's own addresses.**

- **`addresses --from electrum-phrase=<seed>`.** An Electrum native seed (the 2.x `seed_version` system, not BIP-39) was previously refused by `addresses`; it now derives Electrum's **own** addresses. Derivation (validated against Electrum's own `test_wallet_vertical.py` end-to-end vectors @ commit `e1099925`): `PBKDF2-HMAC-SHA512(normalize_text(seed), "electrum"+normalize_text(passphrase), 2048)` → BIP-32 root → **standard** seed `m/<chain>/<index>` (P2PKH), **segwit** seed `m/0'/<chain>/<index>` (P2WPKH). `--chain receive/change` = chain `0`/`1`, as in Electrum.
- **Surface.** The script type + derivation are **fixed by the Electrum seed version**, so `--address-type` must match it (`p2pkh` standard / `p2wpkh` segwit — mismatch refused), `--account` does not apply (refused if non-zero), and `--language` is ignored (the seed is stretched from the raw phrase string, not decoded via a wordlist). **2FA seeds (versions 101/102) are refused.** `--passphrase`/`--passphrase-stdin` is the Electrum seed-extension passphrase. Watch-only out (no `xprv`/`zprv` reaches stdout/`--json`; the 64-byte seed + intermediate xprivs are `Zeroizing`).
- **Crypto note.** Electrum's `normalize_text` is implemented as a dedicated `electrum::normalize_text_electrum` matching upstream byte-for-byte: NFKD → lower → drop chars whose **canonical combining class** is non-zero (NOT the Mark general category — marks like U+034F/U+0489 have ccc=0 and are KEPT, per Python's `unicodedata.combining`) → collapse whitespace → strip CJK-internal whitespace. The `UNICODE_HORROR` passphrase vector pins this.
- **No new clap flag/value-enum** — un-refuses an existing runtime-parsed `--from` node → no GUI `schema_mirror` change. Manual mirror: `docs/manual/src/40-cli-reference/` `addresses` chapter updated. No new error variant (reuses `BadInput`).
- **Tests.** New `cli_addresses_electrum.rs` (3 Electrum e2e vectors incl. the unicode-passphrase normalization torture vector + `--address-type` mismatch / `--account` / 2FA refusals + watch-only + `--json` source-label). Full toolkit suite + clippy `--all-targets` + `make -C docs/manual audit` GREEN. Resolves `electrum-native-seed-address-derivation`. Audit trail: `design/SPEC_addresses_electrum_native_derivation.md` + `design/agent-reports/addresses-electrum-native-derivation-r0-round1-review.md` + `…-phase-2-round{1,2}-review.md`.

## mnemonic-toolkit [0.46.3] — 2026-06-06

**SemVer-PATCH — internal dedup: consolidate the import-parser origin extraction into shared `pipeline` helpers (+ import parsers now tolerate `h`-form hardened origins).**

- **Refactor.** Origin extraction from a concrete descriptor body (`[fp/path]xpub` → typed `(Xpub, Fingerprint, DerivationPath)`) was copy-pasted across the import parsers: `fn build_slot_fields` in 6 (`bsms`, `bitcoin_core`, `sparrow`, `coldcard`, `specter`, `electrum`), `fn extract_origin_components` + apostrophe-only `fn origin_capture_regex` in 4 (`bsms`, `bitcoin_core`, `specter`, `sparrow`), plus inline regex copies in `coldcard`/`electrum`. These collapse into two `pub(crate)` helpers in `wallet_import/pipeline.rs` — `extract_origin_components(body, format_name)` (keyed on the canonical h-form-widened `key_regex`) + `finalize_slot_fields(...)` — consumed by all 6 parsers, which keep only their thin per-parser signature + selection/out-of-range message (bitcoin_core's `entry_idx`, electrum's `…in synthesized descriptor`). Net **−195 LOC**.
- **`h`-form hardened-origin tolerance (the one behavior delta).** Routing all parsers through `key_regex` (`…(?:/\d+(?:'|h)?)+…`) means an `import-wallet` descriptor whose origins use `h`-form hardened markers (`84h/0h/0h`, as some Bitcoin Core / Sparrow exports emit) now parses where the apostrophe-only copies refused it (`"no origin annotations in descriptor"`). This is a **superset** — every apostrophe-form input matches identically — so no existing input changes behavior. Resolves `import-parser-hform-origin-tolerance`.
- **Convergent internal messages.** A few defensive internal error messages converge to the shared wording (the per-slot `xpub decode for slot {n}` context flattens to `xpub decode`; coldcard's `(internal bug)` empty-result folds into `no origin annotations in descriptor`). These branches are proven unreachable (the xpub was already decoded upstream by `concrete_keys_to_placeholders`); no test/manual pins them.
- **No CLI-surface change** — no flag/value/subcommand added → no GUI `schema_mirror`, no manual CLI-reference mirror; no new error variant (reuses `ImportWalletParse`); no sibling-codec change.
- **Tests.** New `cli_import_wallet_bitcoin_core.rs` cell `core_single_descriptor_hform_hardened_path_accepted` (an `h`-form wpkh origin imports like the apostrophe form). Full toolkit suite + clippy `--all-targets` + `make -C docs/manual verify-examples` (20 foreign-format transcripts) GREEN. Resolves `descriptor-origin-extraction-dedup` + `import-parser-hform-origin-tolerance`. Audit trail: `design/SPEC_descriptor_origin_extraction_dedup.md` + `design/agent-reports/descriptor-origin-extraction-dedup-r0-round{1,2,3}-review.md` + `…-phase-2-review.md`.

## mnemonic-toolkit [0.46.2] — 2026-06-05

**SemVer-PATCH — `gui-schema`: project restore's `--from required_unless_present="md1"` conditional rule.**

- **`gui-schema` `conditional_rules` for `restore`.** `restore`'s `--from` is `required_unless_present = "md1"` (single-sig restore needs a seed source; multisig-cosigner restore via `--md1` makes it optional), but the `gui-schema` `conditional_rules` projection had no `restore` arm → emitted `conditional_rules: []`. The downstream GUI (`mnemonic-gui`) therefore modeled the at-least-one constraint as a **hand-authored, ungated** rule. This release adds a `restore_conditional_rules()` builder emitting one rule — `when: not(flag_present "--md1") → effect: {flag: "--from", visibility: required}` — mirroring `bundle`'s existing `--template` Required-unless precedent. Now `mnemonic gui-schema` projects it, so the GUI's `conditional::restore` becomes **drift-gatable** by `mnemonic-gui`'s `gui_schema_conditional_drift` on its next toolkit-pin bump.
- **No CLI-surface change.** `conditional_rules` is gated by the GUI's drift test, NOT `schema_mirror` (flag-NAME parity only). No clap flag/value/subcommand change → no `schema_mirror`, no manual mirror, no sibling-codec change. No `gui-schema` JSON **version** bump (the rule reuses the existing `not`/`flag_present`/`required` grammar — stays v5).
- **Cross-repo follow-on.** This is the **toolkit half** of FOLLOWUP `gui-schema-restore-required-unless-md1-projection`. The GUI consumption half (bump pin → the drift gate enforces the rule + add `("restore", 1)` to `SUBCOMMAND_FLOORS`) is a downstream `mnemonic-gui` cycle; the FOLLOWUP stays open until that GUI tag exists.
- **Tests.** New `cli_gui_schema_conditional_rules.rs` cell `restore_from_required_unless_md1_uses_not_flag_present_predicate` (exactly 1 rule; `not`/`flag_present`/`--md1` → `--from`/`required`). `dispatcher_arm_count_matches_pinned_constant` bumped `6 → 7` (a new dispatcher arm by design trips that guard). Full toolkit suite + clippy `--all-targets` GREEN. Audit trail: `design/SPEC_gui_schema_restore_conditional_projection.md` + `design/agent-reports/gui-schema-restore-conditional-projection-r0-round{1,2}-review.md` + `…-phase-2-review.md`.

## mnemonic-toolkit [0.46.1] — 2026-06-05

**SemVer-PATCH — internal dedup: consolidate the 4-way wallet-format emit dispatch into one `emit_payload` helper.**

- **Refactor.** The `WalletFormatEmitter` dispatch (`collect_missing`-first → `ExportWalletMissingFields` refuse → 11-arm `emit`, including the 6-variant coldcard-multisig `--template` sub-match) was copy-pasted in **four** places: `export-wallet`'s `run` + `run_from_import_json`, and `restore`'s single-sig + multisig `build_*_import_payload`. (The tracking FOLLOWUP said "3-way"; recon found a 4th uncited copy in `run_from_import_json`.) These collapse into one `pub(crate) fn emit_payload(&EmitInputs, CliExportFormat) -> Result<String, _>` homed in `cmd/export_wallet.rs`; each call site keeps only its own per-site `EmitInputs` construction. Net **−124 LOC**, no behavior change for the three byte-identical copies.
- **One user-visible wording change.** Single-sig `restore --format coldcard-multisig` (on a `bip44`/`bip49`/`bip84`/`bip86` template) now routes through the shared helper, so its refusal is the **unified** message — *"--format coldcard-multisig requires a multisig --template (wsh-sortedmulti, …). For Coldcard singlesig export use --format coldcard with bip44/bip49/bip84."* — instead of the old restore-specific *"requires a multisig wallet; restore is single-sig — use --format coldcard"*. **Exit code unchanged (1, `BadInput`).** Note: the unified message's first clause names multisig `--template` values that single-sig restore rejects upfront; its **second** clause carries the correct `--format coldcard` pointer for the restore caller.
- **No CLI-surface change** — no flag/value-enum/subcommand added or removed → no GUI `schema_mirror`, manual mirror, or sibling-codec change; no new error variant (reuses `ExportWalletMissingFields` + `BadInput`).
- **Tests.** New `cli_restore.rs` cell `restore_format_coldcard_multisig_single_sig_refused_exit_1` pins the unified single-sig refusal (exit 1 + `stderr.contains("requires a multisig --template")`). Full toolkit suite (877+) + clippy `--all-targets` GREEN. Resolves `design/FOLLOWUPS.md` entry `restore-emit-dispatch-3way-dedup` (corrected to 4-way). Audit trail: `design/SPEC_restore_emit_dispatch_dedup.md` + `design/agent-reports/restore-emit-dispatch-dedup-r0-round{1,2}-review.md` + `…-phase-2-review.md`.

## mnemonic-toolkit [0.46.0] — 2026-06-05

**SemVer-MINOR — `xpub-search passphrase-of-xpub --passphrase-candidates-file`: scan a candidate-passphrase list against a target xpub.**

- **`--passphrase-candidates-file <PATH>`.** `passphrase-of-xpub` previously verified a single passphrase; it now also accepts a **text file with one candidate passphrase per line** (no argv exposure). The command loops the existing verify oracle (`derive_master_seed` → `match_xpub_against_paths` over BIP-44/49/84/86 + `--add-path`) over the candidates, **aborts on the first match**, and reports the matching **file line number** to stdout (the matching passphrase appears only under `--json`, machine-consumption opt-in). Blank lines are skipped; each non-blank line is a literal candidate (only the trailing newline/CR is stripped — a passphrase is an exact byte string). No match ⇒ exit 4 (`XpubSearchPassphraseCandidatesExhausted`) with the count of candidates tried; an all-blank/empty file ⇒ exit 4 with a tailored "no candidates" note. The candidate line is wrapped `Zeroizing<String>` per the secret-memory-hygiene discipline (a `ZEROIZE_ROWS` entry was added). The candidate file is classified as a path (non-secret) flag, mirroring `--decrypt-password-file`/`--secret-file`.
- **Mutex.** The passphrase source is now a 3-way clap `ArgGroup` (exactly one of `--passphrase` / `--passphrase-stdin` / `--passphrase-candidates-file` required; omitting all three is a clap error, exit 64). `--passphrase-candidates-file` reads a FILE, so the seed may still arrive via `--phrase-stdin`/`--ms1-stdin` (no stdin contention).
- **Scope.** This is bounded **verification of a candidate list you supply** — keyspace *generation* (wordlists, masks, typo models) remains btcrecover's job. The top-level `--help` passphrase-recovery footer (+ its manual mirror) is refined from a flat "`mnemonic` cannot brute-force" to distinguish the candidate-list scan from keyspace generation, keeping the btcrecover pointer. Resolves `design/FOLLOWUPS.md` entry `xpub-search-passphrase-bruteforce` (file-only scope; stdin/generated-wordlist modes intentionally not built).
- **Wire-shape.** `PassphraseOfXpubResult::Match`/`NoMatch` gain optional `matched_candidate_line` / `matched_passphrase` / `candidates_tried` (`skip_serializing_if` — the single-`--passphrase` envelope is byte-unchanged). Not gated by GUI `schema_mirror` (flag-name parity only); the new `--passphrase-candidates-file` flag IS gated (`xpub-search-passphrase-of-xpub` is GUI-schema'd) → the paired GUI update is pin-blocked, tracked as FOLLOWUP `gui-xpub-search-passphrase-candidates-file-flag-pending-pin-bump`.
- **Tests.** New `crates/mnemonic-toolkit/tests/cli_xpub_search_passphrase_candidates.rs` (10 cells: hit→line; `--json` line+passphrase; miss→exit 4 + candidates_tried; abort-on-first; blank-skip; exact-bytes trailing-space; 3-way mutex exit 64; required-one-of exit 64; empty file; secret-hygiene — default stdout doesn't echo the passphrase). Full audit trail: `design/SPEC_xpub_search_passphrase_candidates_file.md` + `design/agent-reports/xpub-search-passphrase-candidates-r0-r{1,2,3,4}-review.md` (R0 GREEN after 4 rounds).

## mnemonic-toolkit [0.45.0] — 2026-06-05

**SemVer-MINOR — `mnemonic restore --md1 --format <X>`: importable multisig wallet payloads.**

- **`restore --md1 --format <X>`.** Multisig-cosigner restore (v0.44.0) emitted only the descriptor doc and refused `--format`. It now emits an importable wallet-software payload — the **same payload class** as `mnemonic export-wallet --template <multisig> --format <X>` — built from the reconstructed cosigner keys + threshold: a multisig `EmitInputs` (`threshold: Some(k)` with `threshold_user_supplied: true` since the md1's `k` is authoritative — load-bearing: the Sparrow emitter refuses a multisig template otherwise; `taproot_internal_key: None`; synthesized `<template>-<account>` wallet name) run through the shared `export-wallet` emitter dispatch. No `--template` is needed (threshold + script type come from the `md1`).
- **Supported:** `bitcoin-core`, `bip388`, `coldcard`, `coldcard-multisig`, `jade`, `sparrow`, `electrum`, `bsms`, `descriptor` (9). **Refused** (identically to `export-wallet`): `specter` (`ExportWalletMissingFields` exit 2 — needs a wallet name restore does not take) and `green` (exit 1 — no multisig support).
- **Output shape (mirrors single-sig restore).** Without `--json`, the payload pipes from stdout while the verification doc (descriptor + cosigner table) moves to stderr; `--output <FILE>` writes the payload to the file. With `--json`, the envelope gains an `import_payload` field. The payload is computed **after** the cross-check mismatch hard-gate, so a `✗ MISMATCH` still exits 4 (`RestoreMismatch`) before any payload is emitted unless `--allow-mismatch`.
- **Watch-only-out preserved.** No `xprv` / WIF / seed reaches stdout, stderr, or `--json` (test-enforced).
- **Scope / refusals (exit 2).** `wsh` / `sh(wsh)` only; a **taproot** multisig `md1` is still refused at the `Tag::Tr` gate before reconstruction (FOLLOWUP `restore-multisig-taproot-reconstruction`).
- **No CLI-surface change** — `--format` is a pre-existing `restore` flag (a previously-refused multisig invocation now emits). No new flag/value-enum → no GUI `schema_mirror` or sibling change. Resolves `design/FOLLOWUPS.md` entry `restore-multisig-format-payloads`.
- **Tests.** New `crates/mnemonic-toolkit/tests/cli_restore_multisig_format.rs` (10 cells: per-format threshold-token fidelity across all 9 emit formats, 3-real-fingerprint containment for the `[fp/…]`-embedding formats, `--format descriptor` exact-equality vs the `--json` descriptor, specter/green refusals, watch-only-out, `--json` `import_payload`, mismatch-precedence exit 4, `sh(wsh)`, `--output`).
- **Code note.** `build_multisig_import_payload`'s emitter dispatch is a byte-identical 3rd copy of `export_wallet.rs`'s (also duplicated in single-sig restore); consolidation tracked as FOLLOWUP `restore-emit-dispatch-3way-dedup`.

## mnemonic-toolkit [0.44.0] — 2026-06-05

**SemVer-MINOR — `mnemonic restore` multisig-cosigner: reconstruct a watch-only multisig descriptor from the shared `md1`.**

- **`mnemonic restore --md1 <card> […]`.** Completes the multisig half deferred from the v0.43.0 single-sig `restore` cycle. A toolkit-emitted multisig `md1` is a **wallet-policy** card — it carries every cosigner's concrete public key — so the concrete watch-only multisig descriptor is reconstructed from the **card alone** (`md_codec::to_miniscript_descriptor` → `template_from_descriptor` → `build_descriptor_string`, emitting the canonical multipath `<0;1>/*` form + BIP-380 checksum, matching single-sig restore's output shape). Emits the descriptor + first receive address(es) + a per-cosigner table. Covers `wsh(sortedmulti|multi)` and `sh(wsh(…))`.
- **Optional, per-position cross-check.** `--from <seed>` proves which cosigner is yours (position inferred by matching the derived key — the 65-byte `chain-code‖pubkey` form, never a normalization-sensitive `Xpub ==`); `--cosigner @N=<mk1|xpub>` asserts another cosigner's key. **Only the positions you actually supply are marked verified** — every other stays `from md1 (not independently verified)`, and the overall verdict is `PARTIAL` until all positions are cross-checked (never present an unchecked key as verified). A supplied key that does NOT match the md1 slot is a hard error (`✗ MISMATCH`, exit 4, `RestoreMismatch`) unless `--allow-mismatch`; with no cross-check input the output carries the `UNVERIFIED` banner.
- **Watch-only-out.** No `xprv` / WIF / seed reaches stdout, stderr, or `--json` (test-enforced) — the reconstructed cosigner xpubs and the own-seed cross-check derivation are public-only.
- **Scope / refusals (exit 2).** A **taproot** multisig `md1` (`tr(sortedmulti_a, …)`) is refused — `md_codec::to_miniscript_descriptor` cannot reconstruct it (rust-miniscript v13 has no `SortedMultiA` fragment); deferred to FOLLOWUP `restore-multisig-taproot-reconstruction`. A template-only `md1` (no concrete keys — never emitted by the toolkit) is refused. `--format`, `--template`, and `--expect-xpub` are single-sig only.
- **CLI.** New `--md1` (repeating; wallet-policy md1 chunks) and `--cosigner @N=<mk1|xpub>` (repeating) flags, both watch-only / non-secret. `--from` becomes optional in multisig mode (`required_unless_present = "md1"`).
- **Docs.** New `### Multisig-cosigner restore` section in `40-cli-reference/41-mnemonic.md` (modes, flags, cross-check policy, worked examples, scope). Resolves `design/FOLLOWUPS.md` entry `restore-multisig-cosigner-scope`.
- **Lockstep.** The paired `mnemonic-gui` `RESTORE_FLAGS` update (add `--md1`/`--cosigner`, flip `--from required:false`) is pin-blocked (the GUI schema cannot lead its toolkit binary pin) and tracked as FOLLOWUP `gui-restore-multisig-flags-pending-pin-bump`; the manual flag-coverage lint gates the toolkit side.

## mnemonic-toolkit [0.43.1] — 2026-06-04

**SemVer-PATCH — `verify-bundle --descriptor` now binds a raw-`entropy` cosigner.**

- **`verify-bundle --descriptor <BIP-388 @N template> --slot @N.entropy=<hex>`.** The descriptor-mode binding loop in `verify_bundle.rs` had arms for `phrase`/`seedqr`, `xpub`, and `ms1`, but **no `entropy` arm** — a raw-entropy cosigner fell through to the catch-all and errored (`DescriptorReparseFailed`, exit 4) with `--slot @N subkey set ["entropy"] not supported in descriptor verify-bundle path`. This was a pure asymmetry: the `bundle` descriptor path and the `verify-bundle` **template** path already derive a cosigner from raw entropy; only `verify-bundle` + `--descriptor` + `entropy` was unbindable. The new arm mirrors the `bundle` Entropy arm's behavior — hex-decode → derive at the descriptor-annotated path (routing through the shared `derive_slot::derive_bip32_from_entropy_at_path`, the step-for-step equivalent of the bundle arm's inline derivation; honoring `--passphrase`/`--language`/`--network`), re-emit the cosigner card, and compare. Output is byte-identical to the `bundle` path, so a bundle built from `@N.entropy=` round-trips to `result: ok`.
- **No CLI-surface change.** `entropy` is a pre-existing `--slot` subkey; this is a behavior-only fix (a previously-erroring invocation now succeeds). No new flag/option/value-enum → no GUI `schema_mirror` or manual-mirror change. Resolves `design/FOLLOWUPS.md` entry `verify-bundle-descriptor-entropy-slot-gap`.
- **Tests.** New `crates/mnemonic-toolkit/tests/cli_verify_bundle_entropy_slot.rs` (5 round-trip + mismatch tests: 16/32-byte lengths, a non-`@0` slot in a 3-cosigner `wsh(andor(…))` descriptor, a `--passphrase` round-trip, and a passphrase-mismatch that proves the derived key is input-dependent and the comparison is live).

## mnemonic-toolkit [0.43.0] — 2026-06-04

**SemVer-MINOR — `mnemonic restore` (single-sig): secret seed + passphrase → watch-only restore document.**

- **`mnemonic restore --from <ms1|phrase|entropy|seedqr>=<value>`.** Takes secret seed material (with an optional BIP-39 `--passphrase` / `--passphrase-stdin`, `@env:`-/stdin-channel for both the seed value and the passphrase) and emits a **watch-only restore document**: a verification block leading with the master fingerprint (the passphrase-correctness oracle) + first receive address(es), then the concrete single-sig descriptor(s) for BIP-44/49/84/86 (or one type via `--template`). Read-only / watch-only-out — emits xpub / fingerprint / addresses / concrete descriptor only and **NEVER** any private material (`xprv` / WIF); does not sign. `--account`, `--network`, `--language`, `--count` mirror the existing derivation surface. A non-seed `--from` node (`xpub` / `xprv` / `wif` / …) is refused (exit 1).
- **Fingerprint hard-gate.** `--expect-fingerprint <hex>` / `--expect-xpub <xpub>` (the latter requires `--template`) cross-check the derived material against a reference; a mismatch is a **hard error (exit 4, `RestoreMismatch`)** that prints derived-vs-expected under a `✗ MISMATCH` banner and emits **no descriptors**. `--allow-mismatch` overrides (descriptors emitted under a loud `✗ MISMATCH (overridden)` banner, exit 0). With no reference at all, output carries a loud `UNVERIFIED` stderr banner pointing at the fingerprint.
- **`--format` / `--json` / `--output`.** `--format <export-format>` emits an importable wallet-software payload via the `export-wallet` emitter dispatch and **requires a single `--template`** (one-descriptor-in/one-out; `--format` with the all-4 default → exit 2); when set, the payload goes to stdout and the verification block to stderr so the payload pipes cleanly. `--json` emits a single structured object (`master_fingerprint`, `passphrase_applied`, `network`, `verification`, `wallets[]`, optional `import_payload`); seed material is never echoed (redacted by construction). `--output <FILE>` redirects the stdout content (banners/advisory stay on stderr).
- **Multisig deferred.** A multisig `--template` (`wsh-sortedmulti`, …) is refused — multisig-cosigner restore (own seed + the shared md1 + other cosigners' mk1s → a concrete multisig descriptor + cosigner cross-check) is a planned additive follow-on (v0.44.0), tracked as `design/FOLLOWUPS.md` entry `restore-multisig-cosigner-scope`.
- **Docs.** New `## mnemonic restore` section in `40-cli-reference/41-mnemonic.md` (Synopsis / Flags / verification policy / worked examples), `mnemonic restore` added to `docs/manual/tests/cli-subcommands.list` (flag-coverage lint), and a headline "I have my seed + passphrase — restore my wallet on a PC" recipe in `30-workflows/35-recovery-paths.md` (verify the fingerprint before funding; hard-gate with `--expect-fingerprint`).
- **Lockstep.** `restore` is a new top-level subcommand → paired `mnemonic-gui` v0.24.0 adds the `restore` `SubcommandSchema` to `src/schema/mnemonic.rs` and bumps its toolkit pin to v0.43.0 (post-tag mini-cycle; the `schema_mirror` drift guard goes green on the GUI pin bump).

## mnemonic-toolkit [0.42.0] — 2026-06-03

**SemVer-MINOR — `export-wallet --format descriptor`: bare canonical descriptor on stdout.**

- **`export-wallet … --format descriptor`.** Emits exactly one line — the canonical descriptor with its BIP-380 checksum, `<descriptor>#<checksum>` — and nothing else (no JSON, no wallet-file wrapper). Works for single-sig and multisig and across every input path: `--template` + `--slot @N.xpub=` (add `--slot @N.fingerprint=<mfp>` for a real key origin, else the origin is the all-zeros `[00000000/…]` placeholder), bare-concrete `--descriptor` passthrough, and `--from-import-json` (lossless — only the BIP-380 checksum is recomputed). This completes the concrete-descriptor in/out at the toolkit layer: A1 (v0.38.1) added the **IN** door (`bundle` / `verify-bundle --descriptor` accept a bare concrete descriptor → cards); this adds the **OUT** door.
- **Taproot via `--from-import-json` remains refused** (the import path does not surface a taproot internal-key designation); emit `tr(...)` through the direct `--descriptor 'tr(...)' --format descriptor` passthrough instead.
- **Docs.** New `Concrete descriptor ↔ bundle round-trip` recipe in the manual (`30-workflows/37-wallet-export.md`) — md1-keyless framing, the IN/OUT loop, taproot-passthrough caveat, and the `--format descriptor` (raw, any policy) vs `--format green` (Green text, single-sig only) distinction; `descriptor` added to the `export-wallet --format` value list in `40-cli-reference/41-mnemonic.md`.
- **Lockstep.** `descriptor` is a new `--format` value-enum entry → paired `mnemonic-gui` v0.23.0 adds it to `src/schema/mnemonic.rs` `EXPORT_FORMATS` and bumps its toolkit pin to v0.42.0 (the `schema_mirror` drift guard goes green on the GUI pin bump).

## mnemonic-toolkit [0.41.0] — 2026-06-03

**SemVer-MINOR — `bundle` / `verify-bundle --slot @N.ms1=` (raw BIP-93 codex32 secret as a slot input). Language-preserving; refuse-on-`--language`-conflict.**

- **`--slot @N.ms1=<codex32-secret>` on `bundle` and `verify-bundle`.** A raw BIP-93 codex32 secret string is now a first-class secret slot subkey (`ms1`), decoded inline and routed through the existing entropy materialization path — byte-identical to `@N.entropy=<hex>` for an `entr`-kind ms1 of the same entropy. The slot is **language-preserving**: a `mnem`-kind ms1 carries its BIP-39 wordlist language on the wire, so the toolkit derives with that language and emits a `mnem` ms1 card that round-trips the language without re-specifying `--language` (load-bearing for `verify-bundle`, which compares whole card strings). Works in template mode (`resolve_slots`), `bundle --descriptor` mode, and `verify-bundle --descriptor` mode.
- **Refuse-on-`--language`-conflict.** Supplying `--language` whose wordlist disagrees with a `mnem` ms1 slot's wire language is refused with `SlotInputViolation` `kind:"language-conflict"` (exit 2); omit `--language` or set it to match the wire language.
- **K-of-N share rejection.** A codex32 K-of-N *share* (not a single-string secret) supplied as `@N.ms1=` is rejected with a friendly pointer to `mnemonic ms-shares combine` — reassemble the secret first, then slot the combined `ms1`.
- **Fix-the-class: canonical-descriptor secret+path slots now refuse with exit-2 `SlotInputViolation`.** Widening the `bundle` canonical-descriptor gate to `phrase | seedqr | ms1` means a `[Ms1, Path]` or `[Seedqr, Path]` slot set against a canonical descriptor now refuses with exit-2 `SlotInputViolation{kind:"conflict"}` (a secret cosigner cannot also carry an explicit `@N.path=` against a canonical descriptor). This normalizes the prior `[Seedqr, Path]` behavior, which had fallen through the binding loop to an exit-1 `BadInput`.
- **Lockstep.** Documented the `ms1` subkey in the `bundle` + `verify-bundle` `--slot` clap help and manual (`41-mnemonic.md`); added `Ms1` to `mnemonic-gui`'s `SlotSubkey` picker + the `SECRET_SLOT_SUBKEYS` secret-redaction snapshot (paired GUI PR; the compile-time drift guard goes green once the GUI bumps its toolkit pin to ≥ v0.41.0). No clap flag-NAME was added (the subkey is a free-form `--slot` value, not a clap value-enum), so no `schema_mirror` change. Files FOLLOWUP `verify-bundle-descriptor-entropy-slot-gap` (raw `@N.entropy=` cosigner in `verify-bundle --descriptor` mode still falls to `DescriptorReparseFailed`; out of scope).

## mnemonic-toolkit [0.40.0] — 2026-06-03

**SemVer-MINOR — `mnemonic ms-shares split|combine` (BIP-93 codex32 K-of-N ms1 shares). Re-pins ms-codec 0.4.0 / ms-cli v0.7.0.**

- **`mnemonic ms-shares split` / `mnemonic ms-shares combine`.** Split an `ms1` secret (an `entr` or `mnem` BIP-39 entropy payload) into N codex32 K-of-N shares such that any K recombine the original secret, using codex32's native threshold(k)+index Shamir mechanism (BIP-93). `split` takes a phrase / hex / ms1 source plus `-k`/`--threshold` and `-n`/`--shares`; `combine` takes K-or-more shares and emits the recovered secret per `--to` (`phrase` / `entropy` / `ms1`). A `mnem` source preserves its BIP-39 wordlist language across the share set; an English-phrase or raw-entropy source splits as `entr`. The recovered `ms1` composes with the rest of the toolkit (`combine --to ms1` → `bundle --slot @0.ms1=…`).
- **Share-aware inspect / decode / repair.** `mnemonic inspect` and `mnemonic convert --from ms1=…` now recognize a K-of-N share (threshold 2..9) and emit a friendly "this is a K-of-N share; use `mnemonic ms-shares combine`" message (mapped exit code) instead of an "unhandled ms_codec::Error variant" — the share routes to `IsShareNotSingleString` rather than the v0.1 single-string `ThresholdNotZero` reject.
- **Output-class.** `ms-shares split` and `combine` are private-key-material producers (the share SET is secret-equivalent) and emit the `warning: stdout carries private key material …` advisory; entropy intermediates are zeroized.
- **Re-pin ms-codec 0.3.0 → 0.4.0 and ms-cli v0.6.0 → v0.7.0** (the K-of-N codecs). ms-codec 0.4.0 adds `Threshold`, `encode_shares`, `combine_shares`, and threshold-routing in `decode`; ms-cli v0.7.0 adds `ms split` / `ms combine` and share-aware `ms inspect`. (Published-crate adoption + TEMP path-override removal is the ship step of this cycle.)
- **Manual + GUI lockstep.** Documented `mnemonic ms-shares` (`41-mnemonic.md`) and `ms split` / `ms combine` / share-aware `ms inspect` (`43-ms.md`); added the flag-coverage list entries; mirrored the `ms-shares` subcommand (+ `split`/`combine` modes, flags, and the `combine --to` value-enum) into `mnemonic-gui/src/schema/`. The new `--json` wire-shapes (`ms split`/`combine`/`inspect`-share, `mnemonic ms-shares`) are NOT schema_mirror-gated (flag-name + dropdown parity only) — tracked as FOLLOWUP `ms-kofn-json-wire-shape-ungated`.

## mnemonic-toolkit [0.39.0] — 2026-06-02

**SemVer-MINOR — `mnem` ms1 consume + emit (per-card wordlist language). Resolves the §6.3 non-English-seed footgun on the toolkit side.**

- **ms-codec 0.3.0 adoption (via TEMP path-override; crates.io publish + override removal is Phase 3 Step 9 of this cycle).** ms-codec 0.3.0 adds a `mnem` ms1 payload kind that stores the BIP-39 wordlist language on the wire alongside the entropy. The toolkit consumes `mnem`-kind ms1 cards by reading the WIRE language for decoding (not `--language`), and emits a `mnem`-kind ms1 card whenever a source is non-English (so the language is faithfully preserved in the bundle). English sources and raw-entropy sources continue to emit the classic `entr`-kind ms1 (byte-identical with prior versions).
- **Non-English seed footgun (§6.3) fixed for `bundle` and `convert --to ms1`.** A BIP-39 seed is `PBKDF2` over the language-specific mnemonic string; the same entropy decoded with the wrong language yields a different wallet. Prior to this release a non-English phrase engraved as an `entr` ms1 silently dropped the language on the wire, requiring the user to remember and supply `--language` at decode time. With `mnem` encoding the language is preserved on the wire; `ms decode` and `mnemonic inspect --ms1` recover it automatically.
- **Manual prose.** Documented `mnem` auto-routing in `43-ms.md` (encode auto-routing, decode `mnem` behavior, inspect `kind`/`language` fields) and added concise faithful-preserve notes in `41-mnemonic.md` (bundle + convert sections, inspect `ms1` bullet).
- **Wire-shape note.** A toolkit-emitted `mnem` ms1 is a new on-wire string shape in `bundle --json`/`export-wallet` envelopes. Any downstream consumer that re-decodes it needs ms-codec ≥0.3.0 or it will reject the string as `UnexpectedStringLength`. Tracked as FOLLOWUP `toolkit-mnem-ms1-wire-shape-downstream-consumers`.

## mnemonic-toolkit [0.38.4] — 2026-06-01

**SemVer-PATCH — mk-cli v0.7.0 re-pin (SLIP-0132 prefix acceptance) + manual prose. No toolkit source or behavior change.**

- **mk-cli re-pin v0.6.1 → v0.7.0** at the three install/CI sites (`scripts/install.sh`, `.github/workflows/manual.yml`, `.github/workflows/quickstart.yml`). mk-cli v0.7.0 lets `mk encode --xpub` and `mk verify --xpub` accept SLIP-0132 extended-public-key prefixes (mainnet `ypub`/`zpub`/`Ypub`/`Zpub`, testnet `upub`/`vpub`/`Upub`/`Vpub`), normalizing them to canonical `xpub`/`tpub` (key material unchanged; a stderr note names the original prefix) and refusing a prefix↔origin-path script-type mismatch with an actionable message.
- **Manual prose.** Documented SLIP-0132 acceptance in `docs/manual/src/40-cli-reference/44-mk-cli.md` under `mk encode` (with a cross-reference from `mk verify`). No clap flag was added, so no GUI schema-mirror change and no toolkit-CLI flag-coverage change.

## mnemonic-toolkit [0.38.3] — 2026-06-01

**SemVer-PATCH — output-class advisory Phase 2 lockstep + md-codec 0.35 (Tier-0 repair).**

- **Phase 2 lockstep.** mk-cli **v0.6.1** + md-cli **v0.6.2** complete the constellation-wide always-emit one-line stderr output-class advisory: mk output-producing subcommands (`decode`, `derive`, `address`, `inspect`) now emit `note: stdout is watch-only …`; md output-producing subcommands (`decode`, `encode`) emit `note: stdout is a keyless descriptor template …`, `md address` emits `note: stdout is watch-only …`. Inert subcommands emit nothing. Closes FOLLOWUP `output-type-stderr-advisory-sibling-sweep-mk-md`. Files new FOLLOWUP `output-class-advisory-byte-parity-test-tautological` (low-severity hardening, cross-repo).
- **md-codec 0.35 (Tier-0).** Toolkit re-pin from md-codec 0.34 → 0.35 (non-chunked descriptor decode for repair path, landed in Phase 0 of this cycle).

## mnemonic-toolkit [0.38.2] — 2026-05-31

### Added
- **Output-type stderr advisory (cycle B, Phase 1).** Every output-producing command now prints a one-line stderr classification of what it wrote to stdout: `warning: stdout carries private key material (can spend) …` / `note: stdout is watch-only …` / `note: stdout is a keyless descriptor template …`. Inert commands (verify-bundle, decode-address, verify-message, compare-cost, gui-schema, xpub-search) emit nothing, so absence of a line reliably means inert output. This subsumes the prior D9 "secret material on stdout" advisory (re-worded), drops the TTY gate on `final-word`/`seed-xor`/`slip39` (the redirected case is the one that matters), and makes spend-capability visible everywhere. stderr-only; no flag added. Phase 2 (`mk`/`md`) is FOLLOWUP `output-type-stderr-advisory-sibling-sweep-mk-md`.

## mnemonic-toolkit [0.38.1] — 2026-05-31

**SemVer-PATCH — `bundle` / `verify-bundle --descriptor` now accept a bare concrete descriptor in addition to the `@N` template form; `export-wallet --descriptor` rejects a keyless `@N` template with a helpful pointer.**

- **`bundle --descriptor` and `verify-bundle --descriptor`** now accept either a BIP-388 `@N` template (keys supplied via `--slot`) **or a bare concrete descriptor** with inline `[fp/path]xpub` keys (watch-only output); both apostrophe and `h`-form hardened paths are accepted.
- **`export-wallet --descriptor`** accepts a concrete descriptor (with or without key origins); a keyless `@N` template is rejected with a pointer to `--template … --slot …` or `--from-import-json`.
- **Internal: shared descriptor-form classifier + `descriptor_concrete_to_resolved_slots` helper** — single source of truth for both the form check and concrete-descriptor key extraction, shared across `bundle`, `verify-bundle`, and `export-wallet`.

## mnemonic-toolkit [0.38.0] — 2026-05-31

**SemVer-MINOR — new `mnemonic addresses` subcommand: batch watch-only address derivation.** The watch-only complement to `export-wallet --range`, mirroring `mk address`. Read-only public derivation — no private keys on stdout, no signing.

- **`mnemonic addresses --from <SOURCE> --address-type <T>`** lists a wallet's receive/change addresses. `--from` accepts `xpub=` (an account xpub, derived directly) or the seed sources `phrase=`/`entropy=`/`seedqr=` (where `--address-type` selects the BIP-44/49/84/86 account path via `--account`, derived through `derive_bip32_from_entropy`). `--count N` (default 10) / `--range A,B` with a BIP-32 normal-index ceiling guard (out-of-range → exit 1, never a `from_normal_idx` panic); `--chain receive|change|both`; `--network` with a main-vs-test kind-agreement guard (xpub source); `--passphrase`/`--passphrase-stdin`, `@env:VAR`, and stdin `-` for secret values (single-stdin guard); `--json` (`{schema_version, source, address_type, network, account?, addresses:[{chain,index,address}]}`). The argv-leak advisory fires for inline secret values; the v0.37.11 non-English advisory does NOT fire (addresses are derived keys — the language is already applied).
- **Internal: de-duplicated the address renderer + network-inference helpers.** `convert::build_address_from_xpub` / `xpub_search::render_address` (byte-identical copies) and the two `network_from_xpub` copies (convert + `address_of_xpub`) are lifted into a single `crate::address_render` module; `convert`, `xpub-search`, and `addresses` share them. Behavior-preserving (full suite unchanged).
- No `--json` wire-shape change to existing subcommands; new clap flags trigger the GUI schema-mirror + manual lockstep (paired).

## mnemonic-toolkit [0.37.11] — 2026-05-30

**SemVer-PATCH — stderr advisory when a non-English BIP-39 seed is encoded into a language-agnostic form (ms1 / raw entropy / SLIP-39 shares). Path A of the `mnem` wordlist-language footgun (the wire-format hint stays filed as `mnemonic-secret mnem-wordlist-language-hint-on-wire`, a v0.2 arc).**

- **The footgun.** A BIP-39 seed is `PBKDF2` over the *language-specific* mnemonic string, so the same entropy in a different wordlist language yields a different seed and a different wallet. ms1 cards, raw entropy, and SLIP-39 shares carry only the entropy — not the wordlist language. Recovering a non-English backup with English-defaulted third-party software silently derives the wrong wallet. `ms decode` already loud-warns on the decode side; this closes the **encode** side in the toolkit (the only place that sees `--language`).
- **New `language::non_english_seed_advisory(lang, form)` helper** — single source of truth, returns `None` for English, else a stderr advisory naming the language (kebab `human_name()`) and the emitted form. Emitted at four chokepoints:
  - `bundle` — once per secret-bearing invocation (single `emit_unified` chokepoint, not per-cosigner), `form = "an ms1 card"`.
  - `convert` — when a language-agnostic target (`entropy` **or** `ms1`) is present (key-deriving targets like `xprv`/`xpub`/`wif` bake the language in, and the `phrase` target re-encodes in `--language` and keeps it, so they do not fire). Emitted after `compute_outputs` succeeds, so a malformed phrase errors out before advising.
  - `slip39 split` — always (shares are language-agnostic), `form = "SLIP-39 shares"`.
  - `slip39 combine --to entropy` — `--to phrase` re-encodes in `--language` and keeps it, so only the entropy shape fires.
- Advisory is **stderr-only** — no new clap flag, no `--json` wire-shape change → no GUI schema-mirror, no manual lockstep. Matches the bip48 "bless + warn" stderr-advisory precedent. `--json` stdout stays byte-identical.

## mnemonic-toolkit [0.37.10] — 2026-05-30

**SemVer-PATCH — adopt `mk-codec 0.4.0` (no-path / depth-0 support) and make every mk1 card's `origin_path` round-trip the xpub it carries. Resolves FOLLOWUP `mk1-card-origin-path-vs-xpub-depth-consistency` (companion `mnemonic-key mk1-no-path-depth0-support`); flips `mk1-depth-child-compensating-check-watch`.**

- **Re-pin `mk-codec` `0.3.1 → 0.4.0`.** 0.4.0 added an encode-time guard (`XpubOriginPathMismatch`) that rejects any mk1 `KeyCard` whose `xpub.depth`/`child_number` disagree with `origin_path` (compact-73 drops + reconstructs them from the path). Adopting it surfaced — and this release fixes — a latent correctness bug: on 0.3.1 the toolkit silently emitted **wrong-metadata mk1 cards** (chain_code/public_key correct so addresses derive, but the reconstructed BIP-32 depth/child wrong) across ~40 import/export/multisig flows.
- **New `synthesize::mk1_origin_path(xpub, descriptor_path)` helper** at all 8 `KeyCard::new` sites derives the mk1 card's origin path from the xpub's own depth/child (truncate / extend / pad). Foreign multisig formats export an **account-level** xpub (`m/48'/0'/0'`, depth-3) with a **full-path** descriptor origin (`m/48'/0'/0'/2'`, depth-4); the mk1 card now carries the account-consistent path while md1's `path_decl` keeps the full descriptor origin independently. The depth-0 / no-path case (a WIF) round-trips as an empty path.
- **`verify-bundle` cross-checks redesigned** to compare the decoded mk1 `origin_path` against md1 on their shared **prefix** (a depth difference is legitimate account-truncation / leaf-extension; only a genuine prefix disagreement is flagged), keyed off the xpub's own depth. No false-positive on correct 3→4 / 4→3 / 4→4 bundles.
- **`bundle --import-json`** sources each cosigner's origin from the envelope's `bundle.origin_path[s]` metadata (the full descriptor origin) rather than the now-account-level mk1 card, so a re-imported origin matches the source descriptor.
- The `bundle --slot @N.wif=…` mk1 card — a write-only card on 0.3.1 (`decode` rejected the empty origin as `PathTooDeep(0)`) — now round-trips (regression test: bundle → `inspect`).
- The `synthesize_multisig_watch_only` SPEC §4.5 depth-reject loop is **removed** (superseded by the helper's consistent-by-construction cards). No CLI flag / subcommand / `--json` wire-shape change → no GUI schema-mirror, no manual lockstep.

## mnemonic-toolkit [0.37.9] — 2026-05-29

**SemVer-PATCH — internal refactor: delete the overloaded `ResolvedSlot.path_raw` field; derive the origin annotation on demand from the typed `fingerprint` + `path` via two new methods (`origin_path_bare()` / `bracketed_origin()`). Resolves FOLLOWUP `path-raw-bracketed-vs-bare-convention-unification`.**

- **Fixes the live cosmetic bug**: `bundle --import-json --json` no longer emits `multisig.cosigners[].origin_path` of shape `"m/[fp/path]"` (bracketed origin with a spurious `m/` prefix and a fingerprint already carried in the sibling `master_fingerprint` field). Cosigner `origin_path` is now bare `m/...`. The engraving-card per-slot origin line is likewise bare (`fp @ m/...`) instead of the bracketed, fingerprint-duplicated form.
- **Dissolves the v0.37.7 F5 band-aid** structurally: emitters (electrum / coldcard / sparrow / the descriptor key + bip388 keys_info) now derive origins from the typed path rather than reading an overloaded string, so the `export_wallet.rs` `path_raw` boundary normalization is gone.
- **Behavior-value changes (no key/flag/wire-key change; `--json` keys unchanged):**
  - `bundle --json` and `export-wallet` now render a CANONICAL origin path (`48h` → `48'`, `m/`-normalized) when a user supplies a non-canonical `--slot @N.path=` — the origin is derived from the typed `DerivationPath` (which folds `h`→`'`).
  - The `ResolvedSlot`-vector BIP-388 distinctness check now compares the typed `path` (converging with the descriptor-mode `check_key_vector_distinctness`); same-xpub cosigners whose paths differ only in `h`-vs-`'` notation now collide (exit 2), per the v0.5 §4.11.b typed-equality reversal.
  - The `ImportWalletSeedMismatch` error's `at path …` clause now prints the bare `m/...` origin instead of the bracketed `[fp/…]` form.
- Toolkit-internal (`ResolvedSlot` is binary-private); no clap surface change → no GUI schema-mirror or manual flag-coverage lockstep. GUI `--json` consumers self-update via the paired-PR rule. Two manual transcripts re-captured for the bare-origin rendering. Full suite 2482/0; reviews persisted to `design/agent-reports/path-raw-unification-*`.

## mnemonic-toolkit [0.37.8] — 2026-05-28

**SemVer-PATCH — universal source-name lift: `export-wallet --from-import-json` preserves the wallet name through every name-carrying format (sparrow / specter / jade / electrum / bitcoin-core / coldcard-multisig) instead of defaulting to the `imported-descriptor` placeholder. Resolves FOLLOWUP `sparrow-from-import-json-wallet-name-preservation` (broadened from sparrow-only to all 6 name-carrying formats per "fix the class, not the instance"). Toolkit-only; no GUI lockstep — no clap surface change.**

- **What.** When `--wallet-name` is absent on `--from-import-json`, the resolved name is now lifted from the envelope's per-format `*_source_metadata.<name-key>` (sparrow `label` / specter `label` / jade `coldcard_compat.name` / electrum `wallet_name` / bitcoin-core `source_metadata.wallet_name` / coldcard-multisig `name`). Explicit `--wallet-name` still wins; the lift only fires when the user supplied no override AND the envelope carries a liftable name. Pre-v0.37.8 the placeholder additionally tripped Specter's `MissingField::WalletName` refusal — that path is now dissolved on lifted names.
- **Wire-shape.** `ImportJsonEnvelope` gains 6 optional per-format `*_source_metadata: Option<serde_json::Value>` carry-fields with `#[serde(default)]` (back-compat: older envelopes deserialize unchanged). New `coldcard_multisig_source_metadata` emit-block in `cmd/import_wallet.rs` (was the only name-carrying format whose projection wasn't emitted). Accessor `ImportJsonEnvelope::resolved_wallet_name()` uses a `walk_str(&Value, &[&str])` path-walker for the jade nested case. Field rename `EmitInputs.wallet_name_was_user_supplied` → `wallet_name_is_non_default` covers both explicit and lifted (Specter check looks at this flag).
- **Tests.** 8 unit cells in `wallet_import/json_envelope.rs` (7 per-format positive + 1 walker negative); 8 integration cells in `tests/cli_export_wallet_universal_name_lift.rs` (6 per-format round-trips + 1 specter-target unblock + 1 explicit-override). Existing `p11c_refusal_matrix_specter_no_wallet_name` narrowed from `[bsms, coldcard-multisig]` to `[bsms]` (BSMS BIP-129 wire shape has no wallet-name field). Sparrow and coldcard-multisig chapter-45 transcripts re-captured (sparrow round-trip now byte-empty `diff`; coldcard-ms `Name:` preserved). Chapter-45 prose addendum updated. Full suite passes.
- Plan-doc R0→R1 GREEN (3C/4I/4M → 0C/0I); persisted at `design/agent-reports/sparrow-name-universal-lift-R{0,1}-review.md`. Spec: `design/SPEC_sparrow_name_universal_lift.md`.

## mnemonic-toolkit [0.37.7] — 2026-05-27

**SemVer-PATCH — F5 bug fix: `export-wallet --from-import-json` no longer corrupts the cosigner derivation path emitted to coldcard-multisig / jade / electrum / sparrow (toolkit-only; no GUI lockstep — no clap surface change). Ships with the new wallet-file cross-format convergence suite that found it.**

- **F5 fix.** `export-wallet --from-import-json --format {coldcard-multisig, jade, electrum, sparrow}` corrupted the cosigner derivation path: coldcard/jade wrote the `Derivation: m/0'/0'` placeholder; electrum wrote an invalid bracketed `derivation` field its own importer rejected. **Root cause:** `mk1_card_to_resolved_slot` (`wallet_import/json_envelope.rs:282`) populates `ResolvedSlot.path_raw` as a bracketed `[fp/path]` origin-annotation (overloaded convention), but the export emitters consume `path_raw` expecting a bare `m/...` derivation path (the `resolve_slots` convention). **Fix:** in `run_from_import_json` (`cmd/export_wallet.rs:629-640`), after `envelope_to_resolved_slots`, normalize each slot's `path_raw` to `format!("m/{}", s.path)` — boundary fix, scoped to export-wallet's from-import-json. Zero ripple to `bundle --import-json` (separate call site) or direct export (path_raw already bare). All four affected export emitters now write the correct shared derivation path. The underlying convention overload tracked at FOLLOWUP `path-raw-bracketed-vs-bare-convention-unification`.
- **Tests.** New `tests/cli_wallet_cross_format_convergence.rs` — the third metamorphic convergence suite (after `cli_cross_start_convergence` and `cli_convert_convergence`). 7 functions / 11 cells: C1 (single-sig 5-format convergence), C2/C3 (multisig sortedmulti + sh-wsh 7-format convergence), C4 (unsorted-multi order preservation on the order-preserving set + coldcard-coercion probe), C-neg (anti-vacuity), H_hop (5 cross-format hop pairs via `export-wallet --from-import-json`), H5 (single-sig electrum↔sparrow hop). Convergence asserted on the decoded key-material tuple (xpub multiset + `(xpub, fingerprint, origin_path)` triples + fingerprint set + threshold/N + md1 policy tag + multi/sortedmulti tag + network), explicitly EXCLUDING raw descriptor string (F1: bitcoin-core multipath split), ms1 sentinels, and format metadata. Construction is export-generate from in-test-derived keys (F2-safe; pattern from cross-start). Found F5 via H2 (sparrow→coldcard-multisig); H4 (specter→jade) was the secondary detection. Full suite 2457/0.
- Retroactive architect R0: `design/agent-reports/v0_37_7-f5-fix-review.md` (GREEN 0C/0I; folded M1 comment reframe + M2 electrum cosmetic-shape unification). Spec: `design/SPEC_wallet_cross_format_convergence_tests.md` (R0 RED 0C/2I → R1 GREEN 0C/0I, both persisted).

## mnemonic-toolkit [0.37.6] — 2026-05-27

**SemVer-PATCH — taproot-multisig + `--multisig-path-family bip48` now emits a non-standard-path stderr advisory (bless + warn; toolkit-only, no GUI lockstep — stderr only, no clap surface change).**

- **Bless + warn.** BIP-48 standardizes only script-type `1'` (sh-wsh) and `2'` (wsh). Taproot multisig templates (`tr-multi-a` / `tr-sortedmulti-a`) under `--multisig-path-family bip48` derive at `m/48'/<coin>'/<account>'/3'` — a toolkit convention, not part of BIP-48. The toolkit continues to **honor** the explicit flag (exit 0, cards emitted) but now emits a stderr advisory pointing to `--multisig-path-family bip87` (`m/87'/…`) for a standardized taproot multisig path. Resolves FOLLOWUP `multisig-tr-bip48-script-type-3-policy`.
- **Where.** Logic centralized in a pure, unit-tested helper `CliTemplate::bip48_nonstandard_script_type_warning(family)` (single source of truth); emitted on stderr by `bundle`, `export-wallet`, and `verify-bundle` (every command that derives the `3'` path). Silent for the `bip87` family, for wsh/sh-wsh (standardized `1'`/`2'`), and for single-sig. Exit codes unchanged.
- **Tests.** `tests/cli_tr_bip48_advisory.rs` (5 cells: bundle/export-wallet/verify-bundle fire for tr+bip48; bip87 + wsh+bip48 stay silent) + a `template.rs` unit test. Full suite 2443/0.

## mnemonic-toolkit [0.37.5] — 2026-05-26

**SemVer-PATCH — F4 bug fix: elided-origin descriptors emit canonical `PathDecl::Shared` for identical inferred paths (cross-start md1 convergence); completes the cross-start convergence + standalone-bijection test matrix (toolkit-only; no GUI lockstep — no clap surface change).**

- **F4 fix.** A multisig descriptor with **elided origins** (e.g. `wsh(andor(pkh(@0/<0;1>/*),...))`, no inline `[fp/path]`) bundled via `bundle --descriptor` emitted `PathDecl::Divergent([p,p,p])` for identical inferred paths, while the explicit-origin / wallet-import path emitted `PathDecl::Shared(p)` — byte-different md1 for the SAME wallet (cross-start non-convergence). The non-canonical default-path inference (`cmd/bundle.rs`) now collapses identical inferred per-`@N` paths to `Shared`, matching `parse_descriptor` (`all_paths_same`) and `synthesize_unified` (`all_same || n==1`); the symmetric verify-bundle inference (`cmd/verify_bundle.rs`) gets the same collapse. **No functional impact** — verify-bundle validates md1 via decode + wallet_policy + xpub-multiset, never the path_decl encoding; derived addresses and wallet semantics are identical. `Divergent` is retained for genuinely distinct per-cosigner paths (SPEC §4.2). Trigger is narrow: only elided-origin descriptor-mode bundles (canonical templates + inline-origin descriptors already emit `Shared`).
- **Tests.** Completes the Property A (cross-start convergence) + Property B (standalone bijection) matrix begun in v0.37.4: new `tests/cli_standalone_bijections.rs` (B1–B6: `xpub↔mk1` + reverse fp/path edges, multisig per-cosigner; `descriptor↔md1` canonical/non-canonical/multisig via `md_codec::chunk` round-trip) + cell A8 (non-canonical `wsh(andor)` descriptor ≡ BSMS wallet-file — the cell that drove the F4 fix). Full matrix 14/14; full suite 2437/0.
- Retroactive architect R0: `design/agent-reports/v0_37_5-f4-fix-review.md`. Resolves FOLLOWUP `cross-start-convergence-remaining-cells`.

## mnemonic-toolkit [0.37.4] — 2026-05-26

**SemVer-PATCH — bug fix: `--multisig-path-family bip48` now reaches seed/entropy-mode multisig derivation (toolkit-only; no GUI lockstep — pre-existing flag, no new clap surface).**

- **Fix.** In seed- or entropy-slot multisig template mode, `bundle` / `export-wallet` / `verify-bundle` derived cosigner keys at the BIP-87 fallback path `m/87'/<coin>'/<account>'` regardless of `--multisig-path-family`, while the JSON `multisig.path_family` field reported the requested family — an internal inconsistency, and the BIP-48 path `m/48'/<coin>'/<account>'/<script_type>'` was unreachable from a seed. Root cause: `resolve_slots` derived via `template.derivation_path()` (documented single-sig-only; returns the BIP-87 fallback for all multisig templates) and never received the family flag. The flag is now threaded into `resolve_slots` and applied at the phrase, entropy, and xpub-path-absent derivation sites via `MultisigPathFamily::default_origin_path(network, account, template.bip48_script_type())`. `derive_bip32_from_entropy` becomes a thin wrapper over the new `derive_bip32_from_entropy_at_path`; `derive_full_at_path` is its phrase-level sibling.
- **Backward compatibility.** For the default `bip87` family the computed path is byte-identical to the prior `template.derivation_path()` output, so every pre-fix default-family bundle is unchanged (full suite 2430/0). `mk1` and `md1` remain mutually consistent (both derive origins from the corrected `ResolvedSlot.path`).
- **Tests (Property A, partial).** New `tests/cli_cross_start_convergence.rs` — 7 cross-start convergence cells asserting that the same key entered as a seed, xpub, descriptor, or wallet file yields byte-identical `mk1`+`md1`: A1/A2/A4/A5 (single-sig), A6 (multisig bip87), A7 (multisig bip48 — the cell that surfaced this bug), A1-neg. Documented findings: F1 (bitcoin-core splits the `<0;1>` multipath, so wallet-file convergence is descriptor-shape-scoped), F2 (a mislabeled in-repo bip84 xpub fixture, sidestepped by in-test derivation). Cell A8 (non-canonical descriptor) + Property B bijections (`xpub↔mk1`, `descriptor↔md1`) deferred to FOLLOWUP `cross-start-convergence-remaining-cells`.
- Retroactive architect R0: `design/agent-reports/v0_37_4-f3-fix-review.md`. Design: `design/SPEC_cross_start_convergence_and_bijection_tests.md`.

## mnemonic-toolkit [0.37.3] — 2026-05-24

**SemVer-PATCH — three combined extensions to `mnemonic repair --max-indel` (toolkit-only; no GUI lockstep until GUI v0.21.3).** Resolves `m-format-indel-cross-region-split`, `m-format-indel-plus-substitution`, and `m-format-indel-hrpmismatch-suggestion-fallback`. `m-format-indel-erasure-decode-extend-to-8` remains open.

1. **Cross-region two-level search.** `recover_indel` restructured as a `prefix_restorations × data_variants` product search; `IndelRegion::CrossRegion` captures simultaneous prefix+data indels. The single-region producers are subsumed (byte-identical output at N=1); cross-region split becomes reachable within the existing `--max-indel` budget.

2. **`--max-subst <E>` candidate-list + verify-advisory (exit 4 on substitution-bearing).** New flag (0..=4, default 0) widens the oracle accept gate from `corrections ⊆ placeholders` to `|corrections \ placeholders| ≤ E`. `IndelCandidate.subst_count` counts residual substitutions. A candidate with `subst_count ≥ 1` is printed as a VERIFY-ME candidate (exit 4, not exit 5); `confident: bool` in `--json` is `true` iff all candidates have `subst_count == 0`. The BCH budget is shared: `placeholders + substitutions ≤ 4`. `--max-subst` without `--max-indel ≥ 1` is a no-op (stderr notice emitted). Non-breaking guarantees: a true indel recovery is always in the candidate set (widening the accept gate never drops correct candidates); exit 5 = trust / exit 4 = verify invariant is preserved across all combinations. **GUI `schema_mirror` lockstep: `mnemonic-gui v0.21.3` (add `--max-subst` to `REPAIR_FLAGS`) is the post-tag paired PR.**

3. **HrpMismatch suggestion-fallback.** Phase 4 of this cycle reversed the v0.37.1 opt-in tradeoff: when `--max-indel ≥ 1` engages and indel recovery fails for an input whose originating `repair_card` error was `HrpMismatch`, the original `HrpMismatch` error (with its "did you mean" suggestion) is now surfaced instead of the generic `IndelUnrecoverable` — so prefix-region recovery AND the helpful typo hint are preserved together. (The v0.37.1 CHANGELOG did not explicitly claim the old behavior; Phase 4 resolved it before tagging.)

R0→R2 GREEN + per-phase reviews GREEN.

## mnemonic-toolkit [0.37.2] — 2026-05-24

**SemVer-PATCH — md1 indel recovery un-refused in `mnemonic repair --max-indel`.** Closes `m-format-indel-md1-chunked`. Mirrors the v0.37.1 mk1 path onto md (toolkit-only — md-codec already exposes `bch::MD_REGULAR_CONST` and `chunk::reassemble`; shared codex32 generator). Re-acquires `MD_REGULAR_TARGET` from `md_codec::bch::MD_REGULAR_CONST`, extends `target_residue(Md1, Regular) => Some(...)`, and mirrors `Mk1IndelOracle`/`mk1_chunk_solve` as `Md1IndelOracle`/`md1_chunk_solve` with the cross-chunk oracle = `md_codec::chunk::reassemble` (which does NOT self-correct, avoiding the unguarded-correction concern). Per-chunk length-restore + BCH-solve locates the corrupted chunk; reassembly validates the recovered set. No new CLI flag or subcommand → NO GUI `schema_mirror` lockstep (flag-name `--max-indel` already existed; md1 just stops being refused). Manual updated: `--max-indel` options table row and prose subsection now cover ms1/mk1/md1. Per-phase review GREEN.

## mnemonic-toolkit [0.37.1] — 2026-05-24

**SemVer-PATCH — `mnemonic repair --max-indel <N>` (indel recovery for ms1/mk1).** Closes `m-format-incorrect-length-recovery`. Additive flag on the existing `repair` subcommand (default 0 = off; no behavior change at default). When a character was inserted (too long) or dropped (too short) during hand-copy, pass `--max-indel <N>` (1–4) to search for the indel and recover the correct string. Engine (`indel.rs`): two producers — P1 prefix-region restore (prefix drop/insert surfaces as `HrpMismatch`, which now joins the indel trigger set) and P2 data-part enumerate-and-validate (delete each position for too-long; BCH-solve the placeholder symbol for too-short). Per-kind oracles: ms1 delegates to `ms_codec::decode`; mk1 attempts per-chunk recovery and then full-card reassembly via `mk_codec`. Budget j≤4 (the BCH t=4 error-correction ceiling). Exit codes: 0 (no indel needed), 5 (unique recovery), 4 (ambiguous — multiple equally-valid candidates, print all), 2 (unrecoverable within budget). `ms1` candidates are secret material (the D9 stderr advisory fires via `any_ms1` flag). `md1` (chunked) is refused with a clear error — FOLLOWUP `m-format-indel-md1-chunked`. `--json` emits an indel envelope (`{schema_version, status: "unique"|"ambiguous", candidates: [{recovered, indel_count, region, direction}]}`). The `resolve_groups` HRP-relax-when-indel-active flag allows a prefix-region indel (e.g. `s10…` = `ms1…` minus `m`) to reach the indel engine instead of being rejected by the typed-flag pre-gate. Files 4 new FOLLOWUPs: (a) `m-format-indel-erasure-decode-extend-to-8`, (b) `m-format-indel-md1-chunked`, (c) `m-format-indel-cross-region-split`, (d) `m-format-indel-plus-substitution`. GUI `schema_mirror` lockstep: `mnemonic-gui v0.21.2` (add `max-indel` to `REPAIR_FLAGS` with `FlagKind::Number { min: 0, max: NumberMax::Static(4) }`) is the paired post-tag PR. Brainstorm-spec opus R0→R1 GREEN; per-phase reviews GREEN.

## mnemonic-toolkit [0.37.0] — 2026-05-24

**SemVer-MINOR — `export-wallet --from-import-json` auto-derives the `--template`.** Closes `export-wallet-from-import-json-template-format-reemit`. Before this, `export-wallet --from-import-json <envelope>` re-emitted only to descriptor-passthrough formats (bitcoin-core/bip388/bsms) + specter; the four template-requiring file-import formats (sparrow/coldcard/jade/electrum) **refused** ("descriptor passthrough is not supported"), and `--from-import-json conflicts_with --template` so the user couldn't supply one — making 5 documented chapter-45 round-trip recipes impossible (the v0.28.1-class breakage, recurring). Now `run_from_import_json` derives the template from the envelope's **parsed descriptor** (new `template_from_descriptor`) and injects it for those formats; passthrough formats keep `template: None` (a regression guard — bip388/sparrow branch on `template.is_some()`). Deriving from the descriptor (which carries `multi` vs `sortedmulti` verbatim) rather than the lossy `WalletScriptType` dissolves the multisig inverse-ambiguity the original FOLLOWUP flagged as the headline risk. The from-import-json path also now sets `threshold_user_supplied = threshold.is_some()` so multisig re-emits to sparrow don't spuriously demand `--threshold` (the threshold is envelope-derived). **Taproot stays walled off** (the pre-existing `wallet-import-taproot-internal-key` refusal precedes the derivation — `tr-multi-a`/`bip86` re-emit remains blocked on that separate FOLLOWUP). No clap flag/value/subcommand change → no GUI `schema_mirror` lockstep. Chapter-45 recipes stripped of the now-conflicting `--template`/`--threshold` in lockstep; recipes smoke-tested end-to-end. Brainstorm-spec opus R0 RED 1C/5I → R1 → R2 → R3 GREEN; plan-doc R0 GREEN; per-phase impl review GREEN.

## mnemonic-toolkit [0.36.4] — 2026-05-24

**SemVer-PATCH — install-pin staleness (config/CI/installer only; no code change).** Closes `manual-yml-and-install-sh-sibling-gui-pin-staleness`. The default installer (`scripts/install.sh`) pinned `mnemonic-gui-v0.10.0` — 11 versions behind the live GUI v0.21.1 — so it handed users a badly-stale GUI; bumped to `mnemonic-gui-v0.21.1`. Also bumped the lagging sibling-CLI pins in `manual.yml` (mk-cli v0.4.1→v0.4.2, md-cli v0.6.0→v0.6.1, ms-cli v0.4.0→v0.4.1, matching `install.sh`) and `quickstart.yml` (mk-cli v0.2.0→v0.4.2 — a 3rd stale site found in R0). None of these were gated by install-pin-check (which only checks the `mnemonic` self-pin); the manual/quickstart workflows validate them lazily on their next docs/tag trigger. `manual-gui.yml` is intentionally version-locked to its GUI-manual authoring snapshot and left untouched. The README-version guard (v0.36.3) forced both README markers → 0.36.4 in lockstep.

Files `export-wallet-from-import-json-template-format-reemit` (a real CLI limitation surfaced by test-running the documented round-trips: template-requiring foreign formats can't re-emit via `--from-import-json`; 5 chapter-45 round-trip recipes are impossible as written — deferred to a MINOR feature cycle per user decision) and updates `manual-prose-command-execution-gate` with that coupling. Plan opus R0 (RED 0C/1I → R1 GREEN).

## mnemonic-toolkit [0.36.3] — 2026-05-24

**SemVer-PATCH — documentation refresh (docs/test-only; no code change).** Remedies the gaps found by a full documentation audit (`cycle-prep-recon-documentation-audit.md`):

- **README refresh (both):** the repo-root README AND the crate-published `crates/mnemonic-toolkit/README.md` had silently decayed to v0.8.0 (28 versions; the crate had no freshness gate while the manual's 6-stage lint kept it current). Both refreshed to v0.36.x — current status, a grouped inventory of all 20 `mnemonic` subcommands, install via `scripts/install.sh` (no stale hand-maintained tag pin), and pointers to the manual (authoritative CLI reference) + CHANGELOG (version history) instead of drift-prone per-version duplication. New `tests/readme_version_current.rs` guards that BOTH READMEs carry a `<!-- toolkit-version: X -->` marker equal to `CARGO_PKG_VERSION` — the anti-decay gate that was missing.
- **Manual hygiene:** `cli-subcommands.list` now lists `electrum-decrypt` + `seedqr {encode,decode}` (the flag-coverage lint was blind to those 2 chapters; both verified flag-complete). The `41-mnemonic.md` intro now enumerates all 20 subcommands (was 14; omitted electrum-decrypt/seedqr/repair/inspect/compare-cost/xpub-search) with an explicit `{#mnemonic-xpub-search}` anchor (its `(v0.26.0)`-suffixed heading otherwise auto-slugs to a dangling link). Stale version stamps fixed ("mirrors v0.13.0", "snapshot as of v0.1's tag").

Files 2 FOLLOWUPs: `manual-prose-command-execution-gate` (the lint never runs documented recipes — the systemic gap behind the v0.28.1 round-trip breakage) + `manual-yml-sibling-cli-pin-staleness`. Plan opus R0 (RED 2C/2I → R1 GREEN — caught a SECOND stale README + a dangling anchor) + end-of-cycle GREEN.

## mnemonic-toolkit [0.36.2] — 2026-05-24

**SemVer-PATCH — test/lint hardening (no CLI surface change).** Rebuilds the argv-leakage audit (`tests/lint_argv_secret_flags.rs`) from a hand-frozen 28-row `CANONICAL_FLAG_ROWS` (frozen at v0.13.0; silently omitted 16 post-v0.13.0 secret-argv routes — nostr/silent-payment/electrum-decrypt/import-wallet/inspect/repair/verify-bundle/xpub-search/seedqr) into a **clap-derived 3-axis closure**: flag-NAME routes from `mnemonic gui-schema` (`secret && !boolean`), `--from` subcommands, and `--slot` subcommands, each **set-equal-checked against the live surface** (a new OR removed secret-argv route now fails the gate — a leading indicator, not a lagging hand-checklist) plus a per-route non-argv-channel evidence anchor (`*-stdin` / `=-` / `@env:` / refusal) with explicit `source_file` mapping. Audit-only: the enforced secret projections (`flag_is_secret` + `secret_in_argv_warning`) were already correct + complete (R0 exhaustively verified no real leak). Closes `lint-argv-secret-flags-canonical-table-rebuild-from-clap`; files `import-wallet-ms1-argv-advisory-gap`. Plan opus R0 (RED 3I→1I→2I→GREEN over 4 rounds) + end-of-cycle GREEN.

## mnemonic-toolkit [0.36.1] — 2026-05-24

**SemVer-PATCH — two additive flags on `mnemonic silent-payment`.**

- **`--passphrase` / `--passphrase-stdin`** — derive the silent-payment address for a BIP-39-passphrase-protected ("25th word") wallet. The passphrase threads into the phrase/ms1/entropy-hex derivation; an xprv input is passphrase-independent and warns-and-ignores. `--passphrase-stdin` reads whitespace-preserving (significant PBKDF2 salt), is `conflicts_with` `--passphrase`, and is refused alongside `--secret-stdin` (one stdin per invocation). Secret-class (zeroize + mlock + argv-leak advisory; already covered by `flag_is_secret`). Absent → byte-identical to v0.35.0/v0.36.0 (empty passphrase).
- **`--change-address`** — also emit the BIP-352 **m=0 change address** (`B_scan ‖ B_m=0`), additive to the normal output and unmistakably tagged "internal change detection only; never hand out as a receiving address" (human line + JSON `change_address` + a `change_address_warning` sibling marker so a JSON consumer can't surface it as a receive target). `--label 0` remains refused as the separate publish-path guard.

Closes FOLLOWUPs `silent-payment-passphrase` + `silent-payment-change-address-m0`. Paired GUI schema-mirror lockstep (3 new flag NAMES). Plan opus R0 GREEN (0C/0I) + per-phase review GREEN + end-of-cycle GREEN.

## mnemonic-toolkit [0.36.0] — 2026-05-23

**SemVer-MINOR — two new public-data subcommands: `decode-address` + `verify-message`.**

- **`mnemonic decode-address <ADDR>`** — decode a Bitcoin address into its network(s), script type, witness version, validity, and scriptPubKey (hex). Public-data utility (no secrets), the inverse of `convert --to address`. The testnet/testnet4/signet network ambiguity (shared `tb1`/base58 prefixes) is reported as a set; `regtest` (`bcrt1`) is distinct. Human + `--json`.
- **`mnemonic verify-message`** — VERIFY-ONLY (no signing) Bitcoin message-signature verification, covering both formats by address type: legacy "Bitcoin Signed Message" (`signmessage`, **P2PKH** via `bitcoin::sign_message`) and **BIP-322 simple** (P2WPKH/P2SH-P2WPKH/P2TR via the new `bip322` dep). `--format auto` (default) dispatches by address type; `--format legacy` on a non-P2PKH address is refused. `--message`/`--message-file`/`--message-stdin` + `--signature`; human + `--json`. Exit convention: valid → 0; decoded-but-invalid → 1 (structured `valid:false` on stdout); malformed input → 1 (error on stderr). The pinned `bip322 0.0.10` crate panics on a crafted P2SH + uncompressed-pubkey witness — verification is isolated with `catch_unwind` so adversarial public input yields a clean error, never a crash.

New deps: `bip322 = "=0.0.10"` (rust-bitcoin org; shares `bitcoin 0.32.8`, no duplicate). New `ToolkitError::{DecodeAddress, VerifyMessage}`. BIP-322 crypto validated against the BIP-322 mediawiki vectors; legacy against a self-generated RFC6979 vector (signing only in tests). Also: a convert `--help` entropy-row lock-test + an electrum-phrase→address refusal lock-test (the spot-check confirmed electrum-phrase is entropy-extraction-only — address derivation stays refused). Paired GUI schema-mirror lockstep. Plan opus R0 (RED 2C/2I → R1 GREEN) + per-phase review (RED 1C → GREEN, caught the bip322 panic) + end-of-cycle GREEN.

## mnemonic-toolkit [0.35.0] — 2026-05-23

**SemVer-MINOR — new `mnemonic silent-payment` subcommand (BIP-352 receiver address).** Derives the BIP-352 Silent Payments **receiver** static address from a seed-bearing secret: the scan key `m/352'/coin'/account'/1'/0` and spend key `.../0'/0` → the base `sp1…`/`tsp1…` address (`B_scan‖B_spend`), plus labeled addresses (`--label <m>`, m≥1; `B_m = B_spend + tagged_hash("BIP0352/Label", b_scan‖m)·G`). `--label 0` is refused (the reserved change label must never be published). Emits the scan/spend pubkeys + derivation paths + the scan (online) / spend (COLD) private keys behind the secret-on-stdout advisory (mlock-pinned + zeroized). Accepts a seed-bearing secret (BIP-39 phrase / ms1 / entropy-hex / master xprv) via `--secret`/`--secret-file`/`--secret-stdin`; single-key WIF/minikey is refused (cannot derive `m/352'`). `--network` (mainnet → `sp` + coin-type 0; testnet/signet/regtest → `tsp` + coin-type 1). New `ToolkitError::SilentPayment`. **No m-format cards** (SP is not an m-format artifact). **Sender output construction + chain scanning are out of scope** (no tx inputs / chain access / signing — the toolkit boundary). Crypto byte-exact-validated against the official BIP-352 `send_and_receive_test_vectors.json` (28 receiving cases, base + labeled). Paired GUI schema-mirror lockstep. Plan opus R0 (RED 1C/2I → R1 GREEN) + end-of-cycle GREEN.

## mnemonic-toolkit [0.34.7] — 2026-05-23

**SemVer-PATCH — process argv-hardening (`PR_SET_DUMPABLE`).** `mnemonic` now calls `prctl(PR_SET_DUMPABLE, 0)` at the top of `main()` (Linux; no-op elsewhere), making `/proc/$PID/` unreadable to OTHER non-root UIDs and disabling core dumps — so a secret passed inline on argv (against the `--*-stdin` advice) can no longer be harvested by another user via `/proc/$PID/cmdline` or a core file. The residual same-UID `/proc/cmdline` window is documented + accepted (a same-UID attacker already has ptrace/`/proc/mem` access). The in-place argv-overwrite alternative was deliberately declined (glibc/musl/static-linking-fragile + racy). New `mnemonic_toolkit::process_hardening` lib module. Cross-repo: the same hardening lands in md-cli v0.6.1 / ms-cli v0.4.1 / mk-cli v0.4.2 (install.sh pins bumped). Closes `argv-overwrite-after-parse`.

## mnemonic-toolkit [0.34.6] — 2026-05-22

**SemVer-PATCH — `import-wallet --network` signet/regtest disambiguation.** New `import-wallet --network <mainnet|testnet|signet|regtest>` re-binds the imported network to recover signet/regtest semantics from the coin-type-1→testnet collapse (BIP-129 BSMS + Bitcoin Core `listdescriptors` use coin-type `1` for testnet/signet/regtest alike, so v0.26.0 collapsed all three to testnet). The override is honored only WITHIN the parsed coin-type class (testnet ↔ {testnet,signet,regtest}; mainnet ↔ mainnet); a cross-class request is refused (`ImportWalletNetworkClassMismatch`, exit 1) because the blob's xpub prefix is coin-type-bound. Adds `CliNetwork::to_bitcoin_network`. Paired GUI schema-mirror lockstep + manual. Closes `wallet-import-signet-regtest-disambiguation`.

## mnemonic-toolkit [0.34.5] — 2026-05-22

**SemVer-PATCH — MiniKey stdout-redaction hardening + `SECRET_NODE_TYPES_ARGV`.** `convert --from minikey=<KEY> --to wif --json` no longer echoes the Casascius mini-key (a private key) unredacted in the JSON `from_value` field: the two `convert` stdout-redaction call sites now use the wider `is_argv_secret_bearing()` predicate (which includes MiniKey) instead of the narrow `is_secret_bearing()` (`:1042` from_value is the real fix; `:1069` secret-on-stdout is a no-op for MiniKey today but kept on one predicate). Promotes `pub const secret_taxonomy::SECRET_NODE_TYPES_ARGV` (the public mirror of that wide set = persistence set + `minikey`), locked by a new parity test against `is_argv_secret_bearing`. Additive public const → no GUI lockstep (existing `SECRET_NODE_TYPES` snapshot unchanged). Closes `convert-minikey-stdout-redaction` + `secret-taxonomy-argv-superset-promotion`.

## mnemonic-toolkit [0.34.4] — 2026-05-22

**SemVer-PATCH — `import-wallet` format-mismatch matrix completion.** Completes the 8×7 off-diagonal `--format X` vs sniff-as-Y refusal matrix: adds the 10 residual `ImportWalletFormatMismatch` arms (coldcard→electrum/jade; electrum→jade; sparrow→coldcard/electrum/jade/specter; specter→coldcard/electrum/jade) so an explicit `--format` against a blob of a different detected format always refuses cleanly (exit 1) instead of attempting a wrong-format parse. `Ambiguous`/`NoMatch` sniff outcomes remain tolerated (explicit opt-in). 10 new cells in `tests/cli_import_wallet_format_mismatch_matrix.rs`. No CLI surface change → no GUI/manual lockstep. Closes `wallet-import-format-mismatch-matrix-completion-discovered-gaps`.

## mnemonic-toolkit [0.34.3] — 2026-05-22

**SemVer-PATCH — wallet-cluster FOLLOWUP hygiene.** No behavior or CLI-surface change. Retires stale BSMS/BIP-129 FOLLOWUPs surfaced by a cycle-prep recon: closes `wallet-import-bsms-encrypted` (the BIP-129 §Encryption envelope shipped v0.31.0 as `--bsms-encryption-token`) and `wallet-import-bsms-round-1` (Round-1 *verify* shipped v0.27.0 as `--bsms-round1`; coordinator descriptor-assembly is out-of-scope per opus architect disposition); rewrites `bsms-bip129-full-cutover` down to its sole remaining sub-item (d) (6-line lenient-parser removal, a future MINOR) + collapses a duplicate stub; refreshes decayed line-citations (`bsms-taproot-emit`, `wallet-import-signet-regtest-disambiguation`). Ships two trivial closes: a direct unit test for the `extract_threshold` taproot defense-in-depth guard (`bsms-extract-threshold-defense-in-depth-direct-unit-test`), and a CLAUDE.md clarification that `schema_mirror` gates clap flag-NAME parity only, not runtime `--json` wire-shape (`schema-mirror-flag-name-vs-wire-shape-conceptual-clarification` option (c); residual (b) deferred to v0.30+). Also applies the lock-regen discipline from `cargo-lock-version-bump-lockstep`.

## mnemonic-toolkit [0.34.2] — 2026-05-22

**SemVer-PATCH — `mnemonic nostr --import` (read-only Bitcoin Core importdescriptors) + FOLLOWUPS hygiene.** New `nostr --import readonly` appends a ready-to-paste **watch-only** `importdescriptors` recipe built from the address descriptor(s) (`active:false`/`internal:false`, single-key non-ranged); `--all-script-types` emits one array with all four. New `nostr --timestamp <now|unix>` (default `0` = rescan-from-genesis) reuses export-wallet's parser. A shared `wallet_export::import_array_single` builder owns the JSON shape (the existing ranged export-wallet emitter is untouched). `--import spending|both` is reserved (deferred — FOLLOWUP `nostr-import-spending-descriptors`). No new secret on stdout (read-only). Also closes 4 stale/no-op FOLLOWUPs from a cycle-prep recon (`hex-dep-unused`, `dead-inner-guard-bundle-watch-only`, `watch-only-stderr-warning-suborder`, `error-rs-exit-code-arm-fragmentation-post-sort`). Paired GUI schema-mirror lockstep (`--import`/`--timestamp` on `nostr`).

## mnemonic-toolkit [0.34.1] — 2026-05-22

**SemVer-PATCH — import-wallet secret-memory hygiene.** Closes two FOLLOWUPs from v0.33.3: (1) `import-wallet-plaintext-blob-mlock-pin` — the wallet `blob` is now `mlock`-pinned for ALL formats via a single re-pinned guard (previously only the BIE1 decrypt arm), so a plaintext `use_encryption:false` seed-bearing Electrum wallet no longer sits swappable; (2) `bsms-decrypt-record-string-zeroizing` — `decrypt_bsms_record` returns `Zeroizing<String>` so the intermediate decrypted BSMS record is scrubbed on drop. Internal type/lifetime only — no CLI/wire/GUI/manual surface change.

## mnemonic-toolkit [0.34.0] — 2026-05-22

**SemVer-MINOR — new `mnemonic nostr` subcommand.** Wraps an existing nostr key (`npub`/`nsec`, NIP-19 bech32 or 64-hex) as Bitcoin addresses, descriptors, and (for `nsec`) a WIF — across taproot (`p2tr`, the native x-only key-path mapping; default) and non-taproot (`p2pkh`/`p2wpkh`/`p2sh-p2wpkh`, via the BIP-340 even-y `02‖x` form). For `nsec`, the secret is normalized to even-y (BIP-340) so the emitted WIF controls the emitted address (a `notice:` fires if normalization negated the key). `--all-script-types` emits all four types; `--json` for structured output; `--secret`/`--secret-file`/`--secret-stdin` for secret input (argv-leak + secret-on-stdout advisories via the shared `secret_advisory` helpers; `mlock`-pinned + zeroized); `--network`; an `electrum:` line emits a script-type-prefixed WIF for Electrum ▸ Import private keys. **No m-format cards** — a single raw nostr key has no xpub/chain-code/seed, so md1/mk1/ms1 are not faithfully expressible (verified against `md-codec`); the descriptor string is the watch-only "wrapper". New `ToolkitError::NostrKeyParse`; uses the `bitcoin::bech32` re-export (no new direct dep). Cross-impl address fixture validated against an independent pure-Python secp256k1 / BIP-340 / BIP-341 oracle. Paired GUI schema-mirror lockstep.

## mnemonic-toolkit [0.33.3] — 2026-05-21

**SemVer-PATCH — secret-memory hygiene.** The `import-wallet` orchestrator's wallet `blob` buffer is migrated from `Vec<u8>` to `Zeroizing<Vec<u8>>`, so in-memory wallet plaintext is scrubbed on drop. Closes the FOLLOWUP `import-wallet-blob-zeroizing` (filed at the v0.33.2 Cycle 19 Phase B close). Internal-only — no CLI/wire/GUI/manual surface change.

### Fixed

- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs`: `read_blob` now returns `Zeroizing<Vec<u8>>`; the BIE1 decrypt reassign `blob = plaintext.to_vec()` becomes `blob = plaintext;` (preserving the `Zeroizing` wrapper from `ecies_decrypt_storage` instead of cloning into a plain `Vec`), and the BSMS Round-2 decrypt reassign re-wraps via `Zeroizing::new(...)`. A plaintext Electrum wallet (`use_encryption:false`) can carry a seed, and the v0.33.2 BIE1 path writes decrypted seed/xprv-bearing JSON into this buffer — both are now wiped on drop regardless of import format. Follows the `resolved-slot-derived-account-zeroizing-field` (v0.10.1) field-migration precedent.

### Notes

- Type-only change; all read sites compile unchanged via `Zeroizing<Vec<u8>> → Vec<u8> → [u8]` deref coercion. 2253 cells unchanged (full regression green; the `Zeroizing` guarantee is type-level — no runtime zeroize assertion, matching the v0.10.1 precedent).
- Two pre-existing, out-of-scope hygiene gaps filed as follow-ons: `bsms-decrypt-record-string-zeroizing` (the `decrypt_bsms_record` intermediate `String`) + `import-wallet-plaintext-blob-mlock-pin` (the non-BIE1 blob is not `mlock`-pinned).

### Review

- Plan-doc opus R0 GREEN 0C/0I/3M (deref-coercion + mlock-reorder + reassigns verified); end-of-cycle opus GREEN 0C/0I/0M.

---

## mnemonic-toolkit [0.33.2] — 2026-05-21

**SemVer-PATCH — Electrum BIE1 storage-encrypted wallet import.** `mnemonic import-wallet` now decrypts and imports an Electrum **whole-file (user-password) storage-encrypted** wallet (`BIE1` magic) — the natural completion of the Electrum-encryption arc. Closes the FOLLOWUP `wallet-import-electrum-encrypted-storage-format-b`. SemVer PATCH (net-new flag NAMEs on an existing subcommand are additive — the Cycle-13 `--from` precedent — with mandatory GUI lockstep; MINOR is reserved for new top-level subcommands / breaking changes).

This is **Phase B** of Cycle 19; Phase A (`a62cf15`) shipped the `electrum_crypto` ECIES library (`derive_storage_eckey` / `ecies_decrypt_message` / `ecies_decrypt_storage`), verified byte-exact against Electrum's OWN committed `test_decrypt_message` KATs.

### Added

- `import-wallet --decrypt-password <VALUE>` / `--decrypt-password-file <PATH>` / `--decrypt-password-stdin` (optional, mutually-exclusive struct-level `ArgGroup`). A storage-encrypted Electrum wallet file is a single base64 blob (decoded magic `BIE1`), NOT JSON; the orchestrator **auto-detects** it (`electrum_crypto::detect_storage_magic`) and decrypts it to the wallet JSON BEFORE sniff/parse (mirroring the BSMS decrypt-then-parse orchestration), then imports watch-only as usual. Detection is `--format`-independent. Crypto: PBKDF2-HMAC-SHA512(pw, salt=`b""`, 1024) → secp256k1 scalar → ECDH → sha512 KDF → AES-128-CBC + HMAC-SHA256 (verify-before-decrypt) → zlib.
- `electrum_crypto::detect_storage_magic` + `ElectrumStorageMagic` (BIE1 / BIE2). New deps `crypto-bigint` (mod-n reduction) + `flate2` (zlib) landed in Phase A.

### Security / secret-handling

- The password is `Zeroizing` + `mlock`-pinned; inline `--decrypt-password` emits the argv-leakage advisory. The `--decrypt-password` / `--decrypt-password-stdin` flag NAMEs were already classified secret in `flag_is_secret` (v0.33.1), so `gui-schema` auto-emits `secret:true` for them on `import-wallet` (GUI masking + zeroize). Wrong password and corruption are unified into one non-leaky `"decryption failed (wrong password or corrupted wallet file)"` message (no oracle); BIE2 is detected before any key derivation.
- `--decrypt-password-stdin` joins the single-stdin-consumer guard (refused alongside `--blob=-` or `--bsms-encryption-token=-`).
- The decrypted whole-file JSON can carry seed/xprv material; it is `mlock`-pinned, but the orchestrator's `blob: Vec<u8>` is not zeroizing (a pre-existing import-wallet property — `read_blob` returns a plain `Vec` for all formats — now slightly more load-bearing). Tracked as FOLLOWUP `import-wallet-blob-zeroizing`. The import OUTPUT is watch-only (xpub/derivation) — non-secret.

### Out of scope

- **BIE2 / hardware-device (xpub) storage encryption** cannot be decrypted from a password (the key is the device's master key); detected and refused with a clear advisory.

### Documentation

- `docs/manual/src/40-cli-reference/41-mnemonic.md`: `import-wallet` gains the three `--decrypt-password*` flag rows + the BIE1-decrypt / BIE2-refusal / wrong-password / ignored-password advisory rows.

### Tests

- 2253 cells (+16): 6 `detect_storage_magic` unit cells + 10 `cli_import_wallet_electrum_bie1` integration cells (inline/file/stdin happy paths; wrong-password; BIE2 refusal; no-password demand; two stdin-contention refusals; `--format bsms`+BIE1 decrypt-then-mismatch; password-on-plaintext soft-ignore). Fixtures generated by the INDEPENDENT pure-Python `ecdsa` regen script `tests/external/regen_electrum_bie1_storage.py` (cross-impl witness of the whole-file `zlib(json)` → ECIES framing).

### Cross-repo lockstep

NEW flag NAMEs on `import-wallet` → mandatory GUI `schema_mirror` lockstep. Paired `mnemonic-gui-v0.18.1` adds the three flags to the import-wallet SubcommandSchema (`secret:true` on inline + stdin). Tracked at FOLLOWUP `gui-import-wallet-decrypt-password-mirror`.

### Review

- Plan-doc opus R0 YELLOW 0C/2I/5M (3-way stdin guard, `--format`-independent precedence, SemVer PATCH confirmed, zeroizing FOLLOWUP, shared trim) → folded → R1 GREEN. End-of-cycle opus GREEN.

---

## mnemonic-toolkit [0.33.1] — 2026-05-21

**SemVer-PATCH — secret-classification fix.** `secrets::flag_is_secret` now classifies the v0.33.0 `electrum-decrypt` password flags `--decrypt-password` and `--decrypt-password-stdin` as secret. This was a gap: the CLI runtime already treated `--decrypt-password` as secret (it fires `secret_in_argv_warning`), but the `flag_is_secret` projection — which drives `mnemonic-gui`'s password-field masking, paste-warn / run-confirm modals, and exit-time zeroize sweeps — omitted them. The `gui-schema` v5 `secret` field for these two flags now emits `true`. This is the exact class the GUI's `schema_mirror_secret_drift` gate exists to catch (the v0.3.0–v0.3.2 BIP-39 persistence-leak class).

### Fixed

- `crates/mnemonic-toolkit/src/secrets.rs`: add `--decrypt-password` + `--decrypt-password-stdin` to `flag_is_secret`. `--decrypt-password-file` is deliberately NOT classified secret (its value is a filesystem path, not the secret itself) — locked by a new entry in the non-secret unit-test list, alongside `--ciphertext` (encrypted material, not plaintext secret).

### Cross-repo lockstep

This is the prerequisite for the paired `mnemonic-gui-v0.18.0` (Cycle 18b): the GUI pins v0.33.1 and mirrors `secret: true` on the two password flags, keeping the `schema_mirror_secret_drift` gate green. Tracked at FOLLOWUP `gui-electrum-decrypt-subcommand-mirror`.

### Test totals

- 2223 cells passing (+2 net: the two new secret-flag unit-test assertions).

---

## mnemonic-toolkit [0.33.0] — 2026-05-21

**SemVer-MINOR release.** New `mnemonic electrum-decrypt` subcommand surfaces the (previously dead-code) `electrum_crypto::decrypt_field` primitive: decrypt an Electrum field-encrypted secret (`base64(iv ‖ aes-256-cbc(plaintext + PKCS7))`, key = `sha256d(password)`) and emit the recovered plaintext (Electrum-native seed phrase or BIP-32 xprv). Closes `electrum-crypto-seed-extraction-subcommand` FOLLOWUP — the first of the final v0.32+ Electrum pair.

### Added

- `mnemonic electrum-decrypt` subcommand. Architect+user-locked Option A (dedicated subcommand, NOT a `convert` source — the decrypted node-type (phrase vs xprv) is unknowable pre-decryption, which `convert`'s commit-types-up-front model cannot express).
  - `--ciphertext <VALUE|->` (the base64 field; `-` from stdin; not secret).
  - 3-form password family: `--decrypt-password <VAL>` (inline; argv-leakage advisory) + `--decrypt-password-file <PATH>` + `--decrypt-password-stdin`. Bound by a struct-level clap arg-group (exactly one required, mutually exclusive; missing/multiple → exit 64).
  - `--json-out <PATH>` (envelope `{schema_version, operation, plaintext}`; no password echo; world-readable advisory).
- New `secret_advisory::secret_on_stdout_warning_unconditional` — the existing `CardKind::Ms1`-gated helper cannot fire for a free-form Electrum plaintext; the gated wrapper now delegates to the unconditional form for `Ms1` (behavior-preserving).
- 12 integration cells (3 password forms + wrong-password + bad-base64 + arg-group none/conflict + stdin-contention + ciphertext-stdin + json-envelope-no-password-echo + realistic-seed fixture via `encrypt_field` + world-readable advisory).

### Security / secret-handling

- Inline `--decrypt-password` emits the argv-leakage advisory; the recovered plaintext on stdout emits a secret-on-stdout advisory; the `--json-out` path emits a world-readable-permissions advisory (the `seed_xor`/`slip39`/`final_word` precedent). Password + plaintext are `Zeroizing` + `mlock`-pinned. NO password echo in the JSON envelope.
- Format A field encryption carries no MAC, so a wrong password (PKCS7-unpad refusal) and a non-UTF-8 result are unified into one `"decryption failed (wrong password or corrupted ciphertext)"` message (no failure-mode leak).

### Documentation

- `docs/manual/src/40-cli-reference/41-mnemonic.md`: new `## mnemonic electrum-decrypt` reference section.

### Test totals

- 2221 cells passing; 12 ignored. +12 net (vs v0.32.3 baseline 2209).

### Cross-repo lockstep

NEW subcommand → the GUI `schema_mirror` gate requires a new `SubcommandSchema` entry. Paired `mnemonic-gui-v0.18.0` (Cycle 18b) adds the `electrum-decrypt` schema + bumps the toolkit pin. Tracked at FOLLOWUP `gui-electrum-decrypt-subcommand-mirror`.

### Cycle topology

Cycle 18 — first 0.33.x MINOR; surfaces the Cycle-6a electrum_crypto library (Format A field decryption). 1 v0.32+ FOLLOWUP remains: `wallet-import-electrum-encrypted-storage-format-b` (whole-file Format B; reuses this cycle's `--decrypt-password*` surface).

### Review

- Plan-doc opus R0: YELLOW 0C/3I (secret-advisory mechanics: unconditional-stdout helper + `warn_if_world_readable` precedent + struct-level ArgGroup, all folded inline).
- End-of-cycle opus: GREEN.

---

## mnemonic-toolkit [0.32.3] — 2026-05-21

**SemVer-PATCH release** (test/fixture/doc-only). Pins the toolkit's BIP-129 §Encryption implementation against the independent Coinkite Python reference via vendored cross-impl fixtures. Closes `bsms-encryption-cross-impl-coinkite-python-smoke` FOLLOWUP — the **third and final** BIP-129-BSMS arc step. With this, the entire `bsms-bip129-encryption-envelope` Cycle-7 follow-on arc is retired.

### Added

- `crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-coinkite-xref-round2-2of3.dat` — a hex `MAC || ciphertext` wire produced by the independent Coinkite reference's `bsms.encryption.encrypt()` (pinned SHA `c30abe3a6d9823b6a3003e89acd66b9f38e11f1c`, frozen 2023-01-24) over the real `bsms-2line-multi-2of3.txt` Round-2 descriptor with an EXTENDED 16-byte token. Combined with the existing `bsms-encrypted-standard-tv3.dat` (Coinkite-generated TV-3 Round-1, STANDARD 8-byte token), both record kinds + both token widths are now cross-validated.
- `crates/mnemonic-toolkit/tests/external/regen_coinkite_vectors.py` + `README.md` — the deterministic regeneration script (reads plaintext as exact bytes, token stripped; self-verifies by re-decrypting its own output before writing) + documentation (pinned SHA, `pyaes` venv recipe, vendored-vs-live-CI rationale).
- 3 integration cells: `coinkite_xref_round2_full_plaintext_byte_equal` (the strong pin — decrypt the Coinkite wire via `bsms_crypto` + assert byte-equality over the FULL ~460-byte plaintext), `coinkite_xref_round2_descriptor_imports` (end-to-end CLI import; first EXTENDED-token wire that actually decrypts via the CLI), `coinkite_xref_round2_wrong_token_mac_mismatch` (exit 2).

### Documentation

- `docs/manual/src/45-foreign-formats.md`: BSMS-encrypted-envelopes bullet documents the cross-impl validation against the Coinkite reference + the regen pointer.

### Scope note

The originating FOLLOWUP also sketched a LIVE CI-gated smoke (clone Coinkite + run `python3 test.py`). That was intentionally NARROWED to vendored-only (no clone/pip/network in CI) per a deliberate user scope-lock — the Coinkite repo is frozen, the toolkit crypto is already byte-exact against BIP-129 TV-3, and a live external-clone CI surface adds fragility for marginal drift-detection value. The live-CI residual is explicitly WAIVED (not deferred); the `regen_coinkite_vectors.py` script is the documented manual-refresh path.

### Test totals

- 2209 cells passing; 12 ignored. +3 net (vs v0.32.2 baseline 2206).

### Cycle topology

Cycle 17 — eighth cycle of the v0.32+ tier; **closes the BIP-129-BSMS arc** (round1-decrypt-then-verify v0.32.1 + per-signer-tokens v0.32.2 + cross-impl-coinkite-smoke v0.32.3). 2 v0.32+ FOLLOWUPs remain (both Electrum). No GUI lockstep (test-only).

### Review

- Plan-doc opus R0: GREEN 0C/1I/2M (full-plaintext-equality cell + regen newline self-verify + scope-audit closure note, all folded inline).
- End-of-cycle opus: GREEN.

---

## mnemonic-toolkit [0.32.2] — 2026-05-21

**SemVer-PATCH release.** `--bsms-encryption-token` is now repeatable, enabling per-Signer BIP-129 encryption tokens (BIP-129 line 74: "one common TOKEN for all Signers, or one per Signer"). Closes `bsms-encryption-per-signer-tokens` FOLLOWUP — the second of the three BIP-129-encryption follow-ons from Cycle 7. Purely additive: a single `--bsms-encryption-token` is unchanged; supplying it multiple times (previously a clap error) now pairs tokens with Signers.

### Changed

- `import-wallet --bsms-encryption-token`: `Option<PathBuf>` → `Vec<PathBuf>` (clap-derive auto-Append, mirroring `--bsms-round1`). Pairing:
  - **1 token (SHARED)** → decrypts every encrypted Round-1 record + the Round-2 `--blob` (backward-compatible; byte-identical to v0.31.0/v0.32.1).
  - **N>1 tokens (PER-SIGNER positional)** → `token[i]` decrypts `--bsms-round1` `record[i]`; requires (a) ≥1 record, (b) all records encrypted, (c) `N == record count`, (d) no encrypted Round-2 `--blob`.
- `verify_bsms_round1_files` takes `tokens: &[BsmsToken]` + the positional pre-checks + per-record token selection.
- Generalized the single-stdin guard: at most one `--bsms-encryption-token=-`; refuse `--blob=- AND any token=-`.

### Added

- 8 integration cells: positional happy-path; single-token-shared (2 records); count-mismatch refusal; mixed plaintext/encrypted refusal; multi-token + encrypted-Round-2-blob refusal; gap-h (N>1 tokens + 0 records) refusal; per-record-`i` MAC-mismatch attribution; two-stdin-token refusal. (2nd encrypted record built via a generalized `reencrypt_with_token` test helper using the `bsms_crypto` pub primitives.)

### Documentation

- `docs/manual/src/40-cli-reference/41-mnemonic.md`: `--bsms-encryption-token` documents repeatability + the shared-vs-positional rules + per-Signer constraints + single-stdin-token rule.

### Behavior preservation

- All single-token paths (Round-1 shared decrypt + Round-2 blob decrypt) are byte-identical to v0.32.1 (the prior 17 encrypted-suite cells + 15 round1 cells green).

### Test totals

- 2206 cells passing; 12 ignored. +8 net (vs v0.32.1 baseline 2198).

### Cross-repo lockstep

GUI lockstep is OPTIONAL (not gate-forced): the GUI `schema_mirror` compares clap flag-NAME parity, and `--bsms-encryption-token`'s name is unchanged — only its `repeating` cardinality changed. Tracked at FOLLOWUP `gui-bsms-encryption-token-repeating-mirror` (GUI v0.17.1: flip the FlagSchema `repeating: false → true` so the GUI can add multiple token rows). Non-blocking.

### Cycle topology

Cycle 16 — seventh cycle of the v0.32+ tier; second of the sequential BIP-129-BSMS arc. SemVer-PATCH (additive). 3 v0.32+ FOLLOWUPs remain (2 Electrum + 1 BIP-129 `bsms-encryption-cross-impl-coinkite-python-smoke`).

### Review

- Plan-doc opus R0: YELLOW 0C/2I/2M (gap-h guard + error-precedence doc + Append-idiom + 2 cells, all folded inline).
- End-of-cycle opus: GREEN.

---

## mnemonic-toolkit [0.32.1] — 2026-05-21

**SemVer-PATCH release.** Behavior expansion: `import-wallet --bsms-round1` now accepts ENCRYPTED Round-1 KEY records (hex `MAC || ciphertext`) in addition to plaintext 5-line records, decrypting them with the shared `--bsms-encryption-token` before the existing BIP-322 signature verify. Closes `bsms-encryption-round1-decrypt-then-verify` FOLLOWUP — the first of the three BIP-129-encryption follow-ons from Cycle 7. Closes the TV-3 decrypt-then-refuse boundary.

### Changed

- `--bsms-round1 <FILE>` auto-detects encrypted vs plaintext records (`is_encrypted_bsms_record`: raw hex with no `BSMS 1.0` header → encrypted). Encrypted records decrypt via the same `bsms_crypto` recipe as the Round-2 path (PBKDF2-SHA512 → AES-256-CTR → HMAC-SHA256, Encrypt-and-MAC), MAC-verify, then flow into the existing `parse_round1` + BIP-322 verify.
- The `--bsms-encryption-token` is now read + width-validated ONCE (new `BsmsToken` struct + `read_and_validate_bsms_token`) and shared between the Round-1 verify path and the Round-2 descriptor-decrypt block (de-duplicating the token read; prerequisite for the per-Signer-token follow-on). The stdin-contention guard was hoisted above the Round-1 verify path so the dual-stdin (`--blob=- AND --bsms-encryption-token=-`) refusal fires before the token consumes stdin.
- New shared `decrypt_bsms_record(text, token, ctx)` helper backs both decrypt paths (the Round-2 block now consumes it; NOTICE + error text byte-identical).

### Added

- 5 integration cells: `tv3_round1_decrypt_then_verify`, `round1_encrypted_without_token_refused`, `round1_encrypted_wrong_token_mac_mismatch`, `round1_plaintext_still_verifies_no_misclassify`, `round1_encrypted_decrypt_ok_but_sig_fail` (lenient NOTICE + `--bsms-verify-strict` fatal; fixture built via test-time re-encryption with the `bsms_crypto` pub primitives). 1 in-file unit cell for `is_encrypted_bsms_record`.

### Documentation

- `docs/manual/src/40-cli-reference/41-mnemonic.md`: `--bsms-round1` documents the dual plaintext/encrypted intake; `--bsms-encryption-token` documents the shared Round-1+Round-2 usage; new encrypted-Round-1 stderr NOTICE row.

### Behavior preservation

- The encrypted Round-2 `--blob` path (NOTICE + MAC + parser refusal of TV-3's 5-line record) is byte-identical (12 prior encrypted-suite cells green). A plaintext Round-1 record via `--bsms-round1` (no token) still verifies — the encrypted-detection never mis-classifies plaintext (the `BSMS 1.0` header is not all-hex).

### Test totals

- 2198 cells passing; 12 ignored. +6 net (vs v0.32.0 baseline 2192).

### Cycle topology

Cycle 15 — sixth cycle of the v0.32+ tier; first of the sequential BIP-129-BSMS arc (`bsms-encryption-round1-decrypt-then-verify` → `bsms-encryption-per-signer-tokens` → `bsms-encryption-cross-impl-coinkite-python-smoke`). No new flag → no GUI lockstep. 4 v0.32+ FOLLOWUPs remain (2 Electrum + 2 BIP-129).

### Review

- Plan-doc opus R0: GREEN 0C/1I/2M (hoist-site reorder + cite-existing-TV3-test + decrypt-OK-sig-FAIL cell, all folded inline).
- End-of-cycle opus: GREEN.

---

## mnemonic-toolkit [0.32.0] — 2026-05-21

**SemVer-MINOR release.** New `--variant <standard|compact>` flag adds CompactSeedQR support to `mnemonic seedqr`. Closes `seedqr-compact-variant` FOLLOWUP — the last of the four SeedQR follow-ons from the v0.30.0 introductory cycle.

CompactSeedQR (SeedSigner's binary-mode QR variant) stores the raw BIP-39 entropy bytes instead of the decimal word-index digits. The toolkit represents the payload as lowercase hex: 16 bytes (32 hex chars) for 12-word, 32 bytes (64 hex chars) for 24-word. Per SeedSigner's `CompactSeedQrEncoder` (primary-source verified), only 12 and 24 words are compact-supported.

### Added

- `--variant <standard|compact>` flag (default `standard`) on both `mnemonic seedqr encode` and `mnemonic seedqr decode`. Derived `SeedqrVariant` ValueEnum.
- `seedqr::encode_compact` / `seedqr::decode_compact` library primitives. `encode_compact` = `Mnemonic::to_entropy()` → hex (the to_entropy bytes are exactly the SeedSigner compact payload: 11-bit index pack minus checksum). `decode_compact` = hex → byte-count check {16,32} → `from_entropy_in` → phrase.
- 3 library-local `SeedqrError` variants: `CompactInvalidHex`, `CompactByteCountUnsupported`, `CompactWordCountUnsupported`.
- 18 new test cells (10 lib unit + 8 CLI integration): compact encode/decode/round-trip 12+24, JSON-envelope `variant: compact`, 15-word + 20-byte + invalid-hex refusals, uppercase/whitespace-hex acceptance, standard-decode-of-64-char-hex clean-error footgun check.

### Changed

- `SeedqrEnvelope.variant` field (present since v0.30.0, hardcoded `"standard"`) now reflects the selected variant on both emit sites. The `digits` field holds the payload (decimal for standard, hex for compact).

### Documentation

- `docs/manual/src/40-cli-reference/41-mnemonic.md` `mnemonic seedqr`: synopsis + flags document `--variant`; §Scope flips CompactSeedQR from "deferred" to shipped (12/24 only, hex form); new worked example with the `xxd -r -p | qrencode -8` binary-QR render recipe.

### Test totals

- 2192 cells passing; 12 ignored. +18 net (vs v0.31.6 baseline 2174).

### Cross-repo lockstep

`--variant` is a net-new flag NAME on TWO subcommands (seedqr-encode + seedqr-decode) → trips the GUI `schema_mirror` flag-NAME-parity gate. Paired GUI release `mnemonic-gui-v0.17.0` (Cycle 14b) adds the `--variant` dropdown to both schema entries + bumps the toolkit pin. Tracked at FOLLOWUP `gui-seedqr-variant-flag-mirror`.

### Cycle topology

Cycle 14 — fifth cycle of the v0.32+ tier; the **first 0.32.x MINOR release** and the close of the SeedQR-completion arc (all four v0.30.0 SeedQR follow-ons now shipped: bundle-slot v0.31.3, 15/18/21 word-counts v0.31.5, --from unification v0.31.6, compact-variant v0.32.0). 5 v0.32+ FOLLOWUPs remain (2 Electrum + 3 BIP-129).

### Review

- Plan-doc opus R0: GREEN 0C/0I/3M (all folded inline: derived ValueEnum; +3 CLI test cells; citation line-drift).
- End-of-cycle opus: GREEN.

---

## mnemonic-toolkit [0.31.6] — 2026-05-21

**SemVer-PATCH release.** Surface unification + deprecation: the SeedQR digit-string input is unified into the shared `--from <node>=<value>` grammar via a new `NodeType::Seedqr`. `mnemonic convert --from seedqr=<digits> --to <node>` is now wired end-to-end, and `mnemonic seedqr decode` gains the canonical `--from seedqr=<digits>` form. The original `--digits` flag is preserved as a deprecated alias (stderr notice; removal in a future release). Closes `seedqr-digits-from-input-unification` FOLLOWUP filed at v0.30.0 Cycle 5 plan-doc R0 §I4.

### Added

- `NodeType::Seedqr` in `cmd/convert.rs` (declared at enum position 1, after Phrase). First-class input node through `classify_edge` + `is_supported_direct_edge` + `compute_outputs`. Supported edges: `(Seedqr, {Phrase, Entropy, Xpub, Xprv, Fingerprint, Ms1, Wif, Bip38, Address})`. The `(Seedqr, Phrase)` edge IS permitted (the canonical decode) — distinct from the `(Phrase, Phrase)` identity barrier.
- `mnemonic seedqr decode --from seedqr=<VALUE|->` canonical input form. Only the `seedqr` node type is accepted on `seedqr decode --from`; other node types are refused (exit 1).
- 12 integration cells in `tests/cli_seedqr_from_unification.rs` covering both surfaces (convert seedqr→phrase/entropy/xpub + stdin + invalid-digits + `--to seedqr` clap rejection; seedqr-decode canonical `--from` + stdin + `--digits` deprecation notice + both-flags clap conflict + required-input + non-seedqr-node refusal).

### Changed

- `mnemonic seedqr decode`: `--digits` is now DEPRECATED. Still accepted, but emits a stderr notice (`--digits is deprecated; use --from seedqr=<VALUE|-> instead`) and is mutually exclusive with `--from` (clap-level `conflicts_with`; exit 64 EX_USAGE at parse-time). Exactly one of `--from seedqr=` or `--digits` is required.
- `NodeType::Seedqr` is secret-bearing (decodes to a BIP-39 phrase) → added to `is_secret_bearing`, `secret_taxonomy::SECRET_NODE_TYPES`, and the `declare_node_type_variants!` parity macro. `is_argv_secret_bearing` auto-flows → `--from seedqr=` emits the argv-leakage advisory. `edge_uses_pbkdf2` extended to include Seedqr (decodes to phrase → PBKDF2 derivation path).
- `--to seedqr` is intentionally absent from the `--to` PossibleValuesParser list (input-only node); clap rejects it at parse-time (exit 64).

### Documentation

- `docs/manual/src/40-cli-reference/41-mnemonic.md`: `mnemonic convert` node list + `--from` / `--to` rows document the `seedqr` input-only node. `mnemonic seedqr decode` synopsis + flags + worked-example switched to the canonical `--from seedqr=` form; `--digits` documented as deprecated. Flag-coverage lint gate green.

### Test totals

- 2174 cells passing; 12 ignored. +12 net (vs v0.31.5 baseline 2162).

### Cross-repo lockstep

`--from` is a NET-NEW flag name on `seedqr decode` — this trips the GUI `schema_mirror` flag-NAME-parity gate (unlike the value-content additions of Cycles 10/12). Paired GUI release `mnemonic-gui-v0.16.2` (Cycle 13b) adds `--from` to the `seedqr-decode` SubcommandSchema + bumps the toolkit pin. Tracked at FOLLOWUP `gui-seedqr-decode-from-flag-mirror`.

### Cycle topology

Cycle 13 — fourth cycle of the v0.32+ tier (first of the sequential SeedQR-completion pair; `seedqr-compact-variant` remains for the MINOR v0.32.0 cycle). Toolkit side; GUI lockstep follows as 13b.

### Review

- Plan-doc opus R0: YELLOW 0C/3I/1M — all folded inline pre-Phase-2 (I1 substitution-cascade ordering; I2 `flag_is_secret("--digits")` preserved; I3 clap-level conflicts_with; M1 stdin end-to-end cell). NOTE: the substitution-to-Phrase approach (R0 I1) was later replaced during Phase 2 with native Seedqr edge-wiring after discovering `(Phrase, Phrase)` is an identity barrier — the `(Seedqr, Phrase)` decode must remain distinct.
- End-of-cycle opus: GREEN.

---

## mnemonic-toolkit [0.31.5] — 2026-05-21

**SemVer-PATCH release.** Behavior expansion: `mnemonic seedqr {encode, decode}` word-count support widened from `{12, 24}` → `{12, 15, 18, 21, 24}` (the complete BIP-39 word-count set). Closes `seedqr-15-18-21-word-counts` FOLLOWUP filed at v0.30.0 Cycle 5 brainstorm close.

### Changed

- `seedqr::decode` digit-length gate at `crates/mnemonic-toolkit/src/seedqr.rs`: from `len != 48 && len != 96` to `matches!(len, 48 | 60 | 72 | 84 | 96)`. The new gate accepts 60/72/84 digits = 15/18/21 BIP-39 words × 4 decimal digits per word.
- `seedqr::encode` word-count gate: from `words.len() != 12 && words.len() != 24` to `matches!(words.len(), 12 | 15 | 18 | 21 | 24)`.
- Error texts updated: `"invalid digit count (expected 48 or 96; got N)"` → `"invalid digit count (expected 48, 60, 72, 84, or 96; got N)"`; `"invalid word count: N (only 12 or 24 supported)"` → `"invalid word count: N (only 12, 15, 18, 21, or 24 supported)"`.

### Added

- 9 new in-file lib unit cells: `decode_15_word_canonical`, `encode_15_word_canonical`, `round_trip_15_word`, and the 18 + 21-word equivalents.
- 1 new lib cell: `encode_rejects_22_word_count` — boundary refusal between 21 and 24 (locks the no-silent-accept claim).
- 3 CLI happy-path cell conversions: `encode_rejects_{15,18,21}_word_count` → `encode_accepts_{15,18,21}_word_count` with byte-exact expected-digits stdout assertions.
- 1 new CLI JSON-envelope cell: `encode_json_mode_15_word` — confirms `word_count: 15` emits in the JSON envelope (R0 I3b fold).
- Canonical Trezor zero-entropy vectors documented in the lib `tests` mod (15-word "abandon ×14 + address"; 18-word "abandon ×17 + agent"; 21-word "abandon ×20 + admit"). Derived empirically via `mnemonic convert --from entropy=<20/24/28-byte-zeros> --to phrase`.

### Documentation

- `docs/manual/src/40-cli-reference/41-mnemonic.md` `mnemonic seedqr` section: `--digits` / `--from phrase=` documentation reflects all 5 word counts. §"Scope" retitled to "(v0.30.0, widened in v0.31.5)" with the new word-count enumeration + SeedSigner spec rationale. Canonical error-text quotes in §"Stderr / exit codes" updated.

### Test totals

- 2162 cells passing; 12 ignored. +10 net (vs v0.31.4 baseline 2152).

### Cycle topology

Cycle 12 — third cycle of the v0.32+ tier. 7 v0.32+ FOLLOWUPs remain from cycles 5-7 (2 SeedQR + 2 Electrum + 3 BIP-129). Toolkit-only (no GUI lockstep; no clap surface change).

### Review

- Plan-doc opus R0: YELLOW 0C/3I/0M — all 3 Importants folded inline pre-Phase-2 (I1 risk-register error-text claim factually wrong; I3a drop duplicate boundary cells; I3b add JSON-envelope cell).
- End-of-cycle opus: GREEN.

---

## mnemonic-toolkit [0.31.4] — 2026-05-21

**SemVer-PATCH release.** Defensive hardening: widens the descriptor-passthrough discriminator at `wallet_import/sparrow.rs::parse` Step 6 from the literal substring `script_template.contains("@0/**")` to a regex `Regex::new(r"@\d+/\*\*").is_match(...)`. Closes `sparrow-import-detection-regex-defensive-widening` FOLLOWUP filed at v0.31.2 Cycle 9 close (end-of-cycle opus M1 finding).

Under the current Sparrow emit invariant (`wallet_export/sparrow.rs:230` indexes placeholders from `(0..n)`) `@0/**` is always present in template-mode blobs, so v0.31.4 produces **no behavior change** in the field. The widening is purely defensive — a hypothetical future emit-side change (e.g., a Sparrow patch that indexes cosigners from 1, or a non-canonical template producer) would have silently mis-classified `wpkh(@1/**)` as descriptor-passthrough under the substring discriminator; the regex catches any digit-indexed placeholder.

### Changed

- `crates/mnemonic-toolkit/src/wallet_import/sparrow.rs::parse` Step 6: `has_at_placeholder` predicate now uses `regex::Regex::new(r"@\d+/\*\*").expect("at-placeholder regex is a fixed string literal").is_match(&script_template)`. Inline `Regex::new` per the project's established pattern at `sparrow.rs:555/566/678`, `bsms.rs:501/520`, `bitcoin_core.rs:530/553/561`, etc. (the R0 reviewer caught that my initial `LazyLock` choice had ZERO usages anywhere in the crate; folded to the precedent).

### Added

- 2 in-file lib unit cells: `at_placeholder_regex_matches_only_template_mode_shapes` (regex-unit; 7 positive cases + 5 negative cases) + `parse_at_0_placeholder_still_routes_to_template_mode_substitution` (backward-compat regression locking the no-behavior-change claim against the existing `sparrow-singlesig-p2wpkh.json` fixture).

### Test totals

- 2152 cells passing; 12 ignored. +2 net (vs v0.31.3 baseline 2150).

### Cycle topology

Cycle 11 — second cycle of the v0.32+ tier. 8 v0.32+ FOLLOWUPs remain from cycles 5-9. Toolkit-only (no GUI lockstep; no clap surface change).

### Review

- Plan-doc opus R0: YELLOW (0C/2I/1M) — both Importants folded inline pre-Phase-2 (I1 LazyLock → inline Regex::new(); I2 test surface → regex-unit + backward-compat cells).
- End-of-cycle opus: GREEN.

---

## mnemonic-toolkit [0.31.3] — 2026-05-21

**SemVer-PATCH release.** Behavior expansion: new `SlotSubkey::Seedqr` variant. `--slot @N.seedqr=<digit-string>` is now accepted on `mnemonic bundle` + `mnemonic verify-bundle` (refused on `mnemonic export-wallet` per the SPEC §3 watch-only-by-definition invariant). The value is a 48- or 96-digit SeedQR string per the SeedSigner SeedQR spec; it's decoded inline via the existing `seedqr::decode` library primitive at slot-emit time, and the resulting BIP-39 phrase is materialized into the slot identically to a `--slot @N.phrase=` invocation. Closes Cycle 10 (`seedqr-bundle-slot-integration` FOLLOWUP — the first v0.32+ tier follow-on from Cycle 5's introductory `mnemonic seedqr` subcommand).

### Added

- New `SlotSubkey::Seedqr` variant in `crates/mnemonic-toolkit/src/slot_input.rs`. Declared at position 1 in the enum (after `Phrase`, before `Entropy`) so derived `Ord` slots Seedqr at index 1 — yielding ascending-sorted legal-set patterns `[Seedqr]`, `[Seedqr, Path]`, and `[Seedqr, Fingerprint, Path]` that mirror the existing v0.19.0 SPEC §6.6.b exception for Phrase.
- Phrase + Seedqr unified branch in `cmd/bundle.rs` slot-consumer (after the Phrase block, before Xpub). The Seedqr branch decodes via `mnemonic_toolkit::seedqr::decode`, maps errors via the canonical `crate::cmd::seedqr::map_seedqr_error` helper (promoted to `pub(crate)` to avoid error-text drift across consumer sites), and dispatches the resulting phrase through the same `derive_full` + `ResolvedSlot` materialization as Phrase.
- Mirrored consumer branch in `cmd/verify_bundle.rs`; path-override pipelines in both files extended to route Seedqr-bearing slots through the per-`@N` override path (parallel to the v0.19.0 `[Phrase, Path]` handling).
- `wallet_export/mod.rs::validate_watch_only` extended to refuse Seedqr alongside Phrase / Entropy / Xprv / Wif (SPEC §3 watch-only invariant correctly fires on the new subkey).
- `--slot` clap help text + manual chapter 41 enumerations updated on all three consumers (`bundle`, `verify-bundle`, `export-wallet`) to document the `seedqr` token + decode-at-slot-emit semantics. Backfilled `master_xpub` token (pre-existing v0.x drift; touch-and-fix at the same edit sites).
- 9 new integration cells: 6 in `tests/cli_bundle_seedqr_slot.rs` (24-word + 12-word byte-equal-vs-phrase happy paths, invalid-digit-count + checksum-failure refusals, stdin-sentinel happy path, double-stdin refusal); 2 in `tests/cli_verify_bundle_seedqr_slot.rs` (round-trip byte-equal vs phrase-slot using bip84-mainnet vector; decode-error path); 1 in `tests/cli_export_wallet_seedqr_slot.rs` (SPEC §3 watch-only refusal cell).
- 6 new in-file lib unit cells in `slot_input.rs` (parse_happy_seedqr, parse_seedqr_stdin_sentinel, validate_single_seedqr_passes, validate_seedqr_plus_path_passes_v0_19_0, validate_seedqr_plus_fingerprint_plus_path_passes_v0_19_0, validate_seedqr_plus_xpub_still_conflict).
- `SECRET_SLOT_SUBKEYS` taxonomy + `declare_slot_subkey_variants!` macro updated to include `seedqr`; the existing secret-taxonomy parity test continues to pass without modification.

### Changed

- `cmd/seedqr.rs::map_seedqr_error` promoted from private fn to `pub(crate)` so the bundle / verify-bundle / export-wallet consumer sites can reuse the canonical `"seedqr: {action}: {e}"` mapping. R0 C3 fold.

### Documentation

- `docs/manual/src/40-cli-reference/41-mnemonic.md` — `--slot` rows on chapter §`mnemonic bundle`, §`mnemonic verify-bundle`, §`mnemonic export-wallet` all updated to enumerate the new `seedqr` token + its inline-decode semantics.

### Test totals

- 2150 cells passing; 12 ignored. +15 net (vs v0.31.2 baseline 2135).

### Cycle topology

Cycle 10 is the FIRST cycle of the v0.32+ tier (the post-v0.28+-residual queue). Wave structure for v0.32+ remains to be planned:
- Wave D (this cycle): one big new surface — picked: `seedqr-bundle-slot-integration`.
- Remaining: 9 other v0.32+ FOLLOWUPs filed during cycles 5-9 — `sparrow-import-detection-regex-defensive-widening` (hardening), `seedqr-compact-variant` (CompactSeedQR), `seedqr-15-18-21-word-counts`, `seedqr-digits-from-input-unification`, `electrum-crypto-seed-extraction-subcommand`, `wallet-import-electrum-encrypted-storage-format-b`, `bsms-encryption-per-signer-tokens`, `bsms-encryption-round1-decrypt-then-verify`, `bsms-encryption-cross-impl-coinkite-python-smoke`.

### R0/R1 review history

Plan-doc opus R0 RED 3C/2I/2M:
- C1: SlotSubkey ordering inverted (would have produced `[Path, Seedqr]` instead of `[Seedqr, Path]`); folded by placing Seedqr at enum position 1.
- C2: Branch placement (placed AFTER Phrase, BEFORE Xpub per fold).
- C3: `map_seedqr_error` is private; promoted to `pub(crate)`.
- I1: SemVer-MINOR rationale was wrong (GUI schema_mirror gate compares flag-NAME parity NOT value-content per memory `v0.28+ Wave 3 SHIPPED R0 I1`); user picked PATCH v0.31.3.
- I2: Refusal-matrix cell `bundle_seedqr_slot_double_stdin_refused` added.
- M1: Byte-equal assertion on both 12-word AND 24-word happy paths.
- M2: master_xpub clap-help drift fixed inline via touch-and-fix.

R1 GREEN after fold (no new Critical / Important issues). End-of-cycle opus review GREEN pre-tag. Reports persisted to `design/agent-reports/v0_32_0-plan-doc-r0-review.md` (filed under intended SemVer-MINOR name before the I1 PATCH pivot), `design/agent-reports/v0_31_3-plan-doc-r1-review.md`, and `design/agent-reports/v0_31_3-end-of-cycle-review.md`.

### Toolkit-only

No clap surface (i.e. flag-name) change; only a new `--slot` value-enumeration token. The GUI schema_mirror gate (which compares clap flag-NAME parity, not value-content) does NOT fire. Optional GUI help-text mirror tracked as a separate follow-on FOLLOWUP `gui-seedqr-slot-subkey-help-mirror`.

---

## mnemonic-toolkit [0.31.2] — 2026-05-21

**SemVer-PATCH release.** Behavior expansion: Sparrow taproot SINGLESIG (Bip86 `tr(@0/**)` template-mode) wallets now import successfully via the standard substitution path. Closes Cycle 9 (`sparrow-taproot-singlesig-template-mode-import` FOLLOWUP — the same-session follow-on filed at Cycle 8 close).

### Changed

- **`mnemonic import-wallet --format sparrow`** with Bip86 taproot singlesig wallets (template-mode emit: `tr(@0/**)` per `wallet_export/sparrow.rs:195`) now succeeds. The Step 5 `@N/**` → `[fp/path]xpub/<0;1>/*` substitution loop produces a clean `tr([fp/86'/0'/0']xpub.../<0;1>/*)` descriptor that flows through `concrete_keys_to_placeholders` + `parse_descriptor` per Phase 0 P0 recon (empirically verified at master HEAD `7fa721d`).
- Cycle 8's narrow refusal block at `wallet_import/sparrow.rs::parse` Step 6 (`has_tr && has_at_placeholder` arm) is REMOVED. The path-split discriminator (`is_descriptor_passthrough`) stays: descriptor-passthrough (taproot MULTISIG) still bypasses Step 5; otherwise template-mode (incl. taproot singlesig) flows through substitution.

### Added

- 2 new sparrow-taproot integration cells: `taproot_singlesig_template_imports_via_substitution` (fixture-driven happy path) + `taproot_singlesig_envelope_blocked_by_wallet_import_taproot_internal_key` (boundary cell documenting the orthogonal `wallet-import-taproot-internal-key` FOLLOWUP that still blocks `--from-import-json` re-emission for ALL taproot envelopes — same boundary that applies to taproot multisig from Cycle 8).
- New fixture `tests/fixtures/wallet_import/sparrow-singlesig-p2tr.json` (Bip86 m/86'/0'/0' singlesig) — closes the p2wpkh / p2sh-p2wpkh / p2tr fixture parity gap.
- Cycle 9 in-file unit test conversion: `parse_p2tr_singlesig_refused` → `parse_p2tr_singlesig_imports_via_substitution`.

### Documentation

- `docs/manual/src/45-foreign-formats.md` §"Taproot import" rewritten to describe BOTH branches (v0.31.1 multisig descriptor-passthrough + v0.31.2 singlesig template-mode substitution). Anchor `#taproot-import-shipped-v0311` preserved (R0 M1 fold). Round-trip note added covering the orthogonal `wallet-import-taproot-internal-key` gap.

### Test totals

- 1097 cells (up from 1095 in v0.31.1; +2 net).

### Cycle topology

Cycle 9 of the v0.28+ residual queue closes the same-session follow-on filed at Cycle 8 close. The v0.28+ residual queue (Cycles 3-9) is fully closed across {`v0.28.5`, `v0.28.6`, `v0.28.7`, `v0.29.0`, `v0.30.0`, `v0.30.1`, `v0.31.0`, `v0.31.1`, `v0.31.2`}.

### R0 review

Opus R0 plan-doc review GREEN (0C / 3I / 3M). Persisted to `design/agent-reports/v0_31_2-plan-doc-r0-review.md`. All 3 Important findings folded inline pre-Phase-2: I1 (sparrow.rs rustdoc + fn-level docstring drift) + I2 (under-specified round-trip cell — converted to orthogonal-boundary cell after empirical discovery that `--from-import-json` refuses ALL taproot envelopes) + I3 (fixture asymmetry — added `sparrow-singlesig-p2tr.json`).

---

## mnemonic-toolkit [0.31.1] — 2026-05-21

**SemVer-PATCH release.** Behavior expansion: Sparrow taproot multisig wallets (`tr-multi-a` / `tr-sortedmulti-a` descriptor-passthrough shape) now import successfully. Closes Cycle 8 (`sparrow-taproot-descriptor-passthrough-import-support` FOLLOWUP) — **the final cycle in the v0.28+ residual queue.**

### Changed

- **`mnemonic import-wallet --format sparrow`** with taproot multisig wallets (Sparrow descriptor-passthrough shape: concrete `[fp/path]xpub` keys embedded in `defaultPolicy.miniscript.script` without `@N/**` placeholders) now succeeds. Previously refused at `wallet_import/sparrow.rs::parse` Step 6 with "taproot scripts are not yet supported".
- Detection heuristic at `sparrow.rs` Step 6: `has_tr && !has_at_placeholder` = descriptor-passthrough (skip Step 5 substitution; feed `script_template` directly through `concrete_keys_to_placeholders` → `parse_descriptor`). Per Sparrow emit-side at `wallet_export/sparrow.rs:215-219`, only `CliTemplate::TrMultiA` / `TrSortedMultiA` currently ship as descriptor-passthrough.
- Taproot SINGLESIG (Bip86: `tr(@0/**)` template-mode) is NOT shipped in v0.31.1 — preserves a narrow refusal with updated stderr template citing the follow-on FOLLOWUP `sparrow-taproot-singlesig-template-mode-import`. The R0 reviewer caught this ambiguity (descriptor-passthrough vs template-mode for taproot) before Phase 2; explicit narrow refusal locks Cycle 8's scope.

### Added

- 6 new integration cells in `tests/cli_import_wallet_sparrow_taproot.rs` covering: tr-multi-a 2-of-3 NUMS happy path + envelope-carries-canonical-descriptor verification + auto-sniff + taproot-singlesig-template-still-refused boundary + 2 no-regression cells (P2WPKH singlesig + wsh-sortedmulti).
- New fixture `tests/fixtures/wallet_import/sparrow-tr-multi-a-nums-2of3.json` (copied from emit-side `tests/export_wallet/sparrow_tr_multi_a_nums_2of3.json` — round-trip-compatible).

### Documentation

- Chapter-45 §"Deferral — taproot import" rewritten as §"Taproot import (shipped v0.31.1)" with the descriptor-passthrough pipeline citation + the narrowing note for taproot singlesig.
- Chapter-45 deferrals-list bullet converted to v0.31.1-shipped strikethrough.

### FOLLOWUP closure

- **Closed:** `sparrow-taproot-descriptor-passthrough-import-support` (resolved by Cycle 8 / v0.31.1).

### Newly filed FOLLOWUPs

- `sparrow-taproot-singlesig-template-mode-import` — Bip86 `tr(@0/**)` template-mode import (Cycle 8 ships descriptor-passthrough only). Tier `v0.31+`.

### Wave 4 closure milestone

This cycle closes the v0.28+ residual FOLLOWUP queue. Cycles 5/6/7/8 + 6a/6b/7a/7b split-cycles all SHIPPED this session-pair. The toolkit's Wave-4 parser-cycle work is complete; future work is queued under v0.32+-tier slugs (per-Signer BSMS tokens, Round-1 encrypted records, taproot singlesig, etc.).

### Tests

- 71 lib + 743+ integration cells (incl. 6 new sparrow-taproot); clippy clean; manual lint 6/6 PASS.

---

## mnemonic-toolkit [0.31.0] — 2026-05-21

**SemVer-MINOR release.** New `--bsms-encryption-token <FILE|->` flag on `mnemonic import-wallet` for BIP-129 §Encryption envelope decrypt. Closes Cycle 7 (`bsms-bip129-encryption-envelope` FOLLOWUP).

### Added

- **`mnemonic import-wallet --bsms-encryption-token <FILE|->`** — BIP-129 encryption-envelope Round-2 decrypt. Reads session TOKEN from PATH (or `-` for stdin); applies PBKDF2-SHA512(`b"No SPOF"`, TOKEN_raw, 2048, 32) → ENCRYPTION_KEY → SHA256(EK) → HMAC_KEY → AES-256-CTR (Ctr128BE; full 16-byte IV as 128-bit BE counter) + HMAC-SHA256 verify per BIP-129 §Encryption. Combine with `--format bsms`. Token width: 16 hex chars STANDARD (8 raw bytes) or 32 hex chars EXTENDED (16 raw bytes). Encrypted blobs lack the `BSMS 1.0` header so `--format bsms` is REQUIRED for the encrypted path. Stdin-contention guard refuses dual `--blob=- + --bsms-encryption-token=-`.
- New `BsmsMacMismatch { token_len_hex }` `ToolkitError` variant (typed per FOLLOWUP body recommendation; exit 2). Alphabetical insertion BEFORE `BsmsRound1Malformed`. Stderr template: `error: import-wallet: bsms: BIP-129 MAC verification failed (token width N hex chars; wrong token or tampered ciphertext)`.
- Stderr NOTICE on successful decrypt: `notice: import-wallet: bsms: BIP-129 encrypted Round-2 envelope decrypted (token width N hex chars; MAC verified)`.
- New library module `mnemonic_toolkit::bsms_crypto` (shipped pre-tag in Cycle 7a `62da111`): pub `derive_encryption_key` / `derive_hmac_key` / `compute_mac` / `decrypt` / `encrypt` + library-local `BsmsCryptoError`. 20 unit cells incl. BIP-129 TV-3 cross-validation.
- New Cargo dep `ctr = "0.9"` (added Cycle 7a; sibling of `cbc` from RustCrypto block-modes family).
- 12 new integration cells in `tests/cli_import_wallet_bsms_encrypted.rs`.
- New fixtures: `bsms-encrypted-standard-tv3.dat` + `bsms-encrypted-standard-tv3-token.hex` (BIP-129 §Test Vectors STANDARD-mode Signer 1 wire + token).

### Documentation

- Chapter-41 `mnemonic import-wallet` flag table gains `--bsms-encryption-token` row.
- Chapter-41 stderr-templates table gains BIP-129 decrypt NOTICE + MAC-mismatch Error rows.
- Chapter-45 §"BSMS encrypted envelopes" deferral converted to v0.31.0-shipped strikethrough with cross-impl-vs-BIP-129-TV-3 citation.

### Architectural notes

Cycle 7 executed as two-session split (7a: library + R0 + recon; 7b: CLI + parser integration + ship). Cycle 7a opus R0 caught the `Ctr64BE` vs `Ctr128BE` critical pre-implementation. Cycle 7b opus R0 (YELLOW 2C/7I/4M) caught the orchestrator-insertion-site mismatch + `BsmsMacMismatch` alphabetical-slot off-by-one before integration. Both R0 cycles paid off the discipline.

Library-side primitives verified byte-exact against BIP-129 + Coinkite Python ref. TV-3 cross-validation locked in unit cells: ENCRYPTION_KEY=`7673ffd9…`, HMAC_KEY=`3d4c4228…`, MAC=`fbdbdb64…`, IV=`fbdbdb64…` (first 16 of MAC). The full 304-hex-char TV-3 wire is the load-bearing integration fixture.

NOTE on TV-3 plaintext shape: BIP-129 TV-3 is a Round-1 KEY record (5-line). The current `BsmsParser` handles Round-2 (4-line/6-line). The decrypt-success-then-parser-refusal boundary is documented in `tv3_decrypt_emits_notice_advisory`. A future cycle (`bsms-encryption-round1-decrypt-then-verify`) adds Round-1 decrypt-then-verify integration.

### FOLLOWUP closure

- **Closed:** `bsms-bip129-encryption-envelope` (resolved by Cycle 7 / v0.31.0; canonical entry).
- **Closed (cross-cite):** `wallet-import-bsms-encrypted` (v0.27+ predecessor; superseded by the canonical entry).

### Newly filed FOLLOWUPs

- `bsms-encryption-per-signer-tokens` — per-Signer TOKEN variants (BIP-129 line 74 allows per-Signer or shared TOKEN; Cycle 7b ships shared-TOKEN only). Tier `v0.31+`.
- `bsms-encryption-round1-decrypt-then-verify` — encrypted Round-1 KEY records (Cycle 7b ships encrypted Round-2 only). Tier `v0.31+`.
- `bsms-encryption-cross-impl-coinkite-python-smoke` — automated cross-impl test against Coinkite Python ref (Cycle 7b cross-checks against the locked recon-dossier values; automated cross-impl smoke is a separate test harness). Tier `v0.31+`.

### Tests

- 71 lib + 743 integration cells (incl. 12 new + 20 Cycle 7a bsms_crypto); clippy clean; manual lint 6/6 PASS.

---

## mnemonic-toolkit [0.30.1] — 2026-05-21

**SemVer-PATCH release.** Behavior expansion: encrypted Electrum wallets (`use_encryption: true`) now import as watch-only instead of refusing at parse time. Closes Cycle 6 (`wallet-import-electrum-encrypted` FOLLOWUP, resolved as watch-only-passthrough per Cycle 6b R0 fold).

### Changed

- **`mnemonic import-wallet --format electrum`** with `use_encryption: true` wallets now succeeds, emitting a stderr NOTICE advisory and importing only the plaintext watch-only material (`keystore.{xpub,derivation,root_fingerprint,label}` + multisig analogues). Previously refused at parse time. The encrypted fields (`keystore.{seed,xprv,passphrase,keypairs}`) are ignored.
- Per Electrum's `electrum/keystore.py`, the field-level encryption protects only seed-material fields; watch-only fields are plaintext under both encrypted and unencrypted wallets. The pre-v0.30.1 refusal was over-restrictive in principle.
- Stderr advisory text: `"notice: import-wallet: electrum: wallet is encrypted (use_encryption=true); importing watch-only material only (encrypted seed/xprv/passphrase/keypairs fields ignored). To extract the encrypted seed, use 'electrum --decrypt-wallet' out-of-band then re-import the plaintext wallet."`

### Added

- 4 new integration-test cells in `tests/cli_import_wallet_electrum.rs` (singlesig + multisig watch-only happy paths + auto-sniff + plaintext-no-regression).
- New fixture `tests/fixtures/wallet_import/electrum-encrypted-watch-only-multisig-2of3.json`.
- Manual chapter-45 §"Encrypted wallets" rewritten across 3 stale-deferred sites; chapter-41 stderr-templates table gains a NOTICE row.

### Architectural pivot (Cycle 6b R0 fold)

The Cycle 6a brainstorm (`design/BRAINSTORM_v0_31_0_electrum_encrypted_v1_path_b.md`, ARCHIVED) assumed the toolkit needed to decrypt `seed`/`xprv` fields. Cycle 6b opus R0 review caught that the Electrum parser reads ONLY plaintext `xpub`/`derivation`/`fingerprint`/`label` — encrypted fields are NEVER consumed. The `--decrypt-password*` flag family (3-form: `--decrypt-password VAL` + `--decrypt-password-file PATH` + `--decrypt-password-stdin`) and supporting machinery were dropped. The 6a-shipped `electrum_crypto.rs` library stays in-tree as an internal utility for a future seed-extraction subcommand (filed forward as FOLLOWUP `electrum-crypto-seed-extraction-subcommand`). No CLI surface change in v0.30.1 → no GUI lockstep.

### Renamed

- `tests/fixtures/wallet_import/electrum-encrypted-refused.json` → `electrum-encrypted-watch-only-singlesig.json` (xpub now a real plaintext value reused from the existing standard-bip84-mainnet fixture; seed/xprv kept as placeholder base64).

### FOLLOWUP closure

- **Closed (resolved-watch-only-passthrough):** `wallet-import-electrum-encrypted`. The FOLLOWUP body's pre-v0.30.0 "PBKDF2 + AES-CBC" scheme citation was wrong; corrected to "sha256d + AES-256-CBC" per Cycle 6 P0 recon §A1.

### Newly filed FOLLOWUPs

- `electrum-crypto-seed-extraction-subcommand` — future use case for the 6a-shipped `electrum_crypto.rs` library (e.g., `mnemonic convert --from electrum-encrypted-wallet --to phrase` or a dedicated subcommand). Tier `v0.31+`.
- `wallet-import-electrum-encrypted-storage-format-b` — Electrum's Format B whole-file storage encryption (version-byte + AES-CBC + 4-byte MAC). NOT JSON-parseable; out of scope of Cycle 6's Format A focus. Tier `v0.31+`.

### Note

Cycle 6 of v0.28+ residual FOLLOWUP release plan, executed as two-session split (6a: library + design; 6b: R0 fold + watch-only-passthrough + ship). Opus brainstorm R0 caught a foundational design error in 6a (RED verdict; Path A pivot). Plan-doc R0 YELLOW (4 mechanical Importants) folded inline. See `design/BRAINSTORM_v0_30_1_electrum_encrypted_watch_only.md` + `design/PLAN_mnemonic_toolkit_v0_30_1.md`.

---

## mnemonic-toolkit [0.30.0] — 2026-05-21

**SemVer-MINOR release.** New top-level `mnemonic seedqr` subcommand for SeedQR encode/decode. Paired with `mnemonic-gui-v0.15.0` (schema-mirror lockstep). Cycle 5 of v0.28+ residual FOLLOWUP release plan.

### Added

- **`mnemonic seedqr decode|encode`** — new top-level subcommand for SeedQR encode/decode. SeedQR is an open spec originated by SeedSigner: BIP-39 mnemonic encoded as a numeric-string QR payload where each English-wordlist index is rendered as a 4-digit zero-padded decimal.
  - `seedqr decode --digits <VALUE|->` reads a 48 or 96 ASCII-digit SeedQR string, validates BIP-39 checksum, emits the BIP-39 phrase.
  - `seedqr encode --from phrase=<VALUE|->` reads a 12- or 24-word English BIP-39 phrase, emits the SeedQR numeric string.
  - Both subsubcommands support `--json-out <PATH>` (envelope: `schema_version: "1"`, `operation: "decode"|"encode"`, `variant: "standard"`, `word_count`, `phrase`, `digits`).
  - **Scope (v0.30.0):** Standard variant only; 12 + 24 words only; English-locked. CompactSeedQR + 15/18/21-word counts + bundle-slot integration filed as FOLLOWUPs.
  - **Exit code:** all `SeedqrError` variants map to `ToolkitError::BadInput` (exit 1).
- New library module `mnemonic_toolkit::seedqr` with `decode()` / `encode()` primitives + library-local `SeedqrError` enum (no new `ToolkitError` variants; mapped via `cmd::seedqr::map_seedqr_error` at the CLI boundary per `lib.rs:14-28` documented pattern matching `final_word` / `seed_xor` / `slip39`).
- `secrets.rs::flag_is_secret` extended to include `"--digits"` (unconditionally secret).
- Secret-memory hygiene applied to `cmd/seedqr.rs` mirroring `cmd/seed_xor.rs:163-178`: `Zeroizing<String>` on phrase/digits buffers, `mlock::pin_pages_for` page pins, `secret_in_argv_warning` advisories for inline-form input.

### Documentation

- New `## mnemonic seedqr` section in manual chapter-41 (`docs/manual/src/40-cli-reference/41-mnemonic.md`). Covers synopsis, flags, scope, worked examples (12-word + 24-word + JSON envelope), cross-impl smoke recipe vs SeedSigner Python reference at `src/seedsigner/models/encode_qr.py::SeedQrEncoder`, exit codes, stderr templates.
- Chapter-45 `### Deferral — SeedQR` rewritten as `### SeedQR (Jade + SeedSigner + others)` redirecting users to the new subcommand. Chapter-45 "Jade SeedQR variant" bullet updated with strike-through redirect.

### Architectural pivot

Predecessor brainstorm filed this cycle under FOLLOWUP slug `wallet-import-jade-seedqr` with the assumption that SeedQR ingest would extend the `wallet-import` surface. Cycle 5 pivots on two findings: (a) SeedQR carries a BIP-39 seed (not a wallet policy), so the wallet-import envelope is the wrong surface — forcing it through requires synthetic empty-policy `ParsedImport`; (b) SeedQR is an open spec used by multiple vendors (SeedSigner originated; Jade / Coldcard / Cobo / Krux adopted), so the slug should not be vendor-named.

### FOLLOWUP closure

- **Closed (resolved-superseded):** `wallet-import-jade-seedqr` — superseded by new vendor-neutral slug `seedqr-encode-decode-subcommand` (Cycle 5; v0.30.0).

### Newly filed FOLLOWUPs

- `seedqr-compact-variant` — CompactSeedQR ingest (raw entropy bytes; 16/32 bytes; ambiguity-handling via explicit `--variant compact --word-count` flag).
- `seedqr-15-18-21-word-counts` — 15/18/21-word BIP-39 phrases (60/72/84 digits).
- `seedqr-bundle-slot-integration` — `mnemonic bundle --slot @N.seedqr=<file>` auto-decode at slot-emit.
- `seedqr-digits-from-input-unification` — long-term surface unification: extend `FromInput` with `seedqr=<value>` node type; deprecate `--digits`.

### Tests

- 2025 → ~2057 (+~32 from `tests/cli_seedqr.rs` + 18 unit cells in `src/seedqr.rs`).
- All 113 test result groups PASS; `cargo clippy --all-targets --workspace -- -D warnings` clean.

### GUI lockstep

Paired tag `mnemonic-gui-v0.15.0`: pin bump v0.29.0 → v0.30.0; schema-mirror gains two new `SubcommandSchema` entries (`seedqr-encode` + `seedqr-decode`) placed between `seed-xor-combine` and `slip39-split` per verb-ordering convention (create-side before recover-side).

### Note

Cycle 5 of v0.28+ residual FOLLOWUP release plan. See `design/BRAINSTORM_v0_30_0_seedqr.md` + `design/PLAN_mnemonic_toolkit_v0_30_0.md` + `design/cycle-5-p0-recon.md`. Opus brainstorm R0 YELLOW (2C/8I/5M) → R1 GREEN. Opus plan-doc R0 RED (4C/6I/5M) → R1 GREEN-with-fix → folded inline.

---

## mnemonic-toolkit [0.29.0] — 2026-05-21

**SemVer-MINOR release.** Driver: xpub-search result wire-shape replacement (struct → tagged enum). Paired with `mnemonic-gui-v0.14.0` (downstream wire-shape consumer).

### Wire-shape break (SemVer-minor)

- **`xpub-search-result-type-level-invariant-blocked-on-wire-shape-evolution`** — `PathOfXpubResult`, `PassphraseOfXpubResult`, and `AccountOfDescriptorResult` are now `#[serde(tag = "result", rename_all = "snake_case")]` tagged enums with `Match { ... }` and `NoMatch { ... }` variants. Consumers checking `.path === null` (or similar null-on-no-match patterns) break — the `path` / `template` / `account` keys are absent on `no_match` rather than null. Discriminator field name preserved as `"result"` (`"match"` / `"no_match"`). 3 v0.27.0 envelope drift cells marked `#[ignore]` with SemVer rationale.

### Refactors (no wire-shape impact)

- **`pr-26-import-provenance-three-variant-cleanup`** — Split `ImportProvenance::Bsms(Option<BsmsAuditFields>)` → `BsmsSixLine(BsmsAuditFields)` + `BsmsTwoLine` (unit variant). P0 STRICT-GATE locked this as a 1-variant split (NOT the FOLLOWUPS body's stale "3-variant" framing). All 7 accessor match blocks + 5 test cells + 1 construction site updated.

- **`error-rs-retroactive-alphabetical-sort`** — Pure reorder: 44 `ToolkitError` variants sorted alphabetically; ~132 arm reorders across `Display`, `exit_code`, `kind` exhaustive match blocks + 1 partial-match `details`. `exit_code` multi-variant grouped patterns broken into single-variant arms post-sort (new FOLLOWUP `error-rs-exit-code-arm-fragmentation-post-sort` for future readability pass). Shipped as a separate commit on the same branch (per bisect-hygiene lock R0-I3).

### Tests

- 2028 → 2025 (-3 net, 3 cells `#[ignore]`-gated for SemVer rationale on v0.27.0 envelope drift fixtures).
- `gui-schema` JSON byte-identical between v0.28.7 and v0.29.0 — confirms zero clap surface drift; the wire-shape break is in serde output only, not in CLI flag definitions.

### GUI lockstep

Paired tag `mnemonic-gui-v0.14.0`: pin bump v0.28.4 → v0.29.0; schema-mirror unchanged (clap surface unchanged; flag-name parity holds). The new FOLLOWUP `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification` documents that GUI's runtime consumers of xpub-search JSON have NO automated drift gate.

### Note

Cycle 4 of v0.28+ residual FOLLOWUP release plan (Wave 3 SemVer-minor cliff). See `design/BRAINSTORM_v0_28_plus_residual_followups.md` + `design/PLAN_mnemonic_toolkit_v0_29_0.md` + `design/cycle-4-p0-recon.md`. Opus plan-doc R0 review YELLOW (4 Important folded inline) → R1 GREEN → end-of-cycle GREEN (0C/0I/0M).

---

## mnemonic-toolkit [0.28.7] — 2026-05-20

Patch release: 4 hardening FOLLOWUPs from the post-A/B/C residual backlog (Wave 2).

### Imports / Exports — defect refusal hardening

- **`bsms-import-taproot-refusal-parity`** — Add import-side parity of `BsmsTaprootRefused`. New variant `BsmsTaprootImportRefused` (no `script_type` field — import parser has no `WalletScriptType` in scope at parse time; alphabetically inserted BEFORE `BsmsTaprootRefused` per CLAUDE.md). BSMS parser now short-circuits on `tr(` substring at parse-entry, mirroring emit-side refusal. Defense-in-depth: `extract_threshold` now refuses `sortedmulti_a(` / `multi_a(` substrings. User-visible: `mnemonic import-wallet --format bsms ... <taproot blob>` now exits 2 with explanatory message + FOLLOWUP slug reference.

- **`green-emitter-multisig-refusal-template-only`** — Refactor green emitter's multisig refusal from `inputs.template.is_some() && t.is_multisig()` → `inputs.script_type.is_multisig()`. Closes the bug where descriptor-mode (`--from-import-json`) multisig green exports silently passed despite Green's import surface being singlesig-only. New `WalletScriptType::is_multisig()` method covers `P2shMulti | P2shP2wshMulti | P2wshMulti | P2trMulti`. Anti-pattern survey at P0 recon: isolated to green.rs; no other emitters share the same bug.

- **`wallet-import-format-mismatch-matrix-completion`** (Option B narrow set) — Extend BSMS / BitcoinCore / ColdcardMultisig dispatch arms in `cmd/import_wallet.rs` to refuse all 17 missing sniff outcomes via `ImportWalletFormatMismatch`. New matrix test file `tests/cli_import_wallet_format_mismatch_matrix.rs`. NOTE: P0 recon discovered the original FOLLOWUPS scope was structurally narrower than actual residuals — Coldcard / Sparrow / Specter / Electrum arms also have residual gaps. Those 10 discovered gaps are filed as NEW FOLLOWUP `wallet-import-format-mismatch-matrix-completion-discovered-gaps`.

- **`wallet-import-taproot-internal-key`** (Fix-α envelope-gate refusal) — Refuse taproot envelopes at the single `EmitInputs` construction gate in `cmd/export_wallet.rs:run_from_import_json` via `matches!(script_type, WalletScriptType::P2tr | WalletScriptType::P2trMulti)` (parse-side detection, not string-sniff). P0 recon confirmed Framing B (envelope-gate-only) over Framing A (per-exporter fan-out); all 8 `wallet_import/*.rs` parsers are uniformly taproot-agnostic. Fix-β (envelope wire-shape evolution to carry the field) remains open for v0.29+.

### Tests

- 4 slug closures contribute +20 net cells: +1 Slug 1 (sortedmulti_a regex side-channel; 1 cell renamed), +1 Slug 2 (descriptor-mode multisig refusal), +17 Slug 3 matrix, +1 Slug 4 envelope-refusal multi-format. Total: 2008 → 2028.
- 2 canary tests flipped to match new (correct) behavior: `p11c_green_descriptor_passthrough_singlesig_passes_multisig_refused` + `core_sniff_smoke` (exit code 1 vs 2 post-Slug-3).

### Note

Cycle 3 of v0.28+ residual FOLLOWUP release plan (Wave 2 hardening). See `design/BRAINSTORM_v0_28_plus_residual_followups.md` + `design/PLAN_mnemonic_toolkit_v0_28_7.md` + `design/cycle-3-p0-recon.md`. Opus end-of-cycle review GREEN (0 critical / 0 important; 1 minor + 1 new FOLLOWUP filed).

No CLI surface change; no wire-shape change; no GUI lockstep.

---

## mnemonic-toolkit [0.28.6] — 2026-05-20

Patch release: 2 test-hygiene FOLLOWUPs from the post-A/B/C residual backlog.

### Tests

- **`cross-format-refusal-matrix-include-coldcard-multisig`** — Extend the `tests/cli_export_wallet_from_import_json.rs` refusal-matrix coverage to include the v0.28.4-added `--format coldcard-multisig` export variant. `TEMPLATE_ONLY_DESTS` grows to 5 entries; `REFUSAL_STDERR_PATTERNS` broadened to match the `"requires a multisig --template"` refusal substring (the v0.28.4 multisig-template precheck text); cell-count assertion bumped 32 → 40 (8 sources × 5 dests). Closes the FOLLOWUP filed in v0.28.4 cycle commit `826efbc`.

- **`coldcard-legacy-mk1-mk2-top-level-xpub-inference`** — Legacy mk1/mk2 Coldcard `wallet.json` fallback parser (already implemented in commit `1304932` from v0.28.0 P3-v2 cycle) now has fixture + test coverage. 3 new fixtures in `tests/fixtures/wallet_import/coldcard-mk1-legacy-bip{44,49,84}-mainnet.json` carry the canonical SLIP-132 published test vectors (xpub/ypub/zpub from the spec's "Bitcoin Test Vectors" section); 4 new test cells in `tests/cli_import_wallet_coldcard.rs` exercise the `infer_bip_from_xpub_prefix` SLIP-132 mapping (BIP-44/49/84 happy paths + 1 unrecognized-prefix refusal). Total toolkit cells: 2004 → 2008.

### Note

Cycle 2 of the v0.28+ residual FOLLOWUP release plan (see `design/BRAINSTORM_v0_28_plus_residual_followups.md`). Wave 1 second ship. No CLI surface change; no toolkit src changes; no GUI lockstep.

---

## mnemonic-toolkit [0.28.5] — 2026-05-20

Patch release: 2 doc-only fixes closing v0.28+ FOLLOWUPs surfaced in the post-A/B/C recon dossier.

### Documentation

- **`design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md` §6.3 step 4** — Replace nonexistent `--ms1` flag (which doesn't exist on the `bundle` subcommand) with `--slot @0.phrase=` per `mnemonic bundle --help`. Closes FOLLOWUP `plan-smoke-step4-ms1-on-bundle-not-supported`.

- **`cmd/import_wallet.rs:87 + :975`** — Add cross-reference doc-comments at both `schema_version` constant sites (outer envelope `"1"` + inner BundleJson `"4"`). The two constants share the name but evolve independently; comments now make the disambiguation explicit at-site. Closes FOLLOWUP `import-wallet-envelope-schema-version-narrative-drift`.

### Note

Cycle 1 of the v0.28+ residual FOLLOWUP release plan (see `design/BRAINSTORM_v0_28_plus_residual_followups.md`). Wave 1 first ship. No CLI surface change; no test cell changes; no GUI lockstep.

---

## mnemonic-toolkit [0.28.4] — 2026-05-20

Patch release: closes the `--format coldcard-multisig` asymmetry between `import-wallet` (accepts both `coldcard` and `coldcard-multisig`) and `export-wallet` (previously only accepted `coldcard`). The new `CliExportFormat::ColdcardMultisig` variant aliases the existing `Coldcard` dispatch with a multisig-template precheck: singlesig templates (`bip44`/`bip49`/`bip84`) refuse with a pointer to `--format coldcard`; multisig templates (`wsh-sortedmulti`/`wsh-multi`/`sh-wsh-*`/`tr-*-a`) delegate to the same `ColdcardEmitter::emit` path that `--format coldcard` already uses. Closes FOLLOWUP `export-wallet-coldcard-multisig-alias`. Paired with `mnemonic-gui-v0.13.0` for schema-mirror lockstep.

### Added

- `--format coldcard-multisig` value on `mnemonic export-wallet` (and `mnemonic export-wallet --from-import-json -`). Refuses singlesig templates with pointer text to `--format coldcard` for SS export.

### Changed

- chapter-45 § Coldcard multisig § "Format-name asymmetry note" prose rewritten to "Format-name parity (v0.28.4+)" with the historical-context framing.

### Tests

- 3 new cells in `tests/cli_export_wallet_coldcard.rs` (happy path + 2 refusal paths). Plus `tests/cli_gui_schema.rs:267` `export_wallet` format-dropdown vendor-count assertion bumped 9 → 10 to include the new variant. Total toolkit cells: 2001 → 2004.

### Companion releases

- `mnemonic-gui-v0.13.0` — paired GUI schema-mirror + dropdown wiring update.

---

## mnemonic-toolkit [0.28.3] — 2026-05-20

Patch release: compile-time enforcement of the `EmitInputs.canonical_descriptor` BIP-380 `#<8-char-csum>` suffix invariant via the new `CheckedDescriptor<'_>` newtype in `wallet_export/mod.rs`. Pre-v0.28.3 the invariant was documented at `wallet_export/bsms.rs:86-90` and enforced only by convention at construction sites — a future code path that constructed `EmitInputs` from a stripped-body descriptor would silently regress BSMS L2 + Specter `descriptor` JSON field + Green plaintext (latent class surfaced by F9 in the manual-v0.2.0 audit cycle). Closes FOLLOWUP `emitinputs-canonical-descriptor-checksum-invariant-enforcement`. No CLI surface change; no GUI lockstep.

### Added

- `CheckedDescriptor<'a>(&'a str)` newtype in `wallet_export/mod.rs` with `new() -> Result<Self, ToolkitError>` constructor that validates the BIP-380 `#<8-char-csum>` suffix (missing-`#` / wrong-length / non-alphanumeric all return `BadInput`). Carries `Deref<Target = str>` + `Display` impls so existing consumer code continues to work via auto-deref.

### Changed

- `EmitInputs.canonical_descriptor` field type from `&'a str` → `CheckedDescriptor<'a>` (compile-time invariant guarantee).
- 2 construction sites in `cmd/export_wallet.rs` (the `--template`/`--descriptor` path at L438 and the `--from-import-json` path at L609) wrap via `CheckedDescriptor::new(...)?` before `EmitInputs` construction.
- 5 consumer-site adjustments where `Deref` auto-coerce didn't fire automatically (`bip388.rs:47`, `bitcoin_core.rs:26`, `bsms.rs:103`, `sparrow.rs:216` with explicit `let desc: &str = &inputs.canonical_descriptor` annotation, `specter.rs:68`).
- `wallet_export/bsms.rs:86-90` invariant comment updated from "by convention" to "by type / compile-time-guaranteed".

### Tests

- 5 new inline unit cells in `wallet_export/mod.rs#[cfg(test)] mod checked_descriptor_tests` (mirrors `bsms.rs:219` convention): constructor positive + 3 negative paths (missing `#`, wrong-length, non-alphanumeric) + `Deref`-coercion compat. Total toolkit cells: 1996 → 2001.

---

## mnemonic-toolkit [0.28.2] — 2026-05-20

Patch release: `export-wallet --from-import-json` BSMS / Specter / Green emitters now carry the BIP-380 `#<8-char>` checksum on the descriptor surface, restoring the `EmitInputs.canonical_descriptor` invariant documented at `wallet_export/bsms.rs:86-90`. Pre-fix, the `--from-import-json` path stripped the checksum via `descriptor_body_no_csum` and passed the body verbatim into emitters that expect the canonical form; downstream BSMS coordinators (Coldcard Mk4) reject Round-2 blobs whose descriptor line lacks `#checksum`. Surfaced by the `manual-v0.2.0` content-audit cycle finding F9 (P1b R1 classification at `design/agent-reports/manual-v0_2_0-p1b-r1-classification.md`). No CLI surface change; no GUI lockstep required.

### Fixed

- `export-wallet --from-import-json --format bsms` 4-line Round-2 output's L2 descriptor now carries the BIP-380 `#<8-char>` checksum (was missing pre-v0.28.2). Same fix simultaneously cures the latent class in `--format specter` (`descriptor` JSON field) and `--format green` (plaintext output line). The fix is at `cmd/export_wallet.rs:566-598` — the strip-validate-then-reparse step now re-emits via miniscript's canonical `Descriptor::Display` (which always appends `#<csum>` per BIP-380 §Checksum-on-emit) before constructing `EmitInputs`.

### Tests

- 2 new regression cells in `tests/cli_export_wallet_from_import_json.rs` — `f9_from_import_json_bsms_l2_carries_bip380_checksum` (BSMS 4-line shape; asserts L2's `#<8-char>` suffix) and `f9_from_import_json_specter_descriptor_carries_bip380_checksum` (Specter JSON field; latent class regression guard). Total toolkit cells: 1994 → 1996.

---

## mnemonic-toolkit [0.28.1] — 2026-05-20

Patch release: cosmetic-only bugfix in the `bundle --import-json` stderr cosigner-summary block. Underlying md1 + mk1 + ms1 strings were always correct (verifiable via `verify-bundle` round-trip); only the human-readable display was wrong. No CLI surface change; no sibling-codec or mnemonic-gui lockstep required.

### Fixed

- `bundle --import-json` stderr `# Threshold:` line + `# Recovery:` line now report the descriptor's true K instead of the cosigner count N. Pre-fix bug: a 2-of-3 multisig wallet imported via `--import-json` rendered `# Threshold: 3 of 3` because `build_unified_card` fell back to N when the `--threshold` CLI arg was None (which it always is on the `--import-json` descriptor-mode path). Fix extracts K from the reassembled descriptor's multi-family node (`Body::MultiKeys` / `Body::Variable`) so every foreign format flowing through this code path renders correctly.

### Tests

- 8 new regression cells in `tests/cli_bundle_import_json.rs` — one per foreign-wallet format on the canonical 2-of-3 multisig fixture (bitcoin-core / bsms / coldcard-multisig / electrum / jade / sparrow / specter) plus a K-not-equal-to-N robustness cell (coldcard-multisig 3-of-5). Total toolkit cells: 1986 → 1994.

---

## mnemonic-toolkit [0.28.0] — 2026-05-20

Headline cycle: 6 new wallet-import format parsers (Sparrow / Specter / Electrum / Coldcard / Coldcard-multisig / Jade) + BSMS BIP-129-canonical 4-line Round-2 input parser + `compare-cost` single-leaf taproot input support. Cross-format conversion matrix grows from a single source→destination cell to a parameterized N×M matrix (74 cells: 24 happy-path + 42 refusal + adjuncts) covering 8 sources × N destinations.

### Added

- 6 new `mnemonic import-wallet --format` parsers:
  - `coldcard` — Coldcard single-sig `wallet.json` (BIP-44 / BIP-49 / BIP-84 / BIP-86 per-path xpub blocks)
  - `coldcard-multisig` — Coldcard multisig text export (descriptor + cosigner list with per-cosigner `Derivation`/xpub blocks)
  - `electrum` — Electrum 4.x plaintext wallet file (singlesig + multisig `x1`/`x2`/... per-cosigner subkeys)
  - `jade` — Blockstream Jade `get_registered_multisig` reply (multisig descriptor + per-cosigner xpub + threshold + signer-fingerprint annotations)
  - `sparrow` — Sparrow Wallet JSON export (singlesig + `sortedmulti` wsh() / sh(wsh()) shapes)
  - `specter` — Specter-DIY JSON descriptor export
- BSMS BIP-129-canonical 4-line Round-2 input parser (SPEC §10) — `BSMS 1.0` / `<descriptor>#<checksum>` / `<path-restrictions>` / `<first-address>` with descriptor cross-validation against path-restrictions + first-address per BIP-129 §Round 2 verify gate. Partial implementation of `bsms-bip129-full-cutover` (sub-items (a)/(b)/(e)); encryption envelope sub-item (c) deferred to v0.28+ as `bsms-bip129-encryption-envelope`.
- `mnemonic compare-cost --descriptor 'tr(IK, M)'` single-leaf taproot input support (SPEC compare-cost v0.28.0 §11). `translate_descriptor` extended to accept `Descriptor::Tr(_)` where the TapTree contains a single leaf-script; multi-leaf TapTree continues to refuse with `UnsupportedWrapper`.
- Cross-format conversion matrix: 24 happy-path + 42 refusal cells in `tests/cli_export_wallet_from_import_json.rs` covering 8 sources × N destinations (closes `cross-format-conversion-matrix-expansion`).
- 8 new Bitcoin Core fixtures + 7 new BSMS fixtures in `tests/fixtures/wallet_import/` (closes `wallet-import-fixture-corpus-expansion`).

### Changed

- BSMS `--format bsms` taproot refusal text now per-script-type discriminated (P2tr / P2trMulti); cites FOLLOWUP `bsms-taproot-emit`; refusal text points users at `--format bitcoin-core` / `--format sparrow` alternatives. Real BSMS taproot emit remains upstream-blocked on BIP-129 §1 prerequisites adding BIP-386.
- CLI `--format` value-set expanded from 2 to 8 values (alphabetical: `bitcoin-core`, `bsms`, `coldcard`, `coldcard-multisig`, `electrum`, `jade`, `sparrow`, `specter`).
- `wallet_import/sniff.rs::sniff_format` dispatch rewrote from 2-bool 2x2 truth-table to N-parser consult-all-then-count pattern (`SniffOutcome` extended with 6 new variants).
- `VENDOR_MARKER_KEYS` exclusion list grew from 5 to 13 entries to cover the new vendor envelope shapes.

### Deprecated

- BSMS 6-line lenient input shape — stderr DEPRECATION NOTICE fires when parsed; planned for removal in a future minor version. Convert to BIP-129-canonical 4-line shape (the new default ingest path).

### Closed FOLLOWUPs (9 resolved + 2 partial-impl sub-deliverables)

Resolved:
- `wallet-import-sparrow` (P1; commit `b20a357`)
- `wallet-import-specter` (P2; commit `8548258`)
- `wallet-import-electrum` (P6; commit `2031609`)
- `wallet-import-coldcard` (P3; commit `1304932`)
- `wallet-import-coldcard-multisig` (P4 instance D; commit `387a709`)
- `wallet-import-jade` (P5 instance E; commit `091a313`)
- `wallet-import-fixture-corpus-expansion` (G3 + H; commits `d7a2859` + `2a803e8`)
- `cross-format-conversion-matrix-expansion` (P11; commit `8bf78ff`)
- `compare-cost-single-leaf-tr-input` (P12; commit `78936ab`)

Partial-implementation sub-deliverables (canonical entries stay open; sub-deliverable notes added):
- `bsms-bip129-full-cutover` — sub-items (a) 6-line deprecation + (b) 4-line parser + (e) SPEC/manual coverage shipped at commits `1444c51` + `d18787f`; (c) encryption envelope + (d) drop legacy shapes remain open
- `bsms-taproot-emit` — refusal-scaffold UX improvements shipped at commit `158897f` (P8A+P8B); real emit remains upstream-blocked

### Filed FOLLOWUPs (9 new)

- `bsms-bip129-encryption-envelope` (v0.28+) — STANDARD/EXTENDED encryption envelope carved out of `bsms-bip129-full-cutover` sub-item (c)
- `wallet-import-jade-seedqr` (v0.28+) — SeedQR ingest deferred from P5 per Q1 lock
- `wallet-import-electrum-encrypted` (v0.28+) — encrypted Electrum 4.x ingest deferred from P6 per Q2 lock
- `wallet-import-format-mismatch-matrix-completion` (v0.28+) — symmetric N×N mismatch matrix completion (promoted from cycle-followups tracker)
- `bsms-import-taproot-refusal-parity` (v0.28+) — BSMS import-side tr() refusal + `extract_threshold` regex side-channel finding (promoted from cycle-followups tracker)
- `sparrow-taproot-descriptor-passthrough-import-support` (v0.29+) — Sparrow taproot import via descriptor-passthrough heuristic
- `coldcard-legacy-mk1-mk2-top-level-xpub-inference` (v0.29+) — legacy Coldcard wallet.json top-level xpub support
- `green-emitter-multisig-refusal-template-only` (v0.28+) — Green's multisig refusal misses descriptor-mode invocations
- `import-wallet-envelope-schema-version-narrative-drift` (v0.28+) — outer envelope vs inner BundleJson `schema_version` name collision

## mnemonic-toolkit [0.27.2] — 2026-05-19

Cleanup cycle closing 7 v0.27-tier FOLLOWUPs. Anchored on Phase 5b's deferred `ImportProvenance` enum refactor (tier promoted from `v0.28+` per Shape A approval). Sibling lockstep: mnemonic-gui v0.11.1 ships separately (workflow trigger filter + toolkit pin bump). Zero wire-shape change; patch bump valid.

### Fixed

- **xpub-search address-of-xpub `searched` count semantic clarified** (item 6, doc-only). The aggregate `searched` field on `ToolkitError::XpubSearchNoMatch` reports **candidate-comparisons performed** (`n_targets × gap_limit × chains`), not unique child-addresses derived. The existing docstring at `error.rs:230-237` previously elided the `n_targets` factor for address mode; restored. Per-target `scanned_external` / `scanned_internal` JSON fields (on `AddressResultJson::NoMatch` entries inside `AddressOfXpubResult.results`) report unique candidates per-target — unchanged. Closes `xpub-search-address-of-xpub-searched-count-semantic`.
- **`mlock_unit::g1_1_single_page_pin_has_page_count_one` no longer flakes under parallel test execution** (item 4). Switched from `vec![0xAAu8; 64]` heap-allocator-luck buffer to `std::alloc::alloc_zeroed` with explicit page-aligned `Layout`. Closes `mlock-g1-1-test-page-alignment-luck`.

### Changed

- **`ParsedImport` internal representation** (item 1, internal refactor). Replaces the representable-invalid `(Option<BsmsAuditFields>, Option<CoreSourceMetadata>)` pair with a single `provenance: ImportProvenance` enum. Variants: `Bsms(Option<BsmsAuditFields>)` (the `Option` accommodates the 2-line BSMS path's no-audit case) and `BitcoinCore(CoreSourceMetadata)` (non-optional). Wire shape unchanged — back-compat accessors (`ParsedImport::bsms_audit()` / `source_metadata()` returning `Option<&_>`) keep envelope JSON emit code structurally identical. Closes `pr-26-import-provenance-enum-internal-refactor` (tier promoted from v0.28+).

### Tests

- **+1 cell** `dispatcher_arm_count_matches_pinned_constant` in `tests/cli_gui_schema_conditional_rules.rs` — regression guard for `build_subcommand_conditional_rules` arm count drift (pinned at 6). Closes `gui-schema-arm-drop-detector`.
- **+4 unit cells** in `wallet_import/mod.rs::provenance_tests` for the new `ImportProvenance` enum + accessors (Bsms-with-audit, Bsms-without-audit, BitcoinCore-with-metadata, accessor-return-shape).
- **Test count:** 1576 → ~1581 toolkit cells.

### Conventions (CLAUDE.md)

- **`enum ToolkitError` alphabetical-by-variant-name ordering** for new variants + new exhaustive match blocks. Pre-v0.27.2 variants not yet sorted — retroactive sort tracked as `error-rs-retroactive-alphabetical-sort` (v0.28+). Closes `error-rs-canonical-ordering-doc`.
- **Per-phase architect-review agent outputs persist verbatim** to `design/agent-reports/<cycle>-phase-N-<round>-review.md` BEFORE the fold-and-commit step. Closes `compare-cost-agent-reports-back-fill`.
- **Plan-doc + spec citations grep-verified at write time** (FOLLOWUPS.md line numbers presumed stale).
- **Reviewer-loop continues after every fold** until 0 Critical / 0 Important.

### Closed FOLLOWUPs (6 toolkit-side)

- `pr-26-import-provenance-enum-internal-refactor` (Phase 2; tier promoted from v0.28+)
- `error-rs-canonical-ordering-doc` (Phase 1.1)
- `compare-cost-agent-reports-back-fill` (Phase 1.2)
- `mlock-g1-1-test-page-alignment-luck` (Phase 1.3)
- `gui-schema-arm-drop-detector` (Phase 1.4)
- `xpub-search-address-of-xpub-searched-count-semantic` (Phase 1.5)

### Filed FOLLOWUPs (2 new)

- `error-rs-retroactive-alphabetical-sort` (v0.28+) — retroactively apply alphabetical ordering to existing ToolkitError variants + match blocks
- `pr-26-import-provenance-three-variant-cleanup` (v0.28+) — promote `Bsms(Option<_>)` to three-variant `BsmsTwoLine` / `BsmsSixLine(BsmsAuditFields)` / `BitcoinCore(_)`

### Sibling repo

- mnemonic-gui v0.11.1 (separate ship) — workflow trigger filter extension (`gui-workflow-trigger-include-release-branches`) + toolkit pin bump v0.26.0 → v0.27.2 + envelope shape smoke cells.

## mnemonic-toolkit [0.27.1] — 2026-05-19

PR-#26 post-merge fold cycle. A 5-agent retrospective audit on v0.26.0 (silent-failure-hunter + comment-analyzer + type-design-analyzer + pr-test-analyzer + code-reviewer) surfaced 19 Important findings across silent-failures, shape-mismatch defaults, comment-rot, test-coverage gaps, and type-design anti-patterns. v0.27.1 folds 5 of the 6 filed FOLLOWUPs in a single patch cycle (the sixth, `compare-cost-single-leaf-tr-input`, ships as filed-only with implementation deferred to a separate SPEC-anchor cycle).

### Fixed

- **`emit_roundtrip_stderr_warning` no longer swallows canonicalize / UTF-8 errors silently** (Phase 1, C1+I7). The function previously returned `Ok(())` on both error arms, suppressing the SPEC §7.4 stderr warning — the only non-`--json` mode feedback that a Bitcoin Core blob isn't round-tripping byte-exactly. The fold emits explicit diagnostics: `warning: import-wallet: roundtrip check skipped: canonicalize_bitcoin_core failed: <ToolkitError>` and `notice: import-wallet: blob is not UTF-8; roundtrip check uses lossy decode`. In `--json` mode, the `roundtrip.canonicalize_failed` envelope branch now carries an additive `error: String` field with the typed `ToolkitError` Display form (SPEC §7.4 v0.27.1 amendment). Closes FOLLOWUP `pr-26-roundtrip-warning-suppression`.
- **Bitcoin Core `active`/`internal` shape-strictness** (Phase 2, I4). The previous `.and_then(.as_bool).unwrap_or(false)` pattern silently flipped non-bool inputs (`"active": "true"`, `1`, etc.) to `false`, producing a misleading downstream `--select-descriptor active-receive` "no active-receive descriptor found" error. The fold distinguishes "absent" (default false) from "shape-wrong" (typed `ImportWalletParse` error with pointer text) via a new `parse_bool_field` helper that mirrors `parse_range_field`'s strictness. Part of FOLLOWUP `pr-26-shape-mismatch-silent-defaults`.
- **`mk1_card_to_resolved_slot` fingerprint substitution NOTICE** (Phase 2, I5). The `card.origin_fingerprint.unwrap_or_else(|| card.xpub.fingerprint())` substitution is now accompanied by a stderr NOTICE naming the slot index, the substituted hex, and a "downstream wallets may show mismatched origins" warning. Closes the self-confessed `let _ = slot_idx; // reserved for future error-context attribution` gap. Part of FOLLOWUP `pr-26-shape-mismatch-silent-defaults`.
- **`extract_threshold` u8 overflow surface** (Phase 2, I6). Return type changes from `Option<u8>` to `Result<Option<u8>, ToolkitError>`. `None` case = no `thresh()` token found; `Err` case = u8 overflow (`thresh(256, …)`) with pointer text. Previously, u8 overflow silently rendered as `"threshold": null` in the envelope, presenting a malformed input as a "no-threshold" descriptor. Applied symmetrically in `wallet_import/bsms.rs` and `wallet_import/bitcoin_core.rs`. Part of FOLLOWUP `pr-26-shape-mismatch-silent-defaults`.

### Changed

- **`mnemonic import-wallet --format bitcoin-core` rejects malformed `active`/`internal` JSON shape with exit 1** instead of silently defaulting to `false`. v0.26.0 / v0.27.0 consumers feeding `"active": "true"` (string) or `"active": 1` (number) saw a "no active-* descriptor found" error downstream; v0.27.1 now surfaces an upfront `ImportWalletParse` error citing the field name. Behavior change in a previously-undefined edge surface (the SPEC always required boolean per Bitcoin Core's own JSON schema); consumer impact is limited to malformed inputs.
- **`mnemonic import-wallet --json` envelope `roundtrip.canonicalize_failed` branch carries `error: String`** as an additive field. v0.26.0 / v0.27.0 consumers parsing `byte_exact` / `semantic_match` / `status` are unaffected; consumers learning to read `error` see a richer payload only on `canonicalize_failed`. SPEC §7.4 amended in lockstep.
- **Comment-rot sweep** (Phase 3) — citation accuracy across `env_sentinel.rs`, `cost/mod.rs`, `cost/strip.rs`, `error.rs`, `wallet_import/json_envelope.rs`. The user-visible error string at `cost/mod.rs:75` no longer contains internal "Phase 2" cycle vocabulary. The `compare-cost-single-leaf-tr-input` FOLLOWUP slug (already cited in source comments since v0.26.0) was filed in v0.27.0 cycle close at `53a1bf6`. Closes FOLLOWUP `pr-26-comment-rot-fold`.
- **Internal API-discipline scaffolding for xpub-search result types** (Phase 5a). New private builder functions in `cmd/xpub_search/{path_of_xpub,passphrase_of_xpub,account_of_descriptor}.rs` enforce the `result:"match"` ↔ `Some(payload)` correlation at the call site. Fields remain `pub` for wire-shape preservation; the type-level invariant fix (tagged enum + `#[serde(skip_serializing_if)]`) requires a wire-shape change deferred to v0.28+. Tracked by new FOLLOWUP `xpub-search-result-type-level-invariant-blocked-on-wire-shape-evolution`.
- **`BsmsAuditFields.signature_verified: bool` → `BsmsVerification` enum** (Phase 5c). Replaces the prior `(bool, Option<reason>)`-class pair with a closed enum (`NotAttempted | Verified | Failed { reason }`). Wire shape preserved via the `BsmsVerification::signature_verified()` derived getter that emits the legacy `"signature_verified": bool` JSON field. v0.26.0 / v0.27.0 inline 2/6-line parsers always construct as `NotAttempted` (no inline cryptographic verification exists). Mirrors v0.27.0 Phase 6.5 I7's `Round1VerificationStatus` precedent. Part of FOLLOWUP `pr-26-type-design-anti-pattern-sweep`.

### Tests

- **+34 new cells** across Phases 1/2/4/5 (Phase 1: 4, Phase 2: 9, Phase 4: 14, Phase 5: 7 drift cells) + Phase 0 fixture captures. Test count 1542 → 1576.
- 6 captured v0.27.0 envelope fixtures at `tests/fixtures/v0_27_0_envelopes/` pinned forever per plan-doc Q5c discipline (drift guards for future minor cycles).
- New test file `tests/cli_xpub_search_drift_v0_27_0.rs` with 7 drift-regression cells pinning xpub-search result wire shapes against the captured fixtures.

### Closed FOLLOWUPS

- `pr-26-roundtrip-warning-suppression` (Phase 1)
- `pr-26-shape-mismatch-silent-defaults` (Phase 2)
- `pr-26-comment-rot-fold` (Phase 3)
- `pr-26-test-coverage-gap-fold` (Phase 4)
- `pr-26-type-design-anti-pattern-sweep` (Phase 5, partial: 5a + 5c shipped; 5b deferred — see new FOLLOWUP `pr-26-import-provenance-enum-internal-refactor`)

### Filed FOLLOWUPS

- `xpub-search-result-type-level-invariant-blocked-on-wire-shape-evolution` (Phase 5a; v0.28+)
- `pr-26-import-provenance-enum-internal-refactor` (Phase 5b deferral; v0.28+)

## mnemonic-toolkit [0.27.0] — 2026-05-19

The **cross-format wallet conversion** cycle. The toolkit now ingests a BSMS Round-2 blob, mediates it through a canonical envelope, and emits any of the eight supported per-format wallet configs (Bitcoin Core, BIP-388, Coldcard, Jade, Sparrow, Specter, Electrum, Green) — or synthesizes the canonical `ms1`/`mk1`/`md1` engraving cards from the same envelope. v0.27.0 also ships BIP-129 Round-1 BIP-322 ECDSA signature verification (`--bsms-round1`), the first toolkit-side cryptographic Round-1 audit, and a `--format bsms` emitter (4-line BIP-129-canonical default + 2-line lenient).

### Added

- **`mnemonic export-wallet --format bsms`** (Phase 3 `4a2b6e7`) — BSMS Round-2 emitter. The default `--bsms-form 4-line` produces the BIP-129-canonical Round-2 plaintext (4 lines: `BSMS 1.0` / `<descriptor>#<checksum>` / `<path-restrictions>` / `<first-address>`). The lenient `--bsms-form 2-line` emits the v0.26.0 import-side parser's symmetric form (no audit fields). Closes FOLLOWUP `wallet-export-bsms-emitter`. The Phase 3 commit additionally wires first-address verification into `import-wallet --format bsms` (closes FOLLOWUP `bsms-first-address-verify`): a stderr WARNING fires when the blob's declared first-address disagrees with the toolkit-derived address at `m/0/0` (informational; not hard-error per BIP-129 §6 self-consistency intent). Taproot descriptors are explicitly out of scope (FOLLOWUP `bsms-taproot-emit`; v0.28+).
- **`mnemonic import-wallet --bsms-round1 <FILE>`** + **`--bsms-verify-strict`** (Phase 2 `149b341`) — BIP-129 5-line Round-1 key-record parser + BIP-322 ECDSA recoverable-signature verification. Each `--bsms-round1` is a separate file; repeating. Default lenient mode emits a stderr NOTICE on verify failure + sets `signature_verified: false` per-record; `--bsms-verify-strict` makes verify failure fatal (`BsmsSignatureMismatch` exit 2). When supplied alongside `--blob`, the per-record verify state propagates into the `--json` envelope's new `bsms_round1_verifications` field. Standalone mode (no `--blob`) emits a verify envelope on stdout and exits 0 on verify success. Verifies against the xpub's OWN embedded pubkey (bytes 45-78 of serialized xpub) per BIP-129 §Specification → Round 1; supports both raw-pubkey KEYs and xpub KEYs. Closes FOLLOWUP `bsms-verify-signatures`. New recipe chapter `docs/manual/src/30-workflows/3A-bsms-round1-verify.md`.
- **`mnemonic bundle --import-json <FILE|->`** + **`--import-json-index <N>`** (Phase 5 `5bf64a1`) — synthesize a bundle from an `import-wallet --json` envelope rather than from `--template` / `--descriptor`. The envelope's `bundle.descriptor` is the source-of-truth descriptor; `bundle.mk1` chunks decode to per-cosigner xpubs + fingerprints + paths per SPEC §3.6.1. Seed overlay (`--slot @N.phrase=`) applies to slots where envelope `ms1[N] == ""`; supplying overlay for an already-seeded slot is `BadInput`. Multi-entry envelopes (e.g., Bitcoin Core `listdescriptors` with multiple descriptors) require `--import-json-index N`; ambiguous-multi-entry without index is `BadInput` exit 2 (intentional footgun guard).
- **`mnemonic export-wallet --from-import-json <FILE|->`** + **`--from-import-json-index <N>`** (Phase 5 `5bf64a1`) — emit a per-format wallet config from an `import-wallet --json` envelope. The envelope's `bundle.descriptor` becomes the emitter's canonical descriptor; cosigner xpubs decode from `bundle.mk1`; network derives from `bundle.network`. `--account` is rejected (envelope's `bundle.account` is authoritative). Template-only destination formats (Sparrow / Jade / Coldcard / Electrum) surface the existing per-emitter "--template required" refusal because the v0.27.0 envelope is always descriptor-mode. New recipe chapter `docs/manual/src/30-workflows/39-cross-format-conversion.md` walks the BSMS → Bitcoin Core end-to-end pipeline.
- **`mnemonic inspect --json`** schema_version backfill (Phase 1 `e908309`) — the new `InspectEnvelope<'a>` wrapper adds a top-level `schema_version: "1"` field to the `--json` output, mirroring the `XpubSearchEnvelope` precedent. Closes FOLLOWUP `inspect-json-schema-version-backfill`. `mnemonic repair --json` was already shipping `schema_version: "1"` since v0.22.0; no Repair-side change.

### Changed

- **`mnemonic import-wallet --json` envelope wire-shape replacement** (Phase 4 `8ac6847`). The `bundle:` field was a parse-side summary `{cosigners: [{fingerprint, path_raw, xpub, has_entropy}], network, threshold}` in v0.26.0; v0.27.0 emits the full toolkit-native `BundleJson` shape (the same `verify-bundle --bundle-json` consumes), synthesized post-parse via `crate::synthesize::synthesize_descriptor`. A new top-level `schema_version: "1"` field is added. **This is a wire-shape REPLACEMENT (not additive).** Downstream consumers encoding against the v0.26.0 summary shape must be updated. The v0.27.0 envelope is byte-exact-pinned via `crates/mnemonic-toolkit/tests/fixtures/wallet_import/envelope_v0_27_0.json`. Closes FOLLOWUP `wallet-import-json-envelope-full-bundle`. The mnemonic-gui consumer cycle picks up this envelope change separately (sibling repo `pinned-upstream.toml` not bumped by this cycle).
- **`docs/m-format-coordinator-runbook.md`** moved to `design/PLAN_v0_26_0_three_way_merge.md` (Phase 1 `e908309`) per the canonical-record convention. The CLAUDE.md Conventions section is updated to point at the new location. Closes FOLLOWUP `coordinator-runbook-into-design-dir`.

### Closed FOLLOWUPS

- `wallet-export-bsms-emitter` (Phase 3)
- `bsms-first-address-verify` (Phase 3)
- `bsms-verify-signatures` (Phase 2)
- `inspect-json-schema-version-backfill` (Phase 1)
- `coordinator-runbook-into-design-dir` (Phase 1)
- `wallet-import-json-envelope-full-bundle` (Phase 4)

## mnemonic-toolkit [0.26.0] — 2026-05-18

### Added

- **`mnemonic xpub-search passphrase-of-xpub`** (C4) — fourth and final v0.26.0 mode of the `xpub-search` umbrella. Given a seed (BIP-39 phrase OR ms1 card) **plus a specific passphrase** + a target xpub (any SLIP-0132 prefix, or an `mk1...` bech32 card), verify that this passphrase produces the target xpub under the seed at one of the standard derivation templates (BIP-44 / BIP-49 / BIP-84 / BIP-86 single-sig + BIP-48 multisig at `script_type ∈ {1', 2', 3'}`) × account range. Same `match_xpub_against_paths` primitive + candidate-set as `path-of-xpub`; semantic difference is that P4 answers **"does THIS passphrase produce the xpub?"** rather than P1's **"what path produced this xpub?"**.

  Clap-derive enforces the **mandatory passphrase group**: exactly one of `--passphrase` / `--passphrase-stdin` must be supplied (`required_unless_present` pair forms the mandatory mutex). Omitting both is a clap arg-parse error (exit 64); supplying both is a clap mutex error (exit 64). MVP scope is single-passphrase verification only — no `--passphrases-file <path>`, no streamed candidates, no generated wordlists; deferred to v0.27+ via FOLLOWUP `xpub-search-passphrase-bruteforce`.

  Every invocation emits a load-bearing stderr advisory BEFORE the search starts: `note: passphrase verification searches the standard BIP-44/49/84/86 + BIP-48 templates × account range; if the wallet uses a non-standard path, supply --add-path or use \`xpub-search path-of-xpub\` to find the path first.` The advisory is intentionally unconditional: a "no match" result does NOT prove the passphrase is wrong — only that no standard path under the (seed, passphrase) pair produces the target. Users with non-standard paths must extend the candidate set via `--add-path`, or solve the path-lookup separately via `path-of-xpub`.

  `--json` envelope shape: `{"schema_version":"1","mode":"passphrase-of-xpub","result":"match|no_match","path":"m/…|null","template":"bip…|null","account":N|null,"target_xpub_canonical":"xpub…","target_xpub_variant":"zpub|…|null","searched_count":N}` — same shape as `path-of-xpub` with `mode` substituted. Separate `PassphraseOfXpubResult` struct (not a re-export of `PathOfXpubResult`) keeps future per-mode divergence clean. Exit codes: 0 match / 1 bad input / 4 no match (`ToolkitError::XpubSearchNoMatch` with `mode: "passphrase-of-xpub"`) / 5 auto-fire short-circuit on `--ms1` decode failure / 64 clap. Seed-intake polymorphism + argv-leak advisories + secret-hygiene (Zeroizing + mlock pinning) match `path-of-xpub`.

  New files: `cmd/xpub_search/passphrase_of_xpub.rs`. The shared verify-helper from plan §2.2 was inlined per orchestrator decision (the verification path is a thin wrapper over `match_xpub_against_paths` already exposed by C1; a separate `passphrase_verify.rs` would have been 5 LOC of pure indirection). Umbrella `cmd/xpub_search/mod.rs` extended with the `PassphraseOfXpub` variant + dispatch arm + `XpubSearchJson::PassphraseOfXpub` enum arm + `pub use` re-export. 10 new integration cells (`tests/cli_xpub_search_passphrase_of_xpub.rs`). gui-schema renamed `gui_schema_lists_all_fifteen_subcommands` → `gui_schema_lists_all_sixteen_subcommands` with `xpub-search-passphrase-of-xpub` in the alphabetically-sorted list (note: `passphrase-of-xpub` sorts BEFORE `path-of-xpub` lexically — `passphrase` < `path`).

- **`mnemonic xpub-search address-of-xpub`** (C3) — third mode of the `xpub-search` umbrella. Given a parent xpub (any SLIP-0132 single-sig prefix: `xpub`/`tpub`/`ypub`/`upub`/`zpub`/`vpub`) OR an mk1 bech32 card carrying an xpub, plus one or more target addresses, scans child receive (`chain=0`) and (default) change (`chain=1`) addresses across the gap-limit window and reports which targets matched at which `(chain, index)`. Per-target first-match-wins; envelope reports per-target match-or-no-match payloads with stable shape. Takes **no seed material** — auto-fire BCH repair does NOT apply, and there is no argv-leakage surface beyond the (non-secret) xpub itself.

  Script-type inference (priority): explicit `--address-type` wins; else SLIP-0132 prefix mapping (`ypub`/`upub` → P2SH-P2WPKH; `zpub`/`vpub` → P2WPKH); else (neutral `xpub`/`tpub` or mk1 input) require `--address-type` explicit. Multisig SLIP-0132 prefixes (`Ypub`/`Zpub`/`Upub`/`Vpub`) are detected via base58check version-byte inspection BEFORE `resolve_target_xpub` and refused with a pointer to `account-of-descriptor` (the single-sig-address derivation from a multisig cosigner xpub is semantically wrong; the full descriptor is required for multisig address materialization).

  Network resolution: explicit `--network` wins; else inferred from the xpub version byte (mainnet ↔ `xpub`/`ypub`/`Ypub`/`zpub`/`Zpub`; testnet ↔ `tpub`/`upub`/`Upub`/`vpub`/`Vpub`). `--network signet` / `--network regtest` overrides the test/signet/regtest ambiguity collapsed by the version byte.

  Scan covers `chain ∈ {0, 1} × index ∈ [0, gap_limit)`; `--external-only` restricts to `chain ∈ {0}`; `--gap-limit` (default 20) tunes the window. Address rendering reuses the v0.26.0-extended `build_address_from_xpub` (P2PKH branch added in this commit — see `### Changed`); byte-equal comparison against each target address.

  `--json` envelope shape: `{"schema_version":"1","mode":"address-of-xpub","results":[{"target":"…","result":"match","chain":"external|internal","index":N,"script_type":"p2pkh|p2sh-p2wpkh|p2wpkh|p2tr"} | {"target":"…","result":"no_match","scanned_external":N,"scanned_internal":N}, ...],"xpub_canonical":"xpub…","xpub_variant":"zpub|ypub|…|null","gap_limit":N}`. Mixed match / no-match payloads supported; envelope shape stays stable. Exit codes: 0 all matched / 1 bad input (xpub parse error, multisig prefix, missing `--address-type` for neutral xpub) / 4 any unmatched (`ToolkitError::XpubSearchNoMatch` with `mode: "address-of-xpub"`) / 64 clap.

  New files: `cmd/xpub_search/address_of_xpub.rs`, `cmd/xpub_search/address_search.rs`. 17 new integration cells (`tests/cli_xpub_search_address_of_xpub.rs`; 16 in the C3 commit + 1 added in the C3 R0 fold for `--network signet` override). Umbrella `cmd/xpub_search/mod.rs` extended with the `AddressOfXpub` variant + dispatch arm.

- **`mnemonic xpub-search account-of-descriptor`** (C2) — second mode of the `xpub-search` umbrella. Given a seed (BIP-39 phrase OR ms1 card) + a wallet descriptor, identify which cosigner role(s) the seed plays and at which account index. Three descriptor input shapes auto-detected per tie-break order: (1) BIP-388 wallet-policy JSON (starts-with `{`); (2) md1 card(s) (`md1` HRP — single inline OR `--descriptor-from md1=-` stdin one-chunk-per-line); (3) external literal-xpub descriptors (Sparrow/Specter/Core/Electrum/Liana/Caravan/Coldcard). Toolkit `@N`-placeholder descriptors are REFUSED (synthetic xpubs are non-searchable). Explicit shape override via `--descriptor-from <node>=<value>` where `<node>` is `literal` / `md1` / `bip388`.

  Two-funnel implementation: BIP-388 JSON + literal-xpub paths feed `rust_miniscript::Descriptor::<DescriptorPublicKey>::from_str` + `iter_pk()` walk (precedent `wallet_export/pipeline.rs:177`); md1 path feeds `md_codec::chunk::reassemble` → direct tree-walk on `desc.tlv.pubkeys` / `desc.tlv.fingerprints` / `desc.tlv.origin_path_overrides` / `desc.path_decl.paths` (no `md_codec::Descriptor → String` serializer required). BIP-388 reconstruction: `@N/**` token in `description_template` → `keys_info[N] + "/<0;1>/*"` (exact inverse of `wallet_export/pipeline.rs:192-198` emitter).

  v0.19.0 silent-default-path inference applies when literal-xpub descriptors omit `[fp/path]` annotations — BIP-48 default path (`m/48'/<coin>'/<account>'/2'`) assigned + stderr `info:` notice emitted (~6 LOC inline mirror of `cmd/bundle.rs:1367-1388`).

  NUMS sentinel cosigners (BIP-341 unspendable internal-key H point) are skipped and reported in JSON `unspendable_internal_keys` array. Zero-xpub guard: descriptors yielding no `DescriptorPublicKey::XPub` / `MultiXPub` entries (string funnel) OR `desc.tlv.pubkeys.is_none()` (tree-walk funnel) refused with `descriptor contains no extended keys`.

  Per-cosigner search reuses C1's `match_xpub_against_paths` primitive over the same candidate set (BIP-44/49/84/86 single-sig + BIP-48 multisig × account range + `--add-path` templates). Multi-cosigner match (one seed matches >1 cosigner via reused mnemonic across roles) → reports all matches.

  `--json` envelope shape: `{"schema_version":"1","mode":"account-of-descriptor","result":"match|no_match","matched_cosigners":[...],"cosigners_total":N,"searched_count_per_cosigner":N,"descriptor_shape":"literal_xpub|md1|bip388","unspendable_internal_keys":[...]}`. Exit codes: 0 match / 1 bad input / 4 no match / 5 auto-fire short-circuit / 64 clap. New files: `cmd/xpub_search/account_of_descriptor.rs`, `cmd/xpub_search/descriptor_intake.rs`, `cmd/xpub_search/account_search.rs`. 14 new integration cells (`tests/cli_xpub_search_account_of_descriptor.rs`) + 5 new unit cells in `descriptor_intake::tests`.

- **`mnemonic xpub-search path-of-xpub`** — new umbrella subcommand `xpub-search` with the first of four planned modes shipped. Given a seed (BIP-39 phrase OR ms1 card) + a target xpub (any SLIP-0132 prefix, or an mk1 bech32 card), searches the standard derivation templates (BIP-44 / BIP-49 / BIP-84 / BIP-86 single-sig + BIP-48 multisig at `script_type ∈ {1', 2', 3'}`) × account range, returning the matching path on first hit. `--add-path <TEMPLATE>` extends the candidate set (literal token `account'` or `account` substituted per iterated account; templates without an `account` token are searched once at the literal path).

  Seed intake: `--phrase` / `--phrase-stdin` / `--ms1` / `--ms1-stdin` / positional ms1 (HRP-autodetect; BIP-39 phrase text rejected positionally — no HRP). BCH auto-fire repair applies ONLY to the `--ms1` decode-failure path (TTY-gated via `MNEMONIC_FORCE_TTY`); `--phrase` BIP-39 parse failure routes direct exit 1.

  `--passphrase` / `--passphrase-stdin` plumbed through to `derive_master_seed`. Target intake accepts both SLIP-0132 xpubs (normalized to canonical xpub/tpub form internally; original variant preserved in output) AND `mk1` bech32 cards carrying an xpub.

  Exit codes: 0 match / 1 bad input / 4 no match / 5 auto-fire short-circuit / 64 clap. New `ToolkitError::XpubSearchNoMatch { mode, searched }` variant routes to exit 4.

  `--json` envelope shape (`tag = "mode"` — deviates from project's `tag = "kind"` convention because "mode" is the natural domain term for `xpub-search`'s four sub-modes; "kind" would conflict with `RepairJson`'s `kind: "ms1"|"mk1"|"md1"` per-card-type semantic):

  ```json
  {
    "schema_version": "1",
    "mode": "path-of-xpub",
    "result": "match",
    "path": "m/84'/0'/0'",
    "template": "bip84",
    "account": 0,
    "target_xpub_canonical": "xpub6...",
    "target_xpub_variant": "zpub",
    "searched_count": 140
  }
  ```

  `target_xpub_variant` always emitted (`null` when the target was supplied in canonical xpub/tpub form); structural stability across runs. The top-level `schema_version: "1"` field is new on `XpubSearchJson`; parallel addition to `InspectJson` is filed as FOLLOWUP `inspect-json-schema-version-backfill` for v0.27+.

  Implementation: 6 new files under `cmd/xpub_search/` (umbrella `mod.rs` + per-mode `path_of_xpub.rs` + shared helpers `candidate_paths.rs` / `path_search.rs` / `seed_intake.rs` / `target_intake.rs`); per-mode-file split enables parallel-disjoint follow-on commits for the remaining 3 modes. 19 new integration cells (`tests/cli_xpub_search_path_of_xpub.rs`) + 8 unit cells.

  Plan: `design/PLAN_v0_26_0_xpub_search.md` (C6 release commit will copy from the plan-mode source-of-truth).

- **`mnemonic import-wallet` subcommand** — parse-side ingest for third-party wallet blobs. Two formats supported in v0.26.0:
  - **BIP-129 BSMS Round-2** (`--format bsms`): 2-line and 6-line lenient shapes. Parses the descriptor body via `MsDescriptor::from_str` after a concrete-keys → `@N`-placeholder adapter rewrite (`wallet_import::pipeline::concrete_keys_to_placeholders`). BIP-380 checksum validated up-front via `miniscript::descriptor::checksum::verify_checksum` (before placeholder substitution). Audit fields (token, signature, first_address, derivation_path) preserved verbatim in the `--json` envelope's `bsms_audit` object; signature verification deferred to v0.27+ FOLLOWUP `bsms-verify-signatures`. Driving seed-case: `wsh(thresh(2, pk, s:pk, sln:older(N)))` decaying-multisig with N=144 / N=4032 / N=32768.
  - **Bitcoin Core `listdescriptors`** (`--format bitcoin-core`): top-level JSON object (`{wallet_name, descriptors: [...]}`) OR bare-array shape. `--select-descriptor <N|active-receive|active-change|all>` filters the multi-descriptor case. Refuses `xprv`-bearing blobs with exit 2 + stderr template directing the user to re-run `bitcoin-cli listdescriptors` without `true`. Tested against testnet (`tprv`), `tpub`, BIP-49 / BIP-84 / wsh-sortedmulti / multipath shapes.
  - **Auto-detect (sniff)** when `--format` is omitted: heuristics in `wallet_import::sniff::sniff_format`. BSMS prefix-match `BSMS 1.0\n` (CRLF tolerant); Bitcoin Core JSON-parse + `descriptors[].desc: String` shape with conservative Specter/Sparrow vendor-marker exclusion (`chain`, `policy`, `version`, `bipname`, `extendedPublicKey`). Ambiguous + no-match cases route to exit 1.
  - **Seed overlay** (`--ms1` repeating; `--slot @<i>.phrase=`): post-parse cosigner-by-cosigner entropy attach. Derives xpub at the cosigner's declared origin path; cross-checks against the blob's declared xpub; exit 4 `ImportWalletSeedMismatch` on mismatch. Watch-only-cosigner empty-string sentinel honored per v0.25.1 contract (cosigner skipped + stderr NOTICE).
  - **`--json` envelope** (SPEC §2.2 / §7.4): emits an array of bundle envelopes per blob. Each envelope carries `bundle` (parse-side summary in v0.26.0; full `BundleJson` deferred to v0.27+ FOLLOWUP `wallet-import-json-envelope-full-bundle`), `roundtrip { byte_exact, semantic_match, diff?, status }`, optional `bsms_audit`, optional `source_metadata`, `source_format`. The `status` extension key takes values `"ok"` / `"blocked_no_emitter"` / `"canonicalize_failed"`.
  - **Round-trip discipline** (SPEC §7): `canonicalize_bsms` + `canonicalize_bitcoin_core` + `unified_diff` helpers. Semantic round-trip via canonicalize equality with unified-diff on byte-mismatch; idempotency cells pin `canonicalize(canonicalize(x)) == canonicalize(x)`. `similar = "2"` dep added (Apache-2.0/MIT). BSMS bundle round-trip blocked on missing export emitter (FOLLOWUP `wallet-export-bsms-emitter` v0.27+).

- **Cross-cutting `@env:<VAR>` value-source sentinel** (`crate::env_sentinel::resolve_env_var_sentinel`). Resolves at clap-parse-side for all secret-bearing flags: `--ms1`, `--mk1`/`--md1`, `--passphrase`, `--bip38-passphrase`, `--share`, `--slot @<i>.phrase=`, `--slot @<i>.ms1=`. VAR must match `[A-Z_][A-Z0-9_]*`. Missing → exit 1 `EnvVarMissing` with `reason: { Unset, InvalidName }` discriminator. Non-secret flags treat `@env:VAR` as literal text (no auto-resolution; per SPEC §3.2 + §5.11 explicit-opt-in rule).

- **`PossibleValuesParser` on `--format`** for clap-side enumeration (post Phase 5 R0 M4 fold).

- **Test count delta:** +161 cells cycle-wide (1153 → 1314 in `cargo test -p mnemonic-toolkit`).

### Changed

- **`mnemonic convert` — P2PKH gap-fix in `build_address_from_xpub`** (C3, plan §5.3) — extends the address-rendering primitive (and the `--script-type` / `--address-type` clap value-parser surface) to support P2PKH alongside the prior `p2sh-p2wpkh` / `p2wpkh` / `p2tr` set. Five-site edit: `ScriptType` enum gains a `P2pkh` variant; `parse_script_type_arg` accepts the `"p2pkh"` token; `script_type_from_template` maps `CliTemplate::Bip44 → ScriptType::P2pkh`; `build_address_from_xpub` adds the `ScriptType::P2pkh => Address::p2pkh(...)` arm; the prior P2PKH refusal in `mnemonic convert --script-type p2pkh` is relaxed (it was a gap left at v0.13.0+ — BIP-44 was supported by `mnemonic bundle` / `mnemonic export-wallet` but `mnemonic convert` refused the script-type at parse-time). Four cells in `tests/cli_convert_address.rs` touched: existing `refusal_address_no_script_type` updated to mention `p2pkh` in the value-parser refusal list; new `bip44_template_infers_p2pkh_v0_26_0`, `refusal_invalid_script_type_value`, and `xpub_to_address_p2pkh_explicit_script_type_v0_26_0`. Required by `xpub-search address-of-xpub --address-type p2pkh`; the gap-fix is bundled with C3 rather than carried as a separate patch because the two land in the same logical surface and share regression-test scope.

- **SPEC `SPEC_mnemonic_toolkit_v0_5.md` amendments** (carry-forward toolkit SPEC):
  - `§5.11 CLI value-source sentinels (NEW)` — generalizes the three sentinel forms (empty-string + stdin + env-var) across all secret-bearing CLI surfaces.
  - `§6.11 import-wallet CLI grammar (NEW)` — clap surface + sniff dispatch + override semantics + exit codes + `--json` envelope shape.
  - `§6.11.a wallet_import round-trip discipline (NEW)` — bundle + semantic blob round-trip; canonicalize per-format algorithms; `status` extension key lock; idempotency + declaration-order-preservation guarantees.

- **Manual mirror surfaces:**
  - `docs/manual/src/40-cli-reference/41-mnemonic.md` — new `## mnemonic import-wallet` section mirrors `--help` byte-shape.
  - `docs/manual/src/45-foreign-formats.md` — new chapter on BSMS Round-2 + Bitcoin Core `listdescriptors` formats; normative BIP-129 / BIP-380 / BIP-389 references.
  - `docs/manual-gui/src/40-mnemonic/4c-import-wallet.md` — new GUI walkthrough.

### Security

- **Env-var sentinel `@env:<VAR>` keeps secrets off the argv vector** (visible to `/proc/<pid>/cmdline` + shell history). v0.11.0 `mnemonic-gui` companion ships SubcommandSchema for import-wallet but does NOT auto-rewrite literal repeating-secret values; GUI users must type `@env:<VAR>` explicitly with `<VAR>` exported in the calling shell to benefit. Auto-rewrite tracked at FOLLOWUP `gui-import-wallet-env-var-secret-channel` (v0.12.0+).

- **BIP-129 token + signature on Round-2 blobs are NOT verified** in v0.26.0 (FOLLOWUP `bsms-verify-signatures` v0.27+). Audit fields preserved verbatim in the envelope for the user to verify manually.

- **`xprv`-bearing Bitcoin Core blobs are hard-refused** (exit 2). Extends to testnet `tprv` and SLIP-132 private-key prefixes (`yprv`/`zprv`/`uprv`/`vprv` etc.) via regex per Phase 3 R0 C1 fold.

### Resolved (FOLLOWUPS)

- `wallet-import-bsms-checksum-delegation-note` — SPEC §4.4 amended in this cycle-close commit to describe the actual mechanism (up-front validation via `miniscript::descriptor::checksum::verify_checksum` BEFORE placeholder substitution). Implementation at `wallet_import/bsms.rs:26-27,140-145` was correct since Phase 2 close; only the SPEC wording is now corrected.

### Cross-repo lockstep

`mnemonic-gui v0.11.0` (companion release) at `feat/import-wallet-v0_11_0` branch:
- `SubcommandSchema` entry for `import-wallet` (schema v5 — no version bump).
- 8 kittest cells pinning argv-emission contracts.
- 1 new FOLLOWUP `gui-import-wallet-env-var-secret-channel` (cross-cited companion).

### FOLLOWUPs filed (13 new this cycle)

- `bsms-first-address-verify`, `wallet-import-signet-regtest-disambiguation`, `wallet-import-bsms-checksum-delegation-note`, `bsms-verify-signatures` (Phase 2 close)
- `wallet-export-bsms-emitter`, `wallet-import-fixture-corpus-expansion` (Phase 4 close)
- `wallet-import-json-envelope-full-bundle` (Phase 5 close)
- `gui-import-wallet-env-var-secret-channel`, `gui-import-wallet-cell-coverage-gap` (Phase 6 close)
- `wallet-import-{sparrow, specter, electrum, coldcard, coldcard-multisig, jade, bsms-round-1, bsms-encrypted}` (Phase 6 cycle-close placeholders)

## mnemonic-toolkit [0.25.1] — 2026-05-18

### Fixed

- **Restore pre-v0.24.0 empty-string `--ms1 ""` watch-only sentinel** per SPEC §5.8. v0.24.0 §2.C.1's strict per-flag HRP gate (`validate_flag_hrp`) accidentally hard-failed empty strings on `--ms1` (and by symmetry on `--mk1` / `--md1`), breaking the positional middle-cosigner watch-only convention. v0.25.1 special-cases empty strings in `validate_flag_hrp` to pass through (alongside the existing `"-"` stdin sentinel). `verify-bundle` emits a one-line stderr NOTICE per empty-`--ms1` cosigner (`notice: cosigner[N] marked watch-only via empty --ms1 sentinel (SPEC §5.8); no seed will be derived for this slot`) — guards the accidental-empty-from-unset-shell-variable footgun by making the intent visible.

  Two equivalent CLI forms for watch-only cosigners (both grounded in SPEC §5.8's wire-level invariant `ms1[i] == ""`):
  - **Middle / trailing skip — empty-string sentinel** `--ms1 <s0> --ms1 "" --ms1 <s2>` (canonical; required for middle-cosigner watch-only).
  - **Trailing skip — flag omission** `--ms1 <s0>` (shorthand; positional vec naturally stops at the last full-path index; works only for trailing cosigners).

  Resolves FOLLOWUP `verify-bundle-empty-ms1-watch-only-sentinel-or-explicit-flag` (filed during v0.25.0 end-of-cycle architect review). SPEC §5.8 prose updated with explicit "CLI input forms" subsection documenting both forms.

## mnemonic-toolkit [0.25.0] — 2026-05-18

### Added

- **`verify-bundle` ms1-driven `parent_fingerprint` defense-in-depth check at depth ≥ 2** (extends v0.24.0 A.1's depth 0/1 structural checks via a new helper `emit_full_path_parent_fingerprint_check` in `cmd/verify_bundle.rs`). For each cosigner with `path.len() >= 2`:
  - **Full-path mode (ms1 supplied):** decode ms1 → derive parent xpub at `path[..N-1]` from the ms1's master seed → compute fingerprint → compare against the claimed `mk1.xpub.parent_fingerprint`. Emit stderr warning on mismatch (catches card-print errors where cosigner mk1s are spliced from different wallets). Passphrase-aware via `--passphrase`; language-aware via `--language`; network-aware via `--network`.
  - **Watch-only mode (no ms1 for this cosigner):** emit explicit stderr notice `notice: cosigner[{idx}] mk1 parent_fingerprint at depth {N} unverified (requires ms1 to derive parent xpub)`. Cryptographic ceiling per BIP-32 child→parent one-wayness — cannot be checked without seed access. Explicit wontfix partition for the depth-≥-2 watch-only case.

  Failure mode: stderr WARNING / NOTICE only; verify-bundle exit code and `result: ok / mismatch` verdict UNCHANGED (permissive-input / expressive-output). Resolves FOLLOWUP `verify-bundle-xpub-parent-fingerprint-derivation` — the original "derive parent from supplied mk1" framing was structurally impossible (BIP-32 `derive_pub` is parent→child only); corrected to ms1-driven derivation with an explicit wontfix partition for the watch-only ceiling. 5 new integration cells (2 watch-only in `tests/cli_verify_bundle_watch_only.rs`; 3 full-path in `tests/cli_verify_bundle_full.rs`).

### Changed

- **`convert` and `inspect` BCH auto-fire short-circuit is now TTY-gated** (matching `verify-bundle`'s v0.22.1 D18 contract). Piped consumers (no TTY) of `mnemonic convert --from ms1=...` and `mnemonic inspect --ms1=...` no longer see auto-fire short-circuit (exit 5) on corrupted card decode by default; they see the typed decode error instead. Interactive users (real TTY) continue to see the helpful auto-fire UX. Piped consumers who want auto-fire (e.g., CI / scripts) opt back in via `MNEMONIC_FORCE_TTY=1` — same mechanism `verify-bundle` has used since v0.22.1, and the same env-var `mnemonic-gui` v0.9.0+ already sets globally on every toolkit subprocess.

  Before (v0.24.0):
  ```sh
  $ echo "" | mnemonic convert --from ms1=ms1corruptedstring --to phrase
  # auto-fired BCH repair; printed repair report; exit 5
  ```

  After (v0.25.0):
  ```sh
  $ echo "" | mnemonic convert --from ms1=ms1corruptedstring --to phrase
  # typed decode error on stderr; exit ≠ 5
  $ MNEMONIC_FORCE_TTY=1 echo "" | mnemonic convert --from ms1=ms1corruptedstring --to phrase
  # auto-fired (same as v0.24.0); exit 5
  ```

  Resolves FOLLOWUP `convert-inspect-auto-fire-tty-gate-asymmetry`.

### Internal

- Extract shared `pub(crate)` card-arg helpers (`count_dashes`, `expand_dashes`, `resolve_groups`) + new TTY helper (`resolve_no_auto_repair`) from `cmd/repair.rs` + `cmd/inspect.rs` into `crate::repair`. Joins `classify_hrp_prefix` + `validate_flag_hrp` from v0.24.0 C.1-fold. `pub(crate) trait CardArgs` parameterizes the dedup. Net -19 LOC across the cmd files. Resolves FOLLOWUP `cmd-repair-inspect-helper-duplication`.
- Add `debug_assert!` for global-vs-local flag-id disjointness in `gui_schema.rs::build_subcommand`. Defensive guard against future global-flag additions whose names collide with subcommand-local flags (B.1 architect review surfaced this as dead defense via the pre-existing `seen_flag_names` HashSet; the debug_assert makes the invariant load-bearing). Positive-invariant test cell runs in both debug + release; `#[cfg(debug_assertions)]`-gated negative-control cell exercises the assert in debug only. Resolves FOLLOWUP `gui-schema-global-flag-id-disjointness-debug-assert`.

## mnemonic-toolkit [0.24.0] — 2026-05-17

v0.24.x cycle: three-tranche follow-up release folding 9 items across
verify-bundle defense-in-depth (Tranche A), gui-schema v5 envelope
(Tranche B), and positional-intake UX (Tranche C). Lockstep with
`mnemonic-gui-v0.10.0` (B + C consumer side), `mk-cli-v0.4.1` (stale
md-codec pin refresh), and `md-codec-v0.35.0` (non-chunked md1 in
`decode_with_correction`).

### Added — Tranche A: verify-bundle hardening

- `emit_watch_only_xpub_path_cross_check` in `cmd/verify_bundle.rs` —
  stderr WARNING-level cross-check between mk1 xpub byte-level fields
  (`depth` / `child_number` / `parent_fingerprint`) and md1's claimed
  `OriginPath`. Three checks per cosigner: xpub depth vs path length,
  final child number (incl. hardened bit), and parent-fingerprint
  structural sanity (master invariant at md_depth 0; equality with
  master fingerprint at md_depth 1; deeper paths skipped — would
  require parent xpub derivation, infeasible without seed). Failure
  mode: stderr WARNING per cosigner; verify-bundle exit code +
  `result: ok / mismatch` verdict UNCHANGED (SPEC §5.4 / §5.7
  VerifyCheck schema intentionally NOT extended). Multi-cosigner
  index naming preserved. 5 new integration cells in
  `tests/cli_verify_bundle_watch_only.rs`.
- `MNEMONIC_FORCE_TTY` env-var promoted from test-only to **first-class
  public API** (semver-stable contract). Doc-comment in
  `cmd/verify_bundle.rs::run` rewritten; `cmd/convert.rs` +
  `cmd/inspect.rs` consumers cite the public contract. New subsection
  in `docs/manual/src/40-cli-reference/41-mnemonic.md` under the
  verify-bundle auto-fire section enumerating accepted values
  (`1`/`0`/unset) + consumer guidance (mnemonic-gui v0.9.0+ is a known
  caller).

### Added — Tranche B: gui-schema v5 envelope

- Three additive fields on every `flags[]` entry in `cmd/gui_schema.rs`:
  - `default_value: Option<String>` — surfaces the clap-derive
    `#[arg(default_value = "...")]` for downstream `is_at_default`
    suppression logic.
  - `global: bool` — emitted `true` for `--no-auto-repair` per-
    subcommand (clap-derive `global = true` previously surfaced only
    via per-subcommand `--help` TEXT, not JSON). Closes
    `gui-schema-global-flag-emission`.
  - `secret: bool` — exposed via new module
    `crates/mnemonic-toolkit/src/secrets.rs` (NEW) with public
    `flag_is_secret(subcommand: &str, flag: &str) -> bool` predicate
    covering 6 flags (`--passphrase`, `--passphrase-stdin`, etc.).
- Schema integer version bumped `4 → 5`. v4 consumers reading the v5
  envelope ignore the additive fields; v5 consumers reading a v4
  envelope hard-fail on the missing fields.

### Added — Tranche C: positional `<STRING>...` intake

- `repair` / `inspect` / `verify-bundle` subcommands accept positional
  ms1/mk1/md1 strings (no flag required). Toolkit-internal
  HRP-autodetect routing via shared helper `classify_hrp_prefix` per
  the SPEC § validation table. `repair`/`inspect`: drops
  `conflicts_with_all` cross-HRP clauses (D35: mixed-HRP inputs now
  accepted in one call). `verify-bundle`'s positional carries
  `conflicts_with = "bundle_json"` (I3: bundle-JSON path is still
  mutually exclusive with positional/--{ms1,mk1,md1} cards).
- Two new `ToolkitError` variants: `HrpMismatch` (extracted via shared
  helper `validate_flag_hrp` called from all 3 subcommands per D34)
  and `UnknownHrp`. Case-mismatch error text improved.
- New tests file `tests/cli_positional_hrp_autodetect.rs` (positional
  + autodetect + mixed-HRP coverage).
- New tests file `tests/cli_gui_schema_v5_extensions.rs` (v5 envelope
  surface coverage).

### Changed

- **CLAP ERROR-TEXT BREAK (Tranche C):** scripts parsing the literal
  error text `error: '--ms1 <MS1>' cannot be used with '--mk1 <MK1>'`
  from `repair` or `inspect` will now succeed instead. Before:

      $ mnemonic repair --ms1 ... --mk1 ...
      error: the argument '--ms1 <MS1>' cannot be used with '--mk1 <MK1>'
      exit 2

  After:

      $ mnemonic repair --ms1 ... --mk1 ...
      <both cards reported in the JSON envelope / TEXT report>
      exit 0 / 5

  `verify-bundle --bundle-json … --ms1 …` still rejects (I3 keeps the
  bundle_json XOR cards mutex). Mismatched-HRP-via-flag still rejects
  with `ToolkitError::HrpMismatch` (D34).
- `mnemonic-toolkit` Cargo version `0.23.0` → `0.24.0`.
- `bech32-correction-api-version-pin` FOLLOWUP body refreshed —
  upstream `bech32` crate still 0.11.1 (no migration unblock signal).

### Resolved (FOLLOWUPS)

- `verify-bundle-watch-only-xpub-path-internal-consistency` (Tranche A,
  primary).
- `gui-schema-global-flag-emission` (Tranche B; GUI consumer side
  closes lockstep at `mnemonic-gui-v0.10.0`).
- `toolkit-mnemonic-force-tty-promote-from-test-only` (Tranche A;
  cross-repo lockstep close).
- `md-codec-decode-with-correction-supports-non-chunked-md1`
  (toolkit-side consumer perspective; primary lands at md-codec
  v0.35.0).
- `repair-inspect-positional-string-intake` (Tranche C; surfaced
  mid-cycle in plan §2.C.1).

### New FOLLOWUPS filed

- `verify-bundle-parent-fp-deeper-paths` — extend the
  parent-fingerprint cross-check to depth ≥ 2 (requires parent xpub
  derivation; infeasible without seed at v0.24.0).
- `convert-inspect-tty-asymmetry` — `convert --json` vs `inspect --json`
  auto-fire envelope wrapping asymmetry surfaced during Tranche C
  test sweep.
- `gui-schema-global-vs-local-id-disjointness` — schema v5's `global`
  field marks per-subcommand entries `true`; some flags appear under
  both the global and local IDs in clap's reflection. Document the
  precedence + add a drift gate.
- `cmd-repair-inspect-helper-duplication` — `cmd/repair.rs` and
  `cmd/inspect.rs` carry parallel HRP-routing logic that could
  consolidate into the existing `classify_hrp_prefix` helper module.

### Companion

`mnemonic-gui v0.10.0` consumes the v5 schema (Tranche B), drops the
v0.9.0 R7 action-bar `--no-auto-repair` checkbox fallback, and
mirrors Tranche C's at-least-one card mutex (`three_way_card_mutex`
→ `three_way_card_at_least_one`).

`mk-cli v0.4.1` is a standalone patch (stale md-codec pin refresh +
`from_md1_derivation` fixture refresh). `md-codec v0.35.0` adds
non-chunked-form `decode_with_correction` support; toolkit consumes
it transparently via the unchanged `repair_via_md_codec` delegation.

## tech-manual [1.1.0] — 2026-05-12

**v0.8.1 wallet-export drift fold.** Adds §V.4.5.9 (eight vendor-format output-shape sub-sub-sections) + §V.4.5.10 (8×8 format×shape compatibility matrix with 7 footnotes). 273pp PDF (up from 258 at v1.0). Tag `tech-manual-v1.1.0`.

### Added

- **§V.4.5.9** `export-wallet` output shapes (`50-rust-api/54-mnemonic-toolkit-api.md`): eight sub-sub-sections — `bitcoin-core` (V.4.5.9.1), `bip388` (V.4.5.9.2), `coldcard` (V.4.5.9.3), `jade` (V.4.5.9.4), `sparrow` (V.4.5.9.5), `specter` (V.4.5.9.6), `electrum` (V.4.5.9.7), `green` (V.4.5.9.8). Each documents the `--format <value>` selector, emitter source path (`wallet_export/<vendor>.rs`), output shape (JSON or text) with worked example, accepted descriptor shapes, refused shapes with refusal mode + variant (`BadInput` vs `ExportWalletMissingFields`), vendor-specific flags, and schema-version semantics. Selector-enum-declaration order matches `cmd/export_wallet.rs:21-39`.

- **§V.4.5.10** `export-wallet` format × shape compatibility matrix: 8 rows (descriptor shapes: `wpkh`, `pkh`, `sh(wpkh)`, `tr(xpub)`, `wsh(multi|sortedmulti)`, `sh(wsh(multi|sortedmulti))`, `tr(NUMS,multi_a|sortedmulti_a)`, `tr(@N,multi_a|sortedmulti_a)`) × 8 columns (vendor formats) = 64 cells. Seven footnotes [a]–[g] enumerate per-emitter refusal sources. Three trailing prose bullets cross-reference §V.4.4 ToolkitError rows, the §III.2 BIP-388 shape ladder, and the SPEC §5.3 byte-exact missing-info refusal contract.

- **Glossary** (`60-back-matter/61-glossary.md`): 433 → 453 lines. Five new `pub(crate)` symbol entries — `EmitInputs` (`wallet_export/mod.rs:327`), `MissingField` (`mod.rs:224`), `TimestampArg` (`mod.rs:122`), `WalletFormatEmitter` (`mod.rs:316`), `WalletScriptType` (`mod.rs:143`). All 8 existing vendor-format entries refined with accepted/refused shape lists + emitter source pointers.

- **Index** (`60-back-matter/62-index-table.md`): 548 → 553 lines. +5 new rows mirroring the new glossary symbol entries.

- **cspell** (`docs/technical-manual/.cspell.json`): +5 entries (`XONLY`, `blockheight`, `libsecp`, `singlesig`, `Singlesig`).

### Changed

- **§V.4.3.8 / §V.4.4 / §V.4.7** (`50-rust-api/54-mnemonic-toolkit-api.md`): cross-references and the `ExportWallet*` ToolkitError table rows refreshed to point readers to §V.4.5.9 + §V.4.5.10 for the authoritative per-vendor treatment.

- **Troubleshooting** (`60-back-matter/65-troubleshooting.md`): 4 `ExportWallet*` rows (`ExportWalletSecretInput`, `ExportWalletFormatStub`, `ExportWalletTaprootMultisigUnsupported`, `ExportWalletMissingFields`) refined to point to §V.4.5.9 / §V.4.5.10 alongside the existing §V.4.4 / §III.2 / §IV.1 references.

### Reviewer rounds

- **r1** (architect, 2026-05-12): 0C / 2I / 0L / 1N. Folded inline: `ELECTRUM_SEED_VERSION_PIN` "Defined" pointer §V.4.3.8 → §V.4.5.9.7; Coldcard sibling cross-reference §V.4.5.9.6 → §V.4.5.9.4. Nit `XONLY` cspell removal reverted (empirically load-bearing).
- **r2** (architect, 2026-05-12): 0C / 4I / 0L / 0N + 3 parent-caught stragglers. Folded inline: 4 glossary entries (`TaprootInternalKey` + `wallet-export` "Defined" pointers; Jade + Coldcard `v0.8.2` → `v0.8.1` tag attribution); 3 chapter prose references to non-existent `v0.8.2` tag collapsed to `v0.8.1`.
- **r3** (architect, 2026-05-12): 0C / 0I / 0L / 0N. All folds verified; `TaprootInternalKey` `pub` visibility confirmed against `wallet_export/mod.rs:68`. Tag-ready.

### Discipline observations

- `zero_followups_from_release_cycles` held: every reviewer finding folded inline; zero new FOLLOWUPs filed by this cycle.
- Three pre-existing toolkit-source doc-comment drifts surfaced during r1 (`wallet_export/mod.rs:1-12` mod-doc lists 3 of 8 submodules; `wallet_export/mod.rs:42-44` SPEC §3 mismatch; `cmd/export_wallet.rs:3-5` cites v0.7 SPEC despite v0.8 realisation). These are source-side hygiene items, not chapter findings; left for a future toolkit-side cleanup commit.
- The `mnemonic-toolkit-v0.8.2` tag was referenced repeatedly in the initial draft but does not exist (`git tag --list 'mnemonic-toolkit-*'` returns v0.5.0 through v0.8.1 only). HEAD's `crates/mnemonic-toolkit/` content is byte-identical with the v0.8.1 tag for that crate. All "v0.8.2" attributions in the chapter were corrected to "v0.8.1" — the actual tagged version that contains this surface.

### Acceptance criteria (SPEC §7 A1–A11)

All 11 green (re-verified at v1.1 cycle exit):

- A2 public-symbol coverage: 92/92 via `tests/api-surface-coverage.sh` (4 crates: md-codec 46 + mk-codec 26 + ms-codec 13 + mnemonic-toolkit 7 JSON envelope types).
- A3 Error variants: 101 tabled (unchanged from v1.0 — `error.rs` counts unchanged).
- A4 glossary: 112 entries (107 at v1.0 + 5 new symbol entries).
- A5 index: 545 rows (540 at v1.0 + 5 new symbol rows).
- A10 PDF: 273pp (up from 258 at v1.0).
- A11 reproducible build: SHA256 `1cf73f9411f6926941015f8dc97b08617aaf4764a56c4cb8653196550af139f6`, 924,968 bytes, byte-identical across two clean `SOURCE_DATE_EPOCH=1746921600` builds.
- A1, A6, A7, A8, A9: unchanged from v1.0 (no Part-structure changes, no new BIPs cited, no worked-example changes, both mirror invariants green).

## tech-manual [1.0.0] — 2026-05-12

**v1.0 release.** Back-matter polish + architect sign-off on the "every aspect of the software" coverage claim. ~258pp PDF. Tag `tech-manual-v1.0.0`.

### Added

- **§65 Troubleshooting** (`60-back-matter/65-troubleshooting.md`): 67 → 219 lines. Full **101-Error-variant coverage** across all four crates:
  - md-codec: 43 variants in 7 emit-site clusters mirroring §V.1.4.
  - mk-codec: 22 variants in 2 clusters mirroring §V.2.4.
  - ms-codec: 10 variants.
  - mnemonic-toolkit: 26 ToolkitError variants in 7 thematic clusters mirroring §V.4.4 with `kind()` + exit-code annotations.
  Each row: variant name (verbatim per HEAD `error.rs`) + likely cause + remediation pointer (cites the relevant chapter section).

- **§66 Bibliography** (`60-back-matter/66-bibliography.md`): 41 → 65 lines. Bibliography completed across Parts I-V:
  - **+9 BIPs** (BIP-38, BIP-44, BIP-45, BIP-48, BIP-49, BIP-84, BIP-85, BIP-86, BIP-87) for a total of 20 BIPs cited.
  - **+1 non-BIP standard** (SLIP-0132).
  - **+1 advisory** (RUSTSEC-2023-0071 — deferred BIP-85 RSA / RSA-GPG applications).
  - **+5 reference implementations** (`rust-bitcoin` v0.32, `bip39`, `getrandom`, `serde` / `serde_json`, `thiserror`); duplicate `rust-codex32` / `codex32` entries from the initial draft merged.
  - **+4 toolkit SPECs** (`SPEC_v0_5`, `SPEC_convert_v0_6`, `SPEC_export_wallet_v0_8`, `SPEC_derive_child_v0_8`).
  - "Cited in" lists extended through Parts III-V on all 11 pre-existing BIP rows.

- **§V.4.3.8 v0.8.x drift fold.** The toolkit's `wallet_export` module grew during the v0.8.1 + v0.8.2 cycle (8 vendor-emitter sub-modules: `bip388`, `bitcoin_core`, `coldcard`, `electrum`, `green`, `jade`, `sparrow`, `specter`, plus `pipeline`). Chapter row updated to enumerate the sub-modules + add the new `pub const ELECTRUM_SEED_VERSION_PIN: u32 = 17` (`wallet_export/electrum.rs:37`).

- **Index expansion**: 530 → 540 rows (vendor-format terms + `ELECTRUM_SEED_VERSION_PIN` + `TaprootInternalKey`).

- **Glossary expansion**: 96 → 107 entries (vendor-format definitions + `wallet-export` + `TaprootInternalKey` + `ELECTRUM_SEED_VERSION_PIN`).

- **cspell additions** (this cycle, cumulative across phases 5.5 + 5.1-5.3): `unconstructed`, `varints`, `formedness`, `multipaths`, `Araoz`, `Matias`, `Alejo`, `Fontaine`, `Weigl`, `Kosakovsky`, `Spigler`, `Satoshi`, `satoshilabs`, `Riccardo`, `Casatta`, `Tolnay`, `Aneesh`, `Karve`, `Jade`.

### Notable corrections folded inline during the cut

Per `zero_followups_from_release_cycles`: every reviewer finding (Critical / Important / Low / Nit) folded inline at tag time.

- Phase 5.5 (3 rounds): r1 caught BIP-86 author misattributed to Pieter Wuille + Greg Maxwell (actual: Ava Chow); broken `docs.rs/rust-codex32` URL (404; package is named `codex32`); r2 caught the same broken URL surviving in `12-the-m-format-star.md:45` + BIP-85 missing co-author Aneesh Karve; r3 confirmed 0C/0I with a multi-author BIP sweep across all 20 entries.
- Phase 5.1+5.2+5.3 combined sweep folded the §V.4.3.8 v0.8.x drift inline.
- Final cycle-exit review (architect sign-off): 1 Low folded inline — stale "v0.1 seed / Parts I + II" preamble text in `61-glossary.md` updated to "v0.1 through v1.0 / all five Parts."

### SPEC §7 v1.0 cumulative acceptance criteria — all green

- **A1** (every BIP-388 shape walk-through across §II.1 + §III): PASS — covered at v0.3 close; §III.2 covers 11 derivation buckets; §V.1's v0.32 release note documents the switch from the v0.14-era 5-shape allow-list to `rust-miniscript`-AST conversion.
- **A2** (every public function referenced): PASS — 92/92 via `tests/api-surface-coverage.sh`.
- **A3** (every Error variant has a chapter row): PASS — 43 + 22 + 10 + 26 = **101 variants** tabled.
- **A4** (glossary ≥80 with section pointers): PASS — 107 entries.
- **A5** (index ≥250, bidirectional): PASS — 540 rows.
- **A6** (TOC auto-generated by pandoc): PASS by design.
- **A7** (BIP cross-ref ≥12 BIPs): PASS — 20 BIPs.
- **A8** (worked examples verified): PASS — 15/15 transcripts.
- **A9** (both mirror invariants green): PASS — CHANGELOG present through v1.0; 2 carry-over cross-repo FOLLOWUPs open (both md1-side, do not block v1.0).
- **A10** (PDF ≥200pp): PASS — 258 pages.
- **A11** (build reproducible): PASS — byte-identical across two clean `SOURCE_DATE_EPOCH=1746921600` builds.

### Architect sign-off

The "every aspect of the software" coverage claim is substantiated:

- Wire formats: Parts I (foundations) + II (md1 / mk1 / ms1).
- Address derivation: Part III (descriptor → miniscript → address; shape coverage; network + SLIP-0132).
- Bundle semantics: Part IV (anatomy / anti-collision invariants / future shares).
- Rust API surface: Part V (4 crate chapters with full 101-variant Error taxonomy + 7 JSON envelope shapes + engraving-card layout).
- CLI surface: intentionally delegated to the end-user manual (`docs/manual/src/40-cli-reference/`) per SPEC §4.2.

### Open carry-over FOLLOWUPS (cross-repo; do NOT block v1.0)

- `cross-repo md1-wsh-multi-unsorted-integration-test`.
- `cross-repo md1-bip49-integration-test`.

Both wait on md-codec test-suite work; neither is a documentation gap.

### Verification

- `cargo test --workspace --all-features`: **561 passed, 0 failed, 2 ignored**.
- `make lint` 6/6 green (markdownlint, cspell, lychee, api-surface-coverage [92/92], glossary-coverage, index bidirectional).
- `make verify-examples` 15/15 transcripts pass.
- PDF SHA256: see release notes attached to the GitHub release.

## tech-manual [0.4.0] — 2026-05-11

Part V added — Rust API reference across all four crates (`md-codec`, `mk-codec`, `ms-codec`, `mnemonic-toolkit`). 242pp PDF (was 145pp at v0.3 close, +97pp). Tag `tech-manual-v0.4.0`.

### Added

- **Part V — Rust API reference** (4 chapters):
  - **§V.1 `md-codec`** (~558 lines). v0.32.0 baseline. 20 public modules (19 unconditional + cfg-gated `to_miniscript` behind the `derive` feature). 43-row Error taxonomy. Encoder pipeline (Descriptor → encode_payload → wrap_payload → card string) + decoder pipeline (unwrap → decode_payload → Descriptor) + chunked reassembly. Advanced notes cover the two declared-but-unconstructed Error variants, `Phrase::from_id_bytes` infallibility, `Address<NetworkUnchecked>` discipline, encoder self-canonicalisation, and `MAX_DECODE_DEPTH=128` anti-DoS rationale.
  - **§V.2 `mk-codec`** (~385 lines). v0.2.2 baseline. 13 public modules (5 top-level + 8 sub-modules). 22-row Error taxonomy. Single feature flag (`gen-vectors`, binary-target only). Library-only — `mk-cli` is a sibling binary, out of Part V scope. Advanced notes: BCH primitives forked from BIP-93 (`mc-codex32` extraction retired 2026-05-03); path dictionary is mk1-internal post the `path-dictionary-mirror-stewardship` retirement; `#[non_exhaustive]` policy (6 marked: `KeyCard`/`Error`/`StringLayerHeader`/`CorrectionResult`/`DecodedString`/`ChunkFragment`; 4 unmarked: `BchCode`/`CaseStatus`/`BytecodeHeader`/`XpubCompact`); non-`Result` panic paths on `reconstruct_xpub` (empty path) and `encode*` (CSPRNG failure); stale `"md1"` doc-comments at `bch.rs:575,603`.
  - **§V.3 `ms-codec`** (~278 lines). v0.1.1 baseline. 7 public modules (`consts`, `decode`, `encode`, `error`, `inspect`, `payload`, `tag`); crate-private `envelope` documented as the v0.2-migration seam. 10-row Error taxonomy. **No feature flags**. **Edition 2021** (distinct from md/mk's edition 2024). BIP-93 codex32 adopted **directly** via `rust-codex32 = "=0.1.0"` (the sole codec leaking into the public surface — only `codex32::Error` via `From<codex32::Error> for Error` at `error.rs:122`). v0.1 is single-string only; share encoding deferred to v0.2; no `Codex32String::shares` API claim (rust-codex32 v0.1 only exposes `interpolate_at`).
  - **§V.4 `mnemonic-toolkit`** (~450 lines). v0.8.0 baseline. **Binary-only crate** (no `[lib]`, no `lib.rs`) — chapter pivots from library-API enumeration to (a) JSON envelope schema documentation (7 envelope types — `BundleJson` / `VerifyBundleJson` / `VerifyCheck` / `MultisigInfo` / `CosignerEntry` / `MkField` / `MsField`), (b) crate-structure reference for the 8 non-CLI orchestration modules, (c) 26-row ToolkitError taxonomy with exit codes + `kind()` strings, (d) engraving-card layout with two worked examples (BIP-86 single-sig + 3-of-5 wsh-sortedmulti). `cmd::*` modules explicitly out of scope. v0.9-or-later library-extraction posture documented. Note: variant 26 (`ExportWalletMissingFields`) is `#[allow(dead_code)]`-reserved at v0.8.0 with full machinery wired; Phase-1 emitters land at v0.8.1.
- **Worked-example crate** at `docs/technical-manual/examples/`:
  - Self-contained Rust crate (own `[workspace]`; isolated from the toolkit workspace at `crates/mnemonic-toolkit`).
  - 4 `[[example]]` entries pinned to specific git tags: `md-codec-v0.32.0`, `mnemonic-key-mk-codec-v0.2.2`, `mnemonic-secret-ms-codec-v0.1.1`, plus `serde` + `serde_json` for the standalone-consumer toolkit example (no dep on `mnemonic-toolkit` itself — binary-only).
  - 4 transcript pairs at `docs/technical-manual/transcripts/<crate>-codec-api-roundtrip.{cmd,out}`. Determinism gated: encoders pinned (e.g. mk-codec uses `encode_with_chunk_set_id` to avoid CSPRNG entropy; ms-codec uses canonical "abandon … about" entropy; toolkit uses a hardcoded schema-4 fixture).
- **API-surface-coverage helper** at `tests/api-surface-coverage.sh` (was a Phase-1.0.3 stub). Hybrid bash + inline Python heuristic walks each crate's `lib.rs`, extracts the public top-level symbol names via regex over `pub use … {a, b as c}` blocks + `pub use … ::name;` re-exports + `pub fn|struct|enum|trait|const|type|static` items, then greps each against the relevant Part V chapter. Emits one warning per gap; exits 0 (hint, not gate, per SPEC §4.4). Binary-only `mnemonic-toolkit` is special-cased to check 7 JSON envelope types at `src/format.rs` against `54-mnemonic-toolkit-api.md`. v0.4 coverage at HEAD: **92/92** symbols across the 4 crates (md-codec 46, mk-codec 26, ms-codec 13, toolkit 7); zero warnings.
- **Back-matter accretion**:
  - Glossary: +23 entries (73 → 96; SPEC §7 A4 v1.0 target ≥80).
  - Index table: +330 rows (200 → 530; SPEC §7 A5 v1.0 target ≥250).
  - BIP cross-reference: 15 → 20 BIPs (BIP-38, BIP-44, BIP-45, BIP-49, BIP-85, BIP-86, BIP-87, BIP-340 added or extended; §V.* columns appended to BIP-32/39/44/48/84/93/173/341/388 rows where Part V cites them).
  - Release-history row for `tech-manual-v0.3.0` added.
- **cspell** allow-list entries (cumulative across cycle): `thiserror`, `usize`, `CHUNKABLE`, `getrandom`, `shibbolethnumskey`, `bijective`, `rustdoc`, `upstreamable`, `impls`, `serialise`, `serialised`, `canonicalised`, `canonicalises`, `keypath`, `Multisignature`, `reconstructor`.

### Notable corrections folded inline during the cut

- Phase 4.0 harvest (3-round reviewer cycle): md-codec Error variant count corrected to 43; mk-codec to 22; mnemonic-toolkit Notes attribution moved from a non-existent `synthesize::check_key_vector_distinctness` to the real `parse_descriptor::check_key_vector_distinctness:1104`; mk-codec stale `"md1"` doc-comments at `bch.rs:575,603` flagged; multiple module/symbol count corrections.
- Phase 4.1 chapter review (2 rounds): inline `use md_codec::{...};` snippets that imported `render_codex32_grouped` and `SINGLE_STRING_PAYLOAD_BIT_LIMIT` from the crate root corrected to module-qualified paths (`md_codec::encode::*`, `md_codec::chunk::*`).
- Phase 4.2 chapter review (2 rounds): `BchCode` and `CaseStatus` falsely claimed `#[non_exhaustive]`; corrected — those two enums are unmarked at HEAD `bch.rs:26,154`. The mk-codec marked set is 6 types (KeyCard / Error / StringLayerHeader / CorrectionResult / DecodedString / ChunkFragment); unmarked set is 4 (BchCode / CaseStatus / BytecodeHeader / XpubCompact).
- Phase 4.3 chapter review (2 rounds): inline decode example `decode("ms10sentrqqq...")` was wire-invalid (`sent` in the id slot, `r` in the share-index slot). Replaced with the byte-exact transcript output. §V.3.5.2 step 3 was missing rule 5 from `discriminate`'s enforcement list — corrected to "rules 2–5, 8".
- Phase 4.4 chapter review (3 rounds): ToolkitError taxonomy was missing the v0.8.1-phase-0-reserved `ExportWalletMissingFields` variant (HEAD `error.rs:109` has 26 variants, chapter said 25); added. §V.4.3.8 `wallet_export` row updated to include `taproot_multisig_unsupported_message` + `build_missing_fields_refusal` and the module path corrected from `wallet_export.rs` to `wallet_export/mod.rs` (split into a directory post v0.8.1 phase-0).
- Final cycle-exit review (2 rounds): §V.4.8 + glossary `non_exhaustive` entry falsely claimed `md_codec::Error` is `#[non_exhaustive]`. Both sites corrected; `md_codec::Error` is the **exception** to the m-format-star non-exhaustive policy (derives `Debug, Error, PartialEq, Eq` only; the toolkit's `md_codec_exit_code` match at `error.rs:174` is consequently exhaustive).

### SPEC §7 v1.0 acceptance criteria progress (cumulative)

- **A2** (every public function referenced in Part V): 92/92 covered via `tests/api-surface-coverage.sh`. ✓
- **A3** (every Error variant has a chapter row): 43 (md) + 22 (mk) + 10 (ms) + 26 (toolkit) = 101 variants tabled. ✓
- **A4** (glossary ≥80): 96 entries. ✓
- **A5** (index ≥250): 530 rows. ✓
- **A7** (BIP cross-reference ≥12 BIPs): 20 BIPs. ✓
- **A8** (worked examples verified): `make verify-examples` 15/15. ✓
- **A9** (both mirror invariants green): `tech-manual-api-surface-mirror` spot-checked; `tech-manual-wire-format-mirror` no-op this cycle. ✓
- **A10** (PDF ≥200pp): 242pp. ✓
- **A11** (reproducible build): byte-identical across two clean `SOURCE_DATE_EPOCH=1746921600` builds. ✓

Pending for v1.0 (Phase 5): A1 (every BIP-388-parseable shape has a bit-level walk in §II.1 + address-derivation walk in §III) — already met by v0.3 close, but Phase 5.6.2 will architect-sign-off; A6 (TOC complete) — pandoc-emitted, no action.

### Open FOLLOWUPS (cross-repo, deferred to md1 work)

- `cross-repo md1-wsh-multi-unsorted-integration-test` (filed v0.2 Phase 2.2).
- `cross-repo md1-bip49-integration-test` (filed post-v0.2 tag).

Both wait on md1 work; neither blocks v0.4 or v1.0.

### Verification

- `make lint` 6/6 green.
- `make verify-examples` 15/15 transcripts (11 pre-existing + 4 new Part-V).
- `cargo test --workspace --all-features`: all green.
- PDF reproducibility: 242pp, 842,175 bytes, SHA256 `ffaa29b94e21a32aa583345965d2366b75d93895d1eac457ae99335417f580cf`, byte-identical across two clean `SOURCE_DATE_EPOCH=1746921600` builds.

## tech-manual [0.3.0] — 2026-05-11

Part IV added — bundle formation end-to-end. 145pp PDF (was 119pp at v0.2 close, +26pp). Tag `tech-manual-v0.3.0`.

### Added

- **Part IV — Bundle formation** (3 chapters):
  - **§IV.1 Bundle anatomy.** Three-card layout (md1 wallet policy + N mk1 xpub records + 0..N ms1 secret records); five `BundleMode` variants (`SingleSigFull` / `SingleSigWatchOnly` / `MultisigMultiSource` / `MultisigWatchOnly` / `MultisigHybrid`) auto-detected by `detect_bundle_mode`; `BundleJson` schema-version-4 envelope (`MkField` discriminated union, `MultisigInfo` block, dense-`MsField` with `""` sentinels for watch-only slots); unified engraving-card layout (SPEC §5.5); `VerifyCheck` per-row forensic-fields. Includes 2 mermaid figures (bundle creation pipeline + bundle verification pipeline). Worked example: BIP-84 abandon-mnemonic single-sig bundle with paired bundle / verify-bundle transcripts.
  - **§IV.2 Anti-collision invariants.** Five invariants policing bundle integrity: (1) shared `chunk_set_id` prefix — md1 prints 16 bits / 4 hex; ms1/mk1 print 20 bits / 5 hex; leading 16 bits agree across all three cards from one bundle (the **engraved** display identifiers, all `policy_id`-derived; md1's *wire* `chunk_set_id` is `Md1EncodingId`-derived, a separate value); (2) multiset `md1_xpub_match` (sort-then-compare on `Vec<[u8; 65]>` with multiplicity, multisig path only — single-sig uses `.first()` comparison via `emit_md1_checks`); (3) four-case ms1 short-circuit table (watch-only / full-decodes / full-malformed / full-absent) with byte-exact `decode_error` strings; (4) mk1 cosigner-mapping diagnostic (`NotSupplied` / `DecodeFailed` / `XpubNotInPolicy`) with `XpubNotInPolicy > DecodeFailed > NotSupplied` precedence; (5) BIP-388 distinct-key enforcement — typed `DerivationPath` equality folding `h` ↔ `'` per SPEC v0.5 §4.11.b deliberate reversal. Documents the live template-mode vs. descriptor-mode bifurcation in `(xpub, path_raw)` raw-string check vs. typed-`(xpub, DerivationPath)` check. Worked example: a 2-of-2 wsh-sortedmulti bundle with both slots resolving to the same `(xpub, path)` aborts at synthesis with byte-exact `error: BIP-388 distinct-key violation: slot @0 and slot @1 resolve to identical (xpub, path)`.
  - **§IV.3 Future shares.** v0.1 → v0.2-shares migration contract locked across all three formats. ms1's four-invariant contract (reserved-prefix byte, prefix-byte grouping discriminator, encoder anti-collision against `RESERVED_TAG_TABLE`, API back-compat via `encode_shares(tag, Threshold::ZERO, &[p])` wire-bit-identical to v0.1 `encode(tag, &p)`). mk1 and md1 v0.2-shares outlook (chunked-card framing leaves room for threshold + share-index header bits; GF(32) interpolation primitive needs to be implemented for HRP-`mk` / HRP-`md` forked-BCH plumbing, since these are NOT codex32 and `rust-codex32 v0.1.0`'s `interpolate_at` doesn't generalize directly). Why ms1 ships first: BIP-93 §"Generating Shares" prescribes the algorithm; migration contract already locked at v0.1 emission; highest-value use case (single-point-of-compromise resolution).
- **Worked-example transcripts** (3 new):
  - `mnemonic-bundle-bip84-abandon` — full single-sig BIP-84 bundle emission with multi-section stdout + engraving-card stderr + secret-on-stdout warning.
  - `mnemonic-verify-bundle-bip84-abandon` — 10-line `ok` log against the v0.3.0 bundle.
  - `mnemonic-bundle-bip388-collision` — 2-of-2 distinct-key violation, exit 2.
- **Back-matter accretion**:
  - Glossary: +16 entries (57 → 73; SPEC §7 A4 v1.0 target ≥80).
  - Index table: +41 rows (159 → 200; SPEC §7 A5 v1.0 target ≥250).
  - BIP cross-reference: extended existing rows for BIP-32, BIP-39, BIP-84, BIP-93, BIP-388, BIP-389 with §IV.* citations.
  - Release-history row for `tech-manual-v0.2.0` (per user directive, this table tracks only the manual's own cuts).
- **cspell**: new word allow-list entries (`subkeys`, `multiset`, `miscategorized`, `misgrouped`, `unmappable`).

### Notable corrections folded inline during the cut

- Phase 3.1: stdout `2` vs `4` `schema_version` corrected (chapter inherited a v0.2 doc-comment value that lagged HEAD's `"4"` emit at `cmd/bundle.rs:572`); md1 4-hex vs mk1/ms1 5-hex `chunk_set_id` asymmetry made explicit; `synthesize.rs:593-725` line range corrected from inverted `:725-593`.
- Phase 3.2: `cs[i].path` type corrected from `Option<DerivationPath>` to `DerivationPath` (CosignerKeyInfo struct has it un-wrapped); BIP-388 raw-string vs. typed-equality bifurcation narrowed to the xpub-slot edge case (phrase/entropy slots cannot reach it because template.rs synthesizes its own `'`-notation paths); ms1 Case-3 `decode_error` table clarified as `format!("{:?}", e)` Debug-repr.
- Phase 3.3: §IV.3 Reason-1 corrected — the chapter originally claimed `rust-codex32 v0.1.0` already exposes a `Codex32String::shares` API for threshold-share generation; it doesn't. The crate's public surface offers only `interpolate_at` (Lagrange-interpolation reconstruction from an existing share set). Share generation is novel implementation work (BIP-93 specifies the math at §"Generating Shares"; only the implementation is new). `Threshold` type call-out in the v0.2 `encode_shares` signature restored.
- Phase 3.4: BIP cross-reference table errors corrected — BIP-32's §IV.3 → §IV.2; BIP-93's spurious §IV.2 removed; BIP-39's §IV.2 added. Glossary alphabetical sort fixed for 3 entries (`cosigner-mapping diagnostic`, `multiset`, `secret-bearing slot`).
- Phase 3.5 (final whole-cut): §IV.2 Invariant 2 disclosure that the multiset semantics apply to the multisig path only (single-sig uses `.first()` at `verify_bundle.rs:1280-1355`); release-history v0.3.0 row added; index-table rows for `abandon test mnemonic` and `BIP-389` extended with §IV.1 (Bundle Anatomy) references.

### SPEC §7 acceptance criteria (v0.3 cut)

- **A1 (cumulative)** — bundle anatomy + anti-collision + future shares all covered ✓
- **A4** — glossary 73 entries (≥72) ✓
- **A5** — index 200 rows (≥199) ✓
- **A6** — Pandoc TOC covers Part IV chapters ✓
- **A8** — 11/11 worked-example transcripts verified by `tests/verify-examples.sh` ✓
- **A10** — PDF 145pp (≥40pp soft floor) ✓
- **A11** — `make pdf` reproducible: byte-identical across two clean `SOURCE_DATE_EPOCH=1746921600` builds (SHA256 `b888fcf55c6d4078f9b5d15d9bd2032e50822fbb33918499f2adcfa21b848a11`, 574,086 bytes) ✓

### Open FOLLOWUPS (carried into v0.4)

Two cross-repo entries open in `docs/technical-manual/FOLLOWUPS.md`, both targeting md1 work:
- `cross-repo md1-wsh-multi-unsorted-integration-test` (filed Phase 2.2).
- `cross-repo md1-bip49-integration-test` (filed post-v0.2-tag).

Both resolve in lockstep when md1 work next opens. No new FOLLOWUPS filed at this tag time per `feedback_zero_followups_from_release_cycles`.

## tech-manual [0.2.0] — 2026-05-11

Part III added — address derivation end-to-end. 119pp PDF (was 97pp at v0.1 close, +22pp). Tag `tech-manual-v0.2.0`.

### Added

- **Part III — Address derivation** (3 chapters):
  - **§III.1 Descriptor → miniscript → address.** The three-tier model (template → derivation → script → address); BIP-388 wallet-policy framing; origin path vs. use-site path; Shared/Divergent origin modes under header bit 4; pre-flight validation (4 rejection branches). Worked example: BIP-84 abandon mnemonic → `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`. Includes 2 mermaid figures (three-tier model; Shared vs. Divergent).
  - **§III.2 Shape coverage.** Exhaustive enumeration of the seven BIP-388-parseable shape buckets handled by md-codec v0.32's AST → `miniscript::Descriptor` converter: wpkh/pkh/sh(wpkh); tr(K); tr(NUMS, ...); tr(@0, <leaf>); tr(@0, {leaf_a, leaf_b}); sh(...); wsh(<miniscript>). Off-limits shapes documented (hardened public derivation, `Tag::RawPkH`, `Tag::SortedMultiA`, top-level wrappers in miniscript context). Every shape's worked address-derivation is grounded in `descriptor-mnemonic/crates/md-codec/tests/address_derivation.rs`. Includes 1 mermaid figure (converter pipeline).
  - **§III.3 Network and addressing.** Five `bitcoin::Network` variants; encoding-vs-script asymmetry; SLIP-0132 prefix interactions (cross-referenced to `mnemonic convert` in end-user manual; not duplicated).
- **Worked-example transcript** (1 new): `md1-address-bip44-receive0` — BIP-44 pkh → `1LqBGSKuX5yYUonjxT5qGfpUsXKYYWeabA` against abandon mnemonic, via `md address --template ... --key @0=<bip44 xpub>`.
- **Back-matter accretion**:
  - Glossary: +26 entries (31 → 57; SPEC §7 A4 target ≥80 at v1.0).
  - Index table: +63 rows (96 → 159; SPEC §7 A5 target ≥250 at v1.0).
  - BIP cross-reference: extended existing rows with §III citations; new rows for BIP-44, BIP-49, BIP-86, BIP-379. New non-BIP cross-references section for SLIP-0132.
  - Release-history row for `tech-manual-v0.1.0` (per user directive, this table tracks only the manual's own cuts).
- **cspell**: new word allow-list entries (`Timelock`, `taptree`, `PSBT`, `merkle`, `hardenedness`, `CKDpub`); new ignore-regexes (bech32 mainnet/testnet/regtest addresses; legacy base58 P2PKH/P2SH).

### Notable corrections folded inline during the cut

- §III.1 explicitly cross-references §II.1's history note on the retired v0.10 `Tag::OriginPaths = 0x36` to correct any reader inheriting that stale value from the SPEC (which was drafted before v0.11 retirement).
- §III.2 originally claimed the NUMS `key_index` field was wire-present even when `is_nums = 1`; the wire layout actually suppresses it. Corrected at Phase 2.2 close; verified against `descriptor-mnemonic/design/SPEC_v0_30_wire_format.md §7.2`.
- §III.3 originally listed four `bitcoin::Network` variants; `Network::Testnet4` is a distinct fifth variant. Corrected at Phase 2.3 close.

### SPEC §7 acceptance criteria (v0.2 cut)

- **A1 (partial)** — seven BIP-388-parseable buckets walk-through covered ✓
- **A4** — glossary 57 entries (≥50) ✓
- **A5** — index 159 rows (≥150) ✓
- **A6** — Pandoc TOC covers Part III chapters ✓
- **A8** — 8/8 worked-example transcripts verified by `tests/verify-examples.sh` ✓
- **A10** — PDF 119pp (≥40pp soft floor) ✓
- **A11** — `make pdf` reproducible: byte-identical across two clean `SOURCE_DATE_EPOCH=1746921600` builds ✓

### Filed FOLLOWUPS

One cross-repo FOLLOWUP filed mid-cycle (Phase 2.2): `cross-repo md1-wsh-multi-unsorted-integration-test` requesting a paired-derivation test for unsorted `wsh(multi(...))` in `descriptor-mnemonic/crates/md-codec/tests/address_derivation.rs`. Routes through `node_to_miniscript::<Segwitv0>` (`Terminal::Multi` arm); cited at §III.2 but untested.

## tech-manual [0.1.0] — 2026-05-11

First releasable cut of the m-format constellation technical manual. Parts I + II + back-matter skeleton; 100pp PDF (`docs/technical-manual/build/m-format-technical-manual.pdf`). Tag `tech-manual-v0.1.0`.

### Added

- **Part I — Foundations** (4 chapters): Introduction (§I.1), The m-format Star (§I.2), codex32 and BCH (§I.3), Conventions and Notation (§I.4). Includes 1 mermaid figure (constellation star).
- **Part II — Wire formats** (3 chapters): md1 §II.1, mk1 §II.2, ms1 §II.3 — full bit-level treatment of each format with worked encode/decode examples. Includes 3 mermaid figures (mk1 encode pipeline, mk1 bytecode layout, ms1 encode + recovery pipeline).
- **Back-matter skeleton** (6 chapters): glossary (31 entries), index table (110 entries), release history (11 rows), BIP cross-reference (11 BIPs), troubleshooting (per-format error → cause → remediation), bibliography.
- **Worked-example transcripts** (6): 2 md1 (`md1-encode-wpkh-basic`, `md1-decode-wsh-multi-2of3`), 2 mk1 (`mk1-decode-bip48-multisig`, `mk1-decode-bip84-no-fingerprint`), 2 ms1 (`ms1-encode-12word-abandon`, `ms1-decode-12word-abandon`). All verified by `tests/verify-examples.sh` against HEAD release binaries.
- **Build pipeline**: cloned-and-adapted from `docs/manual/`. Targets `pdf`, `pdf-docker`, `lint`, `verify-examples`. SOURCE_DATE_EPOCH-byte-identical reproducibility verified across clean rebuilds.
- **Lint** (`tests/lint.sh`): 6 checks — markdownlint, cspell, lychee, api-surface-coverage stub (populated at Phase 4.5), glossary-coverage, index bidirectional.

### Scope

- Wire formats documented exhaustively at bit-level depth.
- Part III (address derivation) deferred to `tech-manual-v0.2.0`.
- Part IV (bundle formation) deferred to `tech-manual-v0.3.0`.
- Part V (Rust API reference) deferred to `tech-manual-v0.4.0`.
- Full back-matter completion (glossary ≥80, index ≥250, BIP cross-ref complete, bibliography complete, troubleshooting complete) deferred to `tech-manual-v1.0.0`.

### SPEC §7 acceptance criteria (v0.1 cut)

A4 (glossary ≥30): **31 ✓**. A5 (index ≥100): **110 ✓**. A6 (TOC auto-generated): ✓. A8 (transcripts verified): **6/6 ✓**. A10 (PDF ≥40pp soft floor): **100pp ✓**.

### Sibling-repo coverage tracked at this cut

md-codec v0.32.0, md-cli v0.4.3, mk-codec v0.2.2, ms-codec v0.1.1, ms-cli v0.1.0, mnemonic-toolkit v0.8.0.

### Notes

- Pre-Draft, AI + reference implementation, awaiting human review. Wire-format claims, BCH-math claims, canonicality rules, and cross-card invariants may be wrong; cross-implementation work is the most valuable bug-finding activity at this stage.
- Two open FOLLOWUPS at tag time, tracked via `docs/technical-manual/FOLLOWUPS.md`: `bibliography-bip-author-canonical-verification` (tier `tech-manual-v1.0-nice-to-have`) and `troubleshooting-mk-codec-variant-coverage-audit` (tier `tech-manual-v0.4`). Both filed during mid-cycle Phase 1.5 per the cycle-discipline rules.

## mnemonic-toolkit [0.18.1] — 2026-05-16

### Fixed — revert v0.18.0 rows 10/11 `disable_options` emissions (UX flaw)

v0.18.0 introduced two `disable_options` rules on bundle's
`--template` flag: row 10 (`slot_count_gte: 2` → disable single-sig
options) + row 11 (`slot_count_eq: 1` → disable multisig options).
**Row 11 was a design flaw**: `slot_count == 1` is the natural
*transient* state when a user is building UP to multisig (slots get
added one at a time, passing through 1 on the way to 2+). Disabling
multisig templates at that transient state prevents the user from
selecting their intended template before completing slot setup —
the user can only ever pick from single-sig, even when they meant
to build a multisig wallet. Row 10 suffered the symmetric issue
during multisig→single-sig template switches (slot_count >= 2
disabled single-sig until the user removed slots first).

v0.18.1 reverts both rules. The template/slot_count mismatch UX
migrates to a **GUI-internal warning banner** in `mnemonic-gui
v0.7.2` (Option A pattern matching the v0.7.1 row-8 slot-contiguity
check) — render the dropdown normally; show an inline warning when
the chosen template + slot_count combination would fail CLI rows
10/11 at runtime, with suggested-fix text. The CLI remains the
authoritative gate per §6.6 rows 10/11 stderr.

### Changed

- `crates/mnemonic-toolkit/src/cmd/gui_schema.rs::bundle_conditional_rules`:
  two `ConditionalRule` entries deleted (row 10 + row 11 disable_options).
  Bundle rule count `13 → 11` (back to v0.17.1 baseline).
- `crates/mnemonic-toolkit/tests/cli_gui_schema_v4_extensions.rs`:
  rewritten. Old assertions (`bundle_emits_disable_options_rule_row_10/11_*`,
  `disable_options_wire_shape_uses_inner_values_key`,
  `v4_schema_includes_all_v3_cycle_surfaces`) replaced with
  `bundle_emits_no_disable_options_rules_after_v0_18_1_rollback`
  (anti-regression guard against re-introduction) +
  `v4_schema_version_pinned_after_v0_18_1_rollback` +
  `bundle_conditional_rules_count_is_eleven_at_v0_18_1`. Bare-string
  + pin_value v4 round-trips preserved.
- `crates/mnemonic-toolkit/tests/cli_gui_schema_conditional_rules.rs`:
  `bundle_emits_conditional_rules` count assertion `13 → 11`.

### Schema version

`4` (unchanged from v0.18.0). The `disable_options` Visibility
variant remains a defined v4 grammar surface (`§6.10.3`); no rule
emits it after the rollback. Future cycles may identify contexts
where greying dropdown options serves UX better than an inline
warning; the grammar stays available.

### Companion

GUI-side bump in lockstep: `mnemonic-gui v0.7.2` drops the matching
`bundle()` conditional-fn visibility pushes + adds the warning-
banner helper + adds the slot-grid-adjacent warning render.

### Closes

(No FOLLOWUP closures; this is a same-cycle bugfix for a v0.18.0
design issue surfaced by user report.)

## mnemonic-toolkit [0.18.0] — 2026-05-16

### Added — SPEC §6.10 v3-cycle extensions to `gui-schema` JSON

Schema `version` bumps `3 → 4`. v3 cycle extensions to the SPEC §6.10
conditional-applicability projection close the v2-cycle deferred rows
9/10/11 partition (effect side of the §6.10.7 closing list):

- **VisibilityProjection (§6.10.3)** gains one new tagged-object variant:
  `disable_options { values: Vec<String> }` — applies to Dropdown
  FlagKind only; greys out specific dropdown options at render time
  while leaving argv emission unchanged. Wire shape mirrors the v3
  `pin_value` inner-key convention:
  `{"disable_options": {"values": [<string>, ...]}}`.
- **Emission table (§6.10.4)** gains a `disable_options(values)` row
  with explicit "no impact" argv contract + accompanying prose. Stale
  state values whose dropdown option was disabled mid-session still
  emit on argv; CLI mode-violation rows 10/11 are the residual safety
  net. Silently suppressing them would create silently-lost-user-value
  bugs.
- **Bundle rules (`bundle_conditional_rules`)** gain two entries —
  total 11 → 13:
  - Row 10 (`slot_count_gte: 2`): disables single-sig template options
    (`bip44`, `bip49`, `bip84`, `bip86`) on `--template`.
  - Row 11 (`slot_count_eq: 1`): disables multisig template options
    (the 6-value set from `CliTemplate::is_multisig()`) on `--template`.
- **Mapping table (§6.10.7)** flips 3 rows from deferred → encoded:
  bundle row 9 (closes GUI-side via `NumberMax::FromSlotCount`
  FlagKind extension — no toolkit wire-format change), row 10, row 11.
  Legend gains `ENCODED v3` + `ENCODED v3 (GUI-internal)` cycle
  prefixes.
- **Row 9 N-equivalence note (§6.6)** added — for GUI projection
  authors, `N` in the row-9 stderr literal equals `slot_count`
  (rows 10/11 reject mixed configs before row 9 fires, so the
  equivalence holds in valid configurations).

### Schema version

`3 → 4`. The bump is **additive** but v3 GUI consumers (v0.6.x) fail
CLOSED on the new `{"disable_options": ...}` tagged-object variant
(per the v0.6.0 custom `Deserialize` impl at
`mnemonic-gui/src/schema_check.rs::VisibilityProjection` which only
accepts bare-string + `pin_value`). Lockstep release with
`mnemonic-gui-v0.7.0` is mandatory.

### Closes FOLLOWUPS

- `gui-schema-effect-on-dropdown-options-vocab` (cross-repo) — Batch
  B-1 lands the toolkit emitter side. Row 9 closes GUI-side without a
  toolkit wire change (single-consumer pragma; promotable to a
  toolkit-emitted Effect if a second `gui-schema` consumer ever
  appears).

### Verification

- TDD discipline: new test file
  `tests/cli_gui_schema_v4_extensions.rs` (7 cells) RED against
  unmodified source (5 of 7 expected failures: row-10 + row-11 rule
  shape, wire-shape inner-key, count==13, version==4); 2 of 7
  already-passing back-compat cells (bare-string + pin_value
  round-trip on v4 doc). GREEN after `VisibilityProjection`
  extension + `bundle_conditional_rules` additions + version bump.
- Allowlist extensions: `predicate_kinds_emitted_in_snake_case`
  gains `slot_count_{eq,gte,lte}` (v3-cycle Predicate variants now
  actually emitted); `effect_visibilities_are_in_allowed_set` gains
  the `disable_options` tagged-object arm with inner-payload shape
  validation.
- `cargo test --offline --workspace`: 30 test binaries pass, no
  regressions vs v0.17.1.

### Companion

GUI-side bump in lockstep: `mnemonic-gui v0.7.0` re-pins to
`mnemonic-toolkit-v0.18.0` and adds the `disable_options` consumer +
GUI-internal `NumberMax::FromSlotCount` FlagKind extension closing
row 9.

## mnemonic-toolkit [0.17.1] — 2026-05-16

### Fixed — drop spurious `meta.template_groups` from `derive-child` gui-schema

`crates/mnemonic-toolkit/src/cmd/gui_schema.rs::build_subcommand_meta`
previously listed `derive-child` in its match arm and emitted a
`meta.template_groups` block for that subcommand. But `derive_child.rs`
has zero `--template` references — the block was spurious. v0.17.1
removes `derive-child` from the match arm so the meta block is emitted
only for the three subcommands that actually consume `--template`:
`bundle`, `verify-bundle`, `export-wallet`.

The bug was silent (no GUI consumer reads derive-child's meta block) but
the emitted JSON was wrong; the matching SPEC §6.10.8 prose enumerated
derive-child in error; and the toolkit test
`derive_child_emits_meta_template_groups` enshrined the wrong invariant.

Surfaced by the `mnemonic-toolkit-v0.17.0` cycle-close opus reviewer
audit (confidence 95). Tracked at FOLLOWUP
`gui-schema-derive-child-meta-template-groups-spurious` (resolved at
this release).

### Changed

- `crates/mnemonic-toolkit/src/cmd/gui_schema.rs`:
  `build_subcommand_meta` match arm drops `| "derive-child"`.
- `design/SPEC_mnemonic_toolkit_v0_5.md` §6.10.8 paragraph 2:
  `derive-child` removed from the template-consumer enumeration;
  parenthetical noting the v0.17.1 correction added.
- `crates/mnemonic-toolkit/tests/cli_gui_schema_v3_extensions.rs`:
  cell `derive_child_emits_meta_template_groups` deleted;
  replacement negative-cell `derive_child_omits_meta_template_groups`
  added as a regression guard against re-introduction.

### Verification

- TDD discipline: negative cell ran RED against unmodified source
  (panic message showed the spurious `multisig: [...], single_sig:
  [...]` block); GREEN after the match-arm fix.
- `cargo test --offline --workspace`: 30 test binaries pass, no
  regressions vs v0.17.0 (was 30 at v0.17.0; same count + one cell
  replaced in v3_extensions).

### Companion

GUI-side bump in lockstep: `mnemonic-gui v0.6.1` re-pins to
`mnemonic-toolkit-v0.17.1` in both `pinned-upstream.toml [mnemonic].tag`
and `Cargo.toml [dependencies] mnemonic-toolkit`. The v0.6.1 patch also
folds two GUI-only defense-in-depth findings from the same cycle-close
reviewer audit (canary tests for `#[serde(other)]` behavior + drift gate
per-subcommand floors + `--slot` PinValue debug_assert).

## mnemonic-toolkit [0.17.0] — 2026-05-16

### Added — SPEC §6.10 v2-cycle extensions to `gui-schema` JSON

Schema `version` bumps `2 → 3`. v2 cycle extensions to the SPEC §6.10
conditional-applicability projection landed in v0.16.0:

- **Predicate AST (§6.10.2)** gains three new tagged-union kinds:
  `slot_count_eq` / `slot_count_gte` / `slot_count_lte`, each carrying
  a `value: N` payload. Predicate semantics: the form's total slot
  count (= `FormState::slot_count()` on the GUI side, exposed in v2)
  compared to literal N. Predicate-machinery only at this release —
  no v0.17 emitted rule consumes them; consumers exist for future
  rule additions per the §6.10.7 closing list (rows 9/10/11 deferred
  pending a dropdown-option-disable Effect grammar).

- **Effect Visibility (§6.10.3)** gains the `pin_value` variant with
  a tagged-object wire shape:
    `{"visibility": {"pin_value": {"value": <JSON>}}}`.
  Unlike `hidden`/`disabled` (suppress emission), the GUI emits
  `--name <V>` using the pinned value V regardless of any pre-pin
  user-typed value, per the §6.10.4 emission-mapping table.
  Closes the v1-cycle DEFERRED row 12 entry in §6.10.7
  (`DESCRIPTOR_WITH_NONZERO_ACCOUNT` → `--account → pin_value(0)`).

- **Per-subcommand `meta` block (§6.10.8 — NEW)** with initial
  `template_groups: { single_sig, multisig }` field. Emitted for
  subcommands that consume `--template` (bundle / verify-bundle /
  export-wallet / derive-child). Source-of-truth:
  `CliTemplate::is_multisig()` in `crates/mnemonic-toolkit/
  src/template.rs:46-56`. Empty meta serializes as omitted (no
  `meta` key in JSON) so subcommands without meta surfaces remain
  byte-identical with v2 docs.

Wire-format details:

- Bare-string Visibility shapes preserve their v2 wire layout
  bit-for-bit (v3 back-compat per SPEC §6.10.6). The new
  `pin_value` form uses the tagged-object shape only.
- v2 consumers encountering a tagged-object `visibility` or a
  `slot_count_*` predicate will fail to deserialize that specific
  rule; the toolkit emits new-content rules at the END of each
  subcommand's `conditional_rules` array so v2 consumers can
  recover the prefix.
- In practice the v3 consumer is `mnemonic-gui v0.6.0` shipped in
  lockstep; v2-consumer back-compat is theoretical concern only
  since the `pinned-upstream.toml` mechanism keeps consumer-version
  in sync with producer-version.

### Added — 1 new rule in `bundle.conditional_rules`

- **Row 12 — `DESCRIPTOR_WITH_NONZERO_ACCOUNT`**: when `--descriptor`
  is present, projects `--account → pin_value(0)`. The GUI coerces
  any nonzero user-typed account value to 0 and emits
  `--account 0` regardless of widget input, mirroring the CLI's
  byte-exact rejection at `bundle.rs::mode_text::
  DESCRIPTOR_WITH_NONZERO_ACCOUNT`.

Existing rules (descriptor mutex, single-sig template
disable-pairs, etc.) ship unchanged.

### Changed

- `crates/mnemonic-toolkit/src/cmd/gui_schema.rs`:
  - `Schema.version`: `2 → 3`.
  - `Predicate` enum: 3 new variants (`SlotCountEq`,
    `SlotCountGte`, `SlotCountLte`), all `#[allow(dead_code)]`.
  - `VisibilityProjection` enum: new `PinValue { value:
    serde_json::Value }` variant. **Dropped `Copy` derive**
    (Value isn't Copy). Manual `Serialize` impl preserves
    bare-string shape for unit variants; emits tagged-object for
    PinValue.
  - `Subcommand` struct: new `meta: BTreeMap<String,
    serde_json::Value>` field, omitted from JSON when empty via
    `#[serde(skip_serializing_if = "BTreeMap::is_empty")]`.
  - New helpers: `multisig_template_values()`,
    `build_subcommand_meta(name)`.

### SPEC patch

- `design/SPEC_mnemonic_toolkit_v0_5.md` §6.10 extended with
  v0.6-cycle banner (preamble); §6.10.2 (slot_count predicates);
  §6.10.3 (`pin_value` Visibility + wire-format details);
  §6.10.4 (Visibility-to-emission mapping table);
  §6.10.6 (v2→v3 bump prose + back-compat guarantee);
  §6.10.7 (row 12 flipped DEFERRED → ENCODED v2; closing list
  regrouped into "closed-in-v2" / "predicate-machinery-available"
  / "still-deferred" partitions);
  §6.10.8 — NEW — meta.template_groups documentation.

### Companion

`mnemonic-gui v0.6.0` (cross-repo lockstep) consumes the schema-v3
extensions: mirrors the Predicate variant additions, mirrors the
`pin_value` Visibility variant on the consumer side, retires the
hand-coded `SINGLE_SIG_TEMPLATES: &[&str]` const in favor of
reading `meta.template_groups.single_sig`, and adds `FormState::
slot_count()` accessor. See `mnemonic-gui CHANGELOG.md [0.6.0]`.

### Verification

- `cargo test --offline --workspace`: all suites pass, 0 failures.
- `cargo test --test cli_gui_schema_v3_extensions --offline`:
  8 passed (new tests covering pin_value rule shape +
  meta.template_groups per template-consuming subcommand).
- Existing v1-era assertions (rule shapes, priority order,
  Predicate kind enumeration) preserved as regression guards
  with v3 folds for the new Visibility shape.

### Closes FOLLOWUPS

The cycle-close commit (master HEAD after v0.6.0 release) flips
these toolkit-side entries from `open` to `resolved <SHA>`:

- `gui-schema-template-groups-meta-field` (cross-repo) — meta
  block source-of-truth shifts to toolkit; GUI retires
  `SINGLE_SIG_TEMPLATES` const.
- `gui-schema-numeric-flag-value-pin-effect` (cross-repo) —
  `pin_value` Visibility variant + row 12 rule emission.
- `gui-schema-runtime-conditional-projection` (cross-repo) —
  partial: predicate-machinery (slot_count_*) shipped; full
  encoding still deferred per §6.10.7 closing list.

Also files NEW FOLLOWUPS this cycle:

- `gui-schema-effect-on-dropdown-options-vocab` — dropdown-option-
  disable Effect grammar needed to close §6.6 rows 9/10/11
  encoding. Unblocked by this cycle's predicate-machinery.
- `gui-schema-cross-slot-predicate-projection` — relational
  predicate types (cross-slot equality, all-distinct) needed to
  close §6.6 rows 8/13/14.

## mnemonic-toolkit [0.16.0] — 2026-05-16

### Added — SPEC §6.10 conditional-applicability projection in `gui-schema` JSON

`mnemonic gui-schema` JSON gains a per-subcommand
`conditional_rules: [ConditionalRule]` array that projects the
CLI's §6.6/§6.9 mutex/conditional rule manifest into a
machine-readable form. The GUI overlay consumes this via the
`mnemonic-gui v0.5.0` companion release; drift between the JSON
projection and the GUI's hand-coded conditional fns is enforced
by the new drift gate at `mnemonic-gui/tests/
gui_schema_conditional_drift.rs`.

The schema `version` bumps `1 → 2`. The bump is **additive**:
v1 consumers that parse only the per-flag set (name, kind,
choices) and ignore unknown fields continue to work on v2 docs.
The GUI's `parse_gui_schema_json` relaxes its version gate from
`!= 1` to `< 1` to honour this in lockstep.

Predicate AST (SPEC §6.10.2 — tagged JSON union, snake_case
kind values):

- `flag_present` — flag has a non-empty value
- `dropdown_value_in` — flag's Dropdown value is in a set
- `composite_node_is` — flag's Composite node token matches
- `positional_present` — positional[index] is non-empty
- `all_of` / `any_of` / `not` — boolean combinators

Effect (SPEC §6.10.3): `(flag, hidden | disabled | required)`.
`visible` is the implicit default and never appears as an
Effect value.

v1 rule coverage (~17 enforceable rules across 5 subcommands):

- bundle: 10 rules (template required-unless,
  descriptor↔descriptor-file mutex × 2 dirs, passphrase mutex
  × 2 dirs, template/threshold/multisig-path-family disabled
  under descriptor-mode, threshold/multisig-path-family
  disabled under single-sig template)
- verify-bundle: 10 rules (mirror of bundle's descriptor +
  passphrase rules; bundle-json XOR --ms1/--mk1/--md1 × 3;
  threshold disabled under single-sig; template disabled
  under descriptor)
- export-wallet: 6 rules (template ↔ descriptor mutex × 2
  dirs; taproot-internal-key Required + Disabled
  rules; threshold/multisig-path-family disabled under
  single-sig)
- convert: 4 rules (passphrase + bip38-passphrase mutex pairs)
- derive-child: 3 rules (passphrase mutex + dice-sides
  Required when application = dice)

First-rule-wins emission order per SPEC §6.10.4: rules
targeting the same flag are emitted with more-specific
predicates first (e.g., bundle's `--threshold`: descriptor-mode
rule precedes single-sig-template rule).

Runtime/dynamic rules deferred to FOLLOWUP
`gui-schema-runtime-conditional-projection` (slot-count
predicates, BIP-388 distinct-key, per-`@N` annotation
consistency).

### SPEC

New §6.10 (subsections 1–7) added to
`design/SPEC_mnemonic_toolkit_v0_5.md` as the canonical home for
the GUI projection, alongside §6.6 (template-mode mode-violation
ladder) and §6.9 (byte-exact error reference). §6.6 remains
untouched per scope-isolation discipline; the pre-existing SPEC
drift between §6.6 enumeration and the v0.3-NEW descriptor-mode
consts at `bundle.rs:120-129` is tracked independently at
FOLLOWUP `spec-v0_5-missing-v0_3-descriptor-mode-rows`.

### Tests

32 new test cells in
`crates/mnemonic-toolkit/tests/cli_gui_schema_conditional_rules.rs`:

- Schema version bump (v2)
- Per-subcommand rule counts
- Predicate priority order per target flag
- Predicate AST `kind` vocabulary lint
- Effect visibility vocabulary lint
- Rationale + spec_ref presence per rule
- Single-sig dropdown values match `CliTemplate::is_multisig()`
  source-of-truth

Full workspace test: 1001 passed, 0 failed, 8 ignored.

### Companion / lockstep

- `bg002h/mnemonic-gui v0.5.0` ships the consumer side:
  `parse_gui_schema_conditional_rules` parser, ~14 new
  conditional rules in `src/form/conditional.rs`,
  `assemble_argv` visibility gate at
  `src/form/invocation.rs` (suppresses Hidden + Disabled
  flags from emission), removal of the
  `--multisig-path-family bip87` default seed at
  `src/main.rs:203`, and the new drift gate test.
- toolkit FOLLOWUP `gui-schema-conditional-rules-v1` flips
  open → resolved at v0.16.0 tag commit.
- mnemonic-gui FOLLOWUP
  `gui-conditional-applicability-drift-fix` flips open →
  resolved at v0.5.0 tag commit.
- mnemonic-gui FOLLOWUP
  `gui-bundle-multisig-flags-conditional` (the motivating
  bug; surfaced 2026-05-15 during the manual-gui v1.0 cycle)
  resolved at mnemonic-gui v0.5.0 cycle commits.

### Predecessor

- `mnemonic-toolkit v0.15.0` (md-codec catchup; wire-format
  clean break) is the unchanged baseline.

## mnemonic-toolkit [0.15.0] — 2026-05-16

### Breaking — `md-codec` catchup v0.16.1 → v0.33.1; `mk-codec` catchup v0.2.1 → v0.3.0

Unblocks `cargo publish`: both sibling-codec deps swap from git tags
to crates.io versions. Tracking issue + plan:
`design/PLAN_md_codec_catchup_v0_15.md`.

#### Wire-format clean break (md-codec v0.30+)

The md1 wire payload changes shape — pre-v0.15 toolkit-emitted md1
strings are forward-incompatible. v0.15 md1 decoders return
`Error::WireVersionMismatch { got: 1 }` against any md1 emitted by
the toolkit at v0.14.x or earlier. **Engraved bundles emitted by
v0.14.x or earlier are NOT readable by v0.15+** — they remain
self-contained (the ms1 + mk1 + pre-v0.30 md1 cards verify against
each other under a pre-v0.15 binary), but the toolkit's verify-bundle
will refuse a v0.30 wire-version mismatch.

This is the *intended* forward-incompatibility contract for
SPEC §6.4.5 routing — versions are a load-bearing axis.

#### `md_codec::Error` variant churn

**Removed** at md-codec v0.30 (toolkit no longer maps these):
- `ReservedHeaderBitSet`
- `UnsupportedVersion { got }` — semantic replacement is `WireVersionMismatch`
- `UnknownPrimaryTag(u8)`
- `UnknownExtensionTag(u8)`

**Removed** at md-codec v0.32:
- `UnsupportedDerivationShape` — semantic replacement is `AddressDerivationFailed { detail }`

**Added** (toolkit's `MdCodecExitCode` + `friendly_md_codec` both route exhaustively):
- `WireVersionMismatch { got }` → exit 3 via `FutureFormat` (was `UnsupportedVersion`'s old routing point)
- `MalformedHeader { detail }` → exit 2
- `TagOutOfRange { primary }` → exit 2 (replaces `UnknownPrimaryTag` + `UnknownExtensionTag`)
- `NUMSSentinelConflict` → exit 2
- `OperatorContextViolation { tag, context: ContextKind }` → exit 2
- `DecodeRecursionDepthExceeded { depth, max }` → exit 2 (added at v0.19, late-routed here)
- `AddressDerivationFailed { detail }` → exit 2

#### `Body::Variable` → `Body::MultiKeys` for multi-family tags (md-codec v0.30 §4)

`Tag::Multi`, `Tag::SortedMulti`, `Tag::MultiA`, `Tag::SortedMultiA`
previously serialized as `Body::Variable { k, children: <N PkK
leaves> }` (per-child `Tag::PkK` + `Body::KeyArg`); now serialize as
`Body::MultiKeys { k, indices: Vec<u8> }` (flat indices, `kiw` bits
per index on the wire). `Body::Variable` is now reserved exclusively
for `Tag::Thresh`. Affected sites in this release:

- `template.rs::wrapper_node` — all 3 multi-family constructors (`WshMulti`/`WshSortedMulti`, `ShWshMulti`/`ShWshSortedMulti`, `TrMultiA`/`TrSortedMultiA`).
- `parse_descriptor.rs::build_multi_node` — switched to indices-only construction.

#### `Body::Tr.is_nums: bool` field (md-codec v0.18+)

`Body::Tr` gained an explicit `is_nums` flag (SPEC §7). The toolkit
emits `is_nums: false` at all 3 construction sites (BIP-86,
`TrMultiA`/`TrSortedMultiA` wrapper, `parse_descriptor::walk_tr`).
FOLLOWUP `toolkit-trmultia-nums-internal-key` tracks the open
question of whether BIP-388 script-path-only taproot multisig wallets
SHOULD emit `is_nums: true` — out of scope for v0.15; revisit when
authoring the v1.0 BRAINSTORM scope-lock.

#### `mk-codec` v0.2.1 → v0.3.0

Additive: new `pub mod test_vectors` (toolkit doesn't consume it).
Single-line bump.

#### Fixtures regenerated

`tests/vectors/v0_1/*.txt` (16 single-sig bundles) +
`tests/vectors/v0_2/bip84-mainnet-0-false-true.txt` (self-check
fixture) regenerated against the v0.30 wire format. The md1 strings
now begin with the `md1fqn8upq…` prefix (was `md1zsx9cpq…` under
v0.16.1).

### Companion

- `descriptor-mnemonic` `md-codec` already on crates.io v0.33.1 (no companion bump needed).
- `mnemonic-key` `mk-codec` already on crates.io v0.3.0 (no companion bump needed).
- `mnemonic-gui` will pick this up at its next release cycle by bumping its `mnemonic-toolkit` dep from git tag `v0.14.2` to crates.io `0.15`.

## mnemonic-toolkit [0.14.2] — 2026-05-16

### Bug fix — v0.14.1 incomplete; lib-internal slip39 still references mlock unconditionally

v0.14.1 cfg-gated `pub mod mlock;` in `lib.rs` but missed four
call sites inside `src/slip39/mod.rs` that reference
`crate::mlock::*` directly (lines 159 + 314 for production pin
calls; tests `slip39_split_invokes_pin_pages_for_on_ems` +
`slip39_combine_invokes_pin_pages_for` on lines 604 + 632). The
v0.14.1 CI's new `lib-cross-platform` Windows job caught it; the
GUI v0.4.1 Windows build (`mnemonic-gui` run 25952017502) also
caught it once it picked up the v0.14.1 toolkit tag.

This release cfg-gates the four call sites:
- The two production `crate::mlock::pin_pages_for(&ems[..])` calls
  in `slip39_split` + `slip39_combine` get `#[cfg(unix)]`. On
  non-unix the pin is a no-op; the slip39 algorithm itself is
  platform-uniform — only the swap-protection sidecar is unix-only.
- The two `*_invokes_pin_pages_for*` tests (which read
  `crate::mlock::attempts_for_test()` to verify the pin fired) are
  cfg-gated entirely; their semantic invariant doesn't apply on
  platforms where mlock is a no-op.

Also fixes the `lib-cross-platform` job's `aarch64-unknown-linux-gnu`
failure: the job was using `dtolnay/rust-toolchain@stable` which
installed `aarch64` target to the wrong toolchain (rustup later
respected the repo-pinned `rust-toolchain.toml@1.85.0` without the
target installed). Pinned the action to `@1.85.0` to match the
toolkit's rust-toolchain.toml.

### Companion

`mnemonic-gui v0.4.2` bumps the toolkit dep tag from v0.14.1 to v0.14.2.

## mnemonic-toolkit [0.14.1] — 2026-05-16

### Bug fix — `pub mod mlock` cfg-gated for Windows

`pub mod mlock` (declared in `crates/mnemonic-toolkit/src/lib.rs`) is
now `#[cfg(unix)]`. The mlock implementation uses POSIX
`libc::mlock` / `libc::munlock` / `libc::sysconf` / `_SC_PAGESIZE` —
none of which exist in `libc`'s Windows surface. Pre-v0.14.0 the
toolkit was binary-only and the binary's Unix-only CI matrix
masked this; v0.14.0 promoted `secret_taxonomy` to public lib API
for `mnemonic-gui` consumption, transitively requiring the entire
lib to compile on every platform the GUI targets — including
Windows. `mnemonic-gui v0.4.0` CI surfaced the regression at
`https://github.com/bg002h/mnemonic-gui/actions/runs/25951528124`
(`x86_64-pc-windows-msvc` job failure).

The binary (`mnemonic` CLI) remains Unix-only; the bin target is
not compiled when downstream consumers (like the GUI) depend on
this crate as a lib dep.

### Added — `lib-cross-platform` CI job

`rust.yml` gains a `lib-cross-platform` matrix job that runs
`cargo check --lib --target <target>` for `x86_64-pc-windows-msvc`
+ `aarch64-unknown-linux-gnu`. Compile-only (no test execution);
exists specifically to catch Windows / aarch64 incompat at
toolkit-CI time rather than at downstream-consumer-CI time.
Addresses architect risk #5 from the
`secret-taxonomy-public-api-promotion` FOLLOWUP.

### Added — FOLLOWUP `secret-taxonomy-feature-gate-toolkit-lib`

Filed in `design/FOLLOWUPS.md`. Tracks the architect's risk #1
mitigation: split the toolkit lib into a default-on `cli` feature
covering the heavy modules (`mlock`, `bitcoin`/`miniscript`/`bip39`
deps) versus a default-off small-surface `secret-taxonomy`-only
feature for GUI-class consumers. Optional; the v0.14.1 cfg-gate
on `mlock` alone unblocks Windows builds.

### Companion

`mnemonic-gui v0.4.1` bumps the toolkit dep tag from v0.14.0 to
v0.14.1.

## mnemonic-toolkit [0.14.0] — 2026-05-16

### Added — `secret_taxonomy` public module

New `pub mod secret_taxonomy` exposing two `pub const &[&str]` arrays:
`SECRET_NODE_TYPES` (7 entries: `phrase, entropy, xprv, wif, ms1,
bip38, electrum-phrase`) and `SECRET_SLOT_SUBKEYS` (4 entries:
`phrase, entropy, xprv, wif`). These mirror the existing
`NodeType::is_secret_bearing` (in private module `cmd::convert`) and
`SlotSubkey::is_secret_bearing` (in private module `slot_input`)
predicates.

### Why

Architect-vetted Option A from FOLLOWUPS entry
`secret-taxonomy-public-api-promotion`. Downstream consumers — chiefly
`mnemonic-gui`'s `persistence::redact_for_persistence` — need to
identify secret-class node tokens and slot subkeys to strip them
before writing session state to disk. Prior to this release, the GUI
scraped the toolkit's private source via `syn::parse_file` at GUI
build time, with a stub fallback that emitted empty `&[]` arrays when
the upstream tree was unresolvable. The empty-array fallback was
triggered for every `cargo install --git mnemonic-gui` invocation
(cargo's sandbox has no adjacent toolkit checkout), silently disabling
the GUI's persistence-redaction filter and leaking BIP-39 phrases to
`state.json` in plaintext (`mnemonic-gui` v0.3.0..v0.3.2; HIGH severity;
patched tactically in GUI v0.3.3 with a committed-fallback in
`build.rs`, then retired structurally by this toolkit release + the
companion GUI v0.4.0).

By publishing the canonical taxonomy as a public crate API, the GUI
compile fails outright if these constants are missing — no
degradation ladder, no silent fallback. `cargo install --git
mnemonic-gui` pulls the toolkit library through cargo's normal
dependency resolver with no env-var ceremony.

### Single-source-of-truth invariant

Per-variant parity tests in `cmd::convert::secret_taxonomy_parity_tests`
(`crates/mnemonic-toolkit/src/cmd/convert.rs`) and `slot_input::tests`
(`crates/mnemonic-toolkit/src/slot_input.rs`) walk every enum variant
of `NodeType` / `SlotSubkey` and assert `V.is_secret_bearing() ==
SECRET_*.contains(&V.as_str())`.

The iteration source-of-truth is generated by a per-module
`declare_*_variants!` macro that takes a list of variant idents and
expands to BOTH a `const ALL_*_VARIANTS: &[<Enum>]` array (consumed
by the parity tests) AND a `fn _exhaustiveness_check` whose
`match v { Variant1 | Variant2 | ... => () }` body has no wildcard.
The two outputs share a single input list, so they cannot diverge.

The enforcement chain a future contributor encounters when adding a
new variant:
1. `_exhaustiveness_check`'s match becomes non-exhaustive →
   **compile error** until they extend the macro input.
2. Extending the macro input automatically adds the new variant to
   `ALL_*_VARIANTS` (same input expands to both outputs).
3. The parity test then iterates the extended array. If the new
   variant's `is_secret_bearing()` is `true` but its `as_str()` is
   not in the corresponding `SECRET_*` array, the assertion fires
   at test time.

A third test pins the intentional MiniKey exclusion (MiniKey is
included in the wider `is_argv_secret_bearing()` set but excluded
from `SECRET_NODE_TYPES` because persistence redaction uses the
narrower predicate — see SPEC §1 and the
`convert-minikey-stdout-redaction` + `secret-taxonomy-argv-superset-promotion`
FOLLOWUPs).

### Stability contract

`SECRET_NODE_TYPES` and `SECRET_SLOT_SUBKEYS` are `pub const &[&str]`
(string slices, not enum re-exports). Renaming, reordering, or
removing entries is a semver-minor event (pre-1.0 minor-axis bump).
Adding entries is additive and minor-safe.

### Affected files

- `crates/mnemonic-toolkit/src/secret_taxonomy.rs` (new)
- `crates/mnemonic-toolkit/src/lib.rs` (re-export)
- `crates/mnemonic-toolkit/src/cmd/convert.rs` (#[cfg(test)] parity tests)
- `crates/mnemonic-toolkit/src/slot_input.rs` (#[cfg(test)] parity tests)

### Companion

`mnemonic-gui` v0.4.0 (planned) consumes the new module as a
`pub use mnemonic_toolkit::secret_taxonomy::*`, deletes its `build.rs`
source-walker entirely, and ships the architect-recommended one-cycle
belt-and-suspenders overlap (retains v0.3.3 `CANONICAL_FALLBACK_*`
arrays under a compile-time assertion that they equal the toolkit's
constants).

Closes FOLLOWUPS `secret-taxonomy-public-api-promotion` (the toolkit
half of the cross-repo lockstep work).

### Reviewer trail

(Backfilled in v0.14.1 — architect-audit Important #3 surfaced that
this section was omitted at v0.14.0 ship time.)

- R0 (architect dispatch, opus): produced the Option A migration sketch
  documented in `design/FOLLOWUPS.md` entry
  `secret-taxonomy-public-api-promotion`.
- R1 (opus): **1 Critical + 6 Importants**. Critical: original
  closure+driver design for `every_*_variant()` was not load-bearing
  — a contributor extending `is_secret_bearing()` with a new variant
  could land the change with only an arm extension; the parity test
  loop would skip the new variant silently. 6 Importants spanned doc
  path correctness (`cmd::convert::tests` → `secret_taxonomy_parity_tests`),
  CHANGELOG over-claim of enforcement, missing
  `is_argv_secret_bearing()` public mirror (deferred via new
  FOLLOWUP `secret-taxonomy-argv-superset-promotion`),
  `SlotSubkey::from_token` private/`pub` asymmetry with
  `NodeType::from_token`, `lib.rs` missing stability-contract
  sentence, and a redundant CHANGELOG cite that R1 self-withdrew.
- R2 (opus): LOCK with-1-folded. Critical was improved but left a
  residual gap (the count-pin assertion could not catch arm-only
  extension because driver-list and `EXPECTED_VARIANT_COUNT` were
  separately hand-maintained).
- R3 fold (informal): replaced closure+driver entirely with a
  declarative `declare_*_variants!` macro that emits BOTH the
  `ALL_*_VARIANTS` array AND a non-wildcard `_exhaustiveness_check`
  match from a single input list — the two outputs cannot diverge by
  construction. 5/5 secret_taxonomy tests green; full suite 983
  passed / 0 failed.

## mnemonic-toolkit [0.13.1] — 2026-05-15

Patch: enumerate accepted values in `--help` output for `convert`,
`bundle`, `verify-bundle`, `derive-child`, and `export-wallet`
subcommands. User-reported gap: `mnemonic convert --from <FROM> --to
<TO>` `--help` did not list the 13 accepted node types; the GUI
dropdown was the only discoverable enumeration. Doc-comment-only
patch — no flag behavior change, no manual updates required (mirror
invariant gates flag presence, not help-text content).

Affected flags:
- `convert`: `--from`, `--to`, `--xpub-prefix`, `--electrum-version`,
  `--electrum-language`, `--script-type`, `--path`. `--to` now also
  uses `PossibleValuesParser` + `value_delimiter = ','` (previous
  comma-separated parsing preserved via clap's `value_delimiter`).
  `gui-schema` for `--to` now emits `kind: "dropdown"` (was `"text"`)
  — a mirror improvement, not a contract break.
- `bundle`: `--slot` 7-subkey grammar (phrase, entropy, xpub,
  fingerprint, path, wif, xprv); empty `--passphrase`, `--json`,
  `--no-engraving-card`, `--privacy-preserving`, `--self-check`,
  `--threshold` descriptions filled in.
- `verify-bundle`: `--passphrase`, `--ms1`, `--mk1`, `--md1`, `--json`
  descriptions filled in.
- `derive-child`: `--from` node tags (xprv, phrase); `--application`
  full 9-app enumeration including SPEC §7 REFUSED set (dice, rsa,
  rsa-gpg).
- `export-wallet`: `--slot` 7-subkey grammar (mirrors `bundle`);
  `--taproot-internal-key` two forms (`nums`, `@N`).

Companion: `descriptor-mnemonic@md-cli-v0.5.2` filled the same gap on
`md decode --json` and `md inspect --json` (the only `md` flag with an
empty description before this patch).

Mechanically: switched the multi-line enumerations to
`verbatim_doc_comment` so clap preserves the aligned plain-text
formatting (clap-derive collapses markdown bullets by default).

## mnemonic-toolkit [0.13.0] — 2026-05-14

New feature: `mnemonic slip39` subcommand (`split` + `combine`
sub-subcommands). Trezor SLIP-0039 K-of-N threshold share-splitter
for cryptocurrency seeds. Splits a master secret (BIP-39 phrase or
raw entropy of 16/20/24/28/32 bytes) into groups × members of SLIP-39
mnemonic shares; ANY K-of-N subset of shares (per the configured
group + member thresholds) reconstructs. Unlike v0.12.0's `seed-xor`
all-N XOR, this IS a true threshold scheme — share loss within
threshold is recoverable; share substitution is detected by an
internal HMAC digest at `combine` time (refusal row 11). Bit-identical
to Trezor SLIP-0039 reference shares; verified against
`python-shamir-mnemonic@17fcce14` (45 fixture vectors); Trezor Model T
+ Safe family hardware compatible (Trezor One predates SLIP-39 and
uses raw BIP-39 only). Cross-impl smoke recipe in the manual chapter
validates against `shamir-mnemonic` 0.3.0 PyPI release.

Second of the two-cycle share-splitting pair planned at
`~/.claude/plans/radiant-seeking-teacup.md`; closes the K-of-N gap
v0.12.0's seed-xor explicitly deferred.

Toolkit-only major-feature minor bump (~2000 LOC of hand-rolled SLIP-39
library + ~700 LOC CLI handler + 443-LOC canonical manual chapter).
No cross-repo work in this tag; sibling-codecs (md/ms/mk) unchanged.
Adjacent `mnemonic-gui` working tree carries an uncommitted FOLLOWUP
(`slip39-gui-schema-flattening-companion`) gated on this tag shipping;
that closure lands in the GUI repo separately as PE+1.

### Added

- **`mnemonic-toolkit` library: new module
  `mnemonic_toolkit::slip39`** — full hand-rolled SLIP-39 reference
  implementation across 7 sub-modules: `gf256` (GF(2^8) arithmetic),
  `lagrange` (Shamir interpolation), `feistel` (4-round PBKDF2-keyed
  Feistel for the SLIP-39 encryption layer), `wordlist` (1024-word
  SLIP-39 wordlist), `rs1024` (Reed-Solomon-style RS1024 checksum),
  `share` (parse + render of SLIP-39 mnemonic shares), and the
  top-level `slip39_split` + `slip39_combine` driver entry points.
  Library-local `Slip39Error` with 21 variants (8 unit, 5 single-field,
  5 named-fields, 3 mixed-shape) per the v0.11.0 final-word +
  v0.12.0 seed-xor library-error precedent. Returns `Vec<Vec<Share>>`
  for split (one inner Vec per group); `Zeroizing<Vec<u8>>` for
  recovered master entropy. Memory hygiene: `Zeroizing` on
  intermediates throughout; `mlock::pin_pages_for` pins on the
  Feistel round-key buffer + per-share-emit (O(N) one-pin-per-share).
- **`mnemonic slip39 split` subcommand** with flags `--from
  <phrase=…|entropy=…> --group-threshold <G> --group <N,T>...
  [--passphrase <P>|--passphrase-stdin] [--iteration-exponent <E>]
  [--language <LANG>] [--json-out <PATH>]`. Emits SLIP-39 mnemonic
  shares to stdout, group-major with blank-line separators between
  groups. Per-share argv-leakage advisory; multi-stdin contention
  refusal; toolkit-policy refusals on `--group 1,1` (row 5) and
  `--group N,T` with `T==1 AND N>1` (row 25; python `split_ems` rule
  mirror) — smallest legal group is `--group 2,2`.
- **`mnemonic slip39 combine` subcommand** with flags `--share
  <slip39-mnemonic-or-> ... [--passphrase <P>|--passphrase-stdin]
  [--to <entropy|phrase>] [--language <LANG>] [--json-out <PATH>]`.
  Defaults to `--to entropy` (hex on stdout); `--to phrase --language
  english` emits the recovered BIP-39 master phrase.
- **JSON envelope v1** with `operation: "split"` / `"combine"`
  discriminator. SHA-pinned via env-var wedge
  (`MNEMONIC_SLIP39_TEST_RNG` + `MNEMONIC_SLIP39_TEST_IDENTIFIER`,
  always-on stderr advisory, NOT suppressible). Split envelope:
  `{schema_version, operation, identifier, iteration_exponent,
  group_threshold, groups: [{member_count, member_threshold,
  shares}]}`. Combine envelope: `{schema_version, operation,
  identifier, iteration_exponent, output_shape, entropy_hex|null,
  phrase|null}` with `entropy_hex` + `phrase` always present (one
  carries value, other is `null`, selected by `output_shape`).
- **Reused advisory class** from v0.12.0: multi-secret-on-stdout
  K-of-N parameterized variant ("SLIP-39 shares on stdout — N=<n>
  shares emitted across <g> groups (group-threshold <G>); each share
  is independently secret material; ...").
- **New advisory class**: `--iteration-exponent E` perf advisory at
  E ≥ 5 (PBKDF2 iterations ≥ 320K; ≈ 200-500ms wall-clock on
  commodity x86; Trezor's reference uses E=1 = 20000 iters as
  default).
- **Test-only env-var class**: `MNEMONIC_SLIP39_TEST_RNG` (32-byte
  hex CSPRNG override) + `MNEMONIC_SLIP39_TEST_IDENTIFIER` (decimal
  u16 identifier override; range 0..=32767 for the 15-bit field).
  Always-on `INSECURE` stderr advisory; documented in SPEC §6 +
  manual chapter §3.9.
- **Extracted helper**: `secret_advisory::warn_if_world_readable` —
  factored from the 3 `--json-out` callsites (final-word, seed-xor,
  slip39) into a single shared helper. Lockstep verified via
  `lint_world_readable_helper.rs` partial-migration guard.
- **CLI test surface**: 5 `cli_slip39_*.rs` files (~1100 LOC) +
  74 tests across happy-paths, refusals (24-class coverage; 25 with
  P3 R1 add — but row 25 reuses row 4 stem so substring assertions
  cover both), advisories (8-row coverage), JSON envelope SHA-pins,
  and stdin-route variants. Aggregate test growth post-v0.12.0:
  ~870 → 978 (+108 tests; net ~ -50 from sibling-test refactoring +
  ~ +160 from new SLIP-39 surface).
- **Lint surface bumps**: `lint_argv_secret_flags.rs` 23 → 28 rows
  (+5 for slip39); `lint_zeroize_discipline.rs` +1 row for slip39
  Zeroizing wrap evidence; `cli_gui_schema.rs` 7 → 10 user-facing
  subcommands assertion (slip39 contributes 2 leaf names via the
  gui-schema flattening fix below; seed-xor's pre-existing 2 leaves
  also surface for the first time).
- **Manual chapter `## mnemonic slip39`** in
  `docs/manual/src/40-cli-reference/41-mnemonic.md` (443 LOC): intro
  + concept signposts + synopsis + dual flag tables + 4 progressive
  worked examples (2-of-2 no-pass; 2-of-2 with-pass; 2-of-3 no-pass;
  2-of-3 of 2-of-3 with-pass) + JSON output schemas + 25-row refusals
  table mirroring SPEC §2.5 + 6-row advisories table mirroring SPEC
  §2.6 + Trezor interop H3 with cross-impl `shamir-mnemonic` 0.3.0
  smoke recipe (validated end-to-end at chapter-write 2026-05-14
  on Linux x86_64).
- **Manual index markers**: 6 `\index{}` markers + 6 matching
  `69-index-table.md` rows (`SLIP-39`, `SLIP-39 share`, `group
  threshold`, `member threshold`, `K-of-N`, `Trezor SLIP-0039
  interop`). Sets new convention for 40-cli-reference chapters which
  previously carried 0 markers each — flat marker form (no LaTeX `!`
  sub-entries; `lint.sh:124-125` source-side normalizer doesn't
  strip `!`).
- **`docs/manual/tests/cli-subcommands.list` adds** `mnemonic slip39
  split` + `mnemonic slip39 combine` rows.
- **cspell additions**: `onev` (from SPEC OOS row name
  `OOS-slip39-import-trezor-onev-format`) + `trezorctl`.

### Changed

- `Command` enum in `src/main.rs` gains a `Slip39` variant + dispatch.
- `cmd/gui_schema.rs::build_schema` gains recursive nested-subcommand
  flattening — emits hyphenated leaf names (`seed-xor-split`,
  `seed-xor-combine`, `slip39-split`, `slip39-combine`) for any
  parent command containing `#[command(subcommand)]`. Repairs the
  pre-existing v0.12.0 `seed-xor` empty-flags rendering as a
  side-effect. Schema `version: 1` preserved (mirror contract is
  forward-compatible).
- Manual chapter intro bumps from 8 to 9 subcommands; cross-link list
  adds `[`slip39`](#mnemonic-slip39)`; mirror-version line v0.12.0 →
  v0.13.0.
- `SPEC_slip39_v0_13_0.md` accumulated 9 SPEC patches across the
  cycle: 8 at P2.2 GREEN (`19f00a5` — §2.1 per-share-pin O(N)
  clarification, §2.5 row 17 wording reconciliation, §2.5 row 24 add
  per Q3, §2.6 row 5 reconciliation per plan §3.3 space form, §2.6
  row 6 add for TEST_RNG advisory, §4 G4 env-var language, §4 G6
  count 23→28 update, NEW §6 test-only-env-vars subsection) + 1 at
  P3 R1 fold (`b90c436` — §2.5 row 25 add for the T=1+N>1
  toolkit-policy refusal class per python `split_ems` rule;
  paired-SPEC-patch mandate per P3 R0 I2). Plus 2 mini-folds at
  P2.2 R1 LOCK (`d40eb0c` — N-1 row 7 stem cleanup + N-2 G5 count
  23→24).

### Deps

- `hmac = "0.12"` added (PBKDF2-HMAC-SHA-256 for the SLIP-39
  encryption layer's PBKDF2 key derivation).
- `pbkdf2 = "0.12"` (default-features = false, features = ["hmac"])
  added.
- `sha2 = "0.10"` already present (used by hmac/pbkdf2 chain).

### Resolved FOLLOWUPS

- `slip39-shamir-secret-sharing` → resolved at this tag (the
  feature itself).
- `slip39-cli-extendable-flag` → still open as `v0.14-feature` tier
  per design/FOLLOWUPS.md:1050; not closed by this tag.

### Reviewer rounds (cycle aggregate)

- Library cycle (P0/P1a/P1b/P1c-A through P1c-E.3): 11 reviewer
  reports across 8 sub-phases.
- CLI cycle (P2.1 + P2.2 + P2.3): 5 reports including 1 R0 plan
  review + 4 LOCKs.
- Manual cycle (P3): 3 reports — R0 architect plan-review (1C/3I/5N/3n;
  caught lint.sh `!` foot-gun + Trezor One mention contradicting SPEC
  + paired-SPEC-patch mandate + 0.3.0 disclosure recommendation),
  R1 LOCK (3C/1I ITERATE; caught toolkit-refuses-1,1 collision with
  examples + Trezor recipe sed-index off-by-blank-separator +
  combine-default-mode prose error), R2 LOCK (clean).
- 18 reviewer reports total persisted in `design/agent-reports/`.

### Discipline observations

- **`feedback-r0-must-read-source-off-by-n` pattern recurred at every
  P3 review checkpoint**: R0 caught the `lint.sh` `!` foot-gun by
  source-reading the normalizer pipeline; R1 caught the
  toolkit-refuses-1,1 contradiction by source-running the binary
  against chapter examples; R2 verified end-to-end via the corrected
  Trezor recipe. Architect lens must extend to "run the prose's own
  commands", not just "verify the prose's claims against
  documentation". Memory captures this as a forward-looking note.
- **Paired-SPEC-patch mandate triggered exactly once** (P3 R1 fold
  added SPEC §2.5 row 25). The R0 I2 fold introduced this as a
  forward-looking constraint; R1 was the first phase to actually
  trigger it. Mirrors P2.2 GREEN's 8-SPEC-patch precedent at
  `d40eb0c`.

## mnemonic-toolkit [0.12.0] — 2026-05-14

New feature: `mnemonic seed-xor` subcommand (`split` + `combine`
sub-subcommands). Coldcard-compatible BIP-39 ↔ BIP-39 all-or-nothing
XOR-based seed splitter. Given a single BIP-39 entropy (12/15/18/21/24
words), split into N BIP-39 phrases such that bytewise XOR of all N
entropies reconstitutes the master. Per-share BIP-39 checksum is
recomputed so each share is itself a parseable, structurally-valid
BIP-39 phrase. NOT a threshold scheme — ALL N shares required to
reconstruct (for K-of-N use SLIP-39, planned for v0.13.0).

Coldcard hardware interop at 12/18/24-word sizes (per `xor_seed.py`
entropy lengths 16/24/32 bytes); 15/21-word are toolkit-only extensions.
No MAC; share substitution is mathematically undetectable — verify
recovered wallet's derived address before trusting.

Toolkit-only minor bump. No cross-repo work; ms-cli unchanged. Closes
FOLLOWUP `seed-xor-coldcard-compat`. First of two cycles in the
share-splitting pair (paired with v0.13.0 SLIP-39, planned at
`~/.claude/plans/radiant-seeking-teacup.md`).

### Added

- `mnemonic-toolkit` library: new module `mnemonic_toolkit::seed_xor`
  exposing `seed_xor_split` (random via supplied CSPRNG),
  `seed_xor_split_deterministic` (Coldcard SHA256d-deterministic
  generation), and `seed_xor_combine`. Library-local `SeedXorError`
  (3 variants: `BadEntropyLength` / `TooFewShares` /
  `MismatchedShareLengths`) per the v0.11.0 final-word precedent.
  Returns `Vec<Zeroizing<Vec<u8>>>` for shares + `Zeroizing<Vec<u8>>`
  for recovered master.
- `mnemonic seed-xor split` subcommand with flags `--from
  <phrase=<value-or-->> --shares N [--language LANG]
  [--deterministic-from-master] [--json-out PATH]`. Emits N BIP-39
  phrases to stdout (one per line) with per-share checksum recompute
  via `Mnemonic::from_entropy_in`.
- `mnemonic seed-xor combine` subcommand with flags `--share
  <phrase=<value-or-->> ... --shares N [--language LANG] [--json-out
  PATH]`. Hard refusal on `--share` count vs `--shares` mismatch;
  hard refusal on multi-stdin contention; per-share argv-leakage
  advisory.
- New advisory class: **multi-secret-on-stdout** for K-of-N share
  emit (first toolkit use; SLIP-39 v0.13.0 will parameterize it).
  Wording calls out share-substitution undetectability + N-physical-
  location distribution discipline.
- `#[cfg(unix)]` permission-mode advisory on `--json-out` when the
  output file is world-readable (mode & 0o077 != 0).
- 15/21-word + `--deterministic-from-master` toolkit-only advisory
  (Coldcard hardware cannot round-trip those sizes).
- JSON envelope v1 with `operation: "split"` / `"combine"` discriminator.
  SHA-pinned anchors:
  - abandon×12 N=2 deterministic: `d368c70aabb6d3bab7d75b79f8a61a8340db6ac94c57250db6354fe235861af3`
  - Trezor legal×12 N=3 deterministic: `85d53f7e83db167b1223b8b23bbe2baca060e7aefad50f6034b5b65750883871`
- 44 CLI tests across 5 files + 17 library tests (2000 round-trip
  property-test pairs + Coldcard byte-pin anchor + length-validation
  refusals + Zeroize-discipline type-binding check + RNG determinism).
- Manual chapter section `## mnemonic seed-xor` in
  `docs/manual/src/40-cli-reference/41-mnemonic.md` (Synopsis, dual
  flag tables for split + combine, Worked example, JSON output schemas,
  Refusals table, Advisories table).
- `docs/manual/tests/cli-subcommands.list` adds `mnemonic seed-xor split`
  + `mnemonic seed-xor combine` rows.

### Changed

- `Command` enum in `src/main.rs` gains a `SeedXor` variant + dispatch.
- Manual chapter intro bumps from 7 to 8 subcommands.
- Glossary `mnemonic` entry: Seven → Eight subcommands.
- `lint_argv_secret_flags.rs`: 21 → 23 rows (+2 for `seed-xor split
  --from phrase=` and `seed-xor combine --share phrase=`).
- `lint_zeroize_discipline.rs`: +1 row for `seed_xor.rs` Zeroizing wrap
  evidence.
- `cli_gui_schema.rs`: 6 → 7 user-facing subcommands assertion.

### Deps

- `rand_core = "0.6"` (features `["std", "getrandom"]`) added as crate
  dep (RustCrypto, MIT/Apache-2.0).
- `rand_chacha = "0.3"` added as dev-dep (deterministic RNG for
  property tests).

### Resolved FOLLOWUPS

- `seed-xor-coldcard-compat` → resolved at this tag.

## mnemonic-toolkit [0.11.0] — 2026-05-14

New feature: `mnemonic final-word` subcommand. Given an N-1-word BIP-39
partial phrase, emits the lexicographically sorted set of wordlist
entries that, when appended as the Nth word, yield a phrase with a
valid BIP-39 checksum. Output set size is a function of N alone:
128 (N=12), 64 (N=15), 32 (N=18), 16 (N=21), 8 (N=24). Use cases
include paper-backup recovery (smudged last word), manual seed
generation (compute the checksum-fixing word for a hand-rolled
partial), and phrase-typo verification.

Toolkit-only minor bump. No cross-repo work; ms-cli `v0.3.0` ships
unchanged. Closes FOLLOWUP `bip39-final-word-completer`.

### Added

- `mnemonic-toolkit` library: new module `mnemonic_toolkit::final_word`
  exposing `final_word_candidates(partial_phrase: &str, language:
  FinalWordLanguage) -> Result<Vec<&'static str>, FinalWordError>`. The
  library carries its own self-contained `FinalWordLanguage` (10 BIP-39
  wordlists) and `FinalWordError` (`BadWordCount` / `UnknownWord`)
  types so the lib surface does not pull in the binary-private
  `ToolkitError`. Algorithm: naïve enumeration over the 2048-entry
  wordlist with `bip39::Mnemonic::parse_in` as the correctness oracle
  (~2048 SHA-256 ops; milliseconds per query). Tracks FOLLOWUP
  `library-error-and-language-surface-promotion` for the future
  crate-shape cleanup that would unify these with `CliLanguage` +
  `ToolkitError`.
- `mnemonic final-word` subcommand:
  - `--from <phrase=<value-or-->>` (required): inline `phrase=<N-1 words>`
    or `phrase=-` (stdin). Inline form emits the Cycle A argv-leakage
    advisory via `secret_in_argv_warning`.
  - `--language <LANGUAGE>` (default `english`): 10 BIP-39 wordlists
    (`english`, `simplifiedchinese`, `traditionalchinese`, `czech`,
    `french`, `italian`, `japanese`, `korean`, `portuguese`, `spanish`).
  - `--json-out <PATH>` (optional): side-effect; writes a versioned
    JSON envelope (schema_version `"1"`, fields `language`,
    `partial_word_count`, `target_word_count`, `candidate_count`,
    `candidates`) without replacing stdout. SHA-pinned in
    `tests/cli_final_word_json.rs` over two anchor vectors (abandon×11,
    beef×11).
- Cycle A discipline on the new CLI handler: `Zeroizing<String>` over
  the parsed partial; `secret_in_argv_warning` for inline secret;
  lint row `final-word --from phrase=` added to
  `tests/lint_argv_secret_flags.rs` (20→21 rows).
- Cycle B discipline on the new CLI handler: mlock Site 1
  `pin_pages_for(partial.as_bytes())` after wrap; lint row added to
  `tests/lint_zeroize_discipline.rs`.
- New advisory class **secret-on-stdout-TTY**: when the candidate set
  is non-empty AND `std::io::stdout().is_terminal()`, emit
  `warning: candidate list is secret material ...` to stderr. First
  use of `std::io::IsTerminal` in the toolkit (stable since Rust 1.70;
  no `atty` dep — RUSTSEC-2021-0145).
- `#[cfg(unix)]` permission-mode advisory on `--json-out` when the
  resulting file is world-readable (`mode & 0o077 != 0`).
- Manual chapter section `## mnemonic final-word` in
  `docs/manual/src/40-cli-reference/41-mnemonic.md` (Synopsis, Flags,
  Worked example, JSON output, Refusals, Advisories).
  `docs/manual/tests/cli-subcommands.list` updated; cspell dictionary
  extended with `cmdline` + `simplifiedchinese` + `traditionalchinese`.
- CLI `gui-schema` automatically picks up `final-word` via
  `clap::CommandFactory` — no code change needed; test expectation
  bumped from 5 to 6 user-facing subcommands.

### Changed

- `Command` enum in `src/main.rs` gains a `FinalWord` variant +
  dispatch arm.
- Glossary entry for `mnemonic` (`docs/manual/src/60-appendices/61-glossary.md`)
  updated from "Five subcommands" to "Seven subcommands" (also adds
  `gui-schema` to the previous pre-existing drift — was actually six
  before this cycle).

### Fixed (SPEC narrative)

- `design/SPEC_final_word_v0_11_0.md` §2.4 / §2.5: refusal exit code
  corrected from `64` to `1` (`ToolkitError::BadInput::exit_code()`
  routes per `error.rs:244`). Tests were already tolerant; this is a
  documentation correction only.

### Resolved FOLLOWUPS

- `bip39-final-word-completer` → resolved at this tag.

## mnemonic-toolkit [0.10.1] — 2026-05-13

Cycle B Path B-lite carve-out completion: `ResolvedSlot.entropy` and
`DerivedAccount.entropy` field-type migration to `Zeroizing<Vec<u8>>`.
Closes FOLLOWUP `resolved-slot-derived-account-zeroizing-field` (the
Path B-lite carve-out tracker, originally deferred from Cycle B Phase 3a)
and FOLLOWUP `pub-struct-drop-semver-risk-monitor` (DerivedAccount-
specific watch — the deletion of `impl Drop for DerivedAccount` removes
the move-out destructure E0509 risk this monitor was watching for).

Toolkit-only patch. No cross-repo work; ms-cli `v0.3.0` (mnemonic-secret
`2e7c275`) ships unchanged.

### Changed

- `ResolvedSlot.entropy: Option<Vec<u8>>` → `Option<zeroize::Zeroizing<Vec<u8>>>`
  (`crates/mnemonic-toolkit/src/synthesize.rs`). Drop-time scrub is now
  structurally guaranteed by the type; the bytes-may-persist-on-heap-
  after-dealloc gap from the Cycle A baseline (which had NO Drop scrub
  on this field) is closed.
- `DerivedAccount.entropy: Vec<u8>` → `zeroize::Zeroizing<Vec<u8>>`
  (`crates/mnemonic-toolkit/src/derive.rs`). Same structural-Drop
  semantics; replaces the v0.9.0 Cycle A `impl Drop for DerivedAccount`.
- `DerivedAccount::into_parts()` body: `mem::take(&mut self.entropy)` →
  `mem::take(&mut *self.entropy)`. Outer signature returning bare
  `Vec<u8>` preserved per the existing caller-wrap contract.
- 12 ctor sites wrap the entropy field at the field-write boundary
  (6 direct `ResolvedSlot {` + 6 via `pub type CosignerKeyInfo = ResolvedSlot;`
  alias trap): `cmd/bundle.rs:{364,435,469,513,1046,1102}`,
  `cmd/verify_bundle.rs:489`, `parse_descriptor.rs:{1179,1743,1758}`,
  `synthesize.rs:{1061,1217}`. The 6 alias-routed sites are the same
  off-by-N pattern that `feedback_r0_must_read_source_off_by_n` warns
  about; R0 round 1 caught this in the plan-write phase.
- 1 ctor at `derive_slot.rs:84` wraps `Zeroizing::new(entropy_bytes)`.

### Read-site adjustments (compile-driven, 7 sites)

- `parse_descriptor.rs:814-820` (`DescriptorBinding::entropy_at_0`):
  `Option::as_deref` is single-step Deref (returns `Option<&Vec<u8>>`);
  chain through `.as_ref().map(|z| z.as_slice())` to reach `Option<&[u8]>`.
- `synthesize.rs:715` (`synthesize_unified` ms1 build): `e.clone()` over
  `&Zeroizing<Vec<u8>>` returns `Zeroizing<Vec<u8>>` (Zeroizing's own
  Clone); use `(**e).clone()` to reach the inner Vec for `Payload::Entr`.
- `cmd/verify_bundle.rs:500-502`: drop the `Zeroizing::new(e.clone())`
  map (would double-wrap); `slot.entropy.clone()` matches `entropy_at_0`'s
  declared type natively.
- `derive.rs:108` (test): `assert_eq!(acc.entropy, vec![...])` →
  `assert_eq!(*acc.entropy, vec![...])` (Zeroizing has no `PartialEq<T>`).
- `parse_descriptor.rs:956`: `c0.entropy = Some((*entropy).clone())` →
  `c0.entropy = Some(Zeroizing::new((*entropy).clone()))` (re-wrap).

### Removed

- `impl Drop for DerivedAccount` (`crates/mnemonic-toolkit/src/derive.rs`).
  Zeroizing's Drop now carries the scrub responsibility. Re-enables
  E0509-free move-out destructuring of `DerivedAccount`; `into_parts()`
  remains the canonical consuming-move path.
- Deferred-FOLLOWUP comment block at
  `tests/lint_zeroize_discipline.rs:109-113` (referenced the obsolete
  `resolved-slot-entropy-zeroizing-field` FOLLOWUP).

### Audit (lint_zeroize_discipline.rs)

- DerivedAccount row relabeled from "impl Drop scrubs entropy on drop"
  → "DerivedAccount entropy field is Zeroizing<Vec<u8>>", anchor
  `pub entropy: zeroize::Zeroizing<Vec<u8>>`.
- New row "ResolvedSlot entropy field is Option<Zeroizing<Vec<u8>>>",
  anchor `pub entropy: Option<zeroize::Zeroizing<Vec<u8>>>`.
- Trailing row-count comment updated `~27 rows` → `~28 rows`.

### What didn't change

- mlock sibling-field discipline (Cycle B Phase 3a) preserved.
  `_entropy_pin` declaration order unchanged on both structs; RFC 1857
  drop order still: entropy field first (now Zeroizing-drives-scrub),
  `_entropy_pin` munlock second.
- Public-API signature on `DerivedAccount::into_parts()` (still returns
  bare `Vec<u8>`).
- All CLI flag surfaces; exit codes; JSON schemas.
- v0.1 + v0.2 fixture-corpus SHA pins.
- ms-codec / mk-codec / md-codec git-dep tags.

### Cycle review history

- R0 (plan review): 3 rounds. Round 1 REWORK (7 Critical + 4 Important):
  off-by-N ctor-count + 7 read-site compile breaks the FOLLOWUP missed.
  Round 2 LOCK with 9 Important narrative-accuracy folds + 2 nits.
  Round 3 LOCK clean. Plan at `~/.claude/plans/v0_10_1-zeroizing-field-migration.md`.
- R1 (impl review): see `design/agent-reports/v0_10_1-zeroizing-field-migration-r1.md`.

### Tests

- 620 tests green (`cargo test -p mnemonic-toolkit`).
- 1 new lint row + 1 relabeled lint row in `tests/lint_zeroize_discipline.rs`.
- `cargo clippy --all-targets -- -D warnings` clean.
- `cargo +nightly miri test -p mnemonic-toolkit mlock::` green (no regression).

## mnemonic-toolkit [0.10.0] — 2026-05-13

v0.9.0 cross-repo Cycle B (`mlock(2)` page-pinning infrastructure),
Phase E release rollup. Companion lockstep release: `ms-cli-v0.3.0`
(mnemonic-secret). Cycle SPEC at
`design/SPEC_secret_memory_hygiene_v0_9_B.md`; Path B-lite RESCOPE
proposal at `~/.claude/plans/2026-05-13-cycle-b-phase-3a-rescope-proposal.md`;
cross-repo audit matrix at
`design/agent-reports/v0_9_B-secret-memory-hygiene-matrix.md`.

POSIX-only (Linux + macOS); Windows `VirtualLock` deferred to a future
cycle (SPEC §3 `OOS-windows-virtuallock`). New public-API surface: the
`mnemonic_toolkit::mlock` module (lib + bin hybrid crate shape, SPEC §4
P2). All errno classes soft-fail in release builds; debug builds trip
`debug_assert!` on the unreachable `EINVAL` path. mlock failures (if
any) emit a 2-line stderr summary at end-of-process per SPEC §6 G2.5.

### Added (Phase 1 — bip85 heap-promote precursor)

- `bip85::derive_entropy(index: u32) -> [u8; 64]` widened to
  `-> Result<Zeroizing<Vec<u8>>, ToolkitError>`. 7 `format_*` callees
  updated to consume the heap-promoted return (the original Phase 1
  framing said "6 callees" — `format_dice_rolls` was the off-by-one
  caught at Phase 1 R0).

### Added (Phase 2 — mlock module + first Rust CI workflow)

- New crate-shape: `crates/mnemonic-toolkit/src/lib.rs` exposes
  `pub mod mlock;` (hybrid lib + bin per SPEC §4 P2 — Option C smallest
  cascade). `[[bin]]` stays at `src/main.rs`; other modules remain
  binary-private.
- New module `crates/mnemonic-toolkit/src/mlock.rs` (~533 LOC). Surface
  per SPEC §4 P2 + §6 G6 manifest:
  - `pin_pages_for(buf: &[u8]) -> PinnedPageRange` slice-fn primitive
    (Fix-B-only after Phase 2 R0 C-1 indirection-trap finding retired
    the `MlockedZeroizing<T>` wrapper).
  - `PinnedPageRange { start, page_count }` with munlock-on-Drop.
    Page-rounding formula pinned in SPEC §2 row 1; zero-length is a
    no-op (no syscall).
  - `MlockState` process-static singleton (atomic counters +
    OnceLock-tracked first errno).
  - `report_at_exit()` end-of-process 2-line stderr emitter (called
    from `main()`).
  - Private `page_size()` cached in `OnceLock<usize>`; sourced via
    `libc::sysconf(libc::_SC_PAGESIZE)`.
  - `#[cfg(test)]` fault-injection harness:
    `MNEMONIC_TEST_MLOCK_FAIL_MODE={eperm,enomem,einval,off}` parsed
    into a OnceLock-cached `FailMode` for per-subprocess mode variation.
- New `libc = "0.2"` dep.
- `.github/workflows/rust.yml` (NEW): first Rust CI workflow for the
  toolkit (`manual.yml` + `quickstart.yml` were docs-build only). Jobs:
  `test` (Ubuntu + macOS matrix with `ulimit -l 65536` on Linux; +
  3 fault-injection steps for G2.1/G2.3-debug/G2.4), `miri` (Ubuntu
  nightly; cfg(miri) shim verifies the 2 unsafe blocks in
  `pin_pages_for` + `PinnedPageRange::drop` per SPEC §6 G4.b), `clippy`
  (`--all-targets -- -D warnings`).
- New lint `tests/lint_safety_first_party_mlock.rs`: enforces a SAFETY:
  comment within ±5 lines of every `unsafe {` opener in `src/mlock.rs`
  (peer of Cycle A's `lint_safety_third_party_blocked.rs`).

### Added (Phase 3a Path B-lite — apply sites 1+2+3+4 + main wire)

Per Phase 3a R0 v3-fold RESCOPE (proposal
`~/.claude/plans/2026-05-13-cycle-b-phase-3a-rescope-proposal.md`;
reviewer reports `v0_9_B-phase-3a-rescope-r0{,-v3,-v3-fold}.md` LOCK
0/0). Path B-lite carves the Cycle-A→Zeroizing field-type migration
(`ResolvedSlot.entropy: Option<Vec<u8>>` and
`DerivedAccount.entropy: Vec<u8>`) out to v0.10.1 patch via FOLLOWUP
`resolved-slot-derived-account-zeroizing-field` (supersedes the Cycle A
FOLLOWUP `resolved-slot-entropy-zeroizing-field`). All struct-sibling
pins on ResolvedSlot + DerivedAccount are preserved; the Cycle A
baseline ships UNCHANGED.

- **Site 1** (per-handler clap-binding pins):
  - `cmd/bundle.rs` + `cmd/verify_bundle.rs`: `pin_pages_for(&synthetic_args)`
    re-binding immediately after `apply_stdin_substitutions()` returns.
  - `cmd/convert.rs`: pins `effective_passphrase` + `effective_bip38_passphrase` +
    `primary_value` after they're bound (no `apply_stdin_substitutions`
    in convert — corrected from Path B-lite §3.1; SPEC §2 row 5).
  - `cmd/derive_child.rs`: pins `from_value: Zeroizing<String>` +
    `stdin_passphrase: Option<Zeroizing<String>>` post-binding.
- **Site 2**: `ResolvedSlot` adds sibling field
  `_entropy_pin: Option<Rc<PinnedPageRange>>` declared AFTER `entropy`.
  `Rc` (not `Arc`) preserves the `derive(Clone)` semantics; `Arc` was
  retracted post-clippy `arc_with_non_send_sync` flagged
  `PinnedPageRange` as `!Send + !Sync` (commit `ddb371c`). 12 ctor
  sites populated (`pub type CosignerKeyInfo = ResolvedSlot;` alias
  adds 6 ctor sites — the recurring off-by-N pattern caught at
  Phase 3a R0 v3-fold per `feedback_r0_must_read_source_off_by_n`):
  `synthesize.rs:{1059,1213}`, `parse_descriptor.rs:{1176,1741,1755}`,
  `cmd/bundle.rs:{371,441,475,518,1049,1099}`, `cmd/verify_bundle.rs:496`.
- **Site 3**: `DerivedAccount` adds sibling field
  `_entropy_pin: PinnedPageRange` declared AFTER `entropy` (plain, not
  Rc — DerivedAccount is not Clone and is consumed via `into_parts`).
  1 ctor site populated: `derive_slot.rs:89`. Cycle A's `impl Drop for
  DerivedAccount` PRESERVED (zeroize-while-still-pinned ordering).
- **Site 4**: bip85's 7 `format_*` functions add
  `let _entropy_pin = mnemonic_toolkit::mlock::pin_pages_for(&entropy[..]);`
  immediately after the `derive_entropy(...)?` binding. Local-binding
  drop order (Rust Reference §"destructors"): `_entropy_pin` munlocks
  first then `entropy: Zeroizing<Vec<u8>>` zeroizes.
- `main.rs:101`: `mnemonic_toolkit::mlock::report_at_exit();` wired
  between the `match result` close and the `ExitCode` return.
- **CI delta**: `.github/workflows/rust.yml` adds the
  `test-release-mlock-einval` Linux-only release-build subprocess job
  per SPEC §6 G2.3 release branch (G2.3-release coverage —
  `debug_assert!` is compiled out in release, so EINVAL must soft-fail
  via `record_failure` not panic).

### Added (Phase 3b — cross-repo ms-cli participation)

- mnemonic-secret `ms-cli-v0.3.0` ships the inline `mlock.rs` copy
  (538 LOC; diff = `//!` mod-doc only after SPEC §6 G6 normalization)
  + Site 5 pin (`parse.rs:65`) + main wire. Reviewer cleared at
  `design/agent-reports/v0_9_B-phase-3b-r1.md`.

### Added (PE — release rollup)

- `design/agent-reports/v0_9_B-secret-memory-hygiene-matrix.md`:
  cross-repo audit matrix (toolkit-side canonical hub per Cycle A
  precedent). §0 cross-repo coverage; §0.5 6 residual classes; §1 SPEC
  §2 site coverage per-site status; §2 Cycle A → Cycle B carry-overs
  closed-out (5 candidates); §3 SPEC §3 FOLLOWUPS forward-visibility;
  §4 Path B-lite v0.10.1 carve-out; §5 SPEC §6 cycle-close gates.
- `tests/mlock_g6_invariant.rs` (NEW): SPEC §6 G6 cross-repo inline-copy
  invariant test. Normalizes toolkit's `mlock.rs` + ms-cli's `mlock.rs`
  (strip `//`, `///`, `//!` comment-only lines at start-of-trimmed-line;
  preserve `use` statements + `#[cfg]` attributes), asserts byte-equal
  + name-export parity against a 14-item static MANIFEST. Sibling-repo
  path discovery via `SIBLING_REPO_PATH` env var with adjacent-dir
  relative fallback.
- `.github/workflows/rust.yml` adds `g6-invariant` job: checks out
  mnemonic-secret at master and runs the G6 test with
  `SIBLING_REPO_PATH=$GITHUB_WORKSPACE/mnemonic-secret`.
- SPEC §2 row 5 + §4 P3b: `parse.rs:45` → `parse.rs:65` line-number
  drift fix (post Cycle A's `Zeroizing<String>` shift).

### Cycle review history

- Phase 0: R1 Opus 2C/3I folded; R2 Opus 0C/0I.
- Phase 1: R0 design lock; R1 post-impl CLEAR.
- Phase 2: R0 Fix B trigger (C-1 indirection-trap → MlockedZeroizing<T>
  retired; slice-fn-only design locked); R0 Fix-B verify; R1 0C/0I;
  R2 0C/0I confirmed.
- Phase 3a (rescope): R0 + R0-v3 + R0-v3-fold (3 rounds; Path B-lite
  LOCK 0/0).
- Phase 3a (impl): R1 CLEAR.
- Phase 3b (cross-repo impl): R1 CLEAR.

### Tests

- 11 new mlock-module unit + subprocess tests in `mod tests` inside
  `src/mlock.rs` (page-rounding + MlockState aggregation + g2_* fault-
  injection arms matching SPEC §6 G2.1 / G2.3 (debug + release) / G2.4).
- 4 new `tests/mlock_unit.rs` integration tests (SPEC §6 G1.1-G1.4
  pin-residency + page-count checks).
- 2 new G6 invariant tests in `tests/mlock_g6_invariant.rs`.
- 1 new lint `tests/lint_safety_first_party_mlock.rs`.
- `cargo test --workspace`: green at PE close.
- `cargo clippy --all-targets -- -D warnings`: clean (Arc → Rc fix at
  `ddb371c` closed the only Cycle-B clippy regression).
- `cargo +nightly miri test -p mnemonic-toolkit mlock::`: green via
  cfg(miri) syscall shims.

### Known residue (carry-forward post-Cycle-B)

Six residual classes per the audit matrix §0.5. The notable ones:
- Live-RAM disclosure via `ptrace` / `/proc/PID/mem` / kernel debugger
  (SPEC §1 "Threat model NOT addressed"; mlock does not defend).
- Co-resident page-residue from non-secret data on pinned pages
  (SPEC §3 `OOS-page-residue-elimination`; Cycle C `dedicated-secret-arena`).
- Windows `VirtualLock` (SPEC §3 `OOS-windows-virtuallock`).
- `ResolvedSlot.entropy` + `DerivedAccount.entropy` field-type migration
  to `Zeroizing<Vec<u8>>` — Path B-lite carve-out to v0.10.1 patch via
  FOLLOWUP `resolved-slot-derived-account-zeroizing-field`.

### What didn't change

- ms-codec / mk-codec / md-codec git-dep tags (no sibling-codec work in
  Cycle B; toolkit continues to pin `ms-codec-v0.1.3`,
  `mk-codec-v0.2.1`, `md-codec-v0.16.1`).
- All CLI flag surfaces preserved (no flag additions / removals; exit
  codes unchanged; JSON schemas unchanged).
- v0.1 + v0.2 fixture-corpus SHA pins continue to hold (SPEC §6 G7 — no
  wire-format regression).

## mnemonic-toolkit [0.9.2] — 2026-05-13

v0.9.0 cross-repo Cycle A (OWNED-buffer secret-memory hygiene), Phase E
release rollup. Cycle SPEC at
`design/SPEC_secret_memory_hygiene_v0_9_0.md`; cycle plan at
`/home/bcg/.claude/plans/v0_9_0-secret-memory-hygiene.md`; cross-repo
hygiene-matrix at `design/agent-reports/v0_9_0-secret-memory-hygiene-matrix.md`.

### Added (Phase 1 — argv-leakage closure)

9 new toolkit flag-rows now have a stdin route + advisory:

- `bundle --passphrase-stdin`, `verify-bundle --passphrase-stdin`,
  `derive-child --passphrase-stdin`, `convert --bip38-passphrase-stdin`
  paired-flag closures (4 new `--*-stdin` flags).
- `bundle --slot @N.<phrase|entropy|wif|xprv>=-` and
  `verify-bundle --slot @N.<secret>=-` `=-` value carve-out (5 slot rows
  via 1 parser extension at `slot_input.rs`).
- `secret_advisory.rs` module emits a `warning: secret material on argv
  (...) — pipe via ... to avoid /proc/$PID/cmdline exposure` stderr line
  per-(flag, slot-index) occurrence whenever an inline secret is
  detected on argv.
- Multi-stdin refused at clap parse-time across all three (`bundle`,
  `verify-bundle`, `derive-child`); per-command stdin source is
  exclusive.

### Added (Phase 2 — Zeroizing wrappers + SAFETY anchors)

- `zeroize = "1.8"` dep.
- ~30 toolkit OWNED-buffer secret allocations now wrapped in `Zeroizing<T>`
  (enumerated by `tests/lint_zeroize_discipline.rs` at 38 row-cells
  across `cmd/bundle.rs`, `cmd/verify_bundle.rs`, `cmd/derive_child.rs`,
  `cmd/convert.rs`, `derive.rs`, `derive_slot.rs`, `bip85.rs`,
  `synthesize.rs`, `parse_descriptor.rs`, `electrum.rs`).
- `DerivedAccount::into_parts(mut self)` consuming method + `impl Drop
  for DerivedAccount` (Phase 2 prereq; E0509-safe consumer migration of
  3 internal move-out sites).
- `derive_master_seed(&Mnemonic, &str) -> Zeroizing<[u8; 64]>` helper
  consolidates 7 BIP-39→BIP-32 production spines into one site.
- `bip85::derive_entropy` return-type widened to
  `Result<Zeroizing<[u8; 64]>, ToolkitError>`.
- 32 SAFETY-anchor doc-comments at upstream-blocked third-party sites
  (Mnemonic / Xpriv / SecretKey) citing the corresponding FOLLOWUP slug.
- New lint `tests/lint_safety_third_party_blocked.rs` scans source for
  the third-party-blocked call patterns and enforces a SAFETY: anchor
  within 3 preceding lines.
- New lint `tests/lint_argv_secret_flags.rs` enumerates the 9 Phase 1
  flag-row closures with per-row evidence.

### Added (Phase 3 — cross-repo audit matrix)

- `design/agent-reports/v0_9_0-secret-memory-hygiene-matrix.md` —
  canonical cross-repo audit matrix (toolkit hub). §0 cross-repo
  coverage; §0.5 "what this cycle does NOT close" (6 residual classes);
  §1 OWNED-row status (CLEAR / PARTIAL-3RD-PARTY / OUT-OF-SCOPE) for
  every survey-§1 row; §2 status for every survey-§5 flag-row; §3
  14 SPEC-§3-OOS + 4 cycle-surfaced FOLLOWUPS forward-visibility;
  §4 Cycle B carry-overs; §5 SPEC §6 cycle-close gates.
- 9 new FOLLOWUPS in `design/FOLLOWUPS.md` (open):
  `argv-overwrite-after-parse`, `clap-argv-pre-parse-residue`,
  `allocator-pool-residue`, `pub-struct-drop-semver-risk-monitor`,
  `dedicated-secret-arena`, `sha3-shake256-zeroize-upstream`,
  `bip38-crate-internal-zeroize-upstream`, `secret-memory-hygiene-cycle-b`,
  `md-mk-private-key-surface-watch`.

### Changed

- ms-codec git dep tag: `ms-codec-v0.1.0` → `ms-codec-v0.1.3` (picks up
  cross-repo Phase 2 ms-codec Zeroizing discipline).

### Known third-party residue

- `bitcoin::bip32::Xpriv` is `Copy + !Drop` — FOLLOWUP
  `rust-bitcoin-xpriv-zeroize-upstream` (external).
- `bip39::Mnemonic` interior buffer not zeroize-aware — FOLLOWUP
  `rust-bip39-mnemonic-zeroize-upstream` (external).
- `secp256k1::SecretKey` no Drop+Zeroize — FOLLOWUP
  `rust-secp256k1-secretkey-zeroize-upstream` (external).

### Cycle review history

- Phase 0: SPEC + plan + survey — R1-R5 architect-review (3 Sonnet + 2 Opus rounds) cleared 0C/0I after R3 SPLIT-CYCLE pushback + user decisions on impl-Drop approach + drop md/mk symmetry-stubs.
- Phase 1: R1 Opus 0C/1I/2N — all folded; R2 Sonnet 0C/0I.
- Phase 2: R1 Opus 0C/4I/5N — all 4 I folded; R2 Sonnet 0C/0I cross-repo.
- Phase 3: R1 Opus 1C/1I/2N (FOLLOWUPS-cite C-1 + slug-rename I-1 + 2 editorial N) — all folded; R2 Sonnet 0C/0I.

### Tests

- 3 new lint tests green at every phase close.
- `cargo test --workspace`: 43/43 green at Phase 2 close.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.

### What didn't change

- All CLI flag surfaces preserved (additions only: 4 new `--*-stdin`
  flags; no flag removals; no exit code changes).
- v0.9.1 → v0.9.2 patch-tag compatibility maintained for external
  library users that access `DerivedAccount.entropy` via borrow
  (`&derived.entropy`); move-out destructure is the documented break
  per `pub-struct-drop-semver-risk-monitor` FOLLOWUP.

## mnemonic-toolkit [0.9.1] — 2026-05-13

v0.8.0 cross-repo BIP-vector adoption cycle, Phases 0 / 3 / 4. Cycle
SPEC at `design/SPEC_test_vector_audit_v0_8_0.md`; cycle plan at
`/home/bcg/.claude/plans/v0_8_0-bip-vector-adoption.md`. Phase 0 lands
the cycle artifacts; Phase 3 adds one missing reference cell; Phase 4
lifts the audit matrix from toolkit-only to cross-repo first-class.

### Added (tests-only; no library API change)

- `tests/cli_derive_child.rs::cell_2b_bip39_24_words_reference_vector` —
  BIP-85 vector 85.3 (24-word BIP-39 application, path
  `m/83696968'/39'/0'/24'/0'`). Closes the v0.7.1 §5 carry-over for
  BIP-85; coverage now 8/9 (only 85.9 DICE remains as a refusal cell).
- `design/SPEC_test_vector_audit_v0_8_0.md` — cycle contract.
- `design/agent-reports/v0_8_0-cross-repo-bip-vector-survey.md` — the
  survey that surfaced the cycle's three high-ROI gaps.
- `design/agent-reports/v0_8_0-phase-{0,3,4}-*-r1.md` — three phase
  R1 disposition reports.
- `design/agent-reports/v0_8_0-bip-test-vector-audit-matrix.md` — v0.8.0
  audit matrix with new §0 cross-repo coverage table naming each
  sibling repo's matrix file as first-class.
- `design/FOLLOWUPS.md` — three entries: `bip-vector-adoption-v0_8`
  (cycle companion), `bip340-schnorr-signing-surface-evaluation`
  (SPEC §3 OOS-PER-LAYER), `bip39-japanese-wordlist-support` (SPEC §3
  OOS-PER-PRODUCT).

### Cross-repo cycle context

Sibling-repo cycle work shipped in lockstep:

- `bg002h/descriptor-mnemonic` v0.32.1 (md-codec): +7 BIP-341
  `scriptPubKey` cells + 2 invariants.
- `bg002h/mnemonic-secret` v0.1.2 (ms-codec): +4 net-new BIP-93
  valid + 64 invalid cells + 1 parametric invariant. (§93.4 was
  already pinned at v0.7.1 via `bip93_cross_format.rs`; v0.8.0 adds
  §93.1–.3 + §93.5 in `bip93_inline_vectors.rs` plus an `invalid_corpus_length_is_64`
  guard. v0.7.1 footnote "42 invalid" was an earlier-snapshot
  artifact and is corrected to 64 at v0.8.0.)
- `bg002h/mnemonic-key` (mk-codec): docs-only (no Cargo bump);
  cross-repo audit-matrix symmetry entry.

Net cycle delta vs v0.7.1: **+94 vectors** across the constellation.

### Corrected

- v0.7.1 audit matrix was toolkit-only; sibling-repo coverage was
  footnoted but not first-class. v0.8.0 matrix §0 cross-repo coverage
  table names each sibling's matrix inline. v0.7.1 matrix carries a
  SUPERSEDED forward-pointer.

### What didn't change

- Public CLI surface unchanged. No flag added or removed; no
  subcommand changed. `mnemonic-gui` schema-mirror contract is
  vectors-only and not impacted.
- Library API surface unchanged.

## mnemonic-toolkit [0.9.0] — 2026-05-12

Additive minor release atop v0.8.1. **No breaking changes.** Introduces `mnemonic gui-schema`, a developer-facing introspection subcommand emitting the SPEC §7 machine-readable flag-surface schema as JSON. Companion to the `mnemonic-gui` v0.2 Phase C.2 schema-mirror contract (`bg002h/mnemonic-gui` `FOLLOWUPS.md` mnemonic-gui-schema-mirror).

### Added

- **`mnemonic gui-schema`** subcommand. Walks the clap-derive `Command` tree via `clap::CommandFactory` and serializes a `{ version: 1, cli: "mnemonic", subcommands: [...] }` JSON document to stdout. Each subcommand carries `flags` (with `name`, `required`, `kind`, optional `choices`) and `positionals`. `kind` is one of `text` / `boolean` / `number` / `dropdown` / `path`; `choices` is non-null only when `kind == "dropdown"`. Self-reference suppression: the `gui-schema` subcommand itself is filtered out of its own output. Complex GUI-side variants (NodeValueComposite / TaggedOrIndexed / Range / Timestamp) map to `"text"` upstream per the SPEC §7 lossy-mapping contract; the GUI re-parses client-side.
- **`cli_gui_schema.rs`** integration tests (16 cases). Pins the SPEC §7 contract: `version == 1`, `cli == "mnemonic"`, all 5 user-facing subcommands listed, self-reference suppression, per-flag `required` is bool, `choices` is non-null iff `kind == dropdown`, `kind` value set is exactly `{text, boolean, number, dropdown, path}`. Spot-checks bundle's `--network` dropdown choices, the 10-template enum, the 8-vendor `--format` enum, derive-child's four required flags, and verify-bundle's `--bundle-json` path classification.
- **Manual mirror**: new `## mnemonic gui-schema` section in `docs/manual/src/40-cli-reference/41-mnemonic.md` documents the synopsis, single `--help` flag, and the SPEC §7 JSON output shape. `docs/manual/tests/cli-subcommands.list` adds `mnemonic gui-schema` so the `flag-coverage` lint covers the new subcommand.

### FOLLOWUPS

- Closes companion of `mnemonic-gui-schema-mirror` (this repo's `design/FOLLOWUPS.md` entry retains `Status: resolved` once `mnemonic-gui` v0.2 ships).

## mnemonic-toolkit [0.8.1] — 2026-05-12

Additive minor release atop v0.8.0. **No breaking changes.** Six new vendor-targeted wallet-import formats added to `mnemonic export-wallet`: `coldcard`, `jade`, `sparrow`, `specter`, `electrum`, `green`. Per-emitter byte-exact fixtures pinned; SPEC §4 missing-info refusal channel exercised end-to-end (Sparrow's missing-threshold + Specter's missing-wallet-name). Internal `wallet_export.rs` reorganized into a `wallet_export/` submodule tree (one file per emitter). v0.7 stable `--format bitcoin-core` / `--format bip388` byte-exact fixtures continue to pass through the new submodule dispatch.

### Added

- **`--format coldcard`** (SPEC v0.8 §5). Coldcard generic JSON skeleton (singlesig BIP-44/49/84) and Coldcard multisig text (wsh / sh-wsh, sorted and unsorted). BIP-86 (P2TR) and `tr-multi-a` / `tr-sortedmulti-a` refuse with byte-exact pointer text per FOLLOWUPS `coldcard-bip86-generic-export-pending-firmware` + `coldcard-tr-multi-a-pending-firmware`.
- **`--format jade`** (SPEC §6). Blockstream Jade `register_multisig.multisig_file` shape — byte-identical to Coldcard's multisig text per Jade's documented format, so the emitter delegates to `emit_coldcard_multisig_text` directly. Singlesig + `tr-multi-a` refuse with byte-exact pointer text.
- **`--format sparrow`** (SPEC §7). Sparrow Wallet wallet-import JSON shape (`drongo/.../wallet/Wallet.java` canonical model). `policyType` from template; `scriptType` from `WalletScriptType`; `defaultPolicy.miniscript.script` built from `@N/**` placeholders for non-taproot templates or descriptor-passthrough (with `#checksum` stripped) for taproot multisig. `--threshold` is REQUIRED for multisig (per SPEC §4 missing-info channel — Sparrow publishes threshold in the miniscript expression and auto-defaulting K=N would emit a wallet that looks like K=N was intentional).
- **`--format specter`** (SPEC §8). Specter Desktop import JSON shape: `{label, blockheight, descriptor, devices}` with canonical BIP-380 `descriptor` (including `#checksum`, contrast with Sparrow). `--wallet-name` is REQUIRED (SPEC §13 R1-L1 — Specter's UX requires explicit labels; defaulting produces a UI regression).
- **`--format electrum`** (SPEC §9). Electrum `wallet_db.py` JSON shape with `seed_version`, `wallet_type`, `use_encryption: false`, and `keystore` (singlesig) or `x1/`..`xN/` (multisig). SLIP-132 conversion: zpub/ypub/upub/vpub for singlesig; capital Zpub/Ypub/Vpub/Upub for multisig. `ELECTRUM_SEED_VERSION_PIN = 17` (historically broadest-accept value; Phase 4 step 0 interactive spike deferred — FOLLOWUPS `electrum-seed-version-spike-pending`). `tr-multi-a` refuses pending Electrum libsecp-taproot support.
- **`--format green`** (SPEC §10). Thin 3-line text file for Blockstream Green's "Import from file" dialog: 2 comment lines + canonical descriptor. Multisig refuses (Green's multisig surface is server-mediated via Green Multisig Shield, not a file-import shape) per FOLLOWUPS `green-native-multisig-pending-server-support`.
- **`--wallet-name <STRING>`** clap flag for formats publishing a wallet label (Coldcard generic JSON, Sparrow, Specter, Electrum, Green). Defaults to `<template-human-name>-<account>` for the template path or `"imported-descriptor"` for descriptor passthrough. Truncated to 20 Unicode scalar values in Coldcard / Jade multisig text per the Coldcard reference format (codepoint-granularity truncation — non-ASCII names are not split mid-codepoint).
- **`@N.master_xpub=<base58>`** slot subkey (depth-0 root xpub, watch-only-class). Parsed by `SlotSubkey::MasterXpub` and validated against `is_legal_set`. Currently refused under `--format coldcard` + singlesig templates pending v0.8.2 plumbing (FOLLOWUPS `coldcard-master-xpub-plumbing-pending`); other formats silently ignore per the per-format ignored-input contract.
- **SPEC §4 missing-info refusal channel** wired end-to-end. Per-emitter `collect_missing` predicates return `MissingField` enum entries (`MasterFingerprint` / `DerivationPath` / `Xpub` / `ScriptType` / `Threshold` / `WalletName` / `IncompatibleFormatForTemplate`). `ToolkitError::ExportWalletMissingFields` routes through `build_missing_fields_refusal` for SPEC §4 byte-exact refusal shape with deterministic field ordering. First emitters to populate: Sparrow (Threshold) and Specter (WalletName).
- **Module reorganization** (internal): `src/wallet_export.rs` → `src/wallet_export/` submodule tree with one file per emitter (`bip388.rs`, `bitcoin_core.rs`, `coldcard.rs`, `jade.rs`, `sparrow.rs`, `specter.rs`, `electrum.rs`, `green.rs`, `pipeline.rs`). The module-root `mod.rs` holds shared types (`WalletScriptType`, `MissingField`, `EmitInputs`, `WalletFormatEmitter` trait, `TaprootInternalKey`, `TimestampArg`) and watch-only validators. No external API changes; v0.7 byte-exact fixtures for `bitcoin-core` / `bip388` continue to pass through the new dispatch.
- **Manual mirror**: `mnemonic export-wallet` flag table updated; new `### Notes` subsection documents the `--wallet-name` 20-char Unicode truncation, the `@N.master_xpub=` parse-but-refuse behavior, the `--threshold` requirement for `--format sparrow` multisig, and the `--wallet-name` requirement for `--format specter`.

### FOLLOWUPS

- New: `coldcard-bip86-generic-export-pending-firmware` (v1+), `coldcard-tr-multi-a-pending-firmware` (v1+), `jade-tr-multi-a-pending-firmware` (v1+), `coldcard-master-xpub-plumbing-pending` (v0.8.2), `electrum-seed-version-spike-pending` (v0.8.2), `electrum-tr-multi-a-pending-libsecp-taproot` (v1+), `electrum-final-seed-version-drift` (informational), `green-native-multisig-pending-server-support` (v1+).
- Resolution-extended on `wallet-export-industry-formats` (entry stays `Status: resolved`; Phase 1 + Phase 5 extension lines added listing the six new formats shipped this cycle).

### Reviewer-loop reports

Persisted under `design/agent-reports/`:
- `v0_8-spec-r1.md`, `v0_8-spec-r2.md` (SPEC promotion + R2 convergence).
- `v0_8-impl-plan-r1.md`, `v0_8-impl-plan-r2.md` (plan promotion + R2 convergence).
- `v0_8-phase-1-coldcard-jade-r1.md`, `v0_8-phase-1_11-r2.md` (Phase 1 R1 + R2 convergence).
- `v0_8-phase-2-sparrow-r1.md`, `v0_8-phase-2-sparrow-r2.md` (Phase 2 R1 + R2 convergence).
- `v0_8-phase-3-specter-r1.md` (Phase 3 R1 convergence).
- `v0_8-phase-4-electrum-seed-version-spike.md` (Phase 4 step 0 deferral record).

## mnemonic-toolkit [0.8.0] — 2026-05-07 [BREAKING]

Minor release atop v0.7.1. **Breaking change** to BIP-38 composite-edge passphrase semantics; new flags + new BIP-85 application + Electrum i18n + taproot multisig export. Two spike-driven deferrals (BIP-38 EC-mult encrypt → v0.8.1; BIP-85 RSA / RSA-GPG → v0.9 pending RUSTSEC-2023-0071 patch). 11 phases (0–10) shipped this cycle.

### [BREAKING] BIP-38 composite-edge passphrase

**Migration:** users running `mnemonic convert --from phrase=... --to bip38 --passphrase X` in v0.7 got BIP-38 output encrypted with passphrase X (dual-purpose); v0.8 produces output encrypted with `""`. Migrate by supplying `--passphrase X --bip38-passphrase X` to preserve v0.7 behavior.

The v0.7 `(phrase, bip38)` and `(entropy, bip38)` composite arms used `--passphrase` for BOTH BIP-39 PBKDF2 (mnemonic extension) AND BIP-38 Scrypt encryption — a dual-purpose dispatch that masked which leg the passphrase was reaching. v0.8 introduces a separate `--bip38-passphrase` flag; the two legs now use independent passphrase inputs. If `--bip38-passphrase` is unset on a composite path, BIP-38 encrypt uses `""` (no fallback to `--passphrase`). Direct `(wif, bip38)` and `(bip38, wif)` edges retain v0.7 single-flag UX: `--bip38-passphrase` falls back to `--passphrase` when unset.

### Added

- **`--bip38-passphrase`** flag on `mnemonic convert` (Phase 1; SPEC v0.8 §12.b). Distinct BIP-38 Scrypt passphrase; see `[BREAKING]` section above.
- **`--passphrase-stdin`** flag on `mnemonic convert` (Phase 1; SPEC v0.8 §5.a). Reads the passphrase from raw stdin preserving NULL bytes. Closes the BIP-38 spec V3 Unicode-NFC NULL-byte gap (POSIX argv cannot carry U+0000); the 2 V3 spec vectors previously `#[ignore]`'d are now active.
- **`mnemonic derive-child --from phrase=...`** (Phase 1, Item #5). Accepts a BIP-39 mnemonic as the master input alongside `--passphrase` for BIP-39 mnemonic extension; internal `phrase → seed → master xprv` conversion before BIP-85 derivation.
- **`mnemonic derive-child --from xprv=-` / `--from phrase=-`** (Phase 1, Item #8). Reads the master from stdin.
- **`mnemonic derive-child --language <code>`** wired to BIP-85 path code + `bip39::Language` wordlist selection (Phase 1, Item #6). Supports the 9 BIP-85-coded languages: `english` (0), `japanese` (1), `korean` (2), `spanish` (3), `simplified-chinese` (4), `traditional-chinese` (5), `french` (6), `italian` (7), `czech` (8). Portuguese refused (no BIP-85 code assigned).
- **`mnemonic derive-child --network`** wired to BIP-85 emission for `--application <hd-seed|xprv>` (Phase 1, Item #7). Testnet emits `c…` WIF / `tprv…` xprv.
- **`mnemonic derive-child --application dice`** (Phase 7). New `--dice-sides <N>` flag; rejection-sampled dice rolls per BIP-85 v1.3.0 §"DICE" via SHAKE256 BIP85-DRNG. Spec reference vector pinned (`m/83696968'/89101'/6'/10'/0'` → `1,0,0,2,0,1,5,5,2,4`).
- **`mnemonic convert --electrum-language <english|spanish|japanese|portuguese|chinese-simplified>`** (Phase 2, Item #9; SPEC v0.8 §14). Adds 4 non-English Electrum wordlists embedded from `spesmilo/electrum` at upstream commit `e1099925e30d91dd033815b512f00582a8795d25`. Distinct from `--language` (BIP-39 wordlist set differs from Electrum's). On Electrum arms, `--electrum-language` wins; `--language` silently ignored. Portuguese is base-1626 (not 2048); base-N arithmetic correctly parameterized.
- **Electrum encode iteration bound** (Phase 2, Item #10). `MAX_ENCODE_ITERATIONS = 2^20` cap on `entropy_to_phrase` rejection-search loop.
- **Electrum SeedVersion stderr info-line** (Phase 2, Item #11). On `(electrum-phrase, entropy)` decode, emits `note: detected Electrum SeedVersion <01|100> (<standard|segwit>)` to stderr.
- **`mnemonic export-wallet --taproot-internal-key <nums|@N>`** (Phase 3, Item #12; SPEC v0.8 §7). `nums` selects the BIP-341 reference NUMS x-only point `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0`. `@N` selects cosigner N as the key-path internal key; cosigner N is removed from the multi_a leaf set, leaving (k-of-(N-1)) script-path multisig. Unblocks `tr-multi-a` / `tr-sortedmulti-a` templates (refused outright in v0.7).
- **`mnemonic export-wallet --descriptor + --format bip388`** interop (Phase 3, Item #13; SPEC v0.8 §6). User-supplied descriptors are parsed, multipath-checked, `#checksum`-stripped, and rendered as BIP-388 wallet_policy with `@N/**` placeholders. Refused for non-multipath descriptors.
- **New direct dependencies**: `unicode-normalization = "0.1"` (Electrum NFKD; was transitive via bip38), `sha3 = "0.10"` (BIP85-DRNG-SHAKE256 for DICE).

### Changed

- **SPEC `design/SPEC_derive_child_v0_7.md`** § 2 + §3 + §4 (Phase 0). §3 BIP-39 byte-slicing formula corrected to canonical `length_in_words * 4 / 3`. §2 + §4 prose updated to document the sentinel-0 convention for fixed-output `--application <hd-seed|xprv>`.
- **`mnemonic derive-child --application <rsa|rsa-gpg>`** refusal text rewritten to reference RUSTSEC-2023-0071 + crate stability (Phase 7). `dice` lifted from out-of-scope to in-scope.
- **`mnemonic export-wallet --template <tr-multi-a|tr-sortedmulti-a>`** refusal text now points the user at `--taproot-internal-key` (Phase 3); was a v0.7 byte-exact "deferred to v0.8" message.
- **BIP-39 §Test Vectors corpus**: `tests/cli_convert_bip39_vectors.rs` refactored from 6 hand-pinned tests to a single parametric loop over the full Trezor 24-vector english corpus (Phase 8). New vendored corpus at `tests/bip39_trezor_vectors.json`.

### Deferred

- **BIP-38 EC-multiplied encrypt** (Phase 5 SPIKE verdict — Phase 4): deferred to v0.8.1 / v0.9. The `bip38 v1.1.1` `Generate` trait covers the owner-only path only with internal `rand::thread_rng()` (non-deterministic) and exposes no intermediate-code workflow + no confirmation code. Hand-rolling the spec-compliant API would cost ~155 LOC of cryptographic code (AES + scrypt + secp256k1 + Unicode normalization). Marginal user value (paper-wallet niche). v0.7.1 EC-multiplied DECRYPT coverage is unchanged. Spike: `design/agent-reports/v0_8-phase-4-bip38-ec-mult-encrypt-spike.md`.
- **BIP-85 RSA + RSA-GPG** (Phase 7 narrowed — Phase 6 SPIKE verdict): deferred pending `rsa` crate stability + user demand. RUSTSEC-2023-0071 (Marvin attack: timing sidechannel against PKCS#1 v1.5 decryption) is **unpatched** as of 2026-05-07 (`patched = []`). Crate is in extended pre-release (`v0.10.0-rc.18`). Adding it as direct dep would import an open advisory into mnemonic-toolkit's `cargo audit` output. Reopen criteria: rsa crate publishes patched stable release OR user requests with stated downstream use case. Spike: `design/agent-reports/v0_8-phase-6-rsa-crate-security-review.md`.

### Internal

- Per-phase code-reviewer rounds: Phase 1 (0C/1I), Phase 2 (0C/2I), Phase 3 (0C/1I), Phase 7 (0C/0I). All findings applied in-phase. Reports persist to `design/agent-reports/v0_8-phase-{N}-review.md`.
- 4 wordlist files added under `crates/mnemonic-toolkit/src/wordlists/electrum_*.txt`. Total ~60KB embedded data.
- `compute_outputs` in `cmd/convert.rs` now returns a triple (`outputs, slip0132_input_variant, electrum_seed_version`) to surface the Electrum SeedVersion to the run-loop for stderr emission.
- `validate_watch_only_resolved` enforced post-resolve in `export-wallet`; Phase 3 cosigner-internal taproot adds `n=1` degenerate-case refusal.

### Test corpus

484 active + 2 ignored (v0.7.1) → 527 active + 2 ignored (v0.8.0). Net: +43 active; the 2 V3 NULL-byte tests previously ignored are now active. The `cli_convert_bip39_vectors.rs` parametric refactor reduces test-function count by 5 (6 hand-pinned → 1 loop) but raises BIP-39 §Test Vectors English coverage from 6/24 to 24/24 cells.

### FOLLOWUPS resolved this cycle (v0.8.0 ship)

- `bip85-spec-prose-byte-formula-clarification` (Phase 0, `4dfea5a`).
- `derive-child-spec-2-grammar-uniformity-tension` (Phase 0, `4dfea5a`).
- `bip38-distinct-passphrase-flag` (Phase 1, `2eef44b`).
- `bip38-spec-vector-3-null-byte-passphrase` (Phase 1, `2eef44b`).
- `bip85-passphrase-protected-master` (Phase 1, `2eef44b`).
- `bip85-non-english-bip39-language-codes` (Phase 1, `2eef44b`).
- `bip85-testnet-emission` (Phase 1, `2eef44b`).
- `bip85-stdin-master-xprv` (Phase 1, `2eef44b`).
- `electrum-non-latin-wordlists` (Phase 2, `5dc83eb`).
- `electrum-encode-iteration-bound` (Phase 2, `5dc83eb`).
- `electrum-version-info-stderr` (Phase 2, `5dc83eb`).
- `tr-multi-a-tr-sortedmulti-a-export-wallet-support` (Phase 3, `86647ca`).
- `export-wallet-descriptor-bip388-interop` (Phase 3, `86647ca`).
- `bip85-dice-application` (Phase 7, `1dde4dc`; split from `bip85-rsa-rsa-gpg-dice-applications`).
- `18-remaining-bip39-trezor-corpus-vectors` (Phase 8, `85694b2`).

### FOLLOWUPS re-tiered

- `bip38-ec-multiplied-encrypt-mode-support`: v0.8 → v0.8.1+ (Phase 4 SPIKE).
- `bip85-rsa-rsa-gpg-applications` (renamed from `bip85-rsa-rsa-gpg-dice-applications` after DICE split): v0.8 → v0.9 / pending-rsa-crate-stability (Phase 6 SPIKE).

## mnemonic-toolkit [0.7.1] — 2026-05-07

Vectors-only patch atop v0.7.0. Pins published §Test Vectors entries from every BIP/SLIP/spec the toolkit cites. No behavior change; no wire-format change. New SPEC `design/SPEC_test_vector_audit_v0_7_1.md` summarizes coverage, discoveries, and out-of-scope classifications. 7 cycle phases (0–7) closed; Phase 8 ships docs + CHANGELOG.

### Added

- **~40 newly-pinned BIP/SLIP test vectors** across 5 specs:
  - **BIP-32** §Test Vectors — 16 derivation cells from TVs 1–4 plus the leading-zero chain-code edge (`tests/bip32_vectors.rs`); Phase 1.
  - **BIP-39** Trezor reference corpus — 6 entries (12-word + 24-word × 3 passphrase variants) at `tests/cli_convert_bip39_vectors.rs`; remaining 18 carry to v0.8. Phase 1.
  - **BIP-49 / BIP-84 / BIP-86** §Test vectors — pinned account-level + receive/change vectors at `tests/cli_convert_address.rs`; Phase 2.
  - **BIP-38** §Test vectors — V3 (`#[ignore]`'d, cite-only — see §3.b of the v0.7.1 audit SPEC) + V5 (Satoshi-compressed) non-EC, plus EC1–EC4 EC-multiplied DECRYPT vectors at `tests/cli_convert_bip38.rs`; Phase 3.
  - **BIP-380** checksum vector 380.1 at `tests/cli_export_wallet.rs::bip380_valid_checksum_round_trip_via_miniscript`; Phase 4.
  - **BIP-388** §Reference Wallet Policies 388.2 + 388.4 template-shape pinning at `tests/cli_export_wallet.rs::cell_{8,9}_*`; Phase 4.
  - **SLIP-0132** §Bitcoin Test Vectors — 3 mainnet single-sig xpubs at `src/slip0132.rs::tests::slip0132_spec_bitcoin_test_vector_*`; Phase 5.
- New SPEC `design/SPEC_test_vector_audit_v0_7_1.md` summarizing audit coverage, discoveries, OOS classifications, and v0.8 carry-overs.

### Changed

- **`SPEC_convert_v0_6.md` §12 erratum** — v0.7.0 incorrectly stated `bip38 = "1.1"`'s `Decrypt` impl rejected EC-multiplied codes. Empirical Phase 3 testing disconfirmed: all 4 spec EC-multiplied vectors (EC1–EC4) decrypt transparently through the existing `(Bip38, Wif)` arm. SPEC §12 now reflects actual capability; encrypt-side EC-mult (intermediate-code workflow) is the new gap, tracked as v0.8 FOLLOWUP. Closed in `2c59b27`.

### Internal

- Audit matrix: `design/agent-reports/v0_7_1-bip-test-vector-audit-matrix.md`. Per-spec §Test Vectors enumerated verbatim with COVERED / MISSING / OUT-OF-SCOPE-PER-{USER,SPEC} classification.
- Test-fn rename in `tests/cli_convert_bip38.rs`: prior `*_vector3_*` (compressed-Satoshi) renamed to `*_vector4_*` to align with BIP-38 spec numbering after Phase 3.A pinned spec V3 (Unicode-NFC) at the canonical V3 slot. No coverage change; rename preserves test behavior byte-for-byte.

### Fixed

— (no impl bug fixes in this cycle.)

### FOLLOWUPS resolved

- `bip38-spec-section-12-ec-multiplied-erratum` — SPEC §12 erratum corrected (Phase 3.B, `2c59b27`).

### FOLLOWUPS filed (v0.8 carry)

- `bip38-ec-multiplied-encrypt-mode-support` — emit BIP-38 EC-multiplied form via intermediate codes.
- `bip38-spec-vector-3-null-byte-passphrase` — NULL-safe passphrase input channel needed to exercise V3 Unicode-NFC vector end-to-end.

### Test corpus

444 lib + integration tests at v0.7.0 → 484 at v0.7.1 (+40 active). 2 → 4 ignored (+2 V3 Unicode-NFC encrypt + decrypt cells, `#[ignore]`'d pending NULL-safe input channel — see FOLLOWUP).

## mnemonic-toolkit [0.7.0] — 2026-05-06

### Added

- `mnemonic convert` gains 4 new `NodeType` targets: `bip38`, `minikey`, `electrum-phrase`, `address`.
- **BIP-38 encrypt/decrypt** edges (`Wif↔Bip38`) plus composite paths (`phrase|entropy → bip38` via the `wif` intermediate). New crate dependency `bip38 = "1.1"` (Apache-2.0). SPEC §12.
- **Casascius mini-private-key** decode (`MiniKey → Wif`); SHA256 self-checksum rule per Casascius's typo-check. One-way edge (no encode direction; key search is non-deterministic). SPEC §13.
- **Electrum native seed format** (`ElectrumPhrase ↔ Entropy`); 4 SeedVersion dispatch (`01` standard, `100` segwit, `101`/`102` 2FA) via HMAC-SHA512 prefix; 2FA versions refused. Composite paths via `entropy` reach `phrase`/`xprv`/`xpub`/`wif`/etc. SPEC §14.
- **Address derivation** (`Xpub → Address`); `--script-type` flag with inference from `--template` for BIP-44/49/84/86 → P2PKH/P2SH-P2WPKH/P2WPKH/P2TR. SPEC §10.a.
- New subcommand **`mnemonic export-wallet`** — Bitcoin Core `importdescriptors` JSON (default) + BIP-388 `wallet_policy` JSON. Sparrow / Specter formats refuse with v0.8 deferral stubs. `--range` / `--timestamp` / `--bitcoin-core-version` overrides. Watch-only by definition (refuses entropy/phrase slot input). New SPEC `design/SPEC_export_wallet_v0_7.md`.
- New subcommand **`mnemonic derive-child`** — BIP-85 deterministic entropy via HMAC-SHA512 at `m/83696968'/<application>'/<index>'`. 6 in-scope applications: `bip39`, `hd-seed`, `xprv`, `hex`, `password-base64`, `password-base85`. RSA / RSA-GPG / DICE applications refused with v0.8 deferral stubs. New SPEC `design/SPEC_derive_child_v0_7.md`.

### Changed

- `NodeType` enum extended with 4 variants (`Bip38`, `MiniKey`, `ElectrumPhrase`, `Address`). `is_secret_bearing` extended for `Bip38` + `ElectrumPhrase`.

### Internal

- SPEC §11 carry-over: new `slip0132::tests::spec_info_line_template_matches_production_render` reads `SPEC_convert_v0_6.md` text via `include_str!` and asserts byte-equality against `render_slip0132_info_line` for all 8 SLIP-0132 variants. Closes the SPEC↔production drift hazard.
- `verify_bundle.rs` callsite-comments at `:208/:261/:336/:406` gain a SPEC §11 v0.7 amendment cross-pointer (Option B per architect R1-I8 — verify-bundle remains silent on SLIP-0132 input-normalization signals; documented as intentional checker semantics).
- New module `bip85.rs` — BIP-85 derivation primitive + 6 application dispatchers.
- New module `electrum.rs` — `SeedVersion` enum + HMAC-SHA512 prefix dispatch + entropy↔phrase encode/decode.
- New module `wallet_export.rs` — descriptor pipeline + Bitcoin Core / BIP-388 formatters + watch-only validator.
- 3 new error variants on `ToolkitError`: `ExportWalletSecretInput`, `ExportWalletFormatStub(&'static str)`, `ExportWalletTaprootMultisigUnsupported(&'static str)` (all exit 2). Plus 3 new derive-child variants: `DeriveChildUnsupportedApp`, `DeriveChildLengthOutOfRange`, `DeriveChildLengthNotApplicable` (all exit 2).

### Fixed

- `convert.rs:565` — `--to` unknown-node hint string was stale since v0.6 (omitted `bip38`, `minikey`, `electrum-phrase`, `address`); now enumerates all 13 NodeType tokens.

### FOLLOWUPS resolved

- `slip0132-info-line-spec-text-not-byte-pinned` — SPEC §11 byte-pin test shipped (Phase 7, `354c945`).
- `verify-bundle-discards-slip0132-input-variant-asymmetry` — Option B locked: 4 callsite-comments cross-pointed to SPEC §11 v0.7 amendment; intentional checker semantics (Phase 7, `354c945`).
- `bip38-encrypted-wif` — `Wif↔Bip38` edges + composite paths via `bip38 = "1.1"` (Phase 1, `c3d0a85`).
- `casascius-mini-private-key` — `MiniKey → Wif` decode-only edge with SHA256 self-checksum (Phase 2, `89d29ab`).
- `bip85-deterministic-entropy` — `mnemonic derive-child` subcommand with 6 in-scope apps (Phase 6, `965cc3e`).
- `electrum-native-seed-format` — `ElectrumPhrase ↔ Entropy` edges with 4-version dispatch + 2FA refusal (Phase 3, `892139c`).
- `address-derivation-from-xpub-path` — `(Xpub, Address)` edge with `--path` mandatory + `--script-type` template-inferred (Phase 4, `940ec0b`).
- `wallet-export-industry-formats` — `mnemonic export-wallet` subcommand with Bitcoin Core importdescriptors + BIP-388 wallet_policy (Phase 5, `3821f66`).

### Test corpus

363 lib + integration tests at v0.6.2 → 444 at v0.7.0 (2 ignored, pre-existing).

## mnemonic-toolkit [0.6.2] — 2026-05-06

### Added

- `mnemonic convert` and `mnemonic bundle` now emit a stderr informational line when a SLIP-0132 input prefix (`ypub | Ypub | zpub | Zpub` mainnet; `upub | Upub | vpub | Vpub` testnet) is silently normalized to its BIP-32 neutral form (`xpub` / `tpub`). Closes the v0.6.1 UX gap where intent signals were lost without trace. Emission is independent of `--json` and `--no-engraving-card`. Multi-slot bundles emit one note per slot in slot-index ascending order.

### Changed

- SPEC §5.5.a relaxed: the secret-on-stdout warning is the last stderr write *when it fires*; informational notes precede the engraving-card block. Deterministic stderr ordering: `informational notes → engraving card → secret-on-stdout warning (conditional)`. See `design/SPEC_mnemonic_toolkit_v0_5.md` §5.5.a (v0.6.2 amendment block).

### Internal

- `slip0132::normalize_xpub_prefix` return type changed from `Result<String, ToolkitError>` to `Result<(String, Option<&'static str>), ToolkitError>` to thread the variant-name signal up to the emission layer. `pub(crate)` API only — no impact on external consumers.
- `bundle::resolve_slots` return type extended with a `Vec<(u8, &'static str)>` slot-index→variant-name signal channel. `pub(crate)` API only.

### Fixed

- `cmd::convert::run` had duplicate `// 8)` step-label comments (`8) Compute outputs.` and `8) Emit.`). Renumbered for sequence clarity. Closes FOLLOWUPS `convert-run-step-numbering-duplicate-8`.

### FOLLOWUPS resolved

- `slip0132-input-normalization-stderr-info` — SLIP-0132 input-normalization stderr info-line shipped (this release).
- `convert-run-step-numbering-duplicate-8` — duplicate `// 8)` step labels in `cmd::convert::run` renumbered (this release).

## mnemonic-toolkit [0.6.1] — 2026-05-06

### What's new (v0.6.1 — `convert` polish + `bundle` retrofit)

A patch release bundling four small additive items consolidated under a single SPEC-amendment cycle (`SPEC_convert_v0_6.md` v0.6.1 + `SPEC_mnemonic_toolkit_v0_5.md` §5.5.a). All four items are additive — no breaking changes; no wire-format change to existing bundles or convert outputs.

- **`phrase`/`entropy` → `wif` edge** (SPEC-A) — previously deferred-in-code (BadInput at `convert.rs:482-484`); now a fully supported edge that derives a leaf privkey at an explicit `--path` and serializes via `bitcoin::PrivateKey::to_wif()` with `compressed: true` (BIP-32 §4 mandate). `--path` is REQUIRED — the toolkit does NOT auto-default a path from `--template`/`--account`. Refusal exits 2 (`ToolkitError::ConvertRefusal`) when `--path` is absent. SPEC §8 invariant: `--passphrase` is meaningful for this edge (the PBKDF2 pipeline is traversed).
- **SLIP-0132 prefix-tolerant input** (SPEC-B / new §11) — `convert --from xpub=...`, `bundle --slot @0.xpub=...`, and `verify-bundle --slot @0.xpub=...` accept SLIP-0132 prefix variants in addition to BIP-32 neutral `xpub`/`tpub`. 8 prefixes recognized: `ypub`/`Ypub`/`zpub`/`Zpub` (mainnet → swap to `xpub`); `upub`/`Upub`/`vpub`/`Vpub` (testnet → swap to `tpub`). Implementation in new `src/slip0132.rs` via base58check decode → version-byte swap → re-encode (key material is unchanged; encoding-only normalization). Unknown prefix exits 1 with byte-exact stderr. Spike: `design/agent-reports/spike-slip0132-v0_6_1-pre-spec.md`.
- **`--xpub-prefix <variant>` output flag** (SPEC-C / new §11.a) — emit `xpub`-typed targets with a SLIP-0132 prefix. 5 flag values (`xpub` default / `ypub` / `Ypub` / `zpub` / `Zpub`); testnet variants are network-context-derived via `--network`, not separate flag values. **`--network` REQUIRED when `--xpub-prefix` is non-default** (refuses with byte-exact stderr; eliminates a "testnet user gets mainnet zpub" bug class). Silent no-op on non-xpub targets. New `(xpub, xpub)` edge in §2 supports the round-trip primitive cited in §11.a.
- **`bundle` secret-on-stdout warning** (SPEC-D / new §5.5.a) — `bundle.rs::emit_unified` now emits the same byte-exact stderr warning as `convert` §7 when `Bundle::any_secret_bearing()` returns true. Watch-only invocations (all `ms1[i] == ""` sentinel per §5.8) suppress it. Wif-only-bundle limitation per SPEC: WIF slots produce empty-string ms1, so the warning is silently suppressed even when WIF is supplied as input — the warning's scope is BIP-39 entropy emission, not WIF.

### Test corpus

- **239 lib + 100 integration tests** at v0.6.1 (was 230 lib + 67 integration at v0.6.0). Net +9 lib unit tests (all in new `slip0132.rs`) + 33 integration tests:
  - `cli_convert_slip0132.rs` (NEW, 15 tests).
  - `cli_convert_round_trips.rs` (NEW, 3 tests).
  - `cli_convert_happy_paths.rs` (+9: 3 from Phase B `phrase/entropy → wif`, 6 from Phase E coverage tightening).
  - `cli_convert_refusals.rs` (+2: Phase B no-`--path` refusal for both phrase and entropy sources).
  - `cli_bundle_full.rs` (+2: Phase D text-mode + JSON-mode positive warning assertions).
  - `cli_bundle_watch_only.rs` (+1: Phase C zpub cross-cut, plus an in-place stderr negative assertion).
  - `cli_descriptor_mode.rs` (+1: Phase C descriptor-mode zpub cross-cut).
  - `cli_bundle_multisig.rs` (in-place stderr negative assertion only; no new test function).
- 16-cell parametric `bundle_full_16_cells_byte_exact_against_pinned_vectors` continues to pass — the new bundle stderr warning does not perturb the wire-format byte-identity invariant.

### FOLLOWUPS resolved

- `secret-on-stdout-warning-bundle-retrofit` (resolved Phase D, commit `66ff7c0`).
- `convert-phrase-to-leaf-wif` (resolved Phase B, commit `62b4f23`).
- `convert-test-coverage-tightening` (resolved Phase E, commit `59140c5`).
- `convert-slip0132-prefix-support` (resolved Phase C, commit `bb77164`).

### Internal

- New module `src/slip0132.rs` with `XpubPrefix` enum + `normalize_xpub_prefix` + `apply_xpub_prefix` + clap value-parser. 9 inline unit tests pin the byte-level swap mechanics against the BIP-84 reference vector.
- `derive_slot::derive_bip32_at_path` — sibling helper to `derive_bip32_from_entropy` for path-driven leaf derivation (used by the `phrase/entropy → wif` edge).
- `convert.rs::edge_uses_pbkdf2` extended to include `Wif` per SPEC §8 v0.6.1 invariant.

## mnemonic-toolkit [0.6.0] — 2026-05-06

### What's new (v0.6.0 — `mnemonic convert` subcommand)

A new orthogonal subcommand for single-format conversions between BIP-39 phrase, BIP-39 entropy, BIP-32 xpriv/xpub, WIF, fingerprint, path, and the codex32 codec encodings ms1 and mk1. The subcommand makes conversions a first-class CLI operation rather than a side-effect of bundle synthesis.

- **New subcommand `mnemonic convert`**, governed by the new `design/SPEC_convert_v0_6.md` (architect-approved 0C/0I at r3).
- **9-node typed conversion graph.** `phrase`, `entropy`, `xpub`, `xprv`, `wif`, `fingerprint`, `path`, `ms1`, `mk1`. Direct edges enumerated in `is_supported_direct_edge`; any (from, to) NOT in the set is auto-refused as a one-way barrier (exit 2). Deferred nodes (`seed`, `raw_privkey`) are documented but not yet emit/accept-supported (gated on ms-codec v0.2). `md1` is deliberately excluded (descriptors are bundle artifacts).
- **Three refusal classes** (one-way cryptographic barrier / lossy compression / cross-format pivot) with byte-exact stderr templates. `xpub → mk1` has a distinct refusal redirecting to `mnemonic bundle` (mk1 cards bind xpubs to specific policies via `policy_id_stubs`; standalone encoding is meaningless).
- **`--from`/`--to` grammar.** Single-from-value v0.6 constraint (one primary value-bearing `--from` plus optional side-input `--from path=...` / `--from fingerprint=...`); multi-value `--from` reserved for future `--slot @N` indexing.
- **`--from <node>=-` stdin convention** for any single-line node; `mk1` reads whitespace-separated tokens from stdin.
- **ConvertJson schema-1 envelope** independent of `BundleJson`. `from_value` omitted when `from_node` is secret-bearing (privacy hygiene); `to` array preserves `--to` argument order.
- **Side-channel hygiene:** stderr warning when secret material is on stdout. New convention in v0.6; bundle retrofit tracked at FOLLOWUP `secret-on-stdout-warning-bundle-retrofit`.
- **`--passphrase` ignored-on-non-PBKDF2-edge stderr warning** — explicit (higher-stakes than other ignored side-inputs).
- **`wif → xpub` sentinel stderr warning** — emits depth-0 sentinel xpub with zeroed chain code; warns the resulting xpub is not BIP-32 derivable. Refuses `wif → xpub --path m/...` (chain code destroyed).
- **`derive::DerivedAccount` extended** with `account_xpriv: Xpriv` field to support the `phrase/entropy → xprv` edge. Both `derive::derive_full` and `derive_slot::derive_bip32_from_entropy` populate it.
- **New error variant** `ToolkitError::ConvertRefusal(String)`; exit code 2.

### Test corpus

230 lib + 67 integration tests pass (was 230 lib + 44 integration in v0.5.2). 23 new convert tests across 4 files: `cli_convert_happy_paths.rs` (11 edges + mk1→xpub decode), `cli_convert_refusals.rs` (7 refusal classes, byte-exact stderr), `cli_convert_json.rs` (3 envelope shape tests), `cli_convert_help_fixtures.rs` (2 help-text smoke tests).

### FOLLOWUPS

- New: `secret-on-stdout-warning-bundle-retrofit` — apply v0.6 §7 secret-on-stdout warning to `bundle` for cross-tool consistency.
- New: `convert-seed-and-raw-privkey-nodes` — add `seed`, `raw_privkey`, `xprv`-via-ms1, `seed`-via-ms1 nodes when ms-codec v0.2 ships.
- New: `convert-phrase-to-leaf-wif` — implement `phrase/entropy → wif` (path-to-leaf-WIF derivation; deferred from v0.6).

### Wire format

Bundle/verify-bundle wire format unchanged. Convert subcommand is additive.

### Architect review reports

- `design/agent-reports/spike-convert-v0_6_0-pre-spec.md` — Phase 0 codec call-shape spike.
- `design/agent-reports/v0_6_0_phase_spec_r3.md` — SPEC 0C/0I at r3.
- `design/agent-reports/v0_6_0_phase_impl_r1.md` — implementation review (0C/2I/2L/1N → 0C/0I after foldings).

## mnemonic-toolkit [0.5.2] — 2026-05-06

### What's new (v0.5.2 — derive_slot helper extraction)

Pure refactor patch. Sets up a shared call site for the upcoming v0.6.0 `mnemonic convert` subcommand without conflating refactor risk with new-feature risk.

- **`derive_slot.rs` (NEW).** `derive_bip32_from_entropy(entropy, passphrase, language, network, template, account) -> Result<DerivedAccount>` consolidates the BIP-39 + BIP-32 derivation spine that was duplicated between `bundle::resolve_slots`'s phrase and entropy branches.
- **`derive::DerivedAccount` extended.** New field `account_path: DerivationPath` populated via the helper. `derive_full` is now a thin wrapper that parses the phrase to entropy and delegates.
- **`bundle::resolve_slots` simplified.** Phrase + entropy branches each shrink from ~22 LOC to ~10 LOC, calling the shared helper. The xpub / wif / xprv-rejected branches stay unchanged.

### Wire format

Byte-identical to v0.5.1. 230 lib + 44 integration tests pass (2 lib ignored, pre-existing). The pre-shipped 16-cell parametric fixture in `cli_bundle_full.rs` continues to match.

### Architect review report

- `design/agent-reports/v0_5_2_phase_extract_r1.md` (0C/0I — APPROVED; 1 unused-import nit folded inline).

## mnemonic-toolkit [0.5.1] — 2026-05-06

### What's new (v0.5.1 — close the v0.5.0 partial-delivery deferrals)

v0.5.1 closes the 2 FOLLOWUPS deferred from v0.5.0 (`legacy-cli-flag-deletion` + `legacy-flag-deprecation`). The unified `--slot @N.<subkey>=<value>` syntax is now the sole input shape for slot-bearing data; the v0.4-era legacy CLI flags are deleted entirely from `BundleArgs` + `VerifyBundleArgs` along with their alias plumbing.

- **Phase A.1a — source-side deletions.** 6 legacy fields (`--phrase`, `--xpub`, `--master-fingerprint`, `--cosigner`, `--cosigners-file`, `--cosigner-count`) deleted from both `BundleArgs` and `VerifyBundleArgs`. `bundle::bundle_args_to_slots` and `slot_input::expand_legacy_to_slots` shims (+ 5 unit tests) deleted entirely. 9 mode-violation guards swept from `bundle.rs::run`; 11 mode-text consts removed (`PASSPHRASE_WITH_XPUB`, `LANGUAGE_WITH_XPUB`, `XPUB_NEEDS_FINGERPRINT`, `FINGERPRINT_WITHOUT_XPUB`, `XPUB_STDIN`, `XPUB_AND_COSIGNER`, `COSIGNER_AND_COSIGNERS_FILE`, `COSIGNER_COUNT_WITHOUT_MULTISIG`, `PRIVACY_WITH_XPUB`, `ACCOUNT_INCOMPATIBLE_TEMPLATE`, `DESCRIPTOR_WITH_COSIGNER_COUNT`); 3 retained guards: `THRESHOLD_WITHOUT_MULTISIG`, `PATH_FAMILY_WITHOUT_MULTISIG`, `DESCRIPTOR_AND_TEMPLATE` (plus the v0.3 retained descriptor-mode set).
- **Phase A.1d — verify-bundle slot dispatch refactor.** `VerifyBundleArgs` gains a `pub slot: Vec<SlotInput>` field with parity to `BundleArgs::slot`. `bundle::resolve_slots` refactored to take an explicit args-tuple `(template, network, account, language, passphrase)` and promoted to `pub(crate)`; both `bundle.rs` and `verify_bundle.rs` share the helper. `verify_bundle::run` reshaped to dispatch via slot-shape detection; `run_full` / `run_watch_only` / `run_multisig` / `descriptor_mode_verify_run` rewired to consume slots through `synthesize_unified` (template mode) or `synthesize_descriptor` (descriptor mode).
- **Phase A.1b/c — test corpus migration.** 3 `cli_mode_violations*.rs` files deleted (~584 lines, 61 legacy-flag references). New `cli_mode_violations_v0_5.rs` (6 tests; byte-exact stderr) covers the 3 retained guards.
- **Phase A.2 — consumer test rewrites.** 13 `cli_*.rs` integration test files rewritten per the v0.5.0 mapping table. Special handling: `cli_unified_slot.rs` row-6 collision test + dead `TREZOR_BIP84_XPUB` const deleted; `cli_bip388_distinctness.rs` row-5-conflict test deleted (trap unreachable post-`--cosigner-count` deletion).
- **Phase A.3 — SPEC §6.6 partial-delivery note removal.** The v0.5.0 SPEC paragraph acknowledging the deferral is deleted; the §6.6 table now reflects shipped state.
- **Path-defaulting refinement.** `bundle::resolve_slots` Xpub branch now defaults the path from `template.derivation_path(network, account)` when the slot lacks an explicit `Path` subkey. Preserves v0.4 watch-only path-default semantics; required for verify-bundle round-trip on bip84/etc account-paths.

### Breaking changes

Per "no users yet → break anything" license:

- **6 legacy CLI flags deleted entirely.** `--phrase`, `--xpub`, `--master-fingerprint`, `--cosigner`, `--cosigners-file`, `--cosigner-count` are now unknown to clap (exit 2 unknown-arg). Use `--slot @N.<subkey>=<value>` instead.
- **Mode-violation pre-check ladder reduced.** 9 guard branches removed; 3 retained. Stderr text for the 3 retained guards is unchanged byte-for-byte.

### Test corpus

230 lib + 44 integration tests pass (2 lib ignored, pre-existing). Net delta: -6 lib (5 expand-legacy unit tests + 1 watch-only-stderr test), -3 integration files (cli_mode_violations*.rs), +1 integration file (cli_mode_violations_v0_5.rs), -2 integration tests within rewritten files.

### Carry-forward

v0.5.0 schema-4 `bundle --json` envelopes continue to emit byte-identically. The legacy-flag → `--slot` rewrite is wire-format-neutral.

### Architect review reports

- `design/agent-reports/v0_5_1_phase_atomic_r1.md` (Commit 1, 0C/0I/0L/2N).
- `design/agent-reports/v0_5_1_phase_spec_r1.md` (Commit 2, 0C/0I).

## mnemonic-toolkit [0.5.0] — 2026-05-06

### What's new (v0.5.0 — bundle the v0.4.5-nice-to-have + open `*-nice-to-have` deferrals)

v0.5.0 closes 13 open FOLLOWUPS across 6 of the 7 planned phases. The user's strongest "no users yet → break anything" license is exercised: a deliberate SPEC §4.11.b reversal (typed-DerivationPath equality), a JSON envelope `engraving_card` field deletion, a four-case ms1 short-circuit table with byte-exact `decode_error` strings, and a `MappingFailure` enum for mk1 cosigner-mapping diagnostics.

A new SPEC document `design/SPEC_mnemonic_toolkit_v0_5.md` is created (v0.4 retained for historical reference). Cycle artifacts: `/home/bcg/.claude/plans/robust-cooking-kazoo.md` (in-plan-mode brainstorm + SPEC + plan all converged 0C/0I across multiple architect rounds).

- **Phase S0 — SPEC v0.5 document.** New `SPEC_mnemonic_toolkit_v0_5.md` with 6 normative amendments: §4.11.b deliberate reversal, §5.7 line 103 multiset semantics for `md1_xpub_match`, §5.7 line 104 four-case ms1 table, §5.7 NEW mk1-mapping-diagnostic paragraph, §5.5 `engraving_card` field deletion, §6.6 legacy-flag-deletion sketch (full deletion deferred to v0.5.1).
- **Phase B — multisig helper polish (5 items).** B.1 new `helper_multisig_full_emits_3plus6n_checks_in_spec_order` unit test. B.2 positional-fallback condition refactored to `match`. B.3 `md1_xpub_match` now multiset (sort-then-compare with multiplicity). B.4 `MappingFailure` enum (`NotSupplied` / `DecodeFailed(String)` / `XpubNotInPolicy`) replaces `Vec<Option<&KeyCard>>`; precedence `XpubNotInPolicy > DecodeFailed > NotSupplied`. B.5 four-case ms1 emission per SPEC §5.7 line 104 — full-mode supplied-absent case now `passed: false` (was `passed: true` in v0.4.5) with byte-exact `decode_error: "error: ms1[{i}] expected (full-mode bundle) but not supplied"`.
- **Phase C — SPEC reversals (3 items).** C.1 `check_key_vector_distinctness` switches from raw-string `path_raw == path_raw` to typed `path == path` (folds `h` → `'`). v0.4.1 `bip388_h_vs_apostrophe_paths_distinct_under_raw_string` test migrated to `bip388_h_vs_apostrophe_paths_collide_under_typed_equality_v0_5`. C.2 SPEC-only codification of watch-only spurious-`--ms1` short-circuit + new integration test. C.3+C.4 `detect_removed_subcommand` trap deleted entirely (~80 lines including 5 inline tests); 2 byte-exact-stderr tests migrated to clap-fallback exit-64 assertions.
- **Phase D — schema-2/3 placeholder rejection deletion.** `load_bundle_json_into_args`'s peek-and-reject `schema_version` branch deleted (~16 lines including the FOLLOWUP placeholder pointer). Schema-mismatch envelopes now fail at the underlying field extraction.
- **Phase E — `origin_path` null unification (single-sig).** New `origin_path_for_json(path_raw)` helper returns `None` when `path_raw.is_empty()` (was `Some("m")` via the v0.4.2 normalize fallback).
- **Phase F — text-mode trailing-space fix.** Three identical `writeln!` emit sites in `cmd/verify_bundle.rs` rewritten to branch on `c.detail.is_empty()` (no more `"md1_xpub_match: skipped "` trailing space).
- **Phase A.3 — engraving-card dead-field cleanup.** `BundleJson.engraving_card: Option<String>` field DELETED + 2 always-`None` initializers DELETED + stale doc-comment rewritten. Active stderr emission path (`build_unified_card` + `engraving_card_unified`) and `--no-engraving-card` CLI flag both preserved.

### Deferred to v0.5.1 (Phase A scope reduction)

- **`legacy-cli-flag-deletion`** — Delete `--phrase`, `--xpub`, `--cosigner`, `--master-fingerprint`, `--cosigner-count`, `--cosigners-file` from `BundleArgs` + `VerifyBundleArgs`. Rewrite ~25 integration tests (~1500 LOC churn) to use `--slot @N.<subkey>=<value>` syntax exclusively.
- **`legacy-flag-deprecation`** — superseded by the deletion above.
- **Mode-violation guard sweep + new `cli_mode_violations_v0_5.rs`** — 9 guards delete; 3 retain (`THRESHOLD_WITHOUT_MULTISIG`, `PATH_FAMILY_WITHOUT_MULTISIG`, `DESCRIPTOR_AND_TEMPLATE`). New test file pinning the 3 retained guards.

Per the plan's explicit scope-reduction trigger, the ~2500 LOC of mechanical-but-error-prone churn is deferred to its own cycle, matching the v0.4.4→v0.4.5 helper-foundation-then-rollout pattern.

### Breaking changes

Per "no users yet → break anything" license:

- **JSON envelope `BundleJson.engraving_card` field DELETED.**
- **JSON envelope `verify-bundle` `mk1_decode[i]` `decode_error` strings changed** (per SPEC §5.7 mk1-mapping diagnostic; was conflated as "skipped: mk1[i] not supplied or decode failed"; now distinguishes 3 modes).
- **JSON envelope `verify-bundle` multisig `ms1_decode[i]` / `ms1_entropy_match[i]` semantics changed** (case 4: `passed: false` for full-mode supplied-absent; was `passed: true` in v0.4.5).
- **JSON envelope `verify-bundle` `md1_xpub_match` is now multiset-equality** (was ordered Vec equality).
- **JSON envelope `bundle` `origin_path` field is `null` for absent paths** (was `"m"` in v0.4.2 unified-slot watch-only).
- **BIP-388 distinctness now treats `48h/0h` and `48'/0'` as the same path** (v0.4 raw-string equality REVERSED). Existing tests using `h`/`'` notation differences as a distinctness lever migrated.
- **`detect_removed_subcommand` trap deleted** — `mnemonic bundle multisig-full` now rejected by clap fallback (exit 64) instead of the byte-exact pre-clap stderr.
- **`--bundle-json` schema-2/3 rejection deleted** — schema-mismatch envelopes fail at field extraction (no more placeholder error pointer).
- **Plain-text `verify-bundle` output no longer has trailing spaces** when `detail` is empty.

### Wire-bit-identical guarantee

v0.4.5 schema-4 `bundle --json` envelopes continue to emit byte-identically EXCEPT for the deleted `engraving_card: null` field and `origin_path: null` (was `"m"`) for unified-slot single-sig watch-only.

### Test corpus

236 lib unit tests + 22 integration suites pass (was 243+22 in v0.4.5; net -7 lib over the cycle from C.3+C.4 trap deletion offsetting B+C+F additions).

### Cycle artifacts

- Plan: `/home/bcg/.claude/plans/robust-cooking-kazoo.md` (in-plan-mode brainstorm + SPEC + plan all converged 0C/0I).
- SPEC: new `design/SPEC_mnemonic_toolkit_v0_5.md`.
- Per-phase reports: `design/agent-reports/phase-{S0,B,C,D,E,F,A}-*-review-r1.md`.

### Architect-review history

- Brainstorm: 2 rounds (r1 0C/2I/3L → addressed; r2 0C/0I/2L → addressed).
- SPEC: 3 rounds (r1 0C/2I/2L → addressed; r2 0C/1I/1L → addressed; r3 0C/0I → APPROVE).
- Implementation plan: 2 rounds (r1 0C/3I/3L → addressed; r2 0C/0I/2L → addressed).
- Per-phase reviews: S0 0C/2I addressed; B-F 0C/0I; A 0C/0I (scope-reduced).
- Final cross-phase review: APPROVED 2026-05-06 (2 Important re: CHANGELOG arithmetic + SPEC §6.6 partial-delivery note both addressed inline; 2 Low/Nit deferred).

---

## mnemonic-toolkit [0.4.5] — 2026-05-06

### What's new (v0.4.5 helper call-site rollout + 9/3+6N descriptor-mode parity)

v0.4.5 finishes the v0.4.4 helper-foundation work by wiring `emit_verify_checks` into all four production verify-bundle dispatch paths (`run_full`, `run_watch_only`, `run_multisig`, `descriptor_mode_verify_run`), expanding the helper to emit the SPEC §5.7 3+6N multisig schema, dropping the legacy `stub_linkage` v0.1 leftover, and adding forensic-field integration tests. Per the user's "no users yet → ignore migration" license, the JSON envelope check-array shape changes are taken directly without compatibility shims.

- **Phase P.3+P.6 — `run_full` + `run_watch_only` via helper.** Replaced ~270 lines of duplicated push-site logic with helper-routed shapes (~50 lines each). Deleted `verify_md1_and_stub` (~107 lines), `verify_md1_only` (~58 lines), `watch_only_checks` (~210 lines), and 5 obsolete unit tests (~165 lines). The single-sig 9-check JSON envelope shape changes: `stub_linkage` is dropped (was a v0.1 leftover with no SPEC §5.7 equivalent); `ms1_decode` joins at position 0 (canonical SPEC §5.7 ordering). `cli_json_envelopes.rs` test pin migrated in lockstep. Runs cmd/verify_bundle.rs from 2365 → 1707 lines (-658 net).
- **Phase P.4 — multisig 3+6N helper expansion.** New `emit_multisig_checks` (~280 lines) implements SPEC §5.7 line 103 multisig schema: 6N per-cosigner [i]-indexed checks (`ms1_decode[i]`, `ms1_entropy_match[i]`, `mk1_decode[i]`, `mk1_xpub_match[i]`, `mk1_fingerprint_match[i]`, `mk1_path_match[i]`) interleaved by cosigner, then 3 shared md1 checks (`md1_decode`, `md1_wallet_policy`, `md1_xpub_match`). Watch-only / wif slots short-circuit ms1 checks per SPEC §5.7 lines 104-106. `run_multisig` body collapses from ~450 lines to ~85 lines via synthesize → SuppliedCards → helper. JSON envelope shape change: per-cosigner `md1_xpub_match[i]` (×N) replaced by single shared `md1_xpub_match`; per-cosigner `stub_linkage[i]` (×N) dropped entirely (no SPEC §5.7 equivalent). New helper unit test `helper_multisig_watch_only_emits_3plus6n_checks_in_spec_order` pins the 3+6N name vec via the watch-only synthesis path; full-mode multisig 3+6N unit-level coverage is open as FOLLOWUP `verify-bundle-multisig-helper-full-mode-unit-test` (covered end-to-end by `cli_bundle_multisig.rs`).
- **Phase P.5 — descriptor-mode rewrite (closes 9/3+6N parity).** `descriptor_mode_verify_run` body's v0.3 3-element coarse ladder (`ms1_entropy_match`, `mk1_match`, `md1_match`) replaced with `emit_verify_checks(&expected, &supplied, descriptor.n > 1)` — yields the same SPEC §5.7 9 / 3+6N schema as template-mode. Plain-text output format also aligned to template-mode (`{name}: ok|fail {detail}` per check + `result: {result}` trailer). Closes FOLLOWUP `verify-bundle-9-3plus6n-descriptor-mode-parity`.
- **Phase L — helper foundation cleanup.** L-1: `emit_verify_checks` doc-comment §5.8 → §5.7 (watch-only short-circuit semantics live in §5.7; §5.8 is the MsField wire format). L-2: `MkField::Multi` early-return arm in the single-sig branch replaced with `unreachable!()` — converts silent data truncation into loud invariant violation now that the helper is live. Closes FOLLOWUP `verify-bundle-helper-foundation-cleanup-v0.4.5`.
- **Phase P.7 — forensic-field integration tests.** New `cli_verify_bundle_forensics.rs` (3 tests): pass-checks omit forensic fields per `#[serde(skip_serializing_if = "Option::is_none")]`; garbage-payload tamper exercises `decode_error` population on `ms1_decode`; watch-only mode emits `decode_error: "skipped: watch-only slot"` on `ms1_decode` + `ms1_entropy_match`.

### Deferred to v0.4.5-nice-to-have / v0.4.6+

- **`verify-bundle-multisig-md1-xpub-match-set-equality`** — `md1_xpub_match` uses ordered Vec equality. SPEC §5.7 "all N pubkeys match" arguably implies set semantics. Triggered only by descriptor-mode where user provides non-canonical slot order. Re-evaluate after descriptor-mode use cases surface.
- **`verify-bundle-multisig-cosigner-mapping-diagnostic`** — distinguish "card not supplied" from "xpub not in policy" failure modes (currently conflated as "skipped: mk1[i] not supplied or decode failed").
- **`verify-bundle-multisig-missing-ms1-passes-true`** — full-mode multisig with no `--ms1` supplied reports `passed: true` for `ms1_decode[i]`/`ms1_entropy_match[i]`. SPEC §5.7 doesn't address this case.
- **`verify-bundle-watch-only-spurious-ms1-handling`** — watch-only with user-supplied `--ms1` produces `ms1_entropy_match: fail` (was silently passed-vacuously pre-v0.4.5). Behavior change; SPEC clarification pending.

### Breaking changes

JSON envelope `verify-bundle --json` check-array shape — internal-only break per "no users yet" license; no consumers to migrate:

- **Single-sig (template-mode + descriptor-mode + watch-only):** `[ms1_entropy_match, mk1_decode, ..., stub_linkage]` (9 names with stub_linkage) → `[ms1_decode, ms1_entropy_match, mk1_decode, ..., md1_xpub_match]` (9 names per SPEC §5.7).
- **Multisig (template-mode + descriptor-mode):** old per-cell shape (`[ms1_entropy_match, mk1_decode[0..N], mk1_xpub_match[0..N], ..., md1_xpub_match[0..N], stub_linkage[0..N]]`) → SPEC §5.7 3+6N (`[ms1_decode[0], ms1_entropy_match[0], mk1_decode[0], ..., mk1_path_match[N-1], md1_decode, md1_wallet_policy, md1_xpub_match]`).
- **Descriptor-mode plain-text output** also aligned to template-mode format (`{name}: ok|fail {detail}` per check + `result: {result}` trailer; was `verify-bundle: {result}` header + `  - {name} [ok|fail]: {detail}`).

### Wire-bit-identical guarantee

v0.4.0 / v0.4.1 / v0.4.2 / v0.4.3 / v0.4.4 schema-4 `bundle --json` envelopes continue to emit byte-identically. The shape changes are confined to `verify-bundle --json` and `verify-bundle` plain-text output.

### Test corpus

243 lib unit tests + 22 integration suites pass (was 244 lib in v0.4.4; -1 from `helper_multisig_returns_todo_stub` deletion replaced by `helper_multisig_full_emits_3plus6n_checks_in_spec_order`; +3 forensic integration tests).

### Cycle artifacts

- Implementation plan: `design/IMPLEMENTATION_PLAN_v0_4_5_helper_call_sites.md` (r2 APPROVE 0C/0I post-r1 fix).
- Phase reports: `design/agent-reports/phase-P3-helper-wire-up-review-r1.md`, `design/agent-reports/phase-P4-multisig-helper-review-r1.md`, `design/agent-reports/phase-P5-descriptor-mode-helper-review-r1.md`.

### Architect-review history

- v0.4.5 impl plan: 2 in-cycle rounds (r1 BLOCK 2I → 0C/0I r2; multisig check-name bracket notation + shared/per-cosigner grouping corrections inline).
- Phase P.3+P.6: 1 review round (1 Important re: stale `#[allow(dead_code)]` attrs addressed inline; 1 Low re: watch-only spurious --ms1 deferred to FOLLOWUP).
- Phase P.4: 1 review round (1 Critical re: stale doc-comment + 2 nits addressed inline; 2 Important + 1 Low deferred via 3 FOLLOWUPS at v0.4.5-nice-to-have).
- Phase P.5: 1 review round (1 Important re: plain-text format divergence + 1 nit addressed inline).
- Final cross-phase review: APPROVED 2026-05-06 (1 Important re: multisig helper test name/fixture mismatch addressed via rename + FOLLOWUP for full-mode unit coverage; 3 Low/Nit deferred via FOLLOWUPS at v0.4.5-nice-to-have tier).

---

## mnemonic-toolkit [0.4.4] — 2026-05-06

### What's new (v0.4.4 verify-bundle helper foundation + DescriptorBinding cleanup)

v0.4.4 closes the 2 v0.4.4-tier FOLLOWUPS from v0.4.3 deferral. Per the user's "no users yet → ignore migration" license, the DescriptorBinding.entropy field is deleted outright (no shim period). The Phase P scope was reduced from "helper + full ~78-site forensic rollout + descriptor-mode 9/3+6N parity" to "helper foundation only"; call-site rollout (P.3-P.7) deferred to v0.4.5.

- **Phase P.1+P.2 — `emit_verify_checks` helper foundation.** New `#[allow(dead_code)]` helper in `cmd/verify_bundle.rs` with the canonical SPEC §5.7 9-check ordering for single-sig template-mode (ms1_decode, ms1_entropy_match, mk1_decode, mk1_xpub_match, mk1_fingerprint_match, mk1_path_match, md1_decode, md1_wallet_policy, md1_xpub_match). New `SuppliedCards<'a>` struct (`{ms1, mk1, md1}` slice triplet — mk1 indexed by cosigner position with placeholder strings for absent slots; documented). New `emit_md1_checks` shared helper. Multisig path returns a TODO stub: `[VerifyCheck { name: "TODO_multisig_v0_4_5", passed: false, decode_error: Some("multisig helper rollout deferred to v0.4.5") }]`. Watch-only short-circuit: ms1[i].is_empty() → `passed: true + decode_error: Some("skipped: watch-only slot")`. 4 unit tests pin: `helper_singlesig_full_emits_9_checks_in_spec_order`, `helper_singlesig_tampered_mk1_populates_forensics`, `helper_singlesig_watch_only_short_circuits_ms1`, `helper_multisig_returns_todo_stub`. Helper landed but not yet wired to run_full / run_multisig / descriptor_mode_verify_run; that consolidation deferred to v0.4.5 (FOLLOWUP `verify-bundle-helper-call-sites-rollout-v0.4.5`). Closes structural piece of FOLLOWUP `verify-bundle-helper-and-full-forensics-rollout-v0.4.4` (superseded by v0.4.5 successor).
- **Phase S — `DescriptorBinding.entropy` field retired.** Bundle-level `entropy: Option<Vec<u8>>` field deleted from `parse_descriptor.rs::DescriptorBinding`; per-slot entropy lives on `binding.cosigners[i].entropy` (post v0.4.3 N's CosignerKeyInfo→ResolvedSlot type alias merge). New `entropy_at_0()` compatibility shim method returns `Option<&[u8]>` reading `cosigners[0].entropy`. `bind_full_mode` sets `cosigners[0].entropy = Some(entropy)` before constructing the binding. `bind_watch_only_singlesig` and `bind_watch_only_multisig` drop the field initializer. ~10 readers (parse_descriptor.rs tests, cmd/verify_bundle.rs, cmd/bundle.rs::bundle_run_unified_descriptor) migrated from `binding.entropy.as_deref()` / `binding.entropy.is_some()` / `binding.entropy.is_none()` to the helper. Closes FOLLOWUP `descriptor-binding-entropy-field-redundant`.

### Deferred to v0.4.5

- **`verify-bundle-helper-call-sites-rollout-v0.4.5`** — Phase P.3-P.7. Wire `emit_verify_checks` into run_full (P.3), run_multisig (P.4 — replace TODO stub with real 3-shared+6N-per-cosigner emission), descriptor_mode_verify_run (P.5 — closes `verify-bundle-9-3plus6n-descriptor-mode-parity` simultaneously), migrate watch_only_tests (P.6), add forensic-field integration tests (P.7).

### Breaking changes

None at the CLI surface or JSON envelope level. Internal Rust API broke: `DescriptorBinding.entropy: Option<Vec<u8>>` field deleted. Per "no users yet" license — no external Rust consumers to migrate. The `entropy_at_0()` helper method is the new accessor.

### Wire-bit-identical guarantee

v0.4.0 / v0.4.1 / v0.4.2 / v0.4.3 schema-4 bundles continue to emit byte-identically. The `bundle --json` and `verify-bundle --json` envelope shapes are unchanged from v0.4.3. The new `emit_verify_checks` helper is `#[allow(dead_code)]` in v0.4.4 — production code paths still emit the v0.4.3 P.0 shape (passed: bool with forensic fields populated only at the v0.4.1 J.7 proof-of-shape site).

### Test corpus

244 lib unit tests pass (was 240 in v0.4.3; +4 from new emit_verify_checks helper unit tests). Integration suites unchanged.

### Cycle artifacts

- Implementation plan: `design/IMPLEMENTATION_PLAN_v0_4_4_verify_bundle_finish_for_real.md` (r1 APPROVE WITH NITS; 2 LOW findings addressed inline before execution).

### Architect-review history

- v0.4.4 impl plan: 1 in-cycle round (r1 APPROVE WITH NITS — 2 LOW addressed: wif-slot handling clarified; SuppliedCards.mk1 indexing convention documented).
- Phase P.1+P.2: scope-reduced to helper foundation only; 244 tests pass post-helper.
- Phase S: scope-minimized field deletion; 244 tests pass post-migration.
- Final cross-phase review: APPROVED 2026-05-06 (1 Important re: stale CHANGELOG check-names addressed inline; 2 Low/Nit deferred via FOLLOWUP `verify-bundle-helper-foundation-cleanup-v0.4.5`).

---

## mnemonic-toolkit [0.4.3] — 2026-05-06

### What's new (v0.4.3 verify-bundle finish + unified-path edges)

v0.4.3 closes 4 of 5 v0.4.3-tagged FOLLOWUPS plus 1 NEW (`wif-multisig-resolution`). Theme: **finish verify-bundle (struct-shape correction + JSON intake) and close the unified-path edges (binding-type merge + wif multisig)**. Per the user's "no users yet → ignore migration" license, the v0.4.1-introduced VerifyCheck struct drift from SPEC §5.7 is corrected directly.

- **Phase N — `CosignerKeyInfo` → `ResolvedSlot` merge.** Sole binding type is now `ResolvedSlot`; `CosignerKeyInfo` retained as a `#[allow(dead_code)]` type alias for source-compat. Per-slot `entropy: Option<Vec<u8>>` lives on every `ResolvedSlot`. Closes FOLLOWUP `cosigner-keyinfo-resolved-slot-merge`. Bundle-level `DescriptorBinding.entropy` field retained for now (semantically redundant; tracked at NEW v0.4.4 FOLLOWUP `descriptor-binding-entropy-field-redundant`).
- **Phase R — wif slots in multisig contexts.** `resolve_slots` (cmd/bundle.rs) lifted the v0.4.2 single-sig-only guard. Wif slots produce ResolvedSlots with the wif's pubkey + zero chain code + empty path. BIP-388 distinctness applies — same WIF twice → SPEC §6.6 row 13 collision (verified by new test). Closes FOLLOWUP `wif-multisig-resolution`. 3 new integration tests in `cli_unified_slot.rs`: hybrid 2-of-3 (phrase + wif + xpub), pure wif 2-of-2 (two distinct WIFs), same-WIF-twice → row 13.
- **Phase P.0 — VerifyCheck struct shape correction.** Long-standing v0.4.1 J.1 drift from SPEC §5.7: `result: &'static str` ("ok"|"fail"|"skipped") → `passed: bool`. Skipped checks: `passed: true` (decode_error population deferred to v0.4.4 with the helper rollout). Mechanical migration of ~78 push sites in `cmd/verify_bundle.rs` + ~30 test assertions. JSON envelope: `"result": "ok"|"fail"` → `"passed": true|false`.
- **Phase Q — `--bundle-json <file>` verify-bundle JSON intake (SPEC §6.7 amended).** New CLI flag mutually exclusive with `--ms1`/`--mk1`/`--md1` triplet via clap `conflicts_with_all`. Reads a `bundle --json` envelope file, peeks `schema_version`, validates `"4"`, extracts `ms1`/`mk1`/`md1` arrays into a synthetic VerifyBundleArgs, then continues dispatch as if user had supplied the explicit triplet. Re-derivation flags (`--slot`/`--phrase`/etc.) are STILL required for expected-bundle computation. Schema-2/3 envelopes rejected with byte-exact stderr pointing at NEW v0.4.4-nice-to-have FOLLOWUP `bundle-json-schema-2-3-retro-compat`. SPEC §6.7 amended in lockstep with v0.4.3 amendment paragraph. Closes FOLLOWUP `bundle-json-cli-flag-and-dispatch`. 3 new integration tests in `cli_bundle_json_intake.rs` (round-trip, unsupported schema, conflicts_with).

### Deferred to v0.4.4

- **`verify-bundle-helper-and-full-forensics-rollout-v0.4.4`** — full Phase P (P.1 emit_verify_checks helper + P.2-P.5 ~78-site forensic rollout + descriptor-mode 9/3+6N parity refactor). Estimated ~800-1000 lines deleted in verify_bundle.rs. v0.4.3 ships the structural pieces (P.0); the heavy refactor lands in v0.4.4. Bundles `verify-bundle-9-3plus6n-descriptor-mode-parity` from v0.4.2 deferral.
- **`descriptor-binding-entropy-field-redundant`** — retire `DescriptorBinding.entropy` field after v0.4.3 N's per-slot ResolvedSlot.entropy. Cleanup-only; no behavior change.

### Breaking changes

- **JSON envelope `VerifyCheck`**: `"result": "ok"|"fail"|"skipped"` → `"passed": true|false` (skipped: `"passed": true`, `decode_error` population in v0.4.4). Per "no users yet" license — internal-only break; no existing JSON consumers to migrate. SPEC §5.7 was always specified this way; v0.4.1 had implementation drift.

### Wire-bit-identical guarantee

v0.4.0 / v0.4.1 / v0.4.2 schema-4 bundles continue to emit byte-identically. The VerifyCheck struct change affects only `verify-bundle --json` output, not `bundle --json` output.

### Test corpus

240 lib + integration suites pass (was 240 in v0.4.2; net 0 — additions: 3 wif-multisig + 3 bundle-json + struct-shape correction touched ~30 test sites; no test count delta because the v0.4.2 wif-multisig-rejected test was replaced by 3 new wif-multisig-supported tests).

### Cycle artifacts

- Implementation plan: `design/IMPLEMENTATION_PLAN_v0_4_3_verify_bundle_finish.md` (r2 APPROVE WITH NITS; nits applied).
- SPEC: `design/SPEC_mnemonic_toolkit_v0_4.md` §6.7 amended in lockstep with Phase Q.

### Architect-review history

- v0.4.3 impl plan + SPEC: 2 in-cycle rounds (r1 BLOCK 2C/3N → r2 APPROVE WITH NITS 0C/0I/1N; SPEC §6.7 amendment for `--bundle-json` landed before execution).
- Phase N: scope-minimized type alias merge; 240 tests pass post-migration.
- Phase R: scope-minimized guard lift; 3 new tests including BIP-388 collision.
- Phase P.0: SPEC §5.7 drift correction (~78-site mechanical migration); P.1-P.5 deferred to v0.4.4 atomic refactor.
- Phase Q: scope-minimized JSON intake (load + dispatch + 3 tests); helper landed without rewriting run() entry.
- Final cross-phase review: pending (this CHANGELOG entry is the gate).

---

## mnemonic-toolkit [0.4.2] — 2026-05-06

### What's new (v0.4.2 unified-path consolidation)

v0.4.2 closes the v0.4 cycle's "delete the dual-path baggage" theme. Per the user's "no users yet → ignore migration work" license, this release deletes the legacy parallel CLI dispatch path and lands the unified `--slot @N.<subkey>=<value>` path as the sole architectural shape, plus extends slot-input support and removes deprecated test patterns.

- **Phase K — additional slot subkey shapes.** `resolve_slots` (cmd/bundle.rs) now handles `{entropy}` (hex-decode → BIP-39 mnemonic → derive at template path), `{wif}` (degenerate single-key in single-sig contexts), and partial `{xpub}` shapes (`{xpub}` alone, `{xpub, fingerprint}`, `{xpub, path}`). `{xprv}` REJECTED with v0.5+ deferral pointer (FOLLOWUP `unified-slot-xprv-resolution-needs-ms-codec-extension`); `{wif}` in multisig contexts REJECTED with v0.4.3 deferral pointer (FOLLOWUP `wif-multisig-resolution`). Per-shape integration tests in `cli_unified_slot.rs`.
- **Phase L — descriptor mode under unified `--slot`.** `bundle_run_unified_descriptor` resolves each `@i` slot against the per-`@i` annotation path from the parsed descriptor (NOT template's path). Cross-checks fingerprint annotation against phrase-derived master fingerprint. Constructs CosignerKeyInfo bridge + ParsedKey + ParsedFingerprint vecs → existing synthesize_descriptor pipeline. 3 new integration tests.
- **Phase M — legacy flag deprecation (delete parallel dispatch).** `bundle::run` rewritten as a thin ~140-line wrapper holding only the SPEC §6.6 v0.2 + v0.3 mode-violation pre-checks (cli_mode_violations*.rs byte-exact pins). All synthesis and emit goes through `bundle_run_unified` regardless of whether `--slot` or legacy `--phrase` / `--xpub` / `--cosigner` was supplied. New `bundle_args_to_slots` helper folds ALL legacy flags into a unified `Vec<SlotInput>` with the locked cosigner offset rule (phrase present → cosigners @1+; phrase absent → cosigners @0+). Deleted ~990 lines: `bundle_full`, `bundle_watch_only`, `bundle_multisig_full`, `bundle_multisig_watch_only`, `emit`, `emit_multisig`, `descriptor_mode_run`, `descriptor_mode_emit`, `derive_threshold_from_descriptor_tree`, `BundleArgs::template_unchecked`. `emit_unified` text-mode preserves v0.3 UX (ms1-omitted markers, "multisig wallet policy" md1 header, "m/" prefix on origin_path).
- **Phase O — engraving card legacy migration.** Deleted `format.rs::engraving_card` function + `EngravingMode` enum + 3 byte-exact unit tests. Sole engraving card surface is now `engraving_card_unified` (Phase I, v0.4.1). ~140 lines removed.
- **Cleanup — deleted 5 v0.2 multisig-full integration tests.** `cli_account_flag.rs`, `cli_privacy_preserving.rs`, `cli_bundle_multisig_full.rs` (whole-file deletes); 2 `#[ignore]`-marked test functions inside `cli_self_check.rs` and `cli_bundle_multisig.rs` deleted in-place. These exercised the v0.2 self-multisig pattern (BIP-388 violating, no migration path).

### Deferred to v0.4.3

Three v0.4.2 FOLLOWUPS are deferred to v0.4.3 to keep the v0.4.2 release window scope-safe:

- `cosigner-keyinfo-resolved-slot-merge` — Phase N. Retire `CosignerKeyInfo` into `ResolvedSlot`. Cleanup-only; no user-visible behavior change.
- `verify-bundle-emit-checks-helper-and-full-forensics-rollout` — Phase P. `emit_verify_checks` helper + full ~78-site forensic field population + descriptor-mode 9/3+6N parity (FOLLOWUP `verify-bundle-9-3plus6n-descriptor-mode-parity`).
- `bundle-json-cli-flag-and-dispatch` — Phase Q. `--bundle-json <file>` verify-bundle intake + schema-version dispatch.

### Breaking changes

None at the CLI level — legacy `--phrase` / `--xpub` / `--cosigner` flags continue to accept the same inputs (they're parsed and folded into `Vec<SlotInput>` internally). Some byte-exact stderr text shifted as a consequence of the dispatch consolidation:

- `bundle --phrase X --template wsh-sortedmulti --threshold 2 --cosigner-count 3` (no actual cosigners) now emits `error: --cosigner-count deprecated and inconsistent with slot indices (declared N=3, derived N=1)` (SPEC §6.6 row 5) instead of v0.4.0's BIP-388 row-13 hard-reject. The architectural diagnosis is more accurate (no actual cosigners → declared/derived N mismatch).
- `bundle --descriptor 'wsh(sortedmulti(2,@0/...,@1/...))' --phrase X` (descriptor with no cosigner specs) now emits `error: descriptor has n=2 placeholders but --slot vec covers 1 slots` instead of v0.3's "requires explicit [fp/path] origin annotation" — fires earlier in the pipeline.

Both shifts are tracked by updated integration tests pinning the new byte-exact stderr.

Promoted to v0.5: FOLLOWUP `legacy-cli-flag-deletion` covers eventually deleting `--phrase` / `--xpub` / `--cosigner` flags entirely (option (b) from the v0.4.2 brainstorm). v0.4.2 ships option (a): inputs preserved, dispatch unified.

### Wire-bit-identical guarantee

v0.4.0 / v0.4.1 schema-4 bundles continue to emit byte-identically. v0.2 watch-only multisig fixtures pass byte-identically (text-mode, no JSON envelope). v0.2 self-multisig fixtures remain BIP-388-rejected (no integration coverage now since the 5 ignored tests are deleted).

### Test corpus

240 lib unit tests + integration suites pass (was 246 in v0.4.1; net -6 after cleanup: -3 deleted EngravingMode unit tests, -3 deleted v0.2 multisig-full whole-file integration tests, +5 new K + L tests, ~- 5 net via direct delete).

### Cycle artifacts

- Implementation plan: `design/IMPLEMENTATION_PLAN_v0_4_2_unified_consolidation.md` (r2 APPROVE WITH NITS; nits applied).

### Architect-review history

- v0.4.2 impl plan: 2 in-cycle rounds (r1 BLOCK 2C/3I/2N → r2 APPROVE WITH NITS 0C/0I/1N; nits applied inline before execution).
- Phase K: scope-minimized; per-shape integration tests directly validate.
- Phase L: scope-minimized; descriptor-mode integration tests + fingerprint cross-check.
- Phase M: substantive cleanup (~990 lines deleted); test reconciliation surfaced 6 regressions, all closed via 3 emit_unified UX-preserving fixes (ms1 omitted marker, md1 multisig header, "m/" path prefix) + 3 test updates (BIP-388 row-13 → row-5; new explicit row-13 test; descriptor missing-annotation → slot-count-gap).
- Phase O: trivial deletion; 240 tests pass after.
- Final cross-phase review: pending (this CHANGELOG entry is the gate).

---

## mnemonic-toolkit [0.4.1] — 2026-05-05

### What's new (v0.4.1 schema-4 cutover + multi-source synthesis + foundations for unified card and forensics)

v0.4.1 lands the three v0.4.0 deferrals:

- **`bundle-json-schema-4-cutover` (Phase H, complete).** `Bundle.ms1` and `BundleJson.ms1` migrate from `Option<String>` to `MsField` (= `Vec<String>`). `schema_version` bumps `"3"` → `"4"`. All 5 producers + 4 emit sites updated. SPEC §5.8 dense-with-empty-string-sentinel layout: single-sig watch-only is `[""]`; pure watch-only multisig N=3 is `["", "", ""]`; multi-source full N=3 is `["ms1...", "ms1...", "ms1..."]`; hybrid is mixed. `mode_str` derivation switches to `bundle.any_secret_bearing()`.
- **Multi-source synthesis (Phase H).** `synthesize_unified(slots, template, threshold, network, privacy)` is the new universal synthesis entry handling all five `BundleMode` variants (SingleSigFull / SingleSigWatchOnly / MultisigMultiSource / MultisigWatchOnly / MultisigHybrid). `ResolvedSlot` carries per-slot xpub + fingerprint + path + path_raw + optional entropy.
- **`bundle::run` unified dispatch (Phase H).** When `--slot @N.<subkey>=<value>` is supplied, `bundle::run` routes through `bundle_run_unified`: `expand_legacy_to_slots → validate_slot_set → detect_bundle_mode → resolve_slots → check_resolved_slots_distinctness → synthesize_unified → emit_unified`. Legacy `--phrase` / `--xpub` / `--cosigner` retain v0.3 dispatch (full deprecation deferred to v0.5+).
- **BIP-388 raw-string path normalization (Phase H.6).** `check_key_vector_distinctness` switches to raw-string `(xpub.to_string(), path_raw)` equality per SPEC §4.11.b literal text. `CosignerKeyInfo` and `ResolvedSlot` both carry `path_raw: String`. Legacy descriptor-placeholder paths preserve the parser's canonical `'`-form; `--slot @N.path=<value>` preserves the user's literal byte sequence end-to-end (so `48h/0h` and `48'/0'` compare unequal under raw-string equality on the slot path).
- **Unified engraving card foundation (Phase I, additive).** `BundleInputForCard` struct + `engraving_card_unified` function per SPEC §5.5. Wired into `bundle_run_unified`'s emit_unified path. The 4 legacy `engraving_card(...)` call sites retain v0.3 behavior (full migration deferred to v0.4.2 per FOLLOWUP `engraving-card-unified-legacy-migration`). Card layout: header / threshold / cosigners block / template OR descriptor (truncation at 80 chars) / md1 reference / recovery hint / language+passphrase footer / hardware caveat for tap-multisig.
- **Verify-bundle forensic-field foundation (Phase J, additive).** `VerifyCheck` gains 4 forensic fields (`expected`, `actual`, `diff_byte_offset`, `decode_error`) per SPEC §5.7, with `#[serde(skip_serializing_if = "Option::is_none")]` so JSON envelopes stay clean for "ok"/"skipped" checks. `VerifyCheck::diff_offset(a, b)` helper. Per-cell forensic field POPULATION is wired at one proof-of-shape site (descriptor-mode `ms1_entropy_match` mismatch); full ~78-site rollout deferred to v0.4.2 alongside the `emit_verify_checks` helper refactor (FOLLOWUP `verify-bundle-emit-checks-helper-and-full-forensics-rollout`).
- **`--ms1` CLI repeating-flag migration (Phase J.5).** `VerifyBundleArgs.ms1: Option<String>` → `Vec<String>` with `ArgAction::Append`. Existing single-value invocations continue to work (clap accepts the single occurrence as a 1-element vec). Multi-source schema-4 verification supplies `--ms1` per slot (`--ms1 "" --ms1 <s>` for hybrid-shaped vectors).

### Deferred to v0.4.2

The following SPEC-mandated v0.4 deliverables are deferred to v0.4.2 to preserve v0.4.1 release-window scope-safety. See `design/FOLLOWUPS.md` entries at tier `v0.4.2`:

- `unified-slot-additional-subkey-shapes` — `entropy` / `xprv` / `wif` / partial-xpub-only resolution under `--slot` (v0.4.1 supports `{phrase}` and `{xpub, fingerprint, path}` shapes).
- `unified-slot-descriptor-mode-support` — descriptor mode under unified `--slot` dispatch.
- `bundle-json-cli-flag-and-dispatch` — `--bundle-json <file>` verify-bundle JSON intake + schema-version dispatch (Phase J.4).
- `cosigner-keyinfo-resolved-slot-merge` — retire `CosignerKeyInfo` into `ResolvedSlot`.
- `engraving-card-unified-legacy-migration` — migrate the 4 legacy `engraving_card()` call sites (Phase I migration tail).
- `verify-bundle-emit-checks-helper-and-full-forensics-rollout` — Phase J.2 + J.3 + ~78-site forensic field population.
- `verify-bundle-9-3plus6n-descriptor-mode-parity` — descriptor-mode 9/3+6N parity (depends on the helper).

### Versioning rationale

v0.4.1 is a patch bump (not a 0.5.0 minor bump) under the framing established in v0.4.0's CHANGELOG: v0.4.0 explicitly deferred these breaking changes "to v0.4.1" with full FOLLOWUPS pointers, designating the v0.4 cycle as the breaking-change unit landing in two releases (v0.4.0 ships the BIP-388 enforcement + CLI surface foundation; v0.4.1 completes the schema-4 wire migration + multi-source synthesis + foundations for the unified card and forensics). Consumers reading either v0.4.x release's CHANGELOG are explicitly warned of the schema-4 cutover. Per the repo's pre-1.0 SemVer convention, the breaking changes WOULD justify 0.5.0; the deliberate choice to land them within 0.4.x is an internal-cycle accounting decision documented at v0.4.0.

### Breaking changes

- **`BundleJson.schema_version`** bumps `"3"` → `"4"` for all bundles emitted by v0.4.1. Consumers that assert `schema_version == "3"` will break; update to `"4"` or to schema-aware dispatch.
- **`BundleJson.ms1`** type changes from `string | null` to `array<string>`. Consumers that read `.ms1` as a string break. Migration: read `.ms1` as an array; for single-sig full, use `.ms1[0]`; for watch-only, the array contains an empty-string sentinel `[""]`.
- **`Bundle.ms1`** (Rust API) type changes from `Option<String>` to `Vec<String>`. Direct consumers of the toolkit's library API need to update their pattern matching.
- **`VerifyBundleArgs.ms1`** (CLI flag) accepts `--ms1` multiple times (`Vec<String>`). Single `--ms1 <s>` invocations continue to work as 1-element vec. **Note for multi-slot verification:** v0.4.1's verify-bundle path compares only the FIRST `--ms1` value against the bundle's slot 0; full per-slot multi-source verification (all elements of `--ms1` checked against all slots) is deferred to v0.4.2 alongside `--bundle-json` intake (FOLLOWUP `bundle-json-cli-flag-and-dispatch`).
- **BIP-388 raw-string path equality** for `--slot @N.path=` paths preserves the user's literal byte sequence; `48h/0h` and `48'/0'` are now treated as distinct paths under the slot-driven path. Legacy descriptor paths continue to use the parser's canonical form.

### Wire-bit-identical guarantee

v0.4.0 v0.2/v0.3 single-sig + watch-only multisig fixtures continue to pass byte-identically (text-mode output for these cases is unchanged; only the JSON envelope shape changes). The 5 v0.2 self-multisig integration tests remain `#[ignore]`d per BIP-388 hard-reject (introduced in v0.4.0).

### Test corpus

246 lib unit tests + integration suites pass (was 227 in v0.4.0; +19). New tests added in v0.4.1:
- 2 BIP-388 raw-string distinctness unit tests.
- 7 `synthesize_unified` shape tests (each BundleMode + threshold-out-of-range + schema-version pin).
- 4 unified `--slot` CLI integration tests (happy path + missing-template/descriptor + unsupported-subkey-shape + row-6 conflict).
- 6 unified engraving card unit tests (single-sig full / watch-only / multisig / privacy-preserving / descriptor truncation / tap caveat).
- 4 VerifyCheck forensic field unit tests.

### Cycle artifacts

- Implementation plan: `design/IMPLEMENTATION_PLAN_v0_4_1_cutover.md` (r2 APPROVE WITH NITS; nits applied).
- Per-phase reviews: `design/agent-reports/phase-H-schema-4-cutover-review-r1.md` (r1 BLOCK 0C/2I/1L → r2 APPROVE 0C/0I/0L).

### Architect-review history

- v0.4.1 impl plan: 2 in-cycle rounds (r1 BLOCK 3C/2I → r2 APPROVE WITH NITS 0C/0I/2N + nits applied inline).
- Phase H: 2 rounds (r1 BLOCK 0C/2I/1L → r2 APPROVE 0C/0I/0L).
- Phase I: scope-minimized to additive only; format.rs unit tests (6) directly cover the new function; per-phase review skipped.
- Phase J: scope-minimized to additive only (J.1 + J.5 + one J.7 proof-of-shape); format.rs unit tests (4) directly cover the new VerifyCheck behavior; per-phase review skipped.
- Final cross-phase review: pending (this CHANGELOG entry).

---

## mnemonic-toolkit [0.4.0] — 2026-05-05

### What's new (v0.4.0 foundation release)

v0.4.0 is the foundation release for the v0.4 cycle. It ships:

- **BIP-388 distinct-key conformance (SPEC §4.11).** The toolkit now hard-rejects any descriptor binding whose `@N` slots resolve to identical `(xpub, derivation_path)` tuples. Symmetric across bundle creation (exit 2 + SPEC §6.6 row 13 byte-exact stderr) and verify-bundle (exit 4 + SPEC §4.11.c stderr). The legacy `bundle multisig-full --cosigner-count > 1` self-multisig path now hard-rejects at the entry point — all v0.2 self-multisig fixtures are excluded from the byte-identical regression matrix per SPEC §10 and the affected integration tests are marked `#[ignore = "deprecated v0.2 pattern; remove after v0.4 release"]`.
- **`--slot @N.<subkey>=<value>` CLI surface (SPEC §6.6.b).** New repeating clap flag with closed subkey vocabulary `phrase | entropy | xpub | fingerprint | path | wif | xprv`. Includes `parse_slot_input` value-parser (SPIKE-2 locked grammar; empty value rejected at parser), `validate_slot_set` (per-slot validity matrix + contiguity check), and `expand_legacy_to_slots` for SPEC §6.6.a deprecation alias mapping.
- **`bundle multisig-full` / `bundle multisig-watch-only` removed-subcommand trap (SPEC §6.6 row 1).** Pre-clap argv inspection emits the byte-exact migration error before clap parses. Two CLI integration tests assert byte-exact stderr from a live binary.
- **`BundleMode` mode-detection foundation (impl plan Phase C.3).** `detect_bundle_mode(slots)` classifier + `pre_check_threshold` / `pre_check_template_n` helpers (SPEC §6.6 rows 9, 9.5, 10, 11). Wired in v0.4.1 follow-on per `bundle-json-schema-4-cutover`.
- **`MsField = Vec<String>` type alias (SPEC §5.8).** Foundation for the schema-4 ms1 dense layout. Live wire-up deferred to v0.4.1.
- **Multi-leaf taproot walker (SPEC §4.9.a).** `walk_tap_tree` generalizes v0.3's single-leaf-only walker via depth-stack folding of miniscript's flat DFS-preorder leaf list. Algorithm transcribed verbatim from Phase 2 SPIKE-1 deliverable. Validated against 6 round-trip probe shapes (1/2/3/4-leaf incl. asymmetric and right-spine) at SPIKE time and 4 in-tree unit tests.

### Out of scope (deferred to v0.4.1)

The following SPEC §9 v0.4 deliverables are deferred to a v0.4.1 follow-on patch to keep the v0.4.0 release scope-safe under autonomous execution. See `design/FOLLOWUPS.md` entries at tier `v0.4.1`:

- **`bundle-json-schema-4-cutover`** — full `BundleJson.ms1: Option<String>` → `MsField` migration + `schema_version: "3" → "4"` bump + verify-bundle schema-4 dispatch + integration test JSON assertion updates + fixture envelope regeneration. v0.4.0 retains the schema-3 envelope; multi-source synthesis primitives sit ready in `format.rs` + `bundle_unified.rs` for v0.4.1 wire-up.
- **`engraving-card-unified-1-master-card`** — Phase E unified `BundleInputForCard` + `engraving_card_unified` per SPEC §5.5. Tightly coupled to schema-4 cutover.
- **`verify-bundle-9-3plus6n-forensics`** — Phase G descriptor-mode parity to template-mode 9 / 3+6N check ladder + per-cell forensic `VerifyCheck` fields per SPEC §5.7.

### Breaking changes

- **`bundle multisig-full --cosigner-count > 1`** hard-rejects (exit 2 + SPEC §6.6 row 13 stderr) per BIP-388 distinct-key rule. The legacy v0.2 self-multisig pattern is no longer producible. Migration: use `--cosigner` triples for watch-only multisig (still works), or wait for v0.4.1's multi-source synthesis (N distinct seeds → N (ms1, mk1) pairs).

### Wire-bit-identical guarantee

v0.2 single-sig + multisig-watch-only fixtures continue to pass byte-identically. v0.2 self-multisig fixtures (33 cells under `wsh-multi`/`sortedmulti`, `sh-wsh-multi`/`sortedmulti`, `tr-multi-a`/`sortedmulti-a` × 4 networks; plus 0/5/0-true variants of `wsh-sortedmulti`) are EXCLUDED from the byte-identical regression matrix per BIP-388 violation. v0.3 fixtures continue to pass byte-identically.

### Test corpus

227 lib unit tests + integration test suites pass; 5 v0.2 multisig-full integration tests are `#[ignore]`d per SPEC §10 fixture exclusions. Tests added in v0.4.0:
- 7 BIP-388 distinct-key unit tests (`parse_descriptor::tests::bip388_*`).
- 1 BIP-388 byte-exact CLI stderr integration test (`cli_bip388_distinctness`).
- 34 slot-input parser/validator/alias-expander unit tests (`slot_input::tests`).
- 24 bundle_unified mode-detection + pre-check + trap unit tests.
- 2 removed-subcommand trap CLI integration tests.
- 4 multi-leaf taproot walker unit tests.

### Cycle artifacts

- SPEC: `design/SPEC_mnemonic_toolkit_v0_4.md` (309 lines; delta over v0.3 SPEC).
- Implementation plan: `design/IMPLEMENTATION_PLAN_v0_4_unified_cli.md` (217 lines; 7 phases A-G + pre-Phase-A SPIKE).
- SPIKE deliverable: `design/agent-reports/spike-toolkit-v0_4-pre-phaseA.md` (architect-cleared at r2 0C/0I).
- Phase reviews: `design/agent-reports/phase-A-bip388-conformance-review-r1.md` (APPROVE WITH NITS), `phase-B-slot-input-review-r1.md` (APPROVE), `phase-C-bundle-unified-review-r1.md` (APPROVE WITH NITS).

### Architect-review history

- Brainstorm convergence: 6 plan-mode rounds (r1 0C/1I/4L → r6 0C/0I/2L APPROVE).
- SPEC + implementation plan: 2 rounds in-toolkit-repo (r2 APPROVE).
- Phase 2 SPIKE: 2 rounds (r2 0C/0I).
- Per-phase: A r1 APPROVE WITH NITS (1L+2N), B r1 APPROVE (0L+2N — L-1 fixed inline + 1 fix during r1 round), C r1 APPROVE WITH NITS (1L+3N), F skipped review (algorithm SPIKE-cleared, in-tree tests are direct SPIKE transcription), D/E/G — explicit deferral to v0.4.1 documented in FOLLOWUPS.

---

## mnemonic-toolkit [0.3.1] — 2026-05-05

### What's new

- `tr(K, sortedmulti_a(...))` in tap-leaves now parses and emits valid bundles. Closes the v0.3.0 deferral — rust-miniscript v13.0.0 had no parser for `sortedmulti_a`, but PR #910 ("Add support for sortedmulti_a") merged 2026-04-03 added it, and PR #915 (2026-04-04) refactored `SortedMultiVec` away.

### Mechanism

- Workspace `Cargo.toml` adds `[patch.crates-io] miniscript = { git = "https://github.com/rust-bitcoin/rust-miniscript", rev = "95fdd1c5773bd918c574d2225787973f63e16a66" }` — pinned to rust-miniscript master post-#910 + #915.
- `parse_descriptor.rs` walker refactored for the post-#915 API: `WshInner` enum removed (Wsh wraps Miniscript directly via `as_inner()`); `ShInner::SortedMulti` removed; `Terminal::SortedMulti` + `Terminal::SortedMultiA` arms added in `walk_miniscript_node`. Wire output unchanged for the `wsh(sortedmulti(...))` path; new `Tag::SortedMultiA` path added for tap-leaf `sortedmulti_a`.
- SPEC `design/SPEC_mnemonic_toolkit_v0_3.md` §4.9.a Layer 1 + Layer 2 patched in lockstep; revision Round 8.

### Future cleanup (v0.3.2)

When a miniscript crates.io release publishes containing PR #910 + #915, v0.3.2 drops the `[patch]` entry and bumps the version. Mechanical; no API or feature changes. Tracked in FOLLOWUP `tr-sortedmulti-a-via-upstream` (tier `v0.3.2`).

### Wire-bit-identical guarantee

v0.2 + v0.3.0 fixture matrices continue to validate byte-identically. New regression test confirms descriptor-mode `tr(@0, sortedmulti_a(2, @0, @1))` produces md1 byte-identical to template-mode `--template tr-sortedmulti-a` for matching keys/cosigners (`descriptor_tr_sortedmulti_a_matches_template_tr_sortedmulti_a_md1` in `parse_descriptor::tests`). This is the strongest correctness signal: the new walker arm produces the same `Tag::SortedMultiA` tree the template encoder has been producing since v0.3.0.

### Test corpus

159 unit tests + 2 ignored (was 156 + 2 in v0.3.0; +3 sortedmulti_a tests: `arm_sorted_multi_via_wsh` regression for the post-#915 `Terminal::SortedMulti` Layer-2 routing, `arm_sorted_multi_a_via_tap` for the v0.3.1 unblock target, `descriptor_tr_sortedmulti_a_matches_template_tr_sortedmulti_a_md1` for wire-bit-identical equivalence). Integration test count unchanged.

### Out of scope (still v0.4)

- Multi-leaf taproot trees (`tr(K, {A,B})` with N≥2 leaves).
- Engraving card in descriptor mode.
- Full 9 / 3+6N descriptor-aware verify-bundle check ladder (v0.3.x ships 3-element direct byte-equality ladder).
- `walker-backport-to-md-cli` — md-cli still rejects all v0.3-NEW miniscript fragments AND `sortedmulti_a` post-v0.3.1; cross-repo coordination cycle pending.

### Architect-review history

- Sketch r1: 0C / 3I / 4L → 5 action items folded into formal plan.
- Formal plan r2: 0C / 1I / 2L → 3 doc-fixes folded inline.
- End-of-phase r3: see `design/agent-reports/v0_3_1-end-of-phase-review-r1.md`.

---

## mnemonic-toolkit [0.3.0] — 2026-05-05

### What's new

- **`--descriptor "<string>"` and `--descriptor-file <path>`** flags accept any BIP-388 descriptor whose miniscript AST is supported by the v0.3 walker. Toolkit synthesizes md1 + mk1 + ms1 bundles for any combination of full / watch-only × single-sig / multisig modes detected from the descriptor's `@N` placeholder count (n=1 → single-sig regardless of outer wrapper; n≥2 → multisig).
- **Walker covers the BIP-388 surface:** all v0.2 wrappers (`wpkh`, `pkh`, `wsh+(Ms|SortedMulti)`, `sh+(Wpkh|Wsh|Ms|SortedMulti)`, `tr` keypath + single-leaf miniscript), plus 23 v0.3-NEW miniscript fragments — hash terminals (`sha256`, `hash256`, `hash160`, `ripemd160`), timelocks (`after`, `older`), wrappers (`v:`, `s:`, `a:`, `j:`, `n:`, `c:`), boolean ops (`and_v`, `and_b`, `andor`, `or_b`, `or_c`, `or_d`, `or_i`), and `thresh()`.
- **`@N[fp/path]/<multipath>/*` annotation syntax.** Full-mode `@0` requires the `[fp/path]` annotation; toolkit derives the xpub at the annotated path and cross-checks the fingerprint against the seed-derived master fp. Multi-cosigner `@N≥1` annotations are cross-checked against `--cosigner` triples.
- **`verify-bundle --descriptor`** mirror of the bundle path. Re-runs the descriptor pipeline, builds the expected ms1/mk1/md1, and compares byte-equality to the supplied cards. New `DescriptorReparseFailed` error variant (exit 4) for re-parse failures.
- **`SELF-MULTISIG WARNING`** detection extended to descriptor mode (fires when full-mode multisig descriptor has any cosigner xpub equal to the seed-derived `@0` xpub).
- **Bundle JSON schema bumped to `"3"`.** `template` field becomes nullable; new top-level `descriptor` field carries the user-supplied descriptor verbatim. Both fields ALWAYS emit (`null` when not set).

### Breaking changes (callers)

- `BundleArgs::template`: `CliTemplate` → `Option<CliTemplate>`. Clap attr `required_unless_present_any = ["descriptor", "descriptor_file"]`. Same change applied to `VerifyBundleArgs::template`.
- `BundleJson::template`: `&'static str` → `Option<&'static str>`. New `descriptor: Option<String>` field.
- `VerifyBundleJson::schema_version` and `BundleJson::schema_version`: `"2"` → `"3"`.

### Wire-bit-identical guarantee

Encoded card strings (ms1 / mk1 / md1) for any v0.2 invocation under the v0.3 binary remain byte-identical. Only the JSON envelope differs: `schema_version "2"→"3"` and a new `"descriptor": null` field appears. The v0.2 fixture corpus is preserved verbatim and continues to validate.

For descriptor-mode invocations that exactly express a v0.2 template (canonical `[fp/path]` annotation matching the BIP-44/49/84/86 paths), the resulting md1 is byte-identical to template-mode emission. Three regression tests confirm this for bip44 / bip84 / bip86 (`descriptor_bipXX_matches_template_bipXX_md1` in `parse_descriptor::tests`).

### Out of scope (deferred to v0.4)

- `tr(@0, sortedmulti_a(...))` — rust-miniscript v13.0.0 has no parser for `sortedmulti_a` in tap-leaves. Tracked in `design/FOLLOWUPS.md` (`tr-sortedmulti-a-via-upstream`); v0.4 gates on upstream parser support.
- Multi-leaf taproot trees (`tr(K, {A,B})` with N≥2 leaves). Deferred per SPEC §6.8 (Merkle-root logic).
- Engraving card in descriptor mode. Existing card builder is template-coupled; v0.4 will add a descriptor-aware card. Tracked in FOLLOWUPS (`descriptor-mode-engraving-card`).
- Full v0.4-style 9 / 3+6N descriptor-aware verify-bundle check ladder. v0.3 ships a 3-element direct-byte-equality ladder (ms1_match, mk1_match, md1_match). Functional but coarser than template-mode's 9-check schema.
- `RawPkH` and `DupIf` `Terminal` arms — descriptor-unreachable in rust-miniscript v13.0.0 (RawPkH only via raw script decode; DupIf type-restrictive). Walker handles them for completeness; tests `#[ignore]`.

### Test corpus

156 unit tests + 9 v0.3 mode-violation integration tests + all v0.2 integration tests (cli_bundle_*, cli_verify_bundle_*, cli_mode_violations_v0_2, cli_json_envelopes, etc.) green; v0.2 fixture matrix continues to pass byte-identically.

### Reproduction

Build: `cargo build --release`. Test: `cargo test --package mnemonic-toolkit`.

The v0.3 SPEC at `design/SPEC_mnemonic_toolkit_v0_3.md` (rounds 1-7, architect-reviewed 0C/0I) is normative for all descriptor-mode behavior. The implementation plan at `design/IMPLEMENTATION_PLAN_v0_3_descriptor_passthrough.md` records phase-by-phase architect-review verdicts (mid-phase + end-of-phase per phase, all addressed to 0C/0I).

---

## mnemonic-toolkit [0.2.0] — 2026-05-05

### What's new

- **Multisig templates (6 BIP-388 wrappers):** `wsh-multi`, `wsh-sortedmulti`, `sh-wsh-multi`, `sh-wsh-sortedmulti`, `tr-multi-a`, `tr-sortedmulti-a`. Threshold `1 ≤ K ≤ N ≤ 16`.
- **`--account <u32>`:** non-zero account index threading; replaces v0.1's hardcoded `account=0`.
- **`--xpub-input` multisig (watch-only):** `--cosigner <xpub>:<fp>:<path>` (repeatable) + `--cosigners-file <path>` for bulk JSON ingestion. Per-cosigner path overrides supported; `--multisig-path-family {bip48,bip87}` selects the global default (default `bip87`).
- **`--privacy-preserving`:** whole-bundle privacy boolean. Suppresses `master_fingerprint` from mk1 origins (multisig only); single-sig watch-only with `--xpub` rejects the flag (would produce inconsistent bundle vs. md1's `tlv.fingerprints`).
- **`--self-check`:** post-emit synthesize-then-verify pass on the bundle just produced. Catches synthesis/verify drift before the user engraves.

### Wire-bit-identical guarantee

Encoded card strings (ms1 / mk1 / md1) are byte-identical to v0.1's output for any v0.1-equivalent invocation (single-sig, account=0, no `--privacy-preserving`, no `--self-check`). v0.1 decoders consuming v0.2-emitted encoded strings work unchanged. The 16-cell v0.1 fixture corpus at `tests/vectors/v0_1/` is preserved verbatim and gated by `cli_bundle_full.rs` as a regression set; SHA-256 pin `81828299c927783d915108f32c9752b3dbf815c1caba5b6f6e7ce7b810ddcbf6` continues to hold for that subdirectory.

### JSON envelope evolution

- `schema_version` bumps `"1"` → `"2"`.
- New `bundle` fields: `multisig` (discriminated-union: `null` for single-sig; `{ k, n, template, path_family, cosigners: [...] }` for multisig), `privacy_preserving` (bool), `origin_paths` (per-cosigner path list when divergent from family default).
- `mk1` field becomes a `oneOf` shape: flat object for single-sig, array of N grouped chunk-set objects for multisig.

### v0.1 SHA pin retired; v0.2 SHA pin

The v0.1 fixture pin (`81828299...`) is retired as the active regression baseline (it remains as the `tests/vectors/v0_1/` byte-identity check). The v0.2 corpus adds 34 new multisig + axis cells under `tests/vectors/v0_2/`. Reproduction command (resolves v0.1 FOLLOWUPS N-1, the missing SHA-reproduction recipe):

```bash
shasum -a 256 crates/mnemonic-toolkit/tests/vectors/v0_2/*.txt | sort | shasum -a 256
# a381761656fd165e8e5af28574a5baaa55973e562c610254ae6f31d6b1f06171
```

### Tests

76 unit + 31 integration test functions = 107 total (`cargo test --workspace`). The 31 integration functions cover ~54 parametric cells across 13 test binaries. New v0.2 integration tests:
- `cli_bundle_multisig_full.rs` — 24-cell multisig fixture parametric (6 templates × 4 networks).
- `cli_account_flag.rs` — 4-cell `--account 5` parametric.
- `cli_privacy_preserving.rs` — 4-cell `--privacy-preserving` parametric.
- `cli_self_check.rs` — 2 happy-path self-check fixtures (single-sig + multisig).
- `cli_mode_violations_v0_2.rs` — 7 v0.2 NEW SPEC §6.6 mode-violation rows (byte-exact text + exit-2 contract).

### Known limitations (v0.3+ deferred)

- K-of-N share encoding (split mk1 / split ms1 / split md1) deferred — ms1 first per BIP-93.
- `--cosigners-file` user-supplied file output / multi-file output deferred.
- Hash-locks / timelocks / advanced descriptor variants deferred.
- `cargo publish` of the toolkit still gated on `ms-codec` / `mk-codec` / `md-codec` reaching crates.io. v0.2.0 distributed via GitHub tag `mnemonic-toolkit-v0.2.0`.

### Wire-format SHA pin

```text
sha256(crates/mnemonic-toolkit/tests/vectors/v0_2/) = a381761656fd165e8e5af28574a5baaa55973e562c610254ae6f31d6b1f06171
```

## mnemonic-toolkit [0.1.0] — 2026-05-04

### What's new

- Initial release. Top-level integration crate of the m-format star.
- 2 subcommands: `bundle` (encode-side: emit 3-card engraving bundle) and `verify-bundle` (round-trip integrity check).
- 2 input modes per command: full (`--phrase`) and watch-only / key-only (`--xpub --master-fingerprint`).
- 4 single-sig wallet templates: BIP-44 (pkh), BIP-49 (sh-wpkh), BIP-84 (wpkh), BIP-86 (tr).
- 4 networks: mainnet / testnet / signet / regtest.
- Account hardcoded `0` in v0.1; `--account` flag deferred to v0.2.
- All 10 BIP-39 wordlists supported via `--language`.
- Multi-section stdout (`# ms1` / `# mk1` / `# md1` headers + chunked engraving form).
- Byte-exact engraving-card stderr per SPEC §5.2.
- `--json` envelope schemas for both subcommands.
- Exit codes 0 / 1 / 2 / 3 / 4 / 64 per SPEC §6.
- Byte-deterministic mk1 `chunk_set_id` derived from the 4-byte `policy_id_stub` (mirrors md-codec's deterministic CSI derivation), so toolkit output is byte-reproducible across runs and the SHA-pinned regression corpus is meaningful.

### Tests

17 integration tests (assert_cmd) + 54 unit tests. Trezor 24-word zero-entropy vector pinned across 16 (template × network) cells.

### Known limitations

- Multisig templates, non-zero account, file output, recovery flow: deferred to v0.2+.
- `cargo publish` blocked until ms-codec / mk-codec / md-codec hit crates.io. v0.1.0 distributed via GitHub tag `mnemonic-toolkit-v0.1.0`.

### Wire-format SHA pin

The 16 fixture files at `crates/mnemonic-toolkit/tests/vectors/v0_1/*.txt` are SHA-256-pinned at this release. Subsequent corpus changes that alter the SHA require a SemVer minor bump per the pre-1.0 breaking-change-axis convention.

```text
sha256(crates/mnemonic-toolkit/tests/vectors/v0_1/) = 81828299c927783d915108f32c9752b3dbf815c1caba5b6f6e7ce7b810ddcbf6
```
