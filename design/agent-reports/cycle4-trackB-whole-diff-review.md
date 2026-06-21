# Cycle-4 Track B — whole-diff execution review (ms-codec M6 Shamir consistency)

- **Worktree HEAD:** `64d391b` (`fix(cycle4-m6): cross-share polynomial-consistency check in combine_shares`)
- **Base:** `6b28918` (`origin/master`)
- **Date:** 2026-06-21
- **Reviewer:** opus software architect — mandatory independent adversarial execution review (post-R0, pre-tag/publish)
- **Diff scope:** `crates/ms-codec/src/shares.rs` (combine_shares step-5 rewrite + 4 tests), `crates/ms-codec/src/error.rs` (`InconsistentShareSet` variant + Display arm), `crates/ms-cli/src/error.rs` (`From` arm + CLI test)
- **Spec:** `BRAINSTORM_cycle4_codec_funds_fixes.md` §6 (§6.0 framing, §6.2 the exact check, §6.5 test plan)

---

## Methodology

Traced the implementation against the live codex32 0.1.0 source (`~/.cargo/registry/.../codex32-0.1.0/src/lib.rs`, the actual dependency), not the spec draft. Verified type/index alignment line-by-line, ran the full workspace suite + clippy, and executed **two independent mutation tests** to prove the new tests are non-vacuous and that BOTH load-bearing mechanisms (the membership loop AND the k-truncation) are individually necessary.

---

## Critical findings

**None.**

---

## Important findings

**None.**

---

## Minor findings

### Minor-1 — pre-existing unrelated flake confirmed orthogonal (informational, no action)

`ms-cli` test `mlock::tests::g4_a_pin_and_zeroize_compose_without_panic` (`crates/ms-cli/src/mlock.rs:434`) intermittently fails ("64-byte buf pins exactly one page: left 2 right 1"). It is **not introduced by this diff** — `git diff origin/master..HEAD --stat -- crates/ms-cli/src/mlock.rs` is empty (mlock.rs untouched). It is the known, FOLLOWUP-tracked, architect-"leave-tracked" g4_a/g6-synced page-pinning environmental flake. It flipped between two full-suite runs in this session (one run: 1 fail; aggregate run: 337/0), confirming nondeterminism unrelated to M6. No bearing on Track B's correctness or tag-readiness.

### Minor-2 — comment-only drift in the `combine_shares` doc header (cosmetic, optional)

The `///` doc block above `combine_shares` (shares.rs:179) still describes step 5 as the pre-M6 `interpolate_at(&parsed, Fe::S)` ("recovers the secret-at-S"); the inline `//` comment at the body (shares.rs:261–290) is fully updated and accurate. The stale doc-header line is harmless (the body comment is load-bearing and correct) but could be aligned in a future no-bump pass. Not a defect.

---

## Adversarial hunt results (the 6 directed items)

### Hunt-1 — Index correctness / `fields[]`↔`parsed[]` alignment (the crux) — **CLEAN**

The decisive question: is `fields[j]` index-aligned with `parsed[j]`, and is `idx_j = Fe::from_char(fields[j].1)` the correct polynomial-evaluation index for `parsed[j]`?

- `fields` is built by `parsed.iter().map(...)` (shares.rs:224–230) in the SAME order as `parsed`. There is **no sort/dedup/reorder** of `parsed` or `fields` anywhere in `combine_shares` between parse (181) and the membership loop (284) — grep-confirmed (the only `sort`/`dedup` hits in the file are inside a test at lines 401–403). Therefore `fields[j].1` is exactly the share-index byte of `parsed[j]`. Alignment is structural and exact.
- `Fe::from_char(fields[j].1 as char)` recovers the field element of that share's index, identical to the conversion the distinct-index check already uses (shares.rs:255). `fields[j].1` is `extract_wire_fields(...).share_index_byte` taken from the canonical lowercased wire string, so the char is already lowercase-canonical — no case mismatch into `Fe::from_char`.
- `interpolate_at(k_set, idx_j)` (codex32 lib.rs:217) reconstructs the FULL `Codex32String` codeword at `idx_j` via Lagrange over the entire payload region (header threshold/id/index chars + data + checksum, lib.rs:268–308), so the reconstructed string carries `idx_j` baked into its index position. The comparison `derived != parsed[j]` is the right object (the full canonical share string vs the reconstructed full share string).
- **No spurious short-circuit:** codex32's `interpolate_at` returns an input share directly only when `target` ∈ input indices (lib.rs:259–262). In the membership loop `target = idx_j` and `k_set = parsed[..k]`; the distinct-index pre-check (step 4) guarantees `idx_j` is NOT among the first-k indices, so the loop genuinely re-derives the curve's value rather than echoing an input. **This is exactly why the k-truncation is mandatory** — Mutation 2 (below) proves removing it re-enables that short-circuit and silently false-accepts.

