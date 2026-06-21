# Cycle-prep STRICT-GATE recon — cycle-4 codec funds (H6 / M4 / M6)

**Mode:** recon ONLY (pre-brainstorm). No brainstorm/spec/plan/code written.
**Source report:** `design/agent-reports/constellation-bughunt-2026-06-20.md` (H6 §172, M4 §258, M6 §468).
**Fix-program plan:** `design/PLAN_constellation_bughunt_fix_program.md` (Tier-2 / WS-MD-BCH §447, WS-MS-CODEC §467).
**Recon date:** 2026-06-21.

## Per-repo sync state (verified against origin default branch)

| repo | path | default branch | origin SHA (verified-against) | local HEAD | verify mode |
|---|---|---|---|---|---|
| md-codec / md-cli | `/scratch/code/shibboleth/descriptor-mnemonic` | `main` | **`58cc9ec`** (`release: md-cli 0.8.0`) | `54dd765` (drifted) | `git show origin/main:<path>` BYTES |
| ms-codec / ms-cli | `/scratch/code/shibboleth/mnemonic-secret` | `master` | **`6b28918`** (`docs(claude): refine ultracode policy`) | `6b28918` (clean) | `git show origin/master:<path>` |
| toolkit (consumer) | `/scratch/code/shibboleth/mnemonic-toolkit` | `master` | n/a (consumer; pins below) | `364d296` (`feature/own-account-subset-search`) | working tree |

**Citation-path note (affects ALL three findings):** the report cites paths as `md-codec/src/…` and `ms-codec/src/…`, but the live tree is `crates/md-codec/src/…` / `crates/ms-codec/src/…`. The `crates/` prefix is omitted throughout the report. Symbol + intra-file line citations are otherwise tracked below. This is a uniform STRUCTURALLY-INCOMPLETE-prefix, not per-citation drift.

**Versions / publish chain (origin):**
- md-codec **0.37.0** (registry crate) → md-cli **0.8.0** pins `md-codec =0.37.0`.
- ms-codec **0.4.4** (registry crate) → ms-cli **0.8.0** pins `ms-codec =0.4.4`.
- toolkit (master) pins `md-codec = "0.37"`, `ms-codec = "0.4.4"`, `codex32 = "=0.1.0"` (transitive, mirrors ms-cli).

---

## Per-slug verification

### H6 — md1 single-string encode runs outside the BCH(93,80,8) regular-code domain (no length cap)
**id:** `encode-no-regular-code-length-cap` (encode side; companion to M4 decode side).
**WHAT:** Default `md encode` (no `--force-chunked`) emits a single codex32 md1 string of arbitrary length via `wrap_payload` → `encode_md1_string`, with NO guard that data ≤ 80 / codeword ≤ 93. codex32's regular code is BCH(93,80,8), defined only to 93 symbols. Auto-chunking exists (`SINGLE_STRING_PAYLOAD_BIT_LIMIT = 320`) but is opt-in (only fires under `--force-chunked`).

