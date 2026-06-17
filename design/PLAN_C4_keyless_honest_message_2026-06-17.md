# PLAN — C4: honest refusal for a KEYLESS concrete descriptor at `bundle --descriptor` (2026-06-17)

> Tier-3 item C4, user-decided. Recon (`design/agent-reports/` — C4 feasibility) **definitively
> disproved "allow keyless bundling"**: a keyless descriptor (`wsh(and_v(v:ripemd160(H),older(N)))`,
> no pubkeys) hits THREE stacked blockers downstream (`concrete_keys_to_placeholders` requires ≥1 key
> `pipeline.rs:298`; `parse_descriptor` requires ≥1 `@N`; the bundle/md1/restore model is
> wallet-policy-only — `synthesize.rs:346 debug_assert!(is_wallet_policy())`, restore refuses
> template-only `restore.rs:1232`, no key = no cosigner = no mk1 card = no coherent bundle). A keyless
> script has no secret to back up — the correct artifact is a watch-only descriptor FILE, which
> `export-wallet --descriptor … --format descriptor` already emits (verified exit 0). **So the fix is
> the user's actual complaint: the refusal MESSAGE is vacuous for a keyless input ("must carry a key
> origin" — there is no key to give an origin to). Replace it with an HONEST message that names the
> real reason + routes to export-wallet.** **Source SHA: toolkit `1dec924`** (HEAD == origin/master ==
> v0.58.1; citations grep-verified). **NO-BUMP** (error-message-only; the refusal — `DescriptorParse`,
> exit 2 — is unchanged; no output/flag/behavior-contract change). Commit to master.

## Citations (grep-verified @ `1dec924`)

| Surface | Location |
|---|---|
| the gate (`(false,false)` arm = keyless/origin-less refusal) | `crates/mnemonic-toolkit/src/wallet_import/pipeline.rs::classify_descriptor_form :132-147`, arm `:141-145` |
| origin-annotated-key probe (requires `[fp/path]`) | `pipeline.rs::key_regex :37-43` |
| bundle call site (refuses keyless FIRST) | `cmd/bundle.rs:325` |
| existing assertions on "must carry a key origin" (BOTH key-but-origin-less → UNCHANGED) | `tests/cli_bip388_policy_intake.rs:298` (bare-xpub policy), `pipeline.rs:459` (raw 66-hex pubkey `wpkh(0279be…)`) |
| the escape hatch (already works for keyless) | `export-wallet --descriptor "<keyless>" --format descriptor` → exit 0 (`cmd/export_wallet.rs:452` passthrough) |

## Design

**New probe** `pub(crate) fn has_any_key_token(s: &str) -> bool` in `pipeline.rs`: matches a key the
bundle would engrave as a cosigner, with OR without an origin —
- an extended key (xpub-family): `[xtyzuvYZUV]pub[A-HJ-NP-Za-km-z1-9]+`, AND
- a 66-hex COMPRESSED pubkey: `\b0[23][0-9a-fA-F]{64}\b` (33 bytes; `02`/`03` prefix).

**Deliberately CONSERVATIVE on 64-hex** (R0 to confirm): a 64-hex token is ambiguous — it is BOTH an
x-only taproot pubkey AND a `sha256()`/`hash256()` hash literal. The probe does NOT match bare 64-hex,
so: (a) a keyless `sha256`-hashlock descriptor → correctly "no key" → honest message (a sha256 hash is
not a cosigner key); (b) the rare degenerate raw-x-only `tr(<64hex>)` with NO origin → would get the
keyless message (acceptable: it's an exotic input the bundle path doesn't otherwise support, and the
status-quo message is no better). ripemd160/hash160 (40-hex) never collide.

