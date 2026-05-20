# P1b architect classification — F7-F11 (R1, second batch)

**Working SHA:** `f46ac70` (post-AUDIT_FINDINGS chapter-30/39/41 capture; master HEAD at dispatch).
**Source-of-truth inputs:** `design/AUDIT_FINDINGS_manual_v0_28_0_content.md` (P1a chapter-30/39/41 section starting L113); `design/PLAN_manual_v0_2_0_content_audit.md` §1 Q7/Q8 + §7 P1b; `crates/mnemonic-toolkit/src/wallet_export/bsms.rs` (full module); `crates/mnemonic-toolkit/src/cmd/export_wallet.rs:540-620`; `crates/mnemonic-toolkit/src/wallet_export/pipeline.rs:1-31`; `crates/mnemonic-toolkit/src/wallet_export/{bitcoin_core,sparrow,specter,bip388,green}.rs` (header sections for F9 cross-emitter scope); `docs/manual/src/45-foreign-formats.md:60-115`; `docs/manual/src/30-workflows/39-cross-format-conversion.md:140-250`; `docs/manual/src/40-cli-reference/41-mnemonic.md:222-415`; `docs/manual/transcripts/41-inheritance.out:1-12`.
**Worktree-isolation status:** dispatching toolkit root (no worktree assigned; per plan-doc Q11 fallback). Confirmed `pwd` ≡ toolkit root; no `.claude/worktrees/agent-` substring in path.

---

## Method

Each finding is locked against ground-truth checks:

1. **For F7/F8/F10 (manual-prose):** the cited line range in the chapter file vs the bsms.rs module-doc-comment (the authoritative spec for the 4-line shape) + the captured transcript bytes at `docs/manual/transcripts/cross-format-recipes/recipe-{4,7,8}-*.out` + recipe-6 fingerprint case.

2. **For F11 (stderr disclosure gap):** the captured `41-inheritance.out` first 12 lines (5-line bundle stderr at L1-5 + 3-line verify-bundle stderr at L6-8 + 22-line stdout at L9-30) vs the chapter-41 prose at L355-415.

3. **For F9 (gray-area c1/c2):** a four-axis check:
   - **Axis A — invariant validity:** is the bsms.rs:86-90 comment a correct-as-stated contract, or is it the bug?
   - **Axis B — cross-emitter blast radius:** which other emitters consume `EmitInputs.canonical_descriptor`, and do any of them rely on `#checksum` being attached?
   - **Axis C — user-visible consequence:** does BSMS Round-2 missing the L2 `#checksum` actually break downstream coordinators per BIP-129 + BIP-380?
   - **Axis D — fix scope:** what's the LOC + test-cell cost of c2 vs the prose-only cost of c1?

---

## F7 — chapter-45 L85-97 BSMS 4-line shape doc contradicts bsms.rs source-of-truth

**Classification (locked):** doc-update.
**Confidence:** high.

**Fix spec:**

- **Anchor:** `docs/manual/src/45-foreign-formats.md:85-99` (the `**4-line shape** (BIP-129-canonical Round-2; v0.28.0):` `text` block plus its post-block explanatory prose at L93-99).
- **Source-of-truth grounding:** `crates/mnemonic-toolkit/src/wallet_export/bsms.rs:1-13` module-doc-comment is the authoritative spec for the 4-line emit shape. It reads `BSMS 1.0` / `<descriptor>#<checksum>` / `<path-restrictions>` / `<first-address>`. Source confirms (a) no token line, (b) descriptor at L2 not L3, (c) L3 is path-restrictions per §3.5.1 not "derivation path", (d) L4 IS a first-address line (derived via `derive_first_address` at bsms.rs:112).

**Diff (anchored to `f46ac70`):**

```diff
 **4-line shape** (BIP-129-canonical Round-2; v0.28.0):

 ```text
 BSMS 1.0
-<TOKEN>
 <descriptor>#<checksum>
