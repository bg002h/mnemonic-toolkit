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
prefix"; no opt-in flag). It **lists**; and where it still reconstructs, it says
loudly that it could not prove uniqueness.

Behaviour by match count, over the full space scan:

| prefix length | 0 matches | exactly 1 match | ≥2 matches |
| --- | --- | --- | --- |
| **< floor** (< 2 B) | `PrefixTooShort`, exit 4 | — | — |
| **< threshold** (NEW) | `✗ NO MATCH`, exit 4 | reconstruct **+ mandatory §3.8 warning**, exit 0 | **list every match**, NO descriptor, exit 0 |
| **≥ threshold** (unchanged) | `✗ NO MATCH`, exit 4 | reconstruct, exit 0 | `✗ AMBIGUOUS`, exit 1 |

**`Ambiguous` at ≥ threshold is reachable and stays an error.** An earlier draft
claimed that cell "cannot occur … collision-free by construction". The code
disagrees: `SearchOutcome::Ambiguous` has no prefix-length condition, and
`cmd/restore.rs:2123` prints `✗ AMBIGUOUS` and returns `BadInput` (**exit 1**).
`required_prefix_bytes`'s own doc sizes the prefix to a false-positive
probability of **≤ ~2e-10** — a bound, not impossibility. This spec does not
touch that path: a long prefix must never silently downgrade into a list.

### 2.1 The lone-match cell — the operator ruling, and what it costs

R0 round 1 raised this as **C1**, and it is the sharpest point in the design.
The threshold being relaxed was never an ambiguity guard. `permutation_search.rs`
lines 310-313 say what it is for, verbatim:

> The `+32` term keeps the false-positive probability of a **lone spurious
> match** ≤ ~2e-10 across the realized range; a fixed 8-byte prefix would hit
> ~1-in-275 at `K=32` — so the prefix MUST size from `S`.

So "short prefix, exactly one match" is **precisely the case the threshold
existed to prevent**. "It is, in fact, unique in this space" confuses *the only
match I found* with *the right wallet*: if the true wallet is NOT in the space —
the operator supplied a wrong cosigner card, or misread a digit off the plate —
a short prefix can still produce exactly one **spurious** match.

Measured worst case over the enumerate band (λ = S·2⁻⁸ᵇ, P(exactly one spurious)
= λe^−λ, maximised at λ=1):

| space | S | required | worst lone-spurious P |
| --- | --- | --- | --- |
| 3 slots, `--account 0` (the demo) | 6 | 5 B | **0.01%** |
| 4 slots, `--account 0` | 24 | 5 B | 0.04% |
| `--own-account-max 32` (n=11, own=4) | 66,902,793,897,139,200 | 11 B | **36.69%** at b=7 |

