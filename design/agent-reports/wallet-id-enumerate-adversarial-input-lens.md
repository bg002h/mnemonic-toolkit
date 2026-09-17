# Lens: DEGENERATE AND HOSTILE INPUT — `SPEC_restore_wallet_id_prefix_enumerate.md`

VERDICT: NO-GO

Critical: 3 / Important: 2 / Minor: 3 / Nit: 2

**Question asked:** what input reaches this feature that the spec never
considered, and what does the tool do then?

**Environment for every measurement below:** repo HEAD `fdf4fd5f`, binary
`target/debug/mnemonic` = `mnemonic 0.98.0`. Fixture = the §1 demo wallet shape:
a mainnet 2-of-3 `wsh-sortedmulti` at the canonical BIP-48 origin
`48'/0'/0'/2'`, built from the repo's own test seeds SEED_A (`b8688df1`),
SEED_B (`28645006`), SEED_C (`3f635a63`), emitted with `bundle --md1-form
template` / `--md1-form policy` exactly as
`crates/mnemonic-toolkit/tests/cli_restore_md1_template_multisig.rs:126-234`
does. Own key = SEED_A. "own @2" means the wallet's slot order is `B,C,A`.
Every quoted string is copy-pasted from a run, not paraphrased.

---

## CRITICAL

### A1 — a `sortedmulti` template on either subset path cannot match its own recorded id, and this spec turns today's accurate refusal into `✗ NO MATCH`

**Concrete input.** The §1 wallet (`wsh-sortedmulti`, 2-of-3) where the
operator's own key sits at slot `@2` rather than `@0` — i.e. they are the third
cosigner, not the first. They do not remember which account they used, so they
reach for the flag built for that:

```
restore --md1 <template> --from phrase=<SEED_A> --own-account-max 4 \
        --cosigner <mk1_B> --cosigner <mk1_C> --expect-wallet-id c3b1b27a
```

**Outcome, measured.** With the **full 16-byte** id (`c3b1b27a5c51f77828d0ec4280eb10bb`):

| invocation | result |
| --- | --- |
| exact pool (no subset flag) | `exit=0`, reconstructs |
| `--own-account-max 4` | **`✗ NO MATCH`, exit 4** |
| `--search-cosigner-subset` | **`✗ NO MATCH`, exit 4** |
| same wallet as `wsh-multi`, `--own-account-max 4` | `exit=0`, reconstructs |
| same wallet as `wsh-multi`, `--search-cosigner-subset` | `exit=0`, reconstructs |

The defect is isolated to `sortedmulti` + a subset path. Cause, from the source:

- `restore.rs:1954` sets `sorted_shape = is_order_independent_shape(&d.tree)`
  — a pure tree-shape test (`synthesize.rs:339-350`, `Tag::SortedMulti` ⇒ true).
- That flag is handed to `Enumeration::OwnAnchored{ sorted }` / `OptIn{ sorted }`
  **regardless of search mode**, and `s_own`/`s_opt`
  (`permutation_search.rs:583-596`, `:603-618`) then *drop the `N!` ordering
  factor*; `own_anchored_unrank` with `sorted: true`
  (`permutation_search.rs:701-730`) emits each subset **once, in identity
  order**.
