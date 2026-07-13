# SPEC — `mnemonic verify-bundle` non-chunked md1 intake + content-id canonicalization

**FOLLOWUP:** `toolkit-inspect-nonchunked-md1-intake-gap` — **verify-bundle leg** (`design/FOLLOWUPS.md:34-38`, the `⚠️ RESIDUAL` note). The inspect leg shipped v0.89.0.
**Companion recon:** `cycle-prep-recon-verify-bundle-nonchunked-md1-canonicalization.md`.
**Sibling precedent:** `design/SPEC_inspect_nonchunked_intake.md` (the inspect leg — same discriminator, mirrored strictly for the classify gate).
**Design-R0 (GREEN, 0C/0I):** `design/agent-reports/verify-bundle-nonchunked-canonicalization-designr0-round-1.md` (Fable architect, main-loop-verified). This SPEC folds its 6 Minors.
**Source SHA:** `de140a08` (`origin/master`, HEAD == origin/master; all citations grep-verified at write time — re-grep at plan/impl time, they decay every merge).
**Toolkit version at recon:** `v0.89.0`; target ship **`v0.90.0`** (MINOR — parity with the v0.89.0 inspect leg; additive intake, no flag change).
**md-codec:** vendored + pinned `0.42.0` (`crates/mnemonic-toolkit/Cargo.toml:34`); public API used (`decode_md1_string`, `compute_md1_encoding_id`) — **NO codec bump**.
**Status:** SPEC — folded design-R0 (6 Minors, GREEN) + specr0 **round-1** (RED: 1 Important `I-1` test-plan defect + 3 Minors; `design/agent-reports/verify-bundle-nonchunked-canonicalization-specr0-round-1.md`). Round-1 folds: §3.2b (what `md1_template_match` verifies) + §4 truth-table rows + §6.3 test #7 re-spec (PROBATIVE INV-4 anchor) + §3.1 subset wording (M-1) + §6.1 test #1 `--mk1` (M-2) + line-cite (M-3). Specr0 **round-2 GREEN — 0C/0I** (`design/agent-reports/verify-bundle-nonchunked-canonicalization-specr0-round-2.md`); its 3 cosmetic Minors (M-3 second cite, §3.2b `:119-123` cite, test-#7 wording) folded inline. **SPEC R0-GREEN — cleared for implementation-plan phase.**

---

## 0. Scope

### IN
- Broaden `verify-bundle`'s SUPPLIED-`--md1` intake so a **non-chunked single-string** template md1 (bare `md encode` form) **classifies** and routes into `verify_singlesig_template` / `verify_multisig_template` instead of falling through the chunk-form-only classify gate (**Facet 1**).
- Replace the single-sig template path's **raw `Vec<String>` string-equality** compare with a **content-id** compare (`compute_md1_encoding_id`), so a non-chunked template md1 that decodes to the same descriptor as the toolkit-synthesized (chunk-form) expected card **verifies** (**Facet 2**).

### OUT (explicit carve-outs)
- **OUT-1: keyed / wallet-policy-form md1 intake.** *Structurally impossible to be non-chunked* (see §5 INV-KEYED): one `Pubkeys` TLV entry is 65 bytes = 520 bits and a single codex32 regular string caps at 80 data symbols = **400 payload bits** (`vendor/md-codec/src/codex32.rs:25` `REGULAR_DATA_SYMBOLS_MAX = 80`). A keyed md1 MUST be chunked. Excluding it is a **proof of completeness**, not a gap.
- **OUT-2: corrupted / partial-decode paths.** The classify gate stays **strict** (`DecodeOpts::default()`, NOT `partial()`); a dead/pathless/corrupted non-chunked card still fails classify and falls through exactly as today. The general path's `partial()` decode (`verify_bundle.rs:2508-2510`, `:3113`) and `md1_partial::supplied_md1_unresolved_indices` (`md1_partial.rs:55-63`) stay **chunk-only** — untouched. Their non-chunked-keyless-dead-card `mismatch`-vs-`partial` asymmetry (both exit 4, fail-closed) is a **filed residual** (FOLLOWUP `verify-bundle-nonchunked-deadcard-verdict-asymmetry`), owned by a future partial-decode cycle.
- **OUT-3: the multisig template compare.** Already content-id-based (`compute_wallet_descriptor_template_id` bytes, `verify_bundle.rs:937-941`); it needs only Facet 1's shared classify-gate fix (free ride) — **no compare change**.
- **OUT-4: any codec change.** `decode_md1_string` and `compute_md1_encoding_id` already exist + are re-exported in md-codec 0.42.0 (`vendor/md-codec/src/lib.rs:51-61`).
- **OUT-5: any clap-flag / `--json` wire-shape / dropdown change.** Purely behavioral intake + compare-canonicalization. No new `--json` `result` state; a mismatch is still `md1_template_match:false → "mismatch"`/exit 4.

