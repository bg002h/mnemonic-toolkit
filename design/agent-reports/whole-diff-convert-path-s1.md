**⚠ 2 Important: I1 `--path ""`/`m/`/unprefixed accepted (master key printed, "literal m/... only" false); I2 fingerprint-only advice loop.**

# Whole-diff review: tk convert --path, slice s1 (executed advice, design, threat)

Target: `fix/convert-path` `6ccfdebc..0a354375`. Reviewer: Opus 5.5 (s1 lens only).

## Findings (appended as found)


Scratch copy: `/scratch/cache/convert-path/wd-s1/` (tools, logs and fix diffs in `tools/`).
Independent reference: `tools/bip32ref.py` (stdlib BIP-39 PBKDF2 + BIP-32 HMAC-SHA512 + secp256k1).

### I1 (Important): `--path ""`, `--path m/` and an unprefixed path are accepted; the "literal `m/...` only" texts are false and an empty path silently yields the master key

- Anchors: `crates/mnemonic-toolkit/src/cmd/convert.rs:1342` (seed xpub/xprv arm, new in this diff) and the
  same parse at :1421 (wif), :1476 (bip38), :1517 (seed address), :1578 (xpub -> address); spec
  `design/SPEC_convert_v0_6.md:231` ("accepts only the literal form `m/...` ... any other string is
  refused"), S7 row, manual `docs/manual/src/40-cli-reference/41-mnemonic.md:1102` ("literal `m/...`
  only"), `--path` help ("literal form `m/...`").
- Cause: `bitcoin::bip32::DerivationPath::from_str` (0.32.8, read in the registry source) accepts `""`
  and `"m/"` as the master path and accepts a path with no `m/` prefix; the diff relies on it alone.
- Observed on 0a354375 (signet, `abandon ... about`):
  - `--to xpub --path ""` -> exit 0, `xpub: tpubD6NzVbkrYhZ4XYa9MoLt4BiMZ4gkt2faZ4BcmKu2a9te4LDpQmvEz2L2yDERivHxFPnxXXhqDRkUNnQCpZggCyEZLBktV7VaSmwayqMJy1s` (the master xpub; reference agrees);
  - `--to wif --path m/` -> exit 0, `wif: cNPn53CWHSMt3MMr3HrymFzxaeDwZrZF2yAEZJ7knzAFD3GTTi2x` (master key);
  - `--from xpub=... --to address --path ""` -> exit 0, `tb1qaxdcv2px5s9r9sjv09u96p43nh3lkpm062kp6s`;
  - `--to xpub --path "84'/1'/0'"` -> exit 0, `XPUB_84`.
- Failure: a refusal that does not refuse plus a wrong key. A script `--path "$P"` with `P` unset
  prints the master xprv/xpub (or the master-key wif/address) with exit 0 instead of the stated exit-1
  refusal. For seed `--to xpub/xprv` this exposure is new: before this diff `--path` was ignored there.
- Fix text (convert.rs): insert before `fn refusal_address_no_path` and replace each of the five
  `bip32::DerivationPath::from_str(path_str).map_err(|e| ToolkitError::BadInput(format!("--path parse: {e}")))?`
  with `parse_literal_path(path_str)?`:
  ```rust
  /// `--path` accepts only the literal form `m` or `m/<child>...` (SPEC_convert_v0_6
  /// §4); `DerivationPath::from_str` alone also accepts "", "m/" and a missing `m/`.
  fn parse_literal_path(s: &str) -> Result<bip32::DerivationPath, ToolkitError> {
      let path = bip32::DerivationPath::from_str(s)
          .map_err(|e| ToolkitError::BadInput(format!("--path parse: {e}")))?;
      if s != "m" && !(s.starts_with("m/") && s.len() > 2) {
          let e = bip32::Error::InvalidDerivationPathFormat;
          return Err(ToolkitError::BadInput(format!("--path parse: {e}")));
      }
      Ok(path)
  }
  ```
  (Parse first, then check the form, so the existing `r5_named_path_form_refused` /
  `r5_hex_path_form_refused` keep `invalid child number format`.) Spec :231 stays true as written:
  the new `<e>` is a genuine `bitcoin::bip32` error, `invalid derivation path format`. FIX doc error
  table (lines 34/47, "anything `DerivationPath::from_str` refuses") gains: "or anything other than `m`
  or `m/<child>...` (e.g. `""`, `m/`, `84'/1'/0'`): `error: --path parse: invalid derivation path format` (exit 1)".
- RED rows (added to `tests/cli_convert_path.rs`, run as printed):
  `const R5_FORM_TEXT: &str = "error: --path parse: invalid derivation path format";`
  - `r5_empty_path_refused`: `--from <phrase()> --network signet --to xpub --path ""` -> `assert_refused(&o, 1, R5_FORM_TEXT)` and `assert!(o.stdout.is_empty())`;
  - `r5_m_slash_path_refused`: same with `--to wif --path "m/"`;
  - `r5_unprefixed_path_refused`: same with `--to xpub --path "84'/1'/0'"`.
- Observed RED on unmodified 0a354375 (`binary(cli_convert_path)`): 33 run, 30 passed, 3 failed:
  `r5_empty_path_refused` "stdout: xpub: tpubD6NzVbkrYhZ4XYa9…"; `r5_m_slash_path_refused`
  "stdout: wif: cNPn53CWHSMt3MMr…"; `r5_unprefixed_path_refused` "stdout: xpub: tpubDC8msFGeGuwn…".
  GREEN with the fix (`binary(cli_convert_path) | test(/convert/) | test(/path/)`): 320/320 passed;
  `cargo clippy --locked --all-targets -- -D warnings` rc 0; `cargo fmt --check` clean; the executed
  matrix below still 1792/0.

### I2 (Important): the fingerprint-only advice loops; the no-template refusal names `--path`, which R4 then refuses

- Anchors: `convert.rs:1362-1367` (the `else if needs_derive` refusal text, changed in this diff to
  name `--path`); spec `SPEC_convert_v0_6.md:232-235`; FIX doc lines 32, 44, 225-227.
- Followed literally on 0a354375 (signet, phrase via `--from phrase=-`):
  1. `--to fingerprint` -> exit 1, `error: --template or --path is required for derivation targets (xpub/xprv/fingerprint)`.
  2. Following it, `--to fingerprint --path m/0` -> exit 2, `error: --path is not used by any requested --to target from this --from source; drop --path.` (R4).
  3. Following that, `--to fingerprint` -> step 1 again. Only `--template bip84` (never named as the
     sole way out) works: `fingerprint: 73c5da0a`.
- Failure: a printed remedy fails as printed (fails closed, but the likelihood test keeps it
  Important: a printed command fails, and `--to fingerprint` from a phrase is an ordinary request,
  not two unusual conditions). Before this diff the text named `--template` only and was true.
- Fix text (convert.rs, `else if needs_derive` branch), replacing the `let template = ...` statement:
  ```rust
                  // A fingerprint-only request does not read --path (R4), so the
                  // refusal names --template alone there.
                  let reads_path = targets.iter().any(|t| matches!(t, Xpub | Xprv));
                  let template = args.template.ok_or_else(|| {
                      ToolkitError::BadInput(if reads_path {
                          "--template or --path is required for derivation targets (xpub/xprv/fingerprint)"
                              .into()
                      } else {
                          "--template is required for --to fingerprint (the master fingerprint; --path is not read)"
                              .into()
                      })
                  })?;
  ```
  Spec :232 bullet: replace "A seed source with a derivation target (`xpub`/`xprv`/`fingerprint`) and
  neither `--template` nor `--path`" with "A seed source with an `xpub` or `xprv` target and neither
  `--template` nor `--path`" (text block unchanged), and add the bullet: "A seed source whose only
  derivation target is `fingerprint`, without `--template`: REFUSED (exit 1). Stderr (byte-exact):
  `error: --template is required for --to fingerprint (the master fingerprint; --path is not read)`".
  Same split in the FIX doc's error table (line 44) and S-text (225-227).
  (Alternative for the architect, not prescribed: drop the template requirement for a
  fingerprint-only request, since the value is the master fingerprint whatever the template; that is a
  behaviour change, so the text fix above is the minimal one.)
- RED row (added, run as printed):
  ```rust
  #[test]
  fn r4_fingerprint_only_no_template_names_template_only() {
      let p = phrase();
      let o = convert(&["--from", &p, "--network", "signet", "--to", "fingerprint"]);
      assert_refused(
          &o,
          1,
          "error: --template is required for --to fingerprint (the master fingerprint; --path is not read)",
      );
      assert!(o.stdout.is_empty(), "stdout: {}", o.stdout);
  }
  ```
  Observed RED (code with I1's fix only): 1 run, 0 passed, 1 failed: "missing `error: --template is
  required for --to fingerprint (the master fingerprint; --path is not read)` in stderr: ... error:
  --template or --path is required for derivation targets (xpub/xprv/fingerprint)". GREEN with both
  fixes (`binary(cli_convert_path) | test(/convert/) | test(/path/) | test(/template/) |
  test(/fingerprint/)`): 527/527 passed; clippy `-D warnings` rc 0; fmt clean. Live after the fix:
  `--to fingerprint` -> the new text; `--to fingerprint --template bip84` -> `fingerprint: 73c5da0a`;
  `--to xpub,fingerprint` -> the unchanged "--template or --path" text; `--to fingerprint --path m/0`
  -> R4. The existing `r2_neither_template_nor_path_names_both` (`--to xpub`) is unaffected.

### Minors

- M1: "BIP-32 path from the master key" (help, manual :1102, spec S7) is false for the `--from xpub
  --to address` edge that `reads_path` includes, where the path is relative to the xpub. Following it
  literally (`--path m/84'/1'/0'/0/0`) exits 1 with "BIP-32 error: cannot derive hardened key from
  public key" (fails closed; conditions: (1) an xpub source, (2) the address target, (3) applying the
  master-key wording there). Wording: "BIP-32 path from the master key (from an xpub source: relative
  to that xpub, e.g. `m/0/0`)".