**Citations:**
- `codex32.rs:67` `wrap_payload` — **ACCURATE.** Live `crates/md-codec/src/codex32.rs:67` `pub fn wrap_payload(payload_bytes: &[u8], bit_count: usize) -> Result<String, Error>`. Body (67–84) reviewed: emits `HRP + data_symbols + 13-sym checksum` with **no** `data_symbols.len()`/`bit_count` ceiling check. CONFIRMED no length cap.
- `encode.rs:136` `encode_md1_string` — **ACCURATE.** Live `crates/md-codec/src/encode.rs:136` `pub fn encode_md1_string(d: &Descriptor) -> Result<String, Error>`; body is `let (bytes, bit_len) = encode_payload(d)?; crate::codex32::wrap_payload(&bytes, bit_len)` — no cap.
- `SINGLE_STRING_PAYLOAD_BIT_LIMIT = 320` — **ACCURATE** (DRIFTED location): live `crates/md-codec/src/chunk.rs:219` `pub const SINGLE_STRING_PAYLOAD_BIT_LIMIT: usize = 64 * 5;` (= 320). Report gave no line; flagged for completeness.
- "Default `md encode` uses this path … auto-chunking is opt-in" — **ACCURATE + strengthened by live trace.** `crates/md-cli/src/cmd/encode.rs:80` `if args.force_chunked { split(...) } else { … encode_md1_string(&descriptor) }`. The `else` branch emits a single string **unconditionally**, regardless of payload length. `split()` is reached ONLY under `--force-chunked` (both text :80 and `--json` :57 paths). There is no length-triggered auto-chunk despite the report's parenthetical "or by automatic chunking when payload exceeds 320 bits" (that prose appears only in `md repair`'s help text at `md-cli/src/main.rs:241` — aspirational/wrong; no such auto-chunk exists in the encode path).
- spec `beta_has_order_93_regular` / generator order 93 — **ACCURATE** (DRIFTED location): live `crates/md-codec/src/bch_decode.rs:477` test `beta_has_order_93_regular` asserts β^93 == 1; `REGULAR_CHECKSUM_SYMBOLS = 13` at `codex32.rs:18`.

**Numeric note (MINOR imprecision, not a defect):** the report's "any descriptor whose payload exceeds **~67 data symbols**" understates the true cap. The regular code's data ceiling is **80 symbols** (93 − 13 checksum), authoritatively (BIP-93: "80 characters of data and 13 characters of the checksum"). Brainstorm should state the guard as **data > 80 symbols / codeword > 93**, not 67. The 67-figure is harmless to the verdict but must not propagate into the SPEC.

**STILL-REPRODUCES verdict: REPRODUCES.** Both functions on origin/main lack any length cap; the default `md encode` else-branch emits over-length single strings with no error. A 2-of-3 keyed template (~1587-bit payload ≈ 331 symbols) is emitted as a single out-of-code md1 that even round-trip-verifies (polymod is length-agnostic). No later commit added a cap.

**Action for brainstorm-spec:** hard length guard in `wrap_payload` (or `encode_md1_string`) rejecting `data_symbols.len() > 80` / codeword > 93 with a new typed `Error` (report suggests `Error::PayloadTooLongForSingleString`); encoders needing more MUST chunk. Cite source SHA **`58cc9ec`**, live lines `codex32.rs:67`, `encode.rs:136`, `chunk.rs:219`, `md-cli/src/cmd/encode.rs:80`.

---

### M4 — `decode_regular_errors` / `chien_search` accept `len > 93` → aliased error positions (decode-side companion to H6)
**id:** `chien-search-unbounded-length`.
**WHAT:** `chien_search` iterates `0..data_with_checksum_len` evaluating `Λ(β⁻ᵈ)` with no upper bound; BETA has order 93, so for `len > 93`, degrees `d` and `d+93` alias → a correction can land at the wrong aliased position while still zeroing the residue, passing re-verify → wrong-but-valid descriptor on repair. Reachable via `md repair` (and toolkit `mnemonic repair --md1`).

