# SPEC — Wave-4 L2: `export-wallet-bundle-descriptor-md1-clearer-error`

**Slug:** `export-wallet-bundle-descriptor-md1-clearer-error`
**Theme:** B-error-diagnostics (Wave-4 low-value tail; recon-ranked HIGHEST-value of the 3 genuine opens)
**FOLLOWUPS entry:** `design/FOLLOWUPS.md:4087` (header) — Status line `design/FOLLOWUPS.md:4092` is `open` (re-verified @940abe9e)
**Source SHA all citations re-grepped against:** `940abe9e7cbf55ab005f3aae6541ec42ab7dbd69` (toolkit master, v0.71.0)
**SemVer:** **NO-BUMP** — additional typed error clarifying an existing hard-error refusal path; no clap-flag/subcommand/dropdown/wire-shape surface change.
**CI coupling:** NONE. No clap flag-NAME / subcommand / dropdown-enum change → **no GUI `schema_mirror` lockstep, no manual flag-coverage lint** (`docs/manual/tests/lint.sh` untouched). No fmt-gate / version-site / fuzz coupling. No new `--`flag → no argv-secret lint impact.
**Author single-source:** this spec is the sole authority; an R0 review gates it before any code.

> **R0 fold note (this revision):** four R0 findings folded — (1) **Important**: the error-message `xpub-search` pointer was to a NON-EXISTENT bare invocation; `--descriptor` lives on the `account-of-descriptor` **subcommand**, so the message + the two CLI-cell assertions now require the full working form `mnemonic xpub-search account-of-descriptor --descriptor <md1…>` and gate the substring `xpub-search account-of-descriptor`. (2/4) **Minor narrative**: §1 now notes the pre-fix bundle message depends on whether the card payload embeds a `*pub` substring; §2.4 notes the flag-neutral-label option (stable label retained, rationale below). (3) **Minor decay**: snapshot anchors corrected to live lines (`is_bip388_policy_shape` :244, `expand_bip388_policy` :259). Source SHA refreshed `e6c36f0c` → `940abe9e`; FOLLOWUPS Status line `:4088` → `:4092`.

---

## 0. RE-GREP MANDATE (implementer MUST run before writing each edit)

Every line number below is a snapshot @940abe9e. **Re-grep against current `origin/master` at write time** — these decay every merge. The recon already flagged one stale citation class (an in-code `S-VERIFY` comment ref). Concretely, re-confirm before editing:

```
# Enum + the 3 EXHAUSTIVE match blocks in error.rs (alphabetical insertion point = between Io and MdCodec):
git show origin/master:crates/mnemonic-toolkit/src/error.rs | grep -n 'Io(std::io::Error)\|MdCodec(md_codec::Error)'
git show origin/master:crates/mnemonic-toolkit/src/error.rs | grep -n 'ToolkitError::Io\b'   # appears in exit_code, kind, message blocks
# Intake sites:
git show origin/master:crates/mnemonic-toolkit/src/cmd/export_wallet.rs | grep -n 'is_bip388_policy_shape\|is_at_n_form\|MsDescriptor::<DescriptorPublicKey>::from_str'
git show origin/master:crates/mnemonic-toolkit/src/cmd/bundle.rs | grep -n 'is_bip388_policy_shape\|let body = match\|classify_descriptor_form'
# Shared predicate home + the md1-HRP probe to MIRROR:
git show origin/master:crates/mnemonic-toolkit/src/wallet_import/pipeline.rs | grep -n 'pub(crate) fn is_bip388_policy_shape\|pub(crate) fn is_at_n_form'
git show origin/master:crates/mnemonic-toolkit/src/cmd/xpub_search/descriptor_intake.rs | grep -n 'all(|t| t.to_lowercase().starts_with("md1"))'
# Pointer-surface VERIFICATION (the Important R0 fold): confirm --descriptor lives on the account-of-descriptor SUBCOMMAND, not bare xpub-search:
git show origin/master:crates/mnemonic-toolkit/src/cmd/xpub_search/mod.rs | grep -n 'command: XpubSearchCommand\|AccountOfDescriptor'
git show origin/master:crates/mnemonic-toolkit/src/cmd/xpub_search/account_of_descriptor.rs | grep -n 'pub descriptor: Option<String>'
```

If any anchor moved, adjust the edit to the live location; do **not** trust the snapshot lines.

---

## 1. Problem / current behavior (verified)

