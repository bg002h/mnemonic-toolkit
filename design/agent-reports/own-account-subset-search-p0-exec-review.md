> Reviewer: opus architect (P0 per-phase EXECUTION review) · 2026-06-22 · branch `feature/own-account-subset-search` HEAD `fa4f8e0a` (`feat(derive-slot): P0 ScrubbedXpriv + derive_account_xpub_only`) · single file `crates/mnemonic-toolkit/src/derive_slot.rs` (+408/−2). Contract: SPEC `design/SPEC_own_account_subset_search_2026-06-20.md` §4.5 + plan P0 + relaunch re-R0 `…-relaunch-r0-review.md`. Adversarial, source-verified: ran the 3 mutation tests (Copy/Clone/derive-perturb), the full suite, and clippy myself; verified the upstream API surface against the lock-pinned crate sources (bitcoin 0.32.8, secp256k1 0.29.1, bitcoin-internals 0.3.0).

**Verdict: GREEN — 0 Critical, 0 Important.**

P0 is a clean, additive, funds-safe secret-hygiene scrub. The four make-or-break gates all hold under mutation: (1) move-only is genuinely enforced — `#[derive(Copy)]` is an E0184 hard error AND the hand-rolled `AmbiguousIfImpl<_>` static assertion is LOAD-BEARING (a manually-injected `impl Clone` makes the test target fail to compile with E0283, the exact gap the implementer reported their first two forms missed); (2) the `Xpriv` provably never escapes — the only `pub` API is `new(Xpriv)` + `&self` accessors returning public `(Xpub, Fingerprint)`, no `into_inner`/`Deref`/`-> Xpriv`/`&mut Xpriv`/pub-field, and the `ScrubbedXpriv`-returning seam is a private fn; (3) the scrub is correct and not UB — `non_secure_erase()` is a real volatile erase, and `chain_code.as_mut_ptr()` (verified to borrow the owned backing `[u8;32]`, not a temp) feeds a genuine `core::ptr::write_volatile` loop that the optimizer cannot elide; (4) the byte-identical golden is NON-VACUOUS — perturbing the derive turns it RED on the first vector. The diff is exactly one file, no `mlock.rs`/`DerivedAccount`/Cargo touched, no new dep. Full suite GREEN (1084 binary-target tests incl. the 5 new scrub tests + ~1300 across all targets, 0 failures); clippy `-D warnings` clean on a forced recompile. Caveat is honestly tracked, not over-claimed. The hard gate is satisfied: **advance to P1.**

---

## 1. Move-only enforcement + the load-bearing assertion (THE CRUX) — VERIFIED

**Upstream baseline (why this matters).** `bitcoin::bip32::Xpriv` is `#[derive(Copy, Clone, PartialEq, Eq)]` (bitcoin-0.32.8 `src/bip32.rs:72`), with `pub private_key: secp256k1::SecretKey` (`:84`) and `pub chain_code: ChainCode` (`:86`) both public. So the INNER type is freely `Copy`+`Clone` with public secret fields — the wrapper's move-only-ness is the ONLY thing standing between the over-supply loop and a `K_own`-fold un-scrubbed residue. The assertions are not ceremony; they are the guarantee.

**No `Copy`/`Clone` derive present.** `derive_slot.rs:195` is `pub struct ScrubbedXpriv(Xpriv);` with NO `#[derive(Copy)]`/`#[derive(Clone)]`. `Clone` is deliberately not derived (`:163-167`). A `// DO NOT add Clone/Copy/into_inner/Deref<Xpriv>` guard-comment sits at `:187`.

**MUTATION 1 — inject `#[derive(Copy, Clone)]` → E0184 (CONFIRMED).** In a detached worktree at `fa4f8e0a` I added `#[derive(Copy, Clone)]` to the struct. `cargo build --bin mnemonic` produced:
`error[E0184]: the trait \`Copy\` cannot be implemented for this type; the type has a destructor`.
The `impl Drop` (`:217`) makes `Copy` structurally impossible — compiler-enforced, not test-enforced. ✓

