# R0 Review — Round 1 — SPEC_bip388_policy_descriptor_expansion.md

> Persisted verbatim from the opus-architect R0 round-1 dispatch (Cycle D — bip388 wallet-policy → concrete-descriptor expansion). Persisted before fold per convention.

---

Confirmed: a single-key concrete descriptor (n=1) → `SingleSigWatchOnly` (`bundle.rs:1657`), NOT `MultisigWatchOnly`. So SPEC §1.3 line 68's claim "any descriptor shape ... → `MultisigWatchOnly`" is over-broad. A single-sig BIP-388 policy (e.g. `wpkh(@0/**)` with one key — which the emitter CAN produce since `wpkh(xpub/<0;1>/*)` is multipath) lands in `SingleSigWatchOnly`. That's still correct behavior (no break), just a wording imprecision → Minor. And the `account_of_descriptor_bip388_json_match` test (`:267`) already uses exactly a single-key `wpkh(@0/**)` policy, proving the single-key shape is real and round-trips. Good corroboration.

I have verified all the key citations and attack vectors. [final citation cross-check elided — all confirmed below]

---

# R0 Review Verdict — SPEC_bip388_policy_descriptor_expansion.md

## **YELLOW** — 0 Critical / 2 Important / 6 Minor

The design direction (option (a)) is sound and confirmed feasible. The ordering invariant (§2) is correct, the module extraction (§1.1) is clean (no import cycle, no serde/visibility problem, bare-`String` return is sufficient for all three callers since network/account/stderr are applied only in the refactored xpub-search caller post-expansion). The checksum concern on the bundle path is a **non-issue** — `verify_checksum` (miniscript `95fdd1c`, `checksum.rs:94-127`) returns `Ok(&s[..s.len()])` when no `#` is present, so a checksum-less expanded descriptor passes `descriptor_body_no_csum` cleanly. **No Critical defects.** Fold the two Important items and it is GREEN; no re-litigation of option (a) is warranted.

---

## CRITICAL (0)
None.

---

## IMPORTANT (2)

### I-1 — The §1.1 `BipPolicyJson` code block as written FAILS the `-D warnings` clippy gate
**Where:** SPEC §1.1 line 28: `struct BipPolicyJson { name: String, description_template: String, keys_info: Vec<String> }`.

The current code (`descriptor_intake.rs:85-90`) deliberately uses `#[serde(rename = "name")] _name: String` — the underscore prefix is what silences `dead_code` for a field that is deserialized but never read. CI runs `cargo clippy --all-targets -- -D warnings` (`.github/workflows/rust.yml:150`). The SPEC's proposed `name: String` (read nowhere) reintroduces a `dead_code` warning → clippy fails → blocks the §7 release gate.

The SPEC's prose at line 42 ("the `_name` field discard becomes `name` captured-but-unused") acknowledges the field is unused but does not address the lint, and the code block contradicts the lint requirement in §6/§7.

**Fix (specify exactly one, or it gets half-fixed into a parse break):**
- `#[allow(dead_code)] name: String` (no rename attr — JSON key `name` already matches the field name), **or**
- keep `#[serde(rename = "name")] _name: String`.

**Trap to call out in the fix:** `_name: String` **without** the `#[serde(rename="name")]` attribute is a parse-breaker — under `#[serde(deny_unknown_fields)]` it would reject the real `"name"` JSON key and demand a `"_name"` key, breaking every real policy. The rename attribute is load-bearing if the underscore form is kept.

### I-2 — §1.1's "regression-pinned by existing cells `descriptor_intake.rs:438-458`" is FALSE
**Where:** SPEC §1.1 line 42: "xpub-search behavior is unchanged (regression-pinned by its existing cells `descriptor_intake.rs:438-458` …)".

