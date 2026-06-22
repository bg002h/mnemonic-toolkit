# BRAINSTORM / SPEC — cycle-15 Group A: bsms HMAC_KEY hygiene + lint allowlist precision

**Repo:** mnemonic-toolkit. **Base:** `origin/master` = `9b7c78a7` (v0.68.0, Lane T just shipped).
**Branch:** `feature/cycle15-groupA-bsms-lint` (worktree `wt-cycle15t`).
**Recon:** `cycle-prep-recon-bsms-bundleallowlist-mdsynthetic.md` (citations re-verified vs `9b7c78a7`: #3 unchanged; #4 drifted `:450→:498` after Lane T edited the lint file).
**Coordination:** the own-account instance owns the MAIN checkout (`feature/own-account-subset-search`); all work here is in the `wt-cycle15t` worktree; version coordinated at ship (no collision — own-account builds a lower number off v0.60.0).

This is the user-approved "Group A" follow-on after Lane T (AskUserQuestion: "Separate follow-on after Lane T"). It burns down 2 of the 4 remaining secret-keymat-sweep slugs + carries 2 nits from the Lane T whole-diff review.

---

## Scope (4 items)

### Item #3 — `bsms-derive-hmac-key-not-zeroizing` (the secret HMAC_KEY)
**Slug:** `bsms-derive-hmac-key-not-zeroizing`. **Source:** `src/bsms_crypto.rs`.

`derive_hmac_key(encryption_key: &[u8;32]) -> [u8;32]` (`:114`) returns the BIP-129 `HMAC_KEY = SHA256(ENCRYPTION_KEY)` as a **bare `[u8;32]`**. The HMAC_KEY is secret-class (derived from the `Zeroizing` ENCRYPTION_KEY at `:98`). The existing doc (`:108-113`) defends the bare return: "short-lived stack value … attacker who reads the stack already has ENCRYPTION_KEY … A caller retaining HMAC_KEY beyond the MAC compute should wrap manually."

Caller census (R0-verified): **1 production** caller `src/cmd/import_wallet.rs:2428` (`let hmac_key = derive_hmac_key(&enc_key);` then `:2435 compute_mac(&hmac_key, &token.hex, &plaintext)`) + **8 in-module test sites** (= 2 `derive_hmac_key(` calls + 6 `compute_mac(&hmac_key,…)` consumption sites at `bsms_crypto.rs:229,236,281,282,313,421-438`; note `:429,:437` use bare `[0x42;32]`-style literals unaffected by the return-type change) + **1 integration-test helper** `tests/cli_import_wallet_bsms_encrypted.rs:316-317` (`reencrypt_with_token`). **All compile + pass under Option A with ZERO edits (R0-proven, 20/20 bsms tests).**

`compute_mac(...) -> [u8;32]` (`:136`) returns the **MAC** — a BIP-129 authentication tag (its first 16 bytes become the public on-wire IV; `mac_expected` is compared against `mac_recv` from untrusted wire at `import_wallet.rs:2436`). A MAC is a *published* authentication value, **not secret-class** (R0 M4 confirmed). NO wrap; add a one-line "MAC is a public auth tag, not secret-class" doc note so a future reader doesn't "fix" it.

**SemVer — DECIDED at R0 round 1: Option A (signature change → `Zeroizing<[u8;32]>`, bump MINOR `0.69.0`).** Architecture/hygiene decision, NOT a product-owner pricing decision (no user-facing behavior / CLI-surface change; outputs byte-identical; mechanical SemVer policy with precedent) → decided at the gate, no user escalation. Rationale:
1. **First-class-hygiene bar** (user-elevated principle): the secret-class HMAC_KEY's scrub obligation lives in the return TYPE, so no present-or-future caller can leak it by forgetting to wrap (Option B left the footgun in the signature).
2. **The break reaches no real consumer** (R0-verified): `pub mod bsms_crypto` is exported (`lib.rs:67`) but the only references anywhere in the tree are the prod caller, the in-module tests, and the one integration-test helper — no fuzz target, no example, no GUI (shells out to the CLI), no sibling repo links the lib. Internal-only `pub` helper.
3. **Option A compiles with ZERO caller edits** (R0 empirically proved: `cargo build -p mnemonic-toolkit --tests` clean + all 20 bsms tests pass, no edits): `hex::encode(hmac_key)` takes `AsRef<[u8]>` via Zeroizing's Deref/AsRef; `compute_mac(&hmac_key, …)` deref-coerces; `mac[..16]`/`&mac[..]` index through Deref. **No `*`/`&*` needed at ANY call/assert site.** (Supersedes the earlier draft's "8 test sites need deref" — that claim is WRONG; the impl MUST NOT add deref noise diffs.)
4. **Precedent:** secret-type migrations bumped MINOR (v0.10.1, v0.67.0, Lane T's 0.68.0). The user's "0.68.1" label was sequencing shorthand, not a SemVer pin.

**Resolution:** signature → `Zeroizing<[u8;32]>` + wrap the body's `out` as `Zeroizing::new(...)` returned by move; **rewrite the `:108-113` doc** (which currently *defends* the bare return) to "returns `Zeroizing`; scrubbed on drop"; `compute_mac` stays bare with the public-auth-tag note; flip the slug `open → resolved` in the shipping commit.

**Lint impact:** `bsms_crypto.rs` is whole-file allowlisted (`NON_ROW_SECRET_FILES`, CRYPTO-INTERNAL) → adding `Zeroizing` there adds/removes NO lint row and no floor change.

### Item #4 — `bundle-unified-whole-file-allowlist-precision`
**Slug:** `bundle-unified-whole-file-allowlist-precision`. **Source:** `tests/lint_zeroize_discipline.rs` + `src/bundle_unified.rs`.

The zeroize-completeness lint whole-file-allowlists `src/bundle_unified.rs` in `NON_ROW_SECRET_FILES` (`:498`). bundle_unified.rs matches `SECRET_PATTERNS` ONLY via its `#[cfg(test)] fn s()` fixture (`:124`, `SecretString::new(` at `:128`); the `#[cfg(test)]` boundary is `:118`, test mod `:119-319`; `s()` is used by ~15 of its own unit tests and **nowhere else**). The FOLLOWUP's concern: a per-FILE allowlist **silently masks a FUTURE *production* secret allocation** added to that file (above `:118`).