**Citations:**
- `bch_decode.rs:284` `chien_search` — **ACCURATE.** Live `crates/md-codec/src/bch_decode.rs:284` `fn chien_search(lambda: &[Gf1024], data_with_checksum_len: usize) -> Option<Vec<usize>>`. Loop `for d in 0..data_with_checksum_len` at **:293** — **no upper bound** on `data_with_checksum_len`. CONFIRMED unbounded.
- `bch_decode.rs:403` `decode_regular_errors` — **ACCURATE.** Live `:403` `pub fn decode_regular_errors(residue_xor_const: u128, data_with_checksum_len: usize)`. Only gate is error-weight `deg == 0 || deg > 4` (:416); **no** `data_with_checksum_len > 93` guard. Docstring (:394–395) even says "in the `0..=93` range" but never enforces it. The position map `k = data_with_checksum_len - 1 - d` (:437) aliases for `len > 93`.
- "Reachable via `md repair` … `parse_chunk_symbols` has no length cap" — **ACCURATE, with refined reachability.** Live call chain: `md-cli/src/cmd/repair.rs:118` → `md_codec::decode_with_correction` (`chunk.rs:502`) → loop calls `parse_chunk_symbols` (`chunk.rs:429`, collects ALL symbols, `Vec::with_capacity(rest.len())`, NO cap) → `decode_regular_errors(residue, symbols.len())` at **`chunk.rs:536`**. So `symbols.len()` flows in uncapped.
  - **Ordering subtlety (important for the SPEC, NOT in the report):** the per-chunk BCH correction (the vulnerable `decode_regular_errors` call) runs in the loop **BEFORE** the `strings.len() == 1` single-string-vs-chunked dispatch (`chunk.rs:603`). So an over-length single string DOES enter the unbounded decoder. **BUT** the loop only calls `decode_regular_errors` when `residue != 0` (`chunk.rs:526` pass-through skips clean strings). Therefore: an H6-produced *clean* over-length md1 (residue==0) does NOT trigger M4 by itself; M4 fires on an over-length md1 *carrying transcription errors* — exactly the report's "independent code change from H6 … a hand-crafted over-length md1 fed to `md repair`."

**STILL-REPRODUCES verdict: REPRODUCES.** `chien_search` is unbounded on origin/main and `decode_regular_errors` has no length gate; `decode_with_correction` passes `symbols.len()` uncapped. A >93-symbol md1 with ≤4 visible errors reaches the aliasing decoder. No later commit added a cap.

**Action for brainstorm-spec:** reject `data_with_checksum_len > 93` at the top of `decode_regular_errors` (and/or `chien_search`); also at the `decode_with_correction` boundary (`chunk.rs` before :536). Defense-in-depth, distinct from H6's encode guard. Cite SHA **`58cc9ec`**, live lines `bch_decode.rs:284/293/403/416/437`, `chunk.rs:429/502/536/603`.

---

### M6 — `combine_shares()` silently reconstructs a WRONG secret from an inconsistent same-id share set
**id:** `w2-ms-slip39-gf256-1`.
**WHAT:** `combine_shares` Lagrange-interpolates over **all** supplied shares (never truncates to exactly k, never verifies extras), and `interpolate_at` checks only that shares agree on hrp/id/threshold/length — **not** that the points lie on one degree-(k−1) polynomial. So shares sharing the same 4-char (20-bit) id with distinct indices but from **different secrets** interpolate to a wrong secret with no error. codex32/BIP-93 K-of-N has no digest share, so nothing else catches it; `dispatch_payload`'s prefix-byte check is only a probabilistic backstop.

**Citations:**
- `shares.rs:186-270` `combine_shares` — **ACCURATE.** Live `crates/ms-codec/src/shares.rs:186` `pub fn combine_shares(shares: &[String]) -> Result<(Tag, Payload)>`. Body reviewed (186–272): validates (1) per-string parse/checksum, (2/C1) reject index-`s` (:235), (3) `parsed.len() < k` → `ThresholdNotPassed` (:246), (4) distinct indices → `RepeatedIndex` (:252–262), then (5) `interpolate_at(&parsed, Fe::S)` over **all** `parsed` (:264) — **no truncation to k, no cross-share consistency check.** CONFIRMED.
- `interpolate_at` "checks only hrp/id/threshold/length" — **ACCURATE.** The step-5 comment (:263) explicitly states it "Surfaces Mismatched{Hrp,Id,Threshold,Length}" — i.e. ONLY header-field agreement, never polynomial membership. (codex32 pinned `=0.1.0`, confirmed `Cargo.toml:13`.)
- `envelope.rs:192-220` `dispatch_payload` — **ACCURATE** (location off by a few lines): live `crates/ms-codec/src/envelope.rs` `dispatch_payload` begins at **:192**, body 193–225. It branches on `data[0]` ∈ {`0x00` RESERVED_PREFIX, `0x02` MNEM_PREFIX} else `ReservedPrefixViolation`, each then `p.validate()?`. This is a probabilistic filter only.
  - **MINOR factual refinement to the report's "~255/256 / ~1/256" framing:** there are **2** accepted prefix bytes (0x00, 0x02) of 256, so a uniformly-random wrong secret's first byte passes the prefix gate ≈ **2/256**, and the length/language `validate()` narrows further. The report's "~1/256 garbage that still parses" is the right order of magnitude but slightly conflates the two prefixes; the SPEC should say "a small constant fraction (≈ prefix-match × length-valid) of wrong secrets still parse — the backstop is NOT a consistency check."
