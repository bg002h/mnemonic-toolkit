# VERDICT: NO-GO

Critical: 2 / Important: 1 / Minor: 3 / Nit: 0

Whole-diff adversarial review of `75f1b569` — *"fix: the ordering collapse is
sound for an ADDRESS target, not a wallet-id"*.

Scope: the one question asked — **is the fix correct and complete, and does it
introduce a new defect?** Both Criticals below are **constructed
counterexamples, run against the real CLI**, not assessments.

Line numbers are at the tree I reviewed, `HEAD = 3d012345` (one commit past the
diff under review; `3d012345` touches `restore.rs` and shifts the diff's own
line numbers by +6).

Evidence binding: the fixture I built independently from the CLI reproduces the
commit message's own quoted wallet-id **`959762f10433386ac68a09c4364f5c97`**
byte-for-byte, so the runs below are against the *same* wallet the regression
test uses.

---

## W1 — CRITICAL. `realized_s` was not changed, so the prefix floor and the §6.4 time cap are now sized from a space `N!` SMALLER than the one actually scanned. The commit's "SECOND DEFECT CLOSED BY THE SAME LINE" is false.

**Where.** The diff changed exactly one thing — the `sorted:` field of the
`Enumeration` (`restore.rs:1995`, `2003`). It did **not** change the two places
that compute `realized_s`, which are still keyed on the **shape alone**:

```
restore.rs:1845   let sorted = crate::synthesize::is_order_independent_shape(&d.tree);
restore.rs:1846   let s = ps::s_opt(k_own, m_cosigners, n, sorted)        // OPT-IN
restore.rs:1899   let sorted = crate::synthesize::is_order_independent_shape(&d.tree);
restore.rs:1900   let s = ps::s_own(k_own, j, m_cosigners, sorted)        // OWN-ANCHORED
```

These are the **only** `s_own` / `s_opt` call sites in the crate outside
`permutation_search.rs`'s own tests (verified by grep), and the only production
`Enumeration` construction sites are `restore.rs:1995 / 2003 / 2008`. They are
computed ~140 lines **before** `addr_search` exists (`restore.rs:1950`), which
is why the fix could not reach them without moving code — and did not.

The engine does **not** use `realized_s` to enumerate. It derives its own count:

```
permutation_search.rs:1012   let perms = enumeration.cardinality()...
```

So after this diff, on `sortedmulti` + (`--own-account-max` | `--search-cosigner-subset`) + `--expect-wallet-id`:

| quantity | value | source |
| --- | --- | --- |
| assignments actually scanned | `S × N!` | `enumeration.cardinality()` |
| `realized_s` fed to `validate_prefix_strength` (`restore.rs:2034`) | `S` | `s_own`/`s_opt`, sorted=true |
| `realized_s` fed to `run_capped_search` → `cap_decision` (`restore.rs:2053`, `2293`, `2301`) | `S` | same |
| `realized_s` checked against `REALIZED_S_MAX` (`restore.rs:68`, 1e15) | `S` | same |

Before the fix these two numbers **agreed** (both collapsed). The fix widened one
and left the other. **The divergence is newly introduced by this diff.**

### W1(a) — the prefix-strength floor is under-sized. MEASURED.

`required_prefix_bytes` (`permutation_search.rs:322`) exists to hold a lone
spurious match at ≤ ~2e-10. Under-feeding it `S` instead of `S × N!` is an
**unmet guarantee** — precisely the guarantee the commit claims to have restored.

Same pool, same scanned space, two different floors:

```
fixture: N=2, 1 cosigner card, --own-account-max 255  (k_own = 255; own_accounts is 0..K, EXCLUSIVE)
both templates now enumerate C(255,1) · 2! = 510 assignments

### wsh-sortedmulti   realized_s = C(255,1) = 255 -> required_prefix_bytes = 5
   5-byte prefix (959762f104):     exit=0   <accepted, no prefix refusal>
   6-byte prefix (959762f10433):   exit=0   <accepted>

### wsh-multi         realized_s = 510            -> required_prefix_bytes = 6
   5-byte prefix (57e632906d):     exit=4   "prefix too weak for this search: "
   6-byte prefix (57e632906d32):   exit=0   <accepted>
```

Identical enumeration size (510). The unsorted template refuses the 5-byte
prefix; the sorted one accepts it. The only difference is that `realized_s`
still collapses. Arithmetic check: `req(255)` = ceil((8+32)/8) = 5;
`req(510)` = ceil((9+32)/8) = 6.

