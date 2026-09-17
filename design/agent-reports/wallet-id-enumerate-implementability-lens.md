VERDICT: NO-GO

Critical: 2 / Important: 7 / Minor: 6 / Nit: 2

# Implementability lens — SPEC_restore_wallet_id_prefix_enumerate.md

**Lens:** *If a competent implementer sat down to build this tomorrow with only
this document, where would they have to GUESS?* Not correctness (closed, round 1),
not fold fidelity (closed, round 2). Every finding below names **the fork in the
road and the two different programs that result**; anything that could not name
two divergent implementations was dropped rather than reported.

**Scope honoured.** The operator ruling on the lone-match cell (§2.1), §5's
verify-bundle ruling, and every `file:line` / arithmetic claim are treated as
fixed. `bash scripts/spec-citation-gate.sh` was run: **GATE: PASS** (16 citation
rows, 6 numeric rows). Nothing below re-derives a gated fact.

**Source read to determine what an implementer actually finds:**
`crates/mnemonic-toolkit/src/cmd/restore.rs`,
`crates/mnemonic-toolkit/src/permutation_search.rs`,
`crates/mnemonic-toolkit/src/derive_address.rs`.

**The shape of the result.** The spec is strong on *decision* and thin on
*output*. Every Critical and all but one Important sits in the gap between "the
tool lists the matches" and "here are the exact bytes on each stream". The spec
has a semantics section and an exit-code table; it has no output contract.

---

## CRITICAL

### L1 — §3.4 adds `uniqueness_proven` to an envelope shared by three completion modes, and the table is keyed on a prefix that two of them do not have

**The fork.** §3.4's table rows are keyed on *prefix length*
(`≥ threshold` / `< threshold`). The envelope those rows modify is built in
exactly one place — `emit_completed_multisig` (`restore.rs:2407-2417`) — and that
function is reached from **three** completion modes, not one:

- id-search (`restore.rs:1943`, the prefix path),
- address-search (`restore.rs:1944`),
- **explicit `--cosigner @N=`**, which returns early from
  `complete_multisig_template` at `restore.rs:1793` via
  `complete_explicit_assignment` and lands at the same emitter call
  (`restore.rs:1436`).

`MultisigCompletionOutcome` (`restore.rs:1207`) carries `completed`, `pool`,
`assignment` — **no mode discriminant**. So the emitter cannot tell which mode
produced the outcome, and §5 does not ask for one.

