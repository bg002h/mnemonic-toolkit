# SPEC — `mnemonic inspect` non-chunked single-string md1 intake

**FOLLOWUP:** `toolkit-inspect-nonchunked-md1-intake-gap` (`design/FOLLOWUPS.md:33-35`).
**Companion brainstorm:** `design/BRAINSTORM_inspect_nonchunked_intake.md`.
**Source SHA:** `a528eba5` (`origin/master`; all citations grep-verified at write time — re-grep at plan/impl time, they decay every merge).
**Toolkit version at recon:** `v0.88.0`; target ship **`v0.89.0`** (MINOR).
**md-codec:** vendored + pinned `0.42.0` (`crates/mnemonic-toolkit/Cargo.toml:34`); public API used — **NO codec bump**.
**Status:** SPEC — pre-R0. This document goes through the mandatory opus R0 gate (0C/0I) BEFORE any implementation.

---

## 0. Scope

### IN
- Broaden `mnemonic inspect`'s md1 intake so a **non-chunked single-string** md1
  (bare `md encode` output, e.g. `md encode 'wpkh(@0/<0;1>/*)'`) decodes and
  renders like any other card, instead of failing `unsupported version 2` /
  exit 3.
- Preserve, byte-for-byte, the existing chunk-form intake (chunked-of-1 and
  multi-chunk) and its partial-decode / exit-4 dead-card behavior.

### OUT (explicit carve-outs)
- **OUT-1: `verify-bundle` non-chunked intake.** Its SUPPLIED-card handling is
  entangled with **string-equality** comparison (`verify_bundle.rs:696`
  `expected.md1 == args.md1`) against toolkit-synthesized **chunk-form** cards;
  broadening intake alone does not make a non-chunked template md1 verify (see
  brainstorm §2). Stays on the FOLLOWUP as remaining scope, with the
  string-compare canonicalization noted as the blocking design question.
- **OUT-2: `repair` / `restore`.** `repair` already handles non-chunked md1 via
  `md_codec::decode_with_correction` single-string auto-dispatch
  (`repair.rs::is_non_chunked_md1:736`); no change here. `restore` intake is out
  of scope.
- **OUT-3: any codec change.** The discriminator and both decode entries already
  exist in md-codec 0.42.0.
- **OUT-4: any clap-flag / `--json` wire-shape / dropdown change.** This is a
  purely behavioral intake broadening.

---

## 1. Background — where and why it fails today (grep-verified)

| Fact | Location | Detail |
|---|---|---|
| Intake site | `crates/mnemonic-toolkit/src/cmd/inspect.rs:242-245` | `CardKind::Md1 => Ok(InspectPayload::Md1(md_codec::reassemble_with_opts(chunks, DecodeOpts::partial())?))` — chunk-form only |
| In-source note of the gap | `inspect.rs:239-241` | "Intake is CHUNK-FORM only … a plain single-string md1 still hits the pre-existing `unsupported version 2` gap" |
| Misread | `vendor/md-codec/src/chunk.rs:68-72` | `ChunkHeader::read` reads a 4-bit version; on a single-payload first symbol it lands on `2` → `Error::WireVersionMismatch { got: 2 }` |
| Single-payload layout | `vendor/md-codec/src/header.rs:1-10, 38-49` | first symbol `[divergent][v3][v2][v1][v0]`, version `4`; NOT the chunk header's `[v3][v2][v1][v0][chunked]` |
| Error remap | `crates/mnemonic-toolkit/src/error.rs:1054-1057` | `WireVersionMismatch { got }` → `ToolkitError::FutureFormat { detail: "unsupported version {got}" }` |
| Exit code | `crates/mnemonic-toolkit/src/error.rs:586-587` | `WireVersionMismatch` → **3** |
| Test workaround proving the gap | `crates/mnemonic-toolkit/tests/cli_inspect.rs:211-239` | single-string fixtures are `rechunk()`'d (decode→split→reassemble) because inspect is reassemble-only |

