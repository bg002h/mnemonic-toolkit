# WHOLE-DIFF REVIEW (verbatim) — bip85 encode-scratch Zeroizing (toolkit v0.69.1)

**Dispatched:** 2026-06-22. **Range:** `1cad7ec9..HEAD` (2 impl commits `b010123b`+`5293e4a6`, off master `d6e8757d`=v0.69.0). **Mandatory independent post-impl review.**
**Verdict:** GREEN (0C/0I/0 Minor) — cleared to push/tag `mnemonic-toolkit-v0.69.1`.

---

Both fmt-flagged bip85.rs locations are at lines 168 and 439 — these are pre-existing lines (`format_hex` at :168 and a test assertion at :439), NOT the cycle's new lines (189, 204, 252, 528-558). The new wraps at 189/204/252 and the fence test at 528+ are NOT flagged by fmt — meaning this cycle's additions are themselves fmt-canonical. The flagged lines are pre-existing non-canonical formatting on master (consistent with the project's standing "never cargo fmt the toolkit" exemption).

This cycle introduced zero new fmt violations. No fmt gate exists in toolkit CI anyway, and the project explicitly forbids running `cargo fmt` on it. Scope is clean.

All verification complete. Here is my verdict.

---

## VERDICT: GREEN (0 Critical / 0 Important / 0 Minor)

Cleared to push/tag `mnemonic-toolkit-v0.69.1`.

### What I verified (adversarial, source-confirmed)

**1. The 3 wraps — correct + ripple-free.** Confirmed at `bip85.rs:189` `Zeroizing::new(base64_standard(&entropy[..]))`, `:204` `Zeroizing::new(base85_btc(&entropy[..]))`, `:252` `out: Zeroizing<Vec<String>> = Zeroizing::new(Vec::with_capacity(...))`. `zeroize::Zeroizing` was already imported (`:18`, pre-existing). `encoded[..length].to_string()` indexes through `Deref<Target=String>`; `out.push/.len/.join` go through `Deref`/`DerefMut`. No wrap is `.clone()`d or `*`-deref'd into a bare escaping local. The encode helpers `base64_standard`/`base85_btc` (`:286`,`:326`) keep their bare-`String` signatures (wrap is at call-site — correct). `buf` (`:253`, 1-4 raw SHAKE bytes overwritten each iteration) correctly left bare per the dispositioned out-of-scope. No signature, no `buf`, no helper touched.

**2. Output byte-identical.** Ran the 3 KATs explicitly: `pwd_base64_matches_spec`, `pwd_base85_matches_spec`, `dice_d6_10_rolls_matches_spec` — all `ok` (1076 filtered out → full bin population). The `SecretString` returns render identical bytes.

**3. Fence test sound — RED-toggle proven.** `internal_encode_dice_scratch_is_zeroizing` assembles needles at runtime (`format!("{}::new", "Zeroizing")`) so it doesn't self-match. I reverted the b64 wrap on the real source → the fence **panicked** with `encode scratch: base64 'encoded' must be Zeroizing::new(base64_standard(...))`, then restored (git diff --stat empty). Non-vacuous; asserts all 3 wraps.

**4. No lint drift.** bip85.rs rows in `lint_zeroize_discipline.rs` (`:92-110`) are unchanged (they anchor `derive_entropy`'s return, the entropy local, and the `format_*` `SecretString` returns — none touched by the new wraps). No new row, no count-guard change (`18..=66`), no `SECRET_FILE_FLOOR` change (37). All 6 lint tests pass.

**5. Version/release.** All 6 sites at 0.69.1 (Cargo.toml:3, both READMEs `toolkit-version: 0.69.1`, install.sh:32, both Cargo.lock toolkit pkgs). No stray 0.69.0 outside design docs (where they correctly reference the prior cycle/base). CHANGELOG `[0.69.1]` entry present + accurate. FOLLOWUP `bip85-encode-helper-internal-scratch-zeroizing` flipped `open → resolved` (v0.69.1) with the dispositioned-out-of-scope note for `buf` and helper-internal `out`. PATCH correct (no signature/public-type reach).

**6. Scope fences.** 9 files, all planned. No schema/manual/clap/GUI/`.json` surface. This cycle's new lines (189/204/252/528+) are fmt-canonical; the two `cargo fmt --check` hits in bip85.rs are at pre-existing lines (168, 439) on master, not this cycle's additions (and the project forbids `cargo fmt` on the toolkit).

### Full gate
- `cargo test -p mnemonic-toolkit`: exit 0, **0 failures** across all targets.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0, zero warnings.
