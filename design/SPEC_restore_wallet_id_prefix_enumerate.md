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
- For the demo's space (own seed + 2 `--cosigner` cards = 3 slots) the
  **measured** `required` is **5 bytes (10 hex)** — not 8. Verbatim from
  `mnemonic 0.98.0`: `prefix too weak for this search: need ≥5 bytes (sized to
  the realized search space), got 4`. So `72d94d49` (4 bytes) is refused, and
  a 10-hex prefix would already be accepted. (8 bytes needs `S > 2^24`; the
  pinned ladder reaches it only at `S = 11! = 39,916,800`.)

The refusal exists so the tool never reconstructs **one** wallet from a prefix
that cannot prove uniqueness — the "never a plausible wrong wallet" guarantee.
But an operator who wrote down only the first 4–8 hex of their id is left with a
dead end, even though the true wallet is provably somewhere in a **small**
enumerable set (a handful of cosigners → tens to low-thousands of permutations).

**The tool advertises a recipe it then refuses.** This is the concrete
motivation, not a hypothetical operator. `bundle --md1-form template` closes
with (measured, a 2-of-3 `wsh-sortedmulti` template):

```text
        wallet-id (hex):    e1ccd788febf8ba06b21d98a846f89bd
        wallet-id (prefix): e1ccd788  (4-byte convenience prefix)
        restore with: restore --md1 <template> --from <seed> --account 0 [--expect-wallet-id e1ccd788]
```

That printed 4-byte prefix — offered by name as the thing to record with the
engraving — is **below** the 5-byte threshold the very next command enforces.
An operator who records exactly what the tool told them to record gets a
refusal. Enumerating is what makes the tool's own advice usable.

## 2. Decision

**A prefix below the uniqueness threshold ENUMERATES all matching assignments
instead of refusing** (operator ruling 2026-09-17: "auto-enumerate on short
prefix"; no opt-in flag). It **lists**; it never auto-**picks**. The funds-safety
guarantee is preserved by construction: a short prefix that matches >1 assignment
yields a printed *list of candidates*, never a single reconstructed wallet.

Behaviour by match count for the supplied prefix, over the full space scan:

| prefix length | 0 matches | exactly 1 match | ≥2 matches |
| --- | --- | --- | --- |
| **≥ threshold** (unchanged) | `✗ NO MATCH`, exit 4 | reconstruct the wallet, exit 0 | `✗ AMBIGUOUS`, exit 1 (unchanged — see below) |
| **< threshold** (NEW) | `✗ NO MATCH`, exit 4 | reconstruct the wallet, exit 0 | **list every match**, exit 0, reconstruct NONE |

So the only new path is `< threshold` with ≥2 matches: enumerate. A short prefix
that happens to match exactly one assignment still completes to that wallet
(it is, in fact, unique in this space — the operator got lucky with a short id).

**`Ambiguous` at ≥ threshold is reachable and must stay an error.** An earlier
draft of this table claimed that cell "cannot occur … collision-free by
construction". That is false, and the code disagrees: `SearchOutcome::Ambiguous`
is an explicit arm with no prefix-length condition, and
`cmd/restore.rs:2123` handles it by printing `✗ AMBIGUOUS` and returning
`BadInput` (**exit 1**). `required_prefix_bytes`'s own doc sizes the prefix to a
false-positive probability of **≤ ~2e-10**, which is a bound, not impossibility.
This spec does **not** touch that path: a ≥-threshold prefix that somehow
matches twice keeps today's refusal. Enumeration is opt-in-by-shortness, and a
long prefix must never silently downgrade into a list.

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
- **`crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` — a SECOND CLI surface
  this change lands on, and the one open design question.** `verify_multisig_
  template` builds a `MultisigCompletionCtx` carrying
  `expect_wallet_id: args.expect_wallet_id.clone()` (line 951) and calls the
  **same** `complete_multisig_template` engine restore uses — its own doc comment
  says the wallet is recomposed "via the IDENTICAL engine restore emits with
  (funds-safety parity)". So `verify-bundle --expect-wallet-id <short>` changes
  behaviour whether or not this spec mentions it.
  **`verify-bundle` is a VERIFIER: a list is not a verdict.** It must not report
  PASS on an enumerate outcome — "these 6 wallets are consistent with what you
  engraved" is not verification, and a PASS there would be precisely the silent
  funds-safety regression §4 exists to prevent. R0 must rule between:
  (a) `verify-bundle` keeps today's `PrefixTooShort` refusal (enumerate is
  restore-only — the engine takes an explicit `allow_enumerate` the verify
  caller passes `false`); or (b) it enumerates and reports FAIL/inconclusive
  with the list as context. **Default recommendation: (a)** — it is the smaller
  blast radius and keeps the verifier's contract binary. Either way the engine
  change MUST NOT default `verify-bundle` into enumeration by inheritance.
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
   **exit 4**. Proves the floor holds. (Measured: `PrefixTooShort` is mapped at
   `cmd/restore.rs:2225` to `ToolkitError::RestoreMismatch`, which
   `error.rs:656` pins to **4**. An earlier draft said exit 2; a test asserting
   2 would fail as written.)
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

