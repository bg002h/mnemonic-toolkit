# Word-Card — POST-IMPLEMENTATION whole-diff adversarial review (P0–P6)

- **Scope:** the COMPLETE Word-Card feature on `feat/wc-p6-toolkit` @ `606e0b1e`
  (worktree `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/agent-a9286242d4b087000`).
  `crates/wc-codec/{field,poly,rs,sync,pipeline,raid,regroup,wordmap,pad,lib}.rs`
  + tests; toolkit `word_card_adapter.rs`, `cmd/word_card.rs`, `error.rs`
  (`WordCard` variant), `main.rs`, `tests/cli_word_card.rs`, fuzz targets.
- **Authoritative:** spec `BRAINSTORM_word_card_encoding_2026-06-24.md` (R0-GREEN)
  + plan `IMPLEMENTATION_PLAN_word_card_encoding.md` (R0-GREEN incl. the §4.2
  ledger-OUTSIDE-RS amendment). Per-phase R0s read for context; this review goes
  beyond them with an independent whole-pipeline fuzz + cross-phase constant/contract audit.
- **Reviewer:** opus architect, independent (did NOT re-confirm the per-phase reviews).
- **Date:** 2026-06-25.

---

## VERDICT: **GREEN — ship-ready, 0 Critical / 0 Important** for the *code*.

The full feature is funds-safe, internally consistent across all 7 phases, and
its suites are green. My independent whole-pipeline adversarial fuzz (74k cases:
60k solo mk1+md1, 6k RAID, 8k RAID over-drop) found **0 wrong-payload escapes and
0 panics**. The constants are singular and agree everywhere; the K′ contract is
consistent across `pipeline.rs ⇄ sync.rs ⇄ rs.rs ⇄ raid.rs`; the end-to-end
never-wrong-payload guarantee holds through every seam (adapter, kind/payload_bits,
RS, sync, integrity tag, RAID, truncation).

**One release-seam item (NON-code, tracked, must-do-before-tag)** and a handful of
Minor/Nit items are listed below. None blocks the *correctness* gate. The
release-seam item (`[patch.crates-io]` local paths) is a ship-step, not a code
defect, and is explicitly documented in-tree + in FOLLOWUPS; I classify it
**Important-for-RELEASE** but it does not lower the code-review verdict to RED
because it is intentional, tracked, and the plan already enumerates it as the P6
publish-then-pin step.

---

## Critical

None.

I specifically hunted for a wrong-payload seam — the feature's entire reason to
exist — and found none (see End-to-end safety trace).

---

## Important

None for the code.

**Important-for-RELEASE (tracked ship-step, not a code defect):**

- **I-REL — DEV-ONLY `[patch.crates-io]` local-path override is live on the
  branch.** `Cargo.toml` patches `md-codec`/`mk-codec` to absolute local paths
  (`/scratch/code/shibboleth/{descriptor-mnemonic,mnemonic-key}/...`) because the
  P0 canonical-payload accessors (`canonical_payload_bytes` /
  `from_canonical_payload_bytes`) are on the sibling MAINLINES but NOT yet on
  crates.io. The branch as-is **will not build off this machine** and is not
  taggable. The plan §7 P6 mandates: publish `mk-codec 0.4.1` + `md-codec 0.39.1`
  (PATCH, additive) to crates.io, bump the toolkit pins, and REMOVE the patch
  block before tag. This is correctly documented in-code (the patch block's own
  comment) and in `FOLLOWUPS.md::word-card-encoding-finish-plan-and-implement`.
  **Action: execute the publish-then-pin at the release seam; the post-impl code
  review does not block on it but the tag MUST NOT ship with the local-path patch.**

---

## Minor / Nit

