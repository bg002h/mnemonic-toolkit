# Cycle Y (toolkit v0.73.3) — Post-Implementation Adversarial Whole-Diff Review

**Cycle:** Cycle Y — LOUD funds-safety advisory for a CUSTOM use-site on a NUMS-taproot card (toolkit PATCH v0.73.2 → v0.73.3, toolkit-only).
**Reviewer:** independent adversarial execution review (fork), over the whole diff.
**Date:** 2026-06-24.
**Branch:** `feat/cycle-y-taproot-advisory`.

**Verdict: GREEN — 0 Critical / 0 Important.**

## Verified (all spec requirements met)

**Message byte-exactness (§3.3):** Captured the runtime message from the *compiled binary* — **BYTE-EXACT** match to spec §3.3, including the em-dash (`—`), `<0;1>/*`, uppercase `NOT`/`PERMANENT LOSS OF FUNDS`, and the `\`-continuation whitespace collapse (no double-spaces, no dropped words). The message is a pure static literal — zero interpolation, no `format!`, no descriptor/key data → no secret leak (descriptor is watch-only public material anyway).

**Predicate (`taproot_override_classify.rs:92`):** `custom_use_site_nums_taproot_card = taproot_override_card(d) && restorable_taproot_override_card(d)`, correct `md_codec::Descriptor` input type. Truth-table proven by 6 unit tests from real md1 wire fixtures: custom override → TRUE; baseline uniform / sortedmulti_a / non-NUMS / hardened → FALSE. Baseline returns FALSE because uniform suffix ⇒ `use_site_path_overrides == None` ⇒ `taproot_override_card == false`. No false-positive leak.

**5 fire sites — all wired, correct variables, correct functions:**
- `bundle.rs:1805, 1855, 2129` (3 distinct functions: unified/concrete/import-json) → `&descriptor`.
- `import_wallet.rs:1329` inside `for p in &parsed` → `&p.descriptor`.
- `restore.rs:3661` inside `fn run_multisig` (3050–3674) → `&d` (the md1-decoded descriptor), placed just before `emit_output_class_advisory` + `Ok(0)` on the success path — proceeds, does not refuse. Confined to `run_multisig`; NOT in `run_multisig_template_completion` or single-sig.

**Mutual exclusion / parity:** Loud trigger (`override && restorable`) is the exact complement of the unchanged calm trigger (`override && !restorable`, `unrestorable_advisory.rs:116-117`). Mutually exclusive over taproot-override cards; baseline fires neither. Proven by `mutually_exclusive_with_unrestorable_taproot_advisory` (`loud ^ calm`).

**Version sites:** All 7 at `0.73.3` (Cargo.toml, Cargo.lock, fuzz/Cargo.lock, both READMEs, install.sh self-pin, CHANGELOG). install.sh only the toolkit self-pin row changed — md/ms/mk sibling rows untouched.

**Manual:** New `### Custom use-site on a NUMS-taproot card {#custom-use-site-nums-taproot}` subsection with matching in-page anchor; quoted prefix matches the real message; truncated-prose form (not a runnable transcript) → no golden-backing needed. Baseline-does-not-warn noted.

**FOLLOWUP:** `restore-md1-taproot-use-site-override-arm` stays PARTIALLY RESOLVED — slug heading still "stays GUARDED", Status line unchanged, appended Cycle Y sub-note explicitly states "status STAYS PARTIALLY RESOLVED" and "no reconstruction gap closed". Not flipped to RESOLVED.

**Tests re-run live:** integration 8/8, predicate 6/6, message 1/1.

## Minor (non-blocking, no action required)

- **M1** — `FundsSafetyShape`, `FundsSafetyAdvisory`, `funds_safety_advisories`, `emit_funds_safety_advisories` are `pub` while only used `crate`-internally. This mirrors the existing `pub` visibility of the sibling `unrestorable_advisories` / `UnrestorableAdvisory` API in the same module (consistency with the established pattern), so it is intentional and harmless — not worth changing.

No regressions introduced. Ready to ship.

## Independently re-verified by the implementing session (before commit)

- Runtime message byte-exact to spec §3.3 at BOTH bundle and restore surfaces.
- Restore custom override: exit 0, warning on stderr, addresses (`<2;3>/*`) still on stdout (proceed-and-warn).
- Exactly 5 `funds_safety_advisories(` fire sites (3 bundle + 1 import + 1 restore); 1 emit in restore (confined to `run_multisig`).
- Gates: full `cargo test -p mnemonic-toolkit` 0 failed; `cargo clippy --all-targets -D warnings` 0; `cargo +1.95.0 fmt --all --check` only `mlock.rs` diffs (exempt); `make -C docs/manual lint` OK; `cargo +nightly fuzz build` 0 errors; install-pin self-pin → `mnemonic-toolkit-v0.73.3`; CHANGELOG section `[0.73.3]` present.
