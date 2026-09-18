#!/usr/bin/env python3
"""spec-citation-gate -- machine-check SPEC_restore_wallet_id_prefix_enumerate.md.

CONTENT-ADDRESSED, so it survives the code moving. Each anchor is
(label, file, regex). The gate resolves the regex against the CURRENT source and
then checks the spec cites that line, separating the two failures that matter:

  GONE / AMBIGUOUS -- the regex matches zero or many lines. A human must look:
      the thing the spec describes may no longer exist, or may have been
      duplicated. NEVER auto-fixed.
  DRIFT -- the code still says it, at a new line; only the spec is stale.
      Mechanical, and `--fix` rewrites it.

WHY CONTENT-ADDRESSED. The first version hardcoded line numbers. A two-line code
fix then broke 18 of 39 assertions at once -- every one of them drift, none of
them substance. A gate that cries wolf on every edit gets ignored, which is worse
than no gate during the implementation phase it exists to guard.

  scripts/spec-citation-gate.py          check   (exit 0 pass / 1 fail / 2 drift)
  scripts/spec-citation-gate.py --fix    rewrite drifted line numbers in the spec

WHAT IT DOES **NOT** COVER -- still a reviewer's job:
  * whether the DESIGN is sound or the threat model complete;
  * whether a test vector can actually FAIL (it checks fixture arithmetic, not
    that an assertion asserts);
  * prose that is true of the cited line but misleading in context;
  * anything in the mnemonic-engrave demo (different repo);
  * the behaviour of code that does not exist yet.
A gate that hides its own blind spot is worse than no gate.
"""
import io, os, re, sys, math

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(ROOT)
SPEC = "design/SPEC_restore_wallet_id_prefix_enumerate.md"
S = "crates/mnemonic-toolkit/src"
T = "crates/mnemonic-toolkit/tests"
R = f"{S}/cmd/restore.rs"
P = f"{S}/permutation_search.rs"
V = f"{S}/cmd/verify_bundle.rs"
TT = f"{T}/cli_restore_md1_template_multisig.rs"