- **M1 — Manual lockstep NOT done (CLAUDE.md mirror-invariant gap, currently a
  SILENT gap, not a build failure).** A new user-facing subcommand `word-card`
  (with flags `--from`, `--decode`, `--decode-plate`, `--parity-words`,
  `--parity-pct`, `--raid`, `--integrity-bits`, `--json`, positional `<WORD>`)
  landed, but `docs/manual/src/40-cli-reference/41-mnemonic.md` was NOT updated and
  `word-card` is NOT in `docs/manual/tests/cli-subcommands.list`. The manual lint
  iterates only that list, so it does **not** fail today — which is exactly why
  this is a silent lockstep gap (the SAME lagging-gate failure mode CLAUDE.md
  warns about for the GUI mirror). Plan §7 P6 + §8 explicitly require the
  `40-cli-reference` mirror. **Action: add `mnemonic word-card` to
  `cli-subcommands.list` + author the chapter in `41-mnemonic.md` (binary-identical
  `--help` block, fixed-seed CLI-output) before tag.** Not a funds/correctness
  issue — a documentation-coverage debt. (Plan-tracked; flag it loudly so it is
  not lost in the release rush.)

- **M2 — GUI `schema_mirror` (downstream `mnemonic-gui`) not in this diff.** The
  toolkit-side `gui-schema` emission is clap-auto-derived (CommandFactory), so
  `word-card` + its flags appear automatically, and the toolkit's own
  `cli_gui_schema.rs` test was correctly bumped 31→32 subcommands (passes). But the
  hand-maintained mirror `mnemonic-gui/src/schema/mnemonic.rs` is a separate repo
  and is NOT updated here. Per CLAUDE.md this is the paired-PR / lagging gate —
  acceptable for a toolkit-only branch, but the paired GUI PR (or the next
  pin-bump backfill) MUST add the `word-card` subcommand schema or the drift gate
  fires later on the cumulative delta. **Action: file/author the paired
  mnemonic-gui schema-mirror update.** (Plan-tracked.)

- **N1 — Asymmetric `--integrity-bits` error message.** The floor (`< 33`) refuses
  in `run_encode` with a friendly "below the 33-bit floor" message (exit 1); the
  ceiling (`> 63`) refuses only deep in `encode_inner` with the generic
  "word-card: invalid encode/decode parameter" (exit 2). Both correctly refuse
  (verified: `--integrity-bits 64` → exit 2, no silent undecodable card;
  `--integrity-bits 63` → success), and `MAX_INTEGRITY_BITS = 63` correctly closes
  the P6-fuzz-found encode-accepted-but-never-decodable footgun. Pure UX polish:
  consider a parallel friendly ceiling check + matching exit code in `run_encode`.
  No safety impact.

- **N2 — `0x805` serves double duty as both `field::MODULUS` and `CRC11_POLY`.**
  Intentional and documented (same primitive poly `x¹¹+x²+1` for the field and the
  header CRC-11). Singular definition each; reused by name. Calling it out only so
  a future reader does not mistake the reuse for an accidental duplicate constant.
  No action.

- **N3 — Checkpoint marker is 3-bit (`0b101`) while ledger/stop markers are 4-bit
  (`0b1110`/`0b1111`).** Verified no aliasing: top-3-bits of a ledger/stop word are
  `111` ≠ the checkpoint's `101`, and the ledger/stop are read positionally at
  known offsets (never content-scanned), while checkpoints are recognized by the
  3-bit top field + positional/mod-8/CRC anchoring. The three word classes never
  collide. No action — documented thoroughly in `sync.rs` / `pipeline.rs`.

---

## Cross-phase consistency (constants + layering)

### Frozen constants — independently grepped; ONE value each, agreeing everywhere

