# Cycle-prep recon — F2: wc-codec RAID `array_id` same-quorum collision (probability ~1)

- **Source finding:** `design/agent-reports/constellation-eval-2026-07-06.md:64-81` (F2, IMPORTANT).
- **Verified against:** master `f67d0be9` (current HEAD). Read-only recon — no code, no edits beyond this file.
- **Drift status of the eval's citations:** `crates/wc-codec/src/raid.rs` is unchanged since it landed
  (`58209936`, P5, 2026-06-25) and `crates/mnemonic-toolkit/src/cmd/word_card.rs` unchanged since
  `606e0b1e` (P6). The eval's 2026-07-06 line numbers are against byte-identical file content — **zero drift**.

## 1. Per-claim verification

### C1 — `array_id` derived ONLY from ordered cosigner master fingerprints — **ACCURATE**
- Seed construction: `crates/mnemonic-toolkit/src/cmd/word_card.rs:223-244` — for each `--from` mk1 card,
  `card.origin_fingerprint` (4 bytes) is appended in order; a privacy-mode card (no fingerprint) contributes
  `[0u8; 4]` (`word_card.rs:239-242`). Nothing else enters the seed — no network, account, script-type,
  payload bytes, or nonce.
- Hash: `crates/wc-codec/src/pipeline.rs:252-257` — `array_id_from_seed` = **top 22 bits of SHA-256(seed)**.
- The BIP-32 master fingerprint is hash160(master pubkey)[0..4] — independent of network version bytes,
  account index, and script type. Same cosigner set ⇒ same seed ⇒ **identical `array_id` with probability 1**
  (not "~1"): mainnet vs testnet, account 1' vs 2', BIP-48 vs BIP-45, and re-issues all collide. Eval ACCURATE.

### C2 — array_id + n + stripe-width are the only cross-array coherence checks — **ACCURATE**
- `crates/wc-codec/src/raid.rs:363-373` (eval cited 364-372 — exact): the group-coherence loop checks exactly
  `d.array_id != array_id || d.n != n || d.stripe.len() != width`.
- The remaining checks (`raid.rs:380-406`) are set-shape only: `wire_index < n`, no duplicate data index, no
  duplicate P1/P2, and `missing ≤ r_available` (`raid.rs:411-418`). **None discriminate two same-quorum arrays**,
  which agree on all of array_id, n, and W (same-shape xpub payloads ⇒ same
  `W = max(2 + ceil(payload_bits/11))`, `raid.rs:240-245`).
- `raid_reconstruct` never RE-derives the array_id from fingerprints — it is equality-compared only
  (`array_id_from_seed` has exactly one caller: `raid_encode`, `raid.rs:238`). Load-bearing for option (a).

### C3 — mixed-plate solve returns Ok; 16-bit length prefix XOR-cancels; garbage parses as a valid xpub ~half the time — **ACCURATE (mechanism confirmed from source; empirical rate not re-run)**
- Solve: with n−1 data plates from array A and P1 from array B (1 missing), `solve_missing`
  (`raid.rs:466-503`) computes `x_j = P1_B ⊕ Σ_{i≠j} A_i = B_j ⊕ Σ_{i≠j}(A_i ⊕ B_i)` — returns `Ok`
  unconditionally (pure linear algebra, no consistency residual is checked).
- Length prefix: `len_prefix_symbols` / `read_len_prefix` (`raid.rs:108-122`). Same-quorum arrays carry
  equal-length payloads per index, so every `lenA_i ⊕ lenB_i` term vanishes ⇒ the garbage stripe carries a
  **correct** length prefix. The only structural gate, `stripe_to_payload` (`raid.rs:159-173`), checks just
  "declared length fits the stripe" — passes.
- Same-quorum arrays also share every structural byte of the mk1 canonical bytecode (version byte,
  stub count, fingerprints — same masters, path table indicator when accounts match in shape), so those
  positions XOR-cancel and the recovered garbage keeps a well-formed frame; only key/chaincode/stub *values*
  scramble. A random 32-byte x-coordinate under a valid 02/03 prefix lies on secp256k1 with probability ≈ 1/2 —
  fully consistent with the eval's empirical **21/36 ≈ 58% exit-0-wrong-xpub** (this recon did not re-run the
  experiment; no-code constraint).

