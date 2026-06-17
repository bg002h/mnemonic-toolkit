# C6 (mk1 SLIP-0132 stderr-hint) impl review — code reviewer (verbatim)

> Reviewer: opus code reviewer (full tools — minted real cards, ran byte-identity suite). Branch
> `feature/mk1-slip0132-derive-from-path` @ v0.58.1. Verdict GREEN (0C/0I).

---

**Verdict: GREEN (0C/0I)**

Tier-3 C6 (PATCH v0.58.1) — `convert --from mk1 --to xpub` path-implied SLIP-0132 stderr hint. The make-or-break safety property (stdout provably unchanged) is verified empirically; correct, complete, ships.

**Item 1 — core safety (empirical).** STDOUT byte-for-byte unchanged: zpub@m/84' card decoded `--to xpub` → text stdout `xpub: xpub6CatW…` (neutral, never zpub), `--json` stdout `"value":"xpub6CatW…"` only. Hint is stderr-only (`let _ = writeln!`, convert.rs:1075), gated on `args.xpub_prefix.is_none()`, never touches `outputs` or exit code. xpub value is `card.xpub.to_string()` (:1615) emitted raw; `apply_xpub_prefix` swap (:1100) only fires for explicit non-default `--xpub-prefix`. No stdout leak path. `cli_standalone_bijections` B1/B2/B3 all 6 green.

**Item 2 — `path_implied_xpub_prefix`.** 49→Ypub, 84→Zpub, 48'/1'→YpubMultisig, 48'/2'→ZpubMultisig, else→Xpub. Uses `.get(3)` (no panic on 3-component purpose-48). Units cover non-hardened/empty/short/44'/45'/86'/unknown-script-type. Empirically m/49'→ypub hint, m/86'→no hint.

**Item 3 — 4-tuple plumbing.** All 9 `Ok((out, …))` arms updated to 4 elements (6 plain None, input_variant @1478, detected_version @1706, Mk1 path_hint @1644). Destructure @989 takes 4. `compute_outputs` has exactly one caller (@981). Err branch diverges via `return Err`. Compiles, clippy clean.

**Item 4 — suppression.** Exactly `args.xpub_prefix.is_none()` (:1073) — `--xpub-prefix xpub` (explicit-neutral) also suppresses (verified). Mk1 arm sets hint `None` for neutral paths via `(!p.is_default()).then_some(p)` (:1640).

**Item 5 — tests.** Non-vacuous. Cell 4 pins neutral-xpub-at-m/84' → byte-identical stdout while the path-driven hint still fires. 4 cells + 19 slip0132 units (3 new C6) green.

**Item 6 — SemVer/lockstep.** PATCH correct (advisory-only, zero stdout/flag delta; older() v0.55.2 precedent). All version sites at 0.58.1 (Cargo.toml:3, both READMEs, install.sh:32, Cargo.lock, fuzz/Cargo.lock, CHANGELOG). `git grep 0.58.0` hits all legitimate history. Manual `--xpub-prefix` row enriched. FOLLOWUP:4206 flipped resolved recording the product-question outcome. No clap flag change → no schema_mirror obligation. No new ToolkitError variant.

**Item 7 — anything else.** No bug, no panic, no interop regression. Hint and input_variant paths orthogonal. Multi-target fires the hint once. fmt "diff" only in mlock.rs (g6 exemption); convert.rs + slip0132.rs fmt-clean.

Ships.