## 2. The intake discriminator (normative)

md-codec's `decode_md1_string_with_opts(s, opts)`
(`vendor/md-codec/src/decode.rs:187-196`) is the dispatching entry:

```rust
let (bytes, symbol_aligned_bit_count) = crate::codex32::unwrap_string(s)?;   // (i) BCH verified HERE
let chunked_flag = bytes.first().map(|b| (b >> 3) & 0x01).unwrap_or(0);      // (ii) in-band flag
if chunked_flag == 1 {
    return crate::chunk::reassemble_with_opts(&[s], opts);                   // chunked-of-1 → oracle path
}
decode_payload_with_opts(&bytes, symbol_aligned_bit_count, opts)            // non-chunked single payload
```

- **Discriminator** = the chunked-flag bit (bit 3 of byte 0 = LSB of the first
  5-bit symbol). Single-payload usable version set `{4, 8, 12}` is all-even
  (LSB 0); the chunk header always sets `chunked = 1` (`chunk.rs:53`). The two
  wire forms are **unambiguous** — a structural bit, **not** a
  try-one-then-the-other heuristic.
- **(i) precedes (ii):** `unwrap_string` verifies the codex32 BCH checksum
  (`vendor/md-codec/src/codex32.rs:183-184`, "BCH checksum verification failed")
  **before** the flag is read. A corrupted first symbol that flips the flag fails
  the checksum → reject. Corruption can never re-route.
- **`opts` threads through** to whichever layer performs the origin check, so
  `DecodeOpts::partial()` reaches `decode_payload_with_opts` for the non-chunked
  case → dead-card partial-decode composes with no extra code.

## 3. The change (normative contract)

### 3.1 Single site: `inspect.rs::decode_card`, `CardKind::Md1` arm

Replace `inspect.rs:242-245` with a length-dispatched decode:

```rust
CardKind::Md1 => {
    // A single supplied md1 may be a NON-chunked single-payload string (bare
    // `md encode` form) OR a chunked-of-1 string. decode_md1_string_with_opts
    // auto-dispatches on the in-band chunked-flag bit (decode.rs:187-196) and
    // routes chunked strings back through reassemble_with_opts — so a
    // chunked-of-1 input is byte-identical to today. Multi-chunk sets keep the
    // reassemble path verbatim. opts=partial threads to whichever layer runs
    // the origin check, inheriting the dead-card exit-4 behavior unchanged.
    let d = if chunks.len() == 1 {
        md_codec::decode_md1_string_with_opts(chunks[0], md_codec::DecodeOpts::partial())?
    } else {
        md_codec::reassemble_with_opts(chunks, md_codec::DecodeOpts::partial())?
    };
    Ok(InspectPayload::Md1(d))
}
```

`md_codec::decode_md1_string_with_opts` is re-exported at the crate root
(`vendor/md-codec/src/lib.rs:51-54`). `chunks: &[&str]` in `decode_card`, so
`chunks[0]: &str` matches the `s: &str` parameter.

### 3.2 Update the stale in-source comment

Rewrite the `inspect.rs:232-241` note that currently says intake is chunk-form
only, to describe the new length-dispatch and cite this SPEC.

### 3.3 What downstream stays UNCHANGED (must be verified, not re-implemented)

- `partial_indices` capture (`inspect.rs:152-157`), the `emit_inspect_text` /
  `emit_inspect_json` render paths, `ORIGIN_UNSPECIFIED_MARKER`
  (`md1_partial.rs:21-22`), the exit-4 dead-card return (`inspect.rs:175-178`),
  and `INSPECT_SCHEMA_VERSION = "2"` (`inspect.rs:349`) are all **unchanged**. A
  non-chunked dead card produces a non-empty `unresolved_origin_indices()` exactly
  like a chunked dead card, so the existing render + exit-4 logic fires with no
  edit.