Verdict: index derivation and `fields[]`↔`parsed[]` alignment are correct. `interpolate_at(&k_set, idx_j)` reconstructs share `j`'s expected on-curve value and is compared against the correct object.

### Hunt-2 — False-reject risk (lockout of a legitimate n>k consistent combine) — **CLEAN**

- **Canonicalization/case:** `parsed` is re-parsed from its `to_ascii_lowercase()` copy (shares.rs:203–209), so `parsed[j]` is lowercase. `interpolate_at` emits lowercase chars unless the HRP is all-uppercase (lib.rs:298–306); HRP here is lowercase `ms`, so `derived` is lowercase. Both sides of `derived != parsed[j]` are byte-canonical lowercase. No case-induced false-reject.
- **k_set selection (first-k, not sorted):** the polynomial of degree <k is uniquely determined by ANY k of its points; a legitimately-encoded extra share lies on the SAME polynomial regardless of which k consistent shares are chosen as `k_set`. So first-k is sound for the all-consistent case.
- **Full-string vs payload-only compare:** the compare is full `Codex32String` (PartialEq on the inner `String`, codex32 lib.rs:101). For a legitimate extra share, `interpolate_at` reconstructs the identical hrp+threshold+id+index+payload+checksum string. This is **empirically proven byte-identical** by the positive control `combine_valid_n_gt_k_all_consistent` (all 3 shares of a real `encode_shares` set → `Ok`, exact payload), which exercises precisely the n>k membership path and stays GREEN.
- The `combine_valid_exactly_k_unchanged` control proves the k==n / exactly-k path (empty membership loop) is bit-identical to prior behavior.

Verdict: no false-reject. A valid extra is byte-identical to its interpolation; both positive controls GREEN.

### Hunt-3 — False-accept risk (silent wrong secret slips through) — **CLEAN**

- **Loop covers EVERY extra regardless of position:** `for j in k..parsed.len()` iterates all of `parsed[k..]`, and any single mismatch `return`s `InconsistentShareSet`. Test #4 (`combine_inconsistent_extra_share_rejected`) places the inconsistent B-share at a NON-terminal position `[A1, A2, B3, A4]` (j=2 catches it before the terminal consistent A4 at j=3) — verified the code iterates all extras, not just the last. GREEN with fix.
- **Inconsistent share cannot hide inside the first k as a free pass:** the check's contract is "the first k define the polynomial; every EXTRA must lie on it." If an inconsistent share lands inside the first k, it simply participates in DEFINING the reference polynomial, and the extras are still checked against that polynomial. The genuinely-undetectable case is exactly-k with a mixed pair (`[A1,B2]`) — k points always define *a* polynomial with zero extras to cross-check — which the spec (§6.2 edge cases) and the test docstring both correctly call out as the irreducible BIP-93 limit, out of scope. No regression: that case was equally undetectable pre-M6.
- **Comparison is not too lenient:** exact `!=` on the full canonical string; no tolerance, no payload-only narrowing.
- **Mutation 1 (revert membership check → pre-M6 `interpolate_at(&parsed, Fe::S)`):** both inconsistent tests went RED — and critically the mutated combine returned `Ok(garbage)` (no error), **empirically reproducing the silent-wrong-secret CVE this fix closes**. Both positive controls stayed GREEN, proving the check is exactly what catches inconsistency and the happy path doesn't depend on it.
- **Mutation 2 (drop the k-truncation: `k_set = &parsed[..]`):** both inconsistent tests went RED (false-accept) because `interpolate_at` then short-circuits on the extra's own index (lib.rs:259–262) and trivially returns `derived == parsed[j]`. This proves the `&parsed[..k]` truncation is independently load-bearing AND that the tests detect its loss. Positive controls stayed GREEN.