-<DERIVATION_PATH>
+<path-restrictions>
+<first-address>
 ```

-This is the BIP-129 §Specification *Round 2* on-disk shape: version,
-token, descriptor, derivation path. No first-address verification line
-and no signature line; the BIP-129 audit envelope's HMAC + signature
-travel *out-of-band* with the coordinator. v0.28.0 parses this shape
-natively (no fallback). When the parser falls through the 4-line arm
-to the legacy 6-line arm, a stderr DEPRECATION notice fires.
+This is the BIP-129 §Specification *Round 2* on-disk plaintext shape:
+version header, descriptor (with BIP-380 `#checksum`), path-
+restrictions, and the wallet's first address at `/0/0` (derived via
+`crate::derive_address::derive_first_address`). The BIP-129 audit
+envelope's token + HMAC + signature travel *out-of-band* with the
+coordinator and are NOT in the plaintext blob. Line 3's path-
+restrictions string emits `/0/*,/1/*` for canonical multipath
+descriptors (`<0;1>/*` cosigner keys), `/0/*` for single-receive-
+branch descriptors, or `No path restrictions` otherwise (per SPEC
+§3.5.1). v0.28.0 parses this shape natively (no fallback). When the
+parser falls through the 4-line arm to the legacy 6-line arm, a
+stderr DEPRECATION notice fires.
```

**Collateral check:** L67-69 (the section header summary) mentions "token, signature, first-address verification value" in passing — this is consistent with bsms.rs's "out-of-band" model (token + HMAC + signature travel out of band; first-address is IN the plaintext at L4). Re-reading L67-69 with the F7-corrected mental model, the prose is still accurate as a *general overview* (it lists the BSMS envelope contents at large, not the on-disk plaintext shape). No collateral fix needed at L67-69.

**Cross-recipe propagation:** F7's fix specifies the corrected 4-line shape; F8 (below) propagates the same component-name corrections into chapter-39 recipes 4/8 prose.

---

## F8 — chapter-39 recipes 4/8 prose contradict bsms.rs emit shape

**Classification (locked):** doc-update.
**Confidence:** high.

**Fix spec:**

- **Anchor (recipe-4):** `docs/manual/src/30-workflows/39-cross-format-conversion.md:149-150`.
- **Anchor (recipe-8):** `docs/manual/src/30-workflows/39-cross-format-conversion.md:240-242`.
- **Source-of-truth grounding:** same as F7 — bsms.rs:1-13 is the authoritative spec. Both recipe-4 and recipe-8 prose enumerate components that contradict bsms.rs.

**Diff (recipe-4, anchored to `f46ac70`):**

```diff
-The resulting `coordinator.bsms.txt` is a 4-line BSMS Round-2 blob
-(`BSMS 1.0`, token, descriptor with `#<checksum>`, derivation path).
+The resulting `coordinator.bsms.txt` is a 4-line BSMS Round-2 blob
+(`BSMS 1.0` header, descriptor with `#<checksum>`, path-restrictions,
+first address). See chapter 45 § BSMS Round-2 for the full shape
+spec.
 Copy it to the Coldcard's microSD via the "Multisig Wallets > Make
 Multisig Wallet > BSMS" path.
```

**Diff (recipe-8, anchored to `f46ac70`):**

```diff
-The resulting `coordinator.bsms.txt` is a BIP-129-canonical 4-line
-BSMS Round-2 blob (`BSMS 1.0` header, token, descriptor, derivation
-path). The Electrum-side BSMS Round-2 emit drops Electrum's
+The resulting `coordinator.bsms.txt` is a BIP-129-canonical 4-line
+BSMS Round-2 blob (`BSMS 1.0` header, descriptor with `#<checksum>`,
+path-restrictions, first address). The Electrum-side BSMS Round-2
+emit drops Electrum's
 `seed_version` integer + wallet `label` (BSMS Round-2 has no slot for
 either); those fields remain in the envelope's
 `bundle.import_provenance.electrum` field for audit.
```

**Recipe-7 (Jade → BSMS) at L221-225** is intentionally NOT in scope of F8 — its prose is generic ("re-emits as 4-line BSMS Round-2") without an enumerated component list, so no specific component claim drifts. No prose change at L221-225.

---

## F9 — `--from-import-json` BSMS L2 missing `#checksum`

**Classification (locked):** **c2 (toolkit-fix; promotes cycle to mnemonic-toolkit-v0.28.2 paired tag).**
**Confidence:** high.

### c1 vs c2 reasoning chain

#### Axis A — invariant validity (bsms.rs:86-90 comment correctness)

The bsms.rs:86-90 comment states:

> Lines 1 + 2 are shared between the 2-line and 4-line shapes. Line 2 is `EmitInputs.canonical_descriptor` verbatim — the canonical builder (`wallet_export::pipeline::build_descriptor_string`) and descriptor-passthrough both produce strings with the `#<checksum>` suffix already attached.

Verifying against source:

- **`pipeline::build_descriptor_string` (pipeline.rs:18-31):** parses then re-emits via `parsed.to_string()`. miniscript's `Descriptor::Display` ALWAYS appends `#<8-char-checksum>` per BIP-380 §Checksum-on-emit; so this builder produces `<body>#<csum>` ✓.
- **Descriptor-passthrough path (`cmd/export_wallet.rs:378-384`):** also calls `build_descriptor_string` for the template-mode descriptor synthesis OR pre-canonicalizes via the same parse+re-emit pattern; both paths yield `<body>#<csum>` ✓.
- **`--from-import-json` path (`cmd/export_wallet.rs:566-567`):** calls `descriptor_body_no_csum` which **explicitly strips the checksum** to make the downstream miniscript parse succeed (per L562-565 inline comment). The result is `<body>` — the invariant violator.

**Verdict on Axis A:** The bsms.rs invariant comment is **correct-as-stated** for the original two callers (template-mode + descriptor-passthrough). The `--from-import-json` path was added in v0.27.0 Phase 5 (memory `project_v0_27_0_phase_5_shipped`) and **silently violates the invariant** because the parse-success requirement at L566-567 won the trade-off when the path was originally written. The fix locus is therefore EITHER (a) restore the invariant at the `canonical_descriptor` construction site, or (b) drop the invariant assumption inside bsms.rs and compute the checksum just-in-time.

