# v0.13.0 SLIP-39 P2.2 R0 pre-GREEN test-design review

**Reviewer model:** Opus 4.7
**Date:** 2026-05-14
**Reviewed commit:** 44f01dc (test(slip39): v0.13.0 P2.2 RED)
**Authoritative artifacts:** design/PLAN_v0_13_0_p2.md + design/SPEC_slip39_v0_13_0.md
**Verdict summary:** 0C / 3I / 5N / 0n

## Summary

The 8 test files at commit `44f01dc` are well-structured and mirror the seed-xor precedent
faithfully. The 24 refusal-test stems are byte-faithful against plan §3.2's `format!`
templates; all 6 advisory classes (1a-1e, 2, 3, 4, 5, 6) are covered with the R0 C1 fold
(empty-passphrase still fires) explicitly pinned; the R0 N4 JSON field-order pin and the
R0 Note 2 trailing-whitespace passphrase round-trip pin both correctly exercise their
target invariants; the lint-anchor count (28) and zeroize +1 are sound; and the
`lint_world_readable_helper.rs` partial-migration guard is well-conceived.

**GREEN is NOT gated on any Critical finding.** Three Important findings surface a pre-coded
GREEN-handler check-order contract gap (rows 21/22 + the implicit identifier-precedence in
row 22) and one assertion-tightness issue (advisory row 5 `--iteration-exponent 5` could
match plan §3.3 row 5 more strictly). All 3 are recommended for fold before GREEN; the
LOCK round can re-evaluate if the GREEN handler implementation makes one or more moot.

## Findings

### Critical

(none)

### Important

#### I-1 — Rows 21/22 check-order contract is undefined; tests will fail nondeterministically against the GREEN handler

`cli_slip39_refusals.rs:541-563` (`refusal_row_21_share_value_length_mismatch`) and
`:569-584` (`refusal_row_22_extendable_mismatch`) acknowledge in their own comments that
the test inputs trigger *multiple* refusal classes simultaneously, and that "which
mismatch surfaces first depends on the GREEN handler's check order." The plan §3.2 mapping
table assigns each of rows 21/22 to a lib variant (`ShareValueLengthMismatch` /
`ExtendableMismatch`) but does NOT specify the check ORDER inside `slip39_combine` (or in
a CLI-side pre-flight). Concretely:

- `refusal_row_21_share_value_length_mismatch` mixes `V4_SHARE_0` (ext=false, 128-bit,
  20-word) with a 33-word ext=true 256-bit share. Both `ShareValueLengthMismatch` AND
  `ExtendableMismatch` apply. Additionally the two shares come from independent splits
  (different identifiers), so `IdentifierMismatch` (row 7) is ALSO a candidate.
- `refusal_row_22_extendable_mismatch` mixes `V43_EXT_TRUE` (ext=true) + `V4_SHARE_0`
  (ext=false). Same value length (both 128-bit), but different identifiers — so
  `IdentifierMismatch` may fire before `ExtendableMismatch` depending on order.

Without a pinned contract, the GREEN handler implementer can pick any of {7, 21, 22} for
either test, and only one ordering will pass the existing assertions. Two of three
plausible orderings will fail the test set despite the handler being algorithmically
correct.

**Fold suggestion:** Either (a) pin the check-order contract in plan §3.2 (recommended:
identifier → iteration-exponent → group_threshold → group_count → value_length → ext-bit
→ per-share parse refusals → digest), and update the test fixtures to isolate ONE
mismatch class per test (e.g. construct row-21 inputs that share identifier+ext but
differ only on value length); OR (b) loosen the row-21/row-22 assertions to accept ANY of
the plausible-firing stems and let LOCK pin the order after observing the GREEN
implementation. Option (a) is structurally cleaner. Note: constructing isolated-mismatch
fixtures may require canonical vectors from vectors.json that don't exist; in that case
the test pair stays but the assertions go disjunctive (`stderr.contains("value length") ||
stderr.contains("extendable bit") || stderr.contains("identifier")`) with a comment
documenting the surface-area amalgam. Either path closes the contract gap.

