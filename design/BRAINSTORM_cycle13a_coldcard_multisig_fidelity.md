# BRAINSTORM / SPEC — cycle-13 Lane A: Coldcard/Jade multisig fidelity PAIR (H11 + H14)

**Status:** DESIGN ONLY — feeds the mandatory R0 loop (spec R0 → plan → plan R0 → TDD). NO code yet.
**Cycle:** constellation bug-hunt cycle-13, Lane A (of 3 file-disjoint lanes — A / B(L8+L9) / C(M1+M7+L18) — all closing the bug-hunt in one toolkit MINOR **v0.66.0**).
**SemVer:** toolkit **MINOR v0.66.0** (H11 export wire-shape change for a previously-malformed case + H14 intake refusal change). md-codec / mk-codec / ms-codec / md-cli / ms-cli / mk-cli: **NO-BUMP**. GUI: NO schema_mirror impact (no clap flag added/removed/renamed).
**Source SHA pinned:** `origin/master = 9b2a8ae3` (toolkit v0.65.2). All citations below re-grepped against this SHA this session. (Re-pinned from the spec's original parent `d55bf4c3` after R0 round 1 — all cited source is byte-identical between the two; only the recon `.md` differs.)

---

## 0. Scope & framing

Two findings, **same format spec (Coldcard multisig text), opposite directions**:

- **H11 (export):** `wallet_export/coldcard.rs::emit_coldcard_multisig_text` collapses divergent cosigner origin paths to a wrong global placeholder `m/0'/0'`. `wallet_export/jade.rs:46` delegates byte-identical → inherits the bug.
- **H14 (import):** `wallet_import/coldcard_multisig.rs` uses each cosigner account-xpub's OWN fingerprint (`xpub.fingerprint()`, depth>0) as the BIP-380 **master** fingerprint, with no `.depth()` guard. Silently substitutes (Row 4) or spuriously warns (Row 2).

**Co-designed third surface (R0 round-1 I-1):** `wallet_import/roundtrip.rs::canonicalize_coldcard_multisig` — the round-trip-verify canonicalizer — re-emits in the **shared-derivation** canonical form using ONLY `parsed.cosigners[0].path` (`:401`, comment "canonicalization ASSUMES homogeneous derivation" `:395-400`). It is LIVE via `cmd/import_wallet.rs:1447` (`--round-trip-verify`) and `roundtrip.rs:570` (Jade round-trip, which delegates to it). H11's divergent exports flowing through this canonicalizer get cosigner-0's path stamped on **all** cosigners → silently discards the per-cosigner paths H11 now preserves → a divergent export would falsely pass / spuriously mismatch round-trip-verify. So H11's divergent emit **requires** a co-designed canonicalizer change (see §2.5, Decision H11-f). `roundtrip.rs` is therefore a **third affected file** (export + import + canonicalize).

**Funds-safety class (both):** **metadata-only / fidelity (DEMOTED, confirmed)** — addresses/xpubs unchanged; **NOT wrong-address**. But both break device round-trip / PSBT key-origin matching for legitimate collaborative-custody and authentic-export cases. Co-designed as ONE lane for spec coherence (same wire format, same fixtures). Different files → no self-conflict with Lanes B/C.

**Blast-radius note (R0 round-1 M-1):** the H14 import refusal lands in the SHARED parser `coldcard_multisig::parse_text`, which **JADE import also delegates to** (`wallet_import/jade.rs:133`; doc at `mod.rs:122` describes the delegation). So a depth>0/no-XFP Jade `multisig_file` blob is refused identically — the refusal surface is coldcard-multisig AND jade import. The I-2 path↔xpub pairing concern (below) is bounded: cycle-2's H10 refusal means EVERY reachable divergent coldcard/jade multisig *export* is SORTED (only `WshSortedMulti`/`ShWshSortedMulti` reach the emitter), so the sorted-slot pairing in H11-b is the only divergent case that can occur.

**Why full R0 (not reviewed-patch):** both change observable contracts in delicate, refuse-vs-substitute ways. H11 changes the export wire-shape; H14 turns a **previously-ACCEPTED** shape (Row 4, older-firmware/third-party) into a **REFUSAL** → blast radius onto older-firmware users. Refuse-vs-substitute semantics are exactly the R0-gated class.

---

## 1. EXTERNAL PROTOCOL FACTS — verified against authoritative source (not the report)

The constellation research-phase rule requires external protocol facts to be verified against authoritative source text. Evidence:

### 1.1 Coldcard multisig text format supports per-cosigner `Derivation:` lines — **VERIFIED**

**Evidence (authoritative, the toolkit's own round-trip parser, which is the canonical contract for this repo):**
`git show origin/master:crates/mnemonic-toolkit/src/wallet_import/coldcard_multisig.rs`, module doc lines **24–49**, documents TWO accepted on-disk shapes:

- **Shape 1 — shared-derivation** (`:26-36`): one `Derivation: m/...` line before the cosigner block, then `<XFP>: <xpub>` lines.
- **Shape 2 — per-cosigner** (`:38-49`): repeated `Derivation: m/...` / `<xpub>` pairs — *"older Coldcard firmware + several third-party coordinators emit this form."*

The parser's `Derivation` arm (`:233-244`) stages **every** `Derivation:` as `pending_per_cosigner_path`; the bare-xpub arm (`:268-274`) consumes it into `RawCosigner.per_line_path`; resolution (`:337-348`) uses **per-cosigner path, falling back to shared**. This is a **proven round-trip**: the toolkit's OWN import parser already accepts per-cosigner `Derivation:` overrides. SPEC §11.4 (`design/SPEC_wallet_import_v0_28_0.md:411-413`) likewise documents *"N per-cosigner blocks of: `Derivation: m/...` / `<xpub>`."*

→ **Conclusion:** the faithful per-cosigner form exists and is parseable. H11's fix (emit per-cosigner `Derivation:` on divergence) produces output the toolkit already round-trips. The `m/0'/0'` collapse is gratuitous corruption, not a format limitation.

**Caveat — RESOLVED by Q1 resolution-A (R0 round-1 RATIFIED):** the toolkit's export form (shape 1) interleaves `Derivation:` ahead of the cosigner block, but the divergent-path emission needs shape-2 ordering (`Derivation:`/`<XFP>: <xpub>` interleaved). TODAY the import `<XFP>:` arm (`:245-256`) builds a `RawCosigner` with `per_line_path: None` and *clears* `pending_per_cosigner_path` defensively (`:256`) — so per-cosigner `Derivation:` + `<XFP>: <xpub>` interleaving does NOT currently round-trip the path. **Resolution A (ratified) extends this arm to consume the pending per-line path** (without clearing `shared_derivation`), making the `Derivation: <path>\n<XFP_master>: <xpub>` per-cosigner form round-trip cleanly under H14-c. See §3.1 (Q1) and Decision H11-b.

### 1.2 The `<XFP>:` / `XFP:` header is the MASTER fingerprint by Coldcard convention — **VERIFIED**

`coldcard_multisig.rs` module doc `:51` — *"an optional leading `XFP: <hex>` line carrying the **master** fingerprint"*; inline `:183` (*"`XFP: <hex>` (optional **master** fingerprint header)"*); `:278` (*"the top-level `XFP:` header carries the **master**"*). The Coldcard convention: the device knows its true depth-0 master fp and stamps it. → A supplied XFP (header or per-line) is authoritative for the master fp.

### 1.3 `Xpub::fingerprint()` is the CURRENT key's own id, NOT parent/master — **VERIFIED against rust-bitcoin source**

Pinned dep: `bitcoin 0.32.8` (`git show origin/master:Cargo.lock`).
`bitcoin-0.32.8/src/bip32.rs:833-842` (Xpub `impl`):
```rust
pub fn identifier(&self) -> XKeyIdentifier {  // BODY: HASH160(self.public_key.serialize()) — :833
pub fn fingerprint(&self) -> Fingerprint { self.identifier()[0..4]... }                     // :840
```
So `Xpub::fingerprint()` = HASH160 of **THIS** key's pubkey — the current key's own id. For an account xpub at `m/48'/0'/0'/2'` (**depth 4**) this is the *account* key's id, **NOT** the master (depth-0) fp.

**Doc-comment caveat (R0 round-1 M-4):** the rust-bitcoin doc-comment directly above `identifier()` (`:832`) reads *"Returns the HASH160 of the chaincode"* — this is **stale/wrong**: the BODY hashes `self.public_key.serialize()` (the pubkey), not the chaincode. Our claim ("`fingerprint()` = HASH160 of the pubkey") matches the BODY, not the doc-comment. Cite the body, not the doc.

The struct also carries:
- `pub depth: u8` (`:111`) — *"How many derivations this key is from the master (which is 0)."* → `depth == 0` ⟺ this IS the master xpub.
- `pub parent_fingerprint: Fingerprint` — the IMMEDIATE parent (depth-1), NOT the master.

→ **Conclusion:** the master fp (depth-0) is **unrecoverable** from a depth>0 account xpub — `fingerprint()` gives the account's own id, `parent_fingerprint` gives only the depth-3 parent. **child→parent is one-way** (HASH160 is preimage-resistant; you cannot ascend the tree from a child xpub). This is the load-bearing fact for the H14 refuse decision.

### 1.4 SPEC §11.4.1 already half-acknowledges the depth distinction — and is itself buggy

`design/SPEC_wallet_import_v0_28_0.md:429` (the "Computed-fingerprint formula" note):
> `Xpub::fingerprint()` on the master xpub (if depth=0) or **on the cosigner xpub itself (if depth>0; per BIP-32 fingerprint-of-current-key semantics).**

This sentence is the **source of the bug rationale**: it conflates *"computed fingerprint = the xpub's own id at depth>0"* with *"the master fingerprint the BIP-380 key-origin needs."* They are NOT the same. The live code (`:359-360`) implements this buggy formula unconditionally. **SPEC §11.4.1 MUST be corrected** as part of this cycle (see §2.4).

---

## 2. The bugs — verified live-code citations (origin/master = 9b2a8ae3)

### 2.1 H11 export — VERIFIED

`crates/mnemonic-toolkit/src/wallet_export/coldcard.rs`:
- `:324-336` builds `derivations` (per-slot `origin_path_bare()` normalized to `m/...`) then collapses:
  ```rust
  let derivation = if !derivations.is_empty()
      && derivations.windows(2).all(|w| w[0] == w[1])   // all-equal
      && !derivations[0].is_empty()
  { derivations[0].clone() } else { "m/0'/0'".to_string() };  // ← collapse on divergence
  ```
- The cosigner emit loop `for cs in cosigners` (`:363`) pushes EXACTLY ONE `Derivation:` line (`:361`), then `<XFP>: <xpub>` per cosigner (`:367`, `cs.fingerprint`/`cs.xpub`) with **no** per-cosigner `Derivation:`.
- Export carries the correct master fp per slot: `cs.fingerprint` (`:367`) = `ResolvedSlot::fingerprint` (the true master fp), and `s.origin_path_bare()` (`:327`) = the per-slot real path. So the only corruption is the collapse — the data is present and correct upstream.
- **CRITICAL emit-ordering hazard (R0 round-1 I-2):** `derivations` is built from `inputs.resolved_slots` in **SLOT order** (`:324-328`), but for the sorted templates the cosigner emit loop iterates `cosigners` **lex-sorted by xpub** (`:339-346` `cosigners.sort_by(... a.xpub ... )`, then `for cs in cosigners` `:363`). A naive per-cosigner emit that paired a separate slot-order `derivations[i]` vector with the i-th SORTED cosigner would SCRAMBLE path↔xpub whenever sort-order ≠ slot-order — a corruption WORSE than the `m/0'/0'` it replaces. **The H11-b fix MUST read each cosigner's path from the SAME sorted slot it emits the xpub from** — never index a separate slot-order vector. See Decision H11-b.
- **Sorted-only reachability (R0 round-1 I-2):** cycle-2's H10 refusal (`cmd/export_wallet.rs:124-135`, `ExportWalletUnsortedMultisigUnsupported` constructed `:134`) refuses UNSORTED (`WshMulti`/`ShWshMulti`) export to electrum/coldcard/coldcard-multisig/jade → `emit_coldcard_multisig_text` is reachable ONLY via `WshSortedMulti`/`ShWshSortedMulti`. So **EVERY reachable divergent coldcard-multisig export is SORTED** (the `cosigners.sort_by` at `:345` always fires). The sort≠slot pairing above is therefore the only divergent case that can actually occur — it MUST be exercised by a RED test (RED test #1 as currently written, `@0`==`@2`, does NOT exercise it).
- **M-2 (toolkit's own export already mis-warns on re-import):** the toolkit's OWN shared-path export already round-trips through H14's Row 2 (spurious warning) TODAY: `synthesize.rs::synthesize_multisig_full` (`:594+`) derives the cosigner `xpub` at the BIP-48 leaf path (**depth 4**) via `derive_xpub_at_path`, while it stamps the true **depth-0 master fp** (`master_fingerprint`/`fp_bytes`) into `ResolvedSlot.fingerprint` (`:639`/`:650`, `pub fingerprint` at `:898`). So export emits `<XFP_master>: <depth-4-xpub>` → on re-import `supplied(master)` ≠ `computed(xpub.fingerprint())` → Row 2 warning on the toolkit's OWN all-agree export. **H14-c silences a warning that mis-fires on the happy path** — confirm the all-agree round-trip test goes SILENT (no warning) during the §3 H14-h fixture rewrite.
- `wallet_export/jade.rs:46` → `=> emit_coldcard_multisig_text(inputs)` for all 4 wsh/sh-wsh multisig templates. Inherits the bug; covered by delegation (no separate Jade fix needed).

### 2.2 H14 import — VERIFIED

`crates/mnemonic-toolkit/src/wallet_import/coldcard_multisig.rs`:
- `:358-360`: `let computed_fp = xpub_parse_result.as_ref().ok().map(|x| x.fingerprint());` — **no `.depth()` guard** anywhere in the file (grep-confirmed: only doc-comment "depth" mentions; `parent_fingerprint` never read).
- `:363-399` 5-row truth table (matches SPEC §11.4.1 `:419-427`):
  - Row 1 `(Some,Some) supplied==computed` → silent use supplied.
  - **Row 2** `(Some,Some) mismatch` → `xfp_header_disagreed=true` + WARNING (`:368-380`). At depth>0, `computed`≠the master XFP essentially always → **spurious warning on every authentic export**.
  - Row 3 `(Some,None)` → use supplied silently.
  - **Row 4** `(None,Some)` → use **computed silently** (`:386`). At depth>0 this **silently substitutes the account-key id as the master fp**.
  - Row 5 `(None,None)` → `ImportWalletParse` hard error.
- `:415`: `path_raw = format!("[{}{}]", effective_fp, path_components_str)` — the wrong fp is stamped into the `[fp/path]` key-origin for every cosigner.
- `:355`: `path_components_str` (and the typed `path: DerivationPath` at `:349`) is available → **the cosigner's path/depth IS known at decision time** (depth = number of path components, and `xpub.depth` is directly readable once parsed). No new data needed.
- **Test fixtures mask the bug:** `:945-947` pin `FP_A/B/C` to the COMPUTED `xpub.fingerprint()` values (`FP_A="34A3A4F1"` = `XPUB_A.fingerprint()`); every fixture asserts equality with the computed value → the divergence is never exercised. (R0 round-1 M-4: const block is `:945-947`, NOT `:939-947`; the first masked fixture `parse_shared_derivation_no_xfp_header_silent` is `:954+`.)

### 2.3 json_envelope NOTICE comparator — VERIFIED (for refuse-vs-NOTICE parity weighing)

`wallet_import/json_envelope.rs:383-410` (`mk1_card_to_resolved_slot`): when `card.origin_fingerprint` is absent it **substitutes `card.xpub.fingerprint()` with a loud stderr NOTICE** (`:392-398`) — *"master-fp and current-xpub-fp may differ; downstream wallets may show mismatched origins."* This is the **substitute-with-NOTICE** alternative H14 must weigh against **refuse**. Key difference: the mk1 path's xpub *may legitimately be depth-0 in some flows*, and the NOTICE is non-fatal. The coldcard-multisig Row-4 case is a depth>0 account xpub where the substitution is **provably** the wrong fp. (Decision in §3 resolves the asymmetry.)

### 2.4 SPEC §11.4.1 — must be corrected

`design/SPEC_wallet_import_v0_28_0.md:419-429`. The truth table conditions stay (header present / computed available / matches), but `:429`'s "Computed-fingerprint formula" is **wrong** and must be replaced with a depth-aware rule (§3.2). The `xfp_header_disagreed` warning must be gated on the depth-0 case.

### 2.5 `canonicalize_coldcard_multisig` collapses divergent paths — VERIFIED (R0 round-1 I-1)

`crates/mnemonic-toolkit/src/wallet_import/roundtrip.rs`:
- `:361` `pub(crate) fn canonicalize_coldcard_multisig(blob) -> Result<String, ToolkitError>` re-parses via `coldcard_multisig::parse_text`, then re-emits in the **shared-derivation** canonical shape: cosigner lines `<XFP>: <xpub>` lex-sorted by the formatted line (`:382-393`), top-level `XFP:` header DROPPED.
- `:395-401` derives the SINGLE shared `Derivation:` from **only** `parsed.cosigners[0].path`, with the comment *"canonicalization ASSUMES homogeneous derivation"* (`:395-400`). Every cosigner is emitted under cosigner-0's path.
- **LIVE callers:** `cmd/import_wallet.rs:1447` dispatches it for `--round-trip-verify` (the `"coldcard-multisig"` arm); `roundtrip.rs:570` `canonicalize_jade` delegates to it (Jade round-trip-verify reuses the inner Coldcard-multisig canonical form).
- **Why H11 breaks it:** once H11 emits per-cosigner `Derivation:` lines on divergence, those blobs run through this canonicalizer get cosigner-0's path stamped on ALL cosigners → the canonical form **discards the per-cosigner paths H11 preserved**. A divergent export run through round-trip-verify would then falsely pass (if the original also collapses) or spuriously mismatch — exactly the fidelity loss H11 fixes, re-introduced at the verify surface. **Resolution: extend the canonicalizer to emit per-cosigner `Derivation:` lines when `parsed.cosigners` carry heterogeneous paths** (mirroring H11's emit), with an idempotence test on a divergent blob. See Decision H11-f.

---

## 3. RESOLVED DESIGN DECISIONS

### Resolved-decisions table

| # | Decision | Resolution | Rationale / evidence |
|---|---|---|---|
| **H11-a** | Divergent paths: collapse to `m/0'/0'` vs per-cosigner `Derivation:` vs refuse | **Emit per-cosigner `Derivation:` lines; refuse if per-cosigner emission is structurally impossible; NEVER emit `m/0'/0'`.** | §1.1: faithful per-cosigner form exists + the toolkit round-trips it. `m/0'/0'` is gratuitous corruption. |
| **H11-b** | Wire-shape of the divergent emission + path↔xpub pairing | **Shape-2 ordering: per cosigner emit `Derivation: <path>` then `<XFP_master>: <xpub>`** (NOT bare-xpub — under Q1 resolution-A, ratified by R0, the import `<XFP>:` arm is extended to consume the pending per-line `Derivation:` path; §3.1-A). This carries BOTH the divergent path AND the master fp, and H14-c silently accepts it (supplied per-line XFP at depth>0). **MANDATORY pairing rule (R0 round-1 I-2): emit each cosigner's `Derivation:` from its OWN sorted-slot origin.** Sort the `(origin_path, xpub, fingerprint)` tuples TOGETHER (one sorted vector of slots, by the same sortedmulti xpub-lex key already used at `coldcard.rs:345`), then for each sorted slot read `cs.origin_path_bare()` + `cs.fingerprint` + `cs.xpub` from the SAME slot. **NEVER index a separate slot-order `derivations[i]` vector against the sorted cosigner loop** — that would scramble path↔xpub whenever sort-order ≠ slot-order (the §2.1 I-2 hazard, WORSE than `m/0'/0'`). Because of H10 (cycle-2), every reachable divergent export is SORTED, so this is the only path that executes. | §3.1-A makes the per-cosigner `Derivation:` + `<XFP>: <xpub>` form round-trip (parser change consumes the pending path). I-2: `derivations` is slot-order (`:324-328`) but the emit loop is xpub-sorted (`:339-346`/`:363`); reading the origin from the same sorted slot is the only non-scrambling form. |
| **H11-c** | Shared-path case (all agree) | **Unchanged: single shared `Derivation:` line + `<XFP>: <xpub>` cosigner lines** (current shape-1, byte-identical to today). | Only the divergent branch is malformed; preserve the common case wire-shape (no regression for the 99% case). |
| **H11-d** | Empty-path case (all slots have empty `origin_path_bare()`) | **Refuse** (exit non-zero) rather than emit `m/0'/0'`. | Empty origin = no faithful export possible; never substitute a placeholder origin into a steel backup. |
| **H11-e** | Jade | **Covered by delegation** (`jade.rs:46`); no separate code; add a Jade-path round-trip assertion. | jade delegates byte-identical. |
| **H11-f** | Round-trip-verify canonicalizer (R0 round-1 I-1) | **Extend `wallet_import/roundtrip.rs::canonicalize_coldcard_multisig` (`:361`) to emit per-cosigner `Derivation:` lines when `parsed.cosigners` carry heterogeneous paths** (mirror H11's divergent emit), instead of stamping `parsed.cosigners[0].path` on all (`:401`, "ASSUMES homogeneous derivation"). Homogeneous case unchanged (single shared `Derivation:`). Covers BOTH `--round-trip-verify` (`cmd/import_wallet.rs:1447`) and Jade round-trip (`roundtrip.rs:570` → delegates here). Add a divergent-blob idempotence test. | §2.5: without this, H11's divergent exports lose their per-cosigner paths at the verify surface (canonical form re-collapses) → false pass / spurious mismatch. The canonicalizer is the THIRD affected file. |
| **H14-a** | depth-0 cosigner xpub, NO supplied XFP | **`xpub.fingerprint()` IS the master fp → use it silently (current Row 4 behavior is CORRECT here).** | §1.3: at depth 0, `fingerprint()` = master id by definition. |
| **H14-b** | depth>0 cosigner xpub, NO supplied XFP | **REFUSE** (`ImportWalletParse`, exit 2). Master fp is unrecoverable from an account xpub (child→parent one-way). | §1.3. Was Row-4 silent substitute → now a clean, actionable refusal. **Blast radius: a previously-accepted older-firmware shape now refuses** — the R0-critical behavior change. |
| **H14-c** | depth>0 cosigner xpub, WITH supplied XFP (header or per-line) | **Accept the supplied XFP as authoritative; do NOT compute `xpub.fingerprint()`; do NOT emit the disagreement warning.** | §1.2: supplied XFP = master fp by Coldcard convention; comparing it to the account-key id (which can't match at depth>0) produces a guaranteed-spurious warning. |
| **H14-d** | depth-0 cosigner xpub, WITH supplied XFP | **Row 1/2 unchanged:** compare supplied vs computed; match→silent, mismatch→WARNING + use supplied. | At depth 0 the computed fp IS comparable to the master XFP → the disagreement warning is meaningful. |
| **H14-e** | Malformed xpub (computed unavailable), WITH supplied XFP | **Row 3 unchanged** (use supplied silently; xpub-parse error surfaces downstream). | Depth unknown when xpub won't parse; supplied XFP is the only signal. |
| **H14-f** | refuse vs substitute-with-NOTICE (json_envelope parity) | **REFUSE for coldcard-multisig depth>0/no-XFP** (NOT NOTICE-substitute). | The coldcard case is **provably** the wrong fp (account-key id ≠ master); json_envelope's mk1 NOTICE-substitute is a soft fallback for a path that may be depth-0. Refusing is the funds-safety-correct choice for a steel backup; a wrong master fp silently engraved is worse than a refusal. **R0 OPEN Q2** documents the dissent option. |
| **H14-g** | SPEC §11.4.1 `:429` formula | **Replace** with: *"the master fingerprint is `Xpub::fingerprint()` IFF `xpub.depth == 0`; at depth>0 the master fp is conveyed ONLY by a supplied XFP (header or per-line `<XFP>:`) and is otherwise unrecoverable → REFUSE."* Add a depth column to the truth table (or a depth-gating preamble). | §2.4. |
| **H14-h** | Test fixtures | **Rewrite** `:945-947` (`FP_A/B/C` consts) + dependent fixtures to use realistic master fps ≠ `xpub.fingerprint()` (supply XFP header / per-line `<XFP>:` carrying a synthetic master fp), so divergence is exercised. During the rewrite, confirm the toolkit's OWN all-agree export round-trips SILENT under H14-c (M-2: it emits depth-4 xpubs under a depth-0 master fp → fires Row 2 today). | §2.2 fixtures currently mask the bug. |
| **SV** | SemVer | toolkit **MINOR v0.66.0**; md/ms/mk codecs+CLIs NO-BUMP; GUI NO schema_mirror (no flag). | H11 wire-shape + H14 intake refusal both observable behavior changes; no clap surface change. |
| **EXIT** | Refusal exit codes | All refusals exit **non-zero**: H11-d/H14-b via `ToolkitError::ImportWalletParse` (export uses `BadInput`) → **exit 2** (import) / **exit 1** (export `BadInput`). Confirm in plan: export refusals are `BadInput` (`error.rs:549` exit 1); import refusals are `ImportWalletParse` (`error.rs:582` exit 2). | Never silently emit/accept a corrupt origin. |

### 3.1 The H11 emitted format (resolved)

```
# all-agree (unchanged, shape-1):
Name: <name>
Policy: <K> of <N>
Derivation: <shared m/...>
Format: <P2WSH|P2SH-P2WSH|P2SH>
<XFP_master>: <xpub>          (one per cosigner)
...

# divergent (NEW, shape-2 per-cosigner — resolution A, RATIFIED by R0 round-1):
Name: <name>
Policy: <K> of <N>
Format: <P2WSH|P2SH-P2WSH|P2SH>
Derivation: <m/path_for_sorted_slot_0>
<XFP_master_0>: <xpub_0>
Derivation: <m/path_for_sorted_slot_1>
<XFP_master_1>: <xpub_1>
...
```
Each `(Derivation:, <XFP>: <xpub>)` pair is read from the SAME sorted slot (H11-b mandatory pairing rule), so path↔xpub↔fp never scramble. The per-line `<XFP_master>` carries the true master fp; H14-c accepts it silently at depth>0.

**Q1 — RATIFIED by R0 round-1: resolution (A).** Extend the import `<XFP>:` cosigner arm (`coldcard_multisig.rs:245-256`) to ALSO consume a pending per-line `Derivation:` path (today it builds `per_line_path: None` and clears `pending_per_cosigner_path = None` at `:256`). Then H11 emits `Derivation: <path>\n<XFP_master>: <xpub>` per cosigner — carrying BOTH the master fp AND the divergent path — which H14 accepts via H14-c (supplied per-line XFP at depth>0, no warning). This is the coherent co-design; it makes the export round-trip cleanly under both fixes; it is a small, additive parser change.

**Q1 implementation conditions (R0 round-1, MANDATORY):**
- The parser change MUST consume ONLY the per-line pending path (`pending_per_cosigner_path.take()`); it MUST NOT clear or disturb `shared_derivation` (set at `:236-237`, fallback-consumed for cosigners 2..N at `:341` `.or(shared_derivation.as_deref())`). Clearing the shared path would break the all-agree shape-1 parse where one shared `Derivation:` precedes N `<XFP>: <xpub>` lines.
- ADD a **3-cosigner shared-path regression test**: a shared `Derivation:` + 3 `<XFP>: <xpub>` lines (NO per-line `Derivation:`) → assert ALL 3 cosigners still resolve to the SHARED path (proving cosigners 2..N still fall back to `shared_derivation` after the arm change). This is RED-test #13 below.
- **(B)** (leave the arm as-is, emit bare-xpub divergent) is **REJECTED** — self-inconsistent: H14 refuses depth>0 bare-xpub-no-XFP, so a toolkit divergent file wouldn't re-import. The pair must round-trip.

### 3.2 The H14 depth-gated truth table (resolved)

Decision matrix by **(cosigner-xpub depth, XFP supplied)**:

| depth | XFP supplied? | computed avail? | action |
|---|---|---|---|
| 0 | no | yes | **use computed (= master fp) — silent** (H14-a; Row-4-at-depth-0 stays) |
| 0 | yes | yes, ==supplied | use supplied — silent (Row 1) |
| 0 | yes | yes, ≠supplied | **WARNING + use supplied** (Row 2 — meaningful at depth 0) |
| >0 | no | yes | **REFUSE** `ImportWalletParse` (exit 2): master fp unrecoverable from a depth-N account xpub; supply the device's XFP (H14-b) |
| >0 | yes | (any) | **use supplied — SILENT (no disagreement warning)** (H14-c) |
| any | yes | no (xpub malformed) | use supplied — silent (Row 3 unchanged) |
| any | no | no | hard error (Row 5 unchanged) |

**Refusal message (H14-b), draft (R0 to finalize byte-exact):**
```
import-wallet: coldcard-multisig: parse error: cosigner <i>: cannot determine master
fingerprint — the cosigner xpub is at depth <d> (m/.../...'), so its own fingerprint is
NOT the master fingerprint a BIP-380 key-origin requires, and the master fingerprint is
unrecoverable from an account xpub. Re-export with the device's XFP (a top-level `XFP:`
header or a per-cosigner `<XFP>: <xpub>` line carrying the master fingerprint).
```

---

## 4. RED-test sketches (TDD — tests before impl; tests live in the BIN target → `cargo test -p mnemonic-toolkit`)

### H11 (export) RED tests
1. **`export_coldcard_multisig_divergent_paths_emits_per_cosigner_derivation`** — build a divergent SORTED 2-of-3 (`--template wsh-sorted-multi`; H10 means only sorted reaches the emitter) with distinct paths; export `--format coldcard-multisig`; assert output contains a `Derivation:` line for EACH cosigner with the real path, contains **NO** `m/0'/0'`, and (resolution-A) each cosigner line is `<XFP_master>: <xpub>` carrying its real master fp. RED today (emits one `Derivation: m/0'/0'`).
1b. **`export_coldcard_multisig_sort_order_ne_slot_order_pairs_correctly`** (R0 round-1 I-2 — the load-bearing pairing test) — build a divergent SORTED multisig whose **xpub-lex sort order ≠ slot order** AND whose per-cosigner paths differ (e.g. construct slots so the xpub that sorts FIRST is at a higher slot index with a distinct path); export `--format coldcard-multisig`; assert each emitted `Derivation:` line is paired with the **correct** `<XFP_master>: <xpub>` for the SAME slot (path↔xpub↔fp all from one sorted slot), NOT scrambled by reading a separate slot-order vector. RED today AND would stay RED under a naive `derivations[i]`-indexed fix → this test is what forces H11-b's same-sorted-slot rule. The original `@0`==`@2` shape (old test #1) does NOT exercise this.
2. **`export_coldcard_multisig_shared_path_unchanged`** — all-equal paths → single shared `Derivation:` line + `<XFP>: <xpub>` cosigner lines (byte-identical to current). GREEN-preserving regression guard.
3. **`export_coldcard_multisig_empty_origin_refuses`** — all slots empty `origin_path_bare()` → refuse, exit ≠ 0, message names the empty-origin cause, NO `m/0'/0'` in any output. RED today (emits `m/0'/0'`).
4. **`export_jade_divergent_paths_inherits_per_cosigner`** — same as (1) via `--format jade --template wsh-sorted-multi`; delegation covers it. RED today.
5. **Round-trip:** **`roundtrip_export_coldcard_multisig_divergent_then_import_matches`** — export divergent (1) → `import-wallet --format coldcard-multisig` → assert each cosigner's resolved `[fp/path]` equals the original (path divergent + master fp preserved). RED today (re-import collapses to `m/0'/0'`); **GREEN only after BOTH H11 and the 3.1-A import-arm change** — the key co-design assertion.

### H14 (import) RED tests
6. **`import_coldcard_multisig_depth_gt0_no_xfp_refuses`** — fixture: account xpub at depth 4 (`m/48'/0'/0'/2'`), shared `Derivation:`, **no** `XFP:` header, **bare** xpub (no `<XFP>:` prefix) → assert REFUSE (exit 2), message mentions "depth"/"master fingerprint unrecoverable". RED today (silently substitutes account fp).
7. **`import_coldcard_multisig_depth_gt0_with_header_xfp_no_warning`** — same depth-4 xpub WITH a top-level `XFP: <synthetic-master-fp>` (≠ `xpub.fingerprint()`) → assert resolved `[fp/path]` uses the **supplied** master fp, stderr is **SILENT** (no "disagrees with computed" warning), exit 0. RED today (Row 2 fires a spurious warning).
8. **`import_coldcard_multisig_depth_gt0_per_line_xfp_no_warning`** — shape-1 per-line `<XFP_master>: <xpub>` at depth 4 with `XFP_master ≠ xpub.fingerprint()` → resolved uses supplied, SILENT, exit 0. RED today (spurious warning).
9. **`import_coldcard_multisig_depth0_no_xfp_uses_computed`** — a genuine depth-0 master xpub, no XFP → uses `xpub.fingerprint()` silently, exit 0 (H14-a unchanged-at-depth-0). Guard against over-refusing.
10. **`import_coldcard_multisig_depth0_xfp_mismatch_warns`** — depth-0 xpub, supplied XFP ≠ computed → WARNING + use supplied (Row 2 still meaningful at depth 0). Guard against under-warning.
11. **Fixture rewrite regression** — the existing masked fixtures (`:954+`) updated to carry supplied master XFPs ≠ `xpub.fingerprint()`; assert they now exercise H14-c (silent, no warning) rather than asserting `xpub.fingerprint()` equality. Confirm the toolkit's OWN all-agree shared-path export (M-2: depth-4 xpubs under depth-0 master fp) now round-trips SILENT.

### Q1 parser-change regression (R0 round-1 condition)
13. **`import_coldcard_multisig_shared_derivation_3_cosigners_all_resolve_shared`** — shared `Derivation: m/48'/0'/0'/2'` + 3 `<XFP>: <xpub>` lines (NO per-line `Derivation:`) → assert ALL 3 cosigners resolve to the SHARED path (proving the extended `<XFP>:` arm does NOT clear `shared_derivation`, so cosigners 2..N still fall back to it via `:341`). Guards the Q1 resolution-A parser change against breaking shape-1.

### Jade-import refusal surface (R0 round-1 M-1)
14. **`import_jade_depth_gt0_no_xfp_refuses`** — a Jade `get_registered_multisig` reply whose inner `multisig_file` carries a depth>0 cosigner xpub with NO XFP (header or per-line) → assert REFUSE (exit 2), via the shared `coldcard_multisig::parse_text` delegated from `wallet_import/jade.rs:133`. Proves H14-b fires on the Jade import surface, not only direct coldcard-multisig import.

### Canonicalizer divergent treatment (R0 round-1 I-1)
15. **`canonicalize_coldcard_multisig_divergent_paths_preserves_per_cosigner`** — feed a divergent-path coldcard-multisig blob (heterogeneous `parsed.cosigners[].path`) to `canonicalize_coldcard_multisig`; assert the canonical form emits per-cosigner `Derivation:` lines (NOT cosigner-0's path stamped on all), and is **idempotent** (`canon(canon(blob)) == canon(blob)`). RED today (re-emits shared-derivation form from `cosigners[0].path` `:401`). Homogeneous-path blob still canonicalizes to the single-shared-`Derivation:` form (existing `canonicalize_coldcard_multisig_idempotent` unchanged).
16. **`roundtrip_verify_divergent_coldcard_multisig_passes`** — `import-wallet --format coldcard-multisig --round-trip-verify` on a divergent-path blob → assert the round-trip-verify PASSES (canonical form preserves the per-cosigner paths, so no spurious mismatch / false pass). Covers the LIVE `cmd/import_wallet.rs:1447` surface; the Jade analogue rides `roundtrip.rs:570`. RED today (canonicalizer collapses).

### Cross-pair round-trip (the co-design proof)
12. **`roundtrip_divergent_master_fp_and_paths_preserved`** (= test 5, the headline) — divergent paths + distinct master fps survive export→import unchanged. This is the single assertion that proves H11 and H14 compose.

---

## 5. Lockstep / manual / CI

### Affected files (R0 round-1 — now THREE source files + SPEC)
- `crates/mnemonic-toolkit/src/wallet_export/coldcard.rs` — H11 divergent per-cosigner emit + sorted-slot pairing (H11-b).
- `crates/mnemonic-toolkit/src/wallet_import/coldcard_multisig.rs` — H14 depth-gated truth table (H14-a..g) + Q1 resolution-A `<XFP>:` arm change (consume pending per-line path, do NOT clear `shared_derivation`).
- `crates/mnemonic-toolkit/src/wallet_import/roundtrip.rs` — **NEW (I-1):** `canonicalize_coldcard_multisig` per-cosigner `Derivation:` emit on heterogeneous paths (H11-f); covers `--round-trip-verify` + Jade round-trip.
- `design/SPEC_wallet_import_v0_28_0.md` §11.4.1 — depth-gated truth-table correction (H14-g).
- (`wallet_export/jade.rs` + `wallet_import/jade.rs` are covered by delegation — no edits, but Jade round-trip + Jade-import-refusal tests added.)

- **schema_mirror:** **NOT touched** — no clap flag/option/subcommand/dropdown-value added/removed/renamed. No `mnemonic-gui` schema PR required. (Per CLAUDE.md the gate is flag-NAME parity; H11/H14 change wire-shape + intake semantics, not flags.)
- **`--json` wire-shape:** unaffected (H11 is a text-format emitter; H14 is import-side resolution). No GUI `--json`-consumer paired-PR concern.
- **Manual prose (same-PR, NOT gate-enforced — voluntary hygiene):**
  - `docs/manual/src/45-foreign-formats.md` — document the per-cosigner `Derivation:` divergent export form + the depth>0-no-XFP import refusal + "supply the device XFP" guidance.
  - `docs/manual/src/30-workflows/37-wallet-export.md` — note the divergent-cosigner coldcard/jade export behavior.
  The manual-mirror lint is flag-name based → it will NOT fire on these prose edits; ship them in-PR as good hygiene.
- **SPEC update (in-repo, same-PR):** `design/SPEC_wallet_import_v0_28_0.md` §11.4.1 (`:419-429`) — depth-gated truth table + corrected computed-fingerprint formula (§3.2, H14-g).
- **NEVER `cargo fmt`** the toolkit (`mlock.rs` permanently fmt-exempt — MEMORY). Tests in the BIN target → `cargo test -p mnemonic-toolkit` (R0/per-phase reviews run the **FULL** package suite, not targeted `--test` targets — the stale-lint lesson).
- **md-codec / mk-codec / ms-codec / md-cli / ms-cli / mk-cli:** NO-BUMP, no cross-repo companion, no FOLLOWUPS sibling-mirror (toolkit-only).
- **Co-ship:** Lane A ships under the full R0 gate; Lanes B (L8+L9) + C (M1+M7+L18) ride as reviewed-patch phases in the SAME cycle → **one toolkit MINOR v0.66.0** closing the constellation bug-hunt. Serialize the version bump; ship order A→B→C (A is the MINOR; B/C fold in).

---

## 6. MANDATORY R0 GATE (CLAUDE.md hard gate)

**NO code before GREEN (0 Critical / 0 Important).** This brainstorm SPEC → opus architect **R0 review** BEFORE any implementation. Fold findings → persist the review verbatim to `design/agent-reports/cycle13a-coldcard-multisig-spec-R<n>-review.md` → re-dispatch → repeat until GREEN (the reviewer-loop continues after EVERY fold — folds introduce drift). Then: plan-doc → its own R0 loop → single-subagent TDD per phase → per-phase reviews → mandatory whole-diff adversarial execution review. Any agent verifying external protocol facts (Coldcard format, BIP-32 fingerprint semantics) re-checks authoritative source, not just this doc.

### Open questions — RATIFIED by R0 round-1 (kept here for the audit trail)
- **Q1 (load-bearing co-design) — RATIFIED resolution (A)** + the two MANDATORY conditions in §3.1: extend the import `<XFP>:` cosigner arm (`coldcard_multisig.rs:245-256`) to consume the pending per-line `Derivation:` path. **Conditions:** (i) consume ONLY the per-line pending path — MUST NOT clear/disturb `shared_derivation` (set `:236-237`, fallback-consumed at `:341`); (ii) ADD the 3-cosigner shared-path regression test (RED test #13) proving cosigners 2..N still resolve to the SHARED path after the arm change.
- **Q2 (refuse-vs-NOTICE parity) — RATIFIED: REFUSE** for coldcard-multisig depth>0/no-XFP (H14-b/H14-f). The master fp is provably unrecoverable from a depth>0 account xpub (HASH160 one-way); refusing before steel-engraving is funds-safety-correct; no real Coldcard export omits XFP at depth>0 (firmware stamps `XFP:`), and the no-XFP older form pairs with depth-0 master xpubs which still pass via H14-a. NOT json_envelope's NOTICE-substitute (that path may legitimately be depth-0).
- **Q3 (depth source) — RATIFIED: `xpub.depth == 0`** (the decoded public field, §1.3) is the discriminator — authoritative, independent of the declared path. Toolkit's own export emits depth-4 account xpubs (so a re-serialized account xpub is never depth 0 — concern moot). Row 3 (malformed xpub, no decoded depth) stays "use supplied" regardless.