**MUTATION 2 — inject manual `impl Clone for ScrubbedXpriv` → assertion FAILS TO COMPILE (CONFIRMED — this is the reported risk).** A manual `impl Clone` does NOT trip E0184, so the `Clone` axis rides entirely on the hand-rolled static assertion (`const _: fn()` block, `:380-400`). I injected:
```rust
impl Clone for ScrubbedXpriv { fn clone(&self) -> Self { ScrubbedXpriv(self.0) } }
```
`cargo test --bin mnemonic --no-run` produced:
`error[E0283]: type annotations needed … multiple \`impl\`s satisfying \`ScrubbedXpriv: AmbiguousIfImpl<_>\` found` (citing both the blanket `impl<T> AmbiguousIfImpl<()> for T` and `impl<T: Clone> AmbiguousIfImpl<Invalid> for T`). The test/binary target goes RED.
**The shipped assertion form is genuinely load-bearing.** It catches the exact `Clone` re-introduction the implementer flagged that their first two forms silently missed. The form is the standard `static_assertions::assert_not_impl_any!` pattern (two overlapping blanket impls keyed on distinct markers + an inferred-`<_>` qualified call), correctly using a trait-method `some_item` (no inherent-method shadowing pitfall), and is sited in `main.rs`'s binary test target (`derive_slot` is mounted unconditionally at `main.rs:11`; the `lib.rs:154` mount is `#[cfg(fuzzing)]`-only, so the doctest-is-dead reasoning is correct — but the `const _` block runs in the normal binary build, which I confirmed by it firing under mutation). ✓

**No escape hatch.** The full accessor surface on `ScrubbedXpriv`: `pub fn new(Xpriv) -> Self` (`:201`), `pub fn xpub(&self, &Secp256k1<All>) -> Xpub` (`:207`), `pub fn fingerprint(&self, &Secp256k1<All>) -> Fingerprint` (`:212`). Grep confirms NO `into_inner`, `Deref`/`DerefMut`, `-> Xpriv`, `&mut Xpriv`, `as_xpriv`, pub inner field, or `impl Clone/Copy`. Inner `self.0` is touched only by `&`-borrows producing public material (`Xpub::from_priv(secp, &self.0)`, `self.0.fingerprint`) or by the Drop scrub. ✓

## 2. `Xpriv` never escapes — VERIFIED

`derive_account_xpub_only` (`:251`) and `derive_accounts_xpub_only` (`:273`) are the two `pub` entrypoints; their return types are `Result<(Xpub, Fingerprint), …>` and `Result<Vec<(Xpub, Fingerprint)>, …>` — public-only. Internally:
- `derive_master_for_xpub_only` (`:301`, **private** `fn`, no `pub`) wraps the master `Xpriv::new_master(...)` in `ScrubbedXpriv::new(master)` and returns `(ScrubbedXpriv, Fingerprint)` — the `ScrubbedXpriv` is module-private and is held by the `pub` caller only as a local that drops at fn exit (`// master drops here → scrub`, `:262`).
- `derive_one_xpub_only` (`:325`, private) wraps the `master.0.derive_priv(secp, path)` result `ScrubbedXpriv::new(...)` IMMEDIATELY (`:337-342`), reads `account.xpub(secp)`, returns only `(Xpub, Fingerprint)`; the `account` `ScrubbedXpriv` drops before return (`:352`).

