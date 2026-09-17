#!/usr/bin/env bash
# spec-citation-gate.sh -- machine-check SPEC_restore_wallet_id_prefix_enumerate.md.
#
# Asserts (a) every file:line the spec cites still contains what the spec claims
# it contains, and (b) every arithmetic claim recomputes. Citations decay every
# time someone edits restore.rs; a fold adds new ones that nobody has checked.
# Run it BEFORE committing any fold and BEFORE dispatching any reviewer -- a
# reviewer reporting a stale line number is being paid design-review rates to
# act as grep.
#
# It has already earned its keep once: the first fold cited
# verify_bundle.rs:951 for `expect_wallet_id` when :951 is `accept_search_time`
# (the real line is :946).
#
# WHAT IT DOES **NOT** COVER -- still a reviewer's job:
#   * whether the spec's DESIGN is sound, or its threat model complete;
#   * whether a §6 test vector can actually FAIL (it checks the fixture
#     arithmetic, not that an assertion asserts);
#   * prose that is true of the cited line but misleading in context;
#   * anything in the mnemonic-engrave demo (different repo, not checked here);
#   * the behaviour of code that does not exist yet.
# A gate that hides its own blind spot is worse than no gate.
cd /scratch/code/shibboleth/mnemonic-toolkit
S=crates/mnemonic-toolkit/src
T=crates/mnemonic-toolkit/tests
fail=0
chk() { # file line pattern label
  got=$(sed -n "${2}p" "$1")
  if grep -qE "$3" <<<"$got"; then printf '  ok   %-46s %s:%s\n' "$4" "$(basename $1)" "$2"
  else printf '  FAIL %-46s %s:%s\n       expected /%s/\n       got      %s\n' "$4" "$(basename $1)" "$2" "$3" "$got"; fail=1; fi
}
echo "== spec citation gate =="
chk $S/permutation_search.rs 311 'lone spurious match'                  'the +32 rationale'
chk $S/cmd/restore.rs        2123 'SearchOutcome::Ambiguous'            'AMBIGUOUS arm'
chk $S/cmd/restore.rs        2461 'hex::decode'                         'odd-hex decode'
chk $S/permutation_search.rs 1165 'SearchOutcome::Ambiguous'            'short-circuit at match 2'
chk $S/permutation_search.rs 1085 'matches.lock\(\).unwrap\(\).extend'  'per-thread collection'
chk $S/cmd/restore.rs        1951 'compute_wallet_policy_id. never sorts' 'sortedmulti id note'
chk $S/cmd/restore.rs        2270 'calibrate_per_candidate'             'cost calibration'
chk $S/cmd/restore.rs        2009 'validate_prefix_strength'            'prefix validation'
chk $S/cmd/restore.rs        2024 'run_capped_search'                   'id-path search call'
chk $S/cmd/restore.rs        2005 'if id_search'                        'the dispatch'
chk $S/cmd/restore.rs        1943 'let id_search'                       'id_search is_some'
chk $S/cmd/restore.rs        1944 'let addr_search'                     'addr_search is_some'
chk $S/cmd/restore.rs        2225 'PrefixTooShort'                      'PrefixTooShort mapping'
chk $S/error.rs              656  'RestoreMismatch \{ \.\. \} => 4'     'exit code 4'
chk $S/cmd/restore.rs        1472 'fn complete_multisig_template'       'the shared engine'
chk $S/cmd/restore.rs        1207 'struct MultisigCompletionOutcome'    'the outcome type'
chk $S/cmd/restore.rs        1435 'complete_multisig_template\('        'restore call site'
chk $S/cmd/verify_bundle.rs  954  'complete_multisig_template\('        'verify call site'
chk $S/cmd/verify_bundle.rs  946  'expect_wallet_id'                    'verify passes the flag'
chk $T/cli_restore_md1_template_multisig.rs 637  'floor_weak_id_prefix_refuses'            'retired test 1'
chk $T/cli_restore_md1_template_multisig.rs 997  'own_account_max_short_id_prefix_refuses' 'retired test 2'
chk $T/cli_restore_md1_template_multisig.rs 1249 'search_cosigner_subset_weak_prefix_refuses' 'retired test 3'
chk $S/permutation_search.rs 1241 'validate_prefix_strength_rejects_short_accepts_long' 'retired unit test'
chk $S/cmd/restore.rs        2377 'fn emit_completed_multisig'          'the single emitter'
chk $S/cmd/restore.rs        1436 'emit_completed_multisig\('           'its one call site'
chk $S/cmd/restore.rs        1793 'complete_explicit_assignment'        '@N= early return'
chk $S/cmd/restore.rs        220  'pub count: u32'                      '--count flag'
chk $S/cmd/restore.rs        2231 'SearchTimeExceedsCeiling'            'ceiling -> bad()'
chk $S/cmd/restore.rs        1954 'is_order_independent_shape'          'sortedmulti shape flag'
chk $S/permutation_search.rs 848  'pub fn n\('                          'Enumeration::n'
chk $S/permutation_search.rs 862  'pub fn cardinality\('                'Enumeration::cardinality'
chk $S/permutation_search.rs 1128 'pub fn search_reference'             'the determinism oracle'
chk $S/permutation_search.rs 1162 'found.len\(\) >= 2'                   'oracle short-circuits at 2'
chk $S/permutation_search.rs 1066 'SearchMode::Id => 0'                 'id-path address_index is 0'
chk $S/permutation_search.rs 1100 'address_index'                       'engine tie-break order'
chk $S/cmd/restore.rs        2160 'key65'                               'bare-xpub decode'
chk $S/cmd/restore.rs        2083 'distinct subsets'                    'subsets => distinct key SETS'
chk $S/cmd/restore.rs        1951 'id-search is NOT collapsed'          'id-search not collapsed'
chk $S/cmd/verify_bundle.rs  947  'search_address'                      'verify copies search_address'
echo
echo "== numeric gate =="
python3 - <<'PY'
import math
def req(S):
    b=0 if S<=1 else (S-1).bit_length(); return (b+32+7)//8
def P(n,r):
    p=1
    for i in range(r): p*=n-i
    return p
ok=True
def eq(label,got,want):
    global ok
    s = "ok  " if got==want else "FAIL"
    if got!=want: ok=False
    print("  %s %-44s got %s want %s" % (s,label,got,want))
eq("required_prefix_bytes(6)  [demo space]", req(6), 5)
eq("required_prefix_bytes(24)", req(24), 5)
eq("required_prefix_bytes(P(39,11))", req(P(39,11)), 11)
eq("P(39,11)", P(39,11), 66902793897139200)
eq("2^64 / S  (the '1-in-N' figure)", round(2**64/P(39,11)), 276)
eq("S needed for >64 matches at 2 bytes", 64*65536, 4194304)
print("  --- lone-spurious worst case per space ---")
for name,S in [("S=6",6),("S=24",24),("P(39,11)",P(39,11))]:
    r=req(S); best=max(((S*2**(-8*b))*math.exp(-(S*2**(-8*b))),b) for b in range(2,r))
    print("  %-12s worst P(exactly 1 spurious) = %6.2f%% at b=%d" % (name,best[0]*100,best[1]))
raise SystemExit(0 if ok else 1)
PY
nrc=$?
echo
[ $fail -eq 0 ] && [ $nrc -eq 0 ] && echo "GATE: PASS" || { echo "GATE: FAIL"; exit 1; }
