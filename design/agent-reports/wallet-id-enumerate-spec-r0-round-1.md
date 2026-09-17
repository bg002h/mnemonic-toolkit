VERDICT: NO-GO

Critical: 1 / Important: 7 / Minor: 7 / Nit: 2

R0 round 1 — `design/SPEC_restore_wallet_id_prefix_enumerate.md`
Reviewer: independent architect (opus). Tree: master `baf288b2` + spec commits
`327e13c7`, `40adca6d`. Scope: the one question — is this spec sound enough to
implement against? Nothing re-measured from §9; everything below is new.

---

## C1 — §2 (table row "< threshold / exactly 1 match"), §3.4, §4 bullet 1: the guard being removed was never about ambiguity, it was about a LONE spurious match

**The spec has inverted the purpose of the check it relaxes.** §2 says of the
new "short prefix, exactly 1 match → reconstruct, exit 0" cell:

> A short prefix that happens to match exactly one assignment still completes to
> that wallet (it is, in fact, unique in this space — the operator got lucky
> with a short id).

The function whose refusal this removes documents the opposite, in the source
being changed (`crates/mnemonic-toolkit/src/permutation_search.rs:310-313`,
verbatim):

> The `+32` term keeps the false-positive probability of **a lone spurious
> match** ≤ ~2e-10 across the realized range; a fixed 8-byte prefix would hit
> **~1-in-275 at `K=32`** — so the prefix MUST size from `S`.

`required_prefix_bytes` does not exist to prevent *two* matches. It exists to
prevent *one wrong* match. The `+32` is a bound on exactly the cell §2 now
admits. "Unique among the assignments I enumerated" implies "this is the
recorded wallet" **only if the recorded wallet is in the space and the recorded
prefix is correct** — and those are the two propositions `--expect-wallet-id`
exists to test. When either fails, today's answer is `✗ NO MATCH`, exit 4
(correct). Under this spec the answer can be a reconstructed wallet at exit 0,
byte-identical in shape to a proven completion, with no warning of any kind.

### Reproduction (quantified, using the code's own number)

Let λ = expected spurious matches = `S · 2^-8b` for a supplied prefix of `b`
bytes over a realized space `S`. `required` is chosen so λ(required) ≤ 2^-32;
every byte dropped below it multiplies λ by 256.

The code's own worked case: `n=11, own=4, --own-account-max 32` →
`S = P(39,11) ≈ 6.65e16`, `required_prefix_bytes = 11` (pinned by
`prefix_ladder_own_account_max_subset_space`). The new enumerate band is
`[2, 11)`, so **an 8-byte prefix is admitted there** — and the doc comment
above states an 8-byte prefix at K=32 hits a lone spurious match at
**~1-in-275** (independently recomputed: 6.65e16 / 2^64 = 1/272 ✓). Today that
command exits 4. Under this spec, 0.36% of the time it reconstructs a wallet
that is not the recorded one and exits 0.

**This is not a cherry-picked corner — the worst case is always inside the new
band.** Spurious-lone-match probability is λe^-λ, maximised at λ=1, i.e. at
`b* = log2(S)/8`. Since `required = ceil(log2(S)/8 + 4)`, we have
`b* ≈ required − 4`, so `b*` lies inside `[2, required)` for every space with
`S ≥ 2^16` (equivalently `required ≥ 6`). At `b = b*` the spurious-unique rate
is **~37%**.

`S ≥ 2^16` is routine on the very paths the spec cites: a 3-slot template with
2 cosigner cards and `--own-account-max 40` gives `S = P(42,3) = 68,880`; the
spec's own §8 test-2 shape with `--own-account-max 256` gives
`S = P(258,3) = 16,974,336`, where `required = 7` and a **6-hex** prefix — an
entirely plausible thing to copy off a plate — sits at λ ≈ 1.01.

### Why the wrong wallet is worse than a wrong ordering

