# verify-bundle non-chunked md1 canonicalization — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Followup:** `toolkit-inspect-nonchunked-md1-intake-gap` — verify-bundle leg. **SPEC:** `design/SPEC_verify_bundle_nonchunked_canonicalization.md` (R0-GREEN 0C/0I). **R0 trail:** `design/agent-reports/verify-bundle-nonchunked-canonicalization-{designr0-round-1,specr0-round-1,specr0-round-2,planr0-round-1,planr0-round-2,planr0-round-3}.md`. **Source SHA:** `de140a08` (master == `mnemonic-toolkit-v0.89.0`; all cites re-grepped at write time). **Status:** **plan R0-GREEN (0C/0I)** — folded plan-R0 rounds 1 (4 Important + 3 Minor) + 2 (1 Important I-A + M-A) + M-i; **round-3 terminal GREEN** — cleared for implementation.

**Goal:** Make `mnemonic verify-bundle` accept a plain NON-chunked single-string template md1 (the bare `md encode` form) — it currently fails to classify (chunk-layer misread → fall-through → exit 2 / false `mismatch` on a valid card).

**Architecture:** Two coupled edits in one file (`crates/mnemonic-toolkit/src/cmd/verify_bundle.rs`). **Facet 1** — length-dispatch the supplied-md1 classify gate (strict `decode_md1_string` for a single string, `reassemble` for multi) so a non-chunked template routes into `verify_singlesig_template`/`verify_multisig_template`. **Facet 2** — replace the single-sig path's raw `Vec<String>` string-equality with a `compute_md1_encoding_id` content-id compare (the multisig path is already id-based). No codec change, no clap-flag change.

**Tech Stack:** Rust; `md-codec` (vendored, pinned `0.42.0`) public API; `assert_cmd` CLI tests; TDD.

## Global Constraints (verbatim from SPEC §0/§5/§7)
- **Facet 1 is STRICT** (`DecodeOpts::default()` via `decode_md1_string`, NOT `partial()`): the classify gate must preserve today's fail-closed routing — a dead/pathless/corrupted non-chunked card still fails classify and falls through (INV-5). Partial-decode stays OUT.
- **Facet 2 uses `compute_md1_encoding_id`** (NOT WDT-id): the faithful minimal relaxation of byte-identity — tolerates only chunk/HRP/checksum re-encodings of the SAME descriptor (INV-3), sensitive to all descriptor content (INV-4).
- **Scope:** template intake + single-sig compare ONLY. Keyed/policy-form is structurally impossible to be non-chunked (400-bit cap; INV-KEYED). Corrupted/partial paths OUT (dead-card verdict-asymmetry residual FILED, not fixed).
- **No new `--json` `result` state.** A mismatch stays `md1_template_match:false → "mismatch"`/exit 4.
- **`md1_template_match` is a canonicality check on the card's own `(tag,body)` type** (expected re-derived from `cli_template_from_tree(&d.tree)`); cross-wallet/seed rejection is carried by `mk1_template_stub_bind` + recompose + `--expect-wallet-id`, NOT this compare (SPEC §3.2b).
- **SemVer MINOR → v0.90.0.** No GUI `schema_mirror`, no manual flag-mirror lockstep (no clap flag change).
- **md_codec crate-root re-exports used** (confirmed `vendor/md-codec/src/lib.rs:46-66`): `Descriptor`, `decode_md1_string`, `chunk::reassemble`, `encode_md1_string`, `compute_md1_encoding_id`, `PathDecl`, `PathDeclPaths`, `OriginPath`, `PathComponent`.
- **Gates (each task + pre-tag):** FULL `cargo test -p mnemonic-toolkit`; `cargo clippy -p mnemonic-toolkit --all-targets`; **`cargo +1.95.0 fmt --all -- --check` (mlock-exempt) BEFORE tag** (v0.87.0 lesson); post-impl whole-diff Fable R0 before tag.

---