Consequence: on this path a short-but-accepted prefix can produce a spurious
`Unique` match when the true wallet is *absent* from the supplied pool — the
silent-wrong-wallet class this module exists to prevent. (When the true wallet
*is* present a collision yields `Ambiguous`, which refuses; the dangerous arm is
the one where it is not.)

### W1(b) — the §6.4 adaptive cap and the 1-hour ceiling are blind by `N!`. MEASURED.

`cap_decision` (`permutation_search.rs:388`) is fed `realized_total =
realized_perms × outer` (`restore.rs:2293`) — i.e. `S`, not `S × N!`.

```
fixture: N=8 wsh-sortedmulti (threshold 4), 7 cosigner cards,
         --search-cosigner-subset, FULL 16-byte --expect-wallet-id

--own-account-max 6 :  realized_s = Sum_j C(6,j)*C(7,8-j) = 1,287
                       actually scanned = 1,287 * 8! = 51,891,840
   exit=0  WALL=23.1s on 24 cores
   stderr: (NO "searching N candidate assignment(s) (est. <= ...)" line)
           -> cap_decision returned RunSilent: it believes the scan is < 30s

   measured per-candidate = 23.1s * 24 / 51.89e6 = 10.7 us
   TRUE serial cost = 51.89e6 * 10.7us = 555s  -> SILENT_THRESHOLD (30s,
   permutation_search.rs:54) breached ~18x with no progress line and no ETA.

--own-account-max 10:  realized_s = 24,265
                       actually scanned = 24,265 * 8! = 978,364,800
   TRUE serial cost = 978.36e6 * 10.7us = 10,468s = 2.9 HOURS
   -> exceeds SEARCH_CEILING (3600s, permutation_search.rs:59), so the tool
      MUST refuse or demand --accept-search-time.
   OBSERVED: it refuses nothing, prints nothing, and runs. Killed at >120s
      wall with only the argv warning on stderr.
```

The commit message's cost argument — *"required_prefix_bytes sizes from realized
S and the ceiling refuses (or asks for `--accept-search-time`) if the scan is
too slow. Both were always correct; they were being fed a wrong S"* — is exactly
inverted. They are **still** being fed the wrong S, and the diff made the error
larger by `N!` rather than closing it. The two refusals named as the reason no
new machinery is needed are the two that no longer fire.

The comment the diff added at `restore.rs:1974-1978` asserts the opposite in the
source:

> *"This also re-sizes the prefix floor: `realized_s` feeds
> `validate_prefix_strength`, so while the space was under-counted the required
> prefix was under-sized too…"*

`realized_s` is not re-sized by this diff. The comment, the commit message and
`design/SPEC_restore_wallet_id_prefix_enumerate.md` §A1 all record a fix that
was not made.

### Reproduction

`bash prefix.sh` and `bash ceiling.sh 6` (scripts transcribed in the appendix).

---

## W2 — CRITICAL. `addr_search` is the wrong predicate. Supplying BOTH `--expect-wallet-id` and `--search-address` re-enables the collapse while the ID evaluator runs — the original defect, reachable verbatim, with a *correct* 16-byte id and a *correct* address.

**Mechanism.** Three lines, no inference needed:

```
restore.rs:1949   let id_search   = ctx.expect_wallet_id.is_some();
restore.rs:1950   let addr_search = ctx.search_address.is_some();
restore.rs:1979   let collapse_orderings = sorted_shape && addr_search;
restore.rs:2030   let outcome = if id_search { /* ID evaluator */ } else if addr_search { ... }
```

`addr_search` answers *"was an address supplied"*, not *"is the address search
the one that will run"*. When both flags are present, `collapse_orderings` is
`true` **and** the **id** evaluator runs — over the collapsed space. The correct
predicate is `addr_search && !id_search`.

There is **no** `conflicts_with` between the two flags on either surface
(grepped: `restore.rs` and `verify_bundle.rs` declare `conflicts_with` only for
`passphrase`, `account`, `descriptor_file`, `bundle_json`). Both are freely
combinable, and the `--search-address` help text actively recommends it
(*"Recommended over `--expect-wallet-id`"*), so an operator holding both records
supplying both is the *expected* cautious behaviour — they are adding a second
correct verification target.