| Constant | Value | Single definition | Status |
|---|---|---|---|
| Field primitive poly | `0x805` (`x¹¹+x²+1`) | `field::MODULUS` | ✓ (reused by name as `CRC11_POLY`) |
| Primitive element α | `0x002` (`x`), ord 2047 | `field::ALPHA` | ✓ primitivity KAT (`α^2047=1, α^23≠1, α^89≠1`) green |
| CRC-5 generator | `0b10_0101` (`x⁵+x²+1`) | `sync::CRC5_POLY` | ✓ |
| Header CRC-11 | `0x805` | `pipeline::CRC11_POLY` | ✓ |
| Checkpoint marker | `0b101` (3b) | `sync::CHECKPOINT_MARKER` | ✓ distinct |
| Ledger marker | `0b1110` (4b) | `pipeline::LEDGER_MARKER` | ✓ distinct |
| Stop-sign marker | `0b1111` (4b) | `pipeline::STOP_MARKER` | ✓ distinct (no collision w/ above) |
| Integrity `t` default | 44 | `DEFAULT_INTEGRITY_BITS` | ✓ |
| Integrity `t` ceiling | 63 (6-bit GEOM field) | `MAX_INTEGRITY_BITS` | ✓ (closes t=64 silent-unrecoverable) |
| Integrity `t` floor | 33 | `MIN_INTEGRITY_BITS` | ✓ |
| Ledger `U` default | 3 | `DEFAULT_U_SLOTS` | ✓ |
| RAID α-exponent | full 5-bit `index-in-array` | `RaidHeaderFields.index` / H1 | ✓ (NEW-I2; r=2 MDS for all n≤32) |
| BIP-39 symbol | 11-bit English index | `wordmap` (bip39 crate SSOT) | ✓ all-2048 round-trip KAT green |
| Stride `b` | `floor(√K+0.5)` DERIVED | `sync::block_stride` (integer isqrt) | ✓ tie-free, not stored |

No constant is defined twice or disagrees between modules.

### Layering / K′ contract — traced one encode + one decode end-to-end

The P4 amendment (ledger + stop-sign OUTSIDE the RS codeword) is implemented
consistently:

- **Encode** (`encode_inner`): `K′ message = [H0] ‖ [H1‖array-id]? ‖ [GEOM 4] ‖
  interleave(payload+tag, checkpoints)`; `parity = rs_parity(K′, m)`; engraved
  stream = `[H0][GEOM 4][ledger 2U][interleave][parity m][stop-sign 2]` (RAID
  splices H1+array-id into the header → `header_words` grows 5→9, geometry still
  closed-form via `header_word_count(has_raid)`).
- **Decode** (`decode` → `rs_decode_and_check`): reads GEOM positionally,
  reconstructs the SAME header words, splices `[header]‖grid‖parity` to rebuild the
  RS codeword, shifts interleave-erasure indices by `header_offset`. The ledger
  region (`2U`) is SKIPPED between header and interleave (`interleave_start =
  header_words + 2U`). Offsets/region-bounding agree with encode.
- **Append-only** holds: filling a blank ledger slot or appending parity never
  mutates K′ (ledger is outside K′; parity is the appendable RS prefix). RAID's
  `has-raid` header grows |header| by exactly 4 (H1 2 + array-id 2), geometry
  remains closed-form. The RS evaluation-form prefix-extensibility (`β_j = α^j`
  fixed sequence) makes the first m parity words identical at any tier — verified
  by the `append_only_prefix` KAT and the P5 P₁-invariance KAT.

The `header_words` / `payload_offset` expressions in the code match the plan §4.2
amended "engraved-stream offsets" (NOT the RS message) — consistent.

---

## End-to-end safety trace (independent whole-pipeline fuzz)

**The funds-safety charter:** no corrupted/crafted card may yield `Ok(payload)`
with `payload != original`, on EITHER the solo path or the RAID path. A single
escape is Critical.

I wrote two throwaway adversarial harnesses (deterministic xorshift PRNG, deleted
after the run — branch left clean) at the library level:

1. **Solo (mk1 + md1), 60,000 cases.** Random payloads 8–120 B; `kind` random;
   md1 shaves 0–7 trailing bits (bit-precise path); `t ∈ [33,63]`; `m ∈ [0,23]`.
   Corruption patterns: k-substitutions (k up to m+4), single deletion, single
   insertion, contiguous run (1–8), and clean. Compared the decoder's recovered
   `(payload-projected-to-payload_bits, payload_bits, kind)` to the truth.
   **Result: exact=35,067, refused=24,933, WRONG=0, panics=0.** Refusals occur
   exactly when corruption exceeds the RS budget / integrity tag rejects a
   miscorrection — the correct custody behavior.