### C4 — no post-reconstruct integrity check on the reassembled payload — **ACCURATE; nothing missed**
- The per-plate t-bit SHA-256 integrity tag (`pipeline.rs:582-587`, `WcError::IntegrityMismatch` doc
  `lib.rs:138-142` — "refuse, NEVER return wrong payload") covers each plate's OWN engraved stripe. The
  reconstructed stripe's tag was engraved on the *lost* plate — **the MDS-solved stripe is never tag-checked**.
  `raid_reconstruct` returns it raw (`raid.rs:424-440`).
- Downstream, `word_card.rs:348` re-parses via `canonical_to_recovered` →
  `KeyCard::from_canonical_payload_bytes` → `mk_codec decode_bytecode`
  (`vendor/mk-codec/src/bytecode/decode.rs:19-56`): structural checks only — bytecode version, stub_count ≥ 1,
  path indicator, xpub version bytes, secp point parse (`InvalidXpubPublicKey`), trailing bytes.
  **The mk1 canonical (pre-chunking) bytecode carries NO checksum/CRC/hash** (the mk1 BCH checksums live in
  the string layer, which is not part of the word-card payload). The ~50% secp curve filter is the ONLY
  incidental rejection — already reflected in the eval's empirical rate. There is no existing integrity
  primitive the finding missed.

### C5 — doc contract `raid.rs:329-330` exists and is violated — **ACCURATE**
- `raid.rs:329-330` verbatim: "``> r`` missing data plates ⇒ [`WcError::RaidUnrecoverable`] (refuse — never a
  silent wrong reconstruction)". Also violated in spirit: the module guarantee `raid.rs:43-45` ("recovers each
  original (payload_bytes, payload_bits) exactly") and the `RaidArrayMismatch` doc `lib.rs:147-151` ("plates
  from two different wallets … Refuse rather than silently mix") — same-quorum different-wallet plates ARE
  two different wallets and are NOT refused.

### C6 — test blind spot
- KAT 6 (`crates/wc-codec/tests/raid.rs:316-348`, `array_id_does_not_mix_distinct_arrays`) uses two arrays
  with **different seeds** (`seed_of(n,100)` vs `seed_of(n,200)`) — it verifies exactly the case that works and
  never the same-fingerprint-set case. The P5 R0 review noted "22-bit array-id is a matcher not a crypto
  separator" as a leave-as-is nit (`design/FOLLOWUPS.md:4790`) — the probability-1 same-quorum collision was
  not surfaced then. No FOLLOWUP tracks F2 today.

## 2. Confirmed collision mechanism (condensed)

Two arrays for the same cosigner set (any network/account/script-type/re-issue) engrave identical
`(array_id, n)` and share stripe width W. Mix n−1 data plates of one with a parity plate of the other (or any
cross-mix summing to ≥ n plates with ≤ r "missing"), and: coherence gate passes (C2) → MDS solve returns Ok
(C3) → length prefix XOR-cancels (C3) → no tag exists for the solved stripe (C4) → mk1 structural re-parse
passes ≈ 50% (secp curve density) → `*recovered` prints a **valid-but-wrong cosigner xpub at exit 0**.
Failure consequence: re-engraved plate / restored watch-only multisig watches a quorum that can't spend.

## 3. Fix options

### (a) Fold payload-derived material into `array_id` — **NON-WIRE (derivation-only), RECOMMENDED**
- **Key recon fact:** this is NOT a wire-LAYOUT change. The array-id field stays 2 words / 22 bits at the same
  header position (`pipeline.rs:271-276`; header layout `pipeline.rs:288-293`), H0_VERSION (=0,
  `pipeline.rs:58`) untouched. Decode never re-derives the id (C2) — it only equality-compares engraved
  values. **Existing engraved plates keep decoding AND keep reconstructing among themselves.** No KAT pins an
  id value (KATs are round-trip/derivation-agnostic; `tests/raid.rs:391` only compares ids across plates of
  one array).