#### I-2 — Advisory row 5 (`E=5` G9 threshold) assertion fails to byte-pin the full stem; SPEC §2.6 row 5 is significantly longer than what's asserted

`cli_slip39_advisories.rs:391-416`
(`advisory_iteration_exponent_5_emits_g9_row_5`) asserts only three substrings:
`"--iteration-exponent E=5"`, `"320000"`, and `"PBKDF2-HMAC-SHA-256"`. The plan §3.3
row 5 / SPEC §2.6 row 5 stem template is much longer:

```
warning: --iteration-exponent E=<E> yields <iters> × PBKDF2-HMAC-SHA-256 iterations;
split + combine performance may be observably slow (sub-second to multi-second).
Trezor's reference uses E=1 (20000 iters) as default; the SLIP-0039 spec gives no
recommended values. E >= 10 may exceed 30s on weak hardware.
```

Note the space between `<iters>` and `×` in plan §3.3 vs `<iters>×` (no space) in SPEC
§2.6. This is a plan-vs-SPEC inconsistency worth surfacing on its own. The 3-substring
test will pass against EITHER form, so it doesn't catch the divergence. More importantly,
it would also pass against a buggy handler that emits e.g. "...320000 iters; ..."
omitting the entire trailing sentence about E=10. A regression that drops the Trezor
reference / E>=10 warning would go silently undetected.

**Fold suggestion:** Tighten the assertion to byte-pin the full stem template — at
minimum add a `stderr.contains("Trezor's reference uses E=1")` and
`stderr.contains("E >= 10 may exceed 30s")` substring check; ideally do a full
`stderr.contains("warning: --iteration-exponent E=5 yields 320000 × PBKDF2-HMAC-SHA-256 iterations; ...")`
byte-pin once plan §3.3 and SPEC §2.6 are reconciled on the `<iters> × ` vs `<iters>×`
space. Mirror the row-6 env-var advisory style which is fully byte-pinned at line 469-473.
Separately, plan §3.3 and SPEC §2.6 should land in lockstep at GREEN with a unified
formatting (recommend adopting the plan's space-separated form which is more readable).

#### I-3 — `cli_slip39_stdin.rs::stdin_passphrase_stdin_preserves_trailing_whitespace_r0_note_2` is silently survivable if the handler strips on BOTH sides

`cli_slip39_stdin.rs:336-394` is the R0 Note 2 silent-correctness pin. The header
comment correctly explains that the bug only surfaces under asymmetric usage (split via
stdin stripping; combine via inline preserving). The test constructs the asymmetry
correctly: split uses `--passphrase-stdin` with `"secret-pass \n"`; combine uses
`--passphrase "secret-pass "` (inline preserves trailing space).

**The concern:** if the GREEN handler INCORRECTLY uses `read_stdin_to_string` (which
calls `.trim()`) for `--passphrase-stdin`, then split sees passphrase `"secret-pass"` (no
trailing space). Combine with inline `--passphrase "secret-pass "` sees passphrase with
trailing space → DigestVerificationFailed → assertion fails. **Good.** But if the
handler instead uses `read_stdin_passphrase` (correct per plan §3.5) AND the BIP-39
spec's NFKD normalization or any other side normalization strips trailing whitespace
internally to the Feistel layer, the test could still pass even though the user-visible
contract differs. Less concerning, but worth a check.

Additionally, the inline `--passphrase "secret-pass "` invocation does NOT first run the
empty-passphrase advisory check via the R0 C1 fold (the test discards stderr); but if
clap auto-trims the trailing space in inline argv values (unlikely but worth verifying
against your clap version), the entire test premise becomes invalid.

**Fold suggestion:** Add a precondition assertion at line 388 that pins the inline
combine's stderr contains the row-1e argv-leakage advisory (proving clap delivered the
exact byte sequence `secret-pass ` to the handler). Optionally add a second variant
covering split=`--passphrase-stdin` with `"  pass\n"` (LEADING spaces, also silent-strip
foot-gun) to catch the symmetric case.

### Note

#### N-1 — Plan §3.2 row 17 stem text diverges from SPEC §2.5 row 17 (test asserts plan form)