- `Error::InconsistentShareSet` (proposed) does **not** exist — **CONFIRMED.** `crates/ms-codec/src/error.rs` enum `Error` (:19) ends at `SecretShareSuppliedToCombine` (:122); no consistency variant. The fix adds a NEW wire-adjacent variant (public-API surface change → drives SemVer; see cross-cutting).

**Not a duplicate of prior combine_shares cycles.** ms FOLLOWUPS shows `combine_shares` was hardened twice — `combine-no-length-validation-panic` (ms-codec **v0.4.1**, Entr-arm `validate()`) and the uppercase-canonicalization + same-id-secret-`S`-bypass fix (ms-codec **v0.4.2**). Neither added a cross-share polynomial-consistency check; M6 is a distinct, still-open gap. (Obs entry `pr2-exposure-claim-verified-sound` at FOLLOWUPS:15 concerns a different padding-bug exposure, not consistency.)

**STILL-REPRODUCES verdict: REPRODUCES.** On origin/master, `combine_shares` interpolates over all supplied shares with no consistency verification; the only post-combine check (`dispatch_payload`) is probabilistic. Combining a same-id inconsistent set yields a wrong secret silently. No later commit added the check.

**Protocol framing (key — read before the SPEC):** Per BIP-93 (authoritative), codex32 K-of-N **deliberately has no digest share and no cross-secret detection** — recovery uses **exactly k shares** via `ms32_recover`, and the spec gives **no** guidance on inconsistent sets (interpolation silently computes a wrong result). So M6 is **NOT a spec-violation**; it is an *inherited spec gap*. The report's fix (interpolate over exactly k, then verify every remaining supplied share lies on the reconstructed polynomial; reject with `InconsistentShareSet`) is **defense-in-depth beyond the spec**, and is sound. The SPEC must frame it as hardening (not conformance) and document that codex32 K-of-N carries no integrity share.

**Action for brainstorm-spec:** in `combine_shares` (`shares.rs:186`): interpolate over exactly k shares, then for each remaining supplied share assert `interpolate_at(k_set, idx) == supplied`; on mismatch return new `Error::InconsistentShareSet`. Document the codex32 no-digest-share gap. Cite SHA **`6b28918`**, live lines `shares.rs:186/235/246/252/263/264`, `envelope.rs:192`, `error.rs:122`.

---

## Cross-cutting observations

1. **Are H6 + M4 the same bug?** No — **distinct code changes, paired workstream.** H6 is an *encode-side* missing guard (`wrap_payload`/`encode_md1_string` emit over-length); M4 is a *decode-side* missing guard (`chien_search`/`decode_regular_errors` accept over-length). They share the same protocol root (codex32 regular code is 93-symbol-bounded) and the same WS-MD-BCH zone, but each must be fixed independently: a hand-crafted over-length md1 fed to `md repair` bypasses the encoder entirely (M4 needs its own gate even after H6 lands). Fix BOTH; they ride one md-codec release.

2. **M6 is independent of H6/M4** (different repo, different protocol, no shared code path). The only commonality is the cross-cutting bug-hunt theme #4 / Wave-2 theme "partial verification as false assurance" + "inherited codex32 integrity gap."