**Split the `(false,false)` arm** of `classify_descriptor_form`:
```rust
(false, false) => {
    if has_any_key_token(input) {
        // keys present but no [fp/path] origin → unchanged, correct message.
        Err(ToolkitError::DescriptorParse(
            "descriptor has neither @N placeholders nor [fp/path]-annotated keys; \
             concrete descriptors must carry a key origin, e.g. [<fp>/84h/0h/0h]xpub…".into()))
    } else {
        // truly KEYLESS (hashlock/timelock only) → no cosigner to engrave → honest route.
        Err(ToolkitError::DescriptorParse(
            "this descriptor has no keys to engrave as a cosigner card — a keyless script \
             (hashlock/timelock only) is not a coherent m-format bundle. Emit it as a watch-only \
             descriptor file: `export-wallet --descriptor '<descriptor>' --format descriptor` \
             (or `--format bitcoin-core`).".into()))
    }
}
```
Same `DescriptorParse` type + exit 2 in both branches — the only change is which message text. The
only direct callers of `classify_descriptor_form` are **bundle + verify-bundle** (R0-r1 M3:
xpub-search does NOT call it — it uses `expand_bip388_policy` directly; export-wallet uses
`is_at_n_form` only) — both inherit the split; the honest message reads acceptably for both (a keyless
descriptor can't be a coherent bundle on either). (R0-r1 M1: a raw-x-only `tr([fp/path]<64hex>)` WITH
an origin also reaches the `(false,false)` arm — key_regex group-3 needs an xpub-family prefix a raw
x-only lacks — so it would get the keyless message; acceptable, the exotic input is unsupported either
way and the status-quo message is equally wrong. A normal `tr(xpub…)` contains an xpub → matched → not
mis-flagged.)

## TDD

Module unit tests in `pipeline.rs::tests` (where `classify_descriptor_form`'s tests already live):
1. **Keyless → honest message.** `classify_descriptor_form("wsh(and_v(v:ripemd160(0000…0),older(1234567)))")`
   → Err whose message contains `export-wallet --descriptor` and NOT "must carry a key origin".
2. **Key-but-origin-less → UNCHANGED.** The existing `:459` cell (`wpkh(0279be…)` raw pubkey) still
   asserts "must carry a key origin" — confirm it stays green (the 66-hex pubkey trips `has_any_key_token`).
   Add a bare-xpub case `wpkh(xpub6CatW…/0/*)` (no origin) → also "must carry a key origin".
3. **`has_any_key_token` unit:** xpub/tpub/ypub/zpub → true; `0279be…`(66-hex) → true; a keyless
   `ripemd160(<40hex>)`/`sha256(<64hex>)`/`older(N)` body → false.
A CLI cell in `tests/cli_bundle_*.rs` (or a new small file): `bundle --descriptor "<keyless>"` → exit 2,
stderr contains `export-wallet --descriptor` (the honest route). The existing
`cli_bip388_policy_intake.rs:298` bare-key-policy cell (→ "must carry a key origin") MUST stay green.

**Non-vacuity:** revert the split → cell 1's keyless input emits "must carry a key origin" again →
the `export-wallet --descriptor` assertion fails.

## Lockstep / SemVer

- **NO-BUMP.** Error-message-only; the refusal (`DescriptorParse`, exit 2) is unchanged — no output,
  flag, or behavior-contract change. No version bump, no tag; commit to master. No GUI `schema_mirror`
  (no flag), no new `ToolkitError` variant. (R0 to confirm NO-BUMP vs a PATCH with a CHANGELOG note —
  the message IS user-facing, but error text is non-stable per the v0.48.0/mstring precedent.)
- **Manual:** optional — the `bundle --descriptor` row could note that keyless descriptors are routed
  to export-wallet, but error-message text isn't manual-mirrored; likely no manual change.
- **FOLLOWUP:** there is no dedicated C4 slug (the recon found it folded into resolved entries). FILE a
  brief `bundle-keyless-descriptor-honest-refusal` slug → resolved, recording: keyless can't bundle
  (3 blockers), shipped an honest routing message (NOT allow), export-wallet --descriptor is the door.
- **fmt gate:** `cargo +1.95.0 fmt --all` then revert `mlock.rs`.

## Execution
1. R0 architect review of THIS plan → GREEN (0C/0I), persist to
   `design/agent-reports/c4-keyless-honest-message-plan-r0-round1-review.md`. (R0: confirm the
   conservative-64-hex call + NO-BUMP vs PATCH + that both existing assertions stay green.)
2. TDD (RED), implement the probe + arm split, GREEN + full suite + clippy + fmt gate.
3. Per-phase impl review → 0C/0I.
4. Commit (NO-BUMP) to master, push, CI green.