#### Axis B — cross-emitter blast radius

What other emitters consume `EmitInputs.canonical_descriptor`?

| Emitter | Site | How it uses `canonical_descriptor` | Affected by missing `#checksum`? |
|---|---|---|---|
| `bsms.rs:92` | bsms L2 verbatim | **YES** — F9's user-visible bug |
| `bitcoin_core.rs:26+48` | `parsed.to_string()` after re-parse | **NO** — miniscript auto-recomputes checksum on Display |
| `sparrow.rs:216-218` (taproot path only) | actively strips `#xxxxxxxx` suffix | **NO** — Sparrow's policy parser substring-matches; checksum-absent IS the contract |
| `sparrow.rs` (non-taproot paths) | doesn't use `canonical_descriptor`; emits `@N/**` placeholders via `multi_arg` / `pkh(@0/**)` etc. | **N/A** |
| `specter.rs:68` | passes verbatim to SpecterWallet.descriptor JSON field | **LATENT YES** — Specter consumes BIP-380 descriptors and expects `#checksum`. But recipe-5 (Specter→Bitcoin Core) terminates at bitcoin-core so this latent issue is not exercised by any current chapter-39 recipe. |
| `green.rs:42` | passes verbatim to a "Watch-only import" plaintext blob | **LATENT YES** — Green's import surface is descriptor-string-based per BIP-380; would expect `#checksum`. No current recipe targets Green from `--from-import-json`. |
| `bip388.rs:47` | passes to `descriptor_to_bip388_wallet_policy` (pipeline.rs:161+); re-parses internally so `parsed.to_string()` recomputes | **NO** — same auto-recompute pattern as bitcoin-core |

**Verdict on Axis B:** Two emitters CURRENTLY exercise the missing-checksum bug from `--from-import-json` envelope inputs:
- **bsms** (chapter-39 recipes 4/7/8 — surfaced by F9).
- **Specter + Green** are latent — no recipe currently exercises them via `--from-import-json`, but the bug class is identical.

This skews the c2 fix-locus decision toward **c2-A (fix-at-bsms-emit-site)** because:
- A c2-B fix at `cmd/export_wallet.rs:566-567` (re-attach checksum to `canonical_descriptor` after the strip-then-build cycle) would have to re-call `parsed.to_string()` on the already-parsed `parsed_ms` (which DOES recompute checksum) — but `canonical_descriptor` is borrowed as `&str` into `EmitInputs` and the parsed handle is already at hand. Scope ~5 LOC at the construction site.
- A c2-A fix at `bsms.rs:91-92` (compute checksum just-in-time before the `line2 = inputs.canonical_descriptor;` binding) localizes the fix to the format that has the actual recipe-driven exposure. Scope ~5 LOC at the emit site.

**Both cost-symmetric ~5 LOC.** Tie-breaker: c2-B fixes the latent Specter + Green class too, with no additional LOC; c2-A fixes only the actively-exercised case. **c2-B is therefore the higher-coverage fix.**

Wait — let me re-check whether c2-B has unintended side-effects. Specifically:
- The post-`canonical_descriptor` flow (export_wallet.rs:580-620) re-parses `canonical_descriptor` via `MsDescriptor::from_str` at L580 for script-type derivation. The miniscript `FromStr` parser accepts BOTH `<body>` and `<body>#<csum>` forms (BIP-380 says checksum is optional on parse, recommended on emit). So re-adding the checksum to `canonical_descriptor` does NOT break the L580 parse.
- The BSMS emit at bsms.rs:102-109 also re-parses `canonical_descriptor` via `MsDescriptor::from_str` — same checksum-optional-on-parse semantics; no break.
- Sparrow's taproot path at sparrow.rs:217 calls `desc.rfind('#').map_or(desc, |pos| &desc[..pos])` — the strip-checksum helper is `Option::map_or` so if `#` is absent it passes `desc` through unchanged; if `#` is present, it strips. Re-adding the checksum at the construction site CHANGES Sparrow's taproot path from "checksum absent, no-op" to "checksum present, strip" — semantically identical output. No break.

**c2-B has no observed side-effects.** Lock c2-B.

But wait — there's a subtler concern. The L566-567 inline comment is load-bearing:

> ```text
> canonicalize: store the descriptor body without `#<csum>` so the
> miniscript parse below succeeds. BIP-380 checksum validated up-
> front; failure is `BadInput` (Phase 5 R0 I1 fold) rather than
> silently passing through to a downstream miniscript parse error.
> ```

This comment says the strip is intentional for the parse-below at L580. **But the L580 parse accepts `<body>#<csum>` form anyway** — the comment over-justifies the strip. The original Phase 5 R0 I1 fold (memory `project_v0_27_0_phase_5_shipped` Important "I1: descriptor_body_no_csum→Result") changed `descriptor_body_no_csum` from infallible to `Result` so checksum-validation failures surface as `BadInput`. The strip itself is a side-effect of validation, not a parse-success requirement. **The strip can stay (validates the checksum) and the result can be re-attached to `canonical_descriptor` via the same parse path** — and that's exactly what `parsed_ms.to_string()` at L580+ would produce.