An md1 card is a `descriptor-mnemonic` **engraved card**, not a raw descriptor. On `xpub-search` it is auto-detected and accepted (`cmd/xpub_search/descriptor_intake.rs::detect_shape` @ :146; md1-HRP all-tokens probe at :156 — `tokens.iter().all(|t| t.to_lowercase().starts_with("md1"))`). The `--descriptor` flag that accepts an md1 card is on the **`account-of-descriptor` subcommand** (`account_of_descriptor.rs:79` `pub descriptor: Option<String>`, auto-detect via `detect_shape`), NOT a bare `xpub-search` flag — see §2.2(d).

But on:

- `export-wallet --descriptor <md1…>` — intake at `cmd/export_wallet.rs:487`. An md1 string is **not** `{`-prefixed (so `is_bip388_policy_shape` @ :494 = false) and contains no `@<digit>` (so `is_at_n_form` @ :503 = false), so it falls through to `MsDescriptor::<DescriptorPublicKey>::from_str(desc)` @ **:512**, which fails with an **opaque rust-miniscript parse error** wrapped as `ToolkitError::DescriptorParse(format!("export-wallet --descriptor: {e}"))` @ :513.
- `bundle --descriptor <md1…>` / `bundle --descriptor-file <file-of-md1>` — intake at `cmd/bundle.rs:313-344`. The body is materialized at :316-325 (`let body = match (&args.descriptor, &args.descriptor_file) { … };`), `is_bip388_policy_shape` @ :332 = false, then `classify_descriptor_form(&body)` @ :338 runs the `@N`/key probes. An md1 card has neither `@N` placeholders nor `[fp/path]`-annotated keys, so today it lands in the `(false,false)` branch of `classify_descriptor_form` (`wallet_import/pipeline.rs`) and emits a misleading message.

  **Pre-fix-message caveat (Minor R0 fold — narrative accuracy only):** *which* misleading message a given md1 card currently receives on the bundle path is **fixture-dependent**, governed by whether the card's base32 payload happens to embed a `*pub` substring matched by `has_any_key_token` (`pipeline.rs:63-72`, regex `[xtyzuvYZUV]pub[…]|…`):
  - If the payload contains **no** `*pub` substring (e.g. the chosen fixture `md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np` — 0 regex hits), `has_any_key_token == false` → the **keyless-script else-arm** (`"keyless script … route to export-wallet"`), which is wrong for an md1 card (it is an encoded card, not keyless).
  - If a *different* md1 card's payload embeds `ypub`/`zpub`/`vpub`, `has_any_key_token` would instead trip the `(false,false)` THEN-arm (`"descriptor has neither @N … nor [fp/path]"`).

  The new pre-check (§2.4) fires **before** `classify_descriptor_form`, so it supersedes **both** arms — the fix is correct regardless of payload. The §3.4 `!stderr.contains("keyless script")` assertion remains meaningful **for the chosen no-`*pub` fixture** (confirmed 0 regex hits), which is why that specific fixture is pinned.

Net: a user who copy-pastes an engraved md1 card into `--descriptor` on either surface gets a cryptic or actively-wrong message instead of "this is a card, decode it first."

This is already a hard error today on both surfaces — **the fix only replaces the message, never accepts new input** → low blast radius.

---

## 2. Design — ONE shared predicate, called on BOTH intake paths

There is no single function both `export_wallet.rs` and `bundle.rs` already funnel `--descriptor` through; they each independently call `is_bip388_policy_shape`. To honor "place the check ONCE," **single-source the predicate** in `wallet_import/pipeline.rs` (both files already `use crate::wallet_import::pipeline::…`) and call it at the head of each intake branch. The predicate mirrors `detect_shape`'s md1-HRP probe so detection logic is not duplicated by hand.

### 2.1 New shared predicate — `crates/mnemonic-toolkit/src/wallet_import/pipeline.rs`

Add **after** `is_bip388_policy_shape` (snapshot starts :244), before `expand_bip388_policy` (snapshot :259):