Moving `s()` out of `src/` is NOT clean — its consumers are bundle_unified.rs's own *unit* tests, which can't import from a `tests/` integration crate.

**Chosen fix — a test-confinement guard.** Introduce a `TEST_ONLY_SECRET_FILES` tier (move bundle_unified.rs there from `NON_ROW_SECRET_FILES`) + a new test asserting, for each such file, that **every line containing a `SECRET_PATTERN` appears AFTER the file's first `#[cfg(test)]` line** (patterns confined to the test region). Effect: if someone later adds a production `Zeroizing::new(...)`/`SecretString::new(...)` ABOVE the cfg(test) marker, the guard FAILS — the latent masking is closed, while today's clean state passes.

**PARTITION COMPOSITION — PINNED (R0 round-1 I1, the gate). The naive move is RED (`every_secret_bearing_src_file_is_declared_or_allowlisted` fails its `assert!` at `:562-569`, `undeclared` non-empty), because the scan's `allowlisted` set at `:547-548` reads ONLY `NON_ROW_SECRET_FILES`.** Both consumers MUST union the new tier:
1. **`every_secret_bearing_src_file_is_declared_or_allowlisted`** (`:539`) — change the `allowlisted` set (`:547-548`) to read `NON_ROW_SECRET_FILES.iter().chain(TEST_ONLY_SECRET_FILES.iter()).copied().collect()`.
2. **`non_row_secret_allowlist_is_non_empty_and_each_entry_still_bears_a_secret`** (`:584`, the staleness tripwire) — iterate BOTH consts (exists + still-bears-a-secret) so `TEST_ONLY_SECRET_FILES` gets the same friction; keep the existing `!NON_ROW_SECRET_FILES.is_empty()` assert and add a non-empty assert for the new tier too.

With both unioned: bundle_unified.rs stays in the partition (still grep-matched as secret-bearing), so the live count stays **38** and `SECRET_FILE_FLOOR=37` is unchanged (1 slack). No floor edit.

**Mechanism detail (for the plan to nail):** the confinement check is line-based and substring-exact (same `SECRET_PATTERNS` strings as the partition scan, NO comment-stripping — keep the two mechanisms consistent): find the index of the first line containing `#[cfg(test)]`; assert no line BEFORE that index contains any `SECRET_PATTERN`. **Factor into pure testable helpers** (e.g. `fn first_cfg_test_line(src: &str) -> Option<usize>` + `fn production_secret_lines(src: &str) -> Vec<usize>`) so the NEGATIVE case (a synthetic source string with a `Zeroizing::new(` above `#[cfg(test)]`) is provable in a unit test WITHOUT mutating the real file. **Accepted friction (R0 M3):** a future `SECRET_PATTERN` substring inside a *comment/doc* above `:118` would false-trip — intended (the guard is deliberately substring-based, mirroring the partition scan); document this in one sentence.

### Nit #2 (from Lane T whole-diff review) — stale lint count-guard message
`tests/lint_zeroize_discipline.rs:429-432`: the `ZEROIZE_ROWS.len()` count-guard message literally says *"live 54 + the new rows"*. Actual `ZEROIZE_ROWS.len()` = **59** (R0-confirmed). The guard RANGE `(18..=66)` is correct and passing; only the human-readable text is stale. **R0 M2: don't just swap 54→59** — the same message also bakes in `"36 + 16 = 52"` and `"ceiling raised from 60 to 66"`, all decay-prone. **Reword to avoid hardcoded decaying counts** — interpolate the live `{n}` (already in scope) for the actual count and describe the bound qualitatively ("the upper bound carries headroom for near-term rows; widen it deliberately when exceeded"), so the next cycle doesn't re-file this nit. Cosmetic (message-only; asserts nothing); same file as #4 → fold here.

