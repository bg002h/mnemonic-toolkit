# Design-recon — faithful general-policy `restore --md1` reconstruction (the long-term fix for C1/C2)

**Source SHA:** `5d599f7` · **Inputs:** the 3-agent backup→restore review (`fragment-backup-restore-review-2026-06-11.md`) + 2 deep-dive design recons (toolkit redesign + md-codec Check fix), 2026-06-11. User directive: "fix the bug, not permit it — the long-term fix now."

## The pivot (confirmed empirically by both recons)
`md_codec::to_miniscript::to_miniscript_descriptor(&d, 0)` ALREADY renders the complete, faithful, watch-only descriptor from the md1 tree (expands all keys + walks the whole tree incl. timelocks/hashlocks/andor). `restore.rs:838` computes it as `ms0` and then **throws it away** — `template_from_descriptor(&ms0)` (`wallet_export/mod.rs:283 Wsh(_) => WshMulti`) collapses any wsh to plain multi, and `:881 build_descriptor_string` rebuilds `wsh(multi(k,all-keys))`, dropping the policy. The fix is to USE `ms0`, not discard it.

## Two parts (one self-contained toolkit cycle + one cross-repo enabler)

### PART 1 — toolkit: faithful reconstruction for general policies (MINOR, self-contained, KILLS the silent bug)
- **Use `ms0` directly for general (non-plain-template) policies**, via a `translate_pk` pass that fixes the 3 mechanical caveats of raw `ms0` (all proven dissolvable at the pinned miniscript rev `95fdd1c`, where `ms0` IS the toolkit's `MsDescriptor` — no string bridge):
  1. single-path `/0/*` → canonical multipath `<0;1>/*` (from `d.use_site_path.multipath`);
  2. depth-0 xpubs (md1 stores chain_code+pubkey only) — so **byte-equality with `export-wallet --descriptor` is impossible by design** (v0.49.1 already hit this); addresses are unaffected (derivation uses only chain_code+pubkey);
  3. network hardcoded `Main` → network-correct xkey.
- **Structural discriminator** `plain_template_from_tree(&d.tree, &d.use_site_path)` — `Some(template)` ONLY for strictly-plain `wsh/sh-wsh(multi|sortedmulti)` with identity key indices `[0..n-1]` and standard `<0;1>` use-site → routes to the existing byte-exact `build_descriptor_string` path (26 goldens stay green). `None` → the faithful arm. Do NOT discriminate on `template_from_descriptor` (that IS the collapse).
- **Addresses**: general arm derives from the EMITTED descriptor string (`MsDescriptor::from_str` + `derive_receive_addresses`), mirroring the taproot R0-v2-C1 precedent (self-consistency: print and address agree). Plain arm unchanged.
- **Funds-safety re-gate: advisory only, NEVER refuse** — refusing to reconstruct a user's existing funded wallet is itself a lockout/funds-safety failure. At most a stderr warning from `Descriptor::sanity_check()`.
- **Scope boundary** (crisp first cycle): reconstructs every md1 whose keys sit inside `multi()`/`sortedmulti()` (timelocks, hashlocks, andor/and_v/or_*, thresh, decay vaults, under wsh/sh-wsh). The `pk(@N)`/`pkh(@N)`-keyed shapes (e.g. the v0.19.0 flagship `wsh(andor(pkh(@0),after(...),...))`) currently error in `to_miniscript` (PART 2) → first cycle maps that to a LOUD, clear `ModeViolation` naming the md-codec slug (no longer silent). After PART 2 ships + pin bump, those shapes flow through the SAME general arm with zero toolkit changes.
- **`--json` wire-shape**: `wallet_type`/header should distinguish "miniscript-policy" from "k-of-n multisig" → GUI paired-PR + manual restore-chapter update (no clap flag change → no `schema_mirror`).

### PART 2 — md-codec (cross-repo): Check double-wrap renderer fix (unblocks pk(@N)/pkh(@N) shapes)
- **Root cause (empirically confirmed):** the toolkit encoder emits `Tag::Check(Tag::PkH)` in non-tap context (`parse_descriptor.rs:601-624`, gated on `tap_context`); the md-codec renderer (`to_miniscript.rs:290-307`) re-applies `Check` to the already-checked `PkH` → `Check(Check(PkH))` = `c:` over type-B → "c:pkh cannot wrap type B". md-cli fixed this in the v0.30 wire redesign (Q12); the toolkit ported the walker 5 days before the freeze and never got the normalization. Multi-keyed policies avoid it (keys go through `Body::MultiKeys`, never the PkK/PkH/Check arms).
- **Fix** in `md-codec/src/to_miniscript.rs::node_to_miniscript`: **Tier A1** (minimal) — `Tag::Check` over a bare `PkK/PkH` returns the child directly (Check-idempotence, mirrors md-cli `format/text.rs:363-385`); **Tier A2** (complete, ~25 LOC) — thread `want_k` to render bare keys at type-K positions, closing the deeper `Check(or_i(pk_k,pk_k))` shape C too. Recommend A2. Both are strictly error→success (no currently-succeeding input changes). A1 is MANDATORY regardless: affected md1 cards already exist on steel (re-engraving impossible).
- **md-codec is crates.io-published 0.35.0** → renderer-tolerance = PATCH 0.35.1 (or MINOR 0.36.0). md-cli exact-pin lockstep. Toolkit pins `md-codec = "0.35"` → `cargo update -p md-codec` after publish. **crates.io publish is irreversible → confirm before executing.**
- **No existing FOLLOWUP** for this bug; file cross-cited companions (descriptor-mnemonic `to-miniscript-check-pkh-double-wrap` + toolkit pin/restore-unblock).

## Fast-follows (separate cycles)
- **C2**: `export-wallet --from-import-json` + template-requiring format collapses general descriptors via the same `template_from_descriptor` (`export_wallet.rs:777-781`) — a second silent surface. Apply the faithful/gate approach there too.
- **PART 3 (optional toolkit MINOR)**: drop the `tap_context` gate in `parse_descriptor.rs:601-624` so the toolkit walker collapses unconditionally → cross-tool `wallet_policy_id` parity (today toolkit & md-cli diverge for wsh-miniscript with bare keys). Wire-content change → MINOR.
- **Bundle advisory**: `bundle --descriptor` currently engraves cards `restore` can't reconstruct (the blocked shapes) with no warning; and the creation door appears ungated.
- The cost-layer hash-position scan bugs (I-1/I-2) + the hash256/ripemd160/hash160 test backfill + md-codec round-trip vectors (from the first review).

## Recommended sequence
1. **PART 1 (toolkit)** — kills the CRITICAL silent bug for all multi-keyed general policies; self-contained; no publish. Headline.
2. **PART 2 (md-codec)** — unblocks the pk-keyed flagship; needs crates.io publish (confirm) + toolkit pin bump → lights up the flagship through PART 1's general arm.
3. C2, then PART 3 / advisories / cost-layer / test-backfill.

Each part is its own R0-gated cycle.