The risk is **entirely space-dependent**: negligible for the exact-pool spaces
this feature is pitched at, severe for the large over-supply modes. (R0's report
quotes `1/272` for the 8-byte figure; recomputed it is `2⁶⁴/S = 1/276`, and the
source doc's "~1-in-275" is right to rounding. No conclusion changes.)

**OPERATOR RULING 2026-09-17, after being shown the table above and three
alternatives (show-only / warn / split-by-space-size): _"Emit it, with a loud
warning."_** The lone-match cell keeps `exit 0` and keeps emitting a descriptor.
The concern was raised and overruled; this section exists so the trade is on the
record rather than rediscovered.

Two consequences follow and are binding:

- The warning is **mandatory and un-suppressible** (§3.8) — not an advisory, and
  there is no flag to silence it.
- It **must reach the machine-readable channel too**. A stderr warning is
  invisible to a script reading `.wallets[0].descriptor`, which is the exact
  consumer most likely to act on a wrong wallet unattended. §3.4 therefore puts
  the same fact in the `--json` envelope.

## 3. Semantics (precise)

1. **Floor.** A prefix shorter than **2 bytes (4 hex)** is still refused
   (`PrefixTooShort`, **exit 4**). 4 hex over a wide space prints a wall of
   matches and teaches nothing.
   **Odd-length hex never reaches any of this** (R0 M4): `decode_wallet_id_prefix`
   (`restore.rs:2461`) uses `hex::decode`, so 5, 7 or 9 hex characters fail
   earlier with *"must be an even-length hex prefix"* — which is what a smudged
   plate actually produces. Out of scope here; filed as a follow-up.
2. **Full scan, collect ALL.** The engine scans the whole space and classifies
   `None`/`Unique`/`Ambiguous`, short-circuiting at the SECOND match for the
   `Ambiguous` decision (`permutation_search.rs:1163-1165`). The change: when the
   prefix is `< threshold`, do NOT short-circuit — collect **every** match. At
   `≥ threshold` keep the short-circuit (the ≥2 case is an error, not a list).
3. **Output, per match.** The full 16-byte wallet-id (32 hex), the cosigner→slot
   assignment (`@0=<fp>, @1=<fp>, …`), and the first receive address.
   **Order requires an explicit sort** (R0 M2): the engine assembles matches
   per-thread (`matches.lock().unwrap().extend(local)`,
   `permutation_search.rs:1085-1087`), so arrival order is nondeterministic. Sort
   by `(permutation_index, address_index)` — the pair, not the permutation alone,
   since an address-mode match carries both. Any test asserting order is flaky
   until both are pinned.
   **Classification is by MATCH COUNT, never by distinct wallets** (L4).
   Two matches that happen to be the same wallet are still two matches, and
   still list. Grouping is a **display annotation only** and never changes
   which row of §2's table applies — otherwise "2 matches, one wallet" would
   reconstruct under one reading and list under another, and §6 vector 1's
   "exactly N rows" would be unwritable.
   **Rows that are the same wallet MUST be marked as such.** For a
   `sortedmulti`/`sortedmulti_a` shape, `compute_wallet_policy_id` never sorts
   (`restore.rs:1951-1953`), so each ordering has a *different id* — but every
   ordering yields *identical addresses and identical spending*. A list can
   therefore show N rows that are one wallet under N labellings. The annotation is
   **one line above the rows, derived from the SHAPE FLAG and not from comparing
   addresses** (L5): `is_order_independent_shape(&d.tree)` (`restore.rs:1954`)
   already tells the renderer that every match is the same wallet under a
   different labelling. Grouping by *derived address* would contradict §3.5's
   "cap before address derivation"; reading the flag does not. **Never merge
   rows — annotate the set.** Unmarked, the operator reads "N wallets it could be" when the
   honest answer is "1 wallet, N labellings" — and for an unsorted `wsh-multi`
   the same display means something entirely different.
   **Summary line (the list outcome's operator-facing text).** After the rows,
   on **stderr** (§8), emit exactly one line naming the count, the prefix, the
   realized space, and the way out:

   ```text
   N assignments match prefix <hex> (of S in the realized space); none
   reconstructed — supply more id, or re-run with --search-address instead.
   ```

   `S` is the **realized** cardinality — `n!`, `s_own` or `s_opt` depending on
   flags (R0 N2) — and the line must name which, since the whole prefix-sizing
   story hangs on that number. "instead" is load-bearing: per §3.7 the two flags
   become mutually exclusive, so the hint must not read as "add `--search-address`
   to what you just ran", which would refuse.
   This line is what §3.7, §5 and §8 refer to as "the summary"/"the hint".

   **Reality check on list length**: for the exact-pool spaces this targets the
   list is **almost always exactly one row**. A second match needs a collision
   at the supplied prefix length, and the odds scale with it — for S=6 the
   chance any other assignment also matches is ~7.6e-5 at the 2-byte floor and
   ~1.2e-9 at 4 bytes. The multi-row list is a large-space phenomenon; write
   §3.3's copy for the one-row case first.
4. **Exit codes and `--json`** (R0 I1). Text and JSON must agree, and `exit 0`
   must never be the only signal distinguishing a wallet from a list:

   | outcome | exit | `--json` |
   | --- | --- | --- |
   | reconstructed, prefix ≥ threshold | 0 | `wallets: [ … ]`, `uniqueness_proven: true` |
   | reconstructed, prefix < threshold (lone match) | 0 | `wallets: [ … ]`, **`uniqueness_proven: false`**, `prefix_bytes`, `required_bytes`, `warning: "<the §3.8 text>"` |
   | listed (≥2 matches, < threshold) | 0 | **`candidates: [ … ]` and NO `wallets` key at all** |
   | no match | 4 | unchanged |
   | ambiguous at ≥ threshold | 1 | unchanged |
   | below floor | 4 | unchanged |

   The `candidates` key is load-bearing: reusing `wallets` would make
   `.wallets[0].descriptor` silently become candidate #1 for every existing
   script. A consumer that has never heard of `candidates` gets a missing key
   and fails loudly, which is the desired behaviour.
### 3.4a Output contract — the section whose absence caused six findings

The implementability lens (`design/agent-reports/wallet-id-enumerate-implementability-lens.md`)
found that L1, L2, L3, L6, L8 and L9 all reduce to one omission: the spec named
*what to decide* without ever fixing *what to emit*. This section is normative
and takes precedence over any looser phrasing elsewhere.

**(a) `uniqueness_proven` is scoped to the ID-SEARCH path only (L1, Critical).**
`emit_completed_multisig` (`restore.rs:2377`) is reached from **three**
completion modes through a single call site (`restore.rs:1436`): id-search,
address-search, and explicit `--cosigner @N=` placement, which returns early via
`complete_explicit_assignment` (`restore.rs:1793`). `MultisigCompletionOutcome`
carries **no mode discriminant**, so the naive implementation stamps
`uniqueness_proven: true` on all three.

That is forbidden. Explicit `@N=` placement proves nothing — its own warning
says a wrong assignment produces a wrong wallet **silently** — so claiming
uniqueness there would attach a funds-safety assertion to the one mode that
cannot support it, which is worse than emitting nothing.

Required: the outcome type gains the mode, and the field is emitted **only** on
the id-search path:

| completion mode | `uniqueness_proven` |
| --- | --- |
| id-search, prefix ≥ threshold, unique | `true` |
| id-search, prefix < threshold, lone match | `false` (+ `prefix_bytes`, `required_bytes`, `warning`) |
| address-search | **key absent** (a scriptPubKey match is collision-free but is not a prefix claim) |
| explicit `--cosigner @N=` | **key absent** |

A consumer must never read absence as `true`. If a future cycle wants a positive
signal for the other modes it needs its own field name and its own justification.

**(b) Stream assignment — every line is placed (L3).**

| line | text mode | `--json` |
| --- | --- | --- |
| candidate rows | **stdout** (they are the payload) | in `candidates[]` |
| the sortedmulti annotation | stderr | `order_independent: true` |
| the §3.3 summary line | stderr | mirrored by the fields below |
| the §3.8 warning | stderr | `warning` |
| a reconstructed wallet | stdout (unchanged) | `wallets[]` (unchanged) |

Rationale: stdout carries what a script consumes and stderr carries what a human
reads, which is the convention the rest of the command already follows. An empty
stdout with exit 0 — the alternative reading — would be indistinguishable from
success-with-no-output.

**(c) `candidates[]` shape, and the sub-fork that matters (L8).**

```json
{ "network": "mainnet",
  "completed_from": "multisig-template-md1",
  "prefix_hex": "e1cc", "prefix_bytes": 2, "required_bytes": 5,
  "realized_space": 6, "realized_space_kind": "n_factorial",
  "match_count": 3, "order_independent": true,
  "truncated": false,
  "candidates": [
    { "wallet_policy_id": "e1ccd788febf8ba06b21d98a846f89bd",
      "assignment": [ {"slot": 0, "fingerprint": "b8688df1"},
                      {"slot": 1, "fingerprint": "28645006"},
                      {"slot": 2, "fingerprint": "3f635a63"} ],
      "first_address": "bc1q…" } ]
}
```

**No `candidates[].descriptor` field, ever.** That is the funds-relevant fork:
a descriptor in the list would make the list importable, which is exactly the
"reconstructs nothing" property §4 rests on. A consumer that wants a descriptor
must re-run with enough id to reconstruct one.
`wallets` MUST be absent whenever `candidates` is present, and vice versa (§3.4).

**(d) Truncation is explicit in BOTH channels (L2, Critical).** §3.5's cap of 64
applies to **rows, not groups** — grouping is annotation only, per §3.3. When it
bites, text mode prints `… and K more; supply more id` and JSON sets
`"truncated": true` with `match_count` carrying the **true total**, not the
truncated length. Silent JSON truncation is the failure this closes: a consumer
seeing 64 candidates and no marker concludes the wallet is not among its keys
when it was #65 — a wrong answer delivered as a complete one.

**(e) `--count` applies to the reconstructed wallet only (L6).** `--count`
(`restore.rs:220`, "Number of first-receive addresses to show per wallet type",
default 1) already reaches the emitter via `args: &RestoreArgs`. A candidate row
shows **exactly one** address regardless of `--count`; only the reconstructed-
wallet path honours it. Otherwise `--count 5` over 64 candidates derives 320
addresses — unbudgeted work outside the ceiling (§3.5), for a list nobody reads
five-deep.

**(f) The ceiling case belongs in the exit table (L7).** A sub-threshold prefix
over a space too large to scan no longer refuses instantly (§3.6): it reaches
`SearchTimeExceedsCeiling`, which `restore.rs:2231` maps through `bad()` to
`BadInput` — **exit 1**, not 4. So the same operator input that exits 4 today
exits 1 after this change:

| case | today | after |
| --- | --- | --- |
| short prefix, space within ceiling | 4 (`PrefixTooShort`) | 0 (list or warn-and-reconstruct) |
| short prefix, space over ceiling | 4 (`PrefixTooShort`) | **1** (`SearchTimeExceedsCeiling`) |

Scripts keying on 4 will see 1. The message must name the prefix length as well
as the time, or the operator is told about seconds when their problem is digits.

**(g) The summary line needs the cardinality KIND, which has no accessor (L9).**
§3.3 requires the line to name which of `n!` / `s_own` / `s_opt` it printed, but
`Enumeration` exposes only `n()` (`permutation_search.rs:848`) and
`cardinality()` (`:862`) — the *kind* must be added. The template therefore has
a slot for it, and "emit exactly one line" means one line **after**
substitution:

```text
N assignments match prefix <hex> (of S <kind> in the realized space); none
reconstructed — supply more id, or re-run with --search-address instead.
```

with `<kind>` ∈ {`permutations`, `own-anchored subsets`, `opt-in subsets`},
mapping 1:1 onto `Enumeration`'s three variants.

5. **Output cap.** Cap the printed list at **64** matches; beyond that print the
   first 64 and `… and K more; supply more id`.
   **Apply the cap BEFORE deriving addresses** (R0 M6). `calibrate_per_candidate`
   (`restore.rs:2270`) times the **id** evaluator only; rendering a first receive
   address costs a descriptor build + miniscript parse + `script_pubkey_at`,
   materially more, and is paid *after* the ceiling decision. Deriving addresses
   for every match and then truncating puts unbudgeted work outside the ceiling.
6. **Cost ceiling.** `SEARCH_CEILING` / `--accept-search-time` are unchanged and
   still apply. Verified sound by R0: `cap_decision`'s estimate is already the
   exhaustive full-space worst case with no early-terminate credit, so removing
   the short-circuit cannot exceed what was budgeted.
   **But the refusal moves from instant to post-scan** (R0 M3). Today
   `validate_prefix_strength` (`restore.rs:2009`) refuses a short prefix in
   milliseconds, naming the bytes needed, *before* `run_capped_search`
   (`restore.rs:2024`). After this change the operator instead enters calibration
   and, on a large space, meets `SearchTimeExceedsCeiling` — a message about
   *time*, for a problem about *prefix length*. That error must say so.
7. **Interaction with `--search-address` — the spec previously described
   something that does not exist** (R0 I7). The real dispatch (`restore.rs:2005`)
   is:

   ```text
   let id_search   = ctx.expect_wallet_id.is_some();   // :1943
   let addr_search = ctx.search_address.is_some();     // :1944
   let outcome = if id_search { … } else if addr_search { … } else { refuse };
   ```

   There is no `conflicts_with` on either flag, so **supplying both silently
   ignores the address**. The draft's "address filtering runs first, then the id
   prefix enumerates within that" is not implemented anywhere.
   This spec does not get to leave that ambiguous, because §3.3's own summary
   line tells the operator to "re-run with `--search-address`" — advice that is a
   **no-op if they leave the id in place**, which is exactly what someone
   re-running a previous command does. Required: **`--expect-wallet-id` and
   `--search-address` become mutually exclusive** (clap `conflicts_with`), so the
   combination refuses with a message naming which to drop. Honouring both is the
   larger change and is NOT in this cycle's scope; refusing is.
   *(Already actioned downstream: the live demo taught "USE BOTH TOGETHER"; that
   text was corrected on 2026-09-17 — see §7.)*
8. **The mandatory warning (the C1 ruling).** On the lone-match-below-threshold
   path the tool MUST emit, before the wallet block and unconditionally:

   ```text
   ! UNIQUENESS NOT PROVEN — the supplied --expect-wallet-id is 4 bytes; this
   ! search space needs 5 to rule out a coincidental match. Exactly one
   ! assignment matched, and it is reported below, but a match this short can
   ! be spurious when the true wallet is NOT among the keys you supplied.
   ! Before receiving to this wallet, confirm the first address below against
   ! a source you already trust, or re-run with --search-address.
   ```

   Requirements: no flag suppresses it; it names the supplied and required byte
   counts (both are already in hand at `restore.rs:2009`); and the same facts
   appear in `--json` per §3.4. It is a warning, not an error — exit stays 0 per
   the operator ruling.

## 4. Funds-safety analysis

- **The guarantee, stated honestly.** "Never a plausible wrong wallet" holds
  unconditionally for the *list* path (it reconstructs nothing) and for every
  `≥ threshold` path. It does **not** hold unconditionally for the
  lone-match-below-threshold path: that path trades a measured residual risk
  (§2.1 — 0.01% for exact-pool spaces, up to 36.69% for `--own-account-max`)
  for usability, by operator ruling, mitigated by the §3.8 warning in both
  output channels. Any documentation claiming the guarantee is absolute must be
  reworded, not left to imply the old absolute form.
- **Strictly better than today's dead end where it lists.** A refusal tempts
  operators toward `--cosigner @N=` explicit placement — the genuinely dangerous
  "asserted without verifying" path the demo warns about. A candidate list is a
  safer off-ramp than a refusal that teaches nothing.
- **The id is not secret** (it is a public wallet fingerprint), so listing ids +
  addresses leaks nothing spendable. Per-match first addresses are watch-only.
- **Retirement.** `Ambiguous` ceases to be an error for a short prefix. Per
  `a-new-gate-makes-old-tests-vacuous`, every test asserting "short prefix →
  refuse" is retired by this change and must be re-pointed, not left green. §6
  and §8 enumerate them.

## 5. Changes

- `crates/mnemonic-toolkit/src/permutation_search.rs`: collect-all below
  threshold; `validate_prefix_strength` keeps the 2-byte floor but signals
  "enumerate mode" between floor and threshold instead of erroring.
- **`complete_multisig_template` (`restore.rs:1472`) — the function that actually
  makes the decision, omitted by the draft** (R0 I6). Both surfaces call it, and
  it returns `MultisigCompletionOutcome` (`restore.rs:1207`), whose three fields
  are `completed: md_codec::Descriptor`, `pool`, `assignment` — a **single**
  descriptor and a **single** assignment. The type **structurally cannot express
  "listed N candidates, reconstructed none"**, so it must gain a variant (or
  become an enum). **Make it an enum, not an added field** — the two refactors
  are not equivalent (L1's neighbouring fork): a `Vec<Match>` field alongside
  `completed` leaves "both populated" and "neither populated" representable, so
  the compiler cannot stop a caller from emitting a descriptor for a list. An
  enum makes list-vs-reconstruct type-level, which IS §4's funds-safety
  argument.
  **It must ALSO carry the completion MODE** (§3.4a(a), L1 Critical): one
  emitter (`restore.rs:2377`) serves id-search, address-search and explicit
  `@N=` placement through a single call site (`restore.rs:1436`), and
  `uniqueness_proven` is meaningful only for the first. With no discriminant on
  the outcome the emitter cannot tell them apart, and the naive implementation
  stamps `uniqueness_proven: true` on the one mode that proves nothing. That type change IS the seam where verify-bundle inheritance
  is decided; without it there is no seam and §5's own requirement below is
  unenforceable.
- `crates/mnemonic-toolkit/src/cmd/restore.rs`: render the list + summary; the
  §3.8 warning; exit codes and the `candidates` envelope per §3.4; the §3.5 cap
  before address derivation; `conflicts_with` per §3.7.
- **`crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` — RULING: option (a),
  verify keeps refusing.** It calls the same engine
  (`verify_bundle.rs:954`) with the same `expect_wallet_id` (`:946`); its own doc
  says the wallet is recomposed "via the IDENTICAL engine restore emits with
  (funds-safety parity)". A verifier must not report PASS on a list — "these 6
  wallets are consistent with what you engraved" is not verification. R0 concurs
  and adds the decisive argument: verify refuses *before* calibration today, so
  enumerating would make it pay a full scan to answer "inconclusive".
  Implementation: `MultisigCompletionCtx` has **no `Default`** and exactly two
  construction sites (`restore.rs:1435`, `verify_bundle.rs:954`), both exhaustive
  struct literals — so adding an `allow_enumerate: bool` forces the compiler to
  make every caller choose. `restore` passes `true`; `verify-bundle` passes
  `false`. Inheritance-by-omission is impossible by construction, which is the
  point.
- **The verify refusal must explain itself** (R0 M7). After this lands,
  `restore --expect-wallet-id <6 hex>` lists while `verify-bundle
  --expect-wallet-id <6 hex>` refuses "prefix too weak" — same flag, same engine,
  opposite answers, minutes apart in the demo's own running order. The verify
  message must say that `restore` enumerates and that a *verifier* requires a
  decisive id, or it reads as a bug.
- **Do NOT put the floor in `decode_wallet_id_prefix`** (R0 M5). The **single-sig**
  template path (`restore.rs:1010-1053`) accepts any prefix length with only an
  advisory below 4 bytes and never calls `validate_prefix_strength` (verified:
  zero occurrences in `restore.rs:900-1100`). Implementing the new floor in the
  shared decoder would silently change single-sig behaviour too. Keep it in
  `validate_prefix_strength`, on the multisig path.
- Tests: see §6.

## 6. Test vectors (must be able to FAIL)

R0 found the draft's vectors unbuildable or vacuous (I3, I4, I5). **Fixture
sizing is part of the vector**, because three of these cases do not exist in a
small space:

| # | asserts | fixture it REQUIRES |
| --- | --- | --- |
| 1 | collect-all: N matches listed, no descriptor | a space with a **real** ≥2-match set |
| 2 | lone match reconstructs **and warns** | any exact-pool space, true wallet present |
| 3 | below floor refuses, exit 4 | any |
| 4 | zero matches → `✗ NO MATCH`, exit 4 | any |
| 5 | `supplied == required` reconstructs, never lists | any — **the off-by-one guard** |
| 6 | cap at 64 | S > 4,194,304 with a 2-byte prefix |
| 7 | `uniqueness_proven` absent on `@N=` and address modes (§3.4a(a)) | any — three invocations |
| 8 | `candidates[]` carries NO `descriptor`, `wallets` absent (§3.4a(c)) | the vector-1 fixture |
| 9 | truncation sets `truncated: true` + a TRUE `match_count` (§3.4a(d)) | the vector-6 fixture |

1. **Enumerate lists all, reconstructs none.** The draft's fixture cannot build
   this: a 2-/3-cosigner space needs a genuine id collision (P ≈ 7.6e-5 at the 2-byte floor; ~1.2e-9 at 4 bytes) to get a
   second match. Use a space large enough that ≥2 matches occur *by construction*
   (`--own-account-max` widens S without more cards), or inject a stub evaluator.
   Assert **exactly N rows and the absence of any descriptor / `wallets` key**.
   The draft's mutation ("re-introduce the short-circuit") is **not sufficient at
   N≥2**: a "collect the first two, then stop" mutant still yields ≥2 rows and
   passes. Pin the exact count against a known-N fixture, or the vector does not
   prove collect-all.
2. **Lone short prefix reconstructs, and the warning fires.** Two assertions, and
   the second is the one that matters: the descriptor is emitted **and** the §3.8
   warning text appears on stderr **and** `uniqueness_proven: false` appears
   under `--json`. Mutation: delete the warning → the test must fail. A vector
   that only checks the descriptor passes against an implementation that
   silently drops the entire C1 mitigation.
3. **Below the floor still refuses.** A 1-byte (2 hex) prefix → `PrefixTooShort`,
   **exit 4** (measured; `restore.rs:2225` → `RestoreMismatch` → `error.rs:656`).
   An earlier draft said exit 2; a test asserting 2 would fail as written.
4. **Zero matches → exit 4** with `✗ NO MATCH`.
5. **`supplied == required` reconstructs and never lists** (R0 I5). Nothing in
   the draft guarded §2's own invariant that a long prefix must not downgrade to
   a list; an off-by-one (`<=` for `<`) passes vectors 1-4 and 6 silently. Assert
   both sides of the boundary: `required` bytes → reconstruct with **no** warning;
   `required − 1` → the warning path.
6. **Cap.** >64 matches at the 2-byte floor needs **S > 4,194,304** (2-byte prefix ⇒
   1/65536 hit rate ⇒ 64·65536). The draft's fixtures give S ∈ {2, 6}, so its cap
   gate could never execute — a gate that never runs is a hypothesis, not a gate.
   Either size the fixture accordingly or drive the cap through a seam that does
   not require a real space.
7. **`uniqueness_proven` never claims what a mode cannot prove** (L1, the
   Critical). Three invocations completing the SAME wallet: via `@N=` explicit
   placement, via `--search-address`, and via a full-length id. Assert the field
   is **absent** in the first two and `true` in the third. Mutation: stamp it
   unconditionally in the emitter → the first two must fail. Without this vector
   the Critical's fix is unguarded — and the emitter has exactly one call site
   (`restore.rs:1436`), so a future edit reaches all three modes at once.
8. **A candidate is not importable.** Assert `candidates[]` exists, that no
   element carries a `descriptor` key, and that top-level `wallets` is
   **absent**. Mutation: add a descriptor per candidate → fails. This is the
   executable form of §4's "reconstructs nothing".
9. **Truncation is visible.** Drive >64 matches; assert `truncated: true` and
   that `match_count` is the TRUE total, not 64. Mutation: truncate the array
   without setting the flag → fails. A consumer that cannot see truncation
   concludes "not among my keys" for a wallet that was #65.

## 7. Demo implication (mnemonic-engrave `demo/sh2` §3b) — ACTIONED

The page stated: *"`72d94d49` refused as too weak … it refuses to guess, and that
is the point."* Shipping this **falsifies that claim** for the listing path.

**Two demo defects found by the §9 gate were live already, independent of this
feature, and were corrected in `mnemonic-engrave` on 2026-09-17** (commit
`3e627ea2`, `demo 3b: teach --search-address as the way out, and retract "use
both"`):

1. **"USE BOTH TOGETHER … together the answer is fully determined"** — falsified
   by the §3.7 dispatch; supplying both ignores the address. The page now teaches
   one or the other and quotes the dispatch.
2. **The ladder 4 hex refused / 8 hex refused / 16 hex accepted** implied 16 hex
   was the threshold; the measured threshold for that 3-slot space is **10 hex**.
   The table gained the 10-hex row.

The page also now presents `--search-address` as the primary way out of a short
id (operator request, 2026-09-17) — which is what `restore --help` itself
recommends: *"Recommended over `--expect-wallet-id` (full-scriptPubKey match —
collision-free)"*.

**Still owed when this feature ships**: reframe 3b so "refuses to guess" covers
the *reconstruct* path only, and describes the list plus its §3.8 warning. The
`bundle` hint quoted in §1 needs its own fix — it prints a 4-byte prefix below
the threshold **and** omits `--cosigner` entirely (R0 N1), so the printed recipe
cannot complete a multisig template at any prefix length.

## 8. Open questions — resolved

- ~~`Enumerated` variant vs an `all_matches: bool`?~~ **Variant**, and §5 now
  requires it at the `MultisigCompletionOutcome` level, where the type cannot
  otherwise express "listed, reconstructed none".
- ~~Is the 2-byte floor right, or should it be data-driven?~~ **Keep 2 bytes.**
  §2.1's table shows the risk is driven by `S`, not by the floor, and the
  operator ruling addresses the risk with a warning rather than a cutoff. A
  data-driven floor was offered and not taken; revisit only if the warning
  proves insufficient in practice.
- ~~stdout or stderr for the summary hint?~~ **stderr**, so stdout stays the
  machine-readable channel — and §3.4 puts the same facts in `--json` so the hint
  is not the only carrier.
- ~~Which tests assert `PrefixTooShort` between floor and threshold?~~ **Four,
  all using a 4-byte prefix, which falls inside the enumerate band:**
  1. `tests/cli_restore_md1_template_multisig.rs:637 floor_weak_id_prefix_refuses`
  2. `tests/cli_restore_md1_template_multisig.rs:997 own_account_max_short_id_prefix_refuses`
  3. `tests/cli_restore_md1_template_multisig.rs:1249 search_cosigner_subset_weak_prefix_refuses`
  4. `src/permutation_search.rs:1241 validate_prefix_strength_rejects_short_accepts_long`
     — the **unit** test of `validate_prefix_strength` itself, which §5 changes.

  **R0 I2: "re-point them to assert the enumerate list" is not achievable for
  (1)-(3).** Each builds `weak = id[..8]` from the **true** id with the true
  wallet in the space, so each finds exactly one match and, under this spec,
  **reconstructs with a warning** — it never lists. Re-point (1)-(3) to assert
  the §3.8 warning path (vector 2's shape). Only a purpose-built ≥2-match fixture
  can assert a list. (4) asserts the error this spec removes in-band and must be
  re-pointed to the new signal, not deleted.
- **NEW, from the §3.2 correction (R0 M1):** §3.2 previously justified keeping
  the short-circuit with "a second match is impossible". The *behaviour* is
  right, the *justification* was the same false claim §2 was corrected to remove;
  an implementer reading it could treat the `≥ threshold` `Ambiguous` arm as dead
  code and delete it. Rewritten above to justify the short-circuit by "the ≥2
  case is an error, not a list."

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
| 10 | `verify-bundle` shares the completion engine | ✓ `verify_bundle.rs:946` — §5 |
| 11 | `bundle` advertises a 4-byte prefix `restore` refuses | ✓ measured, quoted verbatim in §1 |

**NOT covered by this gate** (still a reviewer's job): whether the enumerate
output is legible to an operator mid-recovery; whether the 64 cap and 2-byte
floor are the right numbers; the §5 `verify-bundle` ruling; whether a test as
specified in §6 can actually fail; and the §7 demo rewrite's accuracy.