Verdict: no false-accept. Loop covers all extras; both the membership check and the truncation are each necessary, each test-guarded, and non-vacuous.

### Hunt-4 — Regression to existing combine semantics — **CLEAN**

All pre-M6 guards remain intact and correctly ordered ahead of the new check: empty-input → ThresholdNotPassed (211–217); lowercase canonicalization (203–209); C1 secret-at-S `SecretShareSuppliedToCombine` reject (234–236); threshold `parsed.len() >= k` (242–248); exhaustive distinct-index `RepeatedIndex` (252–259). The new step-5 runs strictly after all of them. The full `combine_*` test family (round-trip entr+mnem all lengths, below-threshold, duplicate-index, secret-share-index, mismatched-threshold, nonstandard-length-no-panic, plus the two positive controls) is GREEN — 81/81 ms-codec lib tests pass.

### Hunt-5 — Error routing — **CLEAN**

- ms-codec `Error::InconsistentShareSet` Display arm present (error.rs:233–238); the `fmt::Display` match has no wildcard, so exhaustiveness is compile-forced — the workspace builds clean, proving the arm exists.
- ms-cli `From<ms_codec::Error>` routes `InconsistentShareSet → CliError::FormatViolation { underlying_kind: "InconsistentShareSet", ... }` (error.rs:236–242), placed ABOVE the `other =>` BadInput wildcard (error.rs:260) — not shadowed (top-to-bottom match resolution; explicit variant wins).
- `FormatViolation { .. } => 2` (error.rs:50) → exit 2 (funds-safety/format-violation class), NOT the exit-1 BadInput wildcard. `kind()` carries `underlying_kind` ("InconsistentShareSet"). CLI test `inconsistent_share_set_maps_to_format_violation_exit_2` asserts kind, exit==2, message contains "same split" — GREEN.

### Hunt-6 — Mutation / non-vacuity — **CLEAN (both mutations bite; positives stay GREEN)**

| Mutation | Inconsistent tests | Positive controls |
|---|---|---|
| (baseline, fix in place) | GREEN (correctly reject) | GREEN |
| M1: remove membership loop (pre-M6 all-share interpolate) | **RED** (Ok garbage, no error) | GREEN |
| M2: drop k-truncation (`k_set = &parsed[..]`) | **RED** (short-circuit false-accept) | GREEN |

Both load-bearing mechanisms are individually necessary and individually test-guarded. shares.rs restored byte-identical to `64d391b` after mutation runs (`git diff HEAD` empty; 81/81 GREEN).

---

## Build / test / lint corroboration

- `cargo test --workspace --all-targets` → **337 passed / 0 failed** (the implementer's claim is confirmed; the lone g4_a flake passed in the aggregate run and is diff-orthogonal per Minor-1).
- `cargo test -p ms-codec` (the M6 home) → all GREEN, incl. all 4 M6 tests.
- `cargo clippy --all-targets -- -D warnings` → exit 0, **clippy clean** (claim confirmed).

---

## Verdict

**TRACK-B WHOLE-DIFF: 0C / 0I** — **GREEN (0C/0I, cleared to tag/publish).**

The M6 fix is correct, minimal, and faithful to spec §6. Index alignment is structural and exact; the membership check neither false-rejects a valid n>k combine (empirically byte-identical reconstruction) nor false-accepts an inconsistent set (loop covers all extras; truncation defeats the codex32 short-circuit). Both safety mechanisms are mutation-proven non-vacuous, error routing lands on exit-2 FormatViolation, and no existing combine semantics regressed. The two Minor items are informational/cosmetic and do not block.