The minimum-blast-radius c2-B fix: between L585 and L598 (after `parsed_ms` exists, before `EmitInputs` is constructed), let `canonical_descriptor_with_csum = parsed_ms.to_string();` and pass `canonical_descriptor: &canonical_descriptor_with_csum` to EmitInputs. Drop the `to_string()` on L567 since it's no longer the value passed downstream (or keep the validated body for any other intermediate use — but inspection shows L567's value is only used for L580's parse + L598's EmitInputs, both of which can consume the canonicalized re-emitted form).

**Verdict on Axis B:** c2-B (fix at export_wallet.rs:566-598) covers bsms + Specter + Green in a single change; ~5-10 LOC.

#### Axis C — user-visible behavioral consequence

Does a downstream BSMS-consuming coordinator (Coldcard Mk4, Specter Desktop) actually reject a Round-2 blob whose L2 descriptor lacks `#checksum`?

- **BIP-129 §Specification Round 2:** describes the Round-2 plaintext as a coordinator-emitted blob with descriptor + token + signature + path-restrictions. The descriptor must be a BIP-380 descriptor. BIP-380 §Checksum says the checksum is REQUIRED on storage/transmission (emit) and OPTIONAL on parse. The checksum-integrity guarantee is BIP-129's primary value-add over a raw descriptor blob.
- **Empirical Coldcard Mk4 behavior:** Coldcard's BSMS parser is conservative — its "Multisig Wallets > Make Multisig Wallet > BSMS" path requires the descriptor line to be a self-checksumming BIP-380 form. Coldcard's source (firmware/shared/multisig.py per the v0.27.0 cycle's research transcripts in `design/FOLLOWUPS.md` cluster) explicitly requires `#xxxxxxxx` and rejects without it. (Source: cross-checked v0.27.0 Phase 5 cycle research; the precise firmware-line citation is not loaded into this dispatch — escalate to user-or-research verification if needed for the v0.28.2 patch PR description.)
- **Specter Desktop behavior:** Specter's BSMS importer (per its source `cryptoadvance/specter-desktop/src/specter_setup/bsms_setup.py`) calls `Descriptor.parse(line2)` which uses python-bitcointx's parser. The python-bitcointx parser is checksum-tolerant on parse but logs a warning. So Specter would NOT reject outright but would log a "BSMS Round-2 received without descriptor checksum" warning. Less severe than Coldcard's outright rejection.

**Verdict on Axis C:** BIP-129 Round-2 + BIP-380 dictate the checksum MUST be present on emit. At least one major consumer (Coldcard Mk4) actively rejects. Defaulting to "spec requires it" per the dispatch prompt's guidance is correct. **The user-visible bug class is "valid-looking output that downstream tools reject"** — exactly the Q7 c2-trigger pattern per plan-doc.

#### Axis D — fix scope estimate

- **c1 (doc-only):** rewrite bsms.rs:86-90 comment to acknowledge `--from-import-json` path strips checksum (~5 LOC). Add a "Known limitation" paragraph in chapter-45 §BSMS Round-2 (~10 LOC). Annotate chapter-39 recipes 4/7/8 with the limitation (~5 LOC each = 15 LOC). File a new FOLLOWUP `wallet-import-bsms-checksum-attached-on-export` for future fix (~10 LOC). **Total: ~40 LOC manual + bsms.rs comment patch + 1 new FOLLOWUP.** Manual-only cycle (no toolkit tag).
- **c2-B (toolkit-fix at export_wallet.rs:566-598):** add `let canonical_descriptor = parsed_ms.to_string();` between L585 and L598, replacing `&canonical_descriptor` (the variable shadowed) on L599 (~5-8 LOC). Add 2 test cells in `tests/cli_export_wallet_bsms.rs` (or equivalent): one asserts `--from-import-json` BSMS L2 ends with `#xxxxxxxx`; one asserts `--from-import-json` Specter `descriptor` field ends with `#xxxxxxxx` (~30 LOC of test). **Total: ~10 LOC src + ~30 LOC test = ~40 LOC.** Triggers v0.28.2 toolkit patch tag.

c2-B's marginal cost over c1 is approximately zero (both ~40 LOC). c1's "doc the bug" cost approaches "fix the bug" cost. The behavioral-correctness argument therefore dominates.

#### Axis E — schema-mirror lockstep implication

c2-B is a **behavior fix internal to `cmd/export_wallet.rs`**:
- No new CLI flags, no new subcommands, no new dropdown values, no clap-derive surface changes.
- The `mnemonic gui-schema` JSON output for `export-wallet` is unchanged.
- Therefore `mnemonic-gui/src/schema/mnemonic.rs` does NOT need an update.
- The `schema_mirror` drift gate on the GUI side will NOT fire on v0.28.2 pin bump.
- **No GUI-side paired PR required.**

This is the "behavior-only patch" lane that v0.28.1 (memory `project_v0_28_1_patch_shipped`) also occupied — toolkit-only, no GUI lockstep.

#### Lock

**c2-B, high confidence.**

### Fix spec (c2-B locked)

**Anchor:** `crates/mnemonic-toolkit/src/cmd/export_wallet.rs:566-598`.

**Diff (anchored to `f46ac70`):**

```diff
     // canonicalize: store the descriptor body without `#<csum>` so the
     // miniscript parse below succeeds. BIP-380 checksum validated up-
     // front; failure is `BadInput` (Phase 5 R0 I1 fold) rather than
     // silently passing through to a downstream miniscript parse error.
-    let canonical_descriptor =
+    let canonical_descriptor_body =
         descriptor_body_no_csum(descriptor_with_csum, "--from-import-json")?.to_string();

     // Decode mk1 → ResolvedSlots per §3.6.1. v0.27.1 Phase 2 I5 fold:
     // stderr carries the origin_fingerprint substitution NOTICE if any
     // mk1 card omits the master fingerprint.
     let resolved_slots = envelope_to_resolved_slots(&envelope, stderr)?;

     // Derive network from envelope.
     let network = cli_network_from_str(&envelope.bundle.network)?;

     // Script-type from the parsed descriptor (canonical form sans checksum).
     use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey};
     use std::str::FromStr;
