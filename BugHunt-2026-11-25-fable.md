# Bug hunt 2026-11-25 (Fable): mnemonic-toolkit

Report-only review of `mnemonic-toolkit` at `d72a18b`, part of a constellation-wide hunt across all 11 bg002h repositories. Nothing in this repository was changed by the hunt; this file is the only addition.

**How it was done.** Eight Fable reviewers each audited an area (one of them the boundaries between repositories), reproducing what they could with locally built binaries or scratch tests that were not committed. Three further Fable verifiers then tried to disprove every High and Medium finding (and several Low ones) and re-graded them. Grades below are the verifiers' final grades where they ran.

**Grades.** Critical: can lose funds or leak a seed in ordinary use. High: wrong wallet, addresses or keys, or secret exposure, plausible in normal use, or recovery broken. Medium: needs an unusual but realistic input, or fails in a way a careful user might not notice. Low: fails closed, hardening, or a small exposure. Info: context, not a bug.

**Categories.** security (secret or key exposure, injection, integrity of what is trusted), funds (wrong address, amount, descriptor or recovery that could lose or strand coins), other.

**Status.** "Confirmed (reproduced)" means a command or test showed it. "Confirmed by reading" means the code path was traced but not executed. "Reviewer-confirmed, not re-verified" means the first reviewer reproduced or read it and no verifier re-checked it.

**Totals:** 6 Medium, 6 Low. No Critical findings.

## Summary

| ID | Grade | Category | Status | Finding |
|---|---|---|---|---|
| toolkit#1 | Medium | funds | Confirmed (reproduced) | `bundle --descriptor` with a canonical shape and bare `@N` builds the wallet on the MASTER xpub (and puts it on public cards) |
| toolkit#2 | Medium | funds | Confirmed (reproduced) | `restore --md1` invents an `m/87'/0'/0'` origin for a key whose card origin is empty |
| toolkit#3 | Medium | funds | Confirmed (reproduced) | Word Card parity budget far below what the CLI and manual state |
| toolkit#4 | Medium | funds | Confirmed (reproduced) | Template mode silently drops `@N.path=` / `@N.fingerprint=` on a seed slot |
| toolkit#5 | Medium | funds | Confirmed (reproduced) | An xpub slot gets the template default origin even when its header contradicts it |
| X4 | Medium | funds | Confirmed (reproduced) | WIF slots write a depth-0, zero-chain-code "xpub" into mk1 and md1 |
| toolkit#6 | Low | other | Confirmed (reproduced) | `--account` >= 2^31 panics in six commands |
| toolkit#7 | Low | other | Confirmed (reproduced) | Legacy `sh(sortedmulti)` defaults to BIP-48 `1'`; the manual says `2'` |
| toolkit#8 | Low | other | Confirmed (reproduced) | A WIF slot's network is never compared with `--network` |
| X2 | Low | other | Confirmed (reproduced) | The toolkit's md-codec pin `e11b4ce` ("0.48.7") lacks a fix listed under 0.48.7 |
| X6 | Low | other | Reviewer-confirmed, not re-verified | Non-ASCII handling in pinned ms/mk codecs; repair output lowercasing |
| X8 | Low | other | Reviewer-confirmed, not re-verified | Pin drift: mk1 chunk_set_id rule, ms-codec 0.7.0 `mnem` tag |

IDs: `md#`, `mk#`, `ms#`, `toolkit#`, `G`, `refugium#`, `wallet#`, `E`, `T`, `SH#`, `M` are per-repository findings; `X` findings are at a boundary between repositories and appear in every report they touch.

## Findings

## toolkit#1: `bundle --descriptor` with a canonical shape and bare `@N` builds the wallet on the MASTER xpub (and puts it on public cards)

**Grade:** Medium · **Category:** funds · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `toolkit`)

### 1. `bundle --descriptor` with a canonical shape and bare `@N` silently builds the wallet on the MASTER xpub