- The auto-fire path (`inspect.rs:130-145`) is unchanged: a *corrupted*
  non-chunked md1 that fails `decode_md1_string_with_opts` is an `MdCodec`-class
  error → falls to `try_repair_and_short_circuit` → `decode_with_correction`
  (which itself single-string-dispatches). Compose, don't touch.

## 4. Behavioral truth table (before → after)

| Input (single supply, `chunks.len()==1`) | Today | After | Note |
|---|---|---|---|
| Valid non-chunked single-payload md1 (resolvable origin) | reject: `unsupported version 2` / exit 3 | **decode + render, exit 0** | the fix |
| Valid non-chunked DEAD card (elided unresolvable origin) | reject exit 3 | **template + `origin: «unspecified»` marker + exit 4** | consistent w/ chunked dead-card path |
| Valid chunked-of-1 string | decode via reassemble | decode via `decode_md1_string_with_opts`→reassemble | **byte-identical output** |
| codex32-invalid (bad BCH) single string | reject (codex32 layer) | reject (codex32 layer, same layer) | no re-route |
| codex32-valid future-version single-payload (v8/v12) | reject exit 3 | reject exit 3 (`Header::read` `WireVersionMismatch{got}`) | still exit 3 — diagnostic more accurate |
| codex32-valid but structurally-invalid *same-version* single payload | reject exit 3 (`got:2` misread) | reject via `decode_payload` (specific error, may be exit 2) | **diagnostic change; not a new acceptance** |
| Multi-chunk set (`len>1`), valid | reassemble, oracle enforced | reassemble, oracle enforced | **unchanged** |
| Multi-chunk set with doctored chunk-set-id | reject `ChunkSetIdMismatch` | reject `ChunkSetIdMismatch` | **oracle unchanged** |

## 5. Funds-safety invariants (normative — must all be test-anchored)

This is an intake-dispatch change on a funds-critical decode path. `inspect` is a
**describe** surface (no comparison, no funds-moving decision), which bounds
blast-radius, but the invariants below are the acceptance floor.

- **INV-1 (structural dispatch, never a fallback).** Dispatch is on
  `chunks.len()` plus md-codec's in-band chunked-flag bit. There is **no**
  try-one-then-the-other / catch-and-retry. A genuine decode error from either
  branch propagates as-is (no error masking).