---

## 1. Background — where and why it fails today (grep-verified @ `de140a08`)

| Fact | Location | Detail |
|---|---|---|
| Sole classify gate | `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:386-408` | `if let Ok(d) = md_codec::chunk::reassemble(&md1_refs)` — chunk-form only; routes `verify_singlesig_template` (`:394`) / `verify_multisig_template` (`:405`). Only call sites of both (defs `:591`, `:859`). |
| Misread mechanism | `vendor/md-codec/src/chunk.rs:68-72` | `ChunkHeader::read` reads a 4-bit version; a non-chunked first symbol `[divergent][v3v2v1v0=0100]` lands on `2` → `Err(WireVersionMismatch{got:2})`, swallowed by `if let Ok`. |
| Non-chunked layout | `vendor/md-codec/src/header.rs:27,31` | first symbol `[divergent][v3][v2][v1][v0]`, version `4`; NOT the chunk header's `[v3][v2][v1][v0][chunked]`. |
| Fall-through outcome (a) | `verify_bundle.rs:435-443` | without `--template` (normal template-card invocation): a valid non-chunked card falls past the short-circuit → `ModeViolation` exit 2 (`error.rs:637`). |
| Fall-through outcome (b) | `verify_bundle.rs:2508-2510`, `:3113` | with `--template` + slots: general path decodes supplied md1 chunk-only (`reassemble_with_opts(..., partial())`) → md1-decode FAIL → `mismatch`/exit 4 on a valid card. |
| Single-sig raw compare | `verify_bundle.rs:696` | `let md1_match = expected.md1 == args.md1;` — raw `Vec<String>` equality; `expected.md1` = `synthesize_unified(..., Md1Form::Template)` chunk-form (`:687-695`), `args.md1` = supplied `Vec<String>`. |
| Multisig compare (already id-based) | `verify_bundle.rs:937-941` | `completed_template_id.as_bytes() == supplied_template_id.as_bytes()` via `compute_wallet_descriptor_template_id`. Model for Facet 2's posture. |
| `d` already decoded in single-sig fn | `verify_bundle.rs:591-598` | `verify_singlesig_template(d: &md_codec::Descriptor, …)`; `&d` passed at `:394`. No re-decode needed for Facet 2. |
| Expected is well-formed chunk-form | `synthesize.rs:29`, `:1232` | `Bundle.md1: Vec<String>` from `chunk::split(&template)` (always chunk-form, `count>=1`). Precedent treats reassemble infallible: `.expect("expected bundle is well-formed")` at `verify_bundle.rs:2904`, `:3131`. |

## 2. The intake discriminator + the content-id (normative)

### 2.1 Discriminator (identical to the inspect leg — `SPEC_inspect_nonchunked_intake.md §2`)
md-codec's `decode_md1_string(s)` = `decode_md1_string_with_opts(s, DecodeOpts::default())` (`vendor/md-codec/src/decode.rs:178-196`) dispatches on the **in-band chunked-flag bit** (bit 3 of byte 0):
- `unwrap_string` verifies the codex32 BCH checksum **before** the flag is read (`codex32.rs:139-195`) — corruption can never re-route.
- `chunked_flag == 1` → routes internally to `reassemble_with_opts(&[s], opts)` — so a **chunked-of-1 single string is byte-identical to today's `reassemble(&[s])`** (`chunk::reassemble ≡ reassemble_with_opts(default)`, `chunk.rs:311-313`).
- `chunked_flag == 0` → `decode_payload_with_opts` (non-chunked single payload).

