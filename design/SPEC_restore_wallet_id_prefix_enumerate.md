# SPEC — `restore --expect-wallet-id`: auto-enumerate short-prefix matches

**Status:** DRAFT, pre-R0. **Owner:** (this cycle). **Baseline:** mnemonic-toolkit
master `baf288b2` (v0.98.0). **Risk set:** (b) touches wallet reconstruction from
keys, (c) changes the admission contract of an existing flag → **R0 gate to
0C/0I before implementation.**

## 1. Problem

`mnemonic restore --md1 <template> --cosigner … --expect-wallet-id <hex>`
completes a keyless multisig template by searching the cosigner→slot
assignments for the one whose completed **wallet-id** the operator recorded. The
id is 16 bytes (32 hex). Today a prefix **shorter than a space-sized threshold
is REFUSED**:

- `permutation_search::validate_prefix_strength(supplied_bytes, S)` returns
  `PrefixTooShort { required, supplied }` when `supplied < required`, where
  `required = required_prefix_bytes(S) = ceil((log2(S) + 32) / 8)` and `S` is the
  number of assignments in the realized search space (floor 4 bytes; S=2 → 5).
- For the demo's 2-cosigner space `required = 8 bytes = 16 hex`, so `72d94d49`
  (4 bytes) is refused as "prefix too weak".

The refusal exists so the tool never reconstructs **one** wallet from a prefix
that cannot prove uniqueness — the "never a plausible wrong wallet" guarantee.
But an operator who wrote down only the first 4–8 hex of their id is left with a
dead end, even though the true wallet is provably somewhere in a **small**
enumerable set (a handful of cosigners → tens to low-thousands of permutations).

## 2. Decision