**Program A** (the path of least resistance, and the only one buildable from
§5's change list as written): add `"uniqueness_proven": <bool>` unconditionally
to the envelope, computed as "false iff we took the short-prefix lone-match
path, true otherwise". Then:

```
mnemonic restore --md1 <tmpl> --from <seed> --cosigner @0=<xpub> --cosigner @1=<xpub> --json
```

emits `"uniqueness_proven": true` for the **explicit `@N=` mode** — the one mode
whose own stderr warning (`restore.rs:2348-2354`) reads verbatim:

> `warning: explicit --cosigner @N= mode builds the wallet from the ASSERTED
> key→slot assignment WITHOUT verifying it against a recorded id/address. A
> wrong assignment produces a wrong wallet silently.`

**Program B:** extend the outcome type with a mode/provenance field so the key is
emitted only when an id-search actually ran; explicit and address-search
envelopes carry no `uniqueness_proven` key at all.

**Why Critical.** §2.1's own binding consequence is that the un-provability
*"must reach the machine-readable channel too… the exact consumer most likely to
act on a wrong wallet unattended."* Program A puts the **opposite** claim in that
same channel for the mode that verifies nothing, and the only counter-signal is a
stderr line that §2.1 has already argued is invisible to that consumer. That is an
unmet guarantee asserted machine-readably, and it is reachable today with two
`--cosigner @N=` flags and `--json`.

The spec must say what `uniqueness_proven` means for a completion that had no
prefix — and §5's change list must carry whatever field makes that expressible,
because the outcome struct cannot express it now.

---

### L2 — §3.5's cap is specified for "the printed list"; under `--json` it truncates `candidates` with no marker the table defines

**The fork.** §3.5: *"Cap the printed list at 64 matches; beyond that print the
first 64 and `… and K more; supply more id`."* §3.4's list row specifies the JSON
payload as **`candidates: [ … ]` and NO `wallets` key at all** — and defines no
truncation field, no total, no `K`.

**Program A** — the cap is a cap: `candidates` holds at most 64 entries in JSON
too (this is also the only reading consistent with §3.5's *"apply the cap BEFORE
deriving addresses"*, since deriving for all N to fill JSON is exactly the
unbudgeted work R0 M6 removed). A machine consumer receives 64 objects, iterates
them, finds nothing it recognises, and concludes **"my wallet is not among these
keys"** — when the true wallet was match #65 of 500.

**Program B** — the cap is a *display* cap: text mode prints 64 + `… and K more`,
`--json` emits all N candidates with their addresses. The ceiling rationale is
violated and a large list can dwarf the id-scan cost that `cap_decision`
(`restore.rs:2276`) budgeted.

**Program C** — capped, plus an invented `"truncated": true` / `"total_matches":
N` field. Diverges from both A and B for every consumer.

**Why Critical.** A is silent truncation of a result set in the machine-readable
channel, with the "not found" and "found, but you were not shown it" cases
indistinguishable — the project's own `empty-output-is-not-absence` /
`a-count-from-a-broken-pipeline-reads-as-zero` class. The text path is protected
by `… and K more`; JSON is protected by nothing the spec writes down. Reachable
whenever §6 vector 6's own fixture condition holds (`S > 4,194,304`).

---

## IMPORTANT

### L3 — §3.3's candidate rows have no stream. The spec assigns stderr to the *summary line only*

**The fork.** The word "stdout" appears twice in the whole document (lines 111,
390), both about the *summary hint* and the machine-readable channel. §3.3 says
the summary line goes *"After the rows, on **stderr** (§8)"* — specifying a
stream for the summary and saying nothing about the rows. §8 resolves only
*"stdout or stderr for the summary hint?"*.

**Program A** mirrors the existing emitter (`restore.rs:2424-2434`, where
`descriptor:` and `first recv:` go to stdout and `✓ wallet-id` goes to stderr):
candidate rows → **stdout**, summary → stderr.

```
$ mnemonic restore --md1 … --expect-wallet-id 72d9 >rows.txt 2>/dev/null; wc -l rows.txt
6 rows.txt
```

**Program B** reads §8's *"stdout stays the machine-readable channel"* plus §2's
**NO descriptor** as meaning text-mode stdout is empty for the list outcome: rows
→ **stderr** too.

```
$ mnemonic restore --md1 … --expect-wallet-id 72d9 >rows.txt 2>/dev/null; wc -l rows.txt
0 rows.txt
```

An operator who pipes to a file gets their candidates or gets an empty file and
exit 0. Every §6 vector that inspects the rows asserts against a different
stream in the two programs.

---

### L4 — §3.3's sortedmulti grouping and §2's match-count table disagree about whether "N rows, one wallet" reconstructs

**The fork.** §2's table classifies on **match count**: exactly 1 → reconstruct +
warning + descriptor; ≥2 → list, **NO descriptor**. §3.3 says that for a
`sortedmulti` shape *"each ordering has a different id — but every ordering
yields identical addresses and identical spending… the honest answer is '1
wallet, N labellings'."* (Confirmed in code: the sorted collapse
`apply_identity_filter` is applied to the **address** evaluator only; the
id evaluator at `restore.rs:2005ff` is explicitly *not* collapsed, so distinct
orderings carry distinct ids.)

Take a short prefix that matches 2 orderings of the same sortedmulti key set:

**Program A** classifies first, groups second: 2 matches → list, no descriptor,
`candidates: [...]`, exit 0.

**Program B** takes §3.3's "the honest answer is 1 wallet" as normative and
groups before classifying: 1 group → the "exactly 1 match" cell → **reconstruct**,
emit the descriptor, fire the §3.8 warning, `wallets: [...]`, exit 0.

Same command, same keys, same prefix: one program hands the operator a wallet,
the other hands them a list. `.wallets[0].descriptor` exists in B and is absent
in A — the precise distinction §3.4 says the `candidates` key exists to make
loud. The spec must state whether grouping is a *display* transform applied after
classification or a *semantic* one applied before it.