`cli_slip39_refusals.rs:454` asserts `"slip39 split: --from only accepts phrase=<value-or-> or entropy=<hex-or->; got xprv="`.
This matches plan §3.2 row 17's `format!` template exactly. But SPEC §2.5 row 17 reads
`"slip39 split --from only accepts phrase=<value-or-> or entropy=<hex-or->"` — note (a)
no colon after "split"; (b) no `"; got <node>="` suffix. The plan version is more
informative, but a SPEC patch is queued only implicitly. The plan §5 P2.2 GREEN SPEC patches
enumeration (lines 360, 396-399) lists §2.1, §2.5 row 24, §4 G4, §B.2, §B.3, §4 G6 — but
NOT a §2.5 row 17 wording update.

**Fold suggestion:** Add row-17 wording update to the plan §5 P2.2 GREEN SPEC patches
list, OR fold inline at GREEN as part of the SPEC §2.5 patches. Either keeps SPEC and
implementation in lockstep.

#### N-2 — Row 18 stem byte-faithful inclusion of `--from` is plan-only (correct for split); test reuses same stem for combine pairwise (a) / (c)

The test asserts `"slip39: at most one stdin consumer per invocation (across --share, --from, and --passphrase-stdin)"` in BOTH split (pairwise b at refusals.rs:488 and stdin.rs:286) AND combine (refusals.rs is split-only; stdin.rs pairwise a + c at stdin.rs:254 + :309). The stem mentions `--from` even in combine context where `--from` is not a valid flag. This is plan §3.2 §3.18's reading.

**Fold suggestion:** Optional: emit a per-subcommand variant `"slip39: at most one stdin consumer per invocation (across --share and --passphrase-stdin)"` for combine. Lower priority — the current uniform stem is fine but slightly misleading in combine context. Could be folded at GREEN at near-zero cost.

#### N-3 — Row 10 (UnknownWord) word-index assertion `at index 5` may be off-by-one against the GREEN handler

`cli_slip39_refusals.rs:316-328` constructs a bad share by replacing the 6th
(0-indexed position 5) word `wildlife` with `xyzzy`, then asserts the stem reads
`word at index 5`. SLIP-39 mnemonic positions are 0-indexed in the assertion, but the
handler's internal word_idx may be 1-indexed depending on what the parser tracks. The
SPEC §B.2.5 row 10 stem ("word at index I") and the lib variant
`Slip39Error::UnknownWord { share_idx, word_idx }` don't pin 0- vs 1-indexed.

**Fold suggestion:** Read `slip39/share.rs` parsing code at GREEN to confirm `word_idx`
is 0-indexed; if not, update the test to `at index 6`. Low-risk because LOCK round will
catch the off-by-one immediately.

#### N-4 — Row 12 `insufficient shares for group 0: need 2, got 1` assumes vector #5 group_idx=0 AND member_threshold=2

`cli_slip39_refusals.rs:355-360` asserts the byte-exact stem `"slip39 combine: insufficient shares for group 0: need 2, got 1"` against `V5_INSUFFICIENT_SINGLE`. Verified: vectors.json #5 = "5. Basic sharing 2-of-3 (128 bits)" with 1 mnemonic. The "2-of-3" reads as member_threshold=2, member_count=3, group_idx=0 (single group at split time). Both 0-indexing and the 2-of-3 reading need to be confirmed at GREEN. The lib variant `InsufficientShares { group_idx: u8, needed: u8, got: u8 }` carries these exact bytes.

**Fold suggestion:** Document in a comment at line 354 that the (group 0, need 2, got 1) tuple is derived from vectors.json #5's "Basic sharing 2-of-3 (128 bits)" semantics + single-share input. If the GREEN handler maps `Slip39Error::InsufficientShares.group_idx` differently (e.g. sentinel 255 → "<groups>"), update the assertion.

#### N-5 — `lint_zeroize_discipline.rs` slip39 row evidence `"zeroize::Zeroizing::new"` is loose

`lint_zeroize_discipline.rs:240-244` adds the slip39 row with evidence `&["zeroize::Zeroizing::new"]`. This passes if `cmd/slip39.rs` contains the string `"zeroize::Zeroizing::new"` ANYWHERE — including a passive doc-comment like `/// Wraps in zeroize::Zeroizing::new for memory hygiene`. The seed-xor row (line 207) uses the same loose evidence. So precedent stands.

