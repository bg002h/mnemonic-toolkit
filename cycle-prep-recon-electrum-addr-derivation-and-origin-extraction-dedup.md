# cycle-prep recon — 2026-06-06 — electrum-native-seed-address-derivation + descriptor-origin-extraction-dedup

**Origin/master SHA at recon time:** `e9ab49a`
**Local branch:** `master`
**Sync state:** `up-to-date (0 ahead / 0 behind)`
**Untracked:** recon/survey scratch + `.claude/` (none load-bearing).

Slug(s) verified: `electrum-native-seed-address-derivation`, `descriptor-origin-extraction-dedup`. **Slug 1 citations ACCURATE but the crypto is subtler than the one-liner (primary-source caveat below); slug 2 has a STRUCTURAL file-set error (lists `coldcard_multisig.rs`, real 6th is `bitcoin_core.rs`) + line drift.** The two slugs touch DISJOINT files → independent cycles, either order.

---

## Per-slug verification

### `electrum-native-seed-address-derivation`
- **WHAT:** Implement Electrum's own seed→address derivation (PBKDF2 `'electrum'` salt → BIP-32 root → per-version paths) so an Electrum native seed yields its Electrum-correct addresses, instead of being honestly refused.
- **Citations:**
  - `src/electrum.rs` "currently seed-version + phrase↔entropy only" — **ACCURATE.** The file has `validate_seed_version`, `phrase_to_entropy`, `entropy_to_phrase` + HMAC/base-N helpers; **no PBKDF2 seed-stretch, no BIP-32 root, no address derivation** anywhere (grep for `pbkdf2.*electrum`/`b"electrum"`/`2048.*electrum` across `src/` is empty). Greenfield derivation. Header pins Electrum ref `electrum/electrum/mnemonic.py @ e1099925e30d91dd033815b512f00582a8795d25`.
  - `cmd/convert.rs` "the `(ElectrumPhrase, address)` edge is refused" — **ACCURATE.** `ElectrumPhrase` is a `NodeType` (`convert.rs:50`); the ONLY wired electrum edges are `(ElectrumPhrase, Entropy)`/`(Entropy, ElectrumPhrase)` (`:640-641`) + the `(Phrase, ElectrumPhrase)` sibling-pivot refusal (`:683`); `(ElectrumPhrase, Address)` falls to the default one-way refusal. Confirmed by the sibling FOLLOWUP `electrum-phrase-address-refusal-honest-wording` (`FOLLOWUPS.md:3318`).
  - **Reusable xpub→address half:** `src/address_render.rs::render_address_from_xpub` (`:18`) already renders an address from a derived xpub + script type — the new work is ONLY the Electrum-specific seed→root→child-xpub derivation; the rendering is reused.
  - **CRYPTO CLAIM — needs careful primary-source verification at SPEC time (the load-bearing item).** The FOLLOWUP says "legacy `m/0/i`,`m/1/i` (version 01) / segwit paths (version 100)". Verified against Electrum source (keystore.py @ the pinned commit): **segwit ('100')** keystore is `add_xprv_from_seed(..., xtype='p2wpkh', derivation="m/0'/")` → receive/change at `m/0'/0/i` and `m/0'/1/i` (the `m/0'` prefix is REAL and easy to miss — the FOLLOWUP's "segwit paths" is vague). **standard ('01')** new-2.x seed → `BIP32_KeyStore`, P2PKH, derivation `m`, addresses `m/0/i`/`m/1/i`. **CAVEAT:** a naive read conflates the '01'-prefix 2.x *standard* seed (BIP-32, `m/0`) with Electrum 1.x `Old_KeyStore` (a NON-BIP-32 sequence/stretched derivation detected via `is_old_seed`, no HMAC seed-version prefix). The toolkit only recognizes the 4 HMAC-prefixed 2.x versions (`01/100/101/102`), so `Old_KeyStore` is out of scope — but the SPEC MUST pin, per version, the exact (PBKDF2 salt+iters, BIP-32 derivation prefix, child path, script type) by reading Electrum's `mnemonic.py::mnemonic_to_seed` + `keystore.py::add_xprv_from_seed` + the wallet-creation `seed_type→keystore` mapping directly, and **validate against Electrum's `test_wallet_vertical.py` vectors** (the FOLLOWUP already names this). 2FA versions `101/102` are already refused upstream (`--electrum-version` 2FA refusal, `convert.rs:424`) → OUT of scope (scope = standard + segwit only).
- **Action for brainstorm spec:** Add Electrum seed→address derivation in `src/electrum.rs` (PBKDF2-HMAC-SHA512(normalize(phrase), b"electrum"+normalize(passphrase), 2048) → `bitcoin::bip32::Xpriv::new_master` → per-version child derivation) + reuse `render_address_from_xpub`. Surface it as EITHER unrefusing the `(ElectrumPhrase, Address)` convert edge OR a `--from electrum-phrase` path on `mnemonic addresses` (decide in brainstorm; the convert-edge route adds no clap flag, the `addresses` route may). Pin the exact per-version derivation from Electrum source + cite the test vectors. Cite source SHA `e9ab49a` + Electrum commit `e1099925`.

### `descriptor-origin-extraction-dedup`
- **WHAT:** Consolidate the duplicated origin-extraction (`build_slot_fields` ×6, `extract_origin_components`/`origin_capture_regex` ×4) + the A1 `pipeline.rs` recovery loop onto the single canonical `key_regex` + one shared helper, parameterizing the per-parser error prefix. Dissolves `import-parser-hform-origin-tolerance`.
- **Citations:**
  - `build_slot_fields` "in 6 import parsers (`bsms.rs:399`, specter, sparrow, coldcard, **coldcard_multisig**, electrum)" — **STRUCTURALLY-WRONG file-set + DRIFTED lines.** The actual 6 `fn build_slot_fields` are: `bsms.rs:400`, `bitcoin_core.rs:449`, `sparrow.rs:618`, `coldcard.rs:501`, `specter.rs:397`, `electrum.rs:912`. **`coldcard_multisig.rs` has ZERO `build_slot_fields`** (grep count 0) — the FOLLOWUP names it but the real 6th is **`bitcoin_core.rs`**. (`bsms.rs:399 → :400`, drift-1.)
  - `extract_origin_components`/`origin_capture_regex` "in 4 (`bsms.rs:362/:516`, specter:362, sparrow:582, bitcoin_core:413)" — **ACCURATE set, DRIFTED lines.** `extract_origin_components`: `bsms.rs:363`, `bitcoin_core.rs:414`, `specter.rs:363`, `sparrow.rs:583` (all drift-1). `origin_capture_regex`: `bsms.rs:514` (FOLLOWUP :516, drift-2), `bitcoin_core.rs:557`, `specter.rs:355`, `sparrow.rs:565`. The 4-file set {bsms, bitcoin_core, specter, sparrow} is correct.
  - canonical `key_regex` "`pipeline.rs:38`" — **DRIFTED-by-1.** `fn key_regex` at `pipeline.rs:37`; used in the A1 recovery loop at `:187` (`key_regex().captures_iter`).
  - **No file overlap with slug 1:** slug 2's `build_slot_fields` in `wallet_import/electrum.rs` is the IMPORT parser, distinct from slug 1's `src/electrum.rs` native-seed codec. Independent.
- **Action for brainstorm spec:** Correct the file set — `build_slot_fields` is in **{bsms, bitcoin_core, sparrow, coldcard, specter, electrum}** (NOT coldcard_multisig); `extract_origin_components`/`origin_capture_regex` in **{bsms, bitcoin_core, specter, sparrow}**. Lift one shared `extract_origin_components`/`build_slot_fields` into `pipeline.rs` keyed on the canonical widened `key_regex` (`:37`), parameterizing the per-parser error prefix (the genuinely-differing part, per the FOLLOWUP's "why deferred"). Note it dissolves `import-parser-hform-origin-tolerance` (the apostrophe-only `origin_capture_regex` copies collapse into the h-form-widened canonical regex). Cite source SHA `e9ab49a`.

---

## Cross-cutting observations
1. **Slug 2 structural file-set error** (`coldcard_multisig.rs` listed but has no `build_slot_fields`; real 6th is `bitcoin_core.rs`) — same count-ambiguity class as the prior dedup recon ([[feedback_r0_must_read_source_off_by_n]]). Correct it in the SPEC + re-grep all line numbers (uniformly drifted 1-2).
2. **Slug 1 crypto is the risk** — the per-version Electrum derivation (esp. the segwit `m/0'` prefix + the standard-vs-Old_KeyStore distinction) is a primary-source claim that a one-line FOLLOWUP can't be trusted for. R0 MUST require the SPEC to pin each version's derivation from Electrum source + the `test_wallet_vertical.py` vectors (per the cycle-prep crypto-claim rule + [[feedback_verify_cited_apis_against_docs_rs]]).
3. **Two slugs are independent** (disjoint files) — no ordering dependency. Slug 2 is low-risk mechanical; slug 1 is a crypto feature.
4. No incidental cross-pin/version staleness surfaced.

---

## Recommended brainstorm-session scope

**Two independent cycles.**

**Cycle 1 — `descriptor-origin-extraction-dedup` (recommend FIRST — low-risk, clears debt, warms up).** **SemVer: PATCH** (pure refactor; behavior-preserving consolidation — the canonical regex is a superset of the apostrophe-only copies, so re-verify no foreign-format transcript regresses). **Size: net-negative LOC** (remove 6× `build_slot_fields` + 4× `extract_origin_components`/`origin_capture_regex`, add one shared pair in `pipeline.rs` + per-parser error-prefix param). **Locksteps: NONE** (no clap surface change). Also flips `import-parser-hform-origin-tolerance` → resolved. Phase-1 RED is light (the existing import-wallet + foreign-format transcript suite already covers behavior — green-stays-green; add a cell pinning h-form tolerance now reaching all parsers). Mirrors the just-shipped `emit_payload` dedup pattern.

**Cycle 2 — `electrum-native-seed-address-derivation` (the feature; needs the most R0 care).** **SemVer: MINOR** (new derivation capability). **Size: moderate** — new PBKDF2+BIP-32 derivation in `src/electrum.rs` (~60-100 LOC) + the surface wiring + Electrum test-vector fixtures. **Locksteps: depends on the surface** — if it unrefuses the `(ElectrumPhrase, Address)` convert edge only → no clap flag change → manual update for the now-supported edge; if it adds a `--from electrum-phrase` value/flag on `addresses`/`derive` → GUI `schema_mirror` + manual mirror. **R0 MUST gate the crypto:** the SPEC pins each version's (salt, iters, derivation prefix, child path, script type) from Electrum primary source + validates against `test_wallet_vertical.py` vectors; scope = standard('01') + segwit('100') only (2FA '101'/'102' already refused). [[feedback_verify_the_actual_artifact_not_an_analogous_emitter]] applies — test against REAL Electrum-derived addresses, not an analogous BIP-32 path.

**Ordering:** Cycle 1 → Cycle 2 (independent, but dedup-first banks a quick low-risk win and the user can review the electrum crypto SPEC carefully). Each gets its own mandatory R0 gate.
