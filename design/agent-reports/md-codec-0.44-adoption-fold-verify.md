# Fold verification — `817e59c4` (C1 + I1 + I2 from the md-codec 0.44 whole-diff review)

**Reviewer:** independent agent (sonnet), 2026-09-19. Scoped verification of one fold commit,
not a fresh audit. Did not author the branch or the fold.

**Fold under test:** `817e59c4801d34e55e62e74ff684926a23f500ad` on `deps/md-codec-0.44.0`.
**Prior review:** `design/agent-reports/md-codec-0.44-adoption-whole-diff-review.md` (1C/2I/5M/2N).
**Binaries:** branch built fresh at this commit (`target/debug/mnemonic`, `mnemonic 0.101.0`,
built `2026-09-19 17:51` — after the fold commit's timestamp `17:50:46`); master reference at
`/tmp/mnemonic_master`, confirmed built from `origin/master` `9513f754` (the exact SHA the prior
review used as its baseline). Repo tree confirmed unmodified before and after (`git status
--short` diffed byte-identical except `target/`, which is gitignored).

**Answer: the fold holds. 0 new findings of any severity.**

---

## Q1 — Does the fold fix C1/I1/I2, and does it introduce a new defect?

### C1 (wrong wallet)

The fold's diff is exactly what it claims: `crates/mnemonic-toolkit/src/cmd/bundle.rs:2348`
changes `if op.components.is_empty() {` to `if op.components.is_empty() && is_non_canonical {` in
the `PathDeclPaths::Divergent` arm — the identical gate already present in the sibling `Shared`
arm eight lines above. `verify_bundle.rs` calls the same shared `bind_descriptor_mode_paths`
function (confirmed at `verify_bundle.rs:1514`), so the fix is structurally symmetric on emit and
verify with no separate code path to diverge.

**Verified by construction**, both binaries run side by side:

- **Exact repro from the brief** (`wsh(sortedmulti(2,[73c5da0a/48'/0'/0'/2']@0,@1))`, phrase slots
  `abandon×11 about` / `zoo×11 wrong`, `--no-engraving-card --allow-argv-secret --json`): master and
  branch emit **byte-identical** JSON (full `md1`/`mk1`/`ms1` sets). The regression is fixed.
- **Probed elsewhere, all pairs run master-vs-branch:**
  - Canonical, no inline origins — **identical**.
  - Canonical, all slots annotated — **identical**.
  - Canonical partial-origin with **xpub** slots (C1(b) shape, fabricated-origin-on-watch-only) —
    **identical**.
  - Canonical, 3 cosigners, only the **first** annotated — **identical** (both un-annotated slots
    stay empty).
  - Canonical, 3 cosigners, only the **middle** annotated — **identical**.
  - Non-canonical (`wsh(and_v(v:pk(@0),pk(@1)))`) with **phrase** slots + `--slot @N.path=` at
    accounts 7/8 — **identical** across versions (this path was never broken; the override loop for
    phrase-bearing slots already worked pre-fold — only the **xpub** branch had the I2 defect, see
    below).
  - Non-canonical with **xpub** slots + `--slot @N.path=` at accounts 7/8, exactly the I2 repro —
    **differs as expected** (the branch now engraves the requested account 7/8 origin instead of
    master's silently-substituted account-0 default). This is the I2 fix working, not a new defect.
  - Row-19 inline-vs-`--slot`-path **mismatch** refusal — branch still refuses
    (`error: slot @0 path mismatch: ... 48'/0'/9'/2' ... disagrees`), confirmed still reachable
    post-fold.
  - Row-19 inline-vs-`--slot`-path **consistent** values — **identical** across versions.
- **Verify-side, cross-version, both directions** (bundle on one binary → `verify-bundle
  --bundle-json` on the other), for the shapes that exercise the fixed gate:
  - C1(a) shape (canonical partial-origin, phrase slots): emit-master/verify-branch → `result: ok`;
    emit-branch/verify-master → `result: ok`.
  - Canonical, 3-cosigner, first-annotated: both directions → `result: ok`.
  - Canonical, no inline origins, phrase slots: both directions → `result: ok`.
  - C1(b) shape (canonical partial-origin, **xpub** slots): both directions report
    `result: mismatch`, but the **failing check is `mk1_fingerprint_match[0]`, identical on both
    master and branch**, with `mk1_xpub_match` and `mk1_path_match` both `ok` on both binaries. This
    reproduces identically on the unmodified `origin/master` baseline, so it predates the fold and
    is not attributable to it — most likely an artifact of this test's fixture xpub/fingerprint pair
    not being mutually consistent (a construction issue on my part, not a code defect the fold could
    have introduced or missed). Flagged here for completeness, not counted as a finding against the
    fold since it is version-invariant.

No new C1-class or emit/verify-asymmetry defect found anywhere in the probed matrix.

### I1 (Cargo.toml comment accuracy)

The corrected comment (`"NOT EVERY ARM ... a rendered key serialised at depth 0"` +
`taproot-wallet-policy-arm-still-renders-depth0` cross-reference) was checked against the current
tree by decoding the depth byte of the actual golden xpubs in `tests/cli_restore_taproot.rs`
(base58check decode + checksum verification, independent of the toolkit):

| golden | declared origin length | decoded depth | checksum |
| --- | --- | --- | --- |
| `GOLDEN_DESC_SINGLE_LEAF` key | 3 (`87'/0'/0'`) | **3** — OK | valid |
| `GOLDEN_DESC_TWO_LEAF` 2nd key | 3 | **3** — OK | valid |
| `GOLDEN_DESC_MULTI_A_2LEAF` pk-leaf key | 3 | **3** — OK | valid |
| `GOLDEN_DESC_NON_NUMS_MULTI_A` trunk key | 3 | **0** — MISMATCH | valid |
| `GOLDEN_DESC_NON_NUMS_MULTI_A` leaf key 1 | 3 | **0** — MISMATCH | valid |
| `GOLDEN_DESC_NON_NUMS_MULTI_A` leaf key 2 | 3 | **0** — MISMATCH | valid |

Two non-wallet-policy shapes render correctly (depth matches origin length); the wallet-policy
`multi_a` arm still renders depth 0 under a depth-3 origin. This is exactly what the corrected
comment and the `taproot-wallet-policy-arm-still-renders-depth0` FOLLOWUPS entry say. **Comment is
accurate.**

### I2 (FOLLOWUPS correction)

Directly measured above: on the shipped-master binary, the non-canonical xpub-slot shape
(`wsh(and_v(v:pk(@0),pk(@1)))`, accounts 7/8) emits an md1 that differs from the branch's — the
requested `origin_paths` JSON field is identical on both (`m/48'/0'/7'/2'`, `m/48'/0'/8'/2'`), but
the **card content** (the actual `md1` strings) differs, consistent with master silently
substituting a default-inferred origin while the branch engraves the requested one. This matches
the FOLLOWUPS correction paragraph's claim exactly. **Correction matches shipped behavior.**

**New defects introduced by the fold: none found.**

---

## Q2 — Is the new regression test real?

`tests/cli_slot_path_reaches_the_card.rs::a_canonical_descriptor_with_partial_inline_origins_invents_nothing`,
mutation-tested by editing `crates/mnemonic-toolkit/src/cmd/bundle.rs` **line 2348 only**
(`if op.components.is_empty() && is_non_canonical {` → `if op.components.is_empty() {`), leaving
the `Shared` arm's line 2304 (`if op.components.is_empty() {`) untouched — confirmed via
`git diff --stat` showing exactly 1 line changed before running.

- **With the mutation:** `cargo test -p mnemonic-toolkit --test cli_slot_path_reaches_the_card
  a_canonical_descriptor_with_partial_inline_origins_invents_nothing` → **FAILED**, panicking on
  the exact assertion the test exists for: `got "48'/0'/0'/2'"` — the invented origin the fold
  exists to prevent.
- **File restored** (`git diff --stat` on the file: empty) **and test re-run: `ok`, 1 passed.**
- Final `git status --short` diffed against the pre-test snapshot: byte-identical (only the
  gitignored `target/` directory changed from building). Repo tree left unmodified.

**Test is real: mutation-proven to fail without the fix and pass with it.**

---

## Verdict

**The fold holds.** C1 is fixed (verified by construction across 10+ input shapes plus
cross-version verify in both directions), I1's corrected comment is accurate (independently
decoded), I2's correction matches shipped-binary behavior (directly measured), and the new
regression test is mutation-proven real.

**New findings: 0 Critical, 0 Important, 0 Minor, 0 Nit.**

One out-of-scope observation (not a fold defect, not counted above): the C1(b) xpub-slot
cross-version verify probe hit a `mk1_fingerprint_match` failure that reproduces identically on
unmodified `origin/master`, so it predates this branch entirely and is most likely my own test
fixture (xpub/fingerprint pair not mutually consistent) rather than a code defect.