**Second fork inside the same sentence.** §3.3 offers two remedies with an "or":
*"Rows sharing a first address must be grouped, **or** annotated `same wallet,
different labelling`."* Grouping collapses N rows into 1; annotating keeps N
rows. §6 vector 1 requires *"exactly N rows"* — unwritable until the spec picks
one, and the §3.5 cap counts a different number in each.

---

### L5 — §3.5 ("cap BEFORE deriving addresses") and §3.3 ("group rows sharing a first address") cannot both be satisfied

**The fork.** Grouping by first address requires the first address of **every**
match. §3.5 (R0 M6) forbids deriving addresses for every match before the cap,
because a descriptor build + miniscript parse + `script_pubkey_at` per match is
materially more than the id evaluator `calibrate_per_candidate`
(`restore.rs:2270`) timed. Compose the two into one routine and they contradict.

For 100 matches that are all one wallet under 100 labellings:

**Program A** (grouping wins): derive all 100 addresses, group → 1 group, print
`1 wallet, 100 labellings`. §3.5's budget rule is violated; the address work sits
outside the ceiling exactly as R0 M6 described.

**Program B** (cap wins): truncate to 64 by `perm_rank`, derive 64 addresses,
group within the survivors, print `… and 36 more; supply more id`. Grouping is
now wrong across the cut — a shown row and a cut row may be the same wallet, and
the operator is told there are 36 further *wallets* when there are zero.

**And the unanswered sub-question the brief names:** does `64` count **rows** or
**groups**? §3.5 says "64 matches"; §3.3 says rows may be collapsed into groups.
`K` in `… and K more` is a different number under each reading.

---

### L6 — `--count` exists, feeds the existing emitter, and the spec never mentions it

**The fork.** `RestoreArgs.count` is a real flag — `restore.rs:220`,
`#[arg(long, default_value_t = 1)]` — and the current multisig emitter passes it
straight into address derivation (`restore.rs:2394-2398`,
`derive_receive_addresses(&parsed, args.count, …)`), printing one `first recv:`
line per address. §3.3 specifies *"the first receive address"*, singular, for a
candidate row. The spec never names `--count`. (Two stale comments in the file,
`restore.rs:2516` and `:2824`, even claim *"restore has no --count flag"* — an
implementer grepping for the flag meets a contradiction in the source itself.)

`mnemonic restore --md1 … --expect-wallet-id 72d9 --count 5` with 6 matches:

**Program A** honours `--count` per row, consistent with the reconstruct path:
6 rows × 5 addresses = 30 addresses; JSON `first_addresses` arrays of length 5.

**Program B** takes §3.3 literally: 6 rows × 1 address, and `--count` silently
does nothing on the list path while still working on the reconstruct path one
prefix-byte away.

This compounds L4/L5: if the grouping key is "the first address", A must decide
whether it groups on address[0] or on the whole set, and B has no such choice.
It also compounds L5's budget: A multiplies the capped address-derivation work by
`--count`, which no ceiling accounts for.

---

### L7 — §3.4 claims to enumerate the outcomes but omits the one this change creates: short prefix + a space over the search ceiling. The exit code silently moves 4 → 1

**The fork.** §3.6 records that the refusal *"moves from instant to post-scan"* —
today `validate_prefix_strength` (`restore.rs:2009`) refuses before
`run_capped_search` (`restore.rs:2024`); afterwards the operator reaches
calibration and meets `SearchTimeExceedsCeiling`. §3.6 then requires only that
*"that error must say so"* — a **message** requirement. It says nothing about the
exit code, and §3.4's table — the document's authority on exit codes, opening
*"Text and JSON must agree"* — has no row for this outcome.

The codes are not the same. `map_search_error` (`restore.rs:2224-2237`):
`PrefixTooShort` → `RestoreMismatch` → **exit 4**;
`SearchTimeExceedsCeiling` → `bad()` → `BadInput` → **exit 1**.

`mnemonic restore --md1 … --own-account-max 32 --expect-wallet-id 72d94d49`:

**Program A** implements §3.6 as written — improve the ceiling message, leave the
mapping alone. The operator's script sees the exit code for this input change
from **4 to 1** with no row in §3.4 authorising it, and `1` is the code §3.4
assigns to `✗ AMBIGUOUS`.

**Program B** treats §3.4's table as exhaustive, notices the regression, and
re-maps the ceiling error back to `RestoreMismatch` (exit 4) when the prefix is in
the enumerate band — preserving the script contract but making the same engine
error exit differently depending on prefix length.

Two programs, two exit codes, same command. §3.4 needs the row.

---

### L8 — the `candidates[]` object shape is never specified: not its field names, not whether a candidate carries a descriptor, not which top-level keys survive

**The fork.** §3.4 names the **key** and one prohibition (no `wallets`). §3.3
names three things for the **text** render (full 16-byte id, the `@0=<fp>, @1=<fp>`
assignment, the first receive address). Nothing maps text to JSON. There is no
precedent to copy: `grep -rn '"candidates"' crates/ --include='*.rs'` returns
**nothing** — the implementer invents the whole shape.