**Fold suggestion:** Optional: tighten to a code-shape evidence anchor like `&["zeroize::Zeroizing::new(args.passphrase", "Zeroizing::new(parsed_from"]` to guard against doc-only false positives. Low priority — the existing seed-xor row hasn't shown drift in v0.12.0 LOCK or follow-on, so the loose anchor pattern is empirically robust.

### Nit

(none)

## Verdict

- Critical count: 0 (NOT BLOCKING; GREEN may proceed)
- Important count: 3 (recommend folding before GREEN; LOCK can re-evaluate)
- Total findings: 8

## Notes for the dispatching session

Fold suggestions for the dispatcher:

1. **I-1 (rows 21/22 check order)** — Highest-value fold. Either pin the check-order
   contract in plan §3.2 + reconstruct isolated-mismatch test fixtures, or loosen the
   row-21/row-22 assertions to disjunctive form. Either path closes the GREEN
   implementation's contract gap. Recommended: pin order in plan + adjust test fixtures
   (cleaner long-term).

2. **I-2 (advisory row 5 tightness)** — Byte-pin the full G9 advisory stem. Also reconcile
   the `<iters> × ` vs `<iters>×` space mismatch between plan §3.3 and SPEC §2.6 (adopt
   plan's space-separated form). At GREEN, the SPEC §2.6 row 5 line should match the plan
   word-for-word.

3. **I-3 (R0 Note 2 silent-correctness pin)** — Add the row-1e argv-leakage advisory
   precondition assertion to confirm clap delivered the byte-exact `"secret-pass "` to the
   inline combine path. Optionally add a leading-space variant.

4. **N-1 (row 17 SPEC wording)** — Queue SPEC §2.5 row 17 wording patch in the plan §5
   P2.2 GREEN SPEC patches list. The current 6-patch list omits this; add it for
   completeness.

Cross-cutting risks the GREEN implementer should be aware of:

- **The check-order contract for `slip39_combine`** is unwritten (I-1). The GREEN handler
  must either match the test's implied order (identifier → ... → value_length → ext-bit
  → digest) or the test fixtures must be restructured. Without one or the other, GREEN
  will land but ~2 tests will fail at LOCK CI, requiring an emergency fix.
- **The R0 Q5 extraction is a 3-call-site lockstep migration** (plan §6 risk 6). The
  `lint_world_readable_helper.rs` test (`cmd_seed_xor_no_longer_defines_private_emit_world_readable`)
  is the partial-migration guard. Good.
- **The env-var SHA-pin tests** (`cli_slip39_json.rs:330-421`) use `EXPECTED: "0"*64`
  placeholders that MUST be captured at GREEN once the JSON envelope serializer is wired
  (same pattern as `cli_seed_xor_json.rs:194`). Without capture, the SHA-pin tests will
  fail by design at first GREEN compile-and-run; the dispatcher should plan the capture
  step into the GREEN commit.
- **The Slip39Error variant set is complete** (21 lib variants per `slip39/error.rs`),
  covering 21 of 24 SPEC rows. CLI-only rows (17, 18, 19 — note row 19 IS a lib variant
  too: `EmptyShares`; in practice only rows 17, 18 are CLI-only refusals). No new lib
  variants needed at GREEN.

Reframing notes:

- The 6th SPEC patch in plan §5 P2.2 GREEN list is `SPEC §4 G6 count 23→28 update`. The
  cli-subcommands.list / 41-mnemonic.md changes are at P2.3, not P2.2. The plan is clear
  on this. No reframing needed.
- The test suite as a whole comprehensively covers SPEC §4 acceptance gates G3 (happy-path
  shape), G4 (JSON envelope SHA-pin + field order), G5 (24 refusal rows), G6 Cycle A
  (argv-leakage advisory + Zeroizing wraps via lint anchors), and G9 (E>=5 iteration-
  exponent threshold). G1, G2, G6-Cycle-B-lib, G7, G8 are out of P2.2 scope per plan §1
  and confirmed not asserted here.