- **Severity:** High
- **Status:** CONFIRMED
- **Evidence:**
  - `crates/mnemonic-toolkit/src/cmd/bundle.rs:2497` (`bind_descriptor_mode_paths`: "Canonical with no inline
    origins: leave them EMPTY") — default-origin inference is gated on `is_non_canonical`.
  - `crates/mnemonic-toolkit/src/cmd/bundle.rs:1674` (phrase arm) and `:1768`: `master.derive_priv(&secp, &anno_path)`
    with `anno_path` empty, so the "account" key is the BIP-32 master key.
  - `crates/mnemonic-toolkit/src/cmd/bundle.rs:1532` and `:1543`: for a canonical shape `--account != 0` and
    `--slot @N.path=` on a seed slot are both refused, so the only way to get a non-master key is an inline
    `[fp/path]@N`.
- **What happens:** for every shape md-codec calls canonical (`pkh`, `wpkh`, `sh(wpkh)`, `tr` key-path,
  `wsh(multi|sortedmulti)`, `sh(wsh(multi|sortedmulti))`), a bare `@N` bound to a phrase/entropy/ms1 slot is
  derived at path `m`. The same command with `--template` derives at `m/84'/0'/0'` (or `m/87'/0'/0'`).
  Non-canonical shapes get a BIP-48 default plus an `info:` line; canonical shapes get the master key and no
  notice at all. The engraving card shows `@ (no path)` but nothing marks it as unusual.
- **User impact:**
  - The wallet is not the one the operator expects (`wpkh(@0/<0;1>/*)` is the BIP-388 spelling of a BIP-84
    wallet, and it is what a hardware wallet shows for a standard policy). Addresses are `m/0/i`, which
    no hardware signer or standard wallet derives.
  - The **master xpub** is engraved on the public mk1/md1 cards and goes into every watch-only export.
    Under BIP-32, a parent xpub plus any one non-hardened child private key yields the parent private key,
    so one leaked `m/0/i` key exposes every account of that seed.
  - The same on-wire encoding (canonical shape, `Shared(empty)` origin) means "canonical origin, elided"
    for a keyless `--md1-form=template` card and "master" for a keyed card, so readers disagree (finding 2).
- **Reproduction:**
  ```
  $M --allow-argv-secret bundle --network mainnet --descriptor 'wpkh(@0/<0;1>/*)' --slot "@0.phrase=$A" --json > d84.json
  $M inspect --mk1 <chunk0> --mk1 <chunk1>
    origin_fingerprint: 73c5da0a
    origin_path: m/
    xpub: xpub661MyMwAqRbcFkPHucMnrGNzDwb6teAX1RbKQmqtEF8kK3Z7LZ59qafCjB9eCRLiTVG3uxBxgKvRgbubRhqSKXnGGb1aoaqLrpMBDrVxga8
  $M --allow-argv-secret convert --from "phrase=$A" --to xpub --path m
    xpub: xpub661MyMwAqRbcFkPHucMnrGNzDwb6teAX1RbKQmqtEF8kK3Z7LZ59qafCjB9eCRLiTVG3uxBxgKvRgbubRhqSKXnGGb1aoaqLrpMBDrVxga8   # identical: the master key
  $M --allow-argv-secret restore --from "phrase=$A" --md1 ... (d84.json md1)
    descriptor: wpkh([73c5da0a]xpub661MyMw.../<0;1>/*)#9w5tkh9r
    first recv: bc1qrh98qvlnec9k9au5auntfj3y2tmmw9w0353lqd          # BIP-84 vector is bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
  ```
  Multisig: `--descriptor 'wsh(sortedmulti(2,@0/**,@1/**))' --slot "@0.phrase=$A" --slot "@1.phrase=$B" --json`
  gives `"origin_path": ""` and both cosigners `xpub661MyMwAqRbc…` with `"origin_path": ""`. stderr carries only
  the generic "stdout carries private key material" warning. The sibling `md decode` (md 0.21.1) reads the card
  as `@0: [73c5da0a/m]`. `verify-bundle --descriptor …` on the same inputs returns `result: ok`, so the round trip
  cannot catch it.
- **Not already tracked:** FOLLOWUPS `bundle-descriptor-drops-per-slot-path` (resolved v0.102.0) covers a supplied
  path being dropped; `restore-multisig-descriptor-carries-master-xpubs-under-derived-origins` covers a
  depth-0 *header* over correct key material. Here the key material itself is the master key (chain code and
  point equal `convert --path m`).
- **Fix direction:** for a canonical shape with a bare `@N` bound to a seed slot, either derive at the canonical
  origin (what `--template` does) or refuse with a message naming `[fp/path]@N`. Do not mint a depth-0-key
  wallet without an explicit opt-in. Settle with md-codec what `Shared(empty)` means on a keyed canonical card.
- **Cross-repo:** descriptor-mnemonic (md-codec `canonical_origin`/`expand_per_at_n` semantics for an empty origin on a
  canonical keyed card); any consumer that reads such a card (mnemonic-engrave, seedhammer fork, refugium).

**Verifier re-check** (source: `verify-1`)

### toolkit#1: descriptor mode builds the wallet on the master xpub

**CONFIRMED-but-downgraded. Medium. funds, with a security side-effect.**

Reproduced: `mnemonic bundle --network mainnet --descriptor 'wpkh(@0/<0;1>/*)' --slot "@0.phrase=<abandon…about>" --json` exits 0; stderr holds only the argv and private-key warnings. `mnemonic inspect` on the mk1 gives `origin_path: m/` and an xpub identical to `convert --to xpub --path m`. `restore` gives `first recv: bc1qrh98qvlnec9k9au5auntfj3y2tmmw9w0353lqd`, which is not the BIP-84 address for that phrase.

Is it intended? Yes, by the code and a design spec, but not by anything a user reads:
- `cmd/bundle.rs:2497-2503`: "Canonical with no inline origins: leave them EMPTY … Defaulting a canonical shape would invent an origin the descriptor never claimed."
- `design/SPEC_toolkit_repin_sh_wpkh.md:27-31` pins it as required behaviour: "the origin stays EMPTY → derives at the MASTER key (`m/`), notice ABSENT (wpkh-parity)". The same spec describes this as "elides the origin like every other canonical shape", which is the confusion in the root-cause section above.
- The manual documents default-path inference only for non-canonical shapes ("Non-canonical descriptor mode", `41-mnemonic.md:607-628`). It says nothing about a canonical shape with a bare `@N` and a seed slot. The one hint is the refusal text "descriptor mode encodes account index in the @i origin path".

Would a real user hit it? Only by choosing `--descriptor` with a seed slot for a shape that `--template` already covers, and omitting `[fp/path]`. That is an unusual but realistic input. The manual's own canonical descriptor example (`37-wallet-export.md:262-268`) uses xpub slots, not phrases.

Why not High: the bundle is self-consistent. `restore --from phrase --md1 …` and `md address` on the card both return the same `bc1qrh98…`, so coins sent to the bundle's addresses are found again from the engraved cards. The harm is a silent non-standard wallet (`m/0/i`) that the operator's hardware wallet will not show under BIP-84, plus the true master xpub on public cards (hardening concern: master xpub plus any one leaked `m/0/i` private key gives the master private key).

**Verifier re-check** (source: `verify-1`)

### Are md#1, toolkit#1 and toolkit#2 the same root cause?

They are one design gap with three separate code defects, not one bug. The gap: nobody has fixed what an empty origin means on a *keyed* card with a canonical wrapper. Three readings are live at once:

- The md BIP says it is elided and MUST be resolved to the canonical origin (`bip/bip-mnemonic-descriptor.mediawiki:181-207`, `:449`).
- md-codec's renderer and the toolkit's descriptor-mode mint both read it as the master node (md#1, toolkit#1). These two agree with each other.
- The toolkit's plain-multisig restore reads it as the template default `m/87'/0'/0'` (toolkit#2), which is neither of the above (md's canonical origin for `wsh(sortedmulti)` is `m/48'/0'/0'/2'`).

They must be fixed together. If md#1 alone is fixed by resolving to the canonical origin, every card minted by toolkit#1 (which really does hold the master key) would start rendering as `[fp/84'/0'/0']<master key>`, which is toolkit#2's defect in a new place.

## toolkit#2: `restore --md1` invents an `m/87'/0'/0'` origin for a key whose card origin is empty

**Grade:** Medium · **Category:** funds · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `toolkit`)

### 2. `restore --md1` (and the template export builder) invents a `m/87'/0'/0'` origin for a key whose card origin is empty

- **Severity:** High (wrong key origin in the descriptor an operator imports)
- **Status:** CONFIRMED
- **Evidence:** `crates/mnemonic-toolkit/src/wallet_export/pipeline.rs:45-60` (`key_origin_str`: when the slot's path
  is empty it substitutes the template default path); pinned as intended by the unit test at `:303`
  (`key_origin_str_pathless_slot_uses_path_bearing_fallback`). Reached from `cmd/restore.rs:4062`
  (`build_descriptor_string` on the plain multisig arm) with slots built at `cmd/restore.rs:4037-4043`.
- **What happens:** for an md1 whose per-key origin is empty (as minted by finding 1, or by descriptor mode with
  `@N.xpub=` slots and no `@N.path=`), `restore` prints `[fp/87'/0'/0']<key>` — an origin the card does not
  carry and at which the key does not live. The cosigner lines in the same output print the origin as `[]`,
  and `--json` reports `"origin": ""`, contradicting the descriptor string beside them.
- **User impact:** the printed descriptor is the artefact that gets imported into a coordinator. A signer
  following `[73c5da0a/87'/0'/0']` derives a different key than the one in the script, so it refuses to
  register the wallet or cannot sign. Watch-only addresses are correct, which is why nothing flags it.
  Two different wallets print the identical origin string.
- **Reproduction:**
  ```
  $M --allow-argv-secret bundle --network mainnet --descriptor 'wsh(sortedmulti(2,@0/**,@1/**))' \
       --slot "@0.phrase=$A" --slot "@1.phrase=$B" --json > dms.json
  $M restore --md1 <each md1 chunk of dms.json>
    descriptor: wsh(sortedmulti(2,[73c5da0a/87'/0'/0']xpub661MyMwAqRbcFkPHuc…/<0;1>/*,[b8688df1/87'/0'/0']xpub661MyMwAqRbcFS99u…/<0;1>/*))#jn4ddzec
    first recv: bc1qlfadjuw8h2ttxqz937jdgxcqpxmqpxfjckmgjd2rgalwvykewt0sppsp3x
    cosigner @0: 73c5da0a []  from md1 (not independently verified)
  # same phrases through --template wsh-sortedmulti --threshold 2, then restore:
    descriptor: wsh(sortedmulti(2,[73c5da0a/87'/0'/0']xpub6BemYiVNp19a1Syy…          # same origin text, different key
    first recv: bc1q7cty630wwazpl882zg2p65laer5rrlrfr6wf5g3588x5rj72wxlq2k805e
  ```
  With `--from "phrase=$A"` the output still says `cosigner @0: 73c5da0a []  ← your seed (verified)` under the
  `87'/0'/0'` descriptor. With xpub slots and no fingerprint the result is `[00000000/87'/0'/0']xpub661My…`.
- **Fix direction:** a pathless slot must render as `[fp]key` (or be refused), never with a guessed path; the
  fallback in `key_origin_str` should apply only where the caller has itself derived the key at that path.

**Verifier re-check** (source: `verify-1`)

### toolkit#2: `restore --md1` invents an `m/87'/0'/0'` origin

**CONFIRMED-but-downgraded. Medium. funds.**

Reproduced on a 2-of-2 minted in descriptor mode: `restore` prints `wsh(sortedmulti(2,[73c5da0a/87'/0'/0']xpub661MyMwAqRbcFkPHuc…` while the next lines print `cosigner @0: 73c5da0a []` and `--json` reports `"origin":""`. The key under that origin is a depth-0 xpub. With xpub slots and no fingerprint (`--descriptor 'wsh(sortedmulti(2,@0/**,@1/**))' --slot @0.xpub=… --slot @1.xpub=…`, which is the manual's example plus `/**`) the result is `[00000000/87'/0'/0']xpub661My…`. Cause: `wallet_export/pipeline.rs:45-60` substitutes the template default for an empty path, pinned by a unit test. md prints the same card as `[73c5da0a]xpub661…`, so the two tools disagree.

Why not High: the address is the same in every reading (`bc1qlfadjuw8…` from both the toolkit and md). The defect is an affirmatively wrong origin in the descriptor an operator imports, which blocks signing until corrected. It needs an input card with empty origins, which only descriptor mode without paths produces. The recorded follow-up `restore-multisig-descriptor-carries-master-xpubs-under-derived-origins` is a different defect (header only).

## toolkit#3: Word Card parity budget far below what the CLI and manual state

**Grade:** Medium · **Category:** funds · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `toolkit`)

### 3. Word Card parity budget is far below what the CLI and manual state; one wrong word in 12 of 92 positions is unrecoverable

- **Severity:** Medium (backup durability overstated; decoder fails closed, never returns wrong data)
- **Status:** CONFIRMED
- **Evidence:**
  - Claim: `mnemonic word-card --help` and `docs/manual/src/40-cli-reference/41-mnemonic.md:5138` —
    "`--parity-words <N>` … corrects `⌊m/2⌋` substitutions / fills `m` erasures".
  - Cause: `crates/wc-codec/src/sync.rs:353,362,730,736` — a block whose CRC-5 fails, or whose checkpoint word
    is unreadable, is turned into a whole-block erasure (`sz` or `sz+1` words) before
    `rs_decode` (`crates/wc-codec/src/pipeline.rs:1138`). One substituted word therefore costs 8–9 units of the
    budget `m`. The 5 header words are CRC-checked before RS and are not correctable at all.
- **Reproduction** (md1 of the BIP-84 `A` bundle, 644 payload bits):
  ```
  $M word-card --from "<3 md1 chunks>" --parity-words 8      # 92 words, advertised: corrects 4 substitutions
  ```
  Replacing one word at each position in turn and decoding with `word-card --decode`:
  positions 0–4 → `header-CRC mismatch (corrupt geometry) — refuse`; positions 19, 28, 37, 46, 55, 64, 73
  (checkpoint words) → `uncorrectable (beyond RS budget) — refuse`. All other single positions decode.
  Random substitutions outside the header (script `work-tk/wcstat.py`, fixed seed):

  | substitutions | m=8 (claimed 4) recovered | m=16 (claimed 8) recovered |
  |---|---|---|
  | 1 | 38/40 | 25/25 |
  | 2 | 7/40 | 24/25 |
  | 3 | 2/40 | 6/25 |
  | 4 | 0/40 | 5/25 |
  | 8 | — | 0/25 |

  No trial returned a wrong payload.
- **User impact:** an operator who sizes parity from the documented rule gets roughly a quarter to an eighth of
  the stated substitution tolerance; a single mis-stamped checkpoint or header word loses the plate.
- **Fix direction:** try errors-only RS decoding (no erasures) first, and fall back to block erasures only when
  that fails; treat a damaged checkpoint as one error rather than a block; or correct the stated budget.

**Verifier re-check** (source: `verify-3`)

### toolkit#3 — Word Card parity is far weaker than the CLI and manual state

- Verdict: **CONFIRMED**. Grade: **Medium**. Category: **funds** (backup durability; fails closed).
- Claim: `mnemonic word-card --help`: "`--parity-words <N>` … corrects `⌊m/2⌋` substitutions / fills `m` erasures".
- Re-run (own script, `work-tk/wcp8.txt`, 92 words, `--parity-words 8`, each word replaced in turn, then
  `word-card --decode`): **12 of 92** single substitutions are not recovered: positions 0-4
  `header-CRC mismatch (corrupt geometry) — refuse`; positions 19, 28, 37, 46, 55, 64, 73
  `uncorrectable (beyond RS budget) — refuse`. Every recovered data-word case reports `erasures_filled: 8`, so one
  wrong word uses the whole m = 8 budget and a second wrong word in another block cannot be corrected (claimed: 4).
  No run returned a wrong payload.
- Cause: `crates/wc-codec/src/sync.rs:351-362`: a failed block CRC-5 erases the whole block; an unreadable
  checkpoint erases block plus checkpoint (9 > 8). No caveat in the manual; not in FOLLOWUPS.

## toolkit#4: Template mode silently drops `@N.path=` / `@N.fingerprint=` on a seed slot

**Grade:** Medium · **Category:** funds · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `toolkit`)

### 4. Template mode silently discards `@N.path=` / `@N.fingerprint=` given with a seed slot

- **Severity:** Medium
- **Status:** CONFIRMED
- **Evidence:** `crates/mnemonic-toolkit/src/slot_input.rs:319-337` admits `{phrase|seedqr|ms1, path}` and
  `{…, fingerprint, path}` (meant for non-canonical descriptor mode); `crates/mnemonic-toolkit/src/cmd/bundle.rs:577-625`
  (`resolve_slots`, phrase/seedqr arm; same for the entropy and ms1 arms) never reads the `Path` or `Fingerprint`
  subkeys. The descriptor-mode refusal lives only in `bundle_run_unified_descriptor`.
- **Reproduction:**
  ```
  echo "$A" | $M bundle --network mainnet --template bip84 --slot @0.phrase=- \
       --slot "@0.path=m/84'/0'/5'" --slot @0.fingerprint=deadbeef --json
    … "origin_path":"m/84'/0'/0'","master_fingerprint":"73c5da0a" …      exit 0, no warning
  ```
  The output is byte-identical to the run without the two extra slots.
- **User impact:** an operator who asks for account 5 by path gets cards for account 0 and no signal. The
  repo's own rule (FOLLOWUPS, `bundle-descriptor-drops-per-slot-path`): "a path the operator supplies must
  reach the artifact or be refused, never be silently dropped".
- **Fix direction:** in template mode refuse `@N.path=`/`@N.fingerprint=` on a seed-bearing slot (pointing to
  `--account`), or honour the path.

**Verifier re-check** (source: `verify-1`)

### toolkit#4: `@N.path=` / `@N.fingerprint=` silently dropped in template mode

**CONFIRMED. Medium. funds.**

`echo "$A" | mnemonic bundle --network mainnet --template bip84 --slot @0.phrase=- --slot "@0.path=m/84'/0'/5'" --slot @0.fingerprint=deadbeef --json` exits 0, reports `origin_path m/84'/0'/0'`, fingerprint `73c5da0a`, and `cmp` shows the output byte-identical to the run without the two extra slots. No warning. An operator asking for account 5 gets account-0 cards. The seed card still recovers everything, and the printed origin shows account 0 to a reader who checks, hence Medium.

## toolkit#5: An xpub slot gets the template default origin even when its header contradicts it

**Grade:** Medium · **Category:** funds · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `toolkit`)

### 5. An xpub slot with no `@N.path=` is given the template default origin even when the xpub's own header contradicts it

- **Severity:** Medium
- **Status:** CONFIRMED behaviour; the missing check is a recorded design decision, so treat the severity as a judgement call
- **Evidence:** `crates/mnemonic-toolkit/src/synthesize.rs:666` (default path applied), `:669-672` (the
  depth-consistency reject was removed in v0.37.10), `synthesize.rs:131-150` (`mk1_origin_path` splices the
  default path's prefix onto the xpub's own child number). `cmd/verify_bundle.rs:3681-3712` only compares the
  overlapping prefix, so it stays silent too.
- **Reproduction:** BIP-48 xpubs (depth 4, child `2'`), fingerprints supplied, no path:
  ```
  X48A=$($M convert --from "phrase=$A" --to xpub --path "m/48'/0'/0'/2'")   # xpub6DkFAXWQ2dHxq2vat…
  $M bundle --network mainnet --template wsh-sortedmulti --threshold 2 \
     --slot @0.xpub=$X48A --slot @0.fingerprint=73c5da0a --slot @1.xpub=$X48B --slot @1.fingerprint=b8688df1 --json
    cosigners: ('73c5da0a', "m/87'/0'/0'"), ('b8688df1', "m/87'/0'/0'")
  $M inspect --mk1 …      →  origin_path: m/87'/0'/0'/2'        # a path that exists in neither wallet
  $M export-wallet … --format descriptor
    wsh(sortedmulti(2,[73c5da0a/87'/0'/0']xpub6DkFAXWQ2dHxq…/<0;1>/*,…
  ```
  No warning at bundle, export or verify time (verify prints only its generic watch-only notices).
- **User impact:** most hardware-wallet multisig xpubs are BIP-48; the toolkit default family is BIP-87. An
  operator who omits the path gets cards and exports asserting `m/87'/0'/0'` for a key at `m/48'/0'/0'/2'`.
  A signer given that origin derives a different key and cannot sign. The xpub header (depth 4 against a
  3-component default) is enough to know the default is wrong.
- **Fix direction:** when the path is defaulted (not supplied) and the xpub's depth or child number disagrees with
  it, refuse or at least warn and require `@N.path=`. Keep the tolerance only for a path the operator typed.

**Verifier re-check** (source: `verify-1`)

### toolkit#5: xpub header contradicting the template default origin

**CONFIRMED. Medium. funds.**

With two `m/48'/0'/0'/2'` xpubs (depth 4), fingerprints given and no `@N.path=`, `bundle --template wsh-sortedmulti --threshold 2` exits 0 with only the watch-only note and records `origin_path: m/87'/0'/0'` for both cosigners. `restore` then prints `[73c5da0a/87'/0'/0']xpub6BemYiVNp19a1…`, a re-stamped depth-3 xpub that is not the one supplied. The removed check is a recorded decision (`synthesize.rs:669-673`, v0.37.10), but that rationale covers a path the operator typed, not one the tool defaulted. BIP-48 xpubs are the common hardware-wallet multisig export, so the input is ordinary; the consequence is wrong origin metadata with correct addresses, hence Medium and not High.

## X4: WIF slots write a depth-0, zero-chain-code "xpub" into mk1 and md1

**Grade:** Medium · **Category:** funds · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `cross`)