On the exact `n!` path a wrong candidate is a re-ordering of the *same* key
set. But the large-`S` spaces where C1 bites are exactly the subset spaces
(`--own-account-max`, `--search-cosigner-subset`), where a wrong candidate is a
different **key SET** — a wallet containing an own key at an account the
operator never used, or a cosigner card that is not in the wallet. The operator
is handed a descriptor + first receive address for a quorum they may not
control, at exit 0, having asked the tool to *verify* precisely that.

### Two plausible entry paths, both mid-recovery

1. **Misread prefix.** One hex character wrong off a worn plate. The true
   wallet is in the space but no longer matches the target; a spurious lone
   match is then the *only* match.
2. **Wrong/incomplete key set.** A cosigner card from a different wallet, a
   missing cosigner, a wrong account — the whole reason the operator widened
   the search with `--own-account-max` / `--search-cosigner-subset` in the
   first place.

### Scope note on the operator ruling

The ruling recorded in §2 is *"auto-enumerate on short prefix"* — a ruling
about **listing**. §2 extends it to a second path the ruling does not mention:
sub-threshold **reconstruction**, silently. That extension, not the ruling, is
the finding. Note also that for the spec's own motivating case (§1: 3 slots,
S=6, 4-byte prefix) the enumerate path is unreachable — P(a second match) =
6·2^-32 = 1.4e-9 — so the behaviour that actually ships for the motivating
operator is *this* path, not the one §4 analyses.