Verified: cells `:438-464` are **`detect_shape`-only** (`detect_shape_json_object_returns_bip388`, `_md1_token_`, `_at_n_placeholder_refused`, `_default_literal`) plus `contains_at_n_walker` (`:466`). **Nothing in `descriptor_intake.rs` pins `parse_bip388_json`'s actual substitution output.** The only expansion coverage is the integration cell `cli_xpub_search_account_of_descriptor.rs:262` (`account_of_descriptor_bip388_json_match`, a single-key `wpkh(@0/**)` happy path). So the SPEC's specific citation is wrong, and the claim that delegation is regression-safe rests on cells that don't test expansion.

This matters because the delegation also changes one behavior the SPEC claims is unchanged: a **malformed policy** (`@N` index ≥ `keys_info.len()`) today produces a miniscript `DescriptorParse("--descriptor parse: …")` from `parse_literal_xpub`; after delegation it produces the new, earlier `DescriptorParse("…@N beyond keys_info…")`. I grepped the xpub-search test files — **no existing cell asserts on that malformed-policy error string**, so nothing breaks. But the §1.1 "behavior unchanged" claim is imprecise.

**Fix:** (a) Correct the citation — the Phase-1 `expand_bip388_policy` unit cell is what *first* pins the substitution output (this is a coverage *gain*, state it as such). (b) Reword "behavior unchanged" → "behavior unchanged on the happy path; the malformed-`@N` error path strictly improves (earlier, explicit message) — verified no existing cell pins the old string." (c) Add a Phase-1 cell asserting the xpub-search malformed path still exits non-zero with the new message, so the improved error is itself pinned.

---

## MINOR (6)

- **M-1 (§1.3 line 68, over-broad bundle-mode claim):** "The expanded body … → `MultisigWatchOnly`" is not universal. A single-key BIP-388 policy (`wpkh(@0/**)` with one `keys_info` entry — emitter-producible, since `wpkh(xpub/<0;1>/*)` satisfies `is_multipath()`) yields n=1 → `BundleMode::SingleSigWatchOnly` (`bundle.rs:1657`), not `MultisigWatchOnly` (`:1660`). Still correct, secret-free behavior; just tighten the wording to "→ watch-only (`MultisigWatchOnly` for n≥2, `SingleSigWatchOnly` for n=1)."

- **M-2 (bundle requires origin-annotated `keys_info`):** A BIP-388 policy with bare (origin-less) `keys_info` entries (`xpub/<0;1>/*`, legal per BIP-388) expands fine and works in **export-wallet** (miniscript parses it), but **breaks the bundle path**: `descriptor_concrete_to_resolved_slots` → `concrete_keys_to_placeholders` requires `[fp/path]xpub` and errors "no [fp/path]xpub keys found in descriptor" (`wallet_import/pipeline.rs:221-225`). Emitter-produced fixtures always carry origins (the emitter's source keys are origin-annotated), so the §6 happy-path fixtures are clean — but add a one-line scope note that **bundle** requires origin-annotated policies, and consider a negative test.

- **M-3 (§1.1 line 35–40, citation drift-narrow):** "body = the current `descriptor_intake.rs:205-218` logic" — the substitution loop is `:210-218`; `:205-209` is the `serde_json::from_str` parse. The function body sans the trailing `parse_literal_xpub` call (`:219`) spans `:205-218`, so the range is defensible but imprecise. Prefer "the parse+substitution at `:205-218`."

- **M-4 (round-trip non-canonical caveat under-tested):** §3's byte-stability claim holds for emitter-produced (canonical) fixtures by construction (canonical policy is idempotent: P→C→miniscript-canonical==C→emit→P'==P). The one real gap is a **hand-authored policy with non-canonical `keys_info`/`sortedmulti` order**, which miniscript reorders on first emit. §3's parenthetical scopes this ("canonicalized by miniscript on first emit, stable thereafter"), so it's not a blocker — but add one Phase-2 cell with a deliberately non-canonical `sortedmulti` policy asserting the reorder, so the implementation doesn't accidentally over-promise byte-stability. (`multi()`, order-significant, round-trips faithfully — no issue.)