### 2.2 Content-id: `compute_md1_encoding_id` (`vendor/md-codec/src/identity.rs:39-45`)
`id = SHA-256(encode_payload(d) bytes)[0..16]`. `encode_payload` canonicalizes **only** placeholder-index ordering (`encode.rs:99-102`); `path_decl` is written **verbatim** (`origin_path.rs:54-66`) — **no** origin-elision canonicalization at encode time. Therefore the id is:
- **invariant to** encoding-form only: chunk boundaries, HRP case, BCH checksum bytes (two chunkings of the same payload decode to the same `Descriptor` → same id).
- **sensitive to** everything semantic: tree, use-site paths, origin-path (explicit-vs-elided **and** account index), fingerprints, keys.

The decoder rejects non-canonical placeholder ordering (`validate.rs:27-35` `PlaceholderFirstOccurrenceOutOfOrder`), so every decoded `Descriptor` is already canonical and `encode_payload` is a faithful re-serialization: **id equality ⇔ identical canonical payload ⇔ identical descriptor content** (up to a 128-bit truncated-SHA-256 collision — computationally infeasible). In this path `d.n == 1`, so placeholder canonicalization is the identity map anyway.

## 3. The change (normative contract)

### 3.1 Facet 1 — length-dispatch the classify gate (`verify_bundle.rs:388`), STRICT

```rust
if !args.md1.is_empty() {
    let md1_refs: Vec<&str> = args.md1.iter().map(|s| s.as_str()).collect();
    // A single supplied md1 may be a NON-chunked single-payload string (bare
    // `md encode` form) OR a chunked-of-1 string. decode_md1_string auto-
    // dispatches on the in-band chunked-flag bit (decode.rs:187-196) and routes
    // chunked strings back through reassemble — so a chunked-of-1 input is
    // byte-identical to today's reassemble(&[s]). Multi-chunk keeps reassemble
    // verbatim. STRICT (default opts, NOT partial): the classify gate must
    // preserve today's routing — a dead/pathless/corrupted non-chunked card
    // still fails decode here and falls through, exactly as before. Partial-
    // decode stays the general path's job (OUT-2).
    let classify = match md1_refs.as_slice() {
        [single] => md_codec::decode_md1_string(single),
        _        => md_codec::chunk::reassemble(&md1_refs),
    };
    if let Ok(d) = classify {
        // …unchanged: is_singlesig_template → verify_singlesig_template(&d, …)
        //             is_multisig_template  → verify_multisig_template(&d, …)
    }
}
```

This one change fixes classify-routing for **both** single-sig and multisig **template** cards (a keyless multisig template can fit a single non-chunked string; a keyed md1 cannot — OUT-1). Strict is safe: `validate_explicit_origin_required` is a no-op when `canonical_origin(&d.tree).is_some()` (`validate.rs:221-224`); single-sig templates are a **subset** of the canonical-elidable types (`synthesize.rs:1132-1141`, all `canonical_origin=Some`; the converse fails — `sh(wpkh)` is canonical-elidable but has no `cli_template_from_tree`/`template_admissible` arm, `synthesize.rs:1120-1125` — and subset is the safe direction: every *emittable* single-sig template still has `canonical_origin=Some`), and the emitter elides origins **only** under `canonical_origin=Some` (C1-conditional mutation 3, `synthesize.rs:1205-1218`) — a general/non-canonical template **keeps explicit origins on the wire**, so no emittable template card trips `MissingExplicitOrigin`.

### 3.2 Facet 2 — content-id compare in `verify_singlesig_template` (`verify_bundle.rs:696`)

```rust
// was: let md1_match = expected.md1 == args.md1;
let expected_md1_refs: Vec<&str> = expected.md1.iter().map(String::as_str).collect();  // Minor #5 (Vec<String>→&[&str], mirror :2902)
let d_expected = md_codec::chunk::reassemble(&expected_md1_refs)?;                       // expected is toolkit-generated, well-formed
let md1_match = md_codec::compute_md1_encoding_id(d)?                                    // d: &Descriptor param (already decoded, :591)
    == md_codec::compute_md1_encoding_id(&d_expected)?;
```