```rust
/// True iff `s` is an md1 `descriptor-mnemonic` CARD (every whitespace-separated
/// token, case-insensitively, begins with the `md1` HRP). Mirrors the md1 funnel
/// in `cmd/xpub_search/descriptor_intake.rs::detect_shape` (case-insensitive PROBE;
/// md-codec remains the case authority). MUST be checked BEFORE
/// `is_bip388_policy_shape` / `is_at_n_form` / `classify_descriptor_form` on the
/// `export-wallet`/`bundle` `--descriptor` intake so an md1 card gets a clear
/// "decode it first" pointer instead of an opaque miniscript parse error
/// (`export-wallet-bundle-descriptor-md1-clearer-error`).
pub(crate) fn is_md1_card(s: &str) -> bool {
    let tokens: Vec<&str> = s.split_whitespace().collect();
    !tokens.is_empty() && tokens.iter().all(|t| t.to_lowercase().starts_with("md1"))
}

/// Refuse an md1 card on a raw-descriptor intake with a typed, surface-pointing
/// error. Returns `Ok(())` for non-md1 input (callers proceed to their existing
/// shape probes). `surface` is the human CLI name for the error text.
pub(crate) fn reject_md1_card(s: &str, surface: &'static str) -> Result<(), ToolkitError> {
    if is_md1_card(s) {
        return Err(ToolkitError::Md1CardNotADescriptor { surface });
    }
    Ok(())
}
```

**Why a separate `is_md1_card` + `reject_md1_card`:** `is_md1_card` is independently unit-testable and matches the existing `is_bip388_policy_shape` / `is_at_n_form` predicate style in this module; `reject_md1_card` is the one-line call the two intake sites share. Implementer MAY inline `is_md1_card` into `reject_md1_card` if R0 prefers fewer surface fns — but keep ONE `is_md1_card` predicate so the probe is single-sourced. Confirm `ToolkitError` is already in scope in `pipeline.rs` (it is — `classify_descriptor_form`/`expand_bip388_policy` return it).

### 2.2 New typed error variant — `crates/mnemonic-toolkit/src/error.rs`

**ALPHABETICAL insertion point (CLAUDE.md mandate):** variant name `Md1CardNotADescriptor` sorts between `Io` and `MdCodec` (ASCII: `Md1` < `MdCodec` because `'1'` 0x31 < `'C'` 0x43; and `Io` < `Md1`). Insert in **all four** sites at exactly that position — the enum decl + the **three exhaustive** `match self` blocks. (The `details()` block uses a `_ => None` catch-all and is NOT exhaustive → **no arm needed** there; the new variant carries no structured details.)

**(a) Enum declaration** — after `Io(std::io::Error),` (snapshot :257), before `MdCodec(md_codec::Error),` (:258):

```rust
    /// An md1 `descriptor-mnemonic` CARD was passed where a raw descriptor is
    /// expected (`export-wallet`/`bundle` `--descriptor`). md1 is an engraved
    /// card, not a descriptor — decode it first. `surface` names the CLI context.
    Md1CardNotADescriptor {
        surface: &'static str,
    },
```

**(b) `exit_code` block** — after `ToolkitError::Io(_) => 1,` (snapshot :588), before `ToolkitError::MdCodec(e) => md_codec_exit_code(e),` (:589):

```rust
            ToolkitError::Md1CardNotADescriptor { .. } => 2,
```

Rationale for **2**: peers in this "input is structurally a refused descriptor-intake form" class (`DescriptorParse`, `ImportWalletParse`, `TemplateFormUnsupportedShape`) all use exit 2. Today the md1→opaque path already exits 2 (via `DescriptorParse`) on export-wallet, so 2 is also behavior-preserving for that surface. Implementer: re-confirm against the live `exit_code` table.

**(c) `kind` block** — after `ToolkitError::Io(_) => "Io",` (snapshot :657), before `ToolkitError::MdCodec(_) => "MdCodec",` (:658):

```rust
            ToolkitError::Md1CardNotADescriptor { .. } => "Md1CardNotADescriptor",
```

**(d) `message` block** — after `ToolkitError::Io(e) => format!("I/O error: {e}"),` (snapshot :831), before `ToolkitError::MdCodec(e) => crate::friendly::friendly_md_codec(e),` (:832):

```rust
            ToolkitError::Md1CardNotADescriptor { surface } => format!(
                "{surface}: this is an md1 descriptor-mnemonic CARD, not a raw descriptor. \
                 md1 cards are not accepted on --descriptor. Decode the card first \
                 (`md decode <md1…>` / `mnemonic restore --md1 <md1…>`), or to search an \
                 xpub from the card use \
                 `mnemonic xpub-search account-of-descriptor --descriptor <md1…>`."
            ),
```

