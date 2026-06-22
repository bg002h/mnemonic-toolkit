# R0 REVIEW (Round 1) — `PLAN_w1_export_refusals_and_reconcile.md` (toolkit v0.70.1)

**Reviewer:** opus architect (independent R0). **Date:** 2026-06-22. **Source SHA:** `origin/master` = `1cea85ea`.
**Persisted verbatim before the fold-and-commit step, per project discipline.**

**VERDICT: 0 Critical / 2 Important / 5 Minor**

The plan is fundamentally sound. The central correctness claim (structural `tap_tree()` discriminator beats the substring draft) is **empirically proven** against the pinned miniscript rev. The two Important findings are a factually-wrong justification in item #4 and an unproven protocol premise behind Q-A that must be reframed before coding. No Critical defects; the funds-adjacent item #2 design is correct.

---

## Critical findings
None.

---

## Important findings

### I-1 — Item #4's DEFER justification is factually WRONG (md-codec DOES expose a cheap per-chunk validator); fix the rationale before amending the FOLLOWUP
The plan justifies DEFER partly on: *"A correct per-token validator needs md-codec to expose chunk-level syntactic+checksum validation it does not currently surface → sibling md-codec scope creep"* (plan §3, line 100). This is **false**. Verified in `descriptor-mnemonic/crates/md-codec/src/bch.rs`:
- `pub fn bch_verify_regular(hrp, data_with_checksum) -> bool` (bch.rs:89) — a public, chunk-level checksum validator that does NOT require a complete descriptor.
- `pub fn polymod_run` (bch.rs:53), `pub fn hrp_expand` (bch.rs:62), `pub const MD_REGULAR_CONST` (bch.rs:17), `pub const GEN_REGULAR` (bch.rs:7) are all public and pinned-public (`tests/bch_visibility_pin.rs`).
- The toolkit **already imports** `md_codec::bch::MD_REGULAR_CONST` (`repair.rs:46`) and has its own per-chunk residue check (`repair.rs:652-692`).

**The bech32-trap itself is TRUE and correct** (md1 is a custom BCH(93,80,8) code: 13-symbol checksum, 5×65-bit generator, target residue `MD_REGULAR_CONST = 0x0815c07747a3392e7` — categorically incompatible with bech32 residue `1` / bech32m `0x2bc830a3`; `bitcoin::bech32::decode("md1…")` fails on every valid md1 string). So the recon-draft's "all real md1 chunks pass bech32 decode" claim is indeed false.

**Required fold:** Keep DEFER (it is the right disposition), but rewrite the justification. The valid reason is: (a) the bech32 approach in the slug's "What" (`bech32::decode(t).is_ok()`) is *fundamentally wrong* for md1's custom BCH code, AND (b) the **current intake is already loud-and-safe** — `descriptor_intake.rs:156` routes by case-insensitive `"md1"` prefix and `:224` hands off to `md_codec::chunk::reassemble`, which yields a typed `ToolkitError::MdCodec` (exit 1) on checksum failure — **no live defect exists to fix**. Do NOT claim "scope creep / md-codec exposes only whole-descriptor decode." The trap-documentation amendment to the FOLLOWUP must record the *correct* primitive (`bch_verify_regular`) for any future attempt, not steer toward "needs a sibling change." (Also note: the plan/slug should cite the *whole-descriptor* `decode_md1_string` at decode.rs:86 vs. the *multi-chunk* `reassemble` at chunk.rs:306 — md1 is multi-chunk up to 64 chunks, so a single token is generally not a complete descriptor, which is the deeper reason a naive per-token decode is wrong.)

