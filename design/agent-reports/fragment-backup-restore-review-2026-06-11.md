# Review — less-common miniscript fragments across backup→restore (2026-06-11)

**Trigger:** user asked Fable to review backup/restore of `older()`/`after()`/`sha256()`/`hash256()`/`ripemd160()`/`hash160()`-containing miniscripts — "less common and may have been under-tested given recent findings" (the v0.53.9 `older()` consensus-mask fix). Three parallel Fable reviewers (field-validation+cost / md-codec wire / backup-restore lifecycle). Source SHA `5d599f7`.

## Headline
The user's instinct was right. **Backup is faithful for all 6 fragments; the md-codec wire codec is faithful (no `older()` analog); but RESTORE has a verified CRITICAL silent funds-safety bug** — `restore --md1` reconstructs a DIFFERENT wallet (lock condition dropped) for a reachable class of general policies, exits 0, and stamps it "verified".

## C1 — CRITICAL (funds-safety, verified on the binary + mechanism confirmed by hand)
`restore --md1` silently reconstructs the WRONG descriptor for general `wsh` policies whose keys sit inside `multi()`/`sortedmulti()`.
- Repro: `bundle --descriptor "wsh(andor(multi(2,@0,@1),older(1000),multi(1,@2)))"` → md1 → `restore --md1` exits 0, prints `wsh(multi(2,K0,K1,K2))#qplcgqnp` — a plain 2-of-3, the timelock/decaying structure GONE. Same for `after()`, `sha256()`, all hashlocks (the fragment is never inspected at restore).
- Mechanism (confirmed): `wallet_export/mod.rs:263 template_from_descriptor` classifies by TOP-LEVEL wrapper only — `Wsh(_) => WshMulti|WshSortedMulti` unconditionally (`:283`), never inspecting the inner script. `bundle.rs:1036 extract_multisig_threshold` find_maps the FIRST `k` anywhere in the tree. `restore.rs:839-844` + `:881 build_descriptor_string` rebuilds a plain `wsh(multi(k, all-N-keys))`, discarding the policy tree.
- Aggravating: first-recv address is derived from the REAL tree (`restore.rs:898`) → printed address doesn't match printed descriptor; `--from <own seed>` cross-check passes → false "verified" banner; `--format sparrow` emits the wrong importable payload.

## C2 — CRITICAL (same root cause, second surface)
`export-wallet --from-import-json` template emitters silently drop the lock (e.g. bitcoin-core blob `wsh(and_v(v:multi(2,…),older(1000)))` → `--format sparrow` → `wsh(multi(2,@0/**,@1/**))`). Same `template_from_descriptor` Wsh catch-all, reached via the `--from-import-json` path.

**Recommended C1/C2 fix:** structural template gate before rebuild — the md1/import tree must be EXACTLY `wsh(multi|sortedmulti)`, `sh(wsh(multi|sortedmulti))`, or `tr(NUMS, multi_a|sortedmulti_a)`; anything else → clear refusal (ModeViolation: "general wallet-policy md1 is not yet reconstructable by restore"). Small diff, converts SILENT-wrong → LOUD-refuse at both sites. Longer term: reconstruct from the tree itself (keyless template + `expand_per_at_n` via `wallet_import::pipeline::expand_bip388_policy`) — dissolves the limitation for all fragments.

## Other findings
- **I1 (Important):** general-policy md1 with `c:pk`/`c:pkh`-shaped keys (e.g. the v0.19.0 flagship `wsh(andor(pkh(@0),after(12000000),or_i(and_v(v:pkh(@1),older(4032)),and_v(v:pkh(@2),older(32768)))))`) fails LOUD but cryptic at `restore.rs:839` ("c:pkh … cannot wrap a fragment of type B"); no toolkit command renders the descriptor back from the card; bundle never warns these cards aren't restorable.
- **I2 (Important, cross-repo md-codec):** md-codec `to_miniscript` renders `Check(PkK)` as `c:pk(…)` → rust-miniscript type error (root of I1). Needs descriptor-mnemonic companion FOLLOWUP.
- **Cost layer (Agent 1, both fail-closed, wrong-blame errors):** I-1 `compare-cost --miniscript` breaks on alpha-leading 40-hex `hash160`/`ripemd160` digests (`cost/translate.rs::collect_abstract_labels:178` hex-skip covers 64/66 not 40). I-2 single-leaf `tr()` `--descriptor` corrupts 64-hex `sha256`/`hash256` during x-only→compressed inflation (`cost/strip.rs::inflate_xonly_to_compressed_even_y`). Both: textual hex scans that don't know key-position vs hash-arg-position. `build-descriptor` preview NOT affected.
- **Field-validation (Agent 1):** CORRECT for all 6 (gate `check_hashlock` len 64/40 + hex; render keywords right; cost cap/enumerate single combined hash arm). UNDER-TESTED: `hash256`/`ripemd160`/`hash160` have ZERO dedicated cells — a `want_len` 40↔64 swap at `gate.rs:241` would pass the whole suite (step-2 from_str still backstops, so degrade not breach).
- **md-codec wire (Agent 2):** NO bugs. 6 distinct tags (`tag.rs:127-132`: after 0x1B, older 0x1C, sha256 0x1D, hash160 0x1E, hash256 0x1F, ripemd160 0x20). `Body::Timelock(u32)` full-width (NO masking — the older() bug has no md-codec analog). Digests byte-exact. UNDER-TESTED: `hash256()`+`ripemd160()` untested above the bitstream; proptest strategy only generates `Older(1..=65535)`.
- **I3 / lifecycle (Agent 3):** ZERO end-to-end bundle→restore tests for any of the 6 fragments — exactly why C1 survived.
- **Minors:** hashlock walker unit tests assert only `Body` variant not digest bytes (`parse_descriptor.rs:2324+`); `inspect --md1` stderr note misleadingly claims "stdout is a keyless descriptor template".

## Recommended sequencing
1. **C1/C2 structural gate (CRITICAL, fix-now)** + RED-first lifecycle tests — stop the silent wrong reconstruction at both sites.
2. Cost-layer hash-position-aware scan (I-1/I-2) — one cycle, same defect class.
3. Test backfill (hash256/ripemd160/hash160 gate + md-codec round-trip vectors, paired companion) + the `want_len`-swap guard.
4. md-codec `Check(PkK)` render gap (I2) + I1 backup-time advisory — cross-repo.
5. Long-term: tree-based general-policy restore reconstruction (makes these vaults actually restorable).