> **Important R0 fold — the pointer MUST name the subcommand.** `xpub-search` is a **parent command** with four subcommands (`path-of-xpub`, `account-of-descriptor`, `address-of-xpub`, `passphrase-of-xpub`); the `--descriptor` flag that accepts md1 cards lives on `account-of-descriptor` (`account_of_descriptor.rs:79`; md1 reaches it via `descriptor_intake::detect_shape`). The bare form `mnemonic xpub-search --descriptor <md1…>` yields a clap *"unrecognized argument"* error — it points the user at a surface that does not work. The message therefore names the full working form `mnemonic xpub-search account-of-descriptor --descriptor <md1…>`. (The other two pointers are real: `md decode <md1…>` — md-cli `main.rs` `Decode`; `mnemonic restore --md1 <md1…>` — `restore.rs:89` `pub md1: Vec<String>`.)

Message constraints (load-bearing for the test assertions in §3):
- MUST contain the substring **`md1 descriptor-mnemonic CARD`** (the test asserts this exact phrase).
- MUST contain the WORKING pointer substring **`xpub-search account-of-descriptor`** — NOT bare `xpub-search` (bare matches the broken pointer too; the Important R0 fold tightens this so the regression test pins a real surface).
- MUST contain the `{surface}` prefix so the message self-identifies which command refused.
- Do NOT echo the full card string (avoid a long opaque dump; the message is generic). Per existing `UnknownHrp` hygiene precedent, never print the whole positional.

### 2.3 Call site — `export-wallet` (`crates/mnemonic-toolkit/src/cmd/export_wallet.rs`)

Insert the pre-check as the **first** statement inside `if let Some(desc) = &args.descriptor {` (snapshot :487), BEFORE the `is_bip388_policy_shape` branch (:494):

```rust
    let canonical = if let Some(desc) = &args.descriptor {
        // md1 cards are engraved descriptor-mnemonic CARDS, not raw descriptors —
        // refuse with a surface-pointing message instead of the opaque miniscript
        // parse error at MsDescriptor::from_str below
        // (`export-wallet-bundle-descriptor-md1-clearer-error`).
        crate::wallet_import::pipeline::reject_md1_card(desc, "export-wallet --descriptor")?;
        // BIP-388 wallet-policy JSON intake: …(existing comment)…
```

### 2.4 Call site — `bundle` (`crates/mnemonic-toolkit/src/cmd/bundle.rs`)

Insert immediately AFTER the `body` is materialized (snapshot :316-325, the `let body = match (&args.descriptor, &args.descriptor_file) { … };`) and BEFORE the `is_bip388_policy_shape` reassignment (snapshot :332). This placement catches both `--descriptor` AND `--descriptor-file`-supplied md1 cards (a card pasted into a file is equally confusing):

```rust
        // md1 cards are engraved descriptor-mnemonic CARDS, not raw descriptors —
        // refuse with a surface-pointing message instead of letting an md1 card
        // fall into classify_descriptor_form's misleading "keyless script" arm
        // (`export-wallet-bundle-descriptor-md1-clearer-error`). Covers both
        // --descriptor and --descriptor-file (body already merged above).
        crate::wallet_import::pipeline::reject_md1_card(&body, "bundle --descriptor")?;
        // BIP-388 wallet-policy JSON intake: expand to a concrete descriptor … (existing comment)
        let body = if crate::wallet_import::pipeline::is_bip388_policy_shape(&body) {
```