- But `compute_wallet_policy_id` never sorts — the code says so itself at
  `restore.rs:1951-1953` ("the recorded id pins a SPECIFIC order the search must
  still resolve. Verified: sortedmulti AB-id ≠ BA-id"), and it is measured:
  the same three keys give `a7cb4596a710a69b69ee2d4b8e937660` in order `A,B,C`
  and `c3b1b27a5c51f77828d0ec4280eb10bb` in order `B,C,A`.

So on the subset paths the id-search enumerates only identity-ordered
assignments while the target id is order-dependent. Independent confirmation
that the ordering factor really is dropped, via the threshold the tool prints
for `--own-account-max 256` (j=1, m=2):

```
wsh-sortedmulti : need ≥5 bytes   (S = C(256,1) = 256)
wsh-multi       : need ≥6 bytes   (S = C(256,1)·3! = 1536)
```

**What this spec changes.** Today the short-prefix operator never reaches the
false negative, because `validate_prefix_strength` refuses first. Measured on
the failing combination with a 4-byte prefix:

```
error: restore: multisig-template-floor mismatch — derived --expect-wallet-id
prefix too weak for this search: need ≥5 bytes (sized to the realized search
space), got 4
```

After this spec that refusal is gone and the same operator gets
`✗ NO MATCH` + `derived no key→slot assignment of the supplied keys` (exit 4).
§2's table promises this input either a candidate list or a warn-and-reconstruct;
it can deliver neither, because the true wallet is not in the enumerated space.
§8's plan to re-point `own_account_max_short_id_prefix_refuses` to "the §3.8
warning path" survives only because that test's fixture happens to be
`wsh-multi` (`tests/cli_restore_md1_template_multisig.rs:1001`); with the §1
fixture it would assert a cell that cannot occur. The implementability lens's
**L13** noticed the collapse but classified it as a fixture-sizing trap only; it
did not reach the false negative.

Aggravating: `bundle` tells this operator the opposite. Measured, verbatim, on
the same template — `note: this is an ORDER-INDEPENDENT (sortedmulti) template —
the cosigner key order does not change the wallet, so any assignment of keys to
slots reproduces it.`

**Classification: refusal** (at minimum) — the combination `id-search + subset
path + order-independent shape` must refuse, naming the collapse, rather than
scan and report a definitive negative. The honest fix is to stop passing
`sorted: true` to the enumeration on the id-search path (address-search keeps
it, correctly), but that is code, not spec; the spec must at least not delete
the only message that was true.

**Why it clears the bar:** the operator is told, definitively and at a
recovery-blocking moment, that no assignment of their keys matches — when one
does. Today they are told exactly which digit count would work. Replacing a true
statement with a false one is strictly worse than telling them nothing, and the
documented next move after a `NO MATCH` is `--cosigner @N=` explicit placement,
which §4 itself names as the dangerous path (and which A2 shows does not check
the id they hold).

---

### A2 — `--expect-wallet-id` supplied together with `--cosigner @N=` is silently discarded, and the tool prints `✓ wallet-id (completed)` over a wallet the operator did not ask for

**Concrete input.** A 2-of-3 `wsh-multi`, true id `a693d0f4447150c94fce12d852c19af5`.
The operator asserts placements *and* supplies the id they recorded, which is
precisely what the tool's own warning tells them to do ("Record + check
`--expect-wallet-id` or a receive address").

Run 1 — a deliberately wrong id:

```
restore … --expect-wallet-id deadbeefdeadbeefdeadbeefdeadbeef \
          --cosigner @1=<mk1_B> --cosigner @2=<mk1_C>
→ exit 0
  ✓ wallet-id (completed): a693d0f4447150c94fce12d852c19af5
```

Run 2 — the **true** id, with the two cosigner cards swapped between `@1` and
`@2` (the exact mistake explicit placement exists to make possible):

```
restore … --expect-wallet-id a693d0f4447150c94fce12d852c19af5 \
          --cosigner @2=<mk1_B> --cosigner @1=<mk1_C>
→ exit 0
  descriptor: wsh(multi(2,[b8688df1/…],[3f635a63/…],[28645006/…]))
  ✓ wallet-id (completed): 4fd92f23d243c80ee55642dbd4b790c7
```

**Outcome.** A different wallet (`4fd92f23…` ≠ `a693d0f4…`), emitted at exit 0,
under a green `✓`. The operator supplied the one value that would have caught
it; the tool had it in hand and never compared it.

**Mechanism.** `restore.rs:1793` — `return complete_explicit_assignment(d,
&own_keys, &assigned_cosigners, stderr);`. The function is not passed `ctx`, so
`ctx.expect_wallet_id` is structurally unreachable on that path. There is no
`conflicts_with` and no cross-check anywhere.

**Why the spec owns this.** §3.4a(a) walks straight past it: it enumerates the
three completion modes reached through the single emitter call site
(`restore.rs:1436`), rules `uniqueness_proven` **absent** for explicit `@N=`
because "explicit `@N=` placement proves nothing", and stops there. The
observation is right and the conclusion is half of one: the mode proves nothing
*because the spec leaves the proof lying unused on the argv*. §3.7 establishes
the governing principle in this very cycle — a flag combination whose advice is
a silent no-op must refuse — and applies it to `--search-address` only.

**Classification: refusal or warning.** Either cross-check the recomputed id
against the supplied prefix and fail with `RestoreMismatch` (exit 4) on
mismatch — the code already computes the id, and already prints it — or refuse
the combination at clap. Refusing is the smaller change; checking is the better
one, and is what the existing warning already promises.

**Why it clears the bar:** the current output is an affirmative claim — a `✓`
line and exit 0 — attached to a wallet that is not the one the operator named.
Receiving to it loses funds. Printing nothing, or refusing the flag pair, is
strictly better than a check mark on the wrong wallet.

---

### A3 — the `order_independent` annotation is derived from a shape flag that cannot see key-set differences, so on the subset paths it asserts "one wallet, N labellings" over N genuinely different wallets

**Concrete input.** `wsh-sortedmulti`, 3 slots, one own slot, with the
enumerate-band prefix the feature is built for:

```
restore --md1 <sortedmulti template> --from phrase=<seed> \
        --own-account-max 256 --cosigner <mk1_B> --cosigner <mk1_C> \
        --expect-wallet-id e1cc          # 2 bytes, the §3.1 floor
```

Realized `S = C(256,1) = 256` (measured above via the printed threshold). At the
2-byte floor λ = S·2⁻¹⁶; widen to three own slots (`--own-account-max 256`,
n=3 all-own, `S = C(256,3) = 2,763,520`) and λ ≈ 42 — a multi-row list is not a
coin flip, it is the expected outcome, and §3.5's 64-row cap bites.

**Outcome the spec mandates.** §3.3 (L5 fold) and §3.4a(c): one line above the
rows, *"derived from the SHAPE FLAG and not from comparing addresses"* —
`is_order_independent_shape(&d.tree)` — telling the operator that "every match is
the same wallet under a different labelling", plus `"order_independent": true`
in the JSON envelope.

**Why that is false here.** On the exact path every match is a permutation of
one key set, so the claim holds. On the subset paths the matches differ in
**which keys are in the wallet** — a different own account, or a different
cosigner subset — not in their order. The repo states the consequence itself, in
the comment justifying address-search early-exit at `restore.rs:2085-2090`:
"distinct subsets ⇒ distinct key SETS ⇒ distinct scriptPubKey". Different
scriptPubKey is a different wallet, receiving to different addresses.
`is_order_independent_shape` (`synthesize.rs:339-350`) inspects only the
descriptor tree; it has no access to the assignment, so it cannot distinguish
"same keys, different order" from "different keys". The spec chose the flag
precisely to avoid deriving addresses before the cap (§3.5) — that reasoning is
sound for the cost question and wrong for the truth question.

**Classification: refusal to annotate** — suppress the line and the JSON field
whenever the enumeration is `OwnAnchored` or `OptIn`, or qualify it to
"orderings of the same key set are one wallet; these rows are different key
sets". The safe default is no annotation: an operator who reads N rows as N
distinct candidate wallets is correct on every path.

**Why it clears the bar:** the annotation exists only to change what the
operator concludes, and on a reachable path it changes it to the wrong thing —
"they are all the same wallet, pick any row" over rows that are different
wallets. The list's whole funds-safety argument (§4: "it reconstructs nothing")
is undone by a sentence telling the reader the rows are interchangeable. Saying
nothing leaves them correct.

---

## IMPORTANT

### A4 — §3.7's mutual exclusion is specified for one surface; `verify-bundle` shares the dispatch and keeps silently ignoring `--search-address`

**Concrete input.**

```
verify-bundle --network mainnet --ms1 … --mk1 … --md1 <template> \
              --expect-wallet-id e1ccd788fe --search-address bc1q…
```

**Outcome.** Accepted. Measured: `verify-bundle --md1 … --expect-wallet-id
aabbccddee --search-address bc1q…` gets past clap (the only error is about the
other required args), and `grep -n conflicts_with` shows no rule between the two
on **either** surface. `verify_bundle.rs:946-947` copies both fields into the
same `MultisigCompletionCtx` that `restore.rs:1427-1428` builds, and the
dispatch at `restore.rs:1943-1945` is `if id_search { … } else if addr_search { … }`
— so the address the operator supplied is discarded, and the bundle is reported
against the id alone.

**Why the spec owns this.** §3.7 requires `conflicts_with` and names no surface;
§5 discusses `verify-bundle` at length (the `allow_enumerate: bool` ruling,
"both callers forced to choose by the compiler") and never mentions the flag
pair. A fix applied only to `restore`'s clap derive leaves the verifier with the
defect — on the surface where it matters more, because a verifier's output is
consumed as "these two independent facts both checked out".

**Classification: refusal**, on both surfaces, worded per §3.7.

**Why it clears the bar:** a verifier that reports PASS while silently dropping
one of the two targets it was given tells the operator something false about
what was verified. Refusing tells them the truth, and the spec has already
decided that this is the right answer — it just did not carry it across.

### A5 — the §3.8 mandatory warning's remedy becomes a usage error once §3.7 lands

**Concrete input.** The operator hits the C1 path (lone match below threshold),
reads the un-suppressible warning, and does what its last line says:

```
! Before receiving to this wallet, confirm the first address below against
! a source you already trust, or re-run with --search-address.
```

They press up-arrow and append the flag:

```
restore --md1 … --from … --expect-wallet-id e1ccd788 --search-address bc1q…
```

**Outcome.** After §3.7 lands this is a clap conflict — a usage error (exit 2),
no search, no verification — at the exact moment the tool has just told them
their wallet might be wrong.

**Why the spec owns this.** R0's I7 identified this trap for §3.3's summary
line, and the fold fixed §3.3 by making the word **"instead"** load-bearing
("re-run with `--search-address` **instead**", with an explicit note that the
hint "must not read as 'add `--search-address` to what you just ran', which
would refuse"). §3.8's block quote was not given the same treatment and still
reads "or re-run with `--search-address`." The same defect, in the one message
the spec declares mandatory and un-suppressible, and the only escape hatch the
C1 operator ruling leaves the operator. The `verify-bundle` refusal text §5
requires ("verify refuses, restore enumerates") needs the same audit.

**Classification: documentation only** — one word, in §3.8's normative block,
plus a check that no other message in this cycle tells the operator to add
`--search-address` to a command that already carries `--expect-wallet-id`.

**Why it clears the bar:** the C1 ruling's entire mitigation is that the warning
is mandatory *and* actionable. A remedy that terminates in a usage error is
worse than no remedy, because the operator concludes the verification step is
broken rather than that they typed the wrong shape.

---

## MINOR

### A6 — `realized_space` and `match_count` as bare JSON numbers exceed IEEE-754 integer precision for spaces the spec itself cites
§3.4a(c)'s envelope emits `"realized_space": 6`. §2.1's own table reaches
`S = 66,902,793,897,139,200` for `--own-account-max 32` (n=11, own=4), which is
`> 2^53 = 9,007,199,254,740,992`. Any consumer on a double-typed JSON parser
(JavaScript, `jq` without `--raw-output` arithmetic care) silently reads a
different number. Classification: **default** — emit the two large-domain fields
as strings, or cap the printed value. Does not change which wallet anyone gets,
hence Minor rather than blocking; it does corrupt the one number the whole
prefix-sizing story is told with.

### A7 — the below-floor refusal reports the *threshold*, not the floor that actually refused
Measured, 1-byte prefix over `S = 6`:
`prefix too weak for this search: need ≥5 bytes (sized to the realized search
space), got 1`. After this spec, 2 bytes is admitted and would list; the message
still demands 5. It is not false (5 does work) but it contradicts the feature
being shipped, and an operator reading it concludes short prefixes are still
refused outright. §3.1 says the floor refusal keeps `PrefixTooShort` and never
says what `required` should carry. Classification: **documentation only** — the
floor refusal should name 2 (the floor that fired) and 5 (what buys a proven
result).

### A8 — three refusal classes on this flag exit 1, and §3.4's exit-code table lists none of them
Measured on `decode_wallet_id_prefix` (`restore.rs:2461`): empty string and
whitespace-only → `exit 1` (`must not be empty`); odd-length hex → `exit 1`;
`0x`-prefixed and non-hex → `exit 1`; **more than 16 bytes** → `exit 1`
(`prefix is 17 bytes; the WalletPolicyId is only 16 bytes`). §3.4's table
enumerates 0/1/4 for six outcomes and covers none of these, while §3.1 mentions
only the odd-length case and defers it. A script that switches on this flag's
exit codes now has three undocumented arrivals at 1 alongside the documented
`Ambiguous`-at-threshold 1. Classification: **documentation only** — add the
decode-refusal row.

---

## NIT

### A9 — `required_prefix_bytes` can exceed 16, making §3.8's "supply more id" literally impossible
`required_prefix_bytes(S) > 16` for `S > 2^96`. Reachable arithmetically at
`--own-account-max 256` with 15 slots: `C(256,10)·15! = 3.6e29 → required = 17`.
The full 16-byte id would then sit *inside* the enumerate band and, if the scan
ran, produce the §3.8 warning saying "the supplied `--expect-wallet-id` is 16
bytes; this search space needs 17" plus a `supply more id` hint against an id
that has no more bytes. Not actually reachable: at ~1 µs/candidate the 1-hour
`SEARCH_CEILING` admits ≈3.6e9 candidates, four orders of magnitude short of
`2^96`, so §3.4a(f)'s ceiling refusal always fires first. Worth one clause
guarding the message (`supplied == 16` ⇒ drop "supply more id").

### A10 — §3.3's summary line advertises the *only* remedy that works for A1, and the `NO MATCH` path never prints it
The list path's hint ends "…or re-run with `--search-address` instead", and
address-search is exactly what succeeds on A1's failing combination (the sorted
collapse is *correct* for an address target — same scriptPubKey under every
ordering). The 0-match path prints only `✗ NO MATCH` plus
`expected the recorded wallet (--expect-wallet-id / --search-address)`, which
names the flag without proposing it. Folding A1 may make this moot; if A1 is
answered with a refusal rather than a fix, the `NO MATCH` copy should carry the
same "instead" hint.

---

## Checked, no finding (so the next lens does not re-walk these)

- **Prefix boundary matrix**, measured end-to-end against `S = 6` (`required =
  5 B`): `2 hex`→exit 4 · `4 hex` (floor)→exit 4 · `8 hex`→exit 4 · `10 hex`
  (`== required`)→exit 0, reconstructs · `32 hex` (full)→exit 0 · `34 hex`→exit
  1 · odd 5 hex→exit 1 · UPPERCASE `A693D0F444`→exit 0, reconstructs ·
  `" a693d0f444 "` (surrounding whitespace)→exit 0, trimmed by
  `decode_wallet_id_prefix`'s `s.trim()` · `""` and `"   "`→exit 1 ·
  `zzzzzzzzzz`→exit 1. Case-insensitivity and trimming are correct and
  undocumented; both fail safe.
- **`--own-account-max 0`** → refused, `must be ≥ 1` (`restore.rs:1485-1489`).
- **`--own-account-max 300`** → refused at the 256 hard ceiling.
- **Every cosigner key identical** → `reject_duplicate_keys`
  (`restore.rs:1940`) refuses on the whole pool *before* the search, so a
  degenerate pool never reaches the enumerate band.
- **A space of size 0** (own candidates fewer than own slots) → refused
  upstream: `not enough own-account candidates to fill the own slots: raise
  --own-account-max, or supply the missing --cosigner card(s)` (exit 2).
  Measured; `EmptySearchSpace` is unreachable from this surface.
- **A space of size 1** (`--own-account-max 1`, one own slot, sorted shape:
  `S = C(1,1) = 1`) → `required = 4` (the `S ≤ 1` clamp), so a 2–3 byte prefix
  enters the enumerate band over a single candidate. Lone match reconstructs
  with the §3.8 warning; the warning's text ("spurious when the true wallet is
  NOT among the keys you supplied") is the correct thing to say about a
  single-candidate recompute. No change needed.
- **A 1-slot template** → unconstructible: `bundle` refuses with `multisig
  template 'wsh-multi' requires N > 1; use a single-sig template for N=1`.
- **`--expect-wallet-id` + `@N=` + `--search-cosigner-subset`** → already
  refused (`restore.rs:1620-1626`): explicit placement is mutually exclusive
  with both subset modes. Only the plain `@N=` + id pair is unguarded (A2).
- **`--expect-wallet-id` + `--search-cosigner-subset`** (no `@N=`) → composes
  and works on a non-sorted template; the sorted case is A1.
- **All `S` assignments matching with `S < 64`** → `match_count = S`,
  `truncated: false`; no contradiction in §3.4a(d). Requires a 0-byte prefix to
  be forced, which the floor refuses.
- **`u128` cardinality vs the `u64` cap** → `restore.rs:2274` saturates with
  `u64::try_from(realized_total).unwrap_or(u64::MAX)`, so an oversized space
  produces a ceiling refusal rather than a wrapped estimate.

---

## What a fold must not skip

1. A1 and A3 share one root — `sorted_shape` is fed to the *enumeration* on both
   subset paths regardless of search mode, and `is_order_independent_shape` is
   shape-only. Fixing A1 by refusing the combination does **not** fix A3, and
   fixing A3 by dropping the annotation does not fix A1. They need separate
   answers.
2. A1's evidence is a five-cell matrix, not one run. Any fold claiming it is
   handled should reproduce all five (sortedmulti/multi × exact/oam/subset) —
   the non-sorted rows are the control that proves it is the flag and not the
   subset search.
3. A2 is one `if` away from closed and one `conflicts_with` away from refused;
   pick one and add the vector, because §6 currently has no case where an id is
   supplied and *not* consulted. §6 vector 7 already runs an `@N=` invocation —
   it asserts `uniqueness_proven` is absent, which passes today and would keep
   passing with the wrong wallet emitted beside it.
4. A5 is one word in a normative block; grep every message this cycle adds for
   `--search-address` and check each reads as a replacement, not an addition.