### Nit #3 (from Lane T whole-diff review) — file a FOLLOWUP slug (no code)
The Lane T review found two pre-existing, out-of-scope bare-`String` residue intermediates in `src/bip85.rs`: (a) `:189,204` the full `encoded` base64/base85 password string before truncate+wrap (the `base64_standard`/`base85_btc` encode helpers return bare `String`); (b) `:252` dice `out: Vec<String>` per-roll scratch. The Lane T FOLLOWUP named the RETURN values (now `SecretString`-wrapped); these helper-internal intermediates are the same residue class as the (also out-of-scope) `compute_outputs` bare-`String` output Vec. File a new FOLLOWUP slug `bip85-encode-helper-internal-scratch-zeroizing` (companion to the constellation derived-output program). NO code this cycle.

---

## Out of scope / non-goals
- No clap-surface / flag / subcommand / dropdown change → **no `schema_mirror`, no manual mirror, no GUI paired-PR**. (Both items are an internal crypto helper + a lint test.)
- No sibling-codec companion (toolkit-internal; the existing cross-cite in the slug body suffices).
- compute_mac output stays bare (public MAC tag).
- The bip85 encode-helper residue (nit #3) is FILED, not fixed.
- No change to BIP-129 MAC/encryption wire behavior — pure memory-hygiene + lint-precision; outputs byte-identical.

## SemVer + release
- **MINOR `0.69.0`** (DECIDED at R0 — Option A pub-signature secret-migration; matches v0.10.1/v0.67.0/0.68.0 precedent).
- Version sites (toolkit release ritual — all currently at 0.68.0, R0-verified complete): `crates/mnemonic-toolkit/Cargo.toml`, root `README.md` + crate `README.md` (`<!-- toolkit-version: -->`), `scripts/install.sh` self-pin (`:32`), root `Cargo.lock` + `fuzz/Cargo.lock`, `CHANGELOG.md`. (No ms-codec pin change — Lane T already moved it to 0.6.)
- FOLLOWUPS: flip #3 + #4 `open → resolved` in the shipping commit; ADD the nit-#3 slug entry (`bip85-encode-helper-internal-scratch-zeroizing`), grep-verifying its `bip85.rs:189,204,252` cites at write-time.

## Test plan (TDD, per item)
- **#3:** (a) a type-level fn-pointer fence `let _: fn(&[u8;32]) -> Zeroizing<[u8;32]> = derive_hmac_key;` — genuinely RED on base / GREEN under A (fn-pointer return types are invariant; no coercion at assignment — R0-confirmed); (b) the existing BIP-129 TV3 vector tests (`tv3_derive_hmac_key_matches_bip129`, `tv3_compute_mac_matches_bip129`, `tv3_end_to_end_round_trip`) MUST stay GREEN **with NO edits** (R0 proved all 20 bsms tests pass under Option A unchanged — Zeroizing Deref/AsRef covers `hex::encode`, `compute_mac(&hmac_key,…)`, and `mac[..]` indexing; the impl MUST NOT add `*`/`&*` deref noise to any of the 1 prod + 8 in-module + 1 integration-helper sites). The unchanged TV3 MAC/IV/round-trip is the funds-relevant byte-invariant.
- **#4:** (a) the new confinement guard passes on today's clean tree; (b) a NEGATIVE unit test over a SYNTHETIC source string (a `Zeroizing::new(` line above a `#[cfg(test)]` line) proving the pure helpers (`first_cfg_test_line` + `production_secret_lines`) flag it — provable without mutating the real file; (c) `every_secret_bearing_src_file_is_declared_or_allowlisted` + `non_row_secret_allowlist_is_non_empty_…` stay GREEN with BOTH consts unioned (I1); live partition count stays 38, floor 37 unchanged.
- **#2:** the count-guard still passes; message reworded (asserts nothing).
- **Full gate:** `cargo test -p mnemonic-toolkit` GREEN + `cargo clippy --workspace --all-targets -- -D warnings` clean + `readme_version_current` against **0.69.0**.

## Risks / R0 questions — RESOLVED at R0 round 1
1. ~~#3 SemVer (A vs B)~~ — **DECIDED: Option A, MINOR 0.69.0** (architecture decision, no user escalation).
2. ~~#3 Option-A caller ripple~~ — **RESOLVED: ZERO caller edits** (R0 built+tested all 20 bsms tests GREEN under A, no deref needed). Census = 1 prod + 8 in-module + 1 integration-helper (`tests/cli_import_wallet_bsms_encrypted.rs:316-317`); impl must NOT add deref noise.
3. ~~#4 mechanism~~ — **PINNED (I1): separate `TEST_ONLY_SECRET_FILES` const; BOTH the partition scan (`:547-548`) AND the staleness tripwire (`:585`) union it; floor stays 37 (38 live, 1 slack); pure testable helpers for the negative case.**
4. ~~#3 Option-B partition status~~ — moot under A (B not chosen); import_wallet.rs is already a row (`:322/:327`) anyway.
5. **Lint file double-edit (still live)** — #4 + nit#2 both edit `lint_zeroize_discipline.rs`; keep them in one coherent diff.