No `Xpriv`/`SecretKey`/`ScrubbedXpriv` is returned from, or stored beyond, any `pub` fn. The network-mismatch guard is preserved verbatim (`:344-350`, identical to the bare helper's `:89-95`) and is exercised positively — `derive_account_xpub_only_testnet_yields_tpub` (`:518`) asserts a Testnet derivation yields a `tpub`. ✓

## 3. Scrub set + volatility — CORRECT, not UB

`Drop` (`:217-239`):
1. `self.0.private_key.non_secure_erase()` (`:221`). Verified upstream: secp256k1-0.29.1 `src/key.rs:972` `pub fn non_secure_erase(&mut self) { self.0.non_secure_erase(); }` — a real best-effort volatile erase (via the `non_secure_erase` macro at `src/macros.rs:66`), on `SecretKey`. ✓
2. Volatile `chain_code` zero-write (`:228-237`): `let cc_ptr = self.0.chain_code.as_mut_ptr();` then a `for i in 0..32` loop of `unsafe { core::ptr::write_volatile(cc_ptr.add(i), 0u8); }`. Verified `ChainCode([u8;32])` (bitcoin-0.32.8 `bip32.rs:50`) gets `as_mut_ptr` from `impl_array_newtype!` (bitcoin-internals-0.3.0 `src/macros.rs:20-23`): `let &mut $thing(ref mut dat) = self; dat.as_mut_ptr()` — a `*mut u8` into the OWNED backing array, no temp/copy, no layout assumption. Each of the 32 in-bounds bytes is written exactly once; `u8` has no invalid bit-patterns and no Drop ⇒ the volatile writes are sound and not elidable (the rationale a plain `= ChainCode::from([0u8;32])` assignment IS elidable on a dropping value is correct — `write_volatile` is the right tool). The `// SAFETY:` comment (`:229-232`) accurately states the pointer is live/aligned/owned. Non-secret metadata (`network`/`depth`/`parent_fingerprint`/`child_number`) is intentionally left intact. ✓

## 4. Byte-identical golden — NON-VACUOUS

`derive_account_xpub_only_byte_identical_to_bare_helper` (`:447`) asserts `derive_account_xpub_only`'s `(xpub, fingerprint)` EQUALS the existing bare `derive_bip32_from_entropy_at_path`'s `account_xpub` + `master_fingerprint` over a real matrix: entropy {Trezor-24 zeros, 16 zeros} × passphrase {"", "TREZOR"} × network {Mainnet, Testnet} × 4 paths incl. the hardened multisig `m/48'/0'/0'/2'` and single-sig `m/84'/0'/0'` (= 32 cases), asserting BOTH the xpub and the fingerprint with a per-case diagnostic.

**MUTATION 3 — perturb the derive (appended `/0` to the path in `derive_one_xpub_only`) → test RED (CONFIRMED).** `cargo test --bin mnemonic byte_identical`:
`derive_account_xpub_only_byte_identical_to_bare_helper … FAILED` — `assertion left == right failed: xpub differs … path=84'/0'/0'`. The guard fires on the first vector/path. It is the genuine regression guard that the scrub-confining refactor did not perturb derived keys; not a tautology. ✓

(Note: under Mutation 3 the fan-out parity test `derive_accounts_xpub_only_matches_single_account_form` (`:489`) stays GREEN, because both fan-out and single forms route through the same perturbed `derive_one_xpub_only` — it guards the master-derived-once equivalence axis, NOT the absolute key value. That is the correct division of labor: the byte-identical golden against the INDEPENDENT bare helper is what pins absolute correctness, and it fired.)

## 5. Additive — no broader-lift creep — CONFIRMED

`git diff --name-only fa4f8e0a^..fa4f8e0a` = exactly `crates/mnemonic-toolkit/src/derive_slot.rs` (1 file). The "−2" deletions are the two `use` lines being widened (`+Fingerprint` to the bip32 import, `+All` to the secp256k1 import) — purely additive, no logic removed. `DerivedAccount` (`derive.rs:23`, `account_xpriv: Xpriv` at `:27`), `into_parts` (`derive.rs:47`), `derive_bip32_from_entropy_at_path` (`:65`) + its ~7 callers, and `mlock.rs` are UNTOUCHED (empty diff). No Cargo.toml/Cargo.lock/fuzz change → no new dep (`static_assertions` correctly NOT added; the assertion is hand-rolled). The 7-site lift remains the FOLLOWUP `derive-slot-account-xpriv-scrub-confinement`. ✓

## 6. Caveat honesty + SAFETY-lint gate — CONFIRMED

The tracked caveat is documented without over-claim: struct doc `:182-185` and `derive_one_xpub_only` `:332-336` both state the by-value `derive_priv()->Xpriv` return temp + secp internals stay under `rust-bitcoin-xpriv-zeroize-upstream` and `non_secure_erase` is best-effort. FOLLOWUPS.md:4524 says the newtype "MINIMIZES (one move-in/derive) but cannot eliminate the residue." No "fully scrubbed" over-claim anywhere. The `derive_priv(...)` result is moved into `ScrubbedXpriv::new(...)` as immediately as the language allows (`:337-342`) — the transient stack temp between the by-value return and the move is exactly the irreducible residue the caveat names.

`lint_safety_third_party_blocked` gate PASSES (1 passed). Each new production third-party construction site has `SAFETY: third-party-blocked` within ±15 lines: `derive_master_for_xpub_only` `Mnemonic::from_entropy_in`/`Xpriv::new_master` (`:311`/`:314`) covered by `:308`; `derive_one_xpub_only` `.derive_priv(` (`:340`) covered by `:332`. The `unsafe { write_volatile }` block has its own correct `// SAFETY:` comment (`:229-232`). ✓

## 7. Build / test / clippy evidence (run by the reviewer)

- `cargo test -p mnemonic-toolkit` (FULL) — **all targets GREEN, 0 failures.** Binary target (where `scrub_tests` live): **1084 passed, 0 failed, 1 ignored**; lib unit target 164 passed; all ~80 integration/lint/prop targets green (totals incl. e.g. 57, 55, 33, 31, 26 across files). The 5 new scrub tests run + pass in the binary target:
  `scrubbed_xpriv_self_accessors_and_drop`, `derive_account_xpub_only_testnet_yields_tpub`, `derive_account_xpub_only_returns_public_types`, `derive_accounts_xpub_only_matches_single_account_form`, `derive_account_xpub_only_byte_identical_to_bare_helper` — all `ok` (1080 filtered out when name-filtered).
- `cargo clippy -p mnemonic-toolkit --tests --bins --lib -- -D warnings` — **clean** on a FORCED recompile (touched `derive_slot.rs`; `Finished` in 4.56s, zero warnings/errors).
- `git diff --name-only fa4f8e0a^..fa4f8e0a` = `crates/mnemonic-toolkit/src/derive_slot.rs` only (no `mlock.rs`, no `derive.rs`, no Cargo.toml/lock, no fuzz).

---

## Critical / Important / Minor

- **Critical:** none.
- **Important:** none.
- **Minor (non-blocking, informational — no action required for the gate):**
  - **m-1 — fan-out parity test is shared-path-blind by construction.** `derive_accounts_xpub_only_matches_single_account_form` (`:489`) cannot catch a drift inside the shared `derive_one_xpub_only` (both arms route through it). This is fine because the byte-identical golden (`:447`) pins absolute correctness against the independent bare helper; just noting that the fan-out test alone is not an anti-vacuity guard for the derive math. No change needed.
  - **m-2 (carried from relaunch re-R0 MINOR m-1) — narrow the `derive-slot-account-xpriv-scrub-confinement` FOLLOWUP slug at ship.** P0 authored the minimal helper, so the slug's Origin line ("does NOT author this", `FOLLOWUPS.md:4515`) is now stale and must be narrowed to the 7-site lift in the P0 (or P6) shipping commit. Not gate-blocking. (Already captured by SPEC §8 P6.)

## Closing

**GREEN — 0 Critical, 0 Important.** Every make-or-break gate was verified by mutation, not inspection: `Copy` is E0184-blocked, the `Clone` static assertion is load-bearing (injected `impl Clone` ⇒ E0283 compile failure — the reported silent-hole risk is closed), no `Xpriv` escapes any `pub` fn, the `chain_code` scrub is a genuine non-elidable volatile write over the owned backing array (upstream API surface confirmed at the lock-pinned crate versions), and the byte-identical golden goes RED on a perturbed derive. Additive single-file diff, no dep, no churn to `DerivedAccount`/the 7 sites, full suite + forced clippy GREEN, caveat honest, SAFETY-lint gate passing. The hard R0-execution gate is satisfied — **implementation may advance to P1 (engine: `unrank_kperm` + own-anchored generator + cardinality helpers, brute-force-reference tested).**

No rubber-stamp: this verdict rests on running the three adversarial mutations (Copy→E0184, Clone→E0283, derive-perturb→RED golden) in a throwaway worktree, reading the upstream `Xpriv`/`ChainCode`/`SecretKey::non_secure_erase`/`impl_array_newtype!` definitions at the lock-pinned versions (bitcoin 0.32.8, secp256k1 0.29.1, bitcoin-internals 0.3.0), grepping the full escape-hatch surface, confirming the single-file additive diff, and running the whole suite + a forced clippy myself.