### X4. Toolkit WIF slots write a "fixed key" (depth 0, all-zero chain code) into mk1 and md1; only unreleased mk-cli knows not to derive from it

- **Severity:** Medium (addresses no wallet watches), needs confirmation by the md and toolkit reviewers
- **Status:** SUSPECTED (reading only)
- **Repos:** mnemonic-toolkit, mnemonic-key, descriptor-mnemonic, mnemonic-gui (pinned mk-cli), mnemonic-refugium, seedhammer
- **Evidence:** `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/bundle.rs:785-822` builds an `Xpub` with
  `depth: 0`, `chain_code: [0; 32]`, fingerprint `00000000` from a `--slot @N.wif=` and emits it in the bundle's
  cards. mk-cli HEAD (Unreleased, bh-mk-C1, `mnemonic-key/crates/mk-cli/CHANGELOG.md` "`mk address` and `mk derive`
  refuse a fixed-key card") refuses to derive children from such a card, because "it has no child keys any wallet
  watches". No other reader has that rule: `grep -rni "zero chain code\|all-zero chain\|fixed-key"` over
  descriptor-mnemonic, mnemonic-refugium, mnemonic-engrave and seedhammer (`mk/`, `gui/`, `address/`) finds nothing.
  The GUI and the installer pin `mk-cli-v0.17.0` (`mnemonic-gui/pinned-upstream.toml:70`), which is before the fix.
- **Impact:** `mk address` 0.17.0 (the pinned release), and any md-side or device-side address derivation over an
  md1 that carries such a key under a `/<0;1>/*` use-site, prints child addresses of a key that is not an HD key.
  Coins sent there are spendable only by software that treats the WIF key as an xprv with a zero chain code.
- **Fix direction:** put the fixed-key predicate in md-codec (derive and render paths) and port it to the Go
  `md`/`mk` packages; release mk-cli 0.18.0 and move the GUI and installer pins.

**Verifier re-check** (source: `verify-1`)

### cross X4: WIF slot writes a depth-0, zero-chain-code xpub onto cards

**CONFIRMED. Medium. funds.**

`mnemonic bundle --network mainnet --template bip84 --slot @0.wif=KwDiBf89…` exits 0 and emits an mk1 with `origin_path: m/` and an md1 `wpkh(@0/<0;1>/*)`. The toolkit's own `restore --md1` prints `first recv: bc1qgl57xjd595rrxmpwys85q40xh4nmqunqfacg4j`, and `md address` prints the same plus `bc1qcw7uxw07…`. Those are children of the WIF public key under an all-zero chain code, not the WIF key's own address. `mk address` at HEAD refuses the card ("holds one fixed key (depth 0, all-zero chain code; the toolkit writes a WIF key this way) … mk derives nothing from it"); md, the toolkit and the GUI's pinned `mk-cli-v0.17.0` (`mnemonic-gui/pinned-upstream.toml:70`) have no such rule. The toolkit's own comment says the key is "non-derivable" (`cmd/bundle.rs:786-789`) while the template gives it a ranged use-site. Coins sent to the printed address are invisible to any wallet that imports the WIF normally, and are spendable only by treating the WIF as an xprv with a zero chain code. Unusual input, recoverable with custom tooling, hence Medium. I did not run the Go device or mnemonic-refugium against this card.

## toolkit#6: `--account` >= 2^31 panics in six commands

**Grade:** Low · **Category:** other · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `toolkit`)

