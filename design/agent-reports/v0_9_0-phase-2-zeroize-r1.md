# v0.9.0 Phase 2 — Zeroizing wrappers R1 (cross-repo)

**Reviewer:** Opus 4.7 via `feature-dev:code-reviewer` agent, 2026-05-13.
**Branches:**
- mnemonic-toolkit: `v0_9_0-phase-2-zeroize` at HEAD `cae4c7a`
- mnemonic-secret: `v0_9_0-phase-2-zeroize` at HEAD `efe9c71`

## Verdict

**0 Critical / 4 Important / 5 Notable.**

Discipline is functionally sound at every site that uses
`Zeroizing::new(...)` directly. Two real OWNED-Vec wraps are
missing (I-1 + I-3), the bip85 stack-bound `SecretKey` locals
warrant a companion FOLLOWUP (I-2), and the lints' substring-match
design (I-4) masked I-1 + I-2 — tightening evidence anchors would
have caught both at RED.

## Important findings

### I-1 — `synthesize_multisig_full` builds `Payload::Entr` from unwrapped `Vec<u8>` (conf 92)

**File:** `crates/mnemonic-toolkit/src/synthesize.rs:399-401`

```rust
let entropy = seed_mnemonic.to_entropy();
let ms1 = ms_codec::encode(ms_codec::Tag::ENTR, &ms_codec::Payload::Entr(entropy))
```

The `to_entropy()` Vec moves into Payload::Entr without wrapping;
the Payload has no Drop scrub. Peer site `cmd/bundle.rs:921` does
this correctly. The lint passes only because `Zeroizing` appears
elsewhere in synthesize.rs (substring match — see I-4).

**Fix:** `let entropy = zeroize::Zeroizing::new(seed_mnemonic.to_entropy());
... &Payload::Entr((*entropy).clone())`.

Adjacent rows that may also need verification: `synthesize.rs:259`
(`synthesize_unified` if-let arm), `synthesize.rs:690`
(`synthesize_full_descriptor_multisig`) — both clone from a
borrowed `&[u8]` whose source IS Zeroizing-wrapped upstream, but
the local-arg-clone produces a transient unwrapped duplicate.

### I-2 — `bip85.rs::format_hd_seed_wif` and `format_xprv_child` leak unscrubbed stack-bound secp256k1::SecretKey locals (conf 88)

**File:** `crates/mnemonic-toolkit/src/bip85.rs:100-107, 124-134`

`SecretKey::from_slice(&entropy[..32])` copies the 32 bytes into a
fresh stack-bound `SecretKey` (third-party-blocked: has
`non_secure_erase` but no Drop+Zeroize per
[rust-secp256k1 docs](https://docs.rs/secp256k1/latest/secp256k1/struct.SecretKey.html)).
The `PrivateKey { inner }` / `Xpriv { private_key: inner, ... }`
aggregates further. None are scrubbed before function exit.

The lint rows pass because bip85.rs has `Zeroizing` elsewhere (the
`derive_entropy` 64-B buffer), not because the per-function wraps
are in place.

**Fix:** add SAFETY: third-party-blocked doc-comment blocks at
each `SecretKey::from_slice` and `Xpriv { ... }` site citing a new
`rust-secp256k1-secretkey-zeroize-upstream` FOLLOWUP. Add
`SecretKey::from_slice` to `lint_safety_third_party_blocked.rs`
`CALL_PATTERNS` so the comments are enforced.

### I-3 — `cmd/derive_child.rs:106-115` reads `stdin_passphrase` into unwrapped `Option<String>` (conf 85)

**File:** `crates/mnemonic-toolkit/src/cmd/derive_child.rs:106-117`

```rust
let stdin_passphrase: Option<String> = if args.passphrase_stdin {
    let mut buf = String::new();
    stdin.read_to_string(&mut buf)...
    Some(buf)
} else { None };
```

The buffer holds the BIP-39 passphrase from stdin; the
`Option<String>` drops at function end without scrubbing — same
hazard as the clap-field rows the ms-cli lint enumerates.

**Fix:** `let stdin_passphrase: Option<Zeroizing<String>> = ...`
and update the usage at L133-136.

### I-4 — Lint evidence-substring matcher is too coarse (conf 90, methodological)

**Files:**
- `crates/mnemonic-toolkit/tests/lint_zeroize_discipline.rs:42, 232`
- `mnemonic-secret/crates/ms-codec/tests/lint_zeroize_discipline.rs:81`
- `mnemonic-secret/crates/ms-cli/tests/lint_zeroize_discipline.rs:116`

All three lints use OR-semantics file-level substring match. With
`evidence: &["Zeroizing"]` as the only needle, ANY occurrence of
the substring anywhere in the file passes EVERY row targeting that
file. This is what masked I-1 and I-2.

**Fix (minimum touch):** replace generic `["Zeroizing"]` with
per-row specific anchors like `["Zeroizing::new(seed_mnemonic.to_entropy())"]`
or `["Option<Zeroizing<String>>"]`. Option 2 is the simplest;
adding a line-number-range to each row (option 1 in the reviewer's
analysis) is also viable.

## Notable findings

### N-1 — ms-cli encode redundant Zeroizing indirection at `cmd/encode.rs:87-88` (conf 70)

Three in-flight copies: `entropy`, `entropy_for_codec`, and the
unwrapped Vec inside `Payload::Entr`. The middle `entropy_for_codec`
is dead weight — `entropy` would scrub on drop anyway. Suggested:
`Payload::Entr((*entropy).clone())` directly.

### N-2 — `into_parts()` technique is correct; E0509 verified (conf 95)

`std::mem::take` swaps OWNED Vec out before Drop runs on the husk.
Standard idiom for Drop-implementing consuming projections.
**No findings.**

### N-3 — `resolved-slot-entropy-zeroizing-field` deferral tier is correct (conf 75)

`v0.9.2-nice-to-have` defensible: 19-site cascade, no public-API
surface, transit covered by producer + consumer wraps.

### N-4 — SAFETY lint `is_in_test_module` heuristic is correct (conf 80)

Walks back to find `#[cfg(test)]` before first top-level `fn`.
Correctly excludes test-mod sites at parse_descriptor / synthesize
/ verify_bundle.

### N-5 — FOLLOWUP tier classifications are correct (conf 80)

- `rust-bip39-mnemonic-zeroize-upstream` + `rust-bitcoin-xpriv-zeroize-upstream`
  → `external` correct.
- `resolved-slot-entropy-zeroizing-field` → `v0.9.2-nice-to-have` correct.

**Suggested addition:** `rust-secp256k1-secretkey-zeroize-upstream`
(companion entry for I-2).

## Disposition plan

Fold-able in-cycle:
- I-1: one-line wrap fix at synthesize.rs:399 (+ peer-site audit).
- I-3: one-line type change at cmd/derive_child.rs:106.
- I-2: SAFETY comments at bip85.rs:100, 124 + new FOLLOWUP entry
  + add `SecretKey::from_slice` to `CALL_PATTERNS` in
  `lint_safety_third_party_blocked.rs`.
- I-4: per-row anchor tightening across 3 lint files.

Notable folds (opportunistic): N-1 (cosmetic encode.rs cleanup).

Post-fold: R2 verification, then mark Phase 2 closed.

## Sources

- [rust-secp256k1 SecretKey](https://docs.rs/secp256k1/latest/secp256k1/struct.SecretKey.html)
- branches at the stated HEADs (cae4c7a / efe9c71)
- Phase 2 RED lints + plan §"Phase 2 — Impl"