**Surface label (Minor R0 fold — author's stable-label choice retained, rationale recorded):** use label `"bundle --descriptor"` on both inline and file inputs. The refusal is about the *form*, not which flag carried it, and a stable label keeps the test assertion simple. **Known cosmetic:** an md1 card pasted into a FILE (`--descriptor-file`) is still labeled `bundle --descriptor:`, which could briefly confuse a user about which flag they used. The refusal + working pointer are still clear, so this is accepted. (Alternative the implementer MAY take if R0 prefers: a flag-neutral label `"bundle descriptor intake"`; if chosen, update the §3.4 assertion accordingly. Default = `"bundle --descriptor"`.) Implementer: re-confirm the `let body = match …` block is still the body-materialization site and that the `is_bip388_policy_shape(&body)` reassignment is still the immediately-following statement.

---

## 3. Test / parity surface

### 3.1 Unit test — `wallet_import/pipeline.rs` `#[cfg(test)]`

Add to the existing test module (which already asserts `!is_bip388_policy_shape("md1qpwmxpzqqsrd")`):

```rust
#[test]
fn is_md1_card_detects_md1_hrp_and_rejects_descriptors() {
    assert!(is_md1_card("md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np"));
    assert!(is_md1_card("  MD1ABCDE  "));                  // case-insensitive + trim
    assert!(is_md1_card("md1aaa md1bbb"));                 // multi-token (chunked card)
    assert!(!is_md1_card(""));                             // empty → not a card
    assert!(!is_md1_card("wpkh([00000000/84h/0h/0h]xpub…/<0;1>/*)")); // concrete
    assert!(!is_md1_card("{\"name\":\"x\"}"));             // bip388 JSON
    assert!(!is_md1_card("wpkh(@0/<0;1>/*)"));             // @N template
    assert!(!is_md1_card("md1abc wpkh(@0/**)"));           // mixed → NOT all-md1
}
```

### 3.2 New error-table rows — `error.rs` `#[cfg(test)]`

Add a row to the existing `exit_code_table_per_variant` and `kind_strings_stable` tests:

```rust
// exit_code_table_per_variant:
assert_eq!(
    ToolkitError::Md1CardNotADescriptor { surface: "export-wallet --descriptor" }.exit_code(),
    2,
);
// kind_strings_stable:
assert_eq!(
    ToolkitError::Md1CardNotADescriptor { surface: "x" }.kind(),
    "Md1CardNotADescriptor",
);
```

### 3.3 CLI cell — export-wallet (`crates/mnemonic-toolkit/tests/cli_export_wallet.rs`)

Use the real fixture card `md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np` (verified present in `tests/cli_repair.rs:295`). Mirror the existing `threshold_greater_than_cosigner_count_refusal` harness:

```rust
/// `export-wallet-bundle-descriptor-md1-clearer-error`: an md1 CARD passed to
/// `--descriptor` gets a clear surface-pointing refusal, NOT an opaque
/// rust-miniscript parse error.
#[test]
fn md1_card_on_descriptor_clear_refusal() {
    let md1 = "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np";
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["export-wallet", "--descriptor", md1, "--network", "mainnet", "--format", "descriptor"])
        .assert().failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("md1 descriptor-mnemonic CARD"), "stderr: {stderr:?}");
    // Important R0 fold: gate the WORKING pointer (subcommand-qualified), not bare `xpub-search`.
    assert!(stderr.contains("xpub-search account-of-descriptor"),
        "must point at the REAL accepting surface (subcommand-qualified): {stderr:?}");
    assert!(!stderr.to_lowercase().contains("unexpected"), "must NOT be the opaque miniscript msg: {stderr:?}");
}
```

(Implementer: confirm the exact `export-wallet` required args via `mnemonic export-wallet --help`; `--format descriptor` + `--network mainnet` is the minimal watch-only descriptor-passthrough invocation. The cell asserts on `.failure()` + message substrings, NOT a fixed exit code, to stay robust — though exit 2 is expected.)

### 3.4 CLI cell — bundle (`crates/mnemonic-toolkit/tests/cli_bundle_keyless_descriptor.rs`)

This file already exercises the bundle `--descriptor` refusal/keyless path (recon-named sibling), so the cell belongs here. The `!stderr.contains("keyless script")` assertion is meaningful for the pinned no-`*pub` fixture (see §1 caveat — confirmed 0 `has_any_key_token` regex hits):

```rust
/// `export-wallet-bundle-descriptor-md1-clearer-error`: an md1 CARD passed to
/// `bundle --descriptor` gets a clear surface-pointing refusal, NOT the
/// misleading classify_descriptor_form "keyless script" message.
#[test]
fn bundle_md1_card_on_descriptor_clear_refusal() {
    let md1 = "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np";
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["bundle", "--descriptor", md1, "--network", "mainnet"])
        .assert().failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("md1 descriptor-mnemonic CARD"), "stderr: {stderr:?}");
    // Important R0 fold: gate the WORKING pointer (subcommand-qualified), not bare `xpub-search`.
    assert!(stderr.contains("xpub-search account-of-descriptor"), "stderr: {stderr:?}");
    assert!(!stderr.contains("keyless script"), "must NOT be classify_descriptor_form's keyless msg: {stderr:?}");
}
```

(Implementer: confirm minimal `bundle --descriptor` invocation via `mnemonic bundle --help`; add `--network mainnet` if required. The cell must reach the descriptor-mode intake — confirm no earlier mode-violation guard fires first; the new pre-check sits right after body-materialization, so for `descriptor_mode` it fires before any slot validation. If the implementer adopts the §2.4 flag-neutral label alternative, update the substring expectation in the message accordingly — default keeps `"bundle --descriptor"`.)

### 3.5 Full suite (MANDATORY — not targeted)

```
cargo test -p mnemonic-toolkit          # FULL package suite per CLAUDE.md / MEMORY (stale-lint class)
cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings
```

Per MEMORY `feedback_r0_review_run_full_package_suite`: CLI/error-surface phases ripple into argv/schema/version lints outside any one target — run the **whole** package suite, never a single `--test`.

---

## 4. Regression-risk handling

- **No golden pins the opaque string** — verified: `cli_export_wallet.rs` @940abe9e contains no assertion on the `export-wallet --descriptor: <miniscript error>` text. The new branch only changes the message for md1-HRP input, which is **already a hard error** on both surfaces.
- **New branch fires ONLY on all-tokens-`md1` input** — any concrete descriptor (`wsh(...)`, `wpkh(...)`, `tr(...)`), any `{`-prefixed BIP-388 policy JSON, any `@N` template, and any mixed `md1 …non-md1…` string returns `Ok(())` from `reject_md1_card` and proceeds to the existing probes unchanged. Covered by the §3.1 negative unit cases.
- **Pointer points at a REAL surface (Important R0 fold):** the message names `mnemonic xpub-search account-of-descriptor --descriptor <md1…>` and the §3.3/§3.4 cells gate the substring `xpub-search account-of-descriptor`, so a future regression that drops the subcommand qualifier (re-introducing the broken bare pointer) fails the suite.
- **Exhaustiveness:** exactly **three** `match self` blocks in `error.rs` are exhaustive (`exit_code`, `kind`, `message`); each gets one new arm at the alphabetical slot. `details()` is non-exhaustive (`_ => None`) → intentionally untouched. The compiler enforces this — a missed exhaustive arm fails to build.
- **Alphabetical ordering** (CLAUDE.md merge-conflict mitigation): `Md1CardNotADescriptor` placed between `Io` (`:257`/`:588`/`:657`/`:831`) and `MdCodec` (`:258`/`:589`/`:658`/`:832`) in the enum and all three match blocks. Re-verify the live neighbors at write time (other concurrent variants may shift the window).
- **No secret-hygiene exposure:** the message does not echo the card payload (md1 cards are public, but keeping the message generic also avoids a long opaque dump and matches the `UnknownHrp` truncation precedent).
- **Surface label coupling:** the `surface: &'static str` is set by each call site (`"export-wallet --descriptor"` / `"bundle --descriptor"`); no runtime allocation, no Display drift. The `--descriptor-file` case is intentionally labeled `bundle --descriptor` (stable label; cosmetic mismatch accepted — see §2.4).

---

## 5. Out of scope (explicit DEFER per the lane decision)

- **`bip388-template-path-wallet-name`** — NOT this lane (recon BATCH-DOC item). Do not touch `bip388.rs` / wallet-name threading.
- **`verify-message-format-requested-debug-string`** — NOT this lane (recon DEFER, latent-only).
- **md1 *acceptance*** on `--descriptor` (decode-and-expand the card in place) — explicitly NOT done; the decision is a clear *refusal* with a pointer, not a new acceptance path (that would be a feature, not a diagnostics fix, and would need wire/round-trip review).

---

## 6. Implementer checklist (TDD order)

1. `cargo test -p mnemonic-toolkit` baseline GREEN.
2. RED: add §3.1 unit test + §3.3/§3.4 CLI cells (they fail to compile / fail assertions).
3. GREEN: add `is_md1_card` + `reject_md1_card` (§2.1), the `Md1CardNotADescriptor` variant + 3 match arms (§2.2 — note the subcommand-qualified pointer in the message), the two call sites (§2.3, §2.4). Add §3.2 error-table rows.
4. Re-run FULL `cargo test -p mnemonic-toolkit` + clippy `-D warnings`.
5. Confirm alphabetical placement in enum + all three match blocks; confirm `details()` untouched.
6. Confirm the message string contains `xpub-search account-of-descriptor` (not bare `xpub-search`) — manually run `mnemonic xpub-search account-of-descriptor --descriptor md1…` once to confirm the pointed-to form is a real (non-clap-error) surface.
7. No version bump, no README/Cargo.toml/manual/schema edits, no tag.
8. Flip `design/FOLLOWUPS.md:4092` Status `open → resolved` in the shipping commit (per MEMORY `feedback_followup_status_discipline`).
