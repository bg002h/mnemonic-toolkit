# SPEC — mk1-repair-set-level-reverify (F4 Cycle 1)

**After a per-string BCH substitution-correction of an mk1 card set, RE-VERIFY the corrected set by
reassembling it through `mk_codec::decode` (cross-chunk SHA-256 hash) before declaring the repair a success —
so a bounded-distance decoder that aliased a >4-error chunk onto a DIFFERENT valid codeword (a "miscorrection")
is caught, not blessed. Plus a seeded harness that MEASURES + PINS the real just-past-threshold miscorrection
rate.**

- **Author:** (this session) — single-author design per CLAUDE.md phase-2 convention.
- **Source SHAs (grep-verified via recon):** toolkit `9866acc7`; mk-codec/mk-cli `mnemonic-key main@85bca69`; md-codec `descriptor-mnemonic main@ef1f3e71`.
- **Finding:** constellation-eval **F4** (`design/agent-reports/constellation-eval-2026-07-06.md:101-124`). Recon: `cycle-prep-recon-f4-bch-repair-miscorrection.md` (2026-07-07). **User scope decision (2026-07-07): mk1 FIRST (mechanical); measure + pin the rate. ms1-advisory = separate Cycle F.**
- **Target releases:** `mnemonic-toolkit` MINOR (`v0.80.0`) + `mk-cli` MINOR. **mk-codec / md-codec / ms-codec NO-BUMP** (mk-codec's `decode`/`reassemble_from_chunks` reused as-is; recon blast-radius map). Manual lockstep.
- **Status:** DRAFT — pending opus-architect R0 loop to 0C/0I before any implementation (CLAUDE.md Conventions bullet 1).

## §0 — Scope

**IN:**
1. **mk1 set-level re-verify at BOTH mk1-repair sites** (the funds fix):
   - toolkit auto-repair mk1 arm — `src/repair.rs` `repair_card` `CardKind::Mk1` branch (`:766-783`; loops `repair_chunk_one` building `corrected_chunks`, NO reassembly today).
   - `mk repair` CLI — `mnemonic-key/mk-cli/src/cmd/repair.rs:63-90` (per-string `decode_string` loop, returns exit 5 on any correction, NO `reassemble_from_chunks`).
2. **Measurement harness** (user-requested): a seeded, reproducible property/Monte-Carlo test that MEASURES the real just-past-threshold (5-substitution) mk1-regular-code miscorrection rate and PINS it below a bound, AND asserts the new re-verify CATCHES a miscorrection. Home in toolkit `tests/` (eval's proposed `prop_repair_never_wrong.rs`). Do NOT cite the eval's un-reproduced `~2⁻¹³·⁹`/`1800×` — cite the value THIS harness measures.
3. **md1 regression-lock** (not a structural fix — recon §Cross-cutting 1: md1 is ALREADY protected by `md_codec::chunk::reassemble`'s 20-bit content-derived `chunk_set_id` check, run unconditionally incl. single-chunk): add a test asserting an md1 wrong-fit correction is already rejected, + a doc note, so no future reader re-discovers "does md1 need F4?".
4. **Manual caveat** — `mk repair` chapter + auto-fire sections: a >4-error correction may alias to a different valid card; the set-level re-verify now rejects an mk1 wrong-fit, and BIP-93 recommends confirming a corrected codex32 string.

**OUT (YAGNI / separate cycles):**
1. **ms1 single-string demotion** — the highest-funds-risk leg, but design-heavy (no payload oracle for raw entropy → needs an advisory/exit-code UX decision). **Separate Cycle F** (user-sequenced after this).
2. Any mk-codec / md-codec / ms-codec wire-format or API change (`decode`/`reassemble_from_chunks` already public + sufficient).
3. Widening md1's 20-bit id to a 32-bit hash (optional hardening, not this cycle).
4. Re-citing the eval's unverified numeric rate as established fact (recon §crypto — measure it instead).

## §1 — Problem (recon-confirmed; crypto verified vs BIP-93)

The codex32 BCH regular code corrects ≤4 substitutions (BIP-93: "adequate to correct 4 errors in up to 93
characters"; long code likewise t=4 — verified vs `bitcoin/bips` master). Beyond t=4, bounded-distance decoding
can land a corrupted chunk within the radius-4 ball of a **different** valid codeword (standard coding-theory
consequence of distance-9; the code's own comment `mk-codec bch.rs:441` "Catches the 5+-error edge case"
acknowledges the shape). The defensive re-verify `bch_verify_regular/long` (`bch.rs:442/:495`) is a **codeword-
membership** test — it passes for ANY valid codeword, so it CANNOT distinguish the original from a wrong-fit
alias. `mk repair` (mk-cli) and the toolkit mk1 auto-repair arm both return success (exit 5 / apply) after only
the per-string correction, **skipping the cross-chunk SHA-256 hash that `mk decode` already enforces**
(`mk_codec::string_layer::reassemble_from_chunks` → `Error::CrossChunkHashMismatch`). So a wrong-fit mk1 repair
is blessed as a successful recovery of a **different wallet's key card**. Toolkit auto-repair fires by DEFAULT
under a TTY (`resolve_no_auto_repair` = `no_auto_repair || !tty`) on `convert`/`inspect`/`verify-bundle`/xpub
intake — so this can fire silently on real recovery. (BIP-93 normative: "implementations SHOULD NOT
automatically proceed with a corrected codex32 string without user confirmation" — recon §primary-source.)

## §2 — Fix mechanism (mirror the existing indel oracle)

The toolkit's INDEL recovery path ALREADY does exactly what F4 asks: `Mk1IndelOracle::validate`
(`src/repair.rs:1036-1056`) calls `mk_codec::decode(&refs)` (full reassembly + cross-chunk hash) on the
candidate-corrected chunk set and accepts ONLY if `decode` succeeds. The substitution-repair path sits in the
same file and skips it. **The fix = apply the same idiom to the substitution path** (not a new mechanism):

- **toolkit `repair_card` `CardKind::Mk1`** — after building `corrected_chunks`, call `mk_codec::decode(&refs)`
  (refs from the corrected chunks). If it **succeeds** → the corrections reassemble to a self-consistent card
  → genuine repair, proceed. If it **fails** (`CrossChunkHashMismatch` or any decode error) → the per-string
  corrections were a wrong-fit → **do NOT bless**: the repair is rejected (auto-repair does NOT short-circuit
  with the miscorrected card; the caller sees "correction did not reassemble — could not repair", the original
  un-repaired failure surfaces). Mirror `Mk1IndelOracle::validate`'s accept/reject exactly.
- **`mk repair` CLI (mk-cli `repair.rs`)** — after the per-string loop, build `refs` from `corrected_chunks`
  and call `mk_codec::decode(&refs)` (or `reassemble_from_chunks`). On success → exit 5 (REPAIR_APPLIED, as
  today). On failure → the correction is a wrong-fit → return a NON-5 outcome (repair-failed / "corrected each
  chunk but the set does not reassemble — likely >4 errors; re-read the plate", exit code per §3). This makes
  `mk repair` no weaker than `mk decode`.
- **Single-chunk mk1** (count=1): `mk_codec::decode` still applies (it validates the single reassembled chunk);
  a single-chunk mk1 has less cross-chunk signal but the decode-layer checks still run — confirm the re-verify
  is a no-weaker gate than today for count=1 (R0 focus §8).

**Funds property:** a corrected mk1 set is blessed as a successful repair IFF it reassembles through the same
`mk_codec::decode` a normal `mk decode` uses — so a wrong-fit alias (which fails the cross-chunk hash with
overwhelming probability) is rejected, never engraved/accepted as a different wallet.

## §3 — Exit-code / behavior semantics (R0 to finalize)

- **toolkit auto-repair:** on re-verify failure, `try_repair_and_short_circuit` must NOT short-circuit as
  repaired — behave as "repair not applied" (the decode/inspect surfaces its normal un-repaired error). No new
  exit code needed for the auto path (it simply doesn't claim success). Confirm no regression to the existing
  exit-4 "VERIFY-ME"/`--max-subst` candidate convention (`indel_exit_code`, `repair.rs:1118-1136`).
- **`mk repair` CLI:** today exit 5 = REPAIR_APPLIED, exit 0 = clean. On re-verify failure the correction is
  provably invalid → **default = treat as repair-failed** (a non-5, non-0 error exit — R0 picks: reuse the
  existing "could not repair" error exit, don't invent a new code unless mk-cli already has a "candidate"
  convention to mirror). Do NOT return exit 5 for a set that fails reassembly.
- This is a **breaking behavior change to the exit-code contract** (a previously-"exit 5 success" wrong-fit now
  errors) → MINOR bump (§7). Manual + any `verify-examples` golden that pinned the old exit must update.

## §4 — Test / oracle matrix (TDD-first)

1. **(FUNDS ANCHOR) mk1 wrong-fit correction is REJECTED** — construct/seed an mk1 multi-chunk card whose
   corruption (≥5 substitutions in one chunk) makes `bch_correct` alias to a valid-but-wrong codeword (the
   measurement harness §4.6 finds such a seed); assert `mk repair` returns NON-5 (not exit 5) AND the toolkit
   auto-repair does NOT short-circuit (the miscorrected card is not accepted). Assert the message says the set
   did not reassemble.
2. **Genuine ≤4-error correction still succeeds** (regression) — a card with ≤4 substitutions in a chunk still
   `mk repair` exit 5 + toolkit auto-repair applies (the corrected set reassembles).
3. **Clean card unchanged** — no correction → exit 0 / no auto-repair.
4. **toolkit `convert`/`inspect` auto-repair** on a wrong-fit mk1 no longer silently emits the wrong card
   (drives the default-TTY path; asserts it does not short-circuit as repaired).
5. **md1 regression-lock (§0.3)** — an md1 wrong-fit correction is ALREADY rejected by the cross-chunk
   content-id check (assert reject); documents md1 is covered without a structural change.
6. **(HARNESS, user-requested) measure + pin the mk1 miscorrection rate** — a seeded Monte-Carlo/property test
   (`tests/prop_repair_never_wrong.rs`): random mk1 payload → encode → inject exactly 5 substitutions in the
   regular-code trailing chunk → `bch_correct` → record how often it (a) returns a valid-but-≠-original
   codeword AND (b) that codeword is now REJECTED by the §2 reassembly re-verify. Pin the measured
   miscorrection rate ≤ a conservative bound; assert the re-verify catches ~100% of the misfits (any that
   reassemble are, by the cross-chunk hash, astronomically unlikely — R0 to bound). Deterministic seed; cite
   the MEASURED number in the CHANGELOG/manual (NOT the eval's `2⁻¹³·⁹`).
7. Determinism; full `cargo test -p` green per repo touched.

## §5 — Cross-source anchors (recon-verified @ the §-header SHAs)
- `mnemonic-key/mk-cli/src/cmd/repair.rs:63-90` (loop, no reassembly) — the mk-cli fix site.
- `mk-codec/src/string_layer/{chunk.rs:109 reassemble_from_chunks, mod.rs:38 pub use, bch.rs:442/:495 bch_verify_*, :173-179 GEN_REGULAR}` — primitives (reused, NOT changed).
- toolkit `src/repair.rs`: `repair_card` Mk1 arm `:766-783`; `repair_chunk_one` defensive `polymod_residue` `:731-739`; `Mk1IndelOracle::validate` `:1036-1056` (the template); `resolve_no_auto_repair` `:411-418`; `indel_exit_code` `:1118-1136`.
- toolkit auto-repair callers: `convert.rs:790-1051`, `inspect.rs:93-133`, `verify_bundle.rs` (×6), `xpub_search/seed_intake.rs:184`.
- md1 already-protected: `md-codec/src/chunk.rs:306-386` (`reassemble` + `:379-386` cross-chunk-id check); toolkit `repair_via_md_codec` `:1219-1262`.
- Manual: `docs/manual/src/40-cli-reference/41-mnemonic.md:2990-3037` (`## mnemonic repair`), `:739-751` (auto-fire); `44-mk-cli.md` (`mk repair`).

## §6 — Cross-repo coordination + release
- **Changes:** mk-cli (`repair.rs` re-verify) + toolkit (Mk1 arm re-verify + harness + md1 test) + manual. **mk-codec/md-codec/ms-codec NO source change** (primitives reused). 
- **SemVer:** mk-cli MINOR (exit-code behavior change); toolkit MINOR (`v0.80.0`). Codecs NO-BUMP.
- **crates.io / pin lockstep:** mk-cli publishes its MINOR; the toolkit's `install.sh` FROZEN mk-cli sibling pin (currently mk-cli v0.11.x) must bump to the new mk-cli tag IN LOCKSTEP — **CAUTION** (memory: bumping a sibling pin without the matching release breaks `sibling-pin-check`; here we DO bump mk-cli, so the toolkit sibling pin bumps with it). The toolkit self-pin (line 32) bumps for the toolkit MINOR. R0 + the release ritual must sequence mk-cli release → toolkit sibling-pin bump.
- **Order:** mk-cli change + release (tag + crates.io publish) FIRST; then toolkit (re-pin mk-cli sibling + self-bump + tag). (Toolkit's mk1 arm uses `mk_codec` the LIBRARY, unchanged — so the toolkit fix is independent of the mk-cli binary; but the sibling-pin lockstep + manual mirror couple them.)
- **Manual lockstep:** mandatory (exit-code semantics change) — `41-mnemonic.md` + `44-mk-cli.md` repair chapters. **GUI:** likely no-op (exit codes aren't in `schema_mirror`'s flag surface; no new flag) — verify at plan time.

## §7 — Risks / R0 focus
1. **The re-verify must be a no-weaker gate** — a GENUINE ≤4-error correction must still reassemble + succeed (no false-reject of a real repair); §4.2 pins it.
2. **Single-chunk mk1 (count=1)** — confirm `mk_codec::decode` on a 1-chunk set is a meaningful re-verify (or at least no-weaker than today), and the fix doesn't regress single-chunk repair.
3. **The measured rate + the re-verify catch-rate** — the harness must show the re-verify rejects the misfits (funds property), and pin a defensible bound; don't hard-code the eval's unverified figure.
4. **Exit-code contract change** — enumerate every `verify-examples`/manual golden that pins `mk repair` exit 5 and update in lockstep.
5. **Cross-repo release sequencing** — mk-cli release before toolkit sibling-pin bump; `sibling-pin-check` stays green.
6. **Auto-repair callers** — all 4 toolkit surfaces (convert/inspect/verify-bundle/xpub) inherit the fix via `try_repair_and_short_circuit`; confirm none bypass it.

---

*R0 gate: converge to 0C/0I via the opus-architect loop (persisted to `design/agent-reports/`) BEFORE any
implementation, per CLAUDE.md.*