2. **RAID, 6,000 cases.** n ∈ 2..7, r ∈ {1,2}, random per-plate payloads, random
   drop of 0..r data plates. **Result: ok=5,495, refused=0, WRONG=0, panics=0**
   (every ≤r-drop reconstructed exactly).

3. **RAID over-drop + survivor-corruption, 8,000 cases.** Dropped MORE than r
   plates (r+1 .. r+2) and/or wrecked a survivor beyond budget — the
   underdetermined / unrecoverable cases. **Result: refused=8,000, ok=0, WRONG=0,
   panics=0.** The MDS solve refuses (`RaidUnrecoverable` / per-plate
   `IntegrityMismatch`) rather than fabricating an xpub.

**Seam-by-seam reasoning (corroborating the fuzz):**
- *Integrity tag* (`rs_decode_and_check` step 5): recompute SHA-256 over the
  recovered canonical payload, require equality with the stored t-bit tag.
  Non-linear, OUT of the RS linear image (in-codeword but a SHA, per C1/NEW-C1), so
  an RS miscorrection-onto-a-valid-but-wrong-codeword survives only at ≤2⁻ᵗ. The
  tag is computed over the CANONICAL payload (trailing sub-byte bits zeroed) on
  BOTH sides — so a clean md1 round-trip never false-rejects.
- *Single-deletion candidate path* (`finish_decode`): each candidate is validated
  by the GLOBAL RS+tag oracle; two candidates yielding DIFFERENT tag-passing
  payloads ⇒ `IntegrityMismatch` refuse (genuine ambiguity); same-payload
  candidates are not an ambiguity. Correct.
- *RS beyond budget*: `rs_decode` refuses (`Uncorrectable`) when `2t+s > m` or the
  punctured system is underdetermined; never panics.