**A prefix below the uniqueness threshold ENUMERATES all matching assignments
instead of refusing** (operator ruling 2026-09-17: "auto-enumerate on short
prefix"; no opt-in flag). It **lists**; it never auto-**picks**. The funds-safety
guarantee is preserved by construction: a short prefix that matches >1 assignment
yields a printed *list of candidates*, never a single reconstructed wallet.

Behaviour by match count for the supplied prefix, over the full space scan:

| prefix length | 0 matches | exactly 1 match | ≥2 matches |
| --- | --- | --- | --- |
| **≥ threshold** (unchanged) | `✗ NO MATCH`, exit 4 | reconstruct the wallet, exit 0 | (cannot occur: a ≥-threshold prefix is collision-free by construction) |
| **< threshold** (NEW) | `✗ NO MATCH`, exit 4 | reconstruct the wallet, exit 0 | **list every match**, exit 0, reconstruct NONE |

So the only new path is `< threshold` with ≥2 matches: enumerate. A short prefix
that happens to match exactly one assignment still completes to that wallet
(it is, in fact, unique in this space — the operator got lucky with a short id).

## 3. Semantics (precise)

1. **Floor.** A prefix shorter than **2 bytes (4 hex)** is still refused
   (`PrefixTooShort` retained below the floor) — 4 hex over a thousands-wide
   space would print a wall of matches and teaches nothing. 4 hex is the
   operator's stated low end; make it the floor, not lower.
2. **Full scan, collect ALL.** The engine already scans the whole space and
   classifies `None`/`Unique`/`Ambiguous`, short-circuiting at the SECOND match
   for the `Ambiguous` decision. The change: when the prefix is `< threshold`,
   do NOT short-circuit — collect **every** assignment whose completed wallet-id
   starts with the prefix. (At `≥ threshold`, keep the short-circuit: a second
   match is impossible, so a Unique/None decision is reached without collecting.)
3. **Output, per match** (stable order = assignment permutation index ascending):
   the full 16-byte wallet-id (32 hex), the cosigner→slot assignment (`@0=<fp>,
   @1=<fp>, …`), and the first receive address. A trailing summary line: `N
   assignments match prefix <hex> (of S in the space); none reconstructed —
   re-run with more id or --search-address to pin one`.
4. **Exit codes.** ≥1 match → **0**; 0 matches → **4** (`✗ NO MATCH`, unchanged).
   Never exit 0 with a single silently-chosen wallet from an ambiguous prefix.
5. **Output cap.** Cap the printed list at **64** matches; beyond that, print the
   first 64 and a line `… and K more; supply more id`. (A realized space is a
   handful of cosigners; 64 is generous. The cap bounds a hostile/garbage input.)
6. **Cost ceiling.** The existing `SEARCH_CEILING` / `--accept-search-time`
   forced-acknowledgment is unchanged and still applies before any enumeration:
   a space too large to scan in the ceiling is refused unless acknowledged.
7. **Interaction with `--search-address`.** Address pins the key SET
   (collision-free full-scriptPubKey match); id pins the LABELLING. When both are
   supplied, address filtering runs first, then the id-prefix enumerates within
   that (near-always 1). Unchanged when the id is ≥ threshold.

## 4. Funds-safety analysis

- **No silent wrong wallet.** Enumerate LISTS; it reconstructs a single wallet
  only on a unique match (0 or ≥ threshold-and-unique). An ambiguous short prefix
  produces a list the operator disambiguates — strictly safer than today's
  dead-end refusal (which tempts operators toward `--cosigner @N=` explicit
  placement, the genuinely dangerous "asserted without verifying" path the demo
  warns about).
- **The id is not secret** (it is a public wallet fingerprint), so listing ids +
  addresses leaks nothing spendable. Per-match first addresses are watch-only.
- **`Ambiguous` is no longer an error for a short prefix** — it becomes the
  enumerate result. This RETIRES any test asserting "short prefix → refuse"
  (see `a-new-gate-makes-old-tests-vacuous`): those tests must be re-pointed to
  assert the enumerate list, not deleted silently.

## 5. Changes

- `crates/mnemonic-toolkit/src/permutation_search.rs`: a new outcome
  `Enumerated(Vec<Match>)` (or extend the caller to request all-matches when
  `supplied < required`), collecting every match rather than short-circuiting;
  `validate_prefix_strength` keeps the **2-byte floor** but no longer errors
  between floor and threshold — it signals "enumerate mode" instead.
- `crates/mnemonic-toolkit/src/cmd/restore.rs`: render the list + summary; exit
  0/4 per §3.4; honor the §3.5 cap.
- Tests: see §6.

## 6. Test vectors (must be able to FAIL)

Generated during implementation from a fixed 2- and 3-cosigner template so the
counts are exact; each row is an executable assertion:

1. **Enumerate lists all, reconstructs none.** A 2-byte prefix that matches K≥2
   assignments → stdout lists exactly K wallet-ids, exit 0, and NO
   "wallet completed" line. Mutation: break the collect-all (re-introduce the
   short-circuit) → the list drops to 2 (or 1) → test fails. Proves the scan
   collects all.
2. **Unique short prefix still completes.** A short prefix matching exactly one
   assignment → reconstructs it, exit 0.
3. **Below the floor still refuses.** A 1-byte (2 hex) prefix → `PrefixTooShort`,
   exit 2. Proves the floor holds.
4. **Zero matches → exit 4.** A short prefix matching nothing → `✗ NO MATCH`.
5. **≥ threshold unchanged.** A 16-hex correct id → unique reconstruct; a wrong
   16-hex → NO MATCH. (Pin the existing behaviour so the change doesn't regress
   it.)
6. **Cap.** A synthetic space with >64 matches for a short prefix → prints 64 +
   "and K more"; exit 0.

## 7. Demo implication (mnemonic-engrave `demo/sh2` §3b)

The live page currently states: *"`72d94d49` refused as too weak … it refuses to
guess, and that is the point."* Shipping this **falsifies that claim** and it
MUST be rewritten (see `a-diff-falsifies-text-it-never-touches`): refuse-to-guess
remains the behaviour for **reconstructing a single wallet**, but a short prefix
now **lists candidates** rather than refusing. Reframe 3b to: "a full id (or
address) pins one wallet; a short id lists the handful it could be — and never
silently picks one." This spec does not change the demo; it records the required
follow-up so the two do not drift.

## 8. Open questions for R0

- Is `Enumerated` a new `SearchOutcome` variant, or does the caller pass an
  `all_matches: bool` and reuse the scan? (Prefer the variant: it makes the
  "list vs pick" distinction type-level, so a caller cannot accidentally
  reconstruct from an ambiguous set.)
- Is the 2-byte floor right, or should it be data-driven (e.g. refuse if the
  predicted match count for the prefix exceeds the cap)?
- Should the summary line's "re-run with more id or --search-address" be a
  refusal-grade hint (stderr) or part of stdout? (Lean stderr — stdout stays the
  machine-readable id list.)
- Does any current test assert `PrefixTooShort` between floor and threshold? Grep
  and re-point them (§4 retirement) — enumerate the exact tests in the plan.
