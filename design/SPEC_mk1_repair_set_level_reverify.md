# SPEC — mk1-repair-set-level-reverify (F4 Cycle 1)

**When the FULL mk1 card set is present, RE-VERIFY a BCH substitution-correction by reassembling through
`mk_codec::decode` (cross-chunk SHA-256 hash) before declaring success — so a >4-error miscorrection (a chunk
aliased onto a DIFFERENT valid codeword) is REJECTED, not blessed as recovery of a different wallet. When only
a PARTIAL set (a single plate) is supplied, per-plate repair is PRESERVED (it is the documented workflow) but
returned as an UNVERIFIED candidate with an advisory. Plus a harness that MEASURES + pins the real
miscorrection rate and a DETERMINISTIC pinned-seed proof that the re-verify catches a known misfit.**

- **Author:** (this session). **Source SHAs (grep-verified):** toolkit `9866acc7`; mk-cli/mk-codec `mnemonic-key main@85bca69`; md-codec `descriptor-mnemonic main@ef1f3e71`.
- **Finding:** constellation-eval **F4** (`design/agent-reports/constellation-eval-2026-07-06.md:101-124`). Recon `cycle-prep-recon-f4-bch-repair-miscorrection.md`. User scope (2026-07-07): mk1 FIRST + measure/pin the rate; ms1-advisory = separate Cycle F.
- **Target:** `mnemonic-toolkit` MINOR (`v0.80.0`) + `mk-cli` MINOR. **mk-codec / md-codec / ms-codec NO-BUMP** (existing public API reused). Manual lockstep + mk-cli sibling-pin advance (§6).
- **Status:** ✅ **R0-GREEN (0C/0I) @ round 3** — rev-3 folded R0-round-1 (1C/3I/4M) + round-2 (0C/1I/2M) + round-3 editorial (M-r3-1 dup paragraph, M-r3-2 §7.2 wording). Reviews `cycleE-spec-r0-round-{1,2,3}.md`. **CLEARED for the IMPLEMENTATION_PLAN + plan-R0.**

## §0 — Scope

**IN:**
1. **mk1 set-level re-verify (tri-state) at BOTH mk1-repair sites** — toolkit `src/repair.rs` `repair_card`
   `CardKind::Mk1` (`:766-783`) AND `mk repair` CLI (`mnemonic-key/crates/mk-cli/src/cmd/repair.rs:63-90`).
   `repair_card` is shared by toolkit auto-repair, the standalone `mnemonic repair` subcommand
   (`src/cmd/repair.rs:143-144`), AND (mirrored) `mk repair` — all must behave per §2/§3.
2. **PRESERVE per-plate (partial-set) repair** — mk1 chunks self-heal independently; `mk repair <one plate>` /
   `mnemonic repair --mk1 <one plate>` is a documented terminal workflow (manual `44-mk-cli.md:247`, a live
   verify-examples golden) and MUST keep working. (R0-round-1 C1 — the funds fix must NOT regress this.)
3. **Measurement harness + deterministic catch-proof** (user-requested; §4).
4. **md1 regression-lock** — md1 is ALREADY protected (`md_codec::chunk::reassemble`'s content-derived 20-bit
   `chunk_set_id` check runs unconditionally for all counts, `chunk.rs:379-387`); add a test + doc note (no
   structural change).
5. **Manual caveat** — `mk repair` / `mnemonic repair` chapters + auto-fire sections: a >4-error correction may
   alias to a different valid card; full-set repair now rejects an mk1 miscorrection; a single-plate correction
   is UNVERIFIED until the full card is reassembled; BIP-93 recommends confirming a corrected codex32 string.

**OUT:** ms1 single-string demotion (separate Cycle F — design-heavy, no payload oracle); any codec wire-format/
API change; widening md1's 20-bit id; re-citing the eval's unverified `~2⁻¹³·⁹` as established fact.

## §1 — Problem (recon + BIP-93 verified; count=1 crux resolved)