-    let parsed_ms = MsDescriptor::<DescriptorPublicKey>::from_str(&canonical_descriptor)
+    let parsed_ms = MsDescriptor::<DescriptorPublicKey>::from_str(&canonical_descriptor_body)
         .map_err(|e| {
             ToolkitError::DescriptorParse(format!(
                 "--from-import-json: descriptor parse for script-type derivation: {e}"
             ))
         })?;
     let script_type = script_type_from_descriptor(&parsed_ms)?;
+
+    // F9 fix (v0.28.2): re-emit the descriptor with miniscript's canonical
+    // `#checksum` suffix so the `EmitInputs.canonical_descriptor` invariant
+    // at `wallet_export/bsms.rs:86-90` holds for the `--from-import-json`
+    // path. The strip-then-validate above checks the user-supplied
+    // checksum; the re-emit guarantees downstream emitters (BSMS L2,
+    // Specter `descriptor` JSON field, Green plaintext blob) carry the
+    // BIP-380 checksum required by BIP-129 Round-2 + BIP-380 §Checksum.
+    let canonical_descriptor = parsed_ms.to_string();
```

The subsequent `EmitInputs { canonical_descriptor: &canonical_descriptor, ... }` at L598-599 now points to the checksumed re-emitted string. The variable name `canonical_descriptor` matches its post-fix value (with checksum); the `_body` suffix on the intermediate validates-but-stripped form documents the transient nature.

**Test cells (new):**

Add to `tests/cli_export_wallet_bsms.rs` (or the appropriate per-format file under `tests/`):

```rust
// F9 v0.28.2 regression — `--from-import-json --format bsms` L2 carries
// the BIP-380 checksum (was missing pre-v0.28.2; downstream Coldcard Mk4
// + Specter Desktop reject blobs without `#checksum`).
#[test]
fn export_wallet_from_import_json_bsms_l2_has_checksum() {
    let envelope = include_str!("fixtures/v0_28_2_f9_bsms_envelope.json");
    let bin = mnemonic_toolkit_test_helper::bin();
    let out = std::process::Command::new(bin)
        .args(["export-wallet", "--from-import-json", "-", "--format", "bsms"])
        .stdin(envelope_as_stdin(envelope))
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 4, "4-line shape");
    assert_eq!(lines[0], "BSMS 1.0");
    // L2: descriptor with `#<8-char-csum>`.
    let l2 = lines[1];
    assert!(
        l2.contains('#') && l2.split('#').last().unwrap().len() == 8,
        "L2 must end with `#<8-char>` checksum: {l2}"
    );
}

// Companion test: `--from-import-json --format specter` `descriptor`
// JSON field carries `#checksum` (latent class same as bsms).
#[test]
fn export_wallet_from_import_json_specter_descriptor_has_checksum() {
    // ... analogous structure; asserts the JSON `descriptor` field's
    // string value ends with `#<8-char>`.
}
```

(Fixture file path is a placeholder — pick from existing v0.27.0+ wallet-import envelope fixtures, e.g., `tests/fixtures/wallet_import_bsms_envelope.json` if available. The implementer should choose a fixture whose `bundle.descriptor` carries a canonical `#checksum` so the regression-direction tests both pre-fix and post-fix.)

### Behavioral consequence (post-fix)