### Task 1: Facet 1 — length-dispatch the classify gate

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:386-408` (the template short-circuit classify gate)
- Test: `crates/mnemonic-toolkit/tests/cli_verify_bundle_md1_template.rs` (keyed multi-chunk lock + dead-card lock + `to_nonchunked`/`keyed_cards` helpers)
- Test: `crates/mnemonic-toolkit/tests/cli_verify_bundle_md1_template_multisig.rs` (non-chunked multisig routing + positive verify — reuses that file's existing `emit_template_md1`/`push_md1`/`verify_json` helpers)

**Interfaces:**
- Consumes: `md_codec::decode_md1_string(&str) -> Result<Descriptor, Error>` (strict), `md_codec::chunk::reassemble(&[&str]) -> Result<Descriptor, Error>`, `md_codec::encode_md1_string(&Descriptor) -> Result<String, Error>`.
- Produces: for later tasks/tests — a shared `to_nonchunked(&[String]) -> String` helper (defined in BOTH `cli_verify_bundle_md1_template.rs` and `…_multisig.rs`, since integration test files are separate crates and cannot share helpers).

- [ ] **Step 1: Write the failing test — non-chunked multisig template ROUTES (RED on master).**

Add to `crates/mnemonic-toolkit/tests/cli_verify_bundle_md1_template_multisig.rs` (reusing its existing `emit_template_md1`, `push_md1`, `mnemonic()`, `SEED_A`, `SEED_B`):
```rust
/// Decode a chunk-form md1 set and RE-ENCODE it as a single NON-chunked md1
/// string (the bare `md encode` form; encode_md1_string always emits the
/// single-payload/non-chunked form — the shape `bundle` never emits).
fn to_nonchunked(chunk_form_md1: &[String]) -> String {
    let refs: Vec<&str> = chunk_form_md1.iter().map(String::as_str).collect();
    let d = md_codec::chunk::reassemble(&refs).expect("chunk-form md1 decodes");
    md_codec::encode_md1_string(&d).expect("re-encode as a single non-chunked md1")
}

#[test]
fn verify_bundle_nonchunked_multisig_template_routes_no_from() {
    // A keyless 2-of-2 wsh-sortedmulti TEMPLATE, re-encoded non-chunked, supplied
    // WITHOUT --from, must REACH verify_multisig_template and refuse naming the
    // seed requirement (proving Facet 1 routed it). Today it falls THROUGH the
    // chunk-form-only classify gate → the general dispatch errors differently
    // (no "--from/seed" refusal). Keyless 2-of-2 template is < 400 bits → fits
    // a single non-chunked string.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let single = to_nonchunked(&md1);
    let assert = mnemonic()
        .args(["verify-bundle", "--network", "mainnet", "--md1", &single])
        .assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--from") || stderr.contains("seed"),
        "non-chunked multisig template must route to verify_multisig_template and \
         name the --from/seed requirement (proves Facet 1 routed): {stderr}"
    );
}
```
(`emit_template_md1(template, threshold, cosigners)` already exists in this file — it wraps `canonical_multisig_template_args` and returns the chunk-form md1 vec; do NOT use single-sig `template_cards` here, which panics at n=1 for a multisig template — `pre_check_template_n`, `bundle_unified.rs:107`.)

- [ ] **Step 2: Run it — verify RED.**

Run: `cargo test -p mnemonic-toolkit --test cli_verify_bundle_md1_template_multisig verify_bundle_nonchunked_multisig_template_routes_no_from -- --nocapture`
Expected: FAIL — the non-chunked card falls through the classify gate today; the error names `--template` (ModeViolation), not `--from`/`seed`.

- [ ] **Step 3: Write the positive non-chunked multisig verify (RED on master, GREEN after Facet 1) — SPEC §6.1 #3 GREEN target.**

Mirror the existing `verify_bundle_canonical_multisig_template_id_search_ok` (`…_multisig.rs:293-361`) but supply the md1 NON-chunked:
```rust
#[test]
fn verify_bundle_nonchunked_multisig_template_verifies_ok() {
    // OUT-3 "free ride": a non-chunked keyless multisig template, completed via
    // --from + --cosigner, verifies GREEN through the (already id-based) WDT-id
    // compare (:937-941). Proves Facet 1 alone closes the multisig leg.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let stubs = emit_template_mk1_stubs("wsh-sortedmulti", "2", cos);
    let id = emit_template_wallet_id("wsh-sortedmulti", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-sortedmulti", "2", cos, 1);

    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &[to_nonchunked(&md1)]);          // <-- NON-chunked single string
    push_mk1_stubs(&mut args, &stubs);
    args.extend(["--from".into(), format!("phrase={SEED_A}"),
        "--account".into(), "0".into(), "--expect-wallet-id".into(), id, "--json".into()]);
    push_cosigners(&mut args, &[mk1_b]);

    let j = verify_json(&args);
    assert_eq!(j["result"], "ok", "non-chunked multisig template must verify OK: {j}");
    let by = |n: &str| j["checks"].as_array().unwrap().iter().find(|c| c["name"] == n).unwrap()["passed"].clone();
    assert_eq!(by("md1_template_match"), true, "md1_template_match must pass: {j}");
}
```
(All `emit_*`/`push_*`/`verify_json` helpers already exist in this file, `…_multisig.rs:244-287`.)

- [ ] **Step 4: Add the two regression LOCKS to `cli_verify_bundle_md1_template.rs` (GREEN before AND after Facet 1).**

```rust
/// Emit a KEYED (wallet-policy) bundle and return its (ms1, mk1, md1) cards.
/// Same as `template_cards` but WITHOUT `--md1-form template`, so the md1 is a
/// keyed policy card — naturally MULTI-chunk (a 65-byte pubkey = 520 bits > the
/// 400-bit single-string cap), exercising the classify `_ => reassemble` arm.
fn keyed_cards(template: &str, phrase: &str, account: &str)
    -> (Vec<String>, Vec<String>, Vec<String>)
{
    let out = mnemonic().args(["bundle", "--template", template, "--network", "mainnet",
        "--account", account, "--group-size", "0",
        "--slot", &format!("@0.phrase={phrase}")]).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let (mut ms1, mut mk1, mut md1) = (Vec::new(), Vec::new(), Vec::new());
    let mut section = "";
    for line in stdout.lines() {
        if line.starts_with("# ms1") { section = "ms1"; continue; }
        if line.starts_with("# mk1") { section = "mk1"; continue; }
        if line.starts_with("# md1") { section = "md1"; continue; }
        let t = line.trim();
        if t.is_empty() { section = ""; continue; }
        match section { "ms1" => ms1.push(t.into()), "mk1" => mk1.push(t.into()),
            "md1" => md1.push(t.into()), _ => {} }
    }
    (ms1, mk1, md1)
}

