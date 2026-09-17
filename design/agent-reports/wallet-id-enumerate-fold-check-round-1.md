FOLD-CHECK — round 1 on `design/SPEC_restore_wallet_id_prefix_enumerate.md`

Mechanical verification pass. Not a re-audit of the spec's design; scope is
strictly (a) did the fold in `0259da6b` address each finding in
`design/agent-reports/wallet-id-enumerate-spec-r0-round-1.md` (persisted
`fe18508a`), and (b) did the fold introduce a new defect.

Artifacts: `git diff fe18508a..0259da6b -- design/SPEC_restore_wallet_id_prefix_enumerate.md`.
Machine-check: `bash scripts/spec-citation-gate.sh` — re-ran, still PASS (23
citation assertions + 6 arithmetic assertions, all `ok`). Not re-verified by
hand; taken as settled per the dispatch brief.

FOLD: 2 ISSUES

## Disposition table

| id | disposition | where |
| --- | --- | --- |
| C1 | ADDRESSED | §2.1 (operator ruling recorded verbatim), §3.4 (exit/`--json` table: `uniqueness_proven: false` + `warning` field on the lone-match row), §3.8 (mandatory, un-suppressible warning text; "no flag suppresses it"; facts duplicated to `--json`) |
| I1 | ADDRESSED | §3.4: `wallets` key (reconstructed cases) vs `candidates` key with **no** `wallets` key at all (listed case) — the exact escalation-to-Critical path I1 warned about is closed by using disjoint keys |
| I2 | ADDRESSED | §8: explicitly states "re-point them to assert the enumerate list" is not achievable for tests (1)-(3) (each uses `weak = id[..8]` against the true id/true wallet), re-points them to the §3.8 warning path instead; (4) re-pointed to the new signal |
| I3 | ADDRESSED | §6 vector 1: states the draft's fixture (S∈{2,6}) cannot build a ≥2-match case, prescribes `--own-account-max` or a stub evaluator; states the "collect first two, stop" mutant still passes at N≥2 and requires pinning the **exact** count against a known-N fixture (functionally requires N≥3, per the report's I3(b), though "K≥3" is not spelled out as a literal number) |
| I4 | ADDRESSED | §6 vector 6: states the exact required bound (`S > 4,194,304`), states the draft's S∈{2,6} fixtures can never trigger it ("a gate that never runs is a hypothesis, not a gate"), offers two implementation paths (size the fixture, or drive the cap through a seam not requiring a real space) — layer choice is left open but the "unconstructible" defect is fixed |
| I5 | ADDRESSED | §6 vector 5 (new): asserts both sides of the boundary — `required` bytes reconstructs with no warning, `required − 1` takes the warning path |
| I6 | ADDRESSED | §5: names `complete_multisig_template` (`restore.rs:1472`) and `MultisigCompletionOutcome` (`restore.rs:1207`) as the omitted seam; requires the type gain a variant; ties the §5(a) ruling to a compiler-enforced `allow_enumerate: bool` field on `MultisigCompletionCtx` (no `Default`, two exhaustive construction sites) — directly answers "this ruling does not clear I6" from the report |
| I7 | ADDRESSED | §3.7: quotes the real dispatch code, states there is no `conflicts_with`, requires `--expect-wallet-id`/`--search-address` become mutually exclusive; §7 records the downstream demo-page correction | 
| M1 | ADDRESSED | §3.2 rewritten to justify the short-circuit by "the ≥2 case is an error, not a list" (not "impossible"); §8 carries an explicit note on the correction |
| M2 | ADDRESSED | §3.3: explicit sort by `(permutation_index, address_index)`, states arrival order is nondeterministic (per-thread collection), covers address-mode explicitly |
| M3 | ADDRESSED | §3.6: states the refusal moves from instant (`restore.rs:2009`, pre-scan) to post-scan (`SearchTimeExceedsCeiling`), requires that error name the real problem |
| M4 | ADDRESSED | §3.1: states odd-length hex fails earlier via `decode_wallet_id_prefix`/`hex::decode`, out of scope, filed as a follow-up |
| M5 | ADDRESSED | §5: "Do NOT put the floor in `decode_wallet_id_prefix`" — single-sig path named, verified zero calls to `validate_prefix_strength` in that range, keeps the floor on the multisig path only |
| M6 | ADDRESSED | §3.5: "Apply the cap BEFORE deriving addresses" — cites `calibrate_per_candidate` timing the id evaluator only, address derivation cost unbudgeted otherwise |
| M7 | ADDRESSED | §5: "The verify refusal must explain itself" — requires the verify-bundle error text to say restore enumerates and a verifier needs a decisive id |
| N1 | ADDRESSED | §7: "the `bundle` hint quoted in §1 needs its own fix — it prints a 4-byte prefix below the threshold **and** omits `--cosigner` entirely" |
| N2 | ADDRESSED (by deletion) — see NEW-1 | The disputed phrase "(of S in the space)" no longer appears anywhere in §3.3; the whole trailing summary-line sentence was deleted rather than clarified. The literal ambiguity N2 named is gone, but the deletion creates a fresh gap — see NEW-1 below |