- **INV-2 (BCH-before-flag → corruption can't re-route).** `unwrap_string`
  verifies the codex32 BCH checksum (`codex32.rs:183-184`) before the chunked-flag
  is read. A single-symbol corruption of the first symbol fails the checksum →
  reject; it can never silently divert a chunked card to the single-payload path
  or vice-versa.
- **INV-3 (content-id oracle stays enforced for chunked cards).** Every chunked
  input (chunked-of-1 and multi-chunk) still routes through
  `reassemble_with_opts`, whose cross-chunk derived-chunk-set-id / content-id
  check is **unconditional regardless of `opts`** (`chunk.rs:321-327, 406-415`). A
  single non-chunked string is self-contained and correctly has **no**
  cross-chunk oracle — expected, because there is exactly one payload to check;
  its integrity is the codex32 BCH checksum + `decode_payload`'s validators.
- **INV-4 (partial threads identically; `EmptyOriginOverride` fatal-in-partial).**
  `opts = DecodeOpts::partial()` reaches `decode_payload_with_opts` for the
  non-chunked case, so a non-chunked dead card exits 4 exactly like a chunked one,
  and `validate_no_empty_origin_overrides` stays a **distinct, always-fatal**
  reject never swallowed by partial (`vendor/md-codec/src/decode.rs:138-153`).
- **INV-5 (no new acceptance of malformed input).** The ONLY input that changes
  from reject→accept is a **valid** non-chunked single-payload md1. Any
  structurally-invalid single string still rejects — it runs `decode_payload`'s
  full gauntlet (version==4, root-tag allow-list `{Sh,Wsh,Wpkh,Pkh,Tr}`,
  placeholder-usage, multipath consistency, taptree validity, xpub bytes;
  `decode.rs:80-154`). Chunked-of-1 and multi-chunk outputs are byte-identical.

## 6. Test plan (TDD — RED before GREEN)

All fixtures use never-fund keys / the frozen md-cli-0.11.2 KAT corpus already in
`cli_inspect.rs:216-228`. Tests are CLI-level (`Command::cargo_bin("mnemonic")`)
unless noted.

### 6.1 RED-proofs (must fail on current `origin/master`, pass after the change)

1. **`inspect_md1_nonchunked_single_string_decodes`** — feed each
   `MD1_TEMPLATE_CORPUS` single-string fixture (e.g. `wpkh` =
   `md1yqpqqxqq8xtwhw4xwn4qh`) **directly** as `mnemonic inspect --md1 <single>`
   (no `rechunk()`). Assert **exit 0** and a `template:` line equal to the frozen
   expected (`wpkh(@0/<0;1>/*)`, etc.). RED today = exit 3 `unsupported version 2`.
2. **`inspect_md1_nonchunked_positional_form_decodes`** — same via the positional
   `mnemonic inspect <single>` intake (self-identifying HRP), asserting exit 0.
3. **`inspect_md1_nonchunked_json_shape`** — `--json` over a non-chunked single
   string: assert `schema_version:"2"`, `kind:"md1"`, correct `template`, and
   **no** `partial` key (resolvable-origin fixture). Byte-shape identical to the
   chunked render of the same descriptor.
4. **`inspect_md1_nonchunked_dead_card_exit4`** — a non-chunked **dead** card
   (canonical-origin-elided, unresolvable): assert stdout carries the `template:`
   line + `ORIGIN_UNSPECIFIED_MARKER`, stderr carries the partial note, and
   **exit 4** — byte-consistent with the chunked dead-card path. (Construct the
   dead-card single string in-crate from a pathless descriptor via
   `md_codec::encode_md1_string`, or reuse the P2.3 dead-card fixture emitted
   non-chunked.)

### 6.2 Regression locks (must stay GREEN before AND after)

5. **`inspect_md1_chunked_of_one_byte_identical`** — take a corpus fixture, build
   BOTH (a) its non-chunked single string and (b) its `split()` chunked-of-1
   string; assert `mnemonic inspect` stdout is **identical** for the descriptor
   either way (proves the chunked-of-1 route is unchanged and the two forms
   render the same card). Also assert the existing `rechunk()`-based tests
   (`cli_inspect.rs:245+`) still pass unchanged.
6. **`inspect_md1_multichunk_content_id_oracle_still_rejects`** — a multi-chunk
   set with a doctored chunk-set-id still rejects (`ChunkSetIdMismatch`); confirm
   the length-dispatch did not weaken the oracle (INV-3).

### 6.3 Negative / diagnostic

7. **`inspect_md1_nonchunked_bad_bch_rejects`** — a non-chunked single string with
   a flipped data symbol (BCH-broken beyond auto-fire, or with `--no-auto-repair`)
   rejects at the codex32 layer (INV-2), not accepted (INV-5).
8. **`inspect_md1_nonchunked_future_version_still_exit3`** — a synthetic
   codex32-valid single-payload with version 8/12 still reports exit 3
   (`Header::read` `WireVersionMismatch`), proving genuine future-format
   single strings keep the FutureFormat/exit-3 contract.

### 6.4 Cross-binary parity (MD_BIN-gated, mirrors the existing v0.75.0 gate)

9. **`inspect_md1_nonchunked_matches_md_inspect`** — for each corpus single
   string, assert `mnemonic inspect --md1 <single>`'s `template:` line ==
   `md inspect <single>`'s (byte-identical; both delegate to
   `md_codec::descriptor_to_template`). This closes the cross-binary asymmetry the
   FOLLOWUP names.

### 6.5 Full-suite requirement