#[test]
fn verify_bundle_keyed_multichunk_unchanged() {
    // A KEYED bip84 bundle md1 is multi-chunk. It enters the classify `match`'s
    // `_ => reassemble` arm (len>1, verbatim-unchanged by Facet 1), skips both
    // template branches (is_wallet_policy=true), and verifies via the general
    // path — GREEN before AND after Facet 1.
    let (ms1, mk1, md1) = keyed_cards("bip84", PHRASE_A, "0");
    assert!(md1.len() > 1, "keyed bip84 md1 must be multi-chunk: {md1:?}");
    // A KEYED wallet-policy md1 REQUIRES --template: it skips the keyless-template
    // short-circuit → verify_bundle.rs:435-443 ModeViolation without it (planr0-r2
    // I-A). The general/keyed path prints lowercase "result: ok" (:558-567), NOT the
    // template-path-only "OK (…recomposed)" string (:824). Mirror the proven keyed
    // verify pattern in cli_verify_bundle_full.rs:30-56.
    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into(),
        "--template".into(), "bip84".into(),
        "--account".into(), "0".into(), "--slot".into(), format!("@0.phrase={PHRASE_A}")];
    for m in ms1.iter().filter(|m| !m.is_empty()) { args.push("--ms1".into()); args.push(m.clone()); }
    for m in &mk1 { args.push("--mk1".into()); args.push(m.clone()); }
    for m in &md1 { args.push("--md1".into()); args.push(m.clone()); }
    let out = mnemonic().args(&args).assert().success();
    assert!(String::from_utf8(out.get_output().stdout.clone()).unwrap().contains("result: ok"));
}