The existing envelope (`restore.rs:2408-2417`) is
`{network, completed_from, wallet_policy_id, own_position, wallets[{descriptor,
first_addresses}]}`. For a list there is no single `wallet_policy_id` and no
single `own_position`.

**Program A:**
`{network, completed_from, candidates: [{wallet_policy_id, assignment, first_addresses}]}`
— top-level `wallet_policy_id`/`own_position` dropped, no descriptors anywhere,
no `uniqueness_proven`, no `prefix_bytes`.

**Program B:**
`{network, completed_from, wallet_policy_id: <the supplied prefix>, own_position,
uniqueness_proven: false, prefix_bytes, required_bytes,
candidates: [{descriptor, wallet_policy_id, own_position, first_addresses}]}`.

Three consumer-visible divergences, each a different program:

1. **Does a candidate carry its `descriptor`?** §2's table says "NO descriptor"
   and §4 leans on the list path *"reconstruct[ing] nothing"* for the
   unconditional half of the funds guarantee — but §3.4's row forbids only the
   `wallets` key. B's consumer can lift `.candidates[0].descriptor` and import it,
   which is the hazard the list path exists to prevent. The spec should say this
   in §3.4's row, not leave it to be inferred from §2's table.
2. **Does the list envelope carry `uniqueness_proven: false`?** §3.4 puts it only
   on the lone-match row. A consumer branching on `uniqueness_proven === false`
   aborts under B and falls through under A.
3. **Do the top-level `wallet_policy_id` / `own_position` keys survive?** A
   consumer reading `.wallet_policy_id` gets `undefined` or the supplied prefix.

---

### L9 — §3.3 gives the summary line as a verbatim "exactly one line" template, then requires it to carry a fact the template has no slot for

**The fork.** §3.3 says *"emit exactly one line"* and supplies it verbatim:

```
N assignments match prefix <hex> (of S in the realized space); none
reconstructed — supply more id, or re-run with --search-address instead.
```

Then immediately: *"`S` is the **realized** cardinality — `n!`, `s_own` or
`s_opt` depending on flags (R0 N2) — and the line must name which, since the
whole prefix-sizing story hangs on that number."* The template has no place to
name which.

**Program A** emits the template verbatim:
`6 assignments match prefix 72d9 (of 6 in the realized space); none reconstructed — …`

**Program B** satisfies the prose:
`6 assignments match prefix 72d9 (of 6 = 3! (full permutation) in the realized space); none reconstructed — …`

**What the implementer must invent.** `Enumeration` (`permutation_search.rs:845ff`)
exposes exactly two accessors — `n()` (`:848`) and `cardinality()` (`:862`). There
is **no** name/kind accessor and no `Display`, so B must add one and choose the
three strings (`n!` / `s_own` / `s_opt`? "full permutation" / "own-anchored" /
"opt-in"?). §6 has no vector on this line, so nothing catches the divergence.

---

## MINOR

### L10 — `warning: "<the §3.8 text>"` does not pin the bytes a test would assert
§3.8 gives the warning as a six-line block with a leading `! ` on every line. §3.4
puts *"the §3.8 text"* in a JSON string. **Program A** emits the block verbatim,
`! ` prefixes and `\n` intact, inside the JSON string. **Program B** flattens it to
one prose sentence without the gutter, on the grounds that `! ` is a terminal
gutter and JSON is not a terminal. §6 vector 2 asserts the warning on **stderr**
and only `uniqueness_proven: false` in JSON, so nothing pins the field either way,
and a consumer grepping `.warning` behaves differently against the two builds.

### L11 — does the §3.8 warning also reach stderr under `--json`?
§3.8 says *"unconditionally"*; §2.1 says it must reach the machine channel
*"too"*, which leans toward both; §3.4's table assigns it to the JSON column.
**Program A** writes the six-line block to stderr *and* the envelope. **Program B**
writes it only into the envelope, so `mnemonic restore --json … 2>err.txt` leaves
`err.txt` without the mandatory warning. "Un-suppressible" should say so per-stream.

