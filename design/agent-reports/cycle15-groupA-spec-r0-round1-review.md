# R0 REVIEW (verbatim) — cycle-15 Group A brainstorm-spec — round 1

**Dispatched:** 2026-06-21. **Base:** `9b7c78a7` (v0.68.0). **Spec under review:** `design/BRAINSTORM_cycle15_groupA_bsms_lint.md`.
**Verdict:** NOT GREEN — 1 Important, 0 Critical. (Re-dispatch after folding I1 + M1–M3; #3 SemVer ruled Option A / 0.69.0, decided at architecture level.)

---

Confirmed: the ONLY consumers are the prod caller (`import_wallet.rs`), the in-module tests, and the one integration test (`cli_import_wallet_bsms_encrypted.rs`). No fuzz target, no example, no GUI/sibling reference. `pub mod bsms_crypto` is exported (`lib.rs:67`) but no external crate links the toolkit lib. The spec's "internal-only `pub`" claim holds.

I have all ground truth. Let me write the verdict.

---

# R0 REVIEW — cycle-15 Group A (bsms HMAC_KEY hygiene + lint allowlist precision)

## VERDICT: NOT GREEN — 1 Important, 0 Critical

The spec is mostly sound and well-reasoned, but it leaves the load-bearing #4 lint-composition as an unresolved "decide whether…" open question. I empirically proved the naive resolution breaks the suite. That must be pinned in the spec before code. Everything else verified clean against real source (base `9b7c78a7`).

---

## Critical
None.

---

## Important

**I1 — #4 leaves the partition-scan composition unspecified; the naive resolution is RED (proven).**
Spec lines 41/43/67(c) say bundle_unified.rs "moves between two allowlist tiers that both count toward the partition" and that the scan must "stay GREEN, floor still 37" — but line 43 explicitly defers the mechanism ("**decide whether** `TEST_ONLY_SECRET_FILES` is a separate const both the partition-scan AND the confinement-guard read"). An R0-GREEN brainstorm must pin this, because the obvious reading (move the entry out of `NON_ROW_SECRET_FILES` into a new const) breaks two tests.

I verified empirically: the partition scan at `tests/lint_zeroize_discipline.rs:557` reads only `declared ∪ allowlisted` where `allowlisted = NON_ROW_SECRET_FILES` (`:547-548`). Simulating the move (delete `"src/bundle_unified.rs"` from `NON_ROW_SECRET_FILES`, add a new const the scan doesn't union) →

```
test every_secret_bearing_src_file_is_declared_or_allowlisted ... FAILED
panicked at .../lint_zeroize_discipline.rs:561  (undeclared not empty)
```

It also drops the live partition count from 38 → 37 (floor is 37, so still ≥, but zero slack — and if the new const isn't unioned the floor logic counts the file as `secret_files` regardless since it's still grep-matched, so the floor itself survives; the hard failure is `undeclared`).

**Fix direction (pin in spec):** specify that BOTH of these MUST union the new tier:
- `every_secret_bearing_src_file_is_declared_or_allowlisted` — change the `allowlisted` set at `:547-548` to `NON_ROW_SECRET_FILES.iter().chain(TEST_ONLY_SECRET_FILES.iter())`.
- `non_row_secret_allowlist_is_non_empty_and_each_entry_still_bears_a_secret` (`:585`, the "still bears a secret" tripwire) — iterate both consts, or add a parallel assertion over `TEST_ONLY_SECRET_FILES` (each entry exists + is secret-bearing), so the new tier gets the same staleness friction.

Once the spec states "the scan and the tripwire both read `NON_ROW_SECRET_FILES ∪ TEST_ONLY_SECRET_FILES`; floor stays 37 (38 live, 1 slack)," this clears. This is the only blocker.

---

## Minor

**M1 — Option-A caller-ripple enumeration is both INCOMPLETE and OVER-PESSIMISTIC; correct it so the plan's RED/GREEN claims are accurate.**
- *Incomplete:* the spec says "Sole production caller: import_wallet.rs:2428" and the test plan (line 73) says "8 test sites + 1 prod." Grep finds a site the spec never names: `tests/cli_import_wallet_bsms_encrypted.rs:316-317` (helper `reencrypt_with_token`, also `derive_hmac_key` + `compute_mac`). True census = 1 prod + 8 in-module test + **1 integration-test helper**.
- *Over-pessimistic:* the spec repeatedly claims in-crate test asserts "need `*`/`&*` deref" (lines 25, 66, 73). I applied Option A (`derive_hmac_key → Zeroizing<[u8;32]>`) and built/tested with **zero caller edits**: `cargo build -p mnemonic-toolkit --tests` clean, all 20 bsms tests pass. Reason: `hex::encode(hmac_key)` takes `AsRef<[u8]>` (satisfied via Zeroizing's Deref/AsRef), `compute_mac(&hmac_key, …)` deref-coerces, `mac[..16]`/`&mac[..]` index through Deref. No `*` is required anywhere. Fix the plan so it doesn't propose unnecessary test edits and so the "8 test sites need deref" claim isn't carried into the GREEN criteria (it would make the impl add noise diffs).

**M2 — nit#2's stale number is "54" but the message also hardcodes other decaying numbers; verify the live count and prefer a non-numeric reword.** Confirmed: `ZEROIZE_ROWS.len()` = **59** (60 `ZeroizeRow {` matches minus the `struct ZeroizeRow {` definition at `:35`). The message at `:432` says "live 54" (stale, spec correct). But the same message also bakes in "36 + 16 = 52" and "ceiling raised from 60 to 66" — all decay-prone. Spec's "(or reword to not hardcode a number that decays)" is the right instinct; the plan should pick the reword, not just swap 54→59, else the next cycle re-files the same nit.

**M3 — confinement guard false-trips on a SECRET_PATTERN inside a comment/doc above `#[cfg(test)]`; note as accepted friction.** Verified today's tree is clean (no SECRET_PATTERN above `:118`; single `#[cfg(test)]` at `:118`), so the guard is GREEN day one. But a future doc-comment like `/// returns Zeroizing::new(...)` above the marker would false-trip (substring match). Spec question 3(a) flags it; the answer is: acceptable/intended friction (the guard is line-substring-based by design), but the plan should add one sentence saying so and the pure helper should match on the SECRET_PATTERN substrings exactly as the partition scan does (no comment-stripping), to keep the two mechanisms consistent.

**M4 — `compute_mac` staying bare is CORRECT.** Confirmed: the MAC is a published BIP-129 auth tag whose first 16 bytes become the on-wire IV (`bsms_crypto.rs:17`, `import_wallet.rs:2429`), and `mac_expected` is compared against `mac_recv` from untrusted wire (`:2436`). It is not secret-class. No wrap. The spec's one-line "MAC is a public auth tag" doc note is the right disposition. (Not even Minor as a defect — recorded as confirmation.)

---

## #3 SemVer fork — RECOMMENDATION: Option A (MINOR 0.69.0). Does NOT need user escalation.

Reasoning:
1. **First-class secret-hygiene bar** (the user-elevated guiding principle) says the scrub obligation belongs in the return TYPE, not in caller discipline. Option B leaves the footgun in the signature; the next BSMS caller can forget to wrap. Option A makes leakage unrepresentable.
2. **The "break" reaches no real consumer.** Verified: `pub mod bsms_crypto` is exported (`lib.rs:67`) but the only references anywhere in the tree are the prod caller, the in-module tests, and one integration-test helper — no fuzz target, no example, no GUI (GUI shells out to the CLI), no sibling repo links the lib. So it is an internal-only `pub` helper.
3. **Option A compiles with ZERO caller edits** (proven above) — the API "break" is purely nominal at the type level; not one call site needs touching. That removes the main argument for B (smaller diff): A's diff is just the signature + the `Zeroizing::new` in the body + the type-fence test.
4. **Precedent:** secret-type migrations bumped MINOR (v0.10.1, v0.67.0, and Lane T's 0.68.0). Consistent.

This is an **architecture/hygiene decision, not a product-owner pricing decision** — there's no user-facing behavior or CLI-surface change, outputs are byte-identical (TV3 unchanged), and the bump is mechanical SemVer policy the project already has precedent for. Decide it here: **Option A, 0.69.0.** The user's "0.68.1" was sequencing shorthand (spec line 25 already reads it that way, correctly). No escalation needed.

---

## Completeness check (spec question 5)
- **Version sites:** complete and accurate — Cargo.toml, root README + crate README (`<!-- toolkit-version: 0.68.0 -->`), install.sh self-pin (`:32`), root Cargo.lock, fuzz/Cargo.lock, CHANGELOG all at 0.68.0. `readme_version_current.rs` exists (spec's full-gate reference is valid).
- **Option-B partition status:** `import_wallet.rs` is already a declared ZEROIZE_ROW (`:322/:327`), 8 secret patterns present — caller-wrap adds no undeclared file. Spec's claim confirmed (moot under Option A rec).
- **Doc rewrite in scope:** yes, correctly (spec line 30) — `bsms_crypto.rs:108-113` must be rewritten under Option A to "returns `Zeroizing`; scrubbed on drop." Plan should make this explicit since the current doc actively defends the bare return.
- **nit#3 FOLLOWUP filing:** filing the slug + flipping #3/#4 in the same shipping commit is fine and conventional (the project flips FOLLOWUP status in the shipping commit). No code, correct. Confirm the bip85 line cites (`:189,204,252`) at write-time per the grep-verify-citations convention — I did not re-verify those since nit#3 is file-only and out of scope for compile/test.

---

## What's already verified GREEN (no action)
- Type-fence test (`let _: fn(&[u8;32]) -> Zeroizing<[u8;32]> = derive_hmac_key;`) is genuinely RED on base / GREEN under A — fn-pointer return types are invariant, no coercion at assignment. Sound RED-then-GREEN.
- TV3 MAC/IV/round-trip bytes UNCHANGED under A (all 20 bsms tests pass, including `tv3_end_to_end_round_trip`). The funds-relevant invariant holds.
- Base lint suite GREEN (4/4); confinement guard would pass day one.

**Re-dispatch after folding I1 (pin the partition-union resolution) + the M1–M3 corrections.** I1 is the sole gate; M1 prevents the plan from carrying false caller-ripple/test-edit claims.