**A second, independent demo defect found by the §9 gate — already wrong today,
before this ships.** Both the page and `CLI_SPINE.md` present the ladder as
`72d9` refused / `72d94d49` refused / **`72d94d49b0aca695` (16 hex) accepted**,
which reads as "16 hex is the threshold". The measured threshold for that space
is **5 bytes = 10 hex**. The rows are each true, but the set implies a wrong
number, and the page's "it sizes the search space and demands enough identifier"
invites exactly that inference. The 3b rewrite should state the real rule: the
tool sizes the requirement to the space — for a 3-slot wallet that is 10 hex,
and it tells you the number it wants. File this as a demo follow-up whether or
not this feature ships; it is not contingent on it.

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
- ~~Does any current test assert `PrefixTooShort` between floor and threshold?~~
  **ANSWERED by grep — 4 tests, every one of them using a 4-byte prefix, which
  falls INSIDE the proposed enumerate band (floor 2 ≤ 4 < required):**
  1. `tests/cli_restore_md1_template_multisig.rs:637 floor_weak_id_prefix_refuses`
     — `n!` space, 4-byte prefix.
  2. `tests/cli_restore_md1_template_multisig.rs:997 own_account_max_short_id_prefix_refuses`
     — `--own-account-max 32` space, 4-byte prefix.
  3. `tests/cli_restore_md1_template_multisig.rs:1249 search_cosigner_subset_weak_prefix_refuses`
     — `--search-cosigner-subset` space, 4-byte prefix.
  4. `src/permutation_search.rs:1241 validate_prefix_strength_rejects_short_accepts_long`
     — **unit** test asserting `validate_prefix_strength(4, 11!) ==
     Err(PrefixTooShort { required: 8, supplied: 4 })`. §5 removes that error in
     this band, so this one must be re-pointed too, not just the CLI three.

  All four assert only `.failure()` + a stderr substring (`prefix`/`weak`/
  `bytes`); none pins an exit code. Re-point each to assert the enumerate list,
  per the §4 retirement rule — and note (1)-(3) may instead survive unchanged if
  R0 picks §5 option (a) for a verify-only refusal, since they are `restore`
  tests. Decide in the plan, do not leave them green-but-vacuous.

## 9. Already machine-checked — do NOT re-derive (build gate, pre-R0)

Run against toolkit master `baf288b2` with the binary `mnemonic 0.98.0`
(`target/debug/mnemonic`), 2026-09-17. Reviewer budget should go to design,
threat model, and §5's `verify-bundle` ruling — not to re-measuring these.

| # | Checked | Result |
| --- | --- | --- |
| 1 | `permutation_search.rs`, `cmd/restore.rs` exist at the cited paths | ✓ (2429 / 3854 lines) |
| 2 | `required_prefix_bytes(S) = ceil((log2 S + 32)/8)`, floor 4 at `S≤1`, `S=2 → 5` | ✓ pinned at `permutation_search.rs:1234-1237` |
| 3 | `required` for the demo-shaped space (own + 2 cosigners) | **5 bytes**, measured end-to-end (§1) — the draft's "8 bytes" was wrong |
| 4 | Exit code for `PrefixTooShort` | **4**, measured at 1/2/4-byte prefixes; chain `restore.rs:2225` → `error.rs:656` |
| 5 | Exit code for `✗ NO MATCH` | **4**, measured (`RestoreMismatch`) — §3.4 correct as written |
| 6 | Exit code for `✗ AMBIGUOUS` | **1**, `restore.rs:2123` → `bad()` → `BadInput` → `error.rs:1096` |
| 7 | Engine classifies `None`/`Unique`/`Ambiguous`, short-circuits at the SECOND match | ✓ `permutation_search.rs:1163-1165`, doc lines 20-27 — §3.2's premise holds |
| 8 | `SEARCH_CEILING` = 1h, `--accept-search-time`, `--search-address` all exist | ✓ `permutation_search.rs:59`, `:391`, `verify_bundle.rs:137` |
| 9 | Which tests assert the refusal in the enumerate band | **4**, enumerated in §8 |
| 10 | `verify-bundle` shares the completion engine | ✓ `verify_bundle.rs:951` — §5 |
| 11 | `bundle` advertises a 4-byte prefix `restore` refuses | ✓ measured, quoted verbatim in §1 |

**NOT covered by this gate** (still a reviewer's job): whether the enumerate
output is legible to an operator mid-recovery; whether the 64 cap and 2-byte
floor are the right numbers; the §5 `verify-bundle` ruling; whether a test as
specified in §6 can actually fail; and the §7 demo rewrite's accuracy.
