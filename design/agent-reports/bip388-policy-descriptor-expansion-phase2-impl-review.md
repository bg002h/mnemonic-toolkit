# Implementation Review — Cycle D (Phase 1+2) — bip388 wallet-policy → descriptor expansion

> Persisted verbatim from the opus code-reviewer dispatch on the implementation diff (4 source files + new test file) against the SPEC.

---

## Verdict: GREEN — 0 Critical / 0 Important

No Critical or Important findings. The implementation matches the SPEC contract on every load-bearing invariant.

**What I verified holds:**

1. **Ordering invariant (SPEC §2) — correct at both sites.**
   - `export_wallet.rs:416` — `is_bip388_policy_shape` precedes the `is_at_n_form` refusal at `:424`. Sole `--descriptor` entry (line 409); export-wallet has no `--descriptor-file`/stdin route into this fork (the `read_to_string` at `:645-650` is the unrelated `--from-import-json` envelope path, which carries an already-concrete descriptor).
   - `bundle.rs:309` — `is_bip388_policy_shape` precedes `classify_descriptor_form` at `:315`. Both `--descriptor` (inline) and `--descriptor-file` are read into `body` at `:295-302` *before* the pre-check, so both routes are guarded.

2. **`desc_owned` deferred-init borrow (export_wallet.rs:415-421) is sound.** `desc_owned` outlives the shadowed `desc` binding; only initialized/borrowed in the policy branch; the expanded string flows into `MsDescriptor::from_str` at `:433`.

3. **bundle expanded `body` flows correctly** into `bundle_run_concrete_descriptor` (moved by value at `:316`). Watch-only path correct: a secret-free policy → `any_secret == false` → `SingleSigWatchOnly` (n=1) / `MultisigWatchOnly` (n≥2) at `bundle.rs:1666-1672`. Tests pin both shapes.

4. **Delegation preserved (descriptor_intake.rs:188-198).** Struct moved to pipeline.rs, no leftover `use serde::Deserialize`, `intake.shape = Bip388Json` set, happy-path Cell 6 (`account_of_descriptor_bip388_json_match`) unchanged. Exit codes consistent: malformed-`@N`-beyond-keys_info is `DescriptorParse`→exit 2 both before and after the dedup; malformed-JSON is `BadInput`→exit 1. No existing test pins the old message, so the cosmetic change does not break the suite.

5. **No dead code / unused imports** after the struct move (clippy clean).

6. **Tests pin the load-bearing ordering.** `export_wallet_raw_policy_not_refused_by_at_n_guard` asserts `.success()` — reorder the pre-check after `is_at_n_form` and a raw policy exits failure, breaking it. Bundle happy-path tests use `run_ok` (success) → reorder → `classify (true,true)` mixed-form error breaks them. Both sites covered, plus `@N`-beyond, bare-key refusal, round-trip, and the xpub-search delegation regression cell.

### Minor (not gating)

- **SPEC §1.3/§6 narrative is stale on *which layer* refuses the bundle bare-key policy.** The SPEC says `"no [fp/path]xpub keys found in descriptor"` from `concrete_keys_to_placeholders`; the actual path refuses earlier — `classify_descriptor_form(&body)` at `bundle.rs:315` returns `(false,false)` → `"must carry a key origin"`. The test (`bundle_descriptor_bip388_bare_key_policy_refused`) was written to the real message. Outcome (refuse) correct and the earlier message is arguably better; only the SPEC prose drifts. → folded: SPEC §1.3/§6 corrected.

### Verification caveats (both closed by reviewer-of-record)
- Old serde wrapper variant was `ToolkitError::BadInput` (confirmed from the pre-edit source) → preserved in `expand_bip388_policy`. ✓
- Full suite green: `cargo test -p mnemonic-toolkit` → 155 test binaries ok, 0 failures (pre-bump). ✓