### 6. `--account` at or above 2^31 panics in six commands

- **Severity:** Low
- **Status:** CONFIRMED
- **Evidence:** `crates/mnemonic-toolkit/src/template.rs:77-78` (`.expect("template paths are well-formed by construction")`).
  FOLLOWUPS `xpub-search-account-window-2p31-panic` fixed the same class for `xpub-search` only.
- **Reproduction:** each exits 101 with
  `thread 'main' panicked at crates/mnemonic-toolkit/src/template.rs:78:14: template paths are well-formed by construction: InvalidChildNumber(2147483648)`:
  ```
  $M bundle --network mainnet --template bip84 --slot "@0.phrase=$A" --account 2147483648
  $M bundle --network mainnet --template bip84 --slot "@0.xpub=<xpub>" --account 2147483648
  $M verify-bundle --network mainnet --template bip84 --slot "@0.phrase=$A" --account 2147483648 --bundle-json b84.json
  $M export-wallet --network mainnet --template bip84 --slot "@0.xpub=<xpub>" --account 2147483648
  $M addresses --from "phrase=$A" --address-type p2wpkh --account 2147483648
  $M restore --from "phrase=$A" --template bip84 --account 2147483648
  $M convert --from "phrase=$A" --to xpub --template bip84 --account 2147483648
  ```
  `--account 2147483647` is refused cleanly but with an internal message (`error: md1 varint overflow: 2147483647`).