**Counterexample. Run. The fixture is the regression test's own.**

```
recorded wallet-id = 959762f10433386ac68a09c4364f5c97   (== the commit's quoted id)

=== A. the NEW regression test's invocation (id ONLY) ===
exit=0
addresses: ['bc1qqz0ggyuhz02f88496t536s5e5d2jpu8h0zk6lzy2rqlyycyl8a5sqgpy4y',
            'bc1qy9tq6jrp4ffmnmklyyauwwsl4emrhp89039ch62hd0vj7kmztazs0ujqh3']

=== B. SAME + that wallet's OWN CORRECT --search-address
       bc1qqz0ggyuhz02f88496t536s5e5d2jpu8h0zk6lzy2rqlyycyl8a5sqgpy4y ===
exit=4
✗ NO MATCH
```

Adding a **second, redundant, correct** proof of the same wallet turns a working
restore into `✗ NO MATCH`. This is the identical failure mode, exit code and
stderr string the commit exists to eliminate, still fully live on a plausible
operator input — and arguably worse than the original, because the operator's
response to a `NO MATCH` is to supply *more* evidence, which here makes it
permanent.

`verify-bundle` inherits it: it threads both `expect_wallet_id` and
`search_address` into the same `MultisigCompletionCtx`
(`verify_bundle.rs:946-947`) and calls `complete_multisig_template`
(`verify_bundle.rs:954`) with no mutual-exclusion gate. The commit's
*"verify-bundle inherits the fix for free"* is true; it also inherits W1 and W2
for free.

### Reproduction

`bash repro.sh` (appendix).

---

## W3 — IMPORTANT. The OPT-IN arm the fix also changes (`restore.rs:1995`) has NO regression test, and the two property tests that should cover the ordering axis are structurally blind: the own key is pinned to slot `@0` in every generated case.

The diff touches two generator arms. The new test
`sortedmulti_own_account_max_id_search_finds_non_identity_placement` exercises
**only** the own-anchored arm (`--own-account-max`), at `N = 2` where `N! = 2`.
The `--search-cosigner-subset` arm (`Enumeration::OptIn`) is changed with zero
coverage. Grepped: no test in `crates/mnemonic-toolkit/tests/` combines
`sortedmulti` + `--search-cosigner-subset` + a non-identity own placement.

Worse, the blindness is systematic rather than an oversight of one test.
`tests/prop_subset_search_roundtrip.rs` *does* proptest `wsh-sortedmulti` /
`sh-wsh-sortedmulti` against `--own-account-max` + `--expect-wallet-id` (the
headline property) **and** against `--search-cosigner-subset` + id
(`cosigner_subset_search_drops_outsider_completes_to_golden`, line 618) — and
both passed throughout the life of the defect. Reason:

```
prop_subset_search_roundtrip.rs:331-345  fn build_case(...)
    let slots = if family == 0 { vec![(SEED_A, own_acct), (SEED_B, b_acct)] }
                else          { vec![(SEED_A, own_acct), (SEED_B, b_acct), (SEED_C, c_acct)] };
```

`SEED_A` is the OWN seed and it is **always slot `@0`**. The generator randomises
the *account* (`own_acct in 0u32..4`) but never the *slot*, so the true
assignment is always the identity placement that the collapsed generator emits —
the entire ordering axis is unreachable by the property suite. A single
`own_slot in 0..n` rotation in `build_case` would have caught the original
defect, would cover the opt-in arm, and would catch W2 if the generator also
varied the target flags.

At `N = 2` the new test also cannot distinguish "the collapse was removed" from
"the collapse was inverted": `N! = 2`, so there are only two orderings and the
test's own is the second. A case with `N ≥ 3` and the own key at a middle slot
pins the enumeration properly.

---

## W4 — MINOR. The diff falsifies text it does not touch.

`design/SPEC_restore_wallet_id_prefix_enumerate.md:191` is normative:

> *"`S` is the **realized** cardinality — `n!`, `s_own` or `s_opt` depending on…"*

After this diff that is false on the sorted + id subset path: the realized
cardinality is `s_own(..., sorted=false)` while the code computes
`s_own(..., sorted=true)`. §A1 (lines 437-460) additionally records the
`realized_s` / `validate_prefix_strength` claim disproved in W1.

