# SPEC: `mnemonic xpub-search address-of-seed`

**Approval:** Approved by Brian 2026-11-26 20:30Z (§10 recommendations 1-9 accepted)

**Status:** DRAFT r3, 2026-11-26: review r2 (fable, RED: 0 C, 6 I, 13 M, 5 N)
folded on top of r1 (opus, RED: 1 C, 7 I, 15 M, 7 N); both disposition tables
are at the end. For Brian's rulings and an R0 re-review. No code exists.
**Base:** mnemonic-toolkit branch `claude/project-thread-utm1aa` at `40b35942`, binary 0.107.1.
**Ask (Brian, 2026-11-26):** a user holds one or more seeds and one or more known
addresses, and wants the wallet found by trying the common paths. No existing
`xpub-search` mode goes from a seed to an address.

## 1. What it does

Given seed(s) and target address(es), it searches the common derivation paths,
and for multisig also the key-to-slot map. On a match it prints the whole path
of each matched address and the whole `mnemonic bundle` command that mints that
wallet. It writes no cards and changes nothing.

**Name.** `xpub-search address-of-seed`. The family's names read "find X, given
Y" (`address-of-xpub` locates an address under an xpub), so this one locates an
address under a seed.

**Why not `restore --search-address`.** That flag completes a keyless md1
template for one own seed at one path family, needs a card, and takes one
address. This verb needs no card, takes many targets, and searches paths per
seed. It reuses the parallel driver of `permutation_search.rs` (threads,
calibration, progress), with a new enumeration and a new evaluator (§4.1).

**Multisig** here means two or more distinct keys. A wallet has one script form,
one key-to-slot map and one path per key.

**Not covered in v1:** guessing passphrases; a different passphrase per seed
(`bundle` has one `--passphrase`); hardened leaf steps (`/*h`, Bitcoin Core
legacy `m/0'/0'/i'`; refused, §2); one seed holding two slots; address lookups
on a node; the master key `m` as a key origin (`bundle` cannot record it:
"the master node cannot be recorded on a card for this descriptor shape");
BIP-45 cosigner-index wallets (`m/45'/cosigner_index/change/index`, BIP-45
lines 40 and 60-85): `bundle` refuses the extra fixed step ("a fixed single
step … is un-representable"), so no card and no command could follow a match
(FOLLOWUP `address-of-seed-bip45-cosigner-index`, filed with the plan);
Electrum-format seeds (not BIP-39).

**Found but not mintable.** A path with any step of 2²⁹ or more (hardened or
not) can be searched and matched, but md1 cannot record it: `bundle` refuses
`[73c5da0a/84h/0h/536870912h]` with "md1 varint overflow: 536870912" and
accepts `536870911h` (checked on 0.107.1; the limit is md1's LP4-ext varint,
descriptor-mnemonic `SPEC_v0_11_wire_format.md` F3). Such a match is reported
with its full path and no `bundle` command, and exits 4 (§5, §6). FOLLOWUP
`address-of-seed-md1-path-step-range`, with a companion in descriptor-mnemonic,
is filed with the plan.

## 2. Command line

```
mnemonic xpub-search address-of-seed
  --target-address <ADDR>            repeat; at least one
  one seed:   --phrase <P|@env:VAR> | --phrase-stdin | --ms1 <M> | --ms1-stdin | <MS1>
  2+ seeds:   --from <phrase|ms1|entropy|seedqr>=<value|-|@env:VAR>      repeat
  --passphrase <v|-|@env:VAR> | --passphrase-stdin     one, used for every seed
  --language <L>                     default english
  --descriptor <D> | --descriptor-file <F> | --descriptor-from md1=<chunk|->  (repeat)
  --cosigner <@N=xpub|mk1>           a key that is not a seed, slot stated
  --cosigner <xpub|mk1>              same, slot searched
  --min-account 0  --number-of-accounts 20  --max-account <N>  --add-path <T>
  --gap-limit 20   --external-only
  --independent-paths                multisig: each seed gets its own path (§4)
  --refugium-seats                   add the Refugium seat paths (§3; open question 3)
  --network mainnet   --json
```

- **One seed** uses the sibling modes' flags unchanged (`seed_intake.rs`,
  `SeedIntakeArgs`). **Two or more** use a repeatable `--from <node>=<value>`.
  This is new code: `restore`'s `--from` takes one value (`Option<String>`,
  `restore.rs:78`), and `convert`'s repeatable one has a different value type.
  The two groups conflict. Seeds are numbered from 0 in the order given
  (`seed 0`, `seed 1`, …), in text and JSON alike.
- **One stdin reader.** At most one value comes from stdin, across every
  `--from …=-`, `--phrase-stdin`, `--ms1-stdin`, `--passphrase -`,
  `--passphrase-stdin` and `--descriptor-from md1=-`; a second is refused
  (`refuse_second_stdin`, exit 2). The others use `@env:VAR`. The argv guard
  already covers `--from <secret>=` on every verb (`argv_guard.rs:296`).
- **No `--descriptor`** means single-sig: `pkh`, `sh(wpkh)`, `wpkh` and `tr`,
  each over `@0/<0;1>/*`. With two or more seeds, each seed is searched on its
  own and reported as its own wallet; "more keys than slots" does not apply.
  `--cosigner` without `--descriptor` is refused (exit 2): a single-sig wallet
  has no cosigner. The exit rule for several seeds is in §5.
- **Descriptor intake.** An `@N` template comes in through
  `parse_descriptor::lex_placeholders` and `resolve_placeholders`, because
  `descriptor_intake.rs:159-163` refuses `@N`. A keyed descriptor, a BIP-388
  policy JSON and md1 chunks (`--descriptor-from md1=`) come in through
  `descriptor_intake.rs`. `bundle --descriptor` itself refuses md1 ("md1 cards
  are not accepted on --descriptor"), so the printed command always carries
  descriptor text.
- **Accepted templates.** Every template that `parse_descriptor` accepts and
  `bundle` mints is searched; the evaluator is rust-miniscript at the
  toolkit's pin (`translate_pk` and `script_pubkey`, §4.1), not `restore`. That
  includes shapes `restore --md1` cannot derive addresses for today:
  `sortedmulti_a` in a multi-leaf tree, `sortedmulti` inside a combinator, and
  a key that is both the internal key and a leaf key (r2 measured them at the
  pin: 1.8 µs and 46 µs per candidate). §9 names the address oracle for those.
- **A keyless md1** is a template. Its recorded origin paths pin the search,
  as a typed `[fp/path]@N` does (below), and the report says the paths came
  from the card. If that search finds nothing, the no-match text adds that the
  card's paths may be `bundle`'s defaults and suggests passing the template as
  text without origins, which searches the full path set.
- **Private keys** (`xprv`, `tprv`, WIF) are refused at intake (exit 2) in
  `--descriptor`, `--descriptor-file` and `--cosigner`, and no error message
  echoes the value. The argv guard covers neither flag, so intake is where this
  is caught.
- **Use-site paths, per slot.** Each slot's multipath alternatives come from the
  template (`PlaceholderOccurrence::multipath_alts`, and
  `use_site_path_overrides` for per-`@N` overrides). "Chain" means "alternative
  k": `<0;1>/*` has two (step 0 and step 1), `<2;3>/*` has two (steps 2 and 3),
  a bare `/*` has one and no chain step, and `/**` means `<0;1>/*` (`bundle`
  accepts it). All slots of one descriptor have the same number of
  alternatives (BIP-389). `--external-only` keeps alternative 0 only. Refused
  as `bundle` refuses them (exit 2): a hardened wildcard `/*h`; a fixed single
  step (`/0/*`); a bare `@N` with no derivation after it (`bundle`: "a bare
  `@0` would be recorded as `@0/*` … Write `@0/<0;1>/*`").
- **A keyed descriptor or keyed md1** is not searched for its addresses: they
  follow from its keys, so the targets are confirmed directly in the gap
  window. The seeds are needed to write the `bundle` command, and are placed
  by **key derivation, not by fingerprint**:
  - for every (seed, key with a stated origin path) pair, derive the seed at
    that path and compare the xpub with the descriptor's key (k × n
    derivations, 89 µs each). Equal xpubs place the seed, whatever the path (a
    Refugium seat, account 25, a custom path) and whatever fingerprint the
    descriptor states;
  - a key with an empty origin (a bare xpub) is compared against each seed's
    candidate path set (`account_search`);
  - the stated fingerprint is only a hint for ordering the comparisons. A
    mismatch is reported, not refused: "key @0 states 00000000; this seed is
    73c5da0a". The toolkit itself writes `00000000` for a key whose
    fingerprint it does not know, and other wallets' watch-only exports do too;
  - the printed command uses the seed's real master fingerprint (the
    passphrase-derived one with a passphrase, §5), never the stated one;
  - a seed that equals no key at any stated path or candidate path is refused
    (exit 2); the message names its fingerprint and says where it was tried;
  - a key that no seed equals stays a watch-only slot in the printed command,
    with the fingerprint and path the descriptor states (`00000000` included:
    `bundle` accepts `--slot @1.fingerprint=00000000`, exit 0, checked).
- **Every slot needs a key.** A slot with neither a seed nor a `--cosigner` key
  is refused (exit 2) and the message names the slot: no address can be derived
  with a key missing.
- **A cosigner xpub is used as given** (the account-level key). Its path is not
  searched; its address contribution does not depend on its origin. Its origin
  comes from the mk1 or a `[fp/path]xpub` form; a bare xpub has none, and §6
  says what the printed command carries then. Unplaced multi-chunk mk1 cards
  are grouped by the chunk-set id in their headers and reassembled as
  `mk-codec` does; a group that does not reassemble is refused (exit 2).
  Placed chunks repeat `@N=` per chunk, as in `restore`.
- **Whatever the descriptor already states is not searched.** `[fp]@N` pins that
  seed's slot. `[fp/path]@N` pins slot and path.
- **`--gap-limit N`** is the index window `[0, N)` on each searched chain, not
  BIP-44's "N consecutive unused addresses" rule; help and output say so.
  `N ≤ 2³¹` (indexes are non-hardened).
- **Network.** Targets are classified by their decoded scriptPubKey (witness
  version and program length, or the base58 version byte), never by a prefix
  string. Accepted forms: mainnet `1…`, `3…`, `bc1…`; testnet and signet `m…`/`n…`,
  `2…`, `tb1…`; regtest `m…`/`n…`, `2…`, `bcrt1…`. A target that does not decode
  under `--network` is refused (`NetworkMismatch`, exit 2). The coin level of
  every path comes from `--network` (0' mainnet, 1' otherwise).

## 3. Path set (checked against the code)

`candidate_paths.rs` yields, for accounts `[min, end)`: BIP-44, 49, 84, 86;
BIP-48 script types 1', 2', 3'; BIP-87; then `m/45'` once; then each
`--add-path`. That is `8 × accounts + 1` (unit test
`standard_templates_at_account_0_default_range`).

- **The default window is 20 accounts (161 paths).** It is `path-of-xpub`'s
  set (Brian's requirement). 0 to 9 is `restore`'s empty-origin search.
- **`m/45'` is `bundle`'s bare-`sh` default origin, not a BIP-45 wallet.** A
  BIP-45 wallet adds a cosigner-index level (§1, not covered). The searched-set
  report names this entry "`m/45'` (the `bundle` bare-sh default, not BIP-45
  cosigner branches)" and never claims BIP-45 coverage.