- **Fix direction:** validate `--account < 2^31` at the clap boundary (one shared value parser), as was done for
  the `xpub-search` account window.

**Verifier re-check** (source: `verify-3`)

### toolkit#6 — `--account` >= 2^31 panics

- Verdict: **CONFIRMED**. Grade: **Low**. Category: **other**.
- `mnemonic bundle --network mainnet --template bip84 --slot "@0.phrase=…" --account 2147483648` ->
  `thread 'main' panicked at crates/mnemonic-toolkit/src/template.rs:78:14: template paths are well-formed by
  construction: InvalidChildNumber(2147483648)`, exit 101. Same panic from `addresses`. `--account 2147483647`
  gives `error: md1 varint overflow: 2147483647`. Fails closed, nothing is written.

## toolkit#7: Legacy `sh(sortedmulti)` defaults to BIP-48 `1'`; the manual says `2'`

**Grade:** Low · **Category:** other · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `toolkit`)

### 7. Legacy `sh(multi|sortedmulti)` in descriptor mode defaults to the BIP-48 P2SH-P2WSH branch (`1'`); the manual says `2'`

- **Severity:** Low
- **Status:** CONFIRMED
- **Evidence:** `crates/mnemonic-toolkit/src/template.rs:293-299` (`bip48_script_type_for_root_tag`: any `Tag::Sh`
  root → `ShWshSortedMulti` → `1'`, without looking at the child). BIP-48 text (fetched from
  `bitcoin/bips` master, `bip-0048.mediawiki` lines 107–110): "`1'`: Nested Segwit (p2sh-p2wsh)",
  "`2'`: Native Segwit (p2wsh)"; BIP-48 defines no script type for bare P2SH.
  `docs/manual/src/40-cli-reference/41-mnemonic.md` ("Non-canonical descriptor mode") states the default is
  `m/48'/<coin>'/<account>'/2'`.
- **Reproduction:**
  ```
  $M bundle --network mainnet --descriptor 'sh(sortedmulti(2,@0/**,@1/**))' --slot "@0.phrase=$A" --slot "@1.phrase=$B" --json
    info: no origin supplied for @0,@1; defaulting to m/48'/0'/0'/1' (BIP-48 cosigner path). …
  ```
- **User impact:** a legacy P2SH multisig reuses the exact keys of the same seeds' P2SH-P2WSH wallet (key reuse
  across script types), on a branch BIP-48 assigns to a different script; the manual and the binary disagree.
- **Fix direction:** look at the `sh` child (`wsh` → `1'`); for bare `sh(multi)` require an explicit origin or pick
  and document a distinct default; align the manual.

**Verifier re-check** (source: `verify-3`)

### toolkit#7 — legacy `sh(sortedmulti)` defaults to BIP-48 script type `1'`

- Verdict: **CONFIRMED**. Grade: **Low**. Category: **other**.
- BIP-48 (fetched from `bitcoin/bips` master, lines 107-112): "`1'`: Nested Segwit (p2sh-p2wsh)", "`2'`: Native
  Segwit (p2wsh)"; nothing for bare P2SH. `crates/mnemonic-toolkit/src/template.rs:293-299` maps every `Tag::Sh`
  root to `1'` without looking at the child. Run: `bundle --descriptor 'sh(sortedmulti(2,@0/**,@1/**))' …` ->
  `info: no origin supplied for @0,@1; defaulting to m/48'/0'/0'/1' (BIP-48 cosigner path)`. The manual says `2'`
  (`docs/manual/src/40-cli-reference/41-mnemonic.md:623`).
- Why Low: the chosen path is announced and recorded on the cards, so the wallet is recoverable; the cost is key
  reuse with the same seeds' P2SH-P2WSH wallet and a manual that disagrees with the binary.

## toolkit#8: A WIF slot's network is never compared with `--network`

**Grade:** Low · **Category:** other · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `toolkit`)

### 8. WIF slot: the key's network is never compared with `--network`

- **Severity:** Low
- **Status:** CONFIRMED
- **Evidence:** `crates/mnemonic-toolkit/src/cmd/bundle.rs:800-812` — `PrivateKey::from_wif` result is used for its
  public key only; the synthetic xpub is stamped with `network.network_kind()`. The xpub arm does check
  (`synthesize.rs:643-653`).
- **Reproduction:** mainnet WIF under `--network testnet`:
  ```
  $M --allow-argv-secret bundle --network testnet --template bip84 \
     --slot @0.wif=KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn --json
    {"schema_version":"4","mode":"watch-only","network":"testnet", …}      exit 0
  ```
- **User impact:** a mainnet key is silently packaged as a testnet wallet (or the reverse). The compressed flag
  of the WIF is also dropped (the compressed point is always used), so an uncompressed-key `bip44` wallet would
  show different addresses.
- **Fix direction:** `assert_network_agrees(priv_key.network, …)` as for xpubs; refuse an uncompressed WIF or state it.

**Verifier re-check** (source: `verify-3`)

### toolkit#8 — a WIF slot's network is never compared with `--network`

- Verdict: **CONFIRMED**. Grade: **Low**. Category: **other**.
- `bundle --network testnet --template bip84 --slot @0.wif=KwDiBf89…VHnoWn --json` (a mainnet WIF) ->
  `"network":"testnet"`, exit 0, no warning. `crates/mnemonic-toolkit/src/cmd/bundle.rs:800-812` uses the parsed key
  for its public point only and stamps `network.network_kind()`. The compressed flag is dropped the same way
  (`pubkey.inner`, read only, not run). A WIF slot is already a degenerate depth-0 key, so reach is small.

## X2: The toolkit's md-codec pin `e11b4ce` ("0.48.7") lacks a fix listed under 0.48.7