- **Preferred variant: deterministic payload digest, engine-side.** In `raid_encode`, derive
  `array_id = top22(SHA-256(seed ‖ H(n ‖ ordered (payload_bytes, payload_bits))))`. Deterministic (repro/KATs
  intact, same-array reprints still group); **exclude r** from the digest so P1 stays byte-identical for
  r=1 vs r=2 (preserves the append-only property, KAT 4 `tests/raid.rs:217`). An identical-payload re-issue
  still collides — harmless, since identical payloads produce identical stripes (mixing is a no-op).
- **Rejected sub-variant: random per-array nonce.** Breaks encode determinism (repro builds, KATs), and two
  prints of the SAME array could no longer be cross-mixed — a usability regression with no safety gain over
  the payload digest.
- **Compat cost (small, fail-closed):** a plate re-encoded on a new binary will not group with survivors
  engraved under the old derivation → loud `RaidArrayMismatch`, never silent wrongness. (Today's replacement
  workflow re-encodes the whole array from the n cards anyway.) Old-vs-old arrays still collide — see (c).
- **Scope:** ~15 LOC in `raid.rs` (or ~10 in `word_card.rs:223-244` if done caller-side; engine-side preferred
  so every future caller inherits it) + a new same-quorum-mix KAT + doc updates. No toolkit CLI surface change.
- **Covers:** r=1 AND r=2, all future arrays; collision probability 1 → ~2⁻²².

### (b) Spare-parity consistency oracle — **NON-WIRE, decode-only**
- When more parity equations are present than unknowns solved — `r_available` is already computed at
  `raid.rs:412` — verify the unused equation(s) over all n stripes after the solve; also verify parity when
  0 plates are missing (catches "chimera" mixes that currently return each plate's genuine-but-mismatched
  payload). Foreign-mix detection probability ≈ 1 − 2⁻¹¹ᐧW (W ≈ 55+ for real xpubs — effectively certain).
- P2 availability: yes — `p1`/`p2` are both indexed at `raid.rs:378-406` and passed to the solver; the spare
  one is simply ignored today.
- **Covers:** r=2 with ≤1 missing, and any-r with 0 missing. **Does NOT cover r=1 with 1 missing (the headline
  case) or r=2 with 2 missing.** Works retroactively on already-engraved r=2 arrays.
- **Scope:** ~40-70 LOC in `raid.rs` + KATs. Error path: reuse `RaidArrayMismatch` or add a variant (if new:
  alphabetical-ordering rule + `ToolkitError` mapping + manual error note).

### (c) Loud "verify this xpub" advisory on any `*recovered` plate — **NON-WIRE, universal mitigation**
- Output sites: text `*recovered` marker `word_card.rs:377-390` (marker at :383); JSON `reconstructed` flags
  `word_card.rs:350-351` / `RecoveredJson` :516-517.
- Text/stderr advisory: trivial. If an advisory FIELD is added to `--json`: that is a `--json` wire-shape
  change → `WORD_CARD_SCHEMA_VERSION` bump (`word_card.rs:35-39`) + paired-PR GUI self-update note
  (NOT `schema_mirror`-gated — no flag change).
- **Covers:** everything, including r=1 and all legacy engraved arrays — but as mitigation (human check), not
  prevention. It is the ONLY always-available measure for r=1-with-1-missing short of (a)/(d).
