## R0 Review — Wave-4 L2 `export-wallet-bundle-descriptor-md1-clearer-error`

**Verdict: GREEN (0 Critical / 0 Important).** Cleared to implement.

All claims re-grepped against working tree @ `940abe9e7cbf55ab005f3aae6541ec42ab7dbd69` (confirmed `git rev-parse HEAD` == the spec's cited SHA; clean of the three untracked recon `.md` files only). Crate path is `crates/mnemonic-toolkit/src/...` (cwd is the repo root).

### Mandated verifications — all PASS

**1. md1-HRP pre-check covers BOTH export-wallet AND bundle — PASS.**
- export-wallet: intake `if let Some(desc) = &args.descriptor` confirmed; `is_bip388_policy_shape` @ `export_wallet.rs:494`, `is_at_n_form` @ `:503`, `MsDescriptor::from_str` @ `:512`, opaque `DescriptorParse(format!("export-wallet --descriptor: {e}"))` @ `:513` — all snapshot lines match exactly. §2.3 inserts `reject_md1_card(desc, "export-wallet --descriptor")?` as the first statement, before `:494`. Correct.
- bundle: body materialized at `bundle.rs:316-325` (`let body = match (&args.descriptor, &args.descriptor_file) {…}`), `is_bip388_policy_shape(&body)` reassignment @ `:332`, `classify_descriptor_form(&body)?` @ `:338`. §2.4 inserts the pre-check between `:325` and `:326` — **inside `if descriptor_mode {` (line 313), after the merge of BOTH `--descriptor` and `--descriptor-file`** — so both inline and file-supplied md1 cards are covered. Verified no earlier guard (the `ModeViolation` pre-checks at 262-311) intercepts an md1 string: they only gate flag *combinations*, never inspect descriptor content. Correct.
- Confirmed md1 is **already a hard error today on the bundle path**: an md1 card → `classify_descriptor_form` `(false,false)` branch → `Err` propagated by the `?` at `:338`. Low blast radius claim holds.

**2. New variant alphabetically ordered in ALL match arms — PASS.**
- Enum decl: `Io(std::io::Error)` @ `:257`, `MdCodec(md_codec::Error)` @ `:258`. `Md1...` sorts between (`'1'`=0x31 < `'C'`=0x43; `Io` < `Md1`). Correct.
- Exactly THREE exhaustive `match self` blocks (compiler-enforced, no catch-all): `exit_code` (550-606, `Io` @ 588 / `MdCodec` @ 589), `kind` (613-677, `Io` @ 657 / `MdCodec` @ 658), `message` (683-902, `Io` @ 831 / `MdCodec` @ 832). Each needs one new arm at the alphabetical slot — spec §2.2(b/c/d) places them exactly there.
- `details()` (908-933) ends in `_ => None` @ 932 → **non-exhaustive → correctly NO arm** (spec is right; the new variant carries no structured details).
- `impl Display` (937-941) delegates to `self.message()` — **no separate exhaustive match**, so no fourth arm needed. Spec's exhaustiveness inventory is complete and correct.
- Exit code **2** verified behavior-preserving: `DescriptorParse` @ `:571` = 2 (today's export-wallet md1 path) and peers `ImportWalletParse`/`TemplateFormUnsupportedShape` = 2.

**3. No golden pins the old opaque string — PASS.** `grep -rn 'export-wallet --descriptor:' crates/mnemonic-toolkit/tests/` → zero hits. The new branch fires only on all-tokens-`md1` input; no existing assertion regresses.

**4. The typed refusal is genuinely clearer and points to a REAL working surface — PASS (this is the spec's strongest point).**
- `account-of-descriptor` is a real `XpubSearchCommand` subcommand (`xpub_search/mod.rs:66` `AccountOfDescriptor(...)`), and `--descriptor` lives on `AccountOfDescriptorArgs` (`account_of_descriptor.rs:79`, `#[arg(long)]`). Its own doc-comment (`:73-74`) states the shape is auto-detected including **"md1 card"** — so `mnemonic xpub-search account-of-descriptor --descriptor <md1…>` genuinely accepts the card. The Important R0 fold (subcommand-qualified pointer; bare `xpub-search --descriptor` would be a clap "unrecognized argument" error) is correct and load-bearing.
- The other two pointers verified real: `restore --md1` (`restore.rs:89` `pub md1: Vec<String>`); `md decode` (md-cli). 
- The mirrored detection probe `tokens.iter().all(|t| t.to_lowercase().starts_with("md1"))` matches `descriptor_intake.rs:156` exactly — `is_md1_card` is a faithful single-source of the existing funnel.

### Empirical fixture verification (the §1 narrative's correctness hinge)
Ran the actual regexes against the pinned fixture `md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np`:
- `has_any_key_token` (`pipeline.rs:63`, regex `[xtyzuvYZUV]pub…`) → **no match** (the `aypu34`/`u8np` substrings are NOT `ypub`/`upub`).
- `@\d` → no match; `is_md1_card` → **true**.
So this fixture lands in `classify_descriptor_form`'s **keyless else-arm** (`pipeline.rs:205-216`, "keyless script"). The §3.4 assertion `!stderr.contains("keyless script")` is therefore meaningful **for this exact fixture** — and the §1 caveat correctly scopes that claim to no-`*pub` payloads. Fixture present at `cli_repair.rs:295`.

### Infrastructure sanity
`ToolkitError` in scope at `pipeline.rs:16`. Test module `#[cfg(test)] mod tests` @ `:437` with `use super::*` (so §3.1 calls `is_md1_card` unqualified). Existing `!is_bip388_policy_shape("md1qpwmxpzqqsrd")` assertion @ `:666` (spec's "already asserts" claim true). Both CLI test files exist; sibling harnesses `threshold_greater_than_cosigner_count_refusal` (`cli_export_wallet.rs:618`) and the keyless `bundle --descriptor` pattern exist. `exit_code_table_per_variant` (`:1008`) and `kind_strings_stable` (`:1316`) are spot-check (non-exhaustive) tests → §3.2 rows are additive, not required for compilation. No name collisions for `is_md1_card` / `reject_md1_card` / `Md1CardNotADescriptor`.

### NO-BUMP / CI-coupling confirmation
Correct: pure internal typed-error + two shared `pub(crate)` helpers; **no clap flag/subcommand/dropdown/wire-shape change** → no GUI `schema_mirror` lockstep, no manual flag-coverage lint, no version-site/fmt/fuzz coupling, no argv-secret lint. The message does not echo the card payload (hygiene + `UnknownHrp` truncation precedent honored). §3.5 correctly mandates the FULL `cargo test -p mnemonic-toolkit` (per MEMORY `feedback_r0_review_run_full_package_suite`) + clippy `-D warnings`.

### Minor notes (no fold required — recorded for the implementer)
1. §3.4's bundle cell omits `--no-engraving-card` (both sibling tests include it). Harmless — the pre-check fires upstream of engraving logic; §3.4 already flags confirming the minimal invocation.
2. §2.4's `--descriptor-file`→`"bundle --descriptor:"` cosmetic label mismatch is explicitly accepted, with a documented flag-neutral alternative. The trim asymmetry (inline `s.clone()` vs file `trim_end()`) is irrelevant — `is_md1_card` uses `split_whitespace()`.
3. The new md1 message cannot false-match the existing `bundle_keyless_descriptor_routes_to_export_wallet` golden (different non-md1 input; new message lacks `no keys to engrave`).

**Gate: GREEN. Proceed to TDD implementation per §6.** Implementer must still honor §0's re-grep mandate at write time (lines decay), but as of @940abe9e every cited anchor is live.