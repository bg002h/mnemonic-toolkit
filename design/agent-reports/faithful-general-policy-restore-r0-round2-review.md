# R0 Review — faithful general-policy restore (PART 1) — ROUND 2

**Source SHA:** `5d599f7` (toolkit, crate at v0.53.9 — SPEC's "v0.53.9 → v0.54.0" correct) · sibling md-codec at workspace HEAD.

**Verdict: 🟡 — 0 Critical / 1 Important / 3 Minor**

## Round-1 findings
- **I1 — RESOLVED.** `k_opt` design verified end-to-end. Flagship reaches the clear refusal: §3 no longer computes `ms0` at classify, so the pk-keyed no-multi flagship passes classify (`None`) + slots, then hits `faithful_multisig_descriptor` → double-Check error → mapped clear message — never the cryptic `:843-844` k-gate. `k_opt.expect("plain arm has k")` safe: plain shape is `Wsh→Body::MultiKeys{k}` so `extract_multisig_threshold` returns `Some(k)`; taproot arm yields a template only for `multi_a/sortedmulti_a` (`MultiKeys`) so `Some(template) ⇒ Some(k)`. "After PART 2, zero toolkit changes" now TRUE. Both RED-cell shapes present.
- **I2 — RESOLVED.** Accept faithful BSMS + pin. `bsms.rs::emit` (:64-119) gates only `P2tr|P2trMulti`; line 2 = `CheckedDescriptor` verbatim; restore passes `BsmsForm::default()` (2-line). Format table COMPLETE: 11 `CliExportFormat` variants = 4 faithful + 7 refusing, all 7 gates re-verified (coldcard/coldcard-multisig/jade/electrum/sparrow template-`None`; green `is_multisig()`; specter `WalletName`).
- **I3 — RESOLVED.** `multipath==None` → empty per-key path (`to_miniscript.rs:116-130`), network-corrected pass-through is identity-faithful; `derive_receive_addresses` handles non-multipath. Panic-free. Cell present.
- **I4 — RESOLVED.** All four sites verified `:1135`/`:1139`/`:1158`/`:1189`; no 5th (the `:474` `wallet_type` is single-sig, unreachable in `--md1`). GUI field-enumeration mandate present; plain arm untouched.
- **M1 — RESOLVED-WITH-NOTE.** Cell counts fixed (13+12). Root-cause citation still `:838`; actual `:839` — see NEW-M2.
- **M2/M3/M4/M5/M6/M7 — RESOLVED.** Legacy `sh` pinned; duplicate-index fixture; M4 out-of-scope ack; `"imported-descriptor"` fallback + threshold fields; Translator totality; CheckedDescriptor wording.

## NEW findings
### Important
**NEW-I1 — the §1.C descriptor-match pseudocode passes literal `None` for `taproot_internal_key`, breaking every taproot md1 restore.** SPEC line: `Some(template) => build_descriptor_string(template, &slots, k_opt.expect(…), network, args.account, None)`. Current `:882` passes `tap_internal_key`; `build_tr_multi_a_descriptor` (`pipeline.rs:116-121`) hard-errors on `None`. A literal transcription REDs all tr-multi-a/tr-sortedmulti-a cells — contradicts §3 "taproot arm UNCHANGED". Loud (STAY-GREEN cells catch it), not Critical, but it's the load-bearing routing line. Fix: the taproot classify arm yields `template_opt = Some(tmpl)` + `tap_internal_key = Some(ik)`; the `Some` arm passes **`tap_internal_key`** (which is `None` only for non-taproot by construction). Spell out the unified `(template_opt, tap_internal_key)` binding so the if/else type unification is explicit.

### Minor
- **NEW-M1 — type/name drift (write-time-grep rule):** `extract_multisig_threshold` returns `Option<u8>` (not `Option<u32>`); `build_descriptor_string` takes `k: u8`. `plain_template_from_tree`'s tree param is `md_codec::tree::Node` (not `Tree`); `UseSitePath` is at `md_codec::use_site_path::UseSitePath`. Compile-time catches, but the citation rule applies.
- **NEW-M2 — M1 residual:** root-cause `:838` → `:839`.
- **NEW-M3 (optional):** a bare-key SINGLE-key wallet-policy md1 (md-cli `wpkh` card) now flows through the general arm — header "miniscript policy restore (1 cosigners)", envelope `mode:"multisig"`. Faithful/watch-only/loud-safe; consider N=1 wording + an optional pinning cell. Not blocking.

## Re-confirmations
Pivot/discriminator/translate_pk unchanged + sound. SemVer MINOR → v0.54.0; NO `schema_mirror`; GUI paired-PR + manual chapter correct. Existing 13+12 cells stay green under the folded design EXCEPT via NEW-I1's literal `None` (the tr cells) — that IS the finding. Label change is general-arm-only and safe (no existing cell asserts the "k-of-n multisig restore" header; `cli_restore_multisig_format.rs:208` `v["threshold"]==2` is a plain-arm bundle).

## Gate status
**NOT GREEN — 1 Important.** One surgical fold (thread `tap_internal_key` through §1.C + explicit `(template_opt, tap_internal_key)` unification) + 3 mechanical Minors. Implementation MUST NOT begin; fold, persist, re-dispatch ROUND 3 (fast confirm expected).