3. **Toolkit reachability of all three (lockstep impact).** The toolkit is NOT just a passive pin-bumper:
   - **M4:** toolkit `mnemonic repair --md1` calls `md_codec::decode_with_correction` (`repair.rs:856`-region) → inherits M4. After md-codec's length cap, the toolkit's repair gains the guard transitively (pin-bump), but a toolkit-side characterization/round-trip test is warranted.
   - **M6:** toolkit `mnemonic ms-shares combine` calls `ms_codec::combine_shares` directly (`cmd/ms_shares.rs:409`, via `ToolkitError::from`). When ms-codec adds `Error::InconsistentShareSet`, the toolkit's `ToolkitError::from(ms_codec::Error)` conversion MUST map the new variant (else a compile error or an opaque fallthrough). **This is a hard lockstep: ms-codec wire-adjacent change → ms-cli error mapping → toolkit `ToolkitError` mapping + `error.rs` alphabetical-variant insert.**
   - **H6:** toolkit `mnemonic` encode/bundle paths that emit md1 (if any go through `encode_md1_string`) inherit the new `Error`; verify the toolkit's md1-emit call sites map it. (H6's primary surface is the `md` CLI; toolkit exposure is secondary.)

4. **SemVer per crate.**
   - **md-codec:** new public `Error` variant (`PayloadTooLongForSingleString`) + a previously-accepted input now rejected (over-length encode/repair) → **MINOR** (additive variant; behavior tightening on out-of-domain input — not a documented contract). Tag + `cargo publish`.
   - **ms-codec:** new public `Error::InconsistentShareSet` variant + a previously-"successful" combine now errors → **MINOR**. Tag + `cargo publish`.
   - **md-cli / ms-cli:** inherit via the exact-pin bump; each gets a PATCH-or-MINOR release (MINOR if they surface the new error text/exit-code as a user-visible behavior; PATCH if purely inherited). md-cli pins `md-codec =0.37.0` → bump to the new md-codec; ms-cli pins `ms-codec =0.4.4` → bump.
   - **toolkit:** PATCH for the pin consumption + the new-error mapping (no toolkit flag added). Per the release ritual: BOTH READMEs + `fuzz/Cargo.lock` version-site updates; re-run suite + fuzz before tag.

5. **Publish → pin chain (must respect ordering).**
   - WS-MD-BCH: `md-codec brainstorm→R0→plan→R0→TDD→tag→cargo publish` → then md-cli release (consumes) → then **toolkit pin-bump** (consumes via `repair.rs`).
   - WS-MS-CODEC: `ms-codec (M6) tag→cargo publish` → ms-cli release (error mapping) → **toolkit pin-bump** (consumes via `ms_shares.rs` + `ToolkitError` mapping).
   - These are PUBLISHED registry crates; the toolkit pin cannot bump until the codec version is on crates.io.

6. **Lockstep flags.**
   - **Manual mirror (`docs/manual/`):** none of H6/M4/M6 add/remove a CLI **flag** — they add error behaviors/exit-codes. The manual's flag-coverage lint (`docs/manual/tests/lint.sh`) gates flag NAMES, so no lint break. BUT if the SPEC chooses to surface new error TEXT/exit-codes in `md repair` / `md encode` / `ms combine` / `mnemonic ms-shares combine` help or error-reference prose, mirror that under `docs/manual/src/40-cli-reference/` in the same/paired PR. **Verdict: no mandatory manual flag-mirror; optional error-doc mirror.**
   - **GUI schema-mirror:** no flag/subcommand/dropdown-value change → no `mnemonic-gui/src/schema/mnemonic.rs` update required.
   - **FOLLOWUP companions:** none of H6/M4/M6 currently has a FOLLOWUP slug in either codec repo (verified — only the report carries them). When fixed, file/flip per the shipping-commit discipline; if any cross-repo action surfaces (e.g. a shared BCH-domain doc note md↔mk), mirror companion entries in both `design/FOLLOWUPS.md`.

7. **`error.rs` alphabetical-variant ordering** (toolkit convention): the toolkit `ToolkitError` new-variant insert (for the mapped ms-codec error, if a distinct toolkit variant is added) must follow alphabetical-by-variant ordering per CLAUDE.md. ms-codec/md-codec `Error` enums are not under that toolkit-specific rule but should insert tidily.

---

## Recommended brainstorm-session scope