- **M-5 (§1.1 / §6 "longest-N-first" cross-cite):** The phrasing is correct for the **expander** (`descriptor_intake.rs:213` sorts `Reverse(n.to_string().len())` — digit count). The cited inverse op `pipeline.rs:190-204` sorts `Reverse(s.len())` — full-string length. Both are correct for their direction; the SPEC is not wrong, but a reader may conflate the two sort keys. One clarifying clause ("expander orders by N-digit-count; emitter by full-key-string length — each correct for its direction") prevents a false-positive re-raise. Note: since `/**` is part of every replaced token, `@1` can never be a substring of `@10/**`, so the ordering is over-defensive but faithful to existing code — do not "fix" it.

- **M-6 (recon emitter cite, already noted in recon):** The recon flagged the emitter range `:178-198`→`:178-209`; the SPEC uses `:166-211` and `:190-204`, both accurate against current source. No action; logged for completeness.

---

## Verified-clean (do not re-raise)

- **Ordering invariant (§2):** Confirmed. A raw policy JSON trips both probes — `at_n_probe` (`@\d`) matches `@0/**` in `description_template` and `key_regex` matches `keys_info` → `classify_descriptor_form` returns `(true,true)` mixed-error (`pipeline.rs:136-138`); `is_at_n_form` returns true → export-wallet `:412` refusal. The leading-`{` pre-check before `:412` (export-wallet) and before `:304` (bundle) correctly precedes both. No bypass path: both call sites read `body`/`desc` and the pre-check shadows it before any classification. The `desc_owned`/shadowing pattern (§1.2) is valid Rust against `desc: &String` at `export_wallet.rs:409`.
- **Bundle checksum:** Non-issue (verified `verify_checksum` source). Do not flag.
- **Module home (§1.1):** `wallet_import/pipeline.rs` is clean — already `pub(crate)` home of `is_at_n_form`/`classify_descriptor_form`/`DescriptorForm`, already imported by both `export_wallet` and `bundle`, already depends on `serde`-adjacent machinery and `ToolkitError`. Moving `BipPolicyJson` there introduces no cycle (`descriptor_intake` under `cmd/xpub_search` would import upward from `wallet_import`, which it can). Bare-`String` return is sufficient for all three callers.
- **`/**` re-lex (§4):** Correct — expansion yields `/<0;1>/*`, never a literal `/**`; fed to `miniscript::from_str`/`key_regex`, never the toolkit single-star lexer.
- **Disposition (§5/§7):** PATCH v0.48.1 is correct. No flag-name/value/subcommand change → no `schema_mirror` lockstep is right. Release-gate markers all verified present: `Cargo.toml:3` (`0.48.0`), `README.md:13` + `crates/mnemonic-toolkit/README.md:9` (`<!-- toolkit-version: 0.48.0 -->`), `scripts/install.sh:32` (`mnemonic-toolkit-v0.48.0`), and the gating tests/workflows (`tests/readme_version_current.rs`, `.github/workflows/install-pin-check.yml`) exist. The "re-run full suite AFTER the bump" instruction is correct given those gates key on the version string.

---

## What to fold before implementation (gate to GREEN)
1. **I-1:** Rewrite the §1.1 `BipPolicyJson` block to either `#[allow(dead_code)] name: String` or `#[serde(rename="name")] _name: String`; add the `deny_unknown_fields` trap note.
2. **I-2:** Correct the `:438-458` regression-pinning citation; reword "behavior unchanged" to scope the improved malformed-`@N` error path; add the xpub-search malformed-path pin cell.
3. **M-1, M-2:** one-line corrections to §1.3 (bundle-mode for n=1; origin-annotated-`keys_info` requirement). M-3/M-4/M-5 are wording/test-cell nits foldable in the same pass.

Folding I-1, I-2, M-1, M-2 takes this to **0 Critical / 0 Important → GREEN**.
