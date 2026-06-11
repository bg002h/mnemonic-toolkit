# R0 round-1 architect review — PLAN_ms_codec_pin_bump_0_4_2 (2026-06-10)

Reviewer: Fable 5 architect agent (R0, pre-implementation). Plan @ design/PLAN_ms_codec_pin_bump_0_4_2.md, master @ 6101fe0 (lock pre-bumped to ms-codec 0.4.2). Verdict: RED (1 Critical / 2 Important / 3 Minor). Review verbatim below.

---

## Critical

**C1 — The 0.4.2 combine-guard SECURITY fix is consumed by the toolkit's `ms-shares combine` but the plan provides ZERO characterization for it; this is a security-relevant behavior change shipping uncovered.**
ms-codec 0.4.2 fixed: uniform-uppercase secret-at-`S` bypassed `SecretShareSuppliedToCombine` (raw `b's'` compare missed `b'S'`; the interpolation short-circuit RETURNED the secret). The toolkit's `ms-shares combine` delegates straight to `ms_codec::combine_shares` (`cmd/ms_shares.rs:385`), so **pre-bump `mnemonic ms-shares combine --share <UPPERCASE secret-at-S> --to entropy` would have LEAKED the secret instead of refusing.** Verified post-bump: now `error: ms1 the secret share (index 's') must not be combined …` (exit 2), byte-matching the lowercase refusal. Pre-fix mechanism confirmed in 0.4.0 source (shares.rs:184/204 parse without lowercasing; guard :208 `!= b's'` misses `b'S'`). The toolkit FOLLOWUP (FOLLOWUPS.md:82) itself flags the leak. **Fix:** add a red-first characterization cell — `ms-shares combine --share <UPPER secret-at-S>` → exit 2 + `SecretShareSuppliedToCombine` prose + NO secret bytes on stdout. Fixture: the existing `VALID_MS1` uppercased IS a secret-at-S card (single-string ms1 = threshold-0/index-s). NOT scope-creep — it's the consumer-side proof of a shipped security fix.

## Important

**I1 — A 4th cell goes semantically stale (false doc/name) but stays GREEN, so CI won't catch it.**
`verify_bundle_positional_uppercase_ms1_no_echo_ms_codec_attributed` (cli_hrp_case_insensitive.rs:327-353) passes pre- and post-bump — but for the WRONG reason: uppercase ms1 now DECODES (`ms1_decode: ok`); the `.failure()` (exit 4) comes entirely from absent mk1/md1 (`mk1_decode: fail ChunkedHeaderMalformed`), NOT any ms1 error. Its name (`..._ms_codec_attributed`), doc (:324-326), and assertion comment (:347 "uppercase ms1 must HRP-classify not UnknownHrp") now assert a FALSE premise. The empirical "exactly 3 flip" sweep misses it because it stays green. **Fix:** add this 4th cell to the inversion list — rewrite name/doc, or convert to a positive `ms1_decode: ok` assertion for the uppercase positional.

**I2 — The MODULE-level doc-comment (cli_hrp_case_insensitive.rs:1-27) carries the same now-false claims.** Lines :5-6 ("ms-codec 0.4.0's envelope layer rejects uppercase past codex32") and :12-13 ("uppercase MS1 positional ... ms-codec-attributed error (WrongHrp 'MS')") are now false. The plan's "rewrite each cell's doc-comment" omits the module header. **Fix:** add the `//!` header (:5-6, :12-13) to the rewrite scope.

## Minor

**M1 — "Watch all THREE tag-gated workflows" is inaccurate.** On a `mnemonic-toolkit-v*` tag only TWO fire: `changelog-check.yml` + `install-pin-check.yml`. `rust.yml` (the inversions) + `manual.yml` (the manual edit, master-push `docs/manual/**` path) + `technical-manual.yml` (`crates/**/tests/**`) fire on the MASTER push. Reword so the implementer watches rust.yml + manual.yml (the gates that catch this cycle) on the push, the 2 on the tag.

**M2 — The 3rd RED cell removes a DISTINCT string.** Cells at :315/:386 reference `ms1 wrong HRP: got "MS"`; the 3rd (`repair_ms1_flag_uppercase...`, :361-371) asserts `repair: chunk 0 HRP mismatch — expected 'ms', found 'MS'` (`RepairError::HrpMismatch`). Note it so the implementer doesn't grep only for WrongHrp. (Confirmed: repair now echoes the card back UPPERCASE, exit 0, advisory on stderr.)

**M3 — `docs/manual/.../43-ms.md:321` references `ms-cli v0.4.0` — LEAVE IT.** That's the sibling CLI version (governed by the sibling repo), NOT ms-codec; not made stale by this bump. Flagging so the implementer doesn't "helpfully" bump it.

## Verified clean

- **Exactly 3 cells flip, nothing else broke:** confirmed full-workspace (only cli_hrp_case_insensitive 3-failed; ms-shares/inspect/verify-bundle/restore/slip39/seed-xor all green).
- **3 inversions match actual behavior** (verified vs target/debug/mnemonic): inspect → exit 0 + kind:ms1/tag:entr + advisory + NO card echo (even `--reveal-secret`); repair → exit 0 + uppercase passthrough + advisory; silent-payment → **uppercase and lowercase produce BYTE-IDENTICAL output** (`sp1qqfqnnv8cz…` + all 6 fields, via diff) — the correctness pin HOLDS.
- **Delegation:** `cmd/ms_shares.rs:385` → `ms_codec::combine_shares` inherits both 0.4.1 length-reject + 0.4.2 secret-guard. The 0.4.1 length characterization is genuinely optional (ms-codec-side); the C1 security characterization is NOT.
- **Cargo.lock isolation:** exactly 2 lines change (ms-codec version + checksum); no codex32/zeroize/transitive moved.
- **SemVer PATCH well-supported (not merely deferred):** v0.53.3 shipped the mk1/md1 legs as PATCH and deferred the ms1 leg to "the toolkit pin bump"; ms-codec classified 0.4.1+0.4.2 PATCH; uppercase ms1 was always spec-valid + merely misattributed (weaker than v0.49.0's genuinely-new format). State PATCH as strongly-favored with the v0.53.3 anchor; advisor-at-tag still fine.
- **Version sites accurate:** Cargo.toml:3, README.md:13 + crates/mnemonic-toolkit/README.md:9 (BOTH gated by readme_version_current), install.sh:32, CHANGELOG. Manual ms1-case target = 41-mnemonic.md:3113-3116. codex32 exact-pin comment stays accurate.

## Verdict

**RED — 1 Critical / 2 Important.** The empirical recon is sound and "exactly 3 flip" is verified true, but the plan under-scopes: (C1) ships a security fix through ms-shares combine with no consumer characterization, (I1) misses a 4th false-but-green cell, (I2) misses the stale module doc. Fold all three + the minors, re-dispatch.