- `d` is the already-decoded supplied card (Facet 1's classify output) — **no re-decode**.
- `?`-propagation mirrors the multisig path (`:937-940`). Both id-computes are effectively infallible here (a card that passed strict decode's validator gauntlet cannot fail `encode_payload`'s subset — design-R0 Q8); an error would be an internal codec-invariant break → `ToolkitError::MdCodec` → nonzero exit (`error.rs:264,509-516,635`), **never a false pass**. A hard error (not a silent "mismatch") is the correct disposition (do not mislabel a toolkit bug as a card defect). **A mismatch never `?`-errors** — id-compute succeeds and equality merely differs, so `md1_match=false` flows to the existing check (§3.3).

### 3.2b What `md1_template_match` actually verifies (semantic clarity — specr0 I-1)

The expected bundle is re-synthesized from the supplied card's **own** classified type (`cli_template_from_tree(&d.tree)` → `synthesize_unified`, `verify_bundle.rs:602,:687-695`), and `cli_template_from_tree` keys only on `(tag, body-variant)` (`synthesize.rs:369-377`). So `md1_template_match` is a **canonicality check on the card's own type**: it asserts the supplied card is the *canonical encoding* of its `(tag, body)` family — NOT that it matches an externally-specified wallet. Two consequences the test plan must respect:
- The md1 is keyless/seed-/network-agnostic, so a "wrong seed" or "wrong-network" *genuine* template card **still passes `md1_template_match`** (its descriptor content is unchanged). **Cross-wallet / wrong-seed rejection is carried elsewhere** — `mk1_template_stub_bind` (`:697-700`), the recompose display, and `--expect-wallet-id` — never by this md1 compare.
- The only way `md1_template_match` fails is a card whose **encoding differs from the canonical synthesis of its own type**: a non-canonical use-site path, a doctored/explicit origin, or an extra TLV (`Fingerprints`) — each enters `encode_payload` (`encode.rs:119-123`: origin `:119`, use-site `:120`, TLV `:123`) and shifts `compute_md1_encoding_id`. Facet 2 preserves this exactly (byte-identity did the same via string bytes); it only additionally tolerates chunk-form/HRP/checksum re-encodings of the *same* descriptor (INV-3).

### 3.3 What stays UNCHANGED (verify, do not re-implement)
- `md1_template_match` `VerifyCheck` (`:702-711`) — `md1_match` feeds `passed` verbatim; verdict routing `any_fail → "mismatch"`/exit 4, else `"ok"`/exit 0 (`:796-831`). **No new `--json` `result` state.**
- `mk1_template_stub_bind` (`:697-700,:712-721`) stays a raw byte-compare (different codec, `mk1`). Facet 2's form-tolerance is md1-only by design — pinned by a negative test (§6, Minor #6).
- The multisig template path compare (`:937-941`) — unchanged (already id-based).
- `--origin`/`--account` recompose + `--expect-wallet-id` (`:634-793`) — unchanged; account correctness is carried here, **not** by the card (§4 note).

## 4. Behavioral truth table (single-sig template path unless noted)

| Supplied `--md1` | Today | After | Note |
|---|---|---|---|
| Non-chunked single-sig template md1, matches expected | reject: exit 2 (no `--template`) / false `mismatch`+exit 4 (`--template`) | **`ok`, exit 0** | the fix (Facet 1 routes + Facet 2 matches) |
| Chunked (of-1 or multi) template md1, matches expected | `ok`, exit 0 | `ok`, exit 0 | **byte-identical**: string-equal ⟹ same descriptor ⟹ same id (no-regression) |
| Non-chunked template md1 with a **non-canonical encoding of its own type** (non-standard use-site / doctored explicit origin / extra `Fingerprints` TLV) | reject (form) | **`md1_template_match` mismatch, exit 4** | funds-negative: id-compare rejects a non-canonical encoding of the same template family (INV-4) |
| Non-chunked template md1 for a **wrong seed / wrong network** (genuine card) | reject (form) | **`md1_template_match` PASSES**; overall verdict carried by `mk1_template_stub_bind` / recompose / `--expect-wallet-id` | the md1 is keyless/seed-agnostic — cross-wallet rejection is not the md1 compare's job (§3.2b) |
| Non-chunked **multisig** template md1 (+`--from`) | reject (classify fall-through) | **routes to `verify_multisig_template`** → WDT-id compare | Facet 1 free ride (OUT-3) |
| Non-chunked multisig template md1, **no** `--from` | reject | refuse naming `--from` (floor `:876-892`) | parity with chunked |
| Non-chunked **dead**/pathless/corrupted card | reject/fall-through | reject/fall-through (strict classify) | **unchanged** (OUT-2) |
| Non-chunked **keyed** md1 | n/a (structurally impossible) | n/a | OUT-1 / INV-KEYED |

**Account note (design-R0 Minor #1 — corrected rationale).** Genuine single-sig template cards are **origin-ELIDED and byte-identical across accounts** (`verify_bundle.rs:634-635`); account correctness is carried by `--account`/`--origin` at recompose, not by the card. So today's byte-compare, WDT-id, **and** `compute_md1_encoding_id` all match a "wrong-account" *genuine* card equally — that is not the discriminator. `compute_md1_encoding_id` is chosen over WDT-id because it stays **strict against hand-crafted explicit-origin / doctored keyless cards** (which WDT-id would blur by dropping origins/fingerprints); for like-for-like keyless-template comparison it is the faithful minimal relaxation of the current byte-identity.

## 5. Funds-safety invariants (normative — test-anchored)

- **INV-1 (structural dispatch, never a fallback).** Facet 1 dispatches on `md1_refs.len()` + md-codec's in-band chunked-flag bit. No try-one-then-the-other/catch-and-retry. A genuine decode error from either branch propagates (via `if let Ok` → fall-through, exactly as today).
- **INV-2 (BCH-before-flag → corruption can't re-route).** `unwrap_string` verifies BCH before the chunked-flag read (`codex32.rs:139-195`); a first-symbol corruption fails the checksum → reject, never diverts chunked↔non-chunked.
- **INV-3 (no-regression — content-id is a strict superset of byte-identity).** Identical supplied/expected strings ⟹ identical descriptor ⟹ identical `encode_payload` ⟹ identical id. Every current byte-compare **pass** stays a pass; the only NEW passes are "different string, same decoded descriptor" = chunk/HRP-case/checksum form-equivalence = same wallet.
- **INV-4 (no new acceptance of a non-canonical encoding — anchored by tests §6.3 #7/#8).** `compute_md1_encoding_id` is sensitive to tree/use-site/origin/fingerprints/keys (§2.2); a supplied card whose encoding differs from the canonical synthesis of its own `(tag,body)` type in ANY of these still `md1_template_match`-mismatches. The relaxation is form-only (chunk/HRP/checksum re-encoding of the SAME descriptor). Cross-wallet/seed rejection is a separate mechanism (mk1-stub-bind + recompose + `--expect-wallet-id`, §3.2b), not this compare's job.
- **INV-5 (strict classify preserves fail-closed routing).** Facet 1 uses `DecodeOpts::default()`; a dead/pathless/corrupted non-chunked card fails classify and falls through as today — no accidental partial-decode broadening (OUT-2).
- **INV-KEYED (keyed non-chunked is structurally impossible — completeness proof).** 65-byte pubkey TLV = 520 bits > the 400-bit single-string cap (`codex32.rs:25`, enforced encode `:89` / decode `:174`). No non-chunked keyed md1 exists → OUT-1 leaves no reachable gap.
- **INV-6 (id-compute error ⟹ hard fail, never a false pass).** `?` on either id-compute lands as `MdCodec` → nonzero exit; a mismatch never triggers it (equality differs, compute succeeds).

## 6. Test plan (TDD — RED before GREEN; CLI-level via `Command::cargo_bin("mnemonic")` unless noted)

Fixtures use never-fund keys / the frozen KAT corpus (reuse the `cli_verify_bundle*.rs` + `cli_inspect.rs:216-228` template corpus). Construct non-chunked single strings via `md_codec::encode_md1_string` (or decode-then-`encode_md1_string` a corpus chunk-form card).

### 6.1 RED-proofs (fail on `origin/master`, pass after)
1. **`verify_bundle_nonchunked_singlesig_template_ok`** — a non-chunked single-sig template md1 that matches the seed/type: `verify-bundle --md1 <single> --mk1 <cards> --slot @0.…=<seed> [--template …]` → **exit 0 / `ok`**, `md1_template_match` passed. **The `--mk1` cards are required** (else `mk1_template_stub_bind` fails → exit 4 regardless of md1) — the harness `verify_args` (`tests/cli_verify_bundle_md1_template.rs:88-91`) always supplies them. RED today = exit 2 / false `mismatch`.
2. **`verify_bundle_nonchunked_singlesig_json_ok`** — `--json`: `result:"ok"`, `mode:"single-sig-template"`, `md1_template_match.passed:true`. Byte-shape identical to the chunked verify of the same descriptor.
3. **`verify_bundle_nonchunked_multisig_template_routes`** — a non-chunked **keyless multisig** template md1 (+`--from`): routes to `verify_multisig_template`, WDT-id `md1_template_match` passes. RED today = classify fall-through.

### 6.2 No-regression locks (GREEN before AND after — INV-3)
4. **`verify_bundle_chunked_template_still_ok`** — the existing chunk-form single-sig template verify stays `ok`/exit 0 (byte-identical verdict).
5. **`verify_bundle_form_equivalence_same_verdict`** — the SAME descriptor supplied as (a) non-chunked and (b) `split()` chunk-form yields an **identical** verdict + `--json` shape (proves form-equivalence, INV-3).
6. **`verify_bundle_multichunk_template_unchanged`** — a multi-chunk template set verifies exactly as today (Facet 1's `_ =>` arm is verbatim `reassemble`).

### 6.3 Funds-negative (the relaxation must still reject — INV-4)
7. **`verify_bundle_nonchunked_noncanonical_encoding_mismatch`** (PROBATIVE INV-4 anchor — specr0 I-1) — a non-chunked keyless single-sig card that STILL classifies as its `(tag,body)` type but whose **encoding differs from the canonical synthesis**. Construct EITHER: (a) a wpkh card with a **non-standard use-site path** (`cli_template_from_tree` matches `(Tag::Wpkh, Body::KeyArg)` regardless of use-site, `synthesize.rs:374`; `use_site_path` enters `encode_payload`, `encode.rs:120`), OR (b) a keyless card with a **retained `Fingerprints` TLV** (decodes fine — `validate_xpub_bytes` is a no-op with no pubkeys, `validate.rs:255-258`; TLV bits enter the id). Assert **`md1_template_match.passed == false`** in `--json` (→ `mismatch`/exit 4). This test stays GREEN **only if the compare is content-sensitive** — a broken `md1_match = true` regression makes it FAIL, which is the whole point. ⚠️ A "wrong seed" card does **NOT** work here: the md1 is seed-agnostic so `md1_template_match` would PASS (see §3.2b / truth-table); cross-wallet rejection is anchored separately by test #10 (mk1) + the recompose display.
8. **`verify_bundle_nonchunked_doctored_origin_stricter_than_wdt`** — a hand-crafted non-chunked keyless card with an **explicit non-canonical origin** that a WDT-id compare would blur (WDT-id excludes origins, `identity.rs:48-53`) → `compute_md1_encoding_id` still `mismatch`es. Anchors the §4/Minor-#1 rationale that the encoding-id is stricter than WDT-id. Realizable: `validate_explicit_origin_required` is a no-op for canonical trees (`validate.rs:221-224`), so the doctored card strict-decodes + classifies.

### 6.4 Fail-closed / carve-out parity (INV-5, OUT-2)
9. **`verify_bundle_nonchunked_dead_card_falls_through`** — a non-chunked dead/pathless card is NOT accepted as a template (strict classify): same outcome as its chunked twin's non-template handling (exit ≠ 0; no `ok`).
10. **`verify_bundle_mk1_tolerance_not_extended`** (Minor #6) — with a matching md1 but a **case-variant mk1**, `mk1_template_stub_bind` still fails → `mismatch`; pins that Facet 2's form-tolerance is md1-only.

### 6.5 Full-suite requirement
Per `feedback_r0_review_run_full_package_suite`: per-phase R0 runs the FULL `cargo test -p mnemonic-toolkit` suite (an intake/compare change can ripple into argv/schema/version lints even with no flag change).

## 7. SemVer + lockstep
- **SemVer:** **MINOR** → **`v0.90.0`**. Additive intake capability + a strictly-superset compare relaxation; every previously-accepted input keeps its verdict, only a previously-rejected valid form is now accepted (design-R0 Minor #4 — write the migration note like v0.89.0: "a non-chunked template md1, previously exit 2 / false-mismatch, now returns its real verdict").
- **GUI `schema_mirror`:** **NOT triggered** — no clap flag/subcommand/dropdown add/remove/rename (gate is flag-NAME parity). No `mnemonic-gui/src/schema/mnemonic.rs` change.
- **`--json` wire-shape:** **unchanged** — same `single-sig-template` shape; no new `result` state (no `partial`/exit-4-new). No GUI wire drift; the resolved exit-4 VERIFY-ME badge class is untouched.
- **Manual mirror:** flag-coverage lint mirrors `--help`; no flag changed → **no lockstep**. OPTIONAL, non-gated: a one-line note in the verify-bundle chapter that a non-chunked md1 is accepted (R0's call; docs-only same-PR if added).
- **Codecs:** md/mk/ms/wc **NO-BUMP** (existing md-codec 0.42.0 public API).
- **Release version sites** (`project_toolkit_release_ritual_version_sites`): BOTH READMEs, `fuzz/Cargo.lock`, `scripts/install.sh` self-pin, `vendor/` (no dep bump → no re-vendor), CHANGELOG (gated by `changelog-check` on the tag), `.examples-build/` (no example-output change → confirm unaffected).
- **Filed residual (this run):** FOLLOWUP `verify-bundle-nonchunked-deadcard-verdict-asymmetry` (Minor #3) — file in the shipping commit's doc-ripple; parked on a future partial-decode cycle (NOT burned down — scoped OUT-2).

## 8. Implementation checklist (single phase; TDD)
1. Write RED tests §6.1 (+ locks §6.2, negatives §6.3, parity §6.4) → confirm RED on `origin/master`.
2. Apply §3.1 classify-gate length-dispatch (STRICT) + §3.2 single-sig content-id compare + the §3.2 `Vec<String>→&[&str]` conversion (Minor #5).
3. GREEN the FULL package suite (`cargo test -p mnemonic-toolkit`).
4. Version bump `v0.90.0` + all §7 version sites + CHANGELOG (migration + keyed-exclusion completeness note, Minors #2/#4).
5. Post-implementation independent adversarial whole-diff review (mandatory, non-deferrable) — Fable, persist verbatim to `design/agent-reports/`.
6. Flip the FOLLOWUP: mark the **verify-bundle leg RESOLVED** (whole `toolkit-inspect-nonchunked-md1-intake-gap` now closed); **file** the new `verify-bundle-nonchunked-deadcard-verdict-asymmetry` residual — same shipping commit (`feedback_followup_status_discipline`).

## 9. R0 convergence questions (design-R0 already GREEN; confirm the folds landed)
1. **Fold fidelity:** do §4's corrected account-rationale (Minor #1), §5 INV-KEYED (Minor #2), §7 migration note (Minor #4), §3.2 type-conversion (Minor #5), and §6.4 test #10 (Minor #6) each land the design-R0 Minor faithfully, with no new Critical/Important introduced by the written contract?
2. **Strict-classify safety restated:** does §3.1's strict-`decode_md1_string` choice remain sound for ALL emittable template shapes (single-sig canonical-elidable + keyless multisig general-with-explicit-origins)?
3. **Compare posture:** is §3.2's `?`-propagate-both (vs `.expect` on the expected side like `:2904`) the right call, and is the "mismatch never `?`-errors" claim airtight?
4. **Residual disposition:** confirm filing (not burning down) the dead-card verdict-asymmetry residual is the correct scope call under OUT-2.