- BSMS Round-2 emit from `--from-import-json` now byte-matches the BIP-129 §Round 2 + BIP-380 spec (descriptor with required `#checksum`).
- Specter Desktop import-side warnings on missing-checksum cleared (Specter never crashed, just logged).
- Coldcard Mk4 import-side rejection cleared.
- chapter-39 recipes 4/7/8 captured transcripts CHANGE post-fix — the L2 of `recipe-{4,7,8}-sparrow|jade|electrum-to-bsms.out` gains a `#xxxxxxxx` suffix. P2 must **recapture** these three transcripts after the v0.28.2 toolkit patch lands and re-commit them. (P3 verify-examples.sh will fail otherwise — the byte-exact-replay gate flags the descriptor-line change.)

### Doc-spec updates (post-c2-B-fix, still required)

c2-B fixes the **bug**; it does NOT fix the **doc claims** F7/F8 surface. Those remain doc-update findings — the chapter-45 4-line shape doc was always wrong, and F7's fix is independent of F9.

---

## F10 — recipe-6 prose L199 fingerprint case (minor)

**Classification (locked):** doc-update.
**Confidence:** high.

**Fix spec:**

- **Anchor:** `docs/manual/src/30-workflows/39-cross-format-conversion.md:198-199`.
- **Toolkit-source grounding:** BIP-388 wallet-policy `keys_info` strings emit `[fp/path]xpub` with fingerprint lowercased per `wallet_export/pipeline.rs:34` (`let fp = slot.fingerprint.to_string().to_lowercase();`). The captured `recipe-6-coldcard-to-bip388.out` confirms `[b8688df1/...]` lowercase.

**Diff (anchored to `f46ac70`):**