Per `feedback_r0_review_run_full_package_suite`, per-phase R0 runs the FULL
`cargo test -p mnemonic-toolkit` suite (not targeted `--test` targets) — an intake
change can ripple into argv/schema/version lints even though no flag changed.

## 7. SemVer + lockstep

- **SemVer:** **MINOR** → `v0.89.0`. Additive intake capability; every
  previously-accepted input is byte-identical, only a previously-rejected valid
  form is now accepted.
- **schema_mirror (GUI):** **NOT triggered** — no clap flag/subcommand/dropdown
  add/remove/rename (`CLAUDE.md` GUI schema-mirror scope note: the gate is
  flag-NAME parity). No `mnemonic-gui/src/schema/mnemonic.rs` change.
- **`--json` wire-shape:** **unchanged** — same `InspectJson::Md1` shape,
  `schema_version` stays `"2"`. No GUI wire drift.
- **Manual mirror:** the flag-coverage lint (`docs/manual/tests/lint.sh`) mirrors
  `--help`; no flag changed → **no lockstep required**. OPTIONAL, non-gated:
  a one-line note in the inspect chapter that the single-string md1 form is now
  accepted (defer to R0's call; if added, it is a docs-only same-PR touch, not a
  gate).
- **Codecs:** md/mk/ms/wc **NO-BUMP** (existing md-codec 0.42.0 public API).
- **Release version sites** (per `project_toolkit_release_ritual_version_sites`):
  BOTH READMEs, `fuzz/Cargo.lock`, `scripts/install.sh` self-pin, `vendor/`
  (no dep bump here → no re-vendor), CHANGELOG (gated by `changelog-check` on the
  tag). Confirm the `.examples-build/` lockstep is unaffected (no example output
  changes).

## 8. Implementation checklist (single phase; TDD)

1. Write RED tests §6.1 (+ locks §6.2) → confirm RED on `origin/master`.
2. Apply §3.1 dispatch + §3.2 comment rewrite.
3. GREEN the suite (`cargo test -p mnemonic-toolkit`, full package).
4. MD_BIN-gated parity §6.4.
5. Version bump `v0.89.0` + all §7 version sites + CHANGELOG.
6. Post-implementation independent adversarial whole-diff review (mandatory,
   non-deferrable) — persist verbatim to `design/agent-reports/`.
7. Flip the FOLLOWUP: mark the **inspect leg RESOLVED** and leave a scoped
   residual for verify-bundle non-chunked intake (OUT-1) in the same shipping
   commit (`feedback_followup_status_discipline`).

## 9. Open questions for the R0 reviewer

1. **Scope confirmation:** inspect-only vs folding verify-bundle in now. Recon
   recommends inspect-only because `verify_bundle.rs:696` compares md1 **strings**
   (`expected.md1 == args.md1`) against toolkit chunk-form; a non-chunked template
   md1 cannot string-equal it without a comparison-canonicalization change (its
   own funds review). Confirm the carve-out (OUT-1) or direct otherwise.
2. **Dispatch form:** Option A's explicit `chunks.len() == 1` branch vs any
   preference for pushing the length dispatch down into a shared codec helper.
3. **Diagnostic/exit-code change acceptance (INV-5 row 6 of §4):** a
   codex32-valid-but-structurally-invalid *same-version* single string moves from
   `exit 3 "unsupported version 2"` to `decode_payload`'s specific error (possibly
   exit 2). Genuine future-version single strings keep exit 3. Acceptable, or must
   we preserve the exact prior exit code for all single-string rejects?
4. **Manual note:** confirm no lockstep required (no flag change) and whether the
   optional inspect-chapter prose note is wanted in-PR.
5. **Dead-card non-chunked fixture:** is synthesizing the non-chunked dead-card
   fixture in-crate via `md_codec::encode_md1_string` (vs freezing a KAT string)
   acceptable for test §6.1.4, given the P2 dead-card fixtures are chunk-form?