The spec's planned `"realized_space"` JSON field (line 274) and the
`realized_space_kind` table (line 341) are not yet implemented (grepped: no
`realized_space` in `crates/mnemonic-toolkit/src/`). When they are, wiring them
from `realized_s` will emit the under-counted number to the operator as the
ambiguity report's denominator. Fix W1 before that field ships.

---

## W5 — MINOR. Enlarging the enumeration without enlarging the ceiling check moves a class of refusal from an actionable message to a late, opaque one.

`REALIZED_S_MAX` (1e15, `restore.rs:68`) is applied to `realized_s`, with a
message naming the lever (*"narrow `--own-account-max` or supply fewer
`--cosigner` cards"*). A sorted + id subset search now passes that gate at
`S ≤ 1e15` while the engine attempts `S × N!`. For large `n` that overflows
`u128` inside `Enumeration::cardinality()` and surfaces as
`SearchError::SearchSpaceTooLarge { n }` from `permutation_search.rs:1013` —
after calibration, phrased in terms of the slot count rather than the flag the
operator can change. Safe direction (it refuses), but the diagnostic regressed.

---

## W6 — MINOR. When the progress line *does* print, it prints the wrong count.

`restore.rs:2300`: `"searching {realized_total} candidate assignment(s) (est. ≤ {estimate:?})…"`.
On this path `realized_total` is `N!` too small, so both the count and the ETA
shown to the operator are wrong by that factor. Downstream of W1; listed
separately because it is operator-visible output, not an internal bound.

---

## What I tried and could NOT break — these close clean

- **Q3 — the ADDRESS path is not regressed.** `collapse_orderings = sorted_shape
  && addr_search`; the address arm is reached only when `!id_search &&
  addr_search`, where `addr_search == true`, so the expression is *identical* to
  the previous `sorted_shape` on every path that previously collapsed. Proven by
  construction, not measurement, so no `N!` blow-up is possible there.
  Confirmed live anyway: `wsh-sortedmulti`, `--own-account-max 255`,
  `--search-address` only → `exit=0 WALL=0.37s`, correct address returned.
  (The *only* way the address path loses its collapse is W2, where it is not the
  path that runs.)
- **The EXACT path is byte-unchanged.** `Enumeration::FullPermutation { n }`
  carries no `sorted` field; `apply_identity_filter = sorted_shape &&
  !over_supply && !opt_in` (`restore.rs:2010`) is untouched and is referenced
  only inside the address evaluator. The id evaluator (`restore.rs:2036-2046`)
  has no identity filter — correct, and it means the exact-path id search always
  enumerated all `n!`, so `realized_s = perm_count_u128(n, n)` agrees with
  `factorial(n)` there. No W1 on the exact path.
- **The explicit `--cosigner @N=` path is unreachable from this code.**
  `complete_explicit_assignment` returns at `restore.rs:1799`, before the pool,
  before `realized_s`, before the enumeration. Unaffected by the diff.
- **`opt_in` vs `over_supply` precedence is consistent.** Both the `realized_s`
  block (`if opt_in … else if over_supply …`) and the enumeration block use the
  same order, so they never select different variants.
- **No other consumer keyed on shape alone.** Grepped the whole crate: the only
  `s_own`/`s_opt` call sites outside the engine's own unit tests are
  `restore.rs:1846` and `restore.rs:1900` (both in W1); the only production
  `Enumeration` construction sites are `restore.rs:1995 / 2003 / 2008` (all in
  the diff). `xpub_search` does not use this engine.
- **No new `Ambiguous` outcome on a well-sized prefix.** `sortedmulti` AB-id ≠
  BA-id, so exactly one ordering matches a full id; un-collapsing cannot turn a
  `Unique` into an `Ambiguous` *unless* the prefix is under-sized — which is W1,
  not an independent defect.
- **The new test is not vacuous.** It asserts `got == golden` against an
  independent rust-miniscript derivation, not merely exit 0, and I reproduced its
  wallet-id (`959762f1…`) and its PASS from a fixture I built independently. Its
  RED was already established by the controller. Its weakness is coverage (W3),
  not falsity.
- **Rust-primary rule.** Re-checked: no Go counterpart to this search exists.
  Nothing to converge.

---

## Minimum to reach GO

1. **W2** — make the predicate `addr_search && !id_search`, or add a
   `conflicts_with` between `--expect-wallet-id` and `--search-address` on both
   `restore` and `verify-bundle`. Note these are *not* equivalent: the first
   silently prefers the id, the second forces the operator to choose. Prefer the
   first plus a test asserting both-flags resolves the non-identity placement,
   since a refusal would break operators who already pass both.
2. **W1** — compute `realized_s` from the same object that enumerates. The
   smallest correct form is to build `enumeration` first and take
   `enumeration.cardinality()` as `realized_s`, leaving the `REALIZED_S_MAX` /
   overflow / `s == 0` gates keyed on that one number. That structurally
   forecloses the class rather than patching two call sites — the divergence is
   only possible because the count is derived twice.
3. **W3** — rotate the own slot in `build_case` (`own_slot in 0..n`) and add an
   opt-in (`--search-cosigner-subset`) sortedmulti id case at `N ≥ 3` with the
   own key at a middle slot.
4. Correct the source comment at `restore.rs:1974-1978`, the commit message's
   "SECOND DEFECT CLOSED BY THE SAME LINE" paragraph, and SPEC §A1.

Items 1 and 2 are both one-line-ish changes in the same function and should land
together; re-run the whole suite after, since W1's fix changes `realized_s` on
paths that existing prefix-strength tests pin (`floor_weak_id_prefix_refuses`,
`own_account_max_short_id_prefix_refuses`,
`search_cosigner_subset_weak_prefix_refuses`,
`search_cosigner_subset_hard_ceiling_refuses`) — expect some of those to need
their boundary values re-derived, and re-derive them from the formula rather
than from whatever the code then prints.

---

## Appendix — reproduction scripts

All three use an independent BIP-32 implementation (`bip32.py`, python `ecdsa`)
rather than the CLI's own derivation, so the fixture does not assume the code
under review is correct. Note `mnemonic convert --template <t> --path m/...`
**ignores `--path`** and derives from `--account` — that cost a fixture; use
an external derivation.

Binary: `target/debug/mnemonic` at `HEAD = 3d012345`.

```
--- fixture (shared by repro.sh / prefix.sh) --------------------------------
SEED_A = "legal winner thank year wave sausage worth useful legal winner thank yellow"
SEED_B = "letter advice cage absurd amount doctor acoustic avoid letter advice cage above"
@0 = SEED_B at 48'/0'/0'/2'   (the cosigner card)
@1 = SEED_A at 48'/0'/3'/2'   (the OWN key -- a NON-identity placement)
template: wsh-sortedmulti, threshold 2, --group-size 0 --no-engraving-card
  mnemonic bundle --md1-form template  -> the md1 + the wallet-id
  mnemonic bundle --md1-form policy    -> mk1[0], the cosigner card
  => wallet-id 959762f10433386ac68a09c4364f5c97

--- repro.sh (W2) ----------------------------------------------------------
A: mnemonic restore --network mainnet --md1 <t> --from phrase=$SEED_A \
     --own-account-max 5 --expect-wallet-id 959762f10433386ac68a09c4364f5c97 \
     --count 2 --json --cosigner <mk1[0] x3>
   -> exit 0, the two golden addresses
B: A + --search-address bc1qqz0ggyuhz02f88496t536s5e5d2jpu8h0zk6lzy2rqlyycyl8a5sqgpy4y
   -> exit 4, "✗ NO MATCH"

--- prefix.sh (W1a) --------------------------------------------------------
same fixture, --own-account-max 255, --count 1, prefix truncated to 5 and 6 bytes,
run once for wsh-sortedmulti and once for wsh-multi.
   wsh-sortedmulti: 5-byte ACCEPTED (exit 0)   <- floor from realized_s = 255
   wsh-multi:       5-byte REFUSED  (exit 4, "prefix too weak for this search: ")
   both enumerate 510 assignments.

--- ceiling.sh (W1b) -------------------------------------------------------
N=8 wsh-sortedmulti threshold 4; @0..@6 from 3 seeds at accounts 0..6;
@7 = SEED_A at 48'/0'/2'/2'. All 7 cosigner cards supplied.
mnemonic restore ... --own-account-max <K> --search-cosigner-subset \
  --expect-wallet-id <full 16 bytes> --count 1 --json
   K=6  -> exit 0, WALL 23.1s on 24 cores, NO "searching ..." line on stderr
   K=10 -> still running at >120s, NO "searching ..." line, no refusal
(own_accounts is `(0..k)` -- EXCLUSIVE; restore.rs:1701.)
```