#[test]
fn verify_bundle_nonchunked_dead_card_falls_through_strict() {
    // A NON-chunked KEYLESS DEAD card: take the frozen keyed wsh(pk) card
    // (`m/48'/0'/0'`, a NON-canonical wrapper), strip its keys → keyless template,
    // elide the origin → unresolvable. Keyless → re-encodes as a single non-chunked
    // string (mirrors the `dead()` helper in cli_repair_dead_card_strict.rs:32-36,
    // minus keys, plus encode_md1_string). Strict decode_md1_string rejects the
    // elided-unresolvable origin (MissingExplicitOrigin) → classify falls THROUGH →
    // never "OK" (SPEC INV-5), before AND after Facet 1.
    const SS_MD1_ORIGIN: &[&str] = &[
        "md1f9xlxpqpqpmvyyyqqcy2pdqhp5gmug4gy80cpxatjnpdtxhjvyuds54ar44wuc0a34",
        "md1f9xlxpq036ekkrhtkv6grq7qcua7ej7xusqaaq2qptxulyg808qnqjq8s570kd4kkd",
        "md1f9xlxpqsz3h36nf43a3dytlcf6saj9lwz9gc9uag7ce95hlcqu95t5qpd0qs94",
    ];
    let mut d = md_codec::chunk::reassemble(SS_MD1_ORIGIN).expect("decode keyed wsh(pk) card");
    d.tlv.pubkeys = None;        // → keyless template (fits a single non-chunked string)
    d.tlv.fingerprints = None;
    d.path_decl.paths =
        md_codec::PathDeclPaths::Shared(md_codec::OriginPath { components: vec![] }); // elide → dead
    let dead_single = md_codec::encode_md1_string(&d).expect("re-encode keyless dead card non-chunked");
    let assert = mnemonic()
        .args(["verify-bundle", "--network", "mainnet", "--md1", &dead_single])
        .assert().failure();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(!stdout.contains("OK"), "non-chunked dead card must never verify OK: {stdout}");
}
```
(`SS_MD1_ORIGIN` copied verbatim from `cli_repair_dead_card_strict.rs:20-24`; `PHRASE_A` already exists at `cli_verify_bundle_md1_template.rs:7`. Add `to_nonchunked` — same body as Step 1 — to this file too.)

- [ ] **Step 5: Run the locks — verify GREEN on current master.**

Run: `cargo test -p mnemonic-toolkit --test cli_verify_bundle_md1_template verify_bundle_keyed_multichunk_unchanged verify_bundle_nonchunked_dead_card_falls_through_strict`
Expected: PASS (regression locks — if RED, the fixture is wrong; fix the fixture, not the source).

- [ ] **Step 6: Apply Facet 1 (the source change).**

In `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs`, replace `:387-388`:
```rust
        let md1_refs: Vec<&str> = args.md1.iter().map(|s| s.as_str()).collect();
        if let Ok(d) = md_codec::chunk::reassemble(&md1_refs) {
```
with:
```rust
        let md1_refs: Vec<&str> = args.md1.iter().map(|s| s.as_str()).collect();
        // A single supplied md1 may be a NON-chunked single-payload string (bare
        // `md encode` form) OR a chunked-of-1 string. decode_md1_string dispatches
        // on the in-band chunked-flag bit (decode.rs:187-196) — a chunked-of-1
        // routes internally back to reassemble, so this is byte-identical to today
        // for chunk-form input. STRICT (default opts, NOT partial): the classify
        // gate must preserve today's routing — a dead/pathless/corrupted card still
        // fails decode here and falls through (SPEC INV-5). Fixes intake routing for
        // BOTH single-sig and multisig template cards.
        let classify = match md1_refs.as_slice() {
            [single] => md_codec::decode_md1_string(single),
            _ => md_codec::chunk::reassemble(&md1_refs),
        };
        if let Ok(d) = classify {
```
(Leave `:389-406` unchanged.)

- [ ] **Step 7: Run the full package suite + clippy.**

Run: `cargo test -p mnemonic-toolkit && cargo clippy -p mnemonic-toolkit --all-targets`
Expected: ALL PASS — Steps 1 & 3 RED cells now GREEN (multisig routes + verifies); the two locks stay GREEN; nothing else regresses.

- [ ] **Step 8: Commit.**

```bash
git add crates/mnemonic-toolkit/src/cmd/verify_bundle.rs \
        crates/mnemonic-toolkit/tests/cli_verify_bundle_md1_template.rs \
        crates/mnemonic-toolkit/tests/cli_verify_bundle_md1_template_multisig.rs
git commit -m "feat(verify-bundle): length-dispatch classify gate for non-chunked md1 (Facet 1)"
```

---

### Task 2: Facet 2 — content-id compare in the single-sig path

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:696` (single-sig `md1_match`)
- Test: `crates/mnemonic-toolkit/tests/cli_verify_bundle_md1_template.rs` (7 single-sig cells)

**Interfaces:**
- Consumes: Task 1's `to_nonchunked`; `md_codec::compute_md1_encoding_id(&Descriptor) -> Result<Md1EncodingId, Error>`; `md_codec::{PathDeclPaths, OriginPath, PathComponent}`; the `d: &md_codec::Descriptor` param of `verify_singlesig_template` (already the decoded supplied card, `:591`).
- Produces: the closed verify-bundle single-sig leg.

- [ ] **Step 1: Write the failing test — non-chunked single-sig template verifies OK.**

```rust
/// Like `verify_args`, but supply the md1 as a single NON-chunked string.
fn verify_args_nonchunked(template: &str, phrase: &str, account: &str) -> Vec<String> {
    let (ms1, mk1, md1) = template_cards(template, phrase, account);
    let single = to_nonchunked(&md1);
    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into(),
        "--account".into(), account.into(), "--slot".into(), format!("@0.phrase={phrase}")];
    for m in ms1.iter().filter(|m| !m.is_empty()) { args.push("--ms1".into()); args.push(m.clone()); }
    for m in &mk1 { args.push("--mk1".into()); args.push(m.clone()); }   // REQUIRED (SPEC M-2)
    args.push("--md1".into()); args.push(single);
    args
}

#[test]
fn verify_bundle_nonchunked_singlesig_template_ok() {
    for template in ["bip44", "bip84", "bip86"] {
        let out = mnemonic().args(verify_args_nonchunked(template, PHRASE_A, "0"))
            .assert().success();
        let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        assert!(stdout.contains("OK"), "{template} non-chunked must verify OK: {stdout}");
    }
}
```

- [ ] **Step 2: Run it — verify RED.**

Run: `cargo test -p mnemonic-toolkit --test cli_verify_bundle_md1_template verify_bundle_nonchunked_singlesig_template_ok`
Expected: FAIL — with Task 1 applied the card ROUTES into `verify_singlesig_template`, but the raw `expected.md1 == args.md1` compare (chunk-form expected vs non-chunked supplied) fails → `md1_template_match:false` → `mismatch`/exit 4.

- [ ] **Step 3: Apply Facet 2 (the source change).**

In `verify_bundle.rs`, replace `:696`:
```rust
    let md1_match = expected.md1 == args.md1;
```
with:
```rust
    // Compare by CONTENT-ID, not raw strings: a non-chunked supplied md1 can never
    // string-equal the chunk-form synthesized expected, yet decodes to the SAME
    // descriptor. compute_md1_encoding_id hashes encode_payload (chunk/HRP/checksum-
    // agnostic, but sensitive to all descriptor content) — the faithful minimal
    // relaxation of byte-identity (SPEC §2.2/INV-3/INV-4). `d` is the already-decoded
    // supplied card; `expected.md1` is toolkit-generated chunk-form (mirror :2902-2904).
    let expected_md1_refs: Vec<&str> = expected.md1.iter().map(String::as_str).collect();
    let d_expected = md_codec::chunk::reassemble(&expected_md1_refs)?;
    let md1_match =
        md_codec::compute_md1_encoding_id(d)? == md_codec::compute_md1_encoding_id(&d_expected)?;
```

- [ ] **Step 4: Run it — verify GREEN.**

Run: `cargo test -p mnemonic-toolkit --test cli_verify_bundle_md1_template verify_bundle_nonchunked_singlesig_template_ok`
Expected: PASS.

- [ ] **Step 5: Add the remaining single-sig cells.**

```rust
#[test]
fn verify_bundle_chunked_template_still_ok() {
    // No-regression (INV-3): byte-compare pass ⟹ same descriptor ⟹ id-compare pass.
    let out = mnemonic().args(verify_args("bip84", PHRASE_A, "0", None)).assert().success();
    assert!(String::from_utf8(out.get_output().stdout.clone()).unwrap().contains("OK"));
}

#[test]
fn verify_bundle_nonchunked_singlesig_json_ok() {
    let mut args = verify_args_nonchunked("bip84", PHRASE_A, "0");
    args.push("--json".into());
    let out = mnemonic().args(&args).assert().success();
    let v: serde_json::Value = serde_json::from_slice(&out.get_output().stdout).expect("json");
    assert_eq!(v["result"], "ok");
    assert_eq!(v["mode"], "single-sig-template");
    let md1c = v["checks"].as_array().unwrap().iter()
        .find(|c| c["name"] == "md1_template_match").expect("md1_template_match check");
    assert_eq!(md1c["passed"], true);
}

#[test]
fn verify_bundle_form_equivalence_same_verdict() {
    // SPEC §6.2 #5: the SAME descriptor as chunk-form vs non-chunked yields the
    // identical verdict — stdout AND stderr (the ✓/✗ check lines print to stderr,
    // verify_bundle.rs:811-820) AND the --json shape (planr0 M-1).
    let chunked = mnemonic().args(verify_args("bip84", PHRASE_A, "0", None)).assert().success();
    let nonchunked = mnemonic().args(verify_args_nonchunked("bip84", PHRASE_A, "0")).assert().success();
    assert_eq!(String::from_utf8(chunked.get_output().stdout.clone()).unwrap(),
               String::from_utf8(nonchunked.get_output().stdout.clone()).unwrap(), "stdout");
    assert_eq!(String::from_utf8(chunked.get_output().stderr.clone()).unwrap(),
               String::from_utf8(nonchunked.get_output().stderr.clone()).unwrap(), "stderr checks");
    let mut cj = verify_args("bip84", PHRASE_A, "0", None); cj.push("--json".into());
    let mut nj = verify_args_nonchunked("bip84", PHRASE_A, "0"); nj.push("--json".into());
    let cjv: serde_json::Value = serde_json::from_slice(&mnemonic().args(&cj).assert().success().get_output().stdout).unwrap();
    let njv: serde_json::Value = serde_json::from_slice(&mnemonic().args(&nj).assert().success().get_output().stdout).unwrap();
    assert_eq!(cjv, njv, "--json shape must be identical across forms");
}

#[test]
fn verify_bundle_nonchunked_noncanonical_encoding_mismatch() {
    // PROBATIVE INV-4 anchor (SPEC §6.3 #7, construction b): inject a Fingerprints
    // TLV the template synthesis never carries. Same (tag,body) → still classifies
    // single-sig + re-derives the SAME (fingerprint-less) expected → encoding-id
    // DIFFERS → md1_template_match FALSE. Stays GREEN only if the compare is
    // content-sensitive (a broken md1_match=true regression FAILS it). Fingerprints
    // are WDT-id-EXCLUDED, so this also proves encoding-id > WDT-id.
    let (ms1, mk1, md1) = template_cards("bip84", PHRASE_A, "0");
    let refs: Vec<&str> = md1.iter().map(String::as_str).collect();
    let mut d = md_codec::chunk::reassemble(&refs).unwrap();
    d.tlv.fingerprints = Some(vec![(0u8, [0xABu8; 4])]);
    let doctored = md_codec::encode_md1_string(&d).unwrap();
    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into(),
        "--account".into(), "0".into(), "--slot".into(), format!("@0.phrase={PHRASE_A}"), "--json".into()];
    for m in ms1.iter().filter(|m| !m.is_empty()) { args.push("--ms1".into()); args.push(m.clone()); }
    for m in &mk1 { args.push("--mk1".into()); args.push(m.clone()); }
    args.push("--md1".into()); args.push(doctored);
    let assert = mnemonic().args(&args).assert().code(4);
    let v: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout).unwrap();
    let md1c = v["checks"].as_array().unwrap().iter()
        .find(|c| c["name"] == "md1_template_match").unwrap();
    assert_eq!(md1c["passed"], false, "non-canonical encoding must mismatch: {v}");
}

#[test]
fn verify_bundle_nonchunked_doctored_origin_stricter_than_wdt() {
    // SPEC §6.3 #8: an EXPLICIT (non-elided) canonical origin. encode_payload writes
    // path_decl verbatim → the explicit form's id differs from the elided expected's;
    // WDT-id EXCLUDES origin-path-decl so it would MATCH — proving encoding-id is
    // strictly stronger. Tree stays canonical → strict-decodes + classifies.
    let (ms1, mk1, md1) = template_cards("bip84", PHRASE_A, "0");
    let refs: Vec<&str> = md1.iter().map(String::as_str).collect();
    let mut d = md_codec::chunk::reassemble(&refs).unwrap();
    d.path_decl.paths = md_codec::PathDeclPaths::Shared(md_codec::OriginPath {
        components: vec![
            md_codec::PathComponent { hardened: true, value: 84 },
            md_codec::PathComponent { hardened: true, value: 0 },
            md_codec::PathComponent { hardened: true, value: 0 },
        ],
    });
    let doctored = md_codec::encode_md1_string(&d).unwrap();
    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into(),
        "--account".into(), "0".into(), "--slot".into(), format!("@0.phrase={PHRASE_A}")];
    for m in ms1.iter().filter(|m| !m.is_empty()) { args.push("--ms1".into()); args.push(m.clone()); }
    for m in &mk1 { args.push("--mk1".into()); args.push(m.clone()); }
    args.push("--md1".into()); args.push(doctored);
    mnemonic().args(&args).assert().code(4);   // md1_template_match mismatch → exit 4
}

#[test]
fn verify_bundle_mk1_tolerance_not_extended() {
    // SPEC §6.4 #10: md1 form-tolerance is md1-ONLY. A matching non-chunked md1 with
    // a case-variant mk1 still mismatches (mk1_template_stub_bind byte-compare :697-700).
    let (ms1, mk1, md1) = template_cards("bip84", PHRASE_A, "0");
    let single = to_nonchunked(&md1);
    let mk1_variant: Vec<String> = mk1.iter().map(|m| m.to_uppercase()).collect();
    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into(),
        "--account".into(), "0".into(), "--slot".into(), format!("@0.phrase={PHRASE_A}")];
    for m in ms1.iter().filter(|m| !m.is_empty()) { args.push("--ms1".into()); args.push(m.clone()); }
    for m in &mk1_variant { args.push("--mk1".into()); args.push(m.clone()); }
    args.push("--md1".into()); args.push(single);
    mnemonic().args(&args).assert().code(4);   // mk1 stub-bind fails → mismatch
}
```
NOTE (`mk1_tolerance`): `verify_singlesig_template` never decodes `args.mk1` — the only touch is the byte-compare `chunks == &args.mk1` (`:697-700`) — so an uppercased mk1 fails it (planr0 verified). If for any reason it binds, swap to a genuinely-different cosigner mk1; the assertion (md1 tolerance ≠ mk1 tolerance) is unchanged.

- [ ] **Step 6: Run the full package suite + clippy.**

Run: `cargo test -p mnemonic-toolkit && cargo clippy -p mnemonic-toolkit --all-targets`
Expected: ALL PASS.

- [ ] **Step 7: Commit.**

```bash
git add crates/mnemonic-toolkit/src/cmd/verify_bundle.rs crates/mnemonic-toolkit/tests/cli_verify_bundle_md1_template.rs
git commit -m "feat(verify-bundle): content-id compare for single-sig template (Facet 2) — closes non-chunked md1 leg"
```

---

### Task 3: Release ritual + FOLLOWUP reconciliation (v0.90.0)

**Files:** `crates/mnemonic-toolkit/Cargo.toml:3`; `Cargo.lock` + `fuzz/Cargo.lock` (repo-root `fuzz/`, planr0 I-4); BOTH READMEs; `scripts/install.sh` toolkit SELF-pin; `CHANGELOG.md`; `design/FOLLOWUPS.md`.

- [ ] **Step 1: Bump version + regenerate locks.**

```bash
sed -i 's/^version = "0.89.0"/version = "0.90.0"/' crates/mnemonic-toolkit/Cargo.toml
cargo check -p mnemonic-toolkit          # refresh Cargo.lock
( cd fuzz && cargo check )               # refresh fuzz/Cargo.lock (repo-root fuzz crate; pins toolkit 0.89.0→0.90.0)
```
Then bump the `toolkit-version` markers in `README.md` and `crates/mnemonic-toolkit/README.md`, and the `mnemonic-toolkit-v0.89.0`→`v0.90.0` SELF-pin in `scripts/install.sh` (leave md/ms/mk git pins + the GUI pin UNTOUCHED — SPEC §7; separate handoff residuals).

- [ ] **Step 2: CHANGELOG entry (migration + completeness — SPEC Minors #2/#4).**

Add a `v0.90.0` entry to `CHANGELOG.md`:
```markdown
## v0.90.0
- **verify-bundle now accepts a non-chunked single-string md1.** A plain `md encode`
  template card (previously exit 2 / a false `mismatch`+exit 4) now returns its real
  verdict: the classify gate length-dispatches (strict) and the single-sig template
  compare is by content-id (`compute_md1_encoding_id`), tolerating only chunk/HRP/
  checksum re-encodings of the same descriptor. Closes the verify-bundle leg of
  `toolkit-inspect-nonchunked-md1-intake-gap` (the inspect leg shipped v0.89.0).
- Scope note: keyed/wallet-policy md1 is structurally always chunked (a 65-byte
  pubkey = 520 bits exceeds the 400-bit single-string cap), so the fix is provably
  complete for the non-chunked intake it covers.
```

- [ ] **Step 3: Confirm `.examples-build/` is unaffected (SPEC §7; planr0 M-3).**

Run: `grep -rl 'verify-bundle' .examples-build/ 2>/dev/null || echo "no verify-bundle examples"`
Expected: no verify-bundle example fence uses a non-chunked md1 → no example-output drift. If any does, regenerate per `.examples-build/`'s gen script and confirm only version drift. (No dep bump → no `vendor/` re-vendor.)

- [ ] **Step 4: Reconcile the FOLLOWUP + file the residual (`feedback_followup_status_discipline`).**

In `design/FOLLOWUPS.md`:
1. `toolkit-inspect-nonchunked-md1-intake-gap` (`:34-38`) — the `⚠️ RESIDUAL — verify-bundle leg STAYS OPEN` note becomes **✓ RESOLVED (verify-bundle leg, v0.90.0)** with the Facet-1/Facet-2 + `compute_md1_encoding_id` summary; the whole slug is now closed.
2. **File** the new residual `verify-bundle-nonchunked-deadcard-verdict-asymmetry` (task #3 of this run): a non-chunked keyless DEAD card in the general path reads `mismatch` where its chunked twin reads `partial` (both exit 4, fail-closed); owned by a future partial-decode cycle; `md1_partial::supplied_md1_unresolved_indices` (`md1_partial.rs:55-63`) is chunk-only. Status `open`, tier `next-cycle`, PARKED (scoped OUT this run).

- [ ] **Step 5: Pre-tag gates.**

Run: `cargo test -p mnemonic-toolkit && cargo clippy -p mnemonic-toolkit --all-targets && cargo +1.95.0 fmt --all -- --check`
Expected: ALL PASS / clean (fmt mlock-exempt; per-package `cargo fmt -p mnemonic-toolkit` only if a fix is needed — NEVER `cargo fmt --all` that rewrites mlock.rs).

- [ ] **Step 6: Commit the release (source + docs + design trail).**

```bash
git add crates/mnemonic-toolkit/Cargo.toml Cargo.lock fuzz/Cargo.lock \
        README.md crates/mnemonic-toolkit/README.md scripts/install.sh CHANGELOG.md design/FOLLOWUPS.md \
        design/SPEC_verify_bundle_nonchunked_canonicalization.md \
        design/IMPLEMENTATION_PLAN_verify_bundle_nonchunked_canonicalization.md \
        design/agent-reports/verify-bundle-nonchunked-canonicalization-{designr0-round-1,specr0-round-1,specr0-round-2,planr0-round-1,planr0-round-2,planr0-round-3}.md \
        cycle-prep-recon-verify-bundle-nonchunked-md1-canonicalization.md
git commit -m "release: mnemonic-toolkit v0.90.0 — verify-bundle non-chunked md1 canonicalization"
```

- [ ] **Step 7: Post-implementation whole-diff Fable R0, THEN commit the report + tag (planr0 M-2).**

Dispatch a Fable adversarial whole-diff review over the full `git diff de140a08..HEAD`; persist verbatim to `design/agent-reports/verify-bundle-nonchunked-canonicalization-postimpl-whole-diff-review.md`; fold to 0C/0I (re-dispatch after folds). Commit the post-impl report (it did not exist at Step 6): `git add design/agent-reports/…-postimpl-whole-diff-review.md && git commit -m "design: post-impl whole-diff R0 (GREEN)"`. ONLY after GREEN: `git tag mnemonic-toolkit-v0.90.0 && git push origin HEAD:master --tags` (direct-FF per the constellation release model; no publish — toolkit is not on crates.io).

## Plan-R0 folds
**Round 1 (Fable, `…-planr0-round-1.md`) — RED 4 Important + 3 Minor, all folded:**
- **I-1** multichunk fixture used non-existent `--force-chunked`/`--chunk-size` + a single-sig template can't be multi-chunk → re-fixtured as `verify_bundle_keyed_multichunk_unchanged` (keyed policy card, naturally multi-chunk) with a new `keyed_cards` helper (Task 1 Step 4).
- **I-2** primary multisig fixture panicked at n=1 (`pre_check_template_n`) → moved multisig routing test to `cli_verify_bundle_md1_template_multisig.rs`, using its `emit_template_md1` helper (Task 1 Step 1).
- **I-3** spec §6.1 #3 positive multisig verify was missing → added `verify_bundle_nonchunked_multisig_template_verifies_ok` (Task 1 Step 3).
- **I-4** fuzz paths wrong (`crates/mnemonic-toolkit/fuzz/` → repo-root `fuzz/`) → corrected Task 3 Steps 1 & 6.
- **M-1** form-equivalence now compares stdout + stderr + `--json` (Task 2 Step 5).
- **M-2** post-impl report now committed before tag (Task 3 Step 7).
- **M-3** `.examples-build/` confirm added (Task 3 Step 3).

**Round 2 (Fable, `…-planr0-round-2.md`) — RED 1 Important + 1 Minor, folded:**
- **I-A** the round-1 I-1 re-fixture (`verify_bundle_keyed_multichunk_unchanged`) was RED on master: a keyed wallet-policy md1 REQUIRES `--template` (else ModeViolation :435-443) and the general path prints lowercase `result: ok`, not `OK` → added `--template bip84` + assert `result: ok`, mirroring cli_verify_bundle_full.rs:30-56 (Task 1 Step 4).
- **M-A** stale cite `:293-321` → `:293-361` (Task 1 Step 3).

**Round 3 (Fable, `…-planr0-round-3.md`) — TERMINAL GREEN (0C/0I).** Confirmed the I-A fold correct + GREEN on master (verified `--ms1` is required and non-empty for a keyed phrase bundle; cell mirrors the proven cli_verify_bundle_full.rs:31-56 precedent). One cosmetic Minor (M-i: stale Status line / R0-trail) folded post-review. **Plan cleared for implementation.**