- **Scope:** ~15-40 LOC + manual lockstep: `docs/manual/src/40-cli-reference/41-mnemonic.md:4633` transcript
  line + prose (binary-identical-docs standing rule → regen with the real binary). `.examples-build` corpus
  only changes if `--help` text changes (it doesn't, unless a flag is added).

### (d) Wire-level array payload digest (true post-reconstruct integrity) — **WIRE-BREAKING, not recommended now**
- No existing payload-level checksum exists to reuse (C4), so a real integrity re-check means engraving new
  material: e.g. 2 extra header words carrying top-22 of SHA-256 over the whole array's ordered payloads,
  verified post-solve. Requires H0_VERSION 0→1, `header_word_count` 9→11 (`pipeline.rs:291-293`), geometry/K′
  shift, dual-version decode path, KAT churn: **~150-300 LOC** across `pipeline.rs`/`raid.rs`/tests + toolkit.
  Covers everything (incl. r=1) with certainty ~1−2⁻²², but only for newly engraved arrays — same future-only
  coverage as (a) at ~10× the cost and a genuine wire break.
- **(d-lite), evaluated and REJECTED as ineffective:** re-deriving array_id from the *recovered* cards'
  fingerprints post-solve. In the same-quorum mix the fingerprint bytes XOR-cancel to the CORRECT values
  (same masters on both sides), so the check passes on exactly the garbage it should catch. Adds nothing.

## 4. Engraved-plate compat verdict

- `crates/wc-codec` is **0.1.0**, path-only workspace dep (`crates/mnemonic-toolkit/Cargo.toml:37`), **never
  published to crates.io** (only mk-codec 0.4.1 / md-codec 0.39.1 were, per FOLLOWUPS `word-card-…` entry).
- BUT the feature is **NOT pre-release**: `mnemonic word-card --raid` shipped in released binaries since
  toolkit **v0.74.0 (2026-06-26)** through current v0.83.0, with install.sh distribution, a manual chapter
  (41-mnemonic.md:4540+), and man pages. Real engraved arrays MAY exist (population unknowable, plausibly tiny,
  ~2 weeks of availability).
- **No frozen wire golden exists** — confirmed (eval §2 #3): zero "golden"/pinned-word-sequence fixtures in
  `crates/wc-codec/tests/` or toolkit tests; all KATs are round-trip/property-based and derivation-agnostic.
- **Verdict:** a wire-LAYOUT change (d) is *moderately costly* (version-gated dual decode to honor v0.74.0+
  plates). An array_id DERIVATION change (a) is **effectively free** — no layout change, no golden churn, old
  arrays fully functional among themselves, cross-vintage grouping refuses loudly (fail-closed).

## 5. CLI / lockstep surfaces

- Reconstruct entry: `mnemonic word-card --decode --decode-plate …` (`run_decode_raid`,
  `word_card.rs:327-393`). `--json` emits the `raid-decode` envelope (schema_version "1", NOT
  schema_mirror-gated — flag-name parity only).
- (a)/(b): no CLI surface change, no GUI/manual flag lockstep. (c): manual transcript + prose update
  (+ JSON schema_version bump iff a field is added → paired GUI note). Any new flag (e.g. an opt-out) would
  ripple: schema_mirror + manual + `.examples-build` regen — recommend NO new flag (advisory always-on).
- r=1 vs r=2 exposure split: unknowable (no telemetry). Default `--raid 0` (solo, unaffected). Structurally,
  (b) leaves r=1-with-1-missing uncovered — (c) is the only universal mitigation, (a) the universal prevention.

## 6. RECOMMENDED approach

**(a) + (b) + (c) as one cycle; skip (d).**

1. **(a)** engine-side deterministic payload digest folded into `array_id` (exclude r; preserve KAT 4) —
   prevention for ALL future arrays incl. r=1; collision 1 → ~2⁻²²; effectively free per §4.
2. **(b)** spare-parity post-solve verification — defense-in-depth catching residual 2⁻²² collisions AND
   protecting **legacy** r=2 arrays (which keep probability-1-colliding ids forever; (a) can't reach them).
3. **(c)** always-on `*recovered` advisory (text + stderr + JSON field with schema_version bump) — the only
   measure covering legacy r=1 arrays and the residual r=1 tail.
4. Add the missing same-quorum-mix KAT (same-fingerprint seed, different payloads → must refuse / advisory)
   regardless of option mix, plus a FOLLOWUP entry (none tracks F2 today).

Rationale: legacy plates engraved under v0.74.0–v0.83.0 retain colliding ids no matter what ships — only
(b)+(c) reach them; (a) closes the class going forward at ~15 LOC with zero wire-layout impact because
reconstruct is equality-only over engraved ids. (d)'s extra assurance over the combo is ~2⁻²² on future-only
arrays — not worth a version-gated wire break. Estimated combined scope ~150-250 LOC incl. tests + manual
regen; MINOR toolkit bump, wc-codec stays 0.1.x (unpublished), no sibling-codec ripple.
