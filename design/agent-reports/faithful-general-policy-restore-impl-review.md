# Impl Review — faithful general-policy restore (v0.54.0) — PRE-COMMIT ROUND 1

**Verdict: 🟡 — 0 Critical / 3 Important / 2 Minor.** Architecture sound + verified; perimeter hardening required before the 0C/0I gate.

## Verified correct (empirically)
Discriminator (`Some` only for plain wsh/sh-wsh(multi|sortedmulti)+identity+standard `<0;1>`; dup-index + legacy-sh route faithful); Translator total + network-correct + no `<0;1>` fabrication for `None`; routing (`Some` arm passes `tap_internal_key` not literal `None`; `k_opt.expect` safe; general derives addresses from emitted string; all 4 labels switch); funds-safety (watch-only; pk-keyed loud refusal names slug incl. no-multi flagship via moved k-gate); regression (1 caller of `build_multisig_import_payload`; plain `--json` unchanged; full suite green).

## Important (FOLDED — see below)
- **I1 — per-key use-site overrides silently dropped (C1-class).** `plain_template_from_tree` checks only baseline `d.use_site_path`; `d.tlv.use_site_path_overrides` invisible → `wsh(multi(2,@0/<0;1>/*,@1/*))` routes plain, prints `<0;1>` for both, md1 fixed-point BROKEN. The general arm is equally upstream-blocked (md-codec applies baseline to every key). FOLD: loud refusal when `use_site_path_overrides.is_some()`.
- **I2 — hardened wildcard `/*h` silently rendered `/*`.** `bundle --descriptor "wsh(multi(2,@0/*h,@1/*h))"` → restore prints `/*`, exit 0, different wallet. FOLD: loud refusal when `d.use_site_path.wildcard_hardened` (a hardened wildcard can't derive watch-only addresses anyway).
- **I3 — test matrix shortfall (7 of ~16).** The `Some(alts)` MultiXPub translator arm had ZERO coverage (all cells bundle bare `@N` → `multipath==None`). FOLD: add `<0;1>` MultiXPub cell, testnet (network correction), I1/I2 refusal cells, `after`, legacy `sh(multi)`.

## Minor (FOLDED)
- **M1 — bip388 faithful-emit overclaim.** bip388 REFUSES wildcard-only general cards (needs `/<0;1>/*`), faithful only for `<0;1>`. FOLD: fix CHANGELOG/manual/comment prose.
- **M2 — error-attribution overreach.** `faithful_multisig_descriptor` maps EVERY `to_miniscript` error to the pkh-double-wrap slug. FOLD: soften wording ("may indicate…").

Both I1/I2 are pre-existing latent infidelity but violate the SPEC's discriminator contract + the release's "silent-collapse closed" claim. New FOLLOWUP filed: `restore-md1-per-key-use-site-and-hardened-wildcard`.