**Grouping: TWO independent codec workstreams, run as ONE cycle-4 umbrella but with separate brainstorm→spec→plan tracks (different repos, no shared code, independent publish chains).**

### Track A — WS-MD-BCH (H6 + M4), md-codec
- **Repo:** `descriptor-mnemonic`, branch off `origin/main` `58cc9ec`.
- **Scope:** encode-side length guard (H6) + decode-side length guard (M4) — pair them; one release.
- **Rough LOC:** ~small. H6: one guard in `wrap_payload`/`encode_md1_string` + new `Error` variant + tests (~40–70 LOC incl. red-first tests). M4: one guard at `decode_regular_errors`/`chien_search` top + `decode_with_correction` boundary + aliasing-regression test (~40–80 LOC). Total ~80–150 LOC.
- **SemVer:** md-codec **MINOR** (→ 0.38.0) tag + publish; md-cli PATCH/MINOR pin-bump.
- **Publish→pin:** md-codec publish → md-cli release → toolkit PATCH pin-bump (consumes via `repair.rs`); add toolkit-side repair round-trip test.
- **Lockstep:** no manual-flag/GUI-schema change; optional error-doc mirror.

### Track B — WS-MS-CODEC (M6), ms-codec
- **Repo:** `mnemonic-secret`, branch off `origin/master` `6b28918`.
- **Scope:** cross-share polynomial-consistency check in `combine_shares` + new `Error::InconsistentShareSet`. (NOT bundled with H4/H5/L5/L26 — those are the broader WS-MS-CODEC ms-cli items; cycle-4's mandate is the funds-critical M6 only. Keep M6 isolated unless the user opts to fold the ms-cli HIGH panics in.)
- **Rough LOC:** ~small-medium. Truncate-to-k + per-extra-share `interpolate_at` membership assertion + new variant + Display/exit-code + same-id-inconsistent-set red-first test (constructed like the existing `encode_shares` fixtures) (~60–110 LOC).
- **SemVer:** ms-codec **MINOR** (→ 0.5.0, new public variant) tag + publish; ms-cli MINOR/PATCH pin-bump (must map the variant) → toolkit PATCH pin-bump (must map in `ToolkitError::from` + `ms_shares.rs`).
- **Publish→pin:** ms-codec publish → ms-cli release → toolkit pin-bump with `ToolkitError` mapping + alphabetical-variant insert.
- **Lockstep:** no manual-flag/GUI-schema change; **hard lockstep on the toolkit error-mapping** (compile-time forced).

### Ordering & parallelism
- Tracks A and B are **fully disjoint** (different repos, no shared files) → can run in parallel (two single-subagent impl tracks), each through its own R0-gated brainstorm→spec→plan→TDD→review→tag→publish→toolkit-pin.
- Within Track A: H6 before M4 in the SPEC narrative (encode emits the over-length artifact M4 then mis-corrects), but both land in one md-codec release; no inter-fix ordering constraint at the code level (each guard is independent).
- The toolkit pin-bumps (A and B) can be a **single toolkit PATCH** that consumes both new codec versions at once, OR two PATCHes; a combined pin-bump is cleaner (one README/fuzz-lock touch, one suite+fuzz run). Sequence the toolkit pin AFTER both codec crates are on crates.io.

### Protocol-fact corrections to carry into the SPECs (do NOT propagate the report prose verbatim)
- H6: state the cap as **data > 80 / codeword > 93 symbols** (BIP-93), NOT the report's "~67 data symbols."
- M6: frame as **defense-in-depth beyond BIP-93** (codex32 K-of-N has no digest share and the spec mandates no consistency check; recovery uses exactly k shares) — NOT a conformance fix. The `dispatch_payload` backstop accepts **2/256** prefix bytes, not ~1/256.

**Gate status:** recon complete, STRICT-GATE PASS for proceeding to brainstorm. All citations verified ACCURATE (modulo the uniform missing `crates/` path prefix + 2 minor location drifts noted); all three findings **REPRODUCE** on current origin; two protocol-fact refinements captured. No blockers.