**Severity: Critical** — unmet guarantee ("never reconstructs a plausible wrong
wallet"), on the funds-safety core, at a documented 1-in-275 and a derived
worst case of ~37%. The defect is the *absence of any signal at all* on that
path: §2's table makes the sub-threshold cell textually identical to the proven
one.

---

## I1 — §3.3/§3.4: exit 0 for "here are 6 candidates", and `--json` is unspecified

§3.4 assigns exit **0** to both "here is your wallet" (reconstructed) and "here
are N candidates, nothing reconstructed". A caller cannot distinguish them by
exit status. The same operator-facing fact — *your wallet is not determined* —
returns failure above the threshold (`✗ AMBIGUOUS`, exit 1,
`cmd/restore.rs:2123`) and success below it, with no rationale given.

§3 never mentions `--json`, which this path supports. The existing success
envelope on this exact code path is (`cmd/restore.rs:2407-2421`):

```
{"master_fingerprint":…, "wallets":[{"descriptor":…, "first_addresses":…}]}
```

§3.3 prescribes per-match output that is field-for-field the same shape (id,
assignment, first receive address). The natural implementation therefore puts
candidates into `wallets[]`, and any consumer reading `.wallets[0].descriptor`
— the only shape that has ever existed here — silently takes candidate #1 as
*the* wallet, at exit 0. **This escalates to Critical if implemented as the
natural `wallets[]` reuse**; it is Important as written because the spec is
merely silent at the point where the default is wrong.

---

## I2 — §8: the re-pointing instruction is wrong for all three CLI tests; none of them will produce a list

§8 says the four in-band tests "must be re-pointed to assert the enumerate
list". Read at source, all three CLI tests use `weak = id[..8]` — **a 4-byte
prefix of the TRUE wallet's id, with the true wallet in the space**:

- `crates/mnemonic-toolkit/tests/cli_restore_md1_template_multisig.rs:637`
  `floor_weak_id_prefix_refuses` — 2 cosigner entries → `S = 2! = 2`.
- `…:997` `own_account_max_short_id_prefix_refuses` — n=2, K=32 →
  `S = P(33,2) = 1056`, `required = 6`.
- `…:1249` `search_cosigner_subset_weak_prefix_refuses` — n=3, K=32, opt-in
  `S_opt`.

For each, λ at 4 bytes is between 4.7e-10 and ~1e-5, so the scan finds **exactly
one match — the true one — and reconstructs it, exit 0**. There is no list to
assert. The three tests must be re-pointed to assert a *successful completion*,
which is the opposite of what §8 instructs, and which would bake C1 into the
suite as asserted behaviour across four tests.

The error is diagnostic of the same confusion as C1: §8 assumes "short prefix ⇒
list", when short prefix ⇒ *reconstruct* in every fixture the repo owns.

---

## I3 — §6 vector 1 cannot prove what it claims, and cannot be built from the stated fixture

§6's preamble: *"Generated during implementation from a fixed 2- and 3-cosigner
template so the counts are exact."* Those fixtures give `S ∈ {2, 6}`.

**(a) It cannot be built there.** V1 requires a 2-byte prefix matching K≥2
assignments. With S=6, P(any second assignment collides in 16 bits) ≈ 5·2^-16 =
7.6e-5. The fixture must be *ground* (search seeds/accounts until two
assignments share a 2-byte id prefix — feasible, ~13k trials, but it is a
deliberate collision search the spec does not call for and an implementer will
not expect from "a fixed template").

**(b) At its stated bound it does not prove its stated property.** V1 says
"K≥2" and claims *"Proves the scan collects all"*. A mutant that collects the
first two matches and stops passes V1 at K=2 while silently truncating every
real list. The vector needs K≥3 to detect that mutant. (The specific mutation
V1 names — re-introducing the short-circuit — *is* caught, because the mutant
returns `SearchOutcome::Ambiguous` → `✗ AMBIGUOUS`, exit 1, failing on exit
code as well as on stdout; but V1's reasoning describes it as "the list drops to
2 (or 1)", which is not what the mutant produces.)

Net: the enumerate path — the entire subject of this spec — risks shipping with
no end-to-end vector that can fail.

---

## I4 — §6 vector 6 (the output cap) is unconstructible from the §6 fixture

To exceed 64 matches at the 2-byte floor (the shortest prefix admitted, hence
the most matches per unit of `S`) requires `S > 65·65536 ≈ 4.26M`. The §6
fixtures give `S ∈ {2, 6}` — seven orders of magnitude short. V6 is specified at
the CLI level ("prints 64 + 'and K more'; exit 0"), so it cannot be satisfied by
a unit test over a synthetic `Vec<Match>` either, and the spec's word "synthetic"
does not resolve which layer it means.

It *is* reachable — `--own-account-max 256` on a 3-slot template gives
`S = P(258,3) = 16,974,336`, ~259 matches at 2 bytes — but that is a
17-million-candidate real search (over `SILENT_THRESHOLD`, so it prints a
progress line, and well inside `SEARCH_CEILING`), which contradicts §6's
preamble and needs saying. The cap is the only bound on output volume; as
specified its gate never runs.

---

## I5 — §6: no vector guards §2's stated invariant "a long prefix must never silently downgrade into a list"

§2 states that invariant explicitly and calls the ≥threshold `Ambiguous` path
load-bearing. The load-bearing comparison is `supplied < required`. An
off-by-one (`<=`) admits a threshold-length prefix into enumeration — and every
one of V1–V6 still passes: V5 uses a 16-hex (8-byte) prefix against
`required = 5` for its fixture, three bytes clear of the boundary, so nothing
tests `supplied == required`. The floor boundary is covered (V1 uses exactly 2
bytes, V3 uses 1); the threshold boundary is not covered at all.

---

## I6 — §5's change list omits the function that actually makes the decision, and the outcome type that cannot express the result

§5 lists `permutation_search.rs`, `cmd/restore.rs`, and `cmd/verify_bundle.rs`,
and requires: *"the engine change MUST NOT default `verify-bundle` into
enumeration by inheritance."* As written that requirement has no seam to attach
to:

- The enumerate **decision point** is `ps::validate_prefix_strength(prefix.len(),
  realized_s)` at `cmd/restore.rs:2009`, inside `complete_multisig_template` —
  a `pub(crate)` function **defined in `cmd/restore.rs`** and **called by
  `cmd/verify_bundle.rs:954`**. "Change restore.rs" is therefore literally
  "change verify-bundle". `✗ NO MATCH` / `✗ AMBIGUOUS` already print from inside
  that shared function today (`restore.rs:2114`, `:2123`), which is the existing
  proof that behaviour there is inherited by verify.
- `MultisigCompletionOutcome` (`restore.rs:~1205`: `completed`, `pool`,
  `assignment`) has **no representation for "listed, reconstructed none"**.
  The load-bearing edit is a change to that return type, and §5 never names it.

Neither `complete_multisig_template` nor `MultisigCompletionCtx` nor
`MultisigCompletionOutcome` appears in §5. An implementer following §5
literally produces exactly the inheritance §5 forbids.

**(This finding is the defect; the following is context for the §5 ruling, not
a prescription.)** Option (a) *is* implementable without inheritance, and the
tree makes it cheap to enforce: `MultisigCompletionCtx` has no `Default` impl
and **both** construction sites are exhaustive struct literals with no
`..Default::default()` (`restore.rs:1418-1434`, `verify_bundle.rs:930-952`), so
a required `allow_enumerate: bool` field on the ctx is compiler-enforced at both
callers — a missed call site cannot compile. That is the only location in the
tree where the two surfaces differ.

### Ruling on §5 (the spec's open question), as requested

**Rule (a): `verify-bundle` keeps `PrefixTooShort`.** Reasoning:

1. A verifier's contract is a verdict. §5 is right that a list is not a verdict;
   it is also not a FAIL, and reporting FAIL for "I could not decide" mislabels
   a wallet that may well be correct — a verifier that cries wolf gets ignored.
2. (a) preserves an operator-protective property (b) destroys: today verify
   refuses a weak prefix **before** calibration and before the scan
   (`restore.rs:2009` precedes `run_capped_search` at `:2090`). Under (b),
   verify would scan first and report inconclusive after — up to the full
   `SEARCH_CEILING` — to reach a verdict it could have refused instantly.
3. **A third option exists and is worse: "the engine returns the match list and
   each CLI decides the verdict."** It is structurally more robust than a
   boolean (a facts-returning engine cannot be defaulted into a wrong verdict),
   but it forfeits point 2 — verify pays the whole scan before refusing — so it
   should be rejected on that ground, not adopted for its elegance.
4. (a)'s cost is an operator-facing asymmetry that must be written into the
   error text (see M7), not a funds risk.

This ruling does **not** clear I6: (a) is only safe if the opt-out lands on
`MultisigCompletionCtx`, and the spec must say so.

---

## I7 — §3.7 describes an interaction that does not exist, and §3.3's escape hatch is a no-op

§3.7: *"When both are supplied, address filtering runs first, then the id-prefix
enumerates within that (near-always 1)."*

The code dispatches the two modes as mutually exclusive branches
(`cmd/restore.rs:2007` / `:2035`):

```rust
let outcome = if id_search { … }        // --expect-wallet-id
             else if addr_search { … }  // --search-address
```

There is no `conflicts_with` between the two flags (`restore.rs:142`, `:174`)
and no runtime refusal. **Supplying both runs id-search only; `--search-address`
is read into the ctx and never used.** No filtering "runs first"; no composition
exists at any prefix length. Consequences:

1. §3.7 is not a description of behaviour after the change — it is an unlisted
   *additional* change (composing two evaluators), absent from §5. Its closing
   clause "Unchanged when the id is ≥ threshold" is self-contradictory: today
   "unchanged" means the address is ignored at ≥threshold too.
2. §3.3's prescribed summary line tells the operator to *"re-run with more id or
   `--search-address` to pin one"*. An operator who adds `--search-address` and
   keeps `--expect-wallet-id` — the literal reading of "re-run with" — gets the
   **identical list**, with no indication the address was ignored, and may
   reasonably conclude the address failed to narrow it. This is the same class
   of defect §1 exists to fix ("the tool advertises a recipe it then refuses"),
   reintroduced in the remedy.
3. Combined with C1: an operator supplying a correct `--search-address` *and* a
   misread short id runs a pure id-search; a spurious lone match then
   reconstructs a wrong wallet at exit 0 while the operator believes the address
   pinned it.

---

## Operator journey (§5 of the brief) — a 4-byte prefix off a plate, mid-recovery

**In hand, exactly:** an md1 template plate; a plate or notebook line reading
`e1ccd788` (8 hex = 4 bytes, the string `bundle --md1-form template` told them
to record); their own seed; 2 cosigner `mk1` cards.

**What the tool does:** today, instant refusal naming 5 bytes. After the change:
scans S=6, finds exactly one match (the true wallet, P = 1 − 1.4e-9),
reconstructs it, exit 0. **This works, and it is the fix §1 asks for.** The
enumerate path never fires for this operator.

| # | What else they reasonably do | What happens | Class |
| --- | --- | --- | --- |
| a | Mis-transcribes one hex char (worn plate) | S=6: still `NO MATCH`, exit 4 — correct. On any space with S ≥ 2^16 (see b, f): C1 window opens | **refusal (C1)** |
| b | Doesn't have all cosigners; adds spares + `--search-cosigner-subset` | `S_opt` explodes; 4 bytes stays in-band; if the true set is not representable → lone spurious match → wrong wallet, exit 0 | **refusal (C1)** |
| c | Gets a list, follows the printed hint: adds `--search-address <their address>`, keeps the id | Identical list; the address is silently ignored (`restore.rs:2007`) | **refusal (I7)** |
| d | Scripts it: `--json \| jq -r .wallets[0].descriptor` | Candidate #1 read as *the* wallet, exit 0 | **refusal (I1)** |
| e | Typed 9 hex chars off a smudged plate | `"--expect-wallet-id must be an even-length hex prefix"` — never PrefixTooShort, never a list | **documentation (M4)** |
| f | Doesn't recall the account; adds `--own-account-max 256` "to be safe" | S = 16.97M; the instant "prefix too weak, need ≥N bytes" becomes a calibrate-then-progress-bar scan, or on a bigger space `"estimated … exceeds the 1h ceiling"` — a message that never names their real problem | **warning (M3)** |
| g | Runs `verify-bundle` first (as the demo teaches) with the same short id | Under §5(a): refused as too weak, minutes after `restore` accepted it | **documentation (M7)** |
| h | Has ≥65 candidates | Sees 64 + "K more", exit 0, no path forward but more id | **refusal (I1, exit code)** |

Divergences (a), (b), (c), (d) each produce an outcome **worse than telling the
operator nothing**, which is the bar for earning a spec change. (e), (g) are
documentation; (f) is a one-line warning.

---

## §3.6 ceiling interaction (brief Q6) — VERIFIED SOUND, no finding

Checked rather than assumed. `cap_decision`'s estimate is
`per_candidate × total_candidates` and its doc states
(`permutation_search.rs:385-387`):

> The exhaustive time is over the FULL space (**no early-terminate credit**) —
> the operator is being asked to accept the worst case (the no-match scan, which
> is also exactly the scan an `Ambiguous`/`None` outcome performs).

The ceiling was therefore calibrated against a *full* scan from the start.
Removing the second-match short-circuit cannot exceed an estimate the operator
already accepted (the `None` and `Unique` outcomes already scan to exhaustion;
only `Ambiguous` terminated early, i.e. the short-circuit was a best case, never
the calibration point). The estimate also ignores parallelism (up to 20 threads,
`MAX_SEARCH_THREADS`), so it is conservative by ~20×. §3.6's "unchanged and still
applies" is **correct**, and the spec would be stronger for stating this
reasoning rather than asserting the conclusion. Residual cost outside the
estimate is M6.

---

## Minor

- **M1 — §3.2 contradicts §2 on reachability.** §3.2: *"At ≥ threshold, keep the
  short-circuit: **a second match is impossible**, so a Unique/None decision is
  reached without collecting."* §2 spends a paragraph establishing the opposite
  (≤~2e-10 is a bound, not impossibility) and requires the ≥threshold
  `Ambiguous` arm (`restore.rs:2123`) to stay an error. The prescribed
  *behaviour* is consistent; the stated *justification* is the same false claim
  §2 was corrected to remove. An implementer reading §3.2 could treat that arm
  as dead code.
- **M2 — §3.3 "stable order" requires an explicit sort, and is under-specified
  in address mode.** The engine's match vector is assembled per-thread
  (`matches.lock().unwrap().extend(local)`, `permutation_search.rs:1085-1087`),
  so `found` is in thread-completion order — nondeterministic run to run. A
  match is `(perm_rank, address_index)`; ordering "by assignment permutation
  index" alone is not a total order when `SearchMode::Address` is in play. Any
  test asserting list order is flaky until both are pinned.
- **M3 — the refusal moves from instant to post-scan.** `validate_prefix_strength`
  runs at `restore.rs:2009`, *before* `run_capped_search` (`:2090`). Today a
  sub-threshold prefix refuses with the number of bytes needed, in milliseconds.
  After the change the operator instead enters calibration and, on a large
  space, meets `SearchTimeExceedsCeiling` — a message about time, for a problem
  about prefix length. Worth one sentence in §3.6.
- **M4 — odd-length hex is refused before any of this applies.**
  `decode_wallet_id_prefix` (`restore.rs:2461`) uses `hex::decode`, so 5, 7 or 9
  hex characters produce *"must be an even-length hex prefix"*. The §3.1 floor
  is stated in bytes and the spec never addresses odd input, which is what a
  smudged plate actually produces.
- **M5 — a third `--expect-wallet-id` consumer is unlisted.** The **single-sig**
  template path (`restore.rs:1010-1053`) accepts any prefix length with only an
  advisory below 4 bytes and never calls `validate_prefix_strength`. After this
  change a 1-byte prefix is refused on the multisig path (below the new floor)
  and accepted-with-advisory on the single-sig one. §5 claims to enumerate the
  surfaces this lands on; if the floor is implemented in the *shared*
  `decode_wallet_id_prefix` rather than in `validate_prefix_strength`, it
  silently changes the single-sig path too.
- **M6 — §3.3's per-match address is outside the calibrated cost.**
  `calibrate_per_candidate` (`restore.rs:2273`) times the **id** evaluator only.
  Rendering a first receive address per match costs a descriptor string build +
  miniscript parse + `script_pubkey_at` — materially more, paid after the
  ceiling decision. Bounded only if the implementation derives addresses *after*
  applying the §3.5 cap; §3.3 and §3.5 do not order those steps.
- **M7 — (if R0 rules §5(a)) the sibling commands must explain themselves.**
  `restore --expect-wallet-id <6 hex>` will list; `verify-bundle
  --expect-wallet-id <6 hex>` will refuse "prefix too weak" — same flag, same
  engine, opposite answers, minutes apart in the demo's own order. The verify
  refusal text needs to say that restore enumerates and a verifier requires a
  decisive id, or it reads as a bug.

## Nit

- **N1 — §1's quoted `bundle` hint has a second defect.** For a 2-of-3
  template the printed recipe is `restore --md1 <template> --from <seed>
  --account 0 [--expect-wallet-id e1ccd788]` — with **no `--cosigner`**, so it
  cannot complete a multisig template at any prefix length. The §7 demo
  follow-up should cover the missing `--cosigner`, not only the prefix length.
- **N2 — §3.3's summary says "(of S in the space)".** `S` is the *realized*
  cardinality — `n!`, `s_own`, or `s_opt` depending on flags. Worth naming which
  number is printed, since the whole prefix-sizing story hangs on it.

---

## What a fold must not skip

C1 lands on the spec's central decision, not on its edges: §2's table, §4's
first bullet and §8's re-pointing instruction all rest on the same inverted
reading of `required_prefix_bytes`. I2 is that inversion showing up as an
instruction that cannot be carried out. A fold that adjusts wording in §2 without
revisiting §4 and §8 will leave the contradiction in place.

Machine-checkable claims in this report, for the fold's gate: the λ arithmetic
(`6.65e16 / 2^64 = 1/272` vs the doc's "1-in-275"); `P(33,2) = 1056` and
`required_prefix_bytes(1056) = 6`; `P(258,3) = 16,974,336`; the three test
fixtures' `weak = id[..8]`; the `if id_search … else if addr_search` dispatch;
the absence of `Default` on `MultisigCompletionCtx`.