- **The Refugium seat paths are not in that set.** `m/270028'/coin'/n'`,
  n = 0..255, exist only in `restore.rs:3961` (`empty_origin_search_paths`). In
  mnemonic-refugium the seat is the slot (`@i`), and the round is per seat
  (`KitCard::seat_derivation_path` reads `self.seats[i].round`,
  `kits.rs:1502-1515`; `new_round` sets each seat's round on its own). Seat i at
  round r is n = 32r + i (`succ.rs:325` `seat_path`), so with n ≤ 255 slot @i
  has exactly **8 candidate paths**, `{32r + i : r = 0..7}`, and the path fixes
  the slot. `--refugium-seats` adds these per slot (multisig) or all 256 (one
  key). Open question 3.
- **Every script type is tried at every path** (Brian's answer 3). BIP-48/87
  paths are searched for one-key wallets too: the "BIP-48/87 only for
  multisig" ruling limits what `bundle` *defaults* to, not what a user may have
  used. A match on a non-standard pair says so (§6).
- **Dedupe.** An `--add-path` that would repeat a standard path is dropped
  analytically before the search: its template is compared with the eight
  family templates and the account window, never by materialising the lazy
  window into a set (it can reach 8 × 2³¹ entries,
  `candidate_paths_full_hardened_window_is_lazy`). This is new behaviour.
- **Common paths outside the set**, named in the no-match hint as `--add-path`
  suggestions: `m/0'` (BIP-32 default account; breadwallet, MultiBit HD; `bundle`
  mints `[fp/0h]@0`, checked), and Samourai's high accounts
  `m/84'/0'/2147483644'` to `2147483646'`. The hint says of the Samourai paths
  that a match there is reported but cannot be put on a card (§1 "Found but not
  mintable"). `--max-account` and `--number-of-accounts` may reach 2²⁹ too; the
  estimate line says when the window includes accounts that no card can record.

## 4. Search space and cost

**Measured** 2026-11-26, one thread (`[search] threads = 1` in `mt.conf`),
4-core cloud container, dev profile (`opt-level = 2`), 0.107.1 binary and the
toolkit's miniscript pin (`ff4732e5`); r2 re-ran the cost rows and agreed:

| step | cost | how measured |
| --- | --- | --- |
| seed to master (PBKDF2) | about 5 ms per seed | process start to exit |
| account xpub (1 to 4 hardened steps; `m/45'` is 1) | 89 µs | `path-of-xpub`, 160,001 paths in 14.2 s (r2: 13.9 s) |
| one child public key | 25 µs | `address-of-xpub`, 100,000 P2WPKH in 5.1 s (two steps each) |
| P2TR output key, extra | 46 µs | same run with `p2tr`: 9.7 s |
| multisig script, wsh/sh, structure-once | 0.8 to 1.4 µs; plan with 7.4 µs | scratch bench: template parsed once, per candidate `translate_pk` + `script_pubkey`, 200,000 candidates of 2-of-3 `multi`, `sortedmulti`, `sh(wsh(multi))`; 7.4 µs is `SPEC_bundle_md1_template_multisig` §6.3 with engine overhead. r2: 4-of-7 1.2 µs, 8-of-15 `sortedmulti` 6.8 µs |
| multisig script, `tr(NUMS, multi_a)` / `sortedmulti_a` | 46 µs; plan with 53 µs | same bench: TapLeaf hash and TapTweak multiply per candidate (46.2 / 46.4 µs; r2: 4-of-7 44.9 µs); plus the 7.4 µs overhead |
| `restore`'s current address evaluator | 165 µs wsh, 240 µs tr | `restore --md1 <template> --own-account-max 256 --cosigner <mk1> --search-address <non-member> --search-addr-max 200`: 102,400 candidates in 16.9 s and 24.6 s |

`restore`'s evaluator builds and parses a descriptor string per candidate, so
it costs about 22 times the wsh planning figure and 4.5 times the taproot one. This verb must
not reuse it (§4.1). Brian's 24-core machine divides the multisig rows by about
its core count.

**Pruning (normative).**
1. Decode the targets to scriptPubKeys once and look candidates up in a hash
   set. No address string is built in the loop.
2. Per seed and path, derive the account xpub once and each chain key once,
   and cache the `alternatives × gap` child keys. The cache holds k × C ×
   alternatives × gap compressed points, 33 bytes each (0.64 MB for three
   seeds at the defaults: 19,320 points); the estimate line states it.
3. The decoded targets fix which script types can match: a 20-byte v0 program
   only P2WPKH, a 32-byte v0 program only P2WSH, v1 only P2TR, base58 P2PKH
   only `pkh`, base58 P2SH only `sh(…)`. The taproot tweak is computed only
   when a target is P2TR. "All script types" therefore costs nothing extra.
4. **Sorted collapse** (one slot order instead of f!). It applies only when
   all of these hold:
   - the template is `wsh(F)`, `sh(F)`, `sh(wsh(F))` or `tr(NUMS,F)`, where F
     is a single `sortedmulti` or `sortedmulti_a` and is the whole script (for
     `tr`, the only leaf);
   - every slot appears in F exactly once and nowhere else (not as an
     internal key, not in a second leaf or fragment);
   - every slot has the same use-site alternatives.

   Why that is sound: F's script at an index is a function of the *set* of
   child keys at that index, and a slot contributes to it nothing but its
   use-site path. With equal use-site paths, which key sits in which slot
   cannot change any address. Path pins do not break this: a `[fp/path]@N`
   ties a seed to a path, which the path tuple carries, and the one order
   searched is the one that puts pinned seeds in their pinned slots and the
   free seeds in the free slots in seed order.

   Every other template is searched over all f! orders. That includes
   `tr(NUMS,{sortedmulti_a(1,@0,@1),sortedmulti_a(1,@2,@3)})` (which seeds
   share a leaf changes the leaf scripts: three distinct wallets) and
   `tr(@0,sortedmulti_a(2,@0,@1,@2))` (the internal key is a slot: three
   wallets), both accepted by `bundle`, and per-slot multipaths
   (`@0/<0;1>/*` beside `@1/<2;3>/*`, row U1). Orders that the full search
   finds to be the same wallet are merged after the scan (§5). A finer
   reduction (f! divided by the factorial of each group of interchangeable
   slots) is left for later; the full search for these shapes is small (4
   keys: 24 orders, 8.2 s for taproot on one thread).
5. Candidates that are the same wallet by a known symmetry are merged after
   the scan (§5).

**Single-sig.** Nominal size is paths × 4 script types × 2 chains × gap =
161 × 4 × 2 × 20 = 25,760 addresses. The work is 161 × 40 = 6,440 child keys.

| run | child keys | time |
| --- | --- | --- |
| default (161 paths, gap 20) | 6,440 | 0.2 s; 0.5 s with a P2TR target |
| plus all 256 Refugium seats (417 paths) | 16,680 | 0.5 s; 1.3 s |
| `--gap-limit 1000`, 417 paths | 834,000 | 21 s; 59 s |
| `--number-of-accounts 1000` (8,001 paths) | 320,040 | 9 s; 24 s |

One thread stays under a minute up to about 2 million child keys, or 0.8
million with a P2TR target.

**Multisig.** Candidates = slot orders × path tuples × alternatives × gap, with
C = 161 paths, k seeds, f keys without a pinned slot:

- slot orders: 1 under the sorted collapse (pruning 4), otherwise f!.
- path tuples: C when all seeds share one path (default), C^k with
  `--independent-paths`, f! × 8^k assignments-and-tuples with
  `--refugium-seats` (the path fixes the slot).
- key cache: k × 6,440 child keys, about 0.2 s per seed.

One thread, planning figures 7.4 µs (wsh, sh) and 53 µs (tr) per candidate,
both chains, gap 20:

| wallet | paths | candidates | wsh / sh | tr |
| --- | --- | --- | --- | --- |
| one sorted fragment (pruning 4), any n | shared | 6,440 | under 0.1 s | 0.3 s |
| unsorted, 3 keys | shared | 38,640 | 0.3 s | 2.0 s |
| unsorted, 4 keys; or two sorted leaves, 4 keys | shared | 154,560 | 1.1 s | 8.2 s |
| unsorted, 5 keys | shared | 772,800 | 5.7 s | 41 s |
| unsorted, 6 keys | shared | 4.6 million | 34 s | 4.1 min |
| unsorted, 7 keys | shared | 32 million | 4.0 min | 29 min |
| sorted, 2 seeds | independent | 1.0 million | 7.7 s | 55 s |
| sorted, 3 seeds | independent | 167 million | 21 min | 2.5 h |
| sorted, 3 seeds, 5 accounts (C = 41) | independent | 2.8 million | 20 s | 2.4 min |
| unsorted, 3 seeds | independent | 1.0 billion | 2.1 h | 15 h |
| Refugium, 3 seats, unsorted | per seat | 122,880 | 0.9 s | 6.5 s |

The twin check (§5) adds O(targets × n log n) per collected match: nothing
measurable.

**Defaults that keep a run under a minute on one thread:** shared path, 20
accounts, gap 20, both chains. That covers every single-sig run, every
single-sorted-fragment shared-path multisig, unsorted shared-path multisig up
to 6 keys for wsh/sh and up to 5 keys for taproot, and independent paths with
two sorted seeds. **To widen:** `--number-of-accounts` or `--max-account`,
`--gap-limit`, `--add-path`, `--refugium-seats`, then `--independent-paths`
(narrow the accounts first). **To narrow:** `--external-only`,
`--cosigner @N=`, `[fp]@N`, `[fp/path]@N`. (A pin narrows the slot orders and
path tuples searched; it does not narrow the twin check, §5.)

The run states its candidate count, cache size and estimated time before it
starts, shows progress when the estimate passes 30 s, and has no time
ceiling: Ctrl-C stops it. This is `restore`'s policy since v0.99
(`cap_decision`), with the same calibration (`calibrate_per_candidate`) and
threads. Two counts are ceilings, and a run above either is refused before it
starts (exit 2, naming the flags that narrow it): `restore`'s hard
`REALIZED_S_MAX` of 10¹⁵ candidates, and the `u64` range the engine's total
is computed in (it clamps silently today; `--independent-paths` with nine
seeds exceeds it). The count is computed with checked arithmetic.

### 4.1 Engine changes (normative)

The engine in `permutation_search.rs` does not do this search as it stands:
its default `stop_at = 2` short-circuits at the second match
(lines 1151-1158, 1270-1276) and `Ambiguous` carries no assignments; it has no
path dimension (`Enumeration` is `FullPermutation`, `OwnAnchored`, `OptIn` over
a key pool, and `SearchMode::Address` packs only chain and index, `flatten`
line 236); and its only symmetry removal is the `sorted` flag. The plan adds:

1. **Mode.** `collect_all = true` (line 1267) and `early_exit = false`. The
   whole space is scanned and every match is kept; the twin check and the
   merge (§5) run on the collected list after the scan. Two engine facts the
   plan handles: with `collect_all`, one match returns `Unique`, not
   `Enumerated` (test `collect_all_with_one_match_is_unique_not_a_list`); and
   `Enumerated { assignments }` drops the address index and repeats an
   assignment once per matched (alternative, index). The new mode returns
   each match with its (path tuple, slot order, alternative, index). Matches
   are counted in full; the list kept is capped, and hitting the cap is
   itself exit 4. The capped list is new engine code.
2. **A new `Enumeration` variant over (path tuple, slot order).** Rank
   r ∈ [0, T × S): t = r div S, σ = `unrank_permutation(r mod S, f)` (S = 1
   under the sorted collapse, f! otherwise). T is C (shared: one digit, every
   seed at path t), Π Cᵢ (independent: t read mixed-radix over the seeds in
   `--from` order, digit i indexing seed i's path list), or the per-slot seat
   sets (`--refugium-seats`). Each seed contributes exactly one key per
   candidate, so no self-multisig assignment arises (v1 refuses it).
   `SearchMode::Address` then runs over alternatives × gap, with "chain"
   meaning alternative index.
3. **A structure-once evaluator.** The template is parsed once per script
   form; per candidate it translates the cached child keys into the template,
   computes the scriptPubKey and looks it up in the target set. No descriptor
   string is built or parsed per candidate. Which target matched is recomputed
   single-threaded for the collected matches only.
4. The phrase "the engine removes symmetric orders it can prove" in r1 was
   wrong and is gone: pruning 4 is the only pre-scan symmetry removal.

The plan specifies the unrank order exactly and tests the engine's result
against a single-threaded reference over the same space.

## 5. Match semantics

A **wallet** is one script form, one slot order and one path per key. All
targets are tested in one pass.

- **The whole space is scanned. A hit does not stop it** (§4.1). Exit 0 needs
  a certificate over the whole searched space.
- **Sorted/unsorted twins.** At any one index, `multi` with its arguments in
  the order that sorts that index's child keys gives the same script as
  `sortedmulti` over the same keys (likewise `multi_a` and `sortedmulti_a`).
  Vector: `wsh(sortedmulti(2,A,Z))` and `wsh(multi(2,Z,A))`, both at
  m/48'/0'/0'/2', agree at receive 0, 2, 4, 6, 7, 10, 12, 13, 14, 18, 19 and
  differ at the other indices below 20; a wallet minted from the wrong one
  shows about half the real addresses. So every collected match gets a twin
  check, per fragment, over the targets that match it:
  - **Unsorted fragment** (`multi`, `multi_a`): the twin is the sorted
    fragment over the same arguments. Evaluate it at every target (one
    script per target).
  - **Sorted fragment** (`sortedmulti`, `sortedmulti_a`): for each matched
    target, at its (alternative, index), compute the order of the fragment's
    arguments that sorts their child keys there. A `multi` twin fits every
    matched target if and only if that order is the same for all of them;
    the twin is then `multi` (or `multi_a`) with the arguments in that order.
    If the orders differ, no `multi` twin exists. Cost O(targets × n log n);
    no order is enumerated, so it works for n up to 16.
  - The twin **permutes the fragment's arguments** (each `@N` keeps its key,
    its path and its use-site path); it does not change the key-to-slot map.
    Pins (`--cosigner @N=`, `[fp]@N`, `[fp/path]@N`) therefore do not reduce
    it: M1 with Z placed by `--cosigner @1=` still exits 4.
  - A template with several fragments gets the check per fragment; each
    fragment with a fitting twin is reported, and the printed twin replaces
    all of them (other mixtures also fit, and the note says so).
  - The check also runs in keyed mode (§2): the descriptor's form is the
    user's record, but one address still cannot confirm it.
- **Ambiguous result.** If a twin fits every target the found wallet fits,
  the result is ambiguous (exit 4). Both wallets are printed, each with its
  `bundle` command, the one built from the user's template marked as such.
  The tool then prints where they differ, so the user can decide from their
  own records: the first three indices in the window, on alternative 0 and
  then the others, at which the two wallets' addresses differ, with both
  addresses at each (looking past the window up to index 999 if the window
  holds fewer than three). For M1: receive 1, 3 and 5 (§6 sample). The same
  is printed whenever exit 4 lists two or more wallets that each fit every
  target. A twin that fits more targets than any searched candidate is also
  reported (exit 4: "the sorted twin of your template matches all targets",
  row M6). Consequently a one-target multisig search with a sorted template,
  or whose found order happens to be sorted at that index, ends at exit 4,
  with the distinguishing addresses printed. Open question 6.
- **Same wallet, two ways.** Not in single-sig once paths are deduplicated:
  different paths give different keys, and different script types give
  different addresses. In multisig, two candidates are merged into one wallet
  only when a known symmetry relates them, never because their addresses
  agree inside the window (with `--gap-limit 1 --external-only` the window is
  one script). The symmetries are: reordering the arguments of one sorted
  fragment among slots with equal use-site paths (in templates outside
  pruning 4), and swapping the two children of any TapBranch (BIP-341 sorts
  them at every TapBranch, lines 75-76). Merged candidates are the same funds;
  the printed command uses the template as the user gave it (seeds in the
  order given, the tree as written) and says the other forms give the same
  addresses but a different wallet-policy id.
- **Exit 0** means: exactly one wallet among the scripts searched, every target
  is in it, no twin fits, and the wallet can be minted by the printed command
  as printed.
- **Exit 4, a wallet was found but verify** (`XpubSearchUnverified`; JSON
  `result: "unverified"` with one or more `reasons`, §6):
  - `twin`: a sorted/unsorted twin also fits (above);
  - `split`: targets match two or more different wallets (addresses from two
    accounts, say);
  - `unmatched`: some targets match nowhere, including a target whose type no
    searched script can produce (a 32-byte v0 target with no `--descriptor`,
    a P2TR target with a wsh template). Only when **no** target can be
    produced is the run refused (exit 2) before it starts;
  - `path_unrecordable`: the match has a path step of 2²⁹ or more (§1); its
    full path is printed and no `bundle` command;
  - `origin_unknown`: a bare cosigner xpub's origin is not known (§6);
  - `cap`: the collected match list hit its cap (§4.1).
  Every finding is printed.
- **Several seeds, no descriptor.** Each seed is its own single-sig search. A
  seed that holds no target is listed ("seed 1: no target found") and does not
  by itself cause exit 4. Exit 0 when every target is matched, each by
  exactly one wallet; the other reasons above apply as written.
- **No match:** exit 4 today (`XpubSearchNoMatch`, like the sibling modes). The
  report prints exactly what was searched: path families (with the `m/45'`
  wording of §3), account window, each `--add-path`, whether Refugium seats
  were included, script types, alternatives, index window `[0, N)`, slot
  orders, shared or independent paths, the candidate count, and whether a
  passphrase was given. It suggests the widening flags and the §3 `--add-path`
  paths. For a `3…` (P2SH) or `bc1p…` (P2TR) target it adds that the address
  may be a script the search was not given: a taproot script wallet, or a
  taproot multisig whose internal key is not the toolkit's NUMS (for example
  BIP-341's `H + rG`, line 157), is found only with its exact descriptor.
- **Passphrase.** The given passphrase is applied to every seed and none is
  guessed. It changes the master fingerprint (abandon + `TREZOR` is
  `b4e3f5ed`, not `73c5da0a`); the output, the JSON and the printed origin use
  the passphrase-derived fingerprint, or `bundle` refuses the command ("derives
  master fingerprint b4e3f5ed"). A no-match says a passphrase-protected wallet
  needs `--passphrase`, and points at `passphrase-of-xpub` for testing
  candidates against a known xpub.

**Refusals that need user input, exit 2** (new variant `XpubSearchRefusal`):
a slot without a key; more keys than slots; the same seed twice; `--cosigner`
without `--descriptor`; a seed that equals no key of a keyed descriptor (§2);
targets none of which a searched script can produce; private-key material in
`--descriptor`, `--descriptor-file` or `--cosigner`; `/*h`, a fixed single
step, a bare `@N`; a multi-chunk mk1 group that does not reassemble; a second
stdin reader; a candidate count above 10¹⁵ or the `u64` range. A
found-but-verify result uses a new `XpubSearchUnverified` (exit 4). Both go in
alphabetical order after `XpubSearchPassphraseCandidatesExhausted`.

| outcome | today | draft exit-code scheme |
| --- | --- | --- |
| one wallet, all targets, no twin, mintable as printed | 0 | 0 |
| found, but `twin`, `split`, `unmatched`, `path_unrecordable`, `origin_unknown` or `cap` | 4 | 4 UNVERIFIED |
| no match | 4 | 1 NO |
| refusals above | 2 | 2 INVALID |
| malformed phrase (`Bip39`), malformed address (`BadInput`) | 1 | 2 INVALID |
| malformed descriptor (`DescriptorParse`) | 2 | 2 INVALID |
| secret on argv; network mismatch | 2 | 8 REFUSED |
| unset `@env:VAR`, stdout failure | 1 | 32 IO |
| clap usage | 64 | 64 |

The right-hand column is the constellation's draft exit-code scheme. It is
not yet a document in `design/` or the shared folder; the plan cites it once it
lands, and until then only the "today" column is normative. The manual's exit
table says that exit 4 has two meanings (found-but-verify, no match) until the
scheme lands; text and JSON `result` tell them apart.

## 6. Output

**Streams.** stdout carries the result only (text blocks, or the JSON
document under `--json`). stderr carries the estimate line, progress, and in
text mode the redaction note, as `bundle`'s does; under `--json` the estimate
and progress still go to stderr (stdout stays one JSON document) and the
redaction note is the `redaction_note` field instead.

Text, one block per wallet. Single-sig:

```
match: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
  path:        m/84'/0'/0'/0/0
  account:     0   (bip84, m/84'/0'/0')
  chain:       alternative 0 (step 0, external)      index: 0
  script type: p2wpkh
  slots:       @0 = seed 0 (73c5da0a)
searched: 6440 child keys on 161 paths, index window [0, 20); 1 wallet matches
to mint this wallet, run:
  mnemonic bundle --network mainnet --descriptor 'wpkh([73c5da0a/84h/0h/0h]@0/<0;1>/*)' --slot '@0.phrase=-'
```

Multisig (row M5: two targets, exit 0), one `slot` line per key with its own
origin and account, one `target` line per matched address with each key's
full path:

```
match: wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))   script type: p2wsh
  slot @0:     seed 0 (73c5da0a)  m/48'/0'/0'/2'  account 0 (bip48, script type 2')
  slot @1:     seed 1 (3f635a63)  m/48'/0'/0'/2'  account 0 (bip48, script type 2')
  target:      bc1qkpcfs6lhqh3rx63we7435nf3hqpghkdtmmmvln0yrssv0ulhmngs92dens
               alternative 0 (step 0, external)  index 0
               @0 m/48'/0'/0'/2'/0/0   @1 m/48'/0'/0'/2'/0/0
  target:      bc1qp3z0f9m2yzgax82hw88jaqznsca2gtktnyuvmh4uwzlsg4736y8s2qes5n
               alternative 0 (step 0, external)  index 1
               @0 m/48'/0'/0'/2'/0/1   @1 m/48'/0'/0'/2'/0/1
  twin:        none (multi orders differ: Z,A at index 0, A,Z at index 1)
searched: 12880 child keys on 161 paths, 6440 candidates, index window [0, 20); 1 wallet matches
to mint this wallet, run:
  mnemonic bundle --network mainnet --descriptor 'wsh(sortedmulti(2,[73c5da0a/48h/0h/0h/2h]@0/<0;1>/*,[3f635a63/48h/0h/0h/2h]@1/<0;1>/*))' --slot '@0.phrase=-' --slot '@1.phrase=@env:MNEMONIC_SEED_1'
```

Ambiguous (row M1: one target, exit 4). After the two wallet blocks, each with
its command:

```
unverified: two wallets give bc1qkpcfs6…92dens at receive index 0
  wallet 1 (your template): wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))
  wallet 2 (its twin):      wsh(multi(2,@1/<0;1>/*,@0/<0;1>/*))
  they differ at:
    receive 1: wallet 1 bc1qp3z0f9m2yzgax82hw88jaqznsca2gtktnyuvmh4uwzlsg4736y8s2qes5n
               wallet 2 bc1qmkt0wm4ypxh0rcpyrm05rsy4r36t042frpuu28tyxvanjnlg2lxq5ng527
    receive 3: wallet 1 bc1qhug5hhjd9ar4g4guurqvln5vk7pthv79v5hnd77kal2ft9a2ly6qz24zfh
               wallet 2 bc1qt99jw7rz5ey5ke3gzngyp4kqa3cehqq5c2z7s46eq6f4q49df8ks8zj3um
    receive 5: wallet 1 bc1qq4wwd93p02d84j9nywleeza6cuj3whehmfgj0wse5h2pm9du9j7qsudgz5
               wallet 2 bc1qp02n8d2ctm99vm7mj2hxdqqjt5y7nyzmfycm8dhn0pnnpz6cjjds8axnts
  whichever of these your wallet shows decides it; or rerun with that address as a second --target-address.
```

(Addresses minted by 0.107.1 `bundle` and read back with `restore --md1`.)

The `path` line, and each per-key path in multisig, is the real full path of
the matched address, including the template's own use-site step (`…/2/0`
under `<2;3>`; no chain step under a bare `/*`). A non-standard pair adds:
`note: p2wpkh at a BIP-44 path is not what BIP-44 describes`. A key at a path
outside the eight families reads `account: –  (--add-path m/…)`, `(bundle
bare-sh default m/45')` or `(Refugium seat 1, round 1)`.

JSON (`XpubSearchEnvelope`, new arm `address-of-seed`). The GUI consumes this
shape and no gate checks it (§8), so it is normative:

```
{"schema_version":"1","mode":"address-of-seed",
 "result":"match|unverified|no_match","reasons":["twin","split","unmatched","path_unrecordable","origin_unknown","cap"],
 "wallets":[{"role":"found|twin","twin_of":null,"from_user_template":true,
   "descriptor":"wpkh([73c5da0a/84h/0h/0h]@0/<0;1>/*)","script_type":"p2wpkh",
   "merged_forms":0,
   "slots":[{"slot":0,"source":"seed|cosigner","seed_index":0,"master_fingerprint":"73c5da0a",
             "stated_fingerprint":null,"origin_path":"m/84'/0'/0'","origin_known":true,
             "template":"bip84|…|bip87|bundle-sh-default|add-path|refugium-seat|keyed",
             "account":0,"refugium_seat":null,"refugium_round":null}],
   "targets":[{"address":"bc1q…","alternative":0,"step":0,"index":0,
               "paths":[{"slot":0,"path":"m/84'/0'/0'/0/0"}]}],
   "distinguishing":[{"alternative":0,"index":1,"addresses":{"0":"bc1q…","1":"bc1q…"}}],
   "bundle_command":"mnemonic bundle …","bundle_command_redacted":false,
   "bundle_command_unavailable":null,"redaction_note":null}],
 "unmatched_targets":[{"address":"…","reason":"not_found|type_not_searched"}],
 "seeds_without_match":[],
 "searched":{"mode":"search|keyed","paths":161,"child_keys":6440,"candidate_scripts":null,
   "min_account":0,"end_account":20,"add_paths":[],"refugium_seats":false,"gap_limit":20,
   "alternatives":[0,1],"script_types":["p2wpkh"],"slot_orders":1,"independent_paths":false,
   "passphrase_supplied":false,"matches_total":1,"matches_kept":1}}
```

- `result` is `match` (exit 0), `unverified` (exit 4, `reasons` non-empty, in
  the order listed) or `no_match` (exit 4 today). `reasons` is empty for the
  other two.
- A twin is its own `wallets[]` entry with `role: "twin"` and `twin_of` set to
  the index of the found wallet; `from_user_template` marks the one built
  from the user's template. `distinguishing` is filled on every wallet of an
  ambiguous pair, keyed by `wallets[]` index. A twin that fits more targets
  than any found wallet is listed first, with reason `twin`.
- `merged_forms` counts the other candidates merged into this wallet by a
  symmetry (§5).
- `template`, `account`: `account` is null for `bundle-sh-default`,
  `add-path`, `refugium-seat` and `keyed`; `refugium_seat` and
  `refugium_round` are set for the seat paths.
- `bundle_command` is null when no command can be printed;
  `bundle_command_unavailable` then says why (`path_unrecordable` or
  `origin_unknown`).
- In keyed mode `searched.mode` is `"keyed"`, `paths` counts the seed
  derivations made to place seeds, and `slot_orders` and
  `candidate_scripts` are null.
- `slot` and `seed_index` are 0-based. `child_keys` counts derivations and
  `candidate_scripts` counts multisig candidates (null in single-sig).

**Building the `bundle` command** (each form below was run on 0.107.1 and
exits as stated):

- **Always descriptor form with the origin inline**, `[fp/path]@N`, so the path
  is stated and no default applies (Brian's requirement). A seed slot cannot
  carry `--slot @N.path=`: `bundle` refuses a slot that has "both
  secret-bearing input and watch-only input". A cosigner slot's `@N.path=`
  beside a seed slot is accepted.
- Hardened steps are written `h`, which `bundle` accepts, so the only shell
  quoting is one pair of single quotes. For a standard pair the md1 is
  byte-identical to `--template bipNN --account N` (checked for bip44/3,
  bip49/0, bip84/0, bip84/3, bip86/2).
- **The argv is built from references, never from secrets.** A seed becomes
  `--slot @N.<phrase|ms1|entropy|seedqr>=…`, with N the slot found. A value
  that came from `-`, `--phrase-stdin`, `--ms1-stdin` or `@env:VAR` is repeated
  as that reference. A value that was inline becomes `-` only if no other
  value in the printed command reads stdin (seeds in seed order first, then
  the passphrase); otherwise `@env:MNEMONIC_SEED_<i>`, i the 0-based seed
  index (not the slot), or `@env:MNEMONIC_PASSPHRASE` for the passphrase. The
  printed command therefore never has two stdin readers. The redaction note
  says to supply those values that way, never on argv (the argv guard refuses
  it). `--language` and `--network` are copied. The finished argv is passed
  through `redact_and_quote` (`bundle.rs:3463`) as a fail-closed second check;
  no real secret is ever placed in the `Vec<String>`. `rerun_command_with`
  (`bundle.rs:3410`) re-prints the current process's argv, the wrong verb
  here, and `REDACTED_NOTE` (`bundle.rs:3244`, "put yours back … or pass one
  as =- on stdin") is replaced for this verb by a note that names `@env:VAR`.
- **A cosigner with a known origin** (mk1, `[fp/path]xpub`, or a keyed
  descriptor's key) becomes `--slot @N.xpub=… --slot @N.fingerprint=…
  --slot @N.path=m/…`, with the fingerprint as stated (exit 0).
- **A bare cosigner xpub** (no origin anywhere). Its addresses do not depend
  on the origin, so the wallet is found, but the origin is unknown and is
  only recorded on the card. The search reads the xpub's own header (depth
  and child number) and compares it with the template's canonical origin
  (`m/48'/coin'/0'/2'` for `wsh`, `m/87'/coin'/0'` for `tr` and bare `sh`, as
  `bundle` chooses):
  - **Header consistent with the canonical origin.** The command states that
    path explicitly, `--slot @N.xpub=… --slot @N.path=m/48'/0'/0'/2'`, with no
    fingerprint (`bundle` accepts this, exit 0, and records `00000000`). The
    text marks the path "assumed: the canonical origin for this shape; your
    xpub carries no origin" and the result exits 4 (`origin_unknown`). Without
    the explicit path, `bundle` would pick the same origin itself, warn and
    exit 4; stating it keeps Brian's "paths stated explicitly" rule.
  - **Header not consistent** (for example zoo's `m/48'/0'/0'/2'` xpub, depth
    4, child 2', in `tr(NUMS,sortedmulti_a(…))`, or any xpub at account ≠ 0).
    `bundle` refuses any command without the real path (exit 2: "the default
    origin m/87'/0'/0' does not match the xpub's own header (depth 4, child
    number 2')"; adding only `@N.fingerprint=` does not help). The match is
    printed with exit 4 (`origin_unknown`); `bundle_command` is null and the
    text prints the command with the literal placeholder
    `--slot '@1.path=<origin of this xpub>'`, saying that only the user's
    records can fill it and that `bundle` refuses the command until they do.
- **A match at a path step of 2²⁹ or more** prints the match and its full
  path, no command, and "md1 cannot record a path step above 536870911; no
  card can be made for this wallet" (exit 4, `path_unrecordable`).
- With a passphrase, the origin's fingerprint is the passphrase-derived one
  (§5).

## 7. Security

- The pre-parser argv guard applies unchanged: `--phrase`, `--ms1` and
  `--from <secret>=` on argv are refused before parsing (exit 2) unless
  `--allow-argv-secret`. A test pins the guard's table against the new verb.
- Intake refuses private-key material in `--descriptor`, `--descriptor-file`
  and `--cosigner` and never echoes the value (§2).
- No secret reaches stdout, stderr or JSON: only addresses, paths, master
  fingerprints, public keys and the reference-built command. A test greps all
  three streams for every seed word and the passphrase.
- Memory as in `path-of-xpub`: `Zeroizing` phrase, passphrase and seed, pages
  pinned with `mlock::pin_pages_for`, master xprv dropped after the account
  xpubs are derived (in keyed mode, after the stated paths are derived).
  Worker threads receive public keys only. The `bundle` argv never holds a
  secret (§6).
- An address is weaker evidence than it looks if it was copied from an
  untrusted place. The output says which address matched, and the user compares
  the minted wallet's first addresses with their own records.

## 8. Lockstep (CLAUDE.md rules)

- Manual: `docs/manual/src/40-cli-reference/41-mnemonic.md`, a new
  `xpub-search address-of-seed` section, the exit table (with the two meanings
  of exit 4) and the argv-channel table; `docs/manual/tests/cli-subcommands.list`.
- GUI manual: a new `docs/manual-gui/src/40-mnemonic/` chapter for the verb
  (next free prefix beside `4g-xpub-search-address-of-xpub.md`), plus
  `docs/manual-gui/src/40-mnemonic/41-overview.md`,
  `docs/manual-gui/src/75-gui-forms/751-mnemonic.md` and
  `docs/manual-gui/src/90-appendices/99-index-table.md`; a new
  `xpub-search-address-of-seed` entry in
  `docs/manual-gui/tests/expected_gui_schema_inventory.json`; a new
  `docs/manual-gui/transcripts/gui/mnemonic-xpub-search-address-of-seed.gui`
  beside the four existing `mnemonic-xpub-search-*.gui`; and the
  `docs/manual-gui/pinned-upstream.toml` bump.
- Toolkit tests that list verbs: the subcommand list in
  `tests/cli_gui_schema.rs` (line 115); a per-verb row in
  `tests/cli_network_fail_open.rs`; the new verb in
  `tests/cli_xpub_search_env_var_sentinel.rs`.
- `address-of-xpub`'s help calls `--gap-limit` a "Per-chain BIP-44 gap-limit
  window" (`address_of_xpub.rs:45`). The same PR rewords it to the index
  window of §2 so the family agrees; its manual lines move with it.
- mnemonic-gui, paired PR:
  - add the subcommand to `src/schema/mnemonic.rs`, then the
    `pinned-upstream.toml` bump and `cargo test --test schema_mirror`;
  - add the verb to `tests/xpub_search_schema_mirror.rs` (verb list, line 47)
    and `tests/f5_nested_argv.rs` (line 44);
  - mark every secret-bearing flag `secret: true`: `--phrase`, `--ms1`,
    `--passphrase` and the repeatable `--from <node>=` composite for its seed
    nodes. This verb family leaked before (`src/secrets.rs:473-477`, FOLLOWUP
    `xpub-search-inline-phrase-not-secret-classified`), and
    `src/persistence.rs:84-95` drops values only by secret name;
  - add the `("xpub-search-address-of-seed", ["xpub-search",
    "address-of-seed"])` entry to `src/form/invocation.rs` (lines 888-896);
  - add a persistence test proving that no seed or passphrase reaches
    `state.json`;
  - consume the JSON shape of §6 (the `schema_mirror` gate checks flag names
    only).
- `gui-schema` and `gen-man` pick the verb up from clap; their snapshots move.
- `ToolkitError` variants (`XpubSearchRefusal`, `XpubSearchUnverified`) and JSON
  enum arms go in alphabetical order.
- FOLLOWUPs filed with the plan: `address-of-seed-bip45-cosigner-index`;
  `address-of-seed-md1-path-step-range` (companion in descriptor-mnemonic).
- Funds-touching work: R0 on this spec and on the plan before code, one
  implementer, whole-diff review after.

## 9. Tests

All vectors use `abandon ×11 about` (A, fingerprint `73c5da0a`; with passphrase
`TREZOR`, `b4e3f5ed`) and, for multisig, `zoo ×11 wrong` (Z, `3f635a63`). Each
address up to r2 was computed with an independent from-scratch script
(secp256k1, BIP-32, bech32/base58, BIP-341). S1 to S10 and M1 to M4 were
rechecked by review r1; S1, S2, S3 also appear in the BIP-84 and BIP-86 texts.
Rows added in r2 (M5, M6, R1, R2, U1, N1, T1, P1) were also derived by the
0.107.1 binary (`bundle --descriptor` then `restore --md1`), which agreed on
every one, and r2's review re-derived them the same way (M4 agrees with the
script; it is on the change chain, which `restore` does not list). Rows added
in r3: M1p, K1, B1, L1, L2 and G1 reuse those addresses or need none; D1's
and B2's addresses were minted with `bundle` and read back with `restore
--md1`; H1's two were computed with the from-scratch script (it reproduces S1
at account 0), and the `536870911'` one also by `bundle` then `restore`. L1
and L2 targets are minted by the test itself.
The implementer re-derives all of them with Bitcoin Core `deriveaddresses`
before pinning.

**Address oracle for the "printed command, run" rows.** `restore --md1`
lists the minted wallet's addresses for the multisig rows it can derive. It
cannot for single-sig `tr` (S3, S7: "taproot tree has no script leaf") nor
for the shapes of §2 "Accepted templates" (`sortedmulti_a` in two leaves, a
trunk key that is also a leaf key, `sortedmulti` in a combinator); those rows
use Core `deriveaddresses` on the minted descriptor (in the `regtest` CI job)
or rust-miniscript directly in the test.

Templates are written in full; `@N` alone is refused (row G1).

| id | target | expected |
| --- | --- | --- |
| S1 | `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu` | m/84'/0'/0'/0/0, p2wpkh (BIP-84 text) |
| S2 | `bc1q8c6fshw2dlwun7ekn9qwf37cu2rn755upcp6el` | m/84'/0'/0'/1/0, change (BIP-84 text) |
| S3 | `bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr` | m/86'/0'/0'/0/0, p2tr (BIP-86 text) |
| S4 | `1LqBGSKuX5yYUonjxT5qGfpUsXKYYWeabA` | m/44'/0'/0'/0/0, p2pkh |
| S5 | `37VucYSaXLCAsxYyAPfbSi9eh4iEcbShgf` | m/49'/0'/0'/0/0, p2sh-p2wpkh |
| S6 | `bc1qmxrw6qdh5g3ztfcwm0et5l8mvws4eva24kmp8m` | m/44'/0'/0'/0/0, p2wpkh (non-standard pair) |
| S7 | `bc1p8knh0enfv47gmpuf66528zd4jtkgjq4sv5w5l2gqwgk8exu2ynns9g8c9m` | m/84'/0'/0'/0/0, p2tr (non-standard pair) |
| S8 | `bc1qvhccx0r4sxxsy03xltrpsgw2dnfry6e5vheaa0` | m/84'/0'/3'/1/17 |
| S9 | `bc1qtncy45g8qy8laah6jvxn6ppvj0s9rluxj5uxf6` | m/48'/0'/0'/2'/0/0, p2wpkh (one key at a BIP-48 path) |
| S10 | `bc1qj4rt65727dc9mn9g0n4ynpvnag060jt95gqrnx` | m/270028'/0'/33'/0/5, p2wpkh (Refugium seat 1, round 1; needs `--refugium-seats` or `--add-path`) |
| T1 | `tb1q6rz28mcfaxtmd6v789l9rrlrusdprr9pqcpvkl` | `--network testnet`: m/84'/1'/0'/0/0, p2wpkh; refused under mainnet |
| P1 | `bc1qv5rmq0kt9yz3pm36wvzct7p3x6mtgehjul0feu` | A + `--passphrase TREZOR`: m/84'/0'/0'/0/0, origin `[b4e3f5ed/84h/0h/0h]`; no match without the passphrase |
| M1 | `bc1qkpcfs6lhqh3rx63we7435nf3hqpghkdtmmmvln0yrssv0ulhmngs92dens` | template `wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))`, both at m/48'/0'/0'/2', 0/0, one target: found, exit 4 (`twin`: `multi(2,@1/<0;1>/*,@0/<0;1>/*)` also fits); distinguishing indices printed (row D1) |
| M1p | M1's target and template, Z given only as `--cosigner @1=<Z's m/48'/0'/0'/2' xpub>` | still exit 4 with the same twin: pins do not reduce the twin check |
| D1 | M1's run | the "differ at" lines are receive 1, 3 and 5 with the addresses of the §6 sample, in text and in JSON `distinguishing` |
| M2 | same address, template `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))` | finds @0 = Z, @1 = A; exit 4, the sortedmulti twin also fits; both wallets printed, the multi one marked as the user's template |
| M3 | `bc1qqr4u57xa6zg0cdt0kwy0zzf29s57h5klqr3gwuyhu6g2qvcf6qzsmuzazu` | `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))`, @0 = A, @1 = Z, 0/0; the sorted twin (M1's address) does not fit: exit 0 |
| M4 | `bc1qlhnjvp36wlaefhk5prxp895zp74kvdac7fphw8ntk7vaas3v3xpqwymvpl` | sortedmulti, A m/48'/0'/0'/2', Z m/87'/0'/2', 1/7: no match shared; match with `--independent-paths --number-of-accounts 3` (keeps the debug suite fast) |
| M5 | M1 plus `bc1qp3z0f9m2yzgax82hw88jaqznsca2gtktnyuvmh4uwzlsg4736y8s2qes5n` (index 1) | sortedmulti template: exit 0; the per-target sorting orders differ (Z,A at 0; A,Z at 1), so no `multi` twin exists (`multi(Z,A)` at 1 is `bc1qmkt0wm4y…5ng527`, `multi(A,Z)` at 0 is M3) |
| M6 | same two targets, template `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))` | no single `multi` wallet fits both; the sorted twin fits both; exit 4 with "the sorted twin of your template matches all targets" |
| R1 | `bc1pzxzsarxcr78t8clalyhtj0gaprlnhkaq7u8yg9keqx97rgpg0yfsq8kfd0` | `tr(NUMS,sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*))`, A and Z at m/87'/0'/0', 0/0; exit 4 (the `multi_a(A,Z)` twin gives the same address) |
| R2 | `bc1psw9rgfn7tku2t6jl8vzg7trwsr98zv23d9jkwnhcmquxujc3mxssffw33l` | `tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))`, @0 = Z, @1 = A, m/87'/0'/0', 0/0; the sorted twin does not fit: exit 0 |
| U1 | `bc1qw6qcv8r9fmd27fucmjsctjcw4jy8fgdmjhd9hlj0lapzeg6wdnsqtq0gq2` | `wsh(sortedmulti(2,@0/<0;1>/*,@1/<2;3>/*))`, @0 = A, @1 = Z at m/48'/0'/0'/2', alternative 0; path lines `…/0/0` and `…/2/0`. Swapping the slots gives `bc1qu97h4hd0…kcapsw`, so the sorted collapse must not apply |
| N1 | `34zdguTXdmTqGbhcWJVWSnKy1sAx1za3gN` | `sh(wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*)))`, A and Z at m/48'/0'/0'/2', 0/0 |
| L1 | four keys (A, Z and two watch-only cosigners, unplaced), a target minted from `tr(NUMS,{sortedmulti_a(1,@0,@2),sortedmulti_a(1,@1,@3)})` (each `@N/<0;1>/*`) | template given as `tr(NUMS,{sortedmulti_a(1,@0,@1),sortedmulti_a(1,@2,@3)})`: no collapse, 24 orders searched, the pairing is found; the engine's count states 24 slot orders |
| L2 | three keys, a target minted from `tr(@1,sortedmulti_a(2,@0,@1,@2))` (in full) | template `tr(@0,sortedmulti_a(2,@0,@1,@2))`: no collapse; the internal key is found |
| K1 | M1's target; keyed descriptor `wsh(sortedmulti(2,[00000000/48h/0h/0h/2h]<A xpub>/<0;1>/*,[3f635a63/48h/0h/0h/2h]<Z xpub>/<0;1>/*))`, seed A only | A is placed at @0 by derivation despite the stated `00000000`; the output notes the mismatch; the printed command carries `[73c5da0a/48h/0h/0h/2h]@0` and Z as `@1.xpub` with `@1.fingerprint=3f635a63` (checked: exit 0, first address M1's); exit 4 for the twin |
| B1 | M1's target; Z given as bare `--cosigner <Z's m/48'/0'/0'/2' xpub>` | found; command carries `--slot @1.path=m/48'/0'/0'/2'` and no fingerprint (checked: exit 0); exit 4 (`origin_unknown`, and `twin`) |
| B2 | `bc1pv8ycp7my2y9ev35rn40m8z962lxdf0jw6nuvpwum6lhp7vf79c7qxq7445` (minted from `tr(NUMS,sortedmulti_a(2,[73c5da0a/87h/0h/0h]@0/<0;1>/*,[3f635a63/48h/0h/0h/2h]@1/<0;1>/*))`, 0/0) | template `tr(NUMS,sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*))`, A as seed, Z as bare `--cosigner <Z's m/48'/0'/0'/2' xpub>`: found (A at m/87'/0'/0'); `bundle_command` null, placeholder command printed, exit 4 (`origin_unknown`, `twin`); `bundle` refuses the same command without the path (checked: exit 2) |
| H1 | `bc1qxc49mhaugahtwnn9hcflmcp0llsxjm3z809mpr` | A, `--add-path "m/84'/0'/536870912'"`: m/84'/0'/536870912'/0/0, p2wpkh; found, full path printed, no command, exit 4 (`path_unrecordable`; `bundle` refuses the path, checked). At `536870911'` the target `bc1qf4z33fsmn5m7f7zt24wf8lr046rz56kxnprfel` is found, the command is printed, and it exits 0 |
| G1 | template `wsh(sortedmulti(2,@0,@1))` | refused, exit 2, with `bundle`'s wording (checked: `bundle` exits 2) |

Further rows:

- **The printed command, run.** For every match that prints a command,
  supply the secrets through `-` and `@env:` and run it: it exits 0, and the
  minted wallet's first addresses equal the found wallet's (oracle above).
  For B2 and H1, `bundle_command` is null and none is run.
- Redaction: an inline phrase becomes `-` or `@env:MNEMONIC_SEED_<i>`; an
  inline passphrase beside a stdin seed becomes `@env:MNEMONIC_PASSPHRASE`;
  given `-` and `@env:` are kept; the printed command never has two stdin
  readers; nothing secret on any stream; the argv vector never holds a
  secret.
- No match: exit 4 and the exact searched set, with the `m/45'` wording. Gap
  edges: index 19 found, index 20 not, found with `--gap-limit 21`;
  `--external-only` skips S2.
- Two targets from two accounts: two wallets, exit 4 (`split`). One target
  unmatched: exit 4 (`unmatched`). A P2TR target beside a P2WSH target with a
  `wsh` template: the P2TR one is unmatched (`type_not_searched`), exit 4; a
  lone 32-byte v0 target without a descriptor: refused, exit 2.
- Two seeds, no descriptor: each searched on its own; A holds every target
  and Z none: exit 0 with "seed 1: no target found".
- Each refusal in §5 exits 2, including `/*h`, an `xprv` in `--descriptor`
  and in `--cosigner` (and its text absent from stderr), `--cosigner` without
  `--descriptor`, a second stdin reader across `--from`, `--passphrase` and
  `--descriptor-from md1=-`, and a count above the ceiling. A cosigner as
  xpub and as mk1 (single- and multi-chunk), placed and unplaced, including
  one unplaced cosigner across 3 slots.
- A keyed descriptor confirms the targets without a search and agrees with the
  template path; a keyed descriptor with a key at a non-standard origin
  (m/48'/0'/25'/2', a Refugium seat) places its seed by derivation; a seed
  that equals no key exits 2.
- A keyless md1 pins its recorded paths; a no-match there carries the
  template-as-text hint.
- Intake by ms1, entropy and seedqr; a non-English phrase; regtest
  (`bcrt1q6rz28mcfaxtmd6v789l9rrlrusdprr9pz3cppk`) beside T1.
- Bare `/*` template: one alternative, no chain step on the `path` line.
  `/**` behaves as `<0;1>/*`.
- Taproot: tap-tree sibling swap gives one wallet with the note; a `bc1p`
  no-match carries the internal-key hint.
- `--refugium-seats`: S10 found with one key; a 2-seat Refugium-shaped
  descriptor found per seat.
- The engine's result equals the single-threaded reference over the same
  space, including the collected match list; a one-match `collect_all` run is
  reported as a match, not dropped.
- JSON snapshot covering `match`, each `unverified` reason and `no_match`;
  `schema_mirror` and the persistence test in the GUI PR; manual lint.

## 10. Open questions for Brian

1. **Multisig paths: shared or independent?** *Recommend:* shared by default
   (every seed at the same path, the usual case, C candidates) and
   `--independent-paths` to opt in (C^k). A no-match prints the independent
   command and its estimated time.
2. **Default bounds.** *Recommend:* 20 accounts, gap 20, both chains, as
   `path-of-xpub`. On one thread every single-sig and single-sorted-fragment
   run then takes under a second; unsorted shared-path runs stay under a
   minute up to 6 keys (wsh/sh) or 5 keys (taproot, which costs about 7 times
   more per candidate).
3. **Refugium seat paths.** The round is per seat and the seat is the slot, so
   slot @i has 8 candidate paths, not 256. *Recommend:* `--refugium-seats`
   opt-in: per slot 8 paths (f! × 8^k tuples: three seats take about 1 s wsh,
   7 s taproot), and for one key all 256 (about 1 s). Should it instead be on
   by default when a `--descriptor` is given? And does a single-key Refugium
   wallet exist at all? If not, the one-key branch is dropped.
4. **Intake for two or more seeds.** *Recommend:* a repeatable `--from
   <node>=<value>` (new code; `restore`'s takes one value), with the one-seed
   flags kept for parity with the sibling modes.
5. **One seed in two slots** (self-multisig at two paths). *Recommend:* refuse
   in v1 and file a follow-up beside `tk-bh1125-r1-m3-self-multisig-two-matches`
   (`FOLLOWUPS.md:6879`).
6. **Twin policy.** One address cannot tell `multi` from `sortedmulti`.
   *Recommend:* report the twin and exit 4 whenever it fits every target,
   in keyed mode too, and print the first three indices where the two
   wallets differ with both addresses, so the user can decide from their own
   wallet without a rerun. A single-target sorted multisig search then always
   ends at exit 4, with that list.
7. **No-match exit.** *Recommend:* 4 today, like the sibling modes, moving to 1
   with the exit-code scheme. That leaves "found, verify" and "not found" on
   the same code until then; text and JSON `result` tell them apart, and the
   manual's exit table says so.
8. **Bare cosigner xpub.** Its origin is unknown and goes only on the card.
   *Recommend:* when the xpub's header fits the template's canonical origin,
   print the command with that path stated explicitly and marked "assumed"
   (it runs, exit 0); otherwise print a placeholder for the path and no
   runnable command. Exit 4 either way.
9. **Accounts at or above 2²⁹** (Samourai's `2147483644'` to `2147483646'`).
   md1 cannot record them. *Recommend:* keep them out of the default set,
   name them in the no-match hint, and on a match print the path with no
   command (exit 4); whether md1 should grow the range is the descriptor-
   mnemonic FOLLOWUP.

Settled by the r1 fold and dropped from the list: whether a hit stops the
search (no; the twin check and merge need the full collected scan, §4.1), and
the printed command form (descriptor with the origin inline, which Brian's
"paths stated explicitly" requires; the md1 identity with `--template` was
confirmed for five standard pairs).

## Review r2 disposition

| id | finding | disposition |
| --- | --- | --- |
| I1 | twin check over "f! orders" misses twins under pins, cannot run at large n | §5: closed form, one `multi` order per matched target, compared across targets, O(targets × n log n); the twin permutes fragment arguments, pins do not reduce it; rows M1p, M5 restated |
| I2 | sorted collapse unsound for multi-fragment templates | §4 pruning 4: collapse only for one sorted fragment that is the whole script, every slot once, equal use-site paths, with the soundness argument; f! otherwise; rows L1, L2; template class in §2 |
| I3 | bare cosigner xpub command exits 2, not 4 | §6: header read; consistent → explicit path, assumed, exit 4; not consistent → placeholder, null command, exit 4 (`origin_unknown`); rows B1, B2; question 8 |
| I4 | path step ≥ 2²⁹ refused by md1 | §1 "Found but not mintable"; §3 Samourai hint; §5 reason `path_unrecordable`; §6 text; row H1; FOLLOWUP `address-of-seed-md1-path-step-range`; question 9 |
| I5 | "give a second address" fails half the time | §5 and §6: first three distinguishing indices with both addresses (M1: receive 1, 3, 5, minted and checked); JSON `distinguishing`; user template marked; row D1; question 6 |
| I6 | fingerprint-first placement refuses real key holders | §2: placement by derivation for every (seed, key) pair, fingerprint a hint and a reported mismatch, real fingerprint in the command; row K1 (`00000000` cosigner fingerprint checked with `bundle`) |
| M1 | accepted template class | §2 "Accepted templates"; §9 oracle |
| M2 | rows use bare `@N` | rows written in full; bare `@N` refused (§2), row G1; `/**` stated |
| M3 | "repeated `--from`, as `restore` does" | §2 corrected (new code); one-stdin rule across all channels |
| M4 | redaction can make two stdin readers; variable names | §6: `-` only if nothing else reads stdin; `MNEMONIC_SEED_<i>` by seed index; `MNEMONIC_PASSPHRASE` |
| M5 | `--json` vs stderr estimate | §6 "Streams" |
| M6 | JSON shape for twins | §6: `result`/`reasons`, `role`, `twin_of`, `from_user_template`, `distinguishing`, `merged_forms`, template values, keyed `searched` |
| M7 | keyless md1 origin paths | §2: they pin the search; no-match hint |
| M8 | 2+ seeds, no descriptor, exit | §5; further row |
| M9 | engine details | §4.1 item 1; §4 ceilings (10¹⁵, `u64`, checked arithmetic) |
| M10 | twin check in keyed mode | §5: it runs; question 6 |
| M11 | lockstep sites | §8 |
| M12 | private material on `--cosigner` | §2, §5, §7; row |
| M13 | mixed-type targets | §5: unproduceable targets unmatched (exit 4); refused only when none can be produced |
| N1 | merge defined by the window | §5: merge by symmetry only |
| N2 | cache size | §4: 0.64 MB |
| N3 | `address-of-xpub` help wording | §8 |
| N4 | `restore` cannot derive single-sig `tr` | §9 oracle |
| N5 | multisig sample | §6 multisig and ambiguous samples |

## Review r1 disposition

| id | finding | disposition |
| --- | --- | --- |
| C1 | `multi`/`sortedmulti` alias gives exit 0 for the wrong wallet; M2 pinned it | §5 twin check on every match, exit 4 if a twin fits every target; exit 0 redefined; M1/M2 now exit 4; rows M5, M6 (two targets), R1, R2 (taproot); open question 6 (r3: closed form, r2 I1) |
| I1 | engine reuse misdescribed | §1 and §4.1: `collect_all = true`, `early_exit = false`; new (path tuple, order) enumeration with unrank; structure-once evaluator (restore's measured at 165/240 µs per candidate); symmetry claim deleted |
| I2 | taproot per-candidate cost missing | measured (46 µs, planning 53 µs); multisig table has wsh/sh and tr columns; under-a-minute envelope and question 2 restated per family |
| I3 | keyed fast path misses stated origins | §2: keyed descriptors confirm targets without search; seeds placed at any stated path (r3: by derivation, r2 I6); test row |
| I4 | sorted collapse wrong with per-slot multipaths or pins | §4 pruning 4 conditions; row U1 (r3: restated, r2 I2) |
| I5 | chain model assumed `<0;1>` | §2 alternatives per slot, bare `/*`, `/*h` refused; real full path printed; JSON `alternative`/`step` |
| I6 | `m/45'` claimed as BIP-45 | §3 renamed in the report; BIP-45 cosigner-index in "Not covered" with a FOLLOWUP |
| I7 | GUI secret classification, invocation, persistence, manual-gui inventory and pin | §8 lists all of them |
| M1 | md1 refused by `bundle --descriptor` | §2 intake restated |
| M2 | `@N.path` refusal is per slot | §6 reworded |
| M3 | bare cosigner placeholders | §6 (r3: rewritten, r2 I3) |
| M4 | network forms by prefix | §2 classify by scriptPubKey, forms per network; rows T1, regtest |
| M5 | `bc1p` no-match hint | §5 |
| M6 | `m`, `m/0'`, Samourai, Electrum | §3 hint paths; `m` and Electrum in "Not covered" (`bundle` refuses `[fp]@0`, checked); r3: Samourai not mintable (r2 I4) |
| M7 | single-sig with 2+ seeds; `--cosigner` alone | §2 |
| M8 | dedupe of lazy window; gap bound; cache memory | §3 analytic dedupe; §2 `N ≤ 2³¹`; §4 cache size stated |
| M9 | exit-2 variant | §5 `XpubSearchRefusal` (and `XpubSearchUnverified`, exit 4) |
| M10 | secrets in a synthetic argv; `REDACTED_NOTE` | §6 reference-built argv, `redact_and_quote` second check, own note naming `@env:` |
| M11 | Refugium rounds per seat | §3 and §4: 8 paths per slot; question 3 rewritten |
| M12 | private keys in `--descriptor` | §2 and §7 refuse, never echo |
| M13 | TapBranch sorting at every depth; which md1 printed | §5 |
| M14 | unplaced multi-chunk mk1 grouping | §2 by chunk-set id |
| M15 | test gaps | §9 rows and further rows; M4 narrowed |
| N1 | full manual-gui paths | §8 |
| N2 | malformed phrase is `Bip39` | §5 table |
| N3 | 1-based seed index; `candidates` meaning | §6 0-based; `child_keys` and `candidate_scripts` |
| N4 | exit-code scheme citation | §5 note: not yet a document; only "today" is normative |
| N5 | `m/45'` is one step | §4 table |
| N6 | gap is an index window | §2, §6 |
| N7 | where the redaction note goes | §6 |