- *Crafted-header decode* (hostile `payload_bits` up to 65535, CRC-valid → K up to
  ~5964 > field cap 2047): the decoder slices the interleave region from the
  ACTUAL (bounded) word stream, not from K; `parity_start`/`interleave_start`
  bound-checks + `rs_decode`'s `n > MAX_N → LengthExceedsField` provide layered
  refusal. No over-allocation, no panic (covered by fuzz's GEOM-corrupting cases).
- *Adapter inverse*: `canonical_to_recovered` rebuilds the KeyCard/Descriptor from
  the recovered `(bytes, payload_bits)`; mk1 re-encode is non-deterministic
  (`chunk_set_id`) so the xpub identity is the assertion target, md1 is
  deterministic. `payload_bits` is carried verbatim (load-bearing for md1) and is
  never replaced by `bytes.len()*8`. ms1 refused via `UnknownHrp`.

**No wrong-payload seam was found.**

---

## API coherence + error mapping

- Public `wc-codec` surface (`encode`/`decode`/`raid_encode`/`raid_reconstruct`,
  `SourceKind`/`EncodeOpts`/`Decoded`/`RepairSummary`/`RaidMeta`/`WcError`/
  `PlateRole`/`RaidPlate`/`RaidRecovery`) is complete and consistent. `WcError`
  variants are alphabetical (HeaderCrcMismatch < IntegrityMismatch < InvalidParams
  < RaidArrayMismatch < RaidUnrecoverable < Regroup < Rs < Sync < Truncated <
  Uncorrectable < UnknownWord). `RegroupError` / `RsError` / `SyncError` variants
  alphabetical (the deferred P1-N1/P2-N1 reorders folded at P4).
- `WcError → ToolkitError::WordCard → exit-code 2` is sound and consistent across
  `error.rs` exit_code/kind/Display + the `From<wc_codec::WcError>` impl. The
  `WordCard` variant is alphabetically placed (after VerifyMessage, before
  XpubSearch) in ALL THREE match blocks + the enum. Exit 2 = the
  format/structural-refusal class, correct for every WcError incl. the funds-safety
  nets (IntegrityMismatch/Uncorrectable/RaidUnrecoverable refuse non-zero, never
  return a payload).
- **No reachable `panic!`/`unwrap`/`expect`/`unreachable!`/`todo!` on any
  decode/CLI path.** The `unwrap`/`expect` sites are all guarded:
  - `parse_header` BitReader unwraps (388–391, 420–448): each read is on a slice
    whose length is pre-checked (`words.is_empty()` guard, `words.len() <
    header_words` guard, RAID `words[1..5]` guaranteed by has_raid⇒len≥9), and the
    bit-widths exactly fit the slice. Never None.
  - `poly.rs` 118/119/190/203: divisor/leading/distinct-node invariants held by the
    only callers (partial-GCD with guarded non-zero divisor; interpolation over the
    distinct β_j). `divmod` debug-asserts (not panics in release) non-zero divisor.
  - `raid.rs:140` `pack_stripe` expect: `symbols_to_bits` over exactly `11·W` bits
    of 11-bit symbols cannot fail (no pad to assert) — a true internal invariant.
  - `pad.rs` assert: documented internal invariant (target ≥ input; the array-wide
    max is ≥ every member). Not decode-reachable.
  Confirmed empirically: 74k fuzz cases + the committed `decode_never_panics` /
  `roundtrip` fuzz targets, 0 panics.

---

## md1/mk1 adapter correctness across the board

Verified by `word_card_adapter.rs` unit tests + `cli_word_card.rs` integration +
my fuzz:
- multi-chunk mk1 (2-chunk fixture) → canonical → wc → back: xpub identity exact;
  literal string intentionally NOT asserted (fresh `chunk_set_id`).
- md1 (3-chunk fixture, embeds the descriptor) → canonical → wc → back: descriptor
  equal AND literal string deterministic (`md_codec::split` re-emit identical).
- keyless-template md1 / wallet-policy md1 with embedded xpubs ride the same
  bit-precise `total_bits` path; `total_bits` is carried verbatim (never
  `bytes.len()*8`) — the load-bearing asymmetry is handled correctly.
- mk1 with/without origin fingerprint: RAID array-id seed uses the 4-byte
  fingerprint, or 4 zero bytes for a privacy-mode card (deterministic seed length).
- ms1 refused (`UnknownHrp`, exit 2) — entropy is a SECRET, intentionally not
  word-card-able. `string_to_canonical`/`chunks_to_canonical` route by HRP.
- `total_bits` survives the full round-trip in every md1 case (fuzz asserted
  `decoded.payload_bits == payload_bits` over 30k md1 cases).

---

## Suite results

- **`cargo test -p wc-codec`: GREEN.** field 10, pad 5, pipeline 24, raid 13,
  regroup 8, rs 12, sync 23, wordmap 5 (+ lib 0, doc 0) = **100 tests, 0 failed**.
- **`cargo test -p mnemonic-toolkit`: GREEN.** All test binaries pass, 0 failures
  (incl. `cli_word_card` 17, `cli_gui_schema` 16 with the 31→32 subcommand bump,
  `lint_argv_secret_flags` with the new `word-card --from` route, the
  `word_card_adapter` unit tests). 4 ignored are the pre-existing env-gated
  bitcoind/network tests, unrelated.
- **`cargo clippy -p wc-codec`: clean** (no warnings/errors; only the pre-existing
  md-codec `at_derivation_index` deprecation, not from this feature).
- **`cargo build -p wc-codec -p mnemonic-toolkit`: clean.**
- **Hygiene:** `mlock.rs` is **byte-identical to master** (no `cargo fmt --all`
  anywhere). `.gitignore` additions sane (wc-codec fuzz transient output ignored,
  Cargo.toml/lock/targets committed). The fuzz crate is correctly NOT a workspace
  member (own `rust-toolchain.toml`). Branch left clean after the review (temp fuzz
  files removed; `git status` shows only the pre-existing untracked design files).

---

## Spec-promise coverage (delivered vs deferred)

| Spec promise | Delivered? | Where |
|---|---|---|
| §9(a) value layer MDS `2t+s≤m` | ✓ | `rs.rs` Gao + erasure puncturing; KATs |
| §9(b) indel layer sync-bounded (detect always-on; 1/word located else ≤b) | ✓ | `sync.rs` trichotomy + whole-block-erasure |
| §9 custody: bounded-distance miscorrection CAUGHT by integrity tag ≤2⁻ᵗ | ✓ | `rs_decode_and_check` step 5; >86k forced-miscorrect fuzz (P4) + my 74k |
| §6.2 append-only systematic eval-form RS (prefix-extensible) | ✓ | `rs.rs` `β_j=α^j`; `append_only_prefix` KAT; P5 P₁-invariance |
| §6.3 stop-sign ≥2 words + front recorded-length ledger; truncation flag | ✓ | `pipeline.rs` ledger/stop-sign; near-2047 truncation KAT |
| §6.1 checkpoint self-id marker + bounded realignment; refuse-on-ambiguity | ✓ | `sync.rs` recognition + AmbiguousRealignment refuse |
| §6.1 indel trichotomy + deleted-checkpoint + compound case | ✓ | `sync.rs`; compound KAT green |
| §7 RAID r=1/r=2 MDS, lose any r of n+r; P₁ append-only; privacy | ✓ | `raid.rs`; n=15/n=32 r=2 every-pair KAT; lone-parity privacy KAT |
| §7.1 canonical fixed-width per-xpub stripe (self-describing length) | ✓ | `raid.rs` len-prefix + array-wide W padding |
| §5.3 NON-LINEAR in-codeword integrity tag (linear forbidden) | ✓ | SHA-256[0..t]; the no-silent-miscorrect oracle |
| §1.1 ms1 OUT of scope | ✓ | adapter refuses `UnknownHrp` |
| §10 deterministic encode (binary-identical docs) | partial | encoder deterministic; **doc CLI-output blocks not yet authored (M1)** |
| §7.4 conditional RAID-vs-md1 suppression | **deferred (nice-to-have)** | not implemented; spec made it explicit opt-in only, NOT funds-relevant |
| §10 display grouping / `@slot` source | **deferred (nice-to-have)** | `--group-size`/`@slot` not wired; cosmetic/source-routing, not funds-relevant |
| §10 GUI schema-mirror + manual lockstep | **NOT done (M1/M2)** | toolkit gui-schema auto-derives ✓; downstream GUI mirror + manual chapter pending |

**Distinguishing the gaps:** the unimplemented items (§7.4 conditional suppression,
display-grouping, `@slot`) are nice-to-haves the spec/plan already flagged as
non-default or deferred — **none is a broken funds-safety guarantee**. The two that
need pre-tag attention are the **lockstep docs** (M1 manual, M2 GUI mirror) and the
**release-seam publish** (I-REL) — all plan-tracked, all loud, none silent in the
code itself.

---

## Bottom line

The Word-Card value engine and toolkit integration are **correct, funds-safe, and
internally consistent across all 7 phases.** The never-wrong-payload guarantee —
the whole feature's reason to exist — holds under 74k independent adversarial
cases with 0 escapes and 0 panics, corroborated by seam-by-seam reasoning. The code
gate is **GREEN (0C/0I).**

Before tagging, the orchestrator MUST complete the three plan-tracked, non-code
release steps: (I-REL) publish `mk-codec 0.4.1`/`md-codec 0.39.1` + pin-bump +
remove the local-path patch; (M1) author the `mnemonic word-card` manual chapter +
add it to `cli-subcommands.list`; (M2) the paired `mnemonic-gui` schema-mirror
update. These are ship-steps, not code defects, and do not lower the code-review
verdict.