### Nits

- N1: `assert_refused` (tests/cli_convert_path.rs) does not assert an empty stdout; the I1/I2 rows add it explicitly (s2 lens owns the rest).
- N2: pre-existing, outside the diff: signet `--to bip38` with `--path` gives "error: bip38: invalid wif private key" (bip38 arm unchanged; mainnet round-trips).

### Verified true (executed)

- Matrix `tools/matrix.py`: 1792 checks, 0 failures. Seeds `abandon…about` and `legal winner…` x
  passphrase ""/TREZOR x signet/mainnet; paths m, m/0, m/0', m/84'/1'/0', m/48'/1'/0'/2', m/87h/1h/0h,
  m/84'/1'/0'/0/7, m/2147483647'/1; sources phrase/entropy/seedqr; targets xpub, xprv, wif,
  fingerprint, template+path, address. Each key equals the independent reference; the fingerprint is
  always the master fingerprint (R2); `--account 0` with `--path` accepted; `--account 3` with `--path`
  refused exit 2 with the R3 text (R3 before R5, R4 before R3 per QT1-QT3); R4 for fingerprint-only,
  xprv-source and xpub-source fingerprint exit 2; neither flag exit 1; every refusal has empty stdout.
- `--to wif`, seed `--to address` and composite `--to bip38` without `--path` refuse (exit 2), as the
  help/manual "required for" says. R3's printed example `m/84'/1'/3'` with `--to xpub` gives the
  account-3 key the reference derives.
- `--xpub-prefix zpub` with signet `--path m/84'/1'/0'` gives a vpub whose payload equals `XPUB_84`;
  `--json` values equal the text values; a Japanese phrase matches the reference; mainnet bip38 via
  `--path` decrypts to the same wif as `--to wif`.

C / I / M / N: 0 / 2 / 1 / 2
