# Implementation plan-doc R0 round 2 — restore per-key use-site override (opus architect, verbatim)

> Reviewer: opus architect (Plan agent; read-only; folds re-verified against descriptor-mnemonic `c85cd49` / mnemonic-toolkit `5e55603`).
> Plan: `design/IMPLEMENTATION_PLAN_restore_per_key_use_site_override_2026-06-19.md` (post round-1 M1-M5 folds). **Verdict: GREEN (0C/0I/0M).** Plan-doc R0 converged.

---

All facts verified. Let me record my confirmations against each fold.

**M1 verification** — Confirmed. `plain_template_from_tree` is the fn at `restore.rs:1140`; its outer `match` spans `1163-1182`. The `Wsh` arm (`1164-1170`) and the `Sh→Wsh` arm (`1171-1178`, requiring `c[0].tag == Tag::Wsh`) are the only matches; a bare `sh(multi)` (`Sh` whose single child is `Tag::Multi`, not `Wsh`) falls to the inner `_ => None` at `1179`, and any non-Wsh/non-Sh root falls to the outer `_ => None` at `1181`. So bare `sh(multi)` → `None` → faithful arm. The plan's cited range `restore.rs:1163-1182` and the routing fact are exactly correct. P2.2 TDD (line 61) and §5 item 7 (line 94) both now list it.

**M2 verification** — Confirmed. `sh(wsh(multi))` divergent now appears in P2.2 TDD (line 61) and §5 item 7 (line 94). It routes through the `Sh→Wsh→plain_leaf` arm (1171-1178): for a *divergent* override card, C1 (P2.1, `use_site_path_overrides.is_some()` gate) forces `template_opt = None` → faithful arm regardless of `plain_leaf`, so the divergent `sh(wsh(multi))` exercises the inner-reconstruction faithful path. No contradiction.

**M3 verification** — Confirmed. `descriptor-mnemonic/CHANGELOG.md` is a SINGLE shared file with crate-prefixed entries (`## md-codec [X]` / `## md-cli [Y]`), exactly as its header states ("Each release entry is prefixed with the crate name"). P1.6 (line 47) now says "add TWO crate-prefixed entries to the SHARED `descriptor-mnemonic/CHANGELOG.md` (`## md-codec [0.37.0]` … `## md-cli [0.7.1]`)". Wording matches repo structure.

**M4 verification** — Confirmed. `install.sh:35` pins `descriptor-mnemonic-md-cli-v0.6.2` while md-cli is at `0.7.0` (pre-existing lag, exactly as stated). `RELEASE_CHECKLIST.md:67` reads "**md/ms/mk-cli release**: Update the corresponding toolkit `scripts/install.sh:35/38/41` pin in lockstep" — citation correct. The checklist itself frames cross-repo PRs as not required ("a follow-up commit to toolkit master is sufficient") and documents historical non-blocking drift (lines 69-76), so "decide at execution, non-gating" is consistent with the repo's own stated stance. Acceptable, non-gating.

**M5 verification** — Confirmed. The display round-trip guard at `restore.rs:1365` (`if parsed.to_string() != descriptor`) is the parse→print fidelity check; its doc-comment (1357-1364) explicitly says the faithful arm's output is `to_string()` of a parsed descriptor and "Display-stable by construction." P2.2 (line 61) now notes it's "relied upon UNCHANGED" and that C2 adds no new round-trip risk. No contradiction — the guard sits AFTER `from_str` (1355) on the reconstructed string, downstream of C2, and a multipath descriptor's `to_string()` round-trips by the same construction the existing translator already relies on.

**Drift check:**
- No fold introduced a new Critical/Important. All five are test-coverage additions (M1/M2), doc-wording (M3), an execution note (M4), and an informational note (M5). No design decision changed — C1 gate, C2 builder, translator reduction, guard/advisory predicates, and version sites are textually identical to the round-1-verified versions.
- No section contradicts another. P2.2/§5 item 7 list the same five faithful shapes; §5.6 matrix rows map 1:1 onto them.
- §5.6 coverage is now complete: Row 1 (`wsh(multi)`, `wsh(sortedmulti)`, `sh(wsh(multi))`, Some/Some + Some/None) → P2.2 + §5 items 3,7 + P2.5; Row 2 (`sh(multi)` bare-P2SH) → P2.2 + §5 item 7 (M1); Row 3 (hardened) → P1.1/P2.3/P2.4 + §5 items 1,7; Row 4 (`tr` override) → P2.3/P2.4/P2.5 + §5 items 5,7; Row 5 (decode rejects) → P1.4 + §5 item 4; Row 6 (non-override regression) → P1.4 corpus round-trip. Every faithful row, every refuse row, decode-rejects, and non-override regression are pinned.
- Plan is internally consistent and execution-ready: phasing/ordering, version sites, the `[patch]`-then-pin mechanic, and the I1 independent golden are unchanged from the round-1-verified text.

One incidental note (not a finding, no action): the round-1 review body (line 79) cites the multipath-emitting translator arm as `restore.rs:1079-1084` and the faithful arm region differently than the current plan's `restore.rs:1105`; the plan's own citations (`faithful_multisig_descriptor` at 1105, guard at 1365) match HEAD `5e55603` exactly. The review-doc line numbers are its own snapshot and are not part of the plan; the plan's citations are the ones that govern execution and they are correct.

---

**Verdict: GREEN** — 0 Critical, 0 Important, 0 Minor.

All five folds (M1-M5) are correctly applied, each cited routing/structure fact re-verified against descriptor-mnemonic `c85cd49` and mnemonic-toolkit `5e55603`. No fold introduced new drift, contradiction, or design change. The §5.6 shape matrix is fully covered by plan §5. The plan is internally consistent and execution-ready.