Not fully addressed: none of the 16 items — all ADDRESSED, one (N2) with a caveat that produced a new defect (below).

## New defects introduced by the fold

**NEW-1 (Important) — §3.7/§5/§8 still reference a "summary line" / "summary hint" that §3.3 no longer specifies, after the M2 rewrite deleted it.**

The pre-fold §3.3 fully specified a trailing summary line ("`N assignments
match prefix <hex> (of S in the space); none reconstructed — re-run with more
id or --search-address to pin one`"). Responding to M2, the fold rewrote §3.3
end-to-end (sort order, same-wallet marking, "reality check on list length")
and **dropped that sentence entirely** — grep confirms no trace of "of S in
the space", "re-run with more id", or "N assignments match" anywhere in the
current file.

But three other sections still assume it exists:
- §3.7 (line ~197-198): "This spec does not get to leave that ambiguous,
  because **§3.3's own summary line tells the operator** to 're-run with
  `--search-address`'" — present tense, describing content §3.3 does not
  contain. (The only surviving "re-run with --search-address" text is inside
  the §3.8 warning block, which fires only on the *lone-match-reconstruct*
  path, not the *list* path §3.7 is discussing.)
- §5 (line ~258): "render the list **+ summary**" — still requires
  implementing a summary whose wording is now specified nowhere.
- §8 (line ~372-373): "stdout or stderr for the **summary hint**? stderr" —
  resolves the channel for a hint whose content is undefined.

Net effect: the operator-facing text for the ≥2-match **list** outcome — the
central new behaviour this spec introduces — has no specified wording
anywhere in the current draft, while three sections still talk about it as a
settled, existing artifact. This is worse than the original N2 finding (which
was "which S is named" ambiguity in existing text): the text itself is gone.

Does not block re-implementation of anything already prescribed elsewhere
(exit codes / `--json` keys in §3.4 are unaffected and are what tests bind
to), but a fold responding to this needs to either restore a summary-line
spec in §3.3 or drop the "summary"/"hint" language from §5/§7/§8.

**NEW-2 (Minor) — §3.3's new "Reality check on list length" cites two
inconsistent figures in the same sentence.**

New text: *"a second match needs a ~2⁻³² collision, so the list is almost
always exactly one row (measured 0.01% for S=6 at the 2-byte floor)."*

`2^-32 ≈ 2.33e-10`. The actual figure for a second match at the 2-byte
(16-bit) floor with S=6 is `(S-1)·2^-16 ≈ 7.63e-5 ≈ 0.0076%` — matching the
report's own I3(a) computation (`5·2^-16 = 7.6e-5`) and rounding to the "0.01%"
the same sentence quotes. The two numbers in this one sentence differ by a
factor of ~**327,680** (verified: `7.63e-5 / 2.33e-10 ≈ 3.28e5`). The "~2⁻³²"
figure appears to be borrowed from `required_prefix_bytes`'s false-positive
design bound (which applies **at the `required`/threshold length**, not at
the 2-byte floor this sentence is about) and does not belong in this
sentence. This text is new in the fold (not present in the pre-fold draft,
not requested by any report item) and is not covered by the citation/numeric
gate (the gate does not assert this specific inline figure).

The practical conclusion ("almost always one row" for exact-pool spaces) is
still correct — it just isn't supported by the number given. Non-normative
exposition; no test or requirement depends on the wrong figure.

## Not independently re-derived

Per the brief: the citation gate's 23 file:line assertions and 6 arithmetic
assertions (including `2^64/S = 1/276`, `P(39,11)`, `required_prefix_bytes`
values, and the cap threshold `S > 4,194,304`) were re-run, not hand-checked,
and came back PASS. The three report-flagged wrong citations
(`1/272`→`1/276`, `restore.rs:2090`→`:2024`, `calibrate_per_candidate`
`:2273`→`:2270`) all appear corrected in the fold and were not re-litigated
here.