**Grade:** Low · **Category:** other · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `cross`)

### X2. "md-codec 0.48.7" names two different codecs: the release commit lacks a fix the CHANGELOG lists under 0.48.7

- **Severity:** Medium
- **Status:** CONFIRMED (source diff)
- **Repos:** descriptor-mnemonic, mnemonic-toolkit (pins it), and every consumer that reads the md CHANGELOG to
  decide what a pin holds (refugium, engrave, refugium-wallet, gui)
- **Evidence:**
  - Release commit `e11b4ce` "release: md-cli 0.21.1 + md-codec 0.48.7". The toolkit pins exactly that
    (`mnemonic-toolkit/Cargo.lock`, commit `8ed773b` "pin e11b4ce1 (= md-codec-v0.48.7)").
  - `4b763d7` (same day, after the release in history: `git log e11b4ce..HEAD -- crates/md-codec/src`) adds
    `check_chain_in_range` to `to_miniscript_descriptor_with_network` and puts its CHANGELOG bullet inside the
    already released `## md-codec [0.48.7]` section (`descriptor-mnemonic/CHANGELOG.md:86-89`). The crate version
    stayed `0.48.7` (`crates/md-codec/Cargo.toml:3`).
  - `diff mnemonic-toolkit/vendor/md-codec/src/to_miniscript.rs descriptor-mnemonic/crates/md-codec/src/to_miniscript.rs`:
    the vendored 0.48.7 has no `check_chain_in_range`; HEAD's 0.48.7 has it (lines 98, 123-143). `git show
    e11b4ce:CHANGELOG.md` has one bullet under 0.48.7; HEAD has two.
- **Impact:** with any pinned md-codec in the suite (0.47.0, 0.48.4, 0.48.5, and the tagged 0.48.7),
  `to_miniscript_descriptor(d, chain)` for a descriptor where no key has a multipath ignores `chain` and returns the
  chain-0 descriptor for every chain; a caller asking for "change" gets the receive descriptor at `Ok`. A reader of
  the CHANGELOG concludes the toolkit's 0.48.7 is safe; it is not. Current callers: the toolkit calls chain 0 only
  (`synthesize.rs:1004`, `template.rs:598,699`); mr-codec calls chains 0 and 1 but first overwrites the use-site
  with a two-alternative multipath (`mnemonic-refugium/crates/mr-codec/src/render.rs:171,182-187`), so no shipped
  caller was found that hits it today. The released md-cli 0.21.1 CHANGELOG also lists `md descriptor --change`
  refusing such a wallet; check whether tag `md-cli-v0.21.1` holds `4b763d7` (the CLI fix needs this codec change).
- **Reproduction:** the `diff` command above; `grep -c check_chain_in_range mnemonic-toolkit/vendor/md-codec/src/to_miniscript.rs` gives 0.
- **Fix direction:** release the change as 0.48.8 and move its CHANGELOG bullet; add a CI check that a released
  section's text does not change after its release commit.

**Verifier re-check** (source: `verify-3`)

### X2 — "md-codec 0.48.7" names two different codecs

- Verdict: **CONFIRMED-but-downgraded**. Grade: **Low**. Category: **other** (release hygiene).
- `e11b4ce` "release: md-cli 0.21.1 + md-codec 0.48.7" (2026-11-22 11:40) is an ancestor of `4b763d7`
  (12:52, `git merge-base --is-ancestor e11b4ce 4b763d7`). `4b763d7` changes `crates/md-codec/src/{derive,error,
  to_miniscript}.rs` and adds bullets under the already released headings: `CHANGELOG.md:86` (md-codec [0.48.7]) and
  `:42` (md-cli [0.21.1]); `git show e11b4ce:CHANGELOG.md` has one bullet under 0.48.7. `crates/md-codec/Cargo.toml`
  is `0.48.7` at both commits. The toolkit pins `e11b4ce` (`mnemonic-toolkit/Cargo.lock:686-688`);
  `grep -c check_chain_in_range mnemonic-toolkit/vendor/md-codec/src/to_miniscript.rs` = 0, HEAD = 2.
- Why Low: no shipped caller hits the missing check. The toolkit calls `to_miniscript_descriptor(_, 0)` only
  (`synthesize.rs:1004`, `template.rs:598,699`). The harm is a CHANGELOG that says a pinned 0.48.7 has a fix it lacks
  (md-cli 0.21.1 likewise). Fix: release as 0.48.8 / 0.21.2 and move the bullets.

## X6: Non-ASCII handling in pinned ms/mk codecs; repair output lowercasing

**Grade:** Low · **Category:** other · **Status:** Reviewer-confirmed, not re-verified

**Reviewer write-up** (source: `cross`)

### X6. Non-ASCII characters are still read as codex32 symbols by the pinned ms-codec and mk-codec

- **Severity:** Low
- **Status:** CONFIRMED (probes)
- **Repos:** mnemonic-toolkit and mnemonic-gui (ms-codec 0.7.0, mk-codec 0.4.1); mnemonic-refugium and
  refugium-wallet (ms-codec 0.9.0, mk-codec 0.5.0); mnemonic-engrave and descriptor-mnemonic (mk-codec 0.5.0);
  refugium-wallet also md-codec 0.48.4 (the md fix is in 0.48.5; CHANGELOG only)
- **Evidence and reproduction:**
  - ms: fixed in ms-codec 0.10.1 (bughunt ms #7). `ms-codec/src/decode.rs:184` (0.7.0) and `:208` (0.9.0):
    `let lc = c as u8;`. A valid card with two characters changed, one of them to U+0133 (which truncates to `3`,
    the right symbol there):
    ```
    $ probe-tk ms-fix "ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc8sh9lf92n5fq3ĳs"
    Ok(("entr", "Entr([0, 1, ..., 15])", 1))          # 0.7.0: accepted, one correction reported
    $ probe-rf ms-fix  (same string)   -> Ok(..., 1)   # 0.9.0
    $ probe-msh ms-fix (same string)   -> Err(Error("codex32 parse error: InvalidChar('ĳ')"))   # HEAD
    ```
    The toolkit reaches this through `repair` (`crates/mnemonic-toolkit/src/repair.rs:1467,1540`). Its
    `apply_ms_corrections` (`repair.rs:1507-1527`) rebuilds the string from `chunk.to_lowercase()` (Unicode, not
    ASCII), so the "corrected" card it prints keeps the non-ASCII character (and shifts positions for a character
    whose lowercase is two characters, such as U+0130): the printed repair is not a valid ms1.
  - mk: fixed in mk-codec 0.5.1. `string_layer/bch.rs:664` (0.4.1) and `:673` (0.5.0) use `s.to_lowercase()`.
    An all-uppercase card with one `K` replaced by U+212A KELVIN SIGN:
    ```
    $ probe-tk mk-dec MK1QPN2ZNPQQSQ64W7VM4EUT<U+212A>S2Q... MK1QPN2ZNPP0F30...   -> Ok(KeyCard {...})   # 0.4.1
    $ probe-rf  (same)  -> Ok(KeyCard {...})                                         # 0.5.0
    $ probe-mkh (same)  -> Err(InvalidChar { ch: 'K', position: 21 })                # 0.6.0
    ```
- **Impact:** a card typed with a look-alike character is accepted (mk) or "repaired" (ms) instead of refused; in
  the toolkit the printed ms1 repair can be an invalid string. No wrong key or wrong seed results: the checksum
  still has to hold.
- **Fix direction:** move the pins (ms-codec >= 0.10.1, mk-codec >= 0.5.1); make `apply_ms_corrections` use
  ASCII lowercasing.

## X8: Pin drift: mk1 chunk_set_id rule, ms-codec 0.7.0 `mnem` tag

**Grade:** Low · **Category:** other · **Status:** Reviewer-confirmed, not re-verified

**Reviewer write-up** (source: `cross`)

### X8. Smaller pin and label drift

- **Severity:** Low. **Status:** CONFIRMED by reading.
- `mnemonic-gui/pinned-upstream.toml:42` pins `md-cli-v0.21.0`; md-cli 0.21.1 is released (`e11b4ce`) with four
  fixes, among them `md encode` minting an undecodable card and duplicate `--key` handling. The comment block above
  it (`:32-41`) still describes v0.20.3 and v0.11.0.
- mk-codec < 0.5 draws a random `chunk_set_id`; the toolkit avoids it for bundles by pinning
  `top20(policy_id_stub) ^ slot` (`synthesize.rs:77-99`), but `word_card_adapter.rs:165` calls plain
  `mk_codec::encode`, and the toolkit's rule differs from mk-codec >= 0.5 / the SeedHammer port
  (`top20(sha256(bytecode))`). The same key card therefore has different strings from `mnemonic bundle`, `mk encode`
  and the device. Decoders accept all three (documented in mk-codec's CHANGELOG as intended); a plate compared
  character by character against another tool's output will not match.
- ms-codec 0.7.0 `encode(Tag "mnem", Payload::Mnem)` returns `Ok("ms10mnems...")`, a card no version decodes
  (`probe-tk ms-enc-mnem 3 <hex> mnem`, then `ms-dec` gives "unknown tag "mnem"" on 0.7.0, 0.9.0 and HEAD). Fixed
  in 0.8.0. The toolkit only ever passes `Tag::ENTR` (`slot_ms1.rs:100,108,162`, `synthesize.rs:395,575,1169`,
  `convert.rs:1452`, `ms_shares.rs:314`), so it is not reached.
- mk-cli resolves md-codec 0.42.0 from crates.io (documented hazard in `mnemonic-key/CLAUDE.md`): it predates wire
  version 8 (0.46.0), so `--from-md1` of a Liana-unspendable card is refused there. Not re-tested.

---

_Generated by [Claude Code](https://claude.ai/code) on 2026-11-25. Findings were not fixed; each fix needs its own review under this repository's gates._

---

## Addendum: the 2026-11-25 bug hunt compared with the 2026-11-19 bug hunt

_Written 2026-11-26 for the coordinator that holds the 2026-11-19 fix queue. Read-only comparison: nothing here was merged or pushed to a default branch._

_This is the per-repo copy. The full addendum, with every repo's 11-19 table, is appended to `BugHunt-2026-11-25-fable.md` in mnemonic-refugium (report PR #247, branch `claude/project-thread-ly86x4`)._

### Why this exists, and how to use it

Two hunts ran a week apart:

- **2026-11-19** (OpenAI Codex, gpt-6). It covered 7 repos: descriptor-mnemonic (md), mnemonic-engrave (me), mnemonic-key (mk), mnemonic-secret (ms), mnemonic-toolkit (tk), mnemonic-transaction (mt) and seedhammer (sh). The reports are `design/agent-reports/bug-hunt-2026-11-19.md` on each default branch. The keys are `bh-<repo>-<id>`.
- **2026-11-25** (Fable). It covered all 11 repos. Each repo has a `BugHunt-2026-11-25-fable.md` report in an open, add-only PR on branch `claude/project-thread-ly86x4`. The fixes are in a second set of open PRs on branch `claude/project-thread-utm1aa`, one per repo.

Before you run any queued 11-19 item:

1. Look the key up in the per-repo tables below. Each table says where the fix is (the merged PR on the default branch), whether the 11-25 hunt found that bug again or a related one, and what is still left to do.
2. If a row's last column says "nothing", the item is done. Don't redo it.
3. Every 11-25 fix branch starts from the current default branch (checked 2026-11-26: 0 commits behind in all 10 repos). So every 11-19 fix merged so far is already underneath the 11-25 fix PRs, and the 11-25 PRs do not undo any of them.

#### The 11-25 PRs

| Repo | Report PR (11-25 findings) | Fix PR (11-25 fixes) |
|---|---|---|
| descriptor-mnemonic | #75 | #76 |
| mnemonic-engrave | #67 | #68 |
| mnemonic-key | #42 | #43 |
| mnemonic-secret | #47 | #48 |
| mnemonic-toolkit | #226 | #227 |
| mnemonic-transaction | #31 | #32 |
| seedhammer | #28 | #29 |
| mnemonic-refugium | #247 | #248 |
| mnemonic-gui | #57 | #58 |
| refugium-wallet | #228 | #229 |
| rust-miniscript | #1 | none (it has only Info findings) |

All of these PRs were open and unmerged when this addendum was written.

#### How the 11-19 fix status was worked out (and its limits)

- The starting point is the closure record `coordinator/agent-reports/recon-bughunt-1119-closure.md` in mnemonic-refugium (2026-11-24). It maps 125 Critical/Important keys to 58 FIX rows and found 27 of them open and 6 unrouted.
- After that record was written, more PRs merged on 11-24 and 11-25. I matched them to keys by the key names their commit titles cite: md#70, #71, #74; mk#37, #38, #39; ms#41, #42; tk#211, #212, #214, #220; mt#26, #27, #29; me#64; sh#25. With those, **every 11-19 Critical/Important key that had a FIX row is now merged.** What is still open are keys that were graded Minor and filed as FOLLOWUPS lines.
- "Merged" means the PR that names the key is on the default branch. I did not re-run each key's reproduction against the merged code.
- The closure record lists ms "I5" as the stdin-alias bug, but that bug is ms M5 in the source report. Source I5 is "verify calls a Japanese Mnem card v0.1 entr". Both are open FOLLOWUPS lines (see the ms table).

### Summary of the overlap

The 11-25 hunt has 50 distinct findings. Compared with the 11-19 hunt:

- **6 are the same bug as an 11-19 finding.** In every case the 11-19 fix merged, but only in one place: one repo, or Rust and not the Go port. The 11-25 hunt found the same bug somewhere else.
- **5 are related, and I'm not sure whether they count as the same bug** (marked UNCERTAIN).
- **About 39 are new.** That includes all 16 findings from the four repos the 11-19 hunt did not cover: mnemonic-gui, mnemonic-refugium, refugium-wallet and rust-miniscript.
- **None of the 11-19 bugs that are still open was found again on 11-25.**

| 11-25 finding | 11-19 key | Match | 11-19 fix (merged) | Where the bug was still present on 11-25 | 11-25 fix PR (open) |
|---|---|---|---|---|---|
| X7 (md-codec pin before 0.48.6 can mint a card the decoder refuses) | bh-md-C2 | SAME | md#59, released in md-codec 0.48.6/0.48.7 | consumers' md-codec pins (engrave, refugium) | engrave#68 `fc5f7c5`, refugium#248 `f8a0d9b`: both pin descriptor-mnemonic `c8cde23` |
| E5 (engrave's mt-codec pin) | bh-mt-I18 | SAME | mt#21 | engrave's mt-codec pin `eec7910b` | engrave#68 `67c3f1c`: pins mt `9b094e0b` |
| X4 (WIF slot writes a depth-0, zero-chain-code xpub onto cards) | bh-mk-C1 | SAME | mk#30 (mk-cli; not released yet) | toolkit mints these cards; md and the Go port derive from them; GUI and installer pin mk-cli 0.17.0 | tk#227 `59057e10` (refuses a WIF slot in template mode), md#76 `59c158e` (md refuses it). Still to do: release mk-cli and move the GUI/installer pins (gui#58 records that no release exists yet) |
| SH#6 (Go: change list equals receive list) | bh-md-I6 | SAME, Go port | md#64 (Rust) | seedhammer `address/`, `gui/` | seedhammer#29 `c2326a3` |
| SH#4 (Go `mk.Encode` lacks the origin refusals) | bh-mk-I9 (same function as bh-sh-I10) | SAME, Go port | mk#29 (Rust, mk-codec 0.6.0); sh#13 fixed only the network check | seedhammer `mk/encode.go` | seedhammer#29 `19b6075` |
| md#3 (placeholder with no wildcard minted as `/*`) | bh-md-C1 | SAME root (same lexer line, `parse/template.rs:345`) | md#60 fixed only the `md decompose` route | md template lexer (encode/address/descriptor) | md#76 `59c158e` and `6604c88` (bare `@N` refused at mint) |
| toolkit#3 (Word Card parity budget below what is stated) | bh-tk-I6 | UNCERTAIN: same weakness (one wrong word loses the card), different positions (header and checkpoint words today; the stop-sign word on 11-19) | tk#212 (stop-sign recovery) | header and checkpoint words | tk#227 `e6fd50c8`, `573dc14f`: the help and manual now state the budget the decoder really has. The decoder itself is unchanged |
| md#4 (`c:` over a non-key child mints a card md cannot restore) | bh-md-I5 | UNCERTAIN: same class (the encoder accepts what restore refuses); md#62 added the mint-time self-decode, but that check does not type-check | md#62 | md-codec mint check | md#76 `f8b4a67`, `ca48a05`, `5ad5a9b`, `fcefa83` (type rule at mint; address/descriptor refuse invalid miniscript unless `--allow-invalid-miniscript`) |
| SH#3 (Go `md.Decode` accepts four shapes Rust refuses) | bh-md-C2 | UNCERTAIN: one of the four shapes (an override keyed on baseline `@0`) is the card C2 used to mint | md#59 (Rust encoder) | Go decoder | seedhammer#29 `216a8af` |
| mk#2 (`mk verify --from-md1` passes a non-cosigner) | bh-mk-I2, I15 | RELATED: encode got the membership check, verify did not | mk#31 | `mk verify` | mk#43 `015d106` |
| toolkit#6 (`--account` >= 2^31 panics in six commands) | toolkit FOLLOWUP `xpub-search-account-window-2p31-panic` (listed as "previously filed" in the 11-19 tk report) | RELATED: same panic class; the earlier fix covered xpub-search only | tk#214 | bundle, verify-bundle, export-wallet, addresses, restore, convert | tk#227 `2d0057e8` |

Not an 11-19 match, but worth knowing:

- X6 (non-ASCII characters read as codex32 symbols by older ms/mk codec pins) repeats a bug from the **October 31** hunt that was fixed in the codecs. mk#43 `34c69b8` covers mk-cli's own md1 input. The consumer pins are tracked by the 11-25 X6/X8 entries.
- toolkit#7 (legacy `sh(sortedmulti)` defaults to BIP-48 `1'`) sits next to the 11-19 "previously filed" toolkit line `xpub-search-sh-wsh-default-origin-notice`, but it is a different wrapper and a different bug. It is fixed by tk#227 `6c99456c` plus the BIP-defaults follow-ups.
- engrave E3 (`--as md1` accepts a contradictory origin) is the md1 route of the ruling in `FIX_sec_canonical_keeps_xpub_depth` (me#54, 11-23). That ruling was not an 11-19 key. engrave#68 `094979e` fixes it.

### Work that the 11-25 PRs create for later

These items follow from the 11-25 fixes. Check the coordinator's queue against them so nothing is done twice:

- **md-codec 0.49.0 re-pins.** md#76 changes minting (explicit origins on keyed cards, the bare `@N` refusal, the type rule, thresh "any k of n"). After it merges, mnemonic-refugium, mnemonic-engrave (their md-codec pins must stay equal), mnemonic-toolkit, refugium-wallet and the seedhammer port each need a re-pin or a check. engrave#68 and refugium#248 pin `c8cde23` (0.48.7), not 0.49.0.
- **mk-cli release.** The bh-mk-C1 fix (mk#30) is still unreleased. gui#58 and the toolkit installer still pin `mk-cli-v0.17.0` (11-25 X4, G5).
- **seedhammer provenance pins.** seedhammer#29 `dde21fe` re-pins md, mk and mt to what the ports converge to (SH#5). Re-check them after md#76 merges.

### mnemonic-toolkit (tk): 11-19 keys

Source: `design/agent-reports/bug-hunt-2026-11-19.md` on `master`. 11-25 fix PR = tk#227.

| Key | 11-19 finding | Fixed by (merged) | 11-25 overlap | Left to do |
|---|---|---|---|---|
| C1 | SeedQR writes the seed to a world-readable file | tk#211 | none | nothing |
| I1 | Conflicting Coldcard header changes the quorum | tk#190 | none | nothing |
| I2 | Failed writes of derived secrets return success | tk#186 | none | nothing |
| I3 | Failed SeedQR replacement destroys the backup | tk#220 | none | nothing |
| I4 | BIP-85 accepts a non-master xprv | tk#189 | none | nothing |
| I5 | An env value switches the source to stdin | tk#191 | none | nothing |
| I6 | One stop-sign word error makes a Word Card unreadable | tk#212 | **toolkit#3 UNCERTAIN** (header/checkpoint words; tk#227 corrects the stated budget only) | decide whether the decoder should also recover a single header/checkpoint error (11-25 toolkit#3 "fix direction") |
| I7 | Mixed Word Card input drops positional words | tk#192 | none | nothing |
| I8 | Address range check allocates 8 GiB (graded M) | tk#214 | none | nothing |
| I9 | compare-cost help example fails | tk#193 | none | nothing |
| I10 | derive-child help names the wrong path | tk#193 | none | nothing |
| I11 | CompactSeedQR manual examples refuse their input | tk#193 | none | nothing |
| M1-M5, N1-N2 | Minors and Nits | FOLLOWUPS lines filed 2026-11-24 (`7e87e067`) | none re-found | as FOLLOWUPS say |
| prior table | 18 "previously filed" FOLLOWUPS lines | as FOLLOWUPS say; `xpub-search-account-window-2p31-panic` fixed in tk#214 | **toolkit#6 RELATED** (same panic in six other commands, tk#227 `2d0057e8`); `xpub-search-sh-wsh-default-origin-notice` sits next to toolkit#7 but is a different bug | tk#227 merge |

11-25 findings here with an 11-19 link: X4 (= bh-mk-C1, toolkit side, tk#227 `59057e10`). toolkit#1, #2, #4, #5, #7, #8, X2, X6, X8 are new.