codex32 BCH corrects ≤4 substitutions (BIP-93). Beyond t=4, bounded-distance decoding can land a chunk within
the radius-4 ball of a DIFFERENT valid codeword (a miscorrection). The defensive `bch_verify_*` (`mk-codec
bch.rs:442/495`) / toolkit `polymod_residue != 0` (`repair.rs:733`) are CODEWORD-MEMBERSHIP tests — they pass
for a wrong-fit alias. `mk repair` + the toolkit mk1 auto-repair arm return success after only per-string
correction, skipping the cross-chunk SHA-256 hash `mk decode` enforces (`mk_codec::string_layer::
reassemble_from_chunks` → `Error::CrossChunkHashMismatch`, `chunk.rs:189-201`). So a wrong-fit FULL-set mk1
repair is blessed as recovery of a different wallet's card. Toolkit auto-repair fires by default under a TTY
(`resolve_no_auto_repair` = `no_auto_repair || !tty`) on convert/inspect/verify-bundle/xpub intake. (BIP-93
normative: "implementations SHOULD NOT automatically proceed with a corrected codex32 string without user
confirmation.")

**count=1 reachability (R0-round-1 I1 — resolved favorably):** min real mk1 bytecode ~80 B (a compact xpub is
73 B; `pipeline.rs:161-168`) and `CHUNKED_FRAGMENT_LONG_BYTES=53` / `SINGLE_STRING_LONG_BYTES=56`
(`consts.rs:33,39`) ⇒ **every real mk1 card is ≥2 chunks.** A single `Chunked` chunk (≤49 B) and a
`SingleString` mk1 (≤56 B) are BOTH encoder-unreachable. A single `Chunked` chunk carries + verifies the 4-byte
SHA unconditionally (`chunk.rs:66-70,189-201`) → strictly stronger; `SingleString` has no hash
(`pipeline.rs:136-143`, ms1-class) but is unreachable and the fix is no-weaker there. **So the fix is
no-weaker in every case, strictly stronger for all reachable cards.**

## §2 — Fix mechanism (TRI-STATE; mirror the indel oracle only for the full-set arm)

The INDEL path already reassembles: `Mk1IndelOracle::validate` (`src/repair.rs:1040-1056`) calls
`mk_codec::decode(&refs)` and accepts only on `Ok` — BUT it is always built with the FULL set (`all_chunks`,
`:1178-1181`), a guarantee the substitution path does NOT have (it loops over whatever the user passed).
Therefore the substitution fix is a **tri-state on (supplied chunk count vs header `total_chunks`)**, NOT a
binary bless/reject:

**The INVARIANT (R0-round-2 I-r2-1 — BLESS iff decode Ok, NOT a named-variant allowlist):** a **confident
BLESS** (mk repair exit 5 / toolkit short-circuits with the repaired card, chunks presented as RECOVERED with
NO unverified caveat) occurs **IFF `mk_codec::decode` on the EXACT supplied corrected set returns `Ok`.** Any
`Err` is NEVER a silent bless. Classify **per `chunk_set_id` GROUP** (parse `total_chunks`/`chunk_set_id` up
front via the public API — `DecodedString::data()` (`bch.rs:604`) → `StringLayerHeader::from_5bit_symbols`
(`header.rs:120`) → public `Chunked{chunk_set_id,total_chunks,chunk_index}` fields (`header.rs:45-53`); no new
codec API → **NO-BUMP**):
1. **`mk_codec::decode(&group_refs)` == `Ok`** → **BLESS** that group (exit 5). Unchanged from today.
2. **Group is complete-and-consistent** (R0-round-2 M-r2-1: indices `0..total_chunks-1` each present exactly
   once, consistent `total_chunks`/`chunk_set_id`) **AND `decode` == `Err` (ANY error** — `CrossChunkHashMismatch`,
   `ChunkSetIdMismatch`, a header-region `ChunkedHeaderMalformed`/`MixedHeaderTypes`, OR a structural
   `decode_bytecode` failure from a hash-colliding miscorrection) → the per-string corrections aliased to a
   wrong-fit → **REJECT** (mk repair → exit 2 via `CliError::Codec`; toolkit auto-repair does NOT short-circuit
   — the caller's original error surfaces). **The funds fix — reject on ANY `Err`, NOT a variant allowlist**
   (the variant informs only the user message).
3. **Group is INCOMPLETE** (supplied `<` `total_chunks`, or gaps/dupes → not complete-and-consistent) → cannot
   set-verify → **UNVERIFIED-CANDIDATE**: emit the corrected chunk(s) with a LOUD advisory ("correction
   UNVERIFIED — a >4-error correction can alias to a different card; reassemble the full card (`mk decode` /
   import the full set) to confirm; BIP-93 recommends confirmation"). `mk repair` keeps **exit 5** (preserves
   the documented per-plate example); `mnemonic repair` maps to the existing **exit-4 VERIFY-ME candidate**
   (`indel_exit_code`, `repair.rs:1118-1136`). Do NOT map an incomplete group to exit 2.

**Multi-group / batch aggregation (R0-round-2 I-r2-1b):** `mk repair` + `mnemonic repair --mk1` accept a BATCH
of strings (`read_mk1_strings` flat-collects; `resolve_groups` → one Vec; 44-mk-cli.md:226). Apply rules 1-3
**per `chunk_set_id` group**; the invocation exit is the **DOMINANT** outcome across groups —
**reject > candidate > bless > clean** — and a rejected group's chunks are **NOT** presented as recovered. A
full-set miscorrection in ANY group must never ship under a batch success exit.

**Header-corruption note:** a substitution corrupting `total_chunks` itself misclassifies the group as
incomplete → downgrades to UNVERIFIED-CANDIDATE-with-advisory, NEVER a clean confident success (bounded — the
user re-verifies at reassembly). Discriminate group-completeness from the parsed indices vs `total_chunks`, NOT
from the overloaded error string (`ChunkedHeaderMalformed` covers both "incomplete" and "broken header").

**Residual partial-set exposure is bounded:** a miscorrected single plate is still caught by the cross-chunk
hash at eventual full reassembly (`mk decode` / toolkit full-set intake); the advisory covers the
engrave-before-reassemble gap. The auto-repair convert/inspect path is unaffected by the partial case (a
partial card cannot convert anyway → the original error surfaces, already correct).

## §3 — Exit-code semantics
- **mk repair:** full-set miscorrection → **exit 2** (name it explicitly; `CrossChunkHashMismatch` →
  `CliError::Codec` → 2, `mk-cli repair.rs:10-11`, `44-mk-cli.md:236`). Full-set clean-correct → exit 5.
  Partial-set correct → exit 5 + advisory (unchanged exit). Clean → exit 0. No new mk-cli exit code.
- **toolkit `mnemonic repair`:** full-set miscorrection → not-repaired (surfaces the decode error); partial-set
  correct → exit-4 VERIFY-ME candidate + advisory (reuse `indel_exit_code`). Auto-repair (convert/inspect/
  verify-bundle/xpub): on full-set miscorrection, `try_repair_and_short_circuit` does NOT short-circuit.
- **Batch (multi-group) exit:** the DOMINANT outcome across all `chunk_set_id` groups (reject > candidate >
  bless > clean; §2). A batch containing one full-set miscorrection group exits reject even if other groups
  bless.
- This is a breaking exit-code behavior change (a previously-exit-5 full-set wrong-fit now exits 2) → MINOR.

## §4 — Test / oracle matrix (TDD-first)
1. **(FUNDS ANCHOR, DETERMINISTIC — R0-round-1 I3) full-set miscorrection REJECTED via a PINNED seed** — a
   known 5-substitution corruption of a real ≥2-chunk mk1 card that `bch_correct` aliases to a valid-but-wrong
   codeword, found ONCE by a bounded search and **pinned as a test constant** (NOT re-searched per run).
   Assert `mk repair <full set>` → exit 2 (any decode `Err` per §2) and toolkit auto-repair does NOT
   short-circuit. This is the non-vacuous proof the re-verify catches a real misfit. **Maintainability
   (R0-round-2 M-r2-2):** the pinned seed can be invalidated by a future mk-codec BCH change (the corruption
   may no longer alias) — the test must, on such a change, fail with an EXPLICIT message ("the pinned F4
   miscorrection seed no longer aliases to a wrong codeword — re-pin via the bounded search in <helper>"), NOT
   a cryptic assertion.
2. **(REGRESSION — R0-round-1 C1) partial-set per-plate repair STILL succeeds** — replay the manual example:
   `mk repair <single chunk of a 2-chunk card>` → exit 5 + the advisory; `mnemonic repair --mk1 <one plate>` →
   exit-4 candidate + advisory. Confirms the fix does NOT break the documented single-plate workflow.
3. **Genuine ≤4-error FULL-set correction still blesses** — exit 5 + toolkit applies (reassembles).
4. **Clean card** — exit 0 / no auto-repair.
5. **toolkit convert/inspect auto-repair** on a full-set wrong-fit mk1 (the §4.1 seed) no longer silently emits
   the wrong card.
5b. **(BATCH — R0-round-2 I-r2-1b) multi-group reject dominates** — a single `mk repair` / `mnemonic repair
   --mk1` invocation with TWO `chunk_set_id` groups {one full-set miscorrection (the §4.1 seed), one clean or
   partial group} exits **reject** (mk repair exit 2) and does NOT emit the miscorrected group's chunks as
   recovered. Pins the aggregation (a batch success must never carry a miscorrection).
6. **md1 regression-lock (R0-round-1 M3)** — an md1 wrong-fit correction is already rejected by the content-id
   check (assert reject); AND assert md1 has NO non-chunked decode path bypassing `reassemble` (mirror the
   SingleString reachability note).
7. **Reachability lock (R0-round-1 I1)** — assert the minimum-size real mk1 card produces ≥2 chunks and that
   `SingleString` mk1 is not encoder-emitted (so a future encoder change that made it reachable trips this).
8. **(HARNESS — rate) measure + pin** — seeded `StdRng` (NO `thread_rng`), fixed sample size N: random mk1
   payload → encode → inject exactly 5 substitutions in the regular-code trailing chunk (BCH(93,80,8),
   `pipeline.rs:281-299`) → `bch_correct`; record the alias-to-valid-≠-original rate. Pin a **Clopper-Pearson
   UPPER confidence bound** (not the point estimate — avoids resampling flake). **N-sizing (R0-round-2
   M-r2-2):** either size N so `E[hits] ≫ 1` at the expected ~10⁻⁴-10⁻⁵ rate (e.g. N≈10⁶ → ~10-100 hits) so an
   `observed-≥1` assertion is robust, OR keep N modest and make `observed-≥1` a SOFT warning (the HARD funds
   proof is §4.1's pinned seed, so §4.8 need not gate on observing a hit). If N≈10⁶ is too slow for the default
   suite, gate it behind `--ignored`/an env flag and run it in CI. Cite the MEASURED bound in the
   CHANGELOG/manual, NOT the eval's `2⁻¹³·⁹`.
9. Determinism; full `cargo test -p` green in each touched repo (toolkit + mk-cli).

## §5 — Cross-source anchors (recon + R0-round-1 verified)
- **`mnemonic-key/crates/mk-cli/src/cmd/repair.rs:63-90`** (loop, exit 5, no reassembly; exit codes 0/5/2/1, NO exit-4 in mk-cli) — the mk-cli fix site (R0-round-1 M2: full `crates/` path).
- mk-codec: `string_layer/chunk.rs:109` `reassemble_from_chunks`; **`:131-136` incomplete-set reject `ChunkedHeaderMalformed` BEFORE the hash** (the C1 crux — verified by running `mk decode` on the manual example); hash `:189-201`; `mk_codec::decode(strings:&[&str])->Result<KeyCard>` (`key_card.rs:158`); `error.rs:19-98` public `#[non_exhaustive]` variants; `consts.rs:33,39` chunk sizes; `bch.rs:442/495`.
- toolkit `src/repair.rs`: Mk1 arm `:766-783`; `repair_chunk_one` `:731-739`; `Mk1IndelOracle::validate` `:1040-1056` (full-set only, `:1178-1181`); `resolve_no_auto_repair` `:411-418`; `indel_exit_code` `:1118-1136`; `mnemonic repair` `cmd/repair.rs:143-144`.
- md1: `md-codec/src/chunk.rs:379-387` (content-id, all counts).
- Manual: `docs/manual/src/40-cli-reference/{41-mnemonic.md:2990-3037, :739-751; 44-mk-cli.md:247/253 (example+golden)}`.

## §6 — Cross-repo coordination + release
- **Changes:** mk-cli (`crates/mk-cli/src/cmd/repair.rs` tri-state) + toolkit (Mk1 arm tri-state + harness + md1/reachability tests + manual) + manual. **mk-codec / md-codec / ms-codec NO source change** (existing public API reused; discrimination on existing public Error variants — R0-round-1 M1 defends NO-BUMP).
- **SemVer:** mk-cli MINOR; toolkit MINOR (`v0.80.0`); codecs NO-BUMP.
- **Sibling-pin decision (R0-round-1 I2 — RESOLVED = advance):** because we ship a NEW mk-cli with the funds
  fix, `curl|sh` users should get it → **advance the mk-cli sibling pin, sequenced AFTER the mk-cli release
  tag** (this is the ritual-consistent case: a pin advance WITH a matching release, unlike the frozen-baseline
  no-release case a prior cycle reverted). `sibling-pin-check` enforces consistency across the WHOLE scan set,
  so advance ALL FIVE mk-cli references in lockstep: `scripts/install.sh:41`, `.github/workflows/
  {manual.yml:79, quickstart.yml:77, technical-manual.yml:109}`, `docs/manual/src/40-cli-reference/
  44-mk-cli.md:12`. The toolkit SELF-pin (`install.sh:32`, v0.79.0→v0.80.0; gated by `install-pin-check`) bumps
  regardless. (Note: the toolkit's own mk1 auto-repair arm uses `mk_codec` the LIBRARY (unchanged), so it is
  NOT build-coupled to the mk-cli binary — the pin advance is purely so curl|sh delivers the fixed `mk repair`.)
- **Release order:** mk-cli release (tag + crates.io publish) → toolkit (advance 5 mk-cli pin refs + self-bump
  + tag). Verify `sibling-pin-check` + `install-pin-check` green.
- **Manual lockstep:** mandatory (exit-code semantics) — `41-mnemonic.md` + `44-mk-cli.md` repair chapters +
  the `44-mk-repair-text.out` golden if the partial-set example output changes (it should NOT — partial repair
  keeps exit 5; confirm the golden is unaffected or regenerate). **GUI/schema_mirror:** no-op unless the
  tri-state adds a CLI flag (it should not) — verify at plan time (R0-round-1 M4).

## §7 — Risks / R0 focus
1. **Partial-set per-plate repair preserved** (C1) — §4.2 pins it at both mk repair + mnemonic repair; the
   tri-state discriminates on supplied-count vs total_chunks, NOT on the overloaded error string.
2. **No false-reject of a genuine FULL-set ≤4 correction** (§4.3) — the re-verify rejects on any full-set
   decode failure; a genuine ≤4 correction yields `mk_codec::decode == Ok` and is blessed, never rejected.
3. **Non-vacuous funds proof** (I3) — §4.1 pinned known-miscorrection seed proves the catch; §4.8 measures the
   rate with a confidence bound + observed-≥1 self-check.
4. **count=1 reachability** (I1) — §4.7 locks min-≥2-chunks + SingleString-unreachable.
5. **Cross-repo release sequencing** (I2) — mk-cli release before the 5-ref pin advance; both gates green.
6. **Exit-code contract** — the `44-mk-repair-text.out` golden must stay valid (partial repair keeps exit 5).

---

*R0 gate: converge to 0C/0I via the opus-architect loop (persisted to `design/agent-reports/`) BEFORE any
implementation, per CLAUDE.md.*