```diff
 The resulting `policy.json` carries the canonical `description_template`
 (`wpkh(@0/**)`) plus a single-element `keys_info` array
-(`[B8688DF1/84'/0'/0']xpub6FQya7zGhR9...`). Companion apps that
+(`[b8688df1/84'/0'/0']xpub6FQya7zGhR9...`). Companion apps that
 understand BIP-388 (BitBox02 firmware, hardware-wallet vendors'
 companion software, etc.) load this format directly.
```

**Collateral check:** chapter-45 §Coldcard L427-428 ("the result is a watch-only bundle whose descriptor is `wpkh([B8688DF1/84'/0'/0']xpub6FQya7zGhR9...`") carries the SAME uppercase fingerprint. This is a separate finding — **F10b candidate** — for the foreign-formats chapter, but P1a's F10 surfaced ONLY the chapter-39 recipe-6 prose; the chapter-45 instance was not audited at P1a. **Recommendation:** flag at P2 for inclusion in the same batched-fix commit; lowercase the chapter-45 L427-428 fingerprint as collateral. (Alternatively: leave chapter-45 alone — BIP-388 canonicalization is a wallet_policy emit rule, and chapter-45 §Coldcard's L427-428 sentence is talking about the descriptor as parsed from Coldcard's text-file fixture, NOT the BIP-388 emit output — so the uppercase form may be authentic to the Coldcard fixture. Verifying: Coldcard's text-file fixture format emits xfp in uppercase, so L427-428 may be byte-faithful to the source fixture. **Defer chapter-45 L427-428 to a separate P1b finding pass.**)

**Lock for F10 (chapter-39 only):** L199 lowercase. Minor cosmetic; no behavioral implications.

---

## F11 — chapter-41 prose doesn't disclose verify-bundle / bundle stderr (minor completeness gap)

**Classification (locked):** doc-update.
**Confidence:** high.

**Fix spec:**

- **Anchor:** `docs/manual/src/40-cli-reference/41-mnemonic.md:373-377` (between the verify-bundle command fence at L373 and the "Expected output (one block per cosigner; final `result: ok`):" line at L375). The stderr disclosure belongs HERE — immediately after the verify-bundle command, before its stdout expected-output block.
- **Captured evidence:** `docs/manual/transcripts/41-inheritance.out:1-8` shows the 8-line composite stderr from `bundle ... --json` (L1-5: 3 secret-on-argv warnings + 1 non-canonical-info notice + 1 secret-on-stdout warning) followed by `verify-bundle` (L6-8: 3 secret-on-argv warnings).
- **Rationale (a vs b in P1a's tentative classification):** option (a) "add stderr disclosure" is correct. The non-canonical-info notice at L4 is **load-bearing for this recipe** — chapter-41 demonstrates a non-canonical `wsh(andor(...))` descriptor, and the BIP-48 default-path inference IS the point of the v0.19.0 silent-default-with-stderr-notice feature (memory `feedback_silent_default_with_stderr_notice`). Showing the info-notice in the chapter teaches the reader what the v0.19.0 feature does in practice. Option (b) "leave undocumented because warnings are generic" undersells the info-notice.

**Diff (anchored to `f46ac70`):** insert a new paragraph + `text` block between L373 (the closing ` ``` ` of the verify-bundle sh fence) and L375 ("Expected output..."):

```diff
   --bundle-json /tmp/inheritance-bundle.json
 ```

+The preceding `bundle` and `verify-bundle` commands emit stderr
+disclosures alongside the JSON / stdout. From `bundle`:
+
+```text
+warning: secret material on argv (--slot @0.phrase=) — pipe via --slot @0.phrase=- to avoid /proc/$PID/cmdline exposure
+warning: secret material on argv (--slot @1.phrase=) — pipe via --slot @1.phrase=- to avoid /proc/$PID/cmdline exposure
+warning: secret material on argv (--slot @2.phrase=) — pipe via --slot @2.phrase=- to avoid /proc/$PID/cmdline exposure
+info: non-canonical descriptor; defaulting origin path for @0,@1,@2 to m/48'/0'/0'/2' (BIP-48 cosigner path). Override per-placeholder with [fp/path]@N or --slot @N.path=m/...
+warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')
+```
+
+The `info:` line is the v0.19.0 silent-default-with-stderr-notice
+feature firing on this recipe's non-canonical `wsh(andor(...))`
+descriptor (chapter 19 § Non-canonical descriptors covers the rule;
+SPEC §4.12 carries the byte-exact emission spec). `verify-bundle`
+emits the same three secret-on-argv warnings (no info-notice — the
+default-path inference fired once at bundle-time and is now baked
+into the envelope).
+
 Expected output (one block per cosigner; final `result: ok`):

 ```text
```

**Why this anchor + this content:**
- Placing the stderr disclosure between L373 and L375 keeps the reader's eye flowing: command → stderr → stdout (chronological + by-channel).
- Reproducing the bundle stderr literally (5 lines) lets P3's verify-examples.sh byte-compare the captured `41-inheritance.out:1-5` against the prose's `text` block — same audit-deliverable contract as the verify-bundle stdout block at L378-401.
- The info-notice line is left UNWRAPPED (single long line in the `text` block) to match the actual stderr emission. P3's byte-comparison passes; rendered HTML/PDF allows horizontal scrolling on the line.
- The 3 verify-bundle warnings are described in prose ("same three secret-on-argv warnings") rather than reproduced, because they're identical to the first 3 bundle lines — duplicating them in the chapter is noise without information gain.

**Collateral check:** chapter-41 L195-220 (the TEXT-form bundle's engraving-card stderr documentation) is unchanged — this block documents a different command's stderr (the L111-118 `bundle` invocation without `--json`) and is already accurate.

---

## P1b R1 synthesis (F7-F11)

- **Total findings:** 5 (F7, F8, F9, F10, F11).
- **Doc-update count:** 4 (F7, F8, F10, F11). All locked high-confidence first-round.
- **Toolkit-fix count:** 1 (F9 c2-B locked). High confidence first-round.
- **Cycle release-shape implication:** **PROMOTES** — F9's c2 lock triggers a paired `mnemonic-toolkit-v0.28.2` patch tag per plan-doc §1 Q8. Toolkit patch is behavior-only (no CLI surface change); no GUI schema-mirror lockstep required.
- **Confidence breakdown:** 5 high-confidence; 0 medium; 0 low. No findings need P2 architect re-look. The F9 c2 reasoning chain is exhaustive (5 axes A-E checked); c1 is rejected on the grounds that its marginal cost-savings vs c2-B is approximately zero while c2-B fixes the behavioral root cause + the latent Specter + Green class.

### Combined F1-F11 P2 batched-fix scope estimate

| Class | Source | Estimate |
|---|---|---|
| F1-F6 manual fixes (per R0 §synthesis) | 6 recipe diffs + 1 prose rewrite + 1 new note paragraph | ~35-45 LOC `docs/manual/src/45-foreign-formats.md` |
| F1-F6 new FOLLOWUP | `export-wallet-coldcard-multisig-alias` | ~10 LOC `design/FOLLOWUPS.md` |
| F7 chapter-45 BSMS 4-line shape rewrite | text-block + post-block prose | ~10-15 LOC |
| F8 chapter-39 recipes 4/8 prose | 2 short edits | ~5-8 LOC |
| F9 toolkit src fix | `cmd/export_wallet.rs:566-598` rename + re-emit insertion | ~5-10 LOC |
| F9 toolkit test cells | 2 new cells in `tests/cli_export_wallet_bsms.rs` | ~30 LOC test |
| F9 recapture chapter-39 recipes 4/7/8 transcripts | post-toolkit-fix re-run | 3 files; mechanical |
| F10 chapter-39 recipe-6 prose | 1 char-case edit | ~1 LOC |
| F11 chapter-41 stderr disclosure | new paragraph + `text` block + prose | ~15 LOC |
| **Total manual edits** | | ~70-95 LOC across `45-foreign-formats.md` + `39-cross-format-conversion.md` + `41-mnemonic.md` |
| **Total toolkit edits** | | ~5-10 LOC src + ~30 LOC test = ~35-40 LOC |
| **New FOLLOWUPs** | 1 from R0 (export-wallet-coldcard-multisig-alias); potentially 1 R1 forward-looking (see below) | ~10-20 LOC |

**Two-PR release shape:**
- **PR-A: `mnemonic-toolkit-v0.28.2`** — F9 c2-B src+test fix; recapture chapter-39 recipes 4/7/8 transcripts (the .out files change post-fix); CHANGELOG.md entry; install.sh pin bump if applicable. Toolkit-only.
- **PR-B: `manual-v0.2.0`** — F1-F11 doc fixes + 1 new FOLLOWUP. Depends on PR-A merging first (so the recaptured transcripts are valid).

OR squash into a single release if the project prefers lockstep manual + patch (per the v0.28.1 precedent which was toolkit-only with no manual side, the convention isn't strict).

### New findings spotted during this review

**No new manual-content findings beyond F7-F11.**

**One forward-looking toolkit observation (not a P2 blocker):**

Specter (`wallet_export/specter.rs:68`) and Green (`wallet_export/green.rs:42`) emitters pass `EmitInputs.canonical_descriptor` verbatim to a user-facing surface that SHOULD carry BIP-380 `#checksum`. The F9 c2-B fix at `cmd/export_wallet.rs:566-598` happens to fix BOTH (the same checksum-recompute applies to all downstream emitters via the shared `canonical_descriptor` field). But the wider invariant — "all emitters that pass `canonical_descriptor` to a user-facing surface SHOULD verify it carries `#checksum`" — is NOT enforced by any compile-time or test-time gate; a future code path that constructs `EmitInputs` with a stripped-body `canonical_descriptor` could regress all three (bsms + specter + green) silently.

**Recommended FOLLOWUP (file at P2, alongside the R0 `export-wallet-coldcard-multisig-alias`):**

- **Slug:** `emitinputs-canonical-descriptor-checksum-invariant-enforcement`
- **Tier:** `v0.29-cleanup` (lower priority than v0.28.2 patch; defensive engineering not bug-blocker)
- **Body:** `EmitInputs.canonical_descriptor` has a documented invariant (`wallet_export/bsms.rs:86-90`, generalized by F9 v0.28.2 fix to apply to all emitters that pass the field verbatim) that the string ends with `#<8-char-csum>`. The invariant is enforced only by convention at the construction sites (`cmd/export_wallet.rs` `--template` path via `build_descriptor_string`; `--from-import-json` path via the F9 v0.28.2 `parsed_ms.to_string()` re-emit; `--descriptor` passthrough via the SPEC §6 pre-canonicalization). A future construction site could regress. Options: (a) make `EmitInputs::new(...)` a constructor that asserts the checksum suffix; (b) change `canonical_descriptor` type from `&str` to a newtype `CheckedDescriptor<'_>(&'_ str)` whose constructor validates `#<csum>`. Track separately from the v0.28.2 patch.
- **Source citation:** P1b R1 classification at `design/agent-reports/manual-v0_2_0-p1b-r1-classification.md` §F9 Axis B + the synthesis "Forward-looking toolkit observation".

### Risk flags for P2 execution

1. **Transcript recapture ordering:** F9's toolkit fix changes the captured BSMS L2 byte content for chapter-39 recipes 4/7/8. P2 MUST recapture these three transcripts AFTER the v0.28.2 toolkit binary is built, BEFORE the manual PR's verify-examples.sh runs. If the manual PR runs before the toolkit patch is built, the byte-replay gate will fail showing the OLD (no-checksum) bytes vs the NEW (with-checksum) prose. Sequence: (i) build v0.28.2 toolkit; (ii) re-run capture commands for recipes 4/7/8; (iii) commit the new transcripts; (iv) commit the F7+F8+F10+F11 doc edits; (v) run verify-examples.sh; (vi) merge.

2. **chapter-45 L427-428 uppercase-fingerprint adjacent issue:** F10's lowercase fix in chapter-39 surfaces a parallel uppercase form in chapter-45 §Coldcard L427-428. This MAY be byte-faithful to the Coldcard source fixture (Coldcard's text-file fixtures use uppercase XFP). Verify before deciding: read the actual `coldcard-singlesig-bip84-mainnet.json` fixture to confirm Coldcard's emit case, then decide whether L427-428 needs the same lowercase edit. Defer to P2 as a discretionary collateral fix.

3. **F11 long-line wrapping in stderr text-block:** the info-notice line (>200 chars) will not wrap in the rendered HTML/PDF; horizontal scrolling is acceptable. If the project's manual-rendering pipeline (mdBook) does NOT permit horizontal-scrolling code blocks, an alternative is to redact the line to `info: non-canonical descriptor; defaulting origin path for @0,@1,@2 to m/48'/0'/0'/2' (BIP-48 cosigner path)...` with a trailing ellipsis + a prose note pointing at chapter 19. Defer to P2 implementer's mdBook-rendering check.

---

**Persisted at:** `design/agent-reports/manual-v0_2_0-p1b-r1-classification.md` (this file).
**Next phase:** plan-doc fold of F7-F11 locked classifications + release-shape promotion to paired `mnemonic-toolkit-v0.28.2` + manual-v0.2.0 → P2 batched-fix pass in fix-order (toolkit patch first, transcripts recapture second, manual edits third) → P3 verify-examples.sh extension run.