### I-2 — Q-A rests on an UNVERIFIED premise: there is no evidence Green's file import accepts a taproot keypath descriptor. Reframe the bip86→green test as behavior-pinning, not correctness, and file a verify-FOLLOWUP
The plan reasons (§1 lines 33-34, Q-A): *"singlesig-only" means single-sig (incl. BIP86 keypath) is supported → discriminate, don't blanket-refuse.* I verified this premise is **not grounded**:
- The restore-side and manual uses of "singlesig-only" are stated in contrast to **multisig** only: `45-foreign-formats.md:844-845` (*"singlesig-only, because Blockstream Green's multisig is server-mediated"*) and `41-mnemonic.md:1221-1225`. Neither asserts Green imports a `tr(KEY)` file. The manual's Green example is `wpkh(...)` (bip84).
- There is **no test** for bip86→green anywhere (`tests/cli_export_wallet_green.rs` covers bip84 singlesig + multisig refusal only). The bip86→green emission today is an **untested structural fall-through** (`P2tr` is not in `is_multisig()`'s set → falls to `Ok(...)` at green.rs:41).
- Authoritative web check: Blockstream's Help Center "Set up watch-only wallet" page is **silent** on whether descriptor *file* import accepts taproot keypath; Green supports taproot wallets in-app, but file-import of a `tr(KEY)` descriptor is unconfirmed.

This does NOT change the disposition — **"discriminate" is still the correct, conservative choice** (the reported bug is the mislabeled *tap-script-tree policy*; both discriminate and blanket-refuse fix it; blanket-refuse would additionally *remove* an existing emission with no positive justification, which the project convention disfavors). But the plan currently presents the Green-accepts-keypath premise as established fact.

**Required fold:** (1) Reword the test `cell_7_green_bip86_keypath_still_emits` and its plan rationale from a *correctness* guard ("Green can import this") to a **behavior-pinning, no-regression** guard ("the fix does not blanket-refuse P2tr; bip86 keypath emission is unchanged from current behavior"). (2) File a FOLLOWUP (e.g. `green-taproot-keypath-file-import-unverified`) to verify against Green whether a `tr(KEY)` descriptor file actually imports; if it does NOT, a future cycle escalates to blanket-refuse P2tr. This keeps the open question tracked rather than silently assumed-resolved.

---

## Minor findings

### M-1 — Stale line citations in item #3 (off by the doc-comment block)
The live H10 guard is at `cmd/export_wallet.rs:124-137` (the plan cites `:109-137`; lines 109-123 are the doc comment). The variant braces are at `error.rs:177-179` (plan cites `:170-179`; :170-176 are the doc comment). Re-grep and use live numbers per the project's citation-decay rule. (The guard *region* and variant both exist as claimed — only the line spans are loose.)

### M-2 — Frame item #3 as an *extension* of the existing guard, not introduction of it
The `ExportWalletUnsortedMultisigUnsupported` variant and the H10 guard already shipped (v0.62.0). The plan's prose "Add a guard … after the existing H10 guard" is correct but could misread as introducing the guard. State explicitly that item #3 adds a **second arm** keyed on `template.is_none() ∧ script_type ∈ {P2wshMulti, P2shP2wshMulti} ∧ unsorted-substring` for the direct-descriptor path. (Guard placement verified reachable — see Verified-correct.)

### M-3 — green.rs needs to ADD `WalletScriptType` to its `use super::{…}`
Confirmed: `green.rs:19` is `use super::{EmitInputs, MissingField, WalletFormatEmitter};` — `WalletScriptType` is **not** currently imported (unlike electrum.rs:15 which has it). The plan's code block references `WalletScriptType::P2tr`, so the import is mandatory. The plan flags this conditionally ("confirm at impl time"); make it a definite step.

### M-4 — Item #5 side-finding: a LIVE sibling-pin drift suggests CI may be RED on master (out of scope, but surface it)
`manual.yml:86` pins `descriptor-mnemonic-md-cli-v0.6.2` while `scripts/install.sh:35` canonical is `v0.7.1`. The Wave-3 slug `manual-yml-sibling-pin-vs-install-sh-drift-gate` is marked **`resolved`** (it shipped `sibling-pin-check.yml`), and that gate (`sibling-pin-check.yml`) does `exit 1` on `tag != canonical`. So master's `sibling-pin-check` workflow should currently be **failing** on the md pin. This is correctly OUT of scope for the 4 vacuity flips (which only require `MD_BIN=md`, a real binary — satisfied), but a live-RED CI gate is gate-relevant context: surface it as a separate item before/independent of shipping v0.70.1, and do not let the v0.70.1 ship-gate's "CI green" criterion mask it.

### M-5 — Version-site set is complete; one nit
Version sites confirmed at `0.70.0`: `Cargo.toml:3`, `README.md:13`, `crates/mnemonic-toolkit/README.md:9`, `scripts/install.sh:32` (`mnemonic-toolkit-v0.70.0`), `fuzz/Cargo.lock:575`, root `Cargo.lock:727`. The plan's list matches. Nit: the plan says "install.sh self-pin" — the file is `scripts/install.sh` (not root `install.sh`); use the correct path in the build plan.

---

## Verified-correct (load-bearing claims confirmed against live source)

- **Pinned SHA is HEAD.** `git rev-parse origin/master` = HEAD = `1cea85ea`. All citations against the live tip. ✓
- **`tap_tree()` API exists at the pinned miniscript rev.** miniscript git pin (`Cargo.lock:694-696`) rev `95fdd1c5773…`; `Tr::tap_tree(&self) -> Option<&TapTree<Pk>>` at `src/descriptor/tr/mod.rs:104`; field `tree: Option<TapTree<Pk>>` at :34. ✓
- **Q-A central claim — structural discriminator beats substring — EMPIRICALLY PROVEN** by compiling a probe against the toolkit's exact miniscript dep:
  - `tr(KEY)` keypath → `tap_tree().is_some()=false`, no `,{` → ALLOW ✓
  - `tr(NUMS,pk(A))` single-leaf → `tap_tree().is_some()=true` BUT `render_has(',{')=false` → **the substring draft would MISCLASSIFY as keypath and emit a mislabeled card; the structural check correctly refuses.** Exactly what test #2 distinguishes. ✓
  - `tr(NUMS,{pk,pk})` branch → `Some`, `,{`=true → both catch ✓
  - `tr(NUMS,multi_a(2,A,B))` → `multi_a` → `P2trMulti` → caught by EXISTING `is_multisig()` at green.rs:36, never reaches the new guard ✓
- **`Descriptor::from_str` accepts/validates the `#csum` suffix.** Probe: `tr(…)#wxhu4yg9` parses OK (`tap_tree().is_some()=true`); `#deadbeef` → `Err("invalid checksum")`. canonical_descriptor is toolkit-built (csum correct), so the re-parse always succeeds. ✓
- **green.rs current behavior.** Guards only `is_multisig()` (green.rs:36); `is_multisig()` (mod.rs:182-187) excludes `P2tr`; general tap-tree policy classifies `P2tr` (mod.rs:237-240, no `multi_a`) → mislabeled emission. Real wrong-LABEL defect. ✓
- **`EmitInputs.canonical_descriptor` is `CheckedDescriptor<'a>`** (string newtype, `Deref<str>`) mod.rs:504/446-485 — parsed descriptor NOT in EmitInputs → parse-inside-green.rs correct. ✓
- **`ToolkitError::BadInput` (exit 1)** matches restore-side; `DescriptorParse` (exit 2) correct for parse-failure map_err. Both exist (error.rs:11/:123; exits :551/:571). No new variant → no ordering concern. ✓
- **Restore-side refusal mirrored** at `restore.rs:2763-2769`: `format == Green && script_type == P2tr` in the `None` route-around branch (`P2tr ⟺ tap-script-tree policy`). Wording divergence intentional. ✓
- **Item #3 guard placement is REACHABLE (not dead code).** `emit_payload` (export_wallet.rs:73-173) runs top-to-bottom: `collect_missing` (:104) → H10 guard (:124-137) → `match format` dispatch to per-format `emit()` (:139+, Electrum :168). Field-less emitters' `ok_or_else("requires --template")` is INSIDE their `emit()` (electrum.rs:51, jade.rs:34, coldcard.rs:111), reached only at :168+; their `collect_missing` is no-op. New guard in the H10 region fires FIRST. Dead-code risk does NOT materialize. ✓
- **Test-to-flip is real/consistent.** `template_none_falls_through_to_generic_badinput_not_h10` (export_wallet.rs:1105) currently `assert_ne!`; fixture `inputs_with_template(None)` (:951) carries `wsh(multi(2,…))#abcdefgh` (unsorted) + `P2wshMulti` → flipped `assert_eq!` consistent. ✓
- **Sorted-ness substring sound** within the gated domain. `contains("multi(") && !contains("sortedmulti(")`: negative clause required (`sortedmulti(`⊃`multi(`); `multi_a(` lacks literal `multi(` (next char `_`); legacy `sh(multi)`→`P2shMulti`, excluded by the script-type clause. Inline-substring more robust than `template_from_descriptor` for the placeholder-xpub fixture. ✓
- **Item #5 CI is real-binary-bound.** manual.yml builds real `mnemonic` (:92-96), binds `make audit` to MNEMONIC_BIN=built / MD_BIN=md / MS_BIN=ms / MK_BIN=mk (:104-109) — no `=true`. Only `*_BIN=true` residue is in out-of-scope doc-CI (quickstart.yml, technical-manual.yml, manual-gui.yml) none of the 4 slugs cite. Doc-only flip safe. ✓
- **Item #4 current behavior loud-and-safe.** descriptor_intake.rs:156 prefix → parse_md1 (:210) → reassemble (:224) → typed MdCodec, exit 1. No bech32::decode on this path today. ✓
- **SemVer = PATCH v0.70.1.** #2 new-refusal → PATCH; #3 NO-BUMP; #4 deferred; #5 doc-only. No clap surface → no schema_mirror trip, no 40-cli-reference edit. ✓
- **No `--help`-visible surface change.** Only refusal behavior + messages + doc flips. GUI paired-PR not needed; manual flag-mirror unaffected. ✓

---

## Answers to Q-A … Q-E

- **Q-A:** **DISCRIMINATE — confirmed**, with caveat I-2 (reframe bip86 test as behavior-pinning + file verify-FOLLOWUP). Structural `tap_tree().is_some()` verified against pinned rev; no impl-time re-check needed.
- **Q-B:** **DEFER — confirmed** with corrected justification (I-1: no live defect; bech32 wrong for custom BCH; trap-doc must cite `md_codec::bch::bch_verify_regular`).
- **Q-C:** **INLINE SUBSTRING — confirmed** (smallest diff; matches mod.rs:264 idiom; more robust for the placeholder-xpub fixture).
- **Q-D:** **CONFIRMED** single PATCH v0.70.1, one tag, no codec/mk/ms/GUI bump.
- **Q-E:** **CONFIRMED** #5 doc-only; md-cli pin held for Wave 3 — but surface M-4 (likely-RED sibling-pin-check) separately; don't mask it under v0.70.1 "CI green".

---

**Gate status:** Not yet GREEN. 2 Important (I-1, I-2) block implementation — both reframe/justification folds (no code-design change) + one new tracked FOLLOWUP. Fold I-1, I-2, the 5 Minors, re-dispatch for convergence.