ANCHORS = [
    ("the +32 rationale", P, r"lone spurious match"),
    ("AMBIGUOUS arm", R, r"ps::SearchOutcome::Ambiguous => \{"),
    ("odd-hex decode", R, r"let bytes = hex::decode"),
    ("short-circuit at match 2", P, r"Two matches is enough to decide"),
    ("per-thread collection", P, r"matches\.lock\(\)\.unwrap\(\)\.extend"),
    ("sortedmulti id note", R, r"id-search is NOT collapsed"),
    ("cost calibration", R, r"let per = ps::calibrate_per_candidate"),
    ("prefix validation", R, r"ps::validate_prefix_strength\(prefix\.len"),
    ("the dispatch", R, r"let outcome = if id_search \{"),
    ("id_search is_some", R, r"let id_search = ctx\.expect_wallet_id\.is_some"),
    ("addr_search is_some", R, r"let addr_search = ctx\.search_address\.is_some"),
    ("PrefixTooShort mapping", R, r"SE::DuplicateKeys \{ \.\. \} \| SE::PrefixTooShort"),
    ("exit code 4", f"{S}/error.rs", r"ToolkitError::RestoreMismatch \{ \.\. \} => 4"),
    ("the shared engine", R, r"pub\(crate\) fn complete_multisig_template"),
    ("the outcome type", R, r"pub\(crate\) struct MultisigCompletionOutcome"),
    ("verify call site", V, r"complete_multisig_template\(d, &ctx, stderr\)"),
    ("verify passes the flag", V, r"expect_wallet_id: args\.expect_wallet_id\.clone"),
    ("verify copies search_address", V, r"search_address: args\.search_address\.clone"),
    ("the single emitter", R, r"fn emit_completed_multisig"),
    ("@N= early return", R, r"return complete_explicit_assignment\("),
    ("--count flag", R, r"pub count: u32"),
    ("ceiling to bad", R, r"SE::SearchTimeExceedsCeiling"),
    ("sortedmulti shape flag", R, r"let sorted_shape = crate::synthesize::is_order_independent_shape"),
    ("the A1 fix", R, r"let collapse_orderings = sorted_shape && addr_search"),
    ("the A2 check", R, r"if let Some\(prefix_hex\) = expect_wallet_id \{"),
    ("Enumeration::n", P, r"pub fn n\(&self\) -> usize"),
    ("Enumeration::cardinality", P, r"pub fn cardinality\(&self\)"),
    ("the determinism oracle", P, r"pub fn search_reference"),
    ("oracle short-circuits at 2", P, r"if found\.len\(\) >= 2"),
    # TWO matches by design: the parallel engine and `search_reference`, its
    # determinism oracle, must agree on this. Pinning the COUNT asserts the
    # oracle still mirrors the engine -- a stronger claim than "it exists".
    ("id-path address_index is 0", P, r"SearchMode::Id => 0,", 2),
    ("bare-xpub decode", R, r"^            fingerprint: Fingerprint::default\(\),"),
    ("subsets to distinct key SETS", R, r"// \(own-only over-supply OR opt-in cosigner-subset\)"),
    ("retired test 1", TT, r"fn weak_id_prefix_now_warns_and_completes"),
    ("retired test 2", TT, r"fn own_account_max_short_id_prefix_warns_and_completes"),
    ("retired test 3", TT, r"fn search_cosigner_subset_weak_prefix_warns_or_lists"),
    ("retired unit test", P, r"fn validate_prefix_strength_rejects_short_accepts_long"),
    ("A1 regression test", TT, r"fn sortedmulti_own_account_max_id_search_finds_non_identity_placement"),
    ("A2 regression test", TT, r"fn explicit_assignment_honours_expect_wallet_id"),
    ("A2 boundary test", TT, r"fn explicit_assignment_with_correct_id_still_completes"),
]

# Anchors the spec cites BY LINE. label -> the citation the spec currently holds.
# `--fix` rewrites both the spec and this map.
CITED = {
    "the +32 rationale": "permutation_search.rs:328",
    "AMBIGUOUS arm": "restore.rs:2391",
    "odd-hex decode": "restore.rs:3108",
    "per-thread collection": "permutation_search.rs:1413-1415",
    "sortedmulti id note": "restore.rs:2198-2200",
    "cost calibration": "restore.rs:2605",
    "prefix validation": "restore.rs:2272",
    "the dispatch": "restore.rs:2254",
    "id_search is_some": "restore.rs:2035",
    "addr_search is_some": "restore.rs:2036",
    "PrefixTooShort mapping": "restore.rs:2556",
    "exit code 4": "error.rs:656",
    "the shared engine": "restore.rs:1688",
    "the outcome type": "restore.rs:1240",
    "verify call site": "verify_bundle.rs:977",
    "verify passes the flag": "verify_bundle.rs:954",
    "the single emitter": "restore.rs:2974",
    "@N= early return": "restore.rs:2009",
    "--count flag": "restore.rs:238",
    "ceiling to bad": "restore.rs:2562",
    "sortedmulti shape flag": "restore.rs:2037",
    "Enumeration::n": "permutation_search.rs:1084",
    "Enumeration::cardinality": "permutation_search.rs:1098",
    "the determinism oracle": "permutation_search.rs:1470",
    "id-path address_index is 0": "permutation_search.rs:1347",
    "bare-xpub decode": "restore.rs:2492-2494",
    "retired test 1": "cli_restore_md1_template_multisig.rs:637",
    "retired test 2": "cli_restore_md1_template_multisig.rs:1169",
    "retired test 3": "cli_restore_md1_template_multisig.rs:1468",
    "retired unit test": "permutation_search.rs:1583",
}

def resolve(path, rx):
    lines = io.open(path, encoding="utf-8").read().split("\n")
    return [i + 1 for i, l in enumerate(lines) if re.search(rx, l)]