### L12 — the `@N=<fp>` row is degenerate for bare-xpub cosigners
`decode_cosigner_card` (`restore.rs:2120ff`) gives a bare xpub
`Fingerprint::default()`. A pool of bare xpubs renders every §3.3 row as
`@0=00000000, @1=00000000, …` — identical across candidates, with only the id
distinguishing them, so the assignment column (the column that tells the operator
*which card goes where*) carries no information on exactly the path that has no
mk1 metadata. **Program A** prints `<fp>` as specified. **Program B** substitutes a
pool index or an xpub prefix when the fingerprint is zero. Note this sits next to
the open follow-up `c972a46d` ("a bare-xpub `--cosigner` fails NO MATCH with no
hint about origin metadata").

### L13 — §6 vector 1's suggested fixture route interacts with the sorted collapse
Vector 1 suggests widening `S` with `--own-account-max`. For an order-independent
shape, `Enumeration::OwnAnchored{ sorted: true }` and `s_own(...)`
(`permutation_search.rs:866-873`) **drop the ordering factor** — each subset is
emitted once in identity order — so a `sortedmulti` fixture widens far less than
the raw count suggests and cannot manufacture the ordering-collision the vector
wants. §6 does hedge (*"or inject a stub evaluator"*), so this is a fixture-sizing
trap rather than an unbuildable vector, but the spec should name the shape
constraint (`wsh-multi`, unsorted) alongside the `--own-account-max` advice.

### L14 — §3.3's sort key is dead in its second component, and reverses the code's existing tie-break
§3.3 pins `(permutation_index, address_index)` and justifies the pair *"since an
address-mode match carries both"*. But enumerate runs only on the id path, where
`address_index` is hard-coded `0` (`permutation_search.rs:1069-1072`,
`SearchMode::Id => 0`), and §3.7 makes `--expect-wallet-id` and
`--search-address` mutually exclusive — so the second component can never vary.
Separately, the engine's own deterministic tie-break is the **reverse** order,
`address_index` then `perm_rank` (`permutation_search.rs:1098-1102`). Behaviourally
inert today; the risk is that a later reader takes §3.3's justification as evidence
the enumerate path can be address-mode.

### L15 — the determinism oracle is not in §5's change list
`search_reference` (`permutation_search.rs:1128`) is the stated *"determinism
oracle… the parallel `search` must agree with it on the outcome for every
input"*, and it short-circuits at two matches by construction (`:1160-1164`,
*"Two matches is enough to decide Ambiguous; stop"*). Collect-all gives the
parallel engine an outcome the reference cannot produce. **Program A** extends
the reference too and keeps the parity property meaningful. **Program B** leaves
it, and the parity tests (`:1390`, `:2196`, `:2215`) quietly stop covering the new
mode. Also note `search_reference` takes `n: usize`, not an `Enumeration`, so it
only ever oracled `FullPermutation` — worth stating rather than discovering.

---

## NIT

### L16 — "before the wallet block" is a cross-stream ordering claim
§3.8 requires the warning *"before the wallet block"*. The wallet block is stdout
(`restore.rs:2432`); the warning is stderr. Ordering between two independently
buffered streams is not observable except in a merged capture, where it depends
on flush points. Say "on stderr, before any stdout write" if the order is meant
to be testable; otherwise drop "before".

### L17 — the enum-vs-variant choice in §5 has one concrete blast-radius item worth naming
`MultisigCompletionOutcome::own_position()` (`restore.rs:1215-1219`) is an
inherent method on the struct. Turning the type into an enum re-homes it; adding
a field does not. Both construction sites (`restore.rs:1435`, `verify_bundle.rs:954`)
are exhaustive struct literals, which is what §5's `allow_enumerate: bool`
argument relies on — that argument holds under either refactor, so the choice is
free *except* for `own_position()` and for whether `verify_bundle.rs:954` must
write a defensive arm for a variant its `allow_enumerate: false` makes
unreachable (panic vs. error — unspecified, but unreachable by construction).

---

## What a fold must not skip

- L1 and L2 are the two that put a false or truncated claim in the machine
  channel. Both are fixed by writing the thing this spec does not have: a
  **§3.4a output contract** giving, for each of the three completion modes and
  both render modes, the exact top-level keys, the exact per-row keys, the stream
  for every line, and the truncation marker. Six of the nine blocking findings
  (L1, L2, L3, L6, L8, L9) collapse into that one section.
- L4 and L5 are a genuine composition conflict, not an omission: §3.3's grouping
  and §3.5's cap-before-derivation cannot both be honoured. One of them has to
  give, and the spec has to say which.
- L7 is a one-row addition to §3.4 plus a sentence in §3.6 about the exit code,
  not just the message.
- **The gate cannot reach any of this.** `scripts/spec-citation-gate.sh` verifies
  paths, lines and arithmetic — all PASS. Every finding above is about text the
  spec does not contain, which is the one class a citation gate is structurally
  blind to.