def main():
    fix = "--fix" in sys.argv
    spec = io.open(SPEC, encoding="utf-8").read()
    fail = drift = 0
    rewrites = []
    print("== anchor resolution (content-addressed) ==")
    for anchor in ANCHORS:
        label, path, rx = anchor[0], anchor[1], anchor[2]
        want = anchor[3] if len(anchor) > 3 else 1
        hits = resolve(path, rx)
        base = os.path.basename(path)
        if len(hits) != want:
            print("  %-9s %-40s %s  (%d matches, expected %d)" %
                  ("GONE" if not hits else "AMBIGUOUS", label, base, len(hits), want))
            fail = 1
            continue
        if want != 1:
            print("  ok        %-40s %s  (%d sites, as expected)" % (label, base, want))
            continue
        line = hits[0]
        cited = CITED.get(label)
        if cited is None:
            print("  ok        %-40s %s:%d  (not cited by line)" % (label, base, line))
            continue
        want = "%s:%d" % (base, line)
        # A range citation is satisfied if it STARTS at the resolved line.
        if cited == want or cited.startswith(want + "-"):
            print("  ok        %-40s %s" % (label, cited))
        else:
            print("  DRIFT     %-40s spec says %-38s code is at %s" % (label, cited, want))
            drift = 1
            new = want
            if "-" in cited:  # preserve the span length
                a, b = cited.split(":")[1].split("-")
                new = "%s:%d-%d" % (base, line, line + (int(b) - int(a)))
            rewrites.append((cited, new, label))
    if fix and rewrites:
        for old, new, label in rewrites:
            spec = spec.replace(old, new)
        io.open(SPEC, "w", encoding="utf-8").write(spec)
        me = io.open(__file__, encoding="utf-8").read()
        for old, new, label in rewrites:
            me = me.replace('"%s": "%s"' % (label, old), '"%s": "%s"' % (label, new))
        io.open(__file__, "w", encoding="utf-8").write(me)
        print("\n  --fix rewrote %d citation(s) in the spec and in this script." % len(rewrites))
        drift = 0

    print("\n== numeric gate ==")
    def req(n):
        b = 0 if n <= 1 else (n - 1).bit_length()
        return (b + 32 + 7) // 8
    def perm(n, r):
        p = 1
        for i in range(r):
            p *= n - i
        return p
    ok = True
    def eq(label, got, want):
        nonlocal ok
        if got != want:
            ok = False
        print("  %s %-44s got %s want %s" % ("ok  " if got == want else "FAIL", label, got, want))
    eq("required_prefix_bytes(6)  [demo space]", req(6), 5)
    eq("required_prefix_bytes(24)", req(24), 5)
    eq("required_prefix_bytes(P(39,11))", req(perm(39, 11)), 11)
    eq("P(39,11)", perm(39, 11), 66902793897139200)
    eq("2^64 / S  (the '1-in-N' figure)", round(2 ** 64 / perm(39, 11)), 276)
    eq("S needed for >64 matches at 2 bytes", 64 * 65536, 4194304)
    print("  --- lone-spurious worst case per space ---")
    for name, n in [("S=6", 6), ("S=24", 24), ("P(39,11)", perm(39, 11))]:
        r = req(n)
        best = max((((n * 2 ** (-8 * b)) * math.exp(-(n * 2 ** (-8 * b)))), b) for b in range(2, r))
        print("  %-12s worst P(exactly 1 spurious) = %6.2f%% at b=%d" % (name, best[0] * 100, best[1]))
    if not ok:
        fail = 1

    print()
    if fail:
        print("GATE: FAIL (an anchor is GONE/AMBIGUOUS, or arithmetic is wrong)")
        return 1
    if drift:
        print("GATE: DRIFT (code moved; re-run with --fix)")
        return 2
    print("GATE: PASS")
    return 0

sys.exit(main())
