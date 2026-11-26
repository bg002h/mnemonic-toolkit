# CLI consistency review — 2026-11-25

**Repo:** `mnemonic-toolkit`. **Scope:** recommendations only; nothing in this file changes
code or behaviour. Every repo in the constellation except seedhammer and
rust-miniscript carries a copy of this file under the same name, next to its
`BugHunt-2026-11-25-fable.md`. Part A is specific to this repo. Part B (the
constellation standard, CLI-01 to CLI-19) is identical in every copy, so an item
id means the same thing wherever it is cited.

The goal, in the words the uniformity spec records from Brian: *"The goal is for
the user to not care if they are dealing with mk or md or mt or ms."* That spec
made the input/output and safety layer uniform. This review does the same for
the vocabulary: flag names, values, defaults, verbs and help.

## Decisions for Brian

These three choices shape several items. Each has a recommendation, and the rest
of the report is written assuming it.

1. **Usage-error exit code (CLI-14).** Five binaries exit 2 on a bad flag (`md`,
   `mt`, `me`, `mr`, `mr-sitting`) and three exit 64 (`mk`, `ms`, `mnemonic`).
   *Recommended: 2*, clap's default and the majority, so three binaries change.
2. **Consent-flag prefix (CLI-08). Decided by Brian on 2026-11-26: `--allow-`.**
   The other candidate was `--accept-` (used across mr). Attestations use
   `--confirm-`; `mr` is still alpha, so renaming its flags is cheapest now.
3. **`--template` and named shapes (CLI-06). Decided by Brian on 2026-11-26.**
   `--template` means BIP-388 template text only. Named wallet shapes become
   `--scheme NAME` (BIP-43: "Scheme described in BIP44"; BIP-388: "Common
   multisig schemes"), and named account paths become `--origin-path NAME`.
   Old names stay as hidden aliases, the toolkit's `--template` for longer than
   usual because it is its most-typed flag. This replaces the earlier
   `--preset` proposal, which failed the test of using BIP and bitcoin-dev
   wording; the consult is in the project's `cli-review/template-naming-2026-11-26.md`.

## Part A. Recommendations for mnemonic-toolkit (`mnemonic`)

The toolkit has the largest surface (26 top-level verbs, 12 sub-verbs) and
most of the constellation's vocabulary drift, because it grew verb by verb over
many cycles. It also owns the manuals every other repo's renames land in, so
the constellation exit-code table (CLI-14) and the glossary (CLI-11) belong here.

**P1, worth doing first.**

| # | item | now | proposed | breaking after the alias goes? |
| --- | --- | --- | --- | --- |
| tk-1 | CLI-03 | `--json-out PATH` (`final-word`, `seed-xor`, `seedqr`, `slip39`, `electrum-decrypt`) "inherits the process umask"; several envelopes carry seed words or shares | write through `write_private` (0600) now; then replace with `--json --out FILE` | permission: no; rename: alias |
| tk-2 | CLI-03 | `--output <OUTPUT>` (`export-wallet`, `restore`, default `-`), file mode not measured here | measure; write through `write_private`; rename to `--out` | alias |
| tk-3 | CLI-02 | `--language` accepts `simplifiedchinese`, `traditionalchinese`, refuses `chinese-simplified`; `--electrum-language` in the same binary accepts `chinese-simplified` | canonical `chinese-simplified`, `chinese-traditional`; fused forms as hidden aliases; standard order | no |
| tk-4 | CLI-01 | `--network main` refused (`decode-address`, `addresses`, …) | accept `main` as a hidden alias | no |

**P2, vocabulary.**

| # | item | now | proposed |
| --- | --- | --- | --- |
| tk-5 | CLI-06 | `--template bip84\|wsh-sortedmulti\|…` (bundle, verify-bundle, convert, restore, export-wallet); `build-descriptor --archetype` | `--scheme` for both (decision 3), `--template` and `--archetype` as aliases; one scheme list shared with `md compose --scheme` |
| tk-6 | CLI-05 | `--origin` (restore, verify-bundle) | `--origin-path` (keep alias) |
| tk-7 | CLI-05 | `--privacy-preserving` | `--no-fingerprints` (keep alias) |
| tk-8 | CLI-05 | `--multisig-path-family bip48\|bip87` | names from the shared named-path table |
| tk-9 | CLI-07 | verb `addresses`; `--script-type` on `convert`, `nostr`; `--search-addr-min` (inclusive) / `--search-addr-max` (exclusive); `--external-only`; `silent-payment --change-address` | verb `address` (keep `addresses`); `--address-type` everywhere; `--search-range A,B` inclusive; `--chain receive`; `--chain` accepts `0`/`1` |
| tk-10 | CLI-08 | `--allow <rule>` (export-wallet, build-descriptor); `restore --allow-mismatch`; deprecated `--accept-search-time` | keep `--allow <rule>`, add one `--allow-<rule>` flag per rule (`--allow-sigless-branch` = `md`'s `--allow-sigless-path`: pick one word for both); remove `--accept-search-time` after its notice |
| tk-11 | CLI-09 | `--to` (convert, combine), `--format` (export-wallet, restore, build-descriptor out; import-wallet, verify-message in), `--md1-form`, `word-card --decode` | `--to` for forms, `--format` for external formats; `--md1-form` → `--to md1-policy\|md1-template` or keep as a documented exception; `word-card encode\|decode` sub-verbs (as `seedqr`) |
| tk-12 | CLI-10 | five source dialects: `--from node=`, `--slot @N.sub=`, `--secret/--secret-file/--secret-stdin` (nostr, silent-payment), `--phrase/--phrase-stdin/--ms1/--ms1-stdin` + positional `[MS1]` (xpub-search), `--share`; deprecated `seedqr decode --digits` | `--from node=value` for single-source verbs (nostr, silent-payment, xpub-search accept it; old flags aliases), `--slot` for multi-slot verbs, `--share` for combine; `-` and `@env:VAR` everywhere; no new `--*-stdin` flags; drop `--digits` after its notice |
| tk-13 | CLI-12 | `ms-shares split --threshold --shares` (no short flags); `ms split -k -n` | add `-k`/`-n` |
| tk-14 | CLI-12 | `derive-child` (BIP-85) | when §5c moves `mk derive`/`ms derive` here: `derive bip85\|xpub\|fingerprint` sub-verbs, `derive-child` as alias |
| tk-15 | CLI-14 | bad flag → 64 | the code Brian picks (decision 1); publish the constellation exit-code table as a manual appendix |
| tk-16 | CLI-11 | manual and help call md1 material "phrase" in places it quotes `md` | adopt the glossary (string, card, plate, phrase) in chapter 40's introduction |

**P3, help.**

- The two global flags (`--no-auto-repair`, `--allow-argv-secret`) print a full
  paragraph under every verb, including version history (`v0.22.0 —`, `v0.22.1
  D18`, `D20`) and the parser mechanism. One line each under the verbs, the full
  text on the root.
- About-lines start lower case (`emit a 3-card engraving bundle …`); every other
  binary uses sentence case.
- Flag help carries cycle and phase tags (`#28 phase 1 —`, `P4 —`, `v0.2
  multisig …`, `SPEC v0.9.0 §1 item 1`, `v0.27.0 —`, `(per I3 fold …)`) and one
  self-contradicting sentence (`--import-json-index`: "optional (and rejected if
  supplied) for single-entry envelopes? — actually SPEC §3.6 leaves …"). Rewrite
  those as plain statements of current behaviour.
- `--network` is required on `bundle`/`verify-bundle` but its help line is
  empty; say why it has no default (CLI-01 policy b).

**Lockstep inside this repo:** every rename here moves `docs/manual` (chapter
41) and `docs/manual-gui` together with mnemonic-gui's `src/schema/mnemonic.rs`
(the GUI schema-mirror rule in this repo's CLAUDE.md). Renames in sibling repos
land in chapters 42 to 46 here; this repo should host the glossary and the
exit-code appendix that Part B proposes.

## Part B. The constellation standard (identical in every repo's copy)

### B0. How this was measured, and what it builds on

Every binary was built from the head of today's open fix PR in its repo, so the
flags those PRs add (for example `md address --allow-invalid-miniscript`) are
included, and each verb's `--help` was dumped recursively. Values quoted below
are what those binaries print or do, run by absolute path with stdin at
`/dev/null`.

| binary | version printed | built from |
| --- | --- | --- |
| `md` | `md 0.21.2` | descriptor-mnemonic PR 76 head `42d87dd` |
| `mk` | `mk 0.18.0-dev` | mnemonic-key PR 43 head `6957f37` |
| `ms` | `ms 0.21.0` | mnemonic-secret PR 48 head `e131854` |
| `mt` | `mt 0.2.0` | mnemonic-transaction PR 32 head `22b80cd` |
| `mnemonic` | `mnemonic 0.107.1` | mnemonic-toolkit PR 227 head `077f79f` |
| `mr`, `mr-sitting` | `0.1.0-alpha.2 (3cabbbf1)` | mnemonic-refugium PR 248 head `3cabbbf` |
| `me` | `me 0.13.0` | mnemonic-engrave PR 68 head `c667773` |
| `mr-gui` (read, not built) | | refugium-wallet PR 229 head `80efcc4` |
| `mnemonic-gui` schema (read) | | mnemonic-gui PR 58 head `f99bbf5` |

**This report builds on `mnemonic-engrave/design/SPEC_constellation_cli_uniformity.md`**
(the 2026-08-26 uniformity spec and its later folds) and does not reopen
anything it settled. That spec owns the IO and safety layer: `--in FILE`, `-`
for stdin, `--out FILE` created 0600 and overwriting (§6b), whitespace-only
separators (§6c), refusing secrets on argv with `--allow-argv-secret` (§6d), the
write gate (§6e), the exit-code table and the D26 repair semantics (§6f), and the
verb rulings in §5b/§5c (the four verbs `encode`/`decode`/`verify`/`inspect` in
every format binary; `split`, `combine`, `compile`, `derive` and `address`
decided to move to the toolkit, "decided, not scheduled"; `repair` stays). Where
this report touches one of those, it says so and only reports drift.

What that spec deliberately left out is what this report covers: **flag names,
value vocabularies, defaults, verb naming beyond the four, help text and usage
conventions**, and the two binaries it did not cover (`mr`, `mr-sitting`) plus
the GUIs that mirror or print command lines.

**Compatibility.** Brian ruled on 2026-11-25 that there are no existing card
holders, so card compatibility is not a constraint. Command-line compatibility
still matters, because scripts, the toolkit installer, the manuals, the
mnemonic-gui schema mirror and refugium-view's printed command lines all type
these flags. So every rename below is proposed the same way:

1. Add the new spelling; keep the old one as a **hidden clap alias** for at
   least one release, and print one `note:` line on stderr when it is used.
2. Move the toolkit manual chapter (`docs/manual/src/40-cli-reference/`), the
   GUI manual where it applies, the mnemonic-gui schema mirror and
   `pinned-upstream.toml`, and any printed command line in refugium-view in the
   same change, as the repos' CLAUDE.md rules already require.
3. Remove the alias in a later release.

So "breaking" in the tables below means **breaking after the alias is removed**.
Nothing here needs a flag day.

**Priority.** P1 = two tools disagree in a way an operator hits while moving
between them (a refusal, or a value one tool prints and the next refuses), or a
file is written with weaker permissions than the constellation's own rule. P2 =
same concept, different names (or same name, different concepts). P3 = help text
and presentation.

### B1. Summary

| id | topic | priority | repos that change |
| --- | --- | --- | --- |
| CLI-01 | One spelling for networks: `mainnet`, `testnet`, `signet`, `regtest` | P1 | refugium (mr, mr-sitting), md, mk, toolkit, ms, refugium-wallet |
| CLI-02 | One spelling for BIP-39 languages (`chinese-simplified`) | P1 | toolkit, mnemonic-gui |
| CLI-03 | Every file a tool writes is created 0600 (`--json-out`, `--output`) | P1 | toolkit |
| CLI-04 | `--separator space` accepted by every encoder | P1 | transaction |
| CLI-05 | Key, fingerprint and path flags: one name per concept | P2 | md, mk, toolkit, refugium, ms |
| CLI-06 | `--template` means a BIP-388 template; named shapes are `--scheme` | P2 | toolkit, ms, refugium, md |
| CLI-07 | Address ranges and chains: one vocabulary | P2 | md, mk, toolkit, refugium |
| CLI-08 | Consent flags start with `--allow-`; attestations with `--confirm-` | P2 | refugium, md, ms, engrave, toolkit |
| CLI-09 | Output form: `--to` for forms, `--format` for external formats, `--out`/`--json` for channels | P2 | toolkit, md, refugium, engrave |
| CLI-10 | Secret and source inputs: one dialect | P2 | toolkit, ms, engrave |
| CLI-11 | An md1 string is never called a "phrase" | P2 | md, toolkit |
| CLI-12 | Verb naming beyond the four core verbs | P2 | toolkit, refugium (mr-sitting), md, mk, ms |
| CLI-13 | Positionals and "another card" flags | P2 | ms, md, mk, refugium |
| CLI-14 | Exit codes: one usage-error code, one published table | P2 | md, mk, ms, mt, me, mr, toolkit |
| CLI-15 | JSON envelopes (the uniformity spec's filed follow-up) | P2 | md, mk, ms, mt, toolkit, refugium |
| CLI-16 | Help text: what it does, not how it was decided | P3 | all |
| CLI-17 | Help structure: short `-h`, metavars, examples, every flag documented | P3 | all |
| CLI-18 | Maintenance verbs: `gen-man` everywhere, `vectors` means one thing, `--version` format | P3 | transaction, engrave, refugium, md, mk, ms |
| CLI-19 | The GUIs: mirror the standard, do not fork it | P2 | mnemonic-gui, refugium-wallet |

### CLI-01. Networks (P1)

**Now.**

| tool | values accepted | default | `main`? | `mainnet`? |
| --- | --- | --- | --- | --- |
| `md` (every verb with `--network`) | mainnet, testnet, signet, regtest | mainnet | **refused** (`invalid value 'main'`) | yes |
| `mk address` | mainnet, testnet, signet, regtest | inferred from the xpub | **refused** | yes |
| `ms derive` | **mainnet, testnet only** | mainnet | refused | yes |
| `mnemonic` (most verbs) | mainnet, testnet, signet, regtest | **required** on `bundle`/`verify-bundle`; inferred on `addresses`; mainnet on `nostr`, `build-descriptor` | **refused** | yes |
| `mr` (global) | main, signet, testnet, regtest | **main** | yes | accepted, not listed |
| `mr-sitting root`/`child` | main, signet, testnet, regtest | **required** | yes | accepted, not listed |
| `mt`, `me` | no flag (the transaction or string carries it) | | | |

`mr` prints `main` in its help, its kit notes and its printed commands, and its
own root help has to warn the reader: *"NETWORK `mainnet`, `testnet`, `signet`
or `regtest` (md's names: md refuses `mr`'s `main`)"*. A value one tool prints
and the next refuses is the P1 case.

**Standard.** Canonical values `mainnet`, `testnet`, `signet`, `regtest`,
listed in that order everywhere. Every tool also accepts `main` as a hidden
alias (non-breaking for `mr` users and scripts). `mr` and `mr-sitting` print
`mainnet` in help, notes and printed command lines. `ms derive` admits `signet`
and `regtest` (coin type 1, as `testnet`), so a signet rehearsal can run end to
end with one value.

**Defaults, written down once.** Keep the three policies that exist, and state
which one each verb uses in its help: (a) inferred from the key, refusing a
disagreement (`mk address`, `mnemonic addresses`, `xpub-search address-of-xpub`);
(b) required with no default where a mistake mints the wrong network's secrets
or a multi-card bundle (`mnemonic bundle`/`verify-bundle`, `mr-sitting root`/`child`);
(c) `mainnet` otherwise. `mr` keeps its default (mainnet) under (c).

**Breaking?** No. Only additions and display changes.

### CLI-02. BIP-39 languages (P1)

**Now.** `ms` and `mr` accept `chinese-simplified`, `chinese-traditional`. The
toolkit's `--language` accepts `simplifiedchinese`, `traditionalchinese` and
refuses the hyphenated form; the toolkit's own `--electrum-language` accepts
`chinese-simplified` (and `zh-hans`, `zh`). Measured:
`mnemonic final-word --language chinese-simplified` and
`ms decode --language simplifiedchinese` both exit with `invalid value`.
mnemonic-gui carries two lists for this reason (`LANG_MS` in `schema/ms.rs`,
with a comment saying the wrong list "silently emits argv rejected by the
binary"). The order of the ten values also differs (the toolkit lists english, the two
Chinese lists, then the rest alphabetically; ms and mr use BIP-39 wordlist order).

**Standard.** `english, chinese-simplified, chinese-traditional, czech, french,
italian, japanese, korean, portuguese, spanish` (english first, then
alphabetical), in every binary. The fused forms stay accepted as hidden aliases
in the toolkit; `ms` and `mr` add them too, so either spelling works anywhere.
mnemonic-gui then needs one list.

**Breaking?** No.

### CLI-03. Files a tool writes are created 0600 (P1)

**Now.** The uniformity spec (§6b) rules that `--out FILE` is created 0600
through `write_private`, because a shell redirect cannot. The toolkit has two
other file channels that predate it:

- `--json-out <PATH>` on `final-word`, `seed-xor split`/`combine`, `seedqr
  encode`/`decode`, `slip39 split`/`combine` and `electrum-decrypt`. Its help
  says: *"On Unix the resulting file inherits the process umask; a
  world-readable result raises a SPEC §2.6 stderr advisory"*. Several of these
  envelopes carry seed words or shares.
- `--output <OUTPUT>` on `export-wallet` and `restore` (default `-`). Its file
  mode was not measured for this report.

**Standard.** Every file a constellation binary writes is created through
`mnemonic-io-lib`'s `write_private` (0600, overwrite, unchanged on a failed
write). Then, for naming: `--out FILE` is the only artifact-to-file flag.
`--output` becomes a hidden alias of `--out`. `--json-out PATH` is replaced by
`--json --out PATH` with the encoders' rule (see CLI-09); until then it at least
writes 0600.

**Breaking?** The permission change is not (a tighter mode). The renames are
alias-protected.

### CLI-04. `--separator space` on every encoder (P1)

**Now.** `md`, `mk`, `ms` and `mnemonic` document `--separator` as "`space`
(keyword) or the literal `" "`", default `space`. `mt encode` documents the
default as the literal `" "` and refuses the keyword: `mt encode --separator
space --group-size 4` prints `REFUSED — §1.1e, --separator "space" is not
whitespace`. An operator who learned the flag on any other encoder is refused on
`mt`.

**Standard.** `mt` accepts the keyword `space` (and keeps the literal), and
shows `[default: space]`. `mt`'s `--group-size` has no default where the others
show `[default: 0]`; same behaviour, so show it the same way.

**Breaking?** No.

### CLI-05. Keys, fingerprints and paths (P2)

**Now.** Five spellings for the three facts a key record carries:

| concept | `md` | `mk` | `mnemonic` | `mr` | `ms` |
| --- | --- | --- | --- | --- | --- |
| one key | `--key @i=XPUB` (or `@i=[fp/path]xpub`), repeatable | `--xpub XPUB` | `--slot @N.xpub=`, `--key` (build-descriptor), `--cosigner` | `--keys @j=KEY …` (multi-value) | |
| a file of key records | `--from-mk1-file` (mk1 strings) | **`--keys FILE`** (and `--in FILE`, same reader) | | `--keys-file FILE` | |
| master fingerprint | `--fingerprint @i=HEX` | `--origin-fingerprint HEX` | `--fingerprint`, `--slot @N.fingerprint=`, `--expect-fingerprint` | `--hot-parent-fp` | |
| origin path (master to key) | `--path PATH` (named, hex or `m/…`) | `--origin-path PATH` | `--origin` (restore, verify-bundle), `--slot @N.path=` | (in records) | `--template bip84` etc. (named only) |
| path relative to the given key | | `mk derive --path m/0/5` | `convert --path` (from the master) | | |
| omit fingerprints | | `--privacy-preserving` | `--privacy-preserving` | `--no-fingerprints` | |

The two clashes that matter: **`--keys` takes a FILE in `mk` and values in
`mr`**, and **`--path` means the origin path in `md` but a relative derivation
in `mk derive`**.

**Standard.**

- `--key VALUE` is one key: an xpub or an origin-notated record
  `[fingerprint/path]xpub`, optionally bound `@i=`. Repeatable, and one
  occurrence may take several values (as `md --from-mk1` already does).
  `--key-file FILE` reads records one per line. `mr --keys`/`--keys-file` and
  `mk --keys FILE` become hidden aliases. (`mk encode --in FILE` already reads
  the same records; `mk --keys` can simply be retired in favour of it.)
- `--fingerprint` is the master fingerprint, everywhere (`@i=HEX` where a slot
  is meant). `mk --origin-fingerprint` and `mr --hot-parent-fp` become aliases of
  `--fingerprint` and `--hot-parent-fingerprint`.
- `--origin-path` is the path from the master to the key. `--path` is a
  derivation **relative to whatever key the input gives** (for a seed input that
  is the master, so `mnemonic convert --path` is already right, and `mk derive
  --path` is right). So `md --path` (encode, verify, descriptor, address) and the
  toolkit's `--origin` move to `--origin-path`, with the old names as aliases.
- One named-path table, shared: `md --path` accepts `bip44|48|49|84|86`; `ms
  derive --template` accepts ten names including `bip48-p2wsh`, `bip48-p2tr`,
  `bg002h-tr`, `bg002h-wsh`; the toolkit has `--multisig-path-family bip48|bip87`.
  Put one table (names, script type, `coin'`/`account'` placement) in one crate
  (md-codec or `mnemonic-io-lib`) and accept the same names wherever an origin
  path is typed.
- Omitting fingerprints: one name. `--no-fingerprints` says what it does;
  `--privacy-preserving` stays as an alias in `mk` and the toolkit.

**Breaking?** Not with aliases. These touch the most documentation (manual
chapters 41, 42, 44, 45; mnemonic-gui `md.rs`, `mk.rs`, `mnemonic.rs`;
refugium-view and `mr`'s own printed recipes, which type `mk encode --xpub
--origin-fingerprint --origin-path`).

### CLI-06. Templates and presets (P2)

**Now.** `--template` has three meanings:

- a BIP-388 template string: `md verify|descriptor|address --template
  'wsh(…)'`; `mr` takes it as a positional `[template]`;
- a named wallet shape: `mnemonic bundle|verify-bundle|convert|restore
  --template bip84|wsh-sortedmulti|…`;
- a named account path: `ms derive --template bip84|bip48-p2wsh|bg002h-tr|…`.

Named shapes are also spelled `mr create --template-preset owner-heir|…`, `md
compose --preset kofn-recovery,2of3,…` and `mnemonic build-descriptor
--archetype kofn-recovery|decaying-multisig|…` (the composer's and the toolkit's
archetype names overlap). `mr --preset none|classic` is a *slot table*, not a
shape.

**Standard.** `--template` is only ever BIP-388 template text. A named shape is
`--scheme NAME` (toolkit `--template` and `--archetype`, `mr --template-preset`,
`md compose --preset`; decided 2026-11-26).
`mr`'s slot-table preset becomes `--slot-preset`. `ms derive --template`
becomes `--origin-path NAME` with the shared named-path table (CLI-05). One
scheme list for `md compose --scheme` and `mnemonic build-descriptor`, or
help text saying why they differ.

**Breaking?** Not with aliases, but the toolkit's `--template` is the most-typed
flag in the toolkit and in the GUI form; keep its alias for longer than one
release.

### CLI-07. Addresses, chains and ranges (P2)

**Now.**

| tool | chain | start / count | range | type |
| --- | --- | --- | --- | --- |
| `md address` | `--chain 0\|1` (integer; `receive` refused), `--change` | `--index` (default 0), `--count` (default 1) | | from the policy |
| `md descriptor` | `--chain 0\|1`, `--change` | | | |
| `mk address` | `--chain receive\|change\|both` (`1` refused) | `--count` | `--range A,B` | `--address-type` |
| `mnemonic addresses` | `--chain receive\|change\|both` | `--count` (default 10) | `--range A,B` | `--address-type` |
| `mnemonic convert`, `nostr` | | | | `--script-type` (same values) |
| `mnemonic verify-bundle`, `restore` | `--search-chain receive\|change\|both` | | `--search-addr-min` (inclusive) / `--search-addr-max` (**exclusive**) | |
| `mnemonic xpub-search address-of-xpub` | `--external-only` | `--gap-limit` | | `--address-type` |
| `mr address` | `--change` | `--index` (required) | | |
| `mr upgrade` | `--change` | `--index` | | |
| `mnemonic silent-payment` | `--change-address` | | | |

**Standard.** `--chain receive|change|both`, also accepting `0`/`1`; `--change`
is sugar for `--chain change` wherever a chain is chosen. `--index N` is the
first index (default 0) and `--count N` how many; `--range A,B` (inclusive) is
the alternative to both. One default count for the listing verbs (pick 10 or 1;
today `md` prints 1 and the toolkit 10). `--address-type` everywhere
(`--script-type` an alias). Search ranges reuse the same words:
`--search-range A,B` inclusive replaces the min/exclusive-max pair, whose
exclusive upper bound is unlike every other range here; `--external-only`
becomes `--chain receive`.

The verb: `md address`, `mk address`, `mr address` are singular; the toolkit's is
`addresses`. Since §5c moves address derivation to the toolkit, name the
toolkit's verb `address` (keep `addresses` as an alias) before the move, so the
moved verbs keep the name operators already type.

### CLI-08. Consent and attestation flags (P2)

**Now.** At least five shapes for "I know, do it anyway":

- `--allow-*`: `--allow-argv-secret` (all), `--allow-world-readable` (`mt`,
  `me`), `--allow-invalid-miniscript` (`md`), `--allow-unsigned-inputs`,
  `--allow-weak` (`me`; "accepted and ignored"), `--allow-mismatch` (toolkit
  `restore`), and the toolkit's value form `--allow <rule>` (`export-wallet`,
  `build-descriptor`: `malleable`, `mixed-timelock`, `repeated-keys`,
  `resource-limit`, `sigless-branch`);
- `--accept-*`: `mr` (`--accept-deadline`, `--accept-tip`,
  `--accept-stub-mismatch`, `--accept-seed-language`, `--accept-12-word`,
  `--accept-clamped`, `--accept-unconfirmed-seeds`, …) and the toolkit's
  deprecated `--accept-search-time`;
- bare words: `mr` `--small-delta`, `--same-seed`, `--lockup-slot`,
  `--lockup-no-window`, `--reorder-slot`, `--far-slot`, `--open-slot`,
  `--epochs`, `--beyond-horizon`, `--hot-cold-seed`, `--heir-holds-hot-path`;
  `md --experimental`, `md compose --md-only`;
- a suffix: `--phrase-looks-like-digest-ok` (`ms hashlock`, `me sysw pack`);
- attestations the tool cannot check: `mr --root-first-card`,
  `--first-hot-child`, `--no-other-addenda`, `--no-other-retired-cards`,
  `--no-other-cards-on-these-paths`.

`md --experimental` and the toolkit's `--allow sigless-branch` relax the same
rule (a spend path with no signature) under two names.

**Standard.** One greppable prefix per kind, which is the argument the
constellation already makes for `--allow-argv-secret` ("Greppable, so a reviewer
can find every place a script opted in"):

- `--allow-<what>` relaxes a refusal the tool would otherwise make. `mr`'s
  `--accept-*` and bare consent flags move to `--allow-*` (for example
  `--allow-small-delta`, `--allow-lockup-slot`, `--allow-deadline`). `md
  --experimental` becomes `--allow-sigless-path` (one name with the toolkit's
  rule), `--md-only` becomes `--allow-no-coordinator`, and
  `--phrase-looks-like-digest-ok` becomes `--allow-digest-like-phrase`. The
  toolkit's `--allow <rule>` stays as an accepted form and also gets one
  `--allow-<rule>` flag per rule, so a GUI can show checkboxes.
- `--confirm-<fact>` states something the tool cannot check (`mr
  --confirm-root-first-card`, `--confirm-no-other-addenda`, …), so a reader can
  tell an attestation from a consent.
- Flags that select behaviour rather than consent keep plain names (`mr
  --open-slot` on `member`, where it means "hand out an open member", is a
  selection; on `create` it is a consent and becomes `--allow-open-slot`).
- `--allow-weak` ("accepted and ignored") is removed after its notice period.

`mr` is at `0.1.0-alpha.2`, so this is the cheapest moment to rename its flags.

### CLI-09. Output form and output channel (P2)

**Now.** Choosing *what* to print: `mnemonic convert --to`, `ms combine --to`,
`mnemonic slip39|ms-shares combine --to`, `md descriptor --emit md1`, `md
decompose --emit template|keys|…`, `mr member --as descriptor|md1|policy`, `me
sysw pack --as descriptor|md1`, `mnemonic bundle --md1-form policy|template`,
`mt encode --qr`, `mnemonic word-card --decode`, and `--format` on
`export-wallet`, `restore`, `build-descriptor` (output) and `import-wallet`,
`verify-message` (input). Choosing *where*: `--out` (encoders, CLI-03),
`--output` and `--json-out` (toolkit), `--manifest FILE` (`me bundle`), and the
`me` converter's `--stdout`, `--hex`, `--base64`.

**Standard.**

- `--to <form>` chooses which constellation or key form to print (`convert`,
  `combine`, `md descriptor`/`decompose`, `mr member`). `--emit` and `--as`
  become aliases.
- `--format <name>` names an external interchange format (`bitcoin-core`,
  `bsms`, `sparrow`, …), for input or output.
- `--out FILE` is the only artifact-to-file flag; `--json` switches stdout (and
  `--out`, per the encoders' §6b rule) to JSON. `--json-out` and `--output`
  retire (CLI-03). `me bundle --manifest FILE` becomes `--out FILE` (the manifest
  is that verb's artifact). The `me` converter's `--stdout`/`--hex`/`--base64`
  become `--to raw|hex|base64` with `--out`.

### CLI-10. Secret and source inputs (P2)

**Now.** The uniformity spec settled that secrets never arrive on argv
(`--allow-argv-secret` to override) and that `-` means stdin. What still differs
is how a tool is told which source it is reading. The toolkit alone has five
dialects:

- `--from <node>=<value>`: `convert`, `addresses`, `derive-child`, `final-word`,
  `restore`, `seedqr`, `slip39 split`, `ms-shares split`, `verify-bundle`;
- `--slot @N.<subkey>=<value>`: `bundle`, `export-wallet`, `import-wallet`;
- `--secret` / `--secret-file` / `--secret-stdin`: `nostr`, `silent-payment`;
- `--phrase` / `--phrase-stdin` / `--ms1` / `--ms1-stdin` plus a positional
  `[MS1]…`: `xpub-search`;
- `--share` (repeatable): `seed-xor combine`, `slip39 combine`, `ms-shares
  combine`; and the deprecated `seedqr decode --digits`.

Stdin is requested four ways across the constellation: a value of `-`, a
boolean `--<x>-stdin`, `@env:VAR`, and `--<x>-file PATH`. `ms` already documents
`--passphrase -` as "same as `--passphrase-stdin`".

`--passphrase` itself means the BIP-39 passphrase in `ms`, the toolkit and `mr`,
but `me sysw pack --passphrase-words`, `--passphrase-ask` and `--no-passphrase`
are the sealed container's passphrase, a different secret.

`--hex` means entropy input in `ms encode`/`split`, a preimage in `ms hashlock`,
and an output encoding in the `me` converter; the toolkit calls entropy
`entropy=`.

**Standard.**

- A seed or key source is `--from <node>=<value>` (the toolkit's majority form,
  with the node names `convert` lists). `nostr`, `silent-payment` and
  `xpub-search` accept it; their current flags become aliases. `--slot @N.…`
  stays for multi-slot verbs (it is the same grammar with a slot index).
- Value conventions, one set: `-` reads stdin, `@env:VAR` reads the
  environment, and a file is `--in FILE` (the tool's own input) or
  `--<x>-file FILE` (a second input). Boolean `--<x>-stdin` flags stay as
  aliases of `--<x> -`, and new verbs do not add them.
- `--passphrase` is reserved for the BIP-39 passphrase. Other secrets carry a
  qualified name, as the toolkit already does with `--bip38-passphrase` and
  `--decrypt-password`: in `me`, `--seal-passphrase-words`,
  `--seal-passphrase-ask`, `--no-seal-passphrase`.
- `--entropy HEX` for entropy input (`ms encode`/`split`, matching the toolkit's
  `entropy=` node), `--preimage HEX` in `ms hashlock`; `ms --hex` stays as an
  alias.

### CLI-11. An md1 string is never a "phrase" (P2)

**Now.** `md` calls its strings "phrases": the positional is `[PHRASES]` on
`md descriptor`, `md address` and `md shape-key`, the help says "One or more md1
phrases", and `md encode --json` puts the artifact under the key `"phrase"`.
Everywhere else in the constellation, and in BIP-39, a phrase is seed words: `ms
--phrase`, the toolkit's `phrase=` node, `--phrase-stdin`. A tool that handles
both (the toolkit, a GUI, an operator's notes) has to remember that one
"phrase" is public and the other spends everything.

**Standard.** A glossary used in every help text and manual chapter:

- **string**: one `md1…`/`mk1…`/`ms1…`/`mt1…`/`mr1…` string (one chunk);
- **card**: all strings of one encoding (a chunk set, or one string);
- **plate**: the engraved object;
- **phrase**: BIP-39 (or SLIP-39, Electrum) words only.

`md`'s positionals become `[MD1]...` (as `mk`'s are `[MK1_STRINGS]`, which
should become `[MK1]...` for the same pattern), and the JSON key moves with the
JSON work in CLI-15.

### CLI-12. Verb names beyond the core four (P2)

The core four are uniform (§5b). Outside them:

- **Singular or plural.** `address` (md, mk, mr) versus `addresses` (toolkit):
  see CLI-07.
- **`derive`.** `mk derive` (a child xpub), `ms derive` (fingerprint and account
  xpub), `mnemonic derive-child` (BIP-85), `mr child` and `mr-sitting child`
  (a root's children). §5c already moves `derive` to the toolkit. When it moves,
  name by output: `derive xpub`, `derive fingerprint`, `derive bip85`, so the
  three become one verb with sub-verbs. `mr child` is a different operation
  (Refugium roots) and keeps its name.
- **Nesting.** The constellation already nests when one noun has several
  operations: `me sysw pack|wipe|show`, `mnemonic seed-xor|slip39|ms-shares
  split|combine`, `mnemonic seedqr encode|decode`, `mnemonic xpub-search …`.
  Apply that rule everywhere: `mr-sitting kit-write`/`kit-wipe` become `kit
  write`/`kit wipe`; `mnemonic word-card --decode` becomes `word-card
  encode|decode` (as `seedqr` is).
- **Flat hyphenated verbs** are verb-first: `verify-bundle`, `decode-address`,
  `export-wallet`, `build-descriptor`, `erase-sealed`. Keep that order for new
  ones (`mr-sitting erase-sealed` already does; `kit-write` did not).
- **Sharing.** `ms split`/`combine` and `mnemonic ms-shares split`/`combine` do
  the same job with different flags: `ms` has `-k/--threshold`, `-n/--shares`,
  `--phrase|--hex|--in`; the toolkit has `--threshold`, `--shares` (no short
  forms) and `--from phrase=|entropy=`. Until §5c's move, give them the same
  flags (add `-k`/`-n` to the toolkit; `ms` accepts `--from`).

### CLI-13. Positionals and "another card" flags (P2)

**Now.**

- `ms repair` takes the string only as `--ms1 <MS1>` (or `--in`); `md repair`,
  `mk repair`, `mnemonic repair` and every other `ms` reading verb take a
  positional.
- A second card is passed as `--from-mk1 STRING…` (md, multi-value),
  `--from-md1 STRING` (mk, one string per occurrence), `--md1`/`--mk1`/`--ms1`
  (toolkit), `--card <mr1>` (mr, one card per value with chunks joined by
  spaces), `--against`, `--witness-md1`, `--sibling`, `--bundle-card` (mr).
- The file forms are `--from-mk1-file` (md), `--from-md1-set` (mk) and
  `--keys-file` (mr).

**Standard.** Every reading verb takes its own strings as positionals, plus
`--in FILE` and `-`. `ms repair` gains the positional (`--ms1` stays as an
alias). Flags naming a card of another format are `--from-<hrp>` and take
several values per occurrence (md's form), with `--from-<hrp>-file FILE` for a
file: `mk --from-md1-set` becomes `--from-md1-file`. The toolkit's
`--md1`/`--mk1`/`--ms1` on `verify-bundle`, `inspect` and `repair` stay as they
are (in those verbs the cards *are* the input, not a second card). `mr`'s chain
flags (`--card`, `--sibling`, `--bundle-card`) carry Refugium meanings and keep
their names; they should accept a chunked card as several values, like
`--from-mk1`, rather than one space-joined value.

### CLI-14. Exit codes (P2)

**Now.** The uniformity spec's §6f table still holds for the encoders, and `mk`'s
invalid-artifact code is now 1, as that spec required. Two gaps remain:

- **Usage errors** (a bad flag): `md`, `mt`, `me`, `mr`, `mr-sitting` and
  `mr-gui` exit **2**; `mk`, `ms` and `mnemonic` exit **64**. Measured with
  `--bogus-flag` on each binary.
- **No input** differs too: `md decode -` on empty stdin exits 2, `mk decode -`
  64, `mt decode -` 1.

The "answer is no" codes are already semantic and documented per verb (`ms
verify` 4 on a phrase mismatch, `verify-bundle` 4 on mismatch, `md descriptor
--verify-against` 5 when not spend-equal, D26 for `repair`), but there is no one
table an operator can read.

**Standard.** One usage-error code for every binary. Five binaries use 2 (clap's
own default) and three use 64; changing three is less churn. The decision is
Brian's (see "Decisions for Brian" at the top). Empty input is a usage error, with that code.
Publish one constellation exit-code table in the toolkit manual (a new appendix
beside chapter 40), generated or checked from each binary's own help, and point
every verb's help at it.

**Breaking?** Yes for scripts that test the old usage code. Usage errors are rare
in scripts, so the cost is small, but it needs the manual and GUI to move
together (mnemonic-gui's schema help text quotes "exit 64" in several places).

### CLI-15. JSON envelopes (P2)

**Now** (the uniformity spec filed this as a follow-up in §6b; still true):
`md encode --json` gives `{"network", "phrase", "schema": "md-cli/2"}`,
pretty-printed; `ms encode --json` gives `{"schema_version": "1", "ms1", …}`,
compact; `mk` uses `mk1_strings` and a numeric `schema_version`. `repair --json`
is already one shape across `md`, `mk`, `ms` and `mnemonic` (`RepairJson`), so
the constellation already knows how to do this. `mr` has no `--json` on any
verb; `mt` has `--json` reports; `me bundle` writes a manifest.

**Standard.** For a new major schema version on each tool: `schema_version` as
an integer, the artifact under its HRP (`"md1"`, `"mk1"`, `"ms1"`, `"mt1"`,
always an array of strings, one per chunk), `network` where the tool knows it,
compact output (`jq` pretty-prints). `mr` gains `--json` on its read verbs
(`decode`, `verify`, `member`, `address`, `export`) before refugium-wallet
starts parsing its text. This is a contract change for the GUIs (mnemonic-gui's
CLAUDE.md calls JSON shapes a contract), so bump each schema version and move
the consumers in the same change.

### CLI-16. Help text says what a flag does (P3)

**Now.** A great deal of help text records how a decision was reached rather
than what the flag does: section numbers and finding ids (`SPEC §6b, F-244`,
`(R1 I-1 fold; bug hunt 2026-11-25 G1)`, `(fold I, spec §7 create)`, `P3 §6b —`),
version history (`v0.22.0 — skip auto-fire …`, `#28 phase 1 —`, `P4 —`,
`DEPRECATED (v0.31.6)`), implementation notes (`Wraps
md_codec::decode_with_correction`, `` `requires = "emit"` refuses it
structurally ``, `apply_root_options enforces …`), and arguments with past
reviewers (`do not "tidy" them together`, `that is the design rather than an
oversight`). The toolkit's two global flags print a full paragraph under every
one of its verbs.

**Standard.**

- The first line says what the flag does; the long help adds what an operator
  needs to use it safely (defaults, interactions, the hazard). Rationale,
  rulings and ids go in code comments, the spec or the manual. A help text may
  keep one short "why" when it changes what the operator should do (for example
  `mt`'s "a raw transaction is a BEARER instrument").
- Global flags get a one-line short help and are described in full once, on the
  root command.
- About-lines in sentence case, no trailing period (the toolkit's are lowercase).

### CLI-17. Help structure (P3)

- **`-h` is short, `--help` is long.** `mr` attaches about 1,480 lines of
  procedure (the first sitting, restores, the journal) to its root help, and
  `mr -h` prints all of it. Those pages already exist as `mr procedure <name>`;
  the root help should list the verbs, the global flags and one line pointing at
  `mr procedure`.
- **Metavars**: upper case, `FILE` for files, `DIR` for directories, the HRP for
  card strings (`MD1`, `MK1`, `MR1`). Today: `<IN_PATH>`, `<OUT>` (me seal),
  `<mr1>`, `<mr1 strings>`, `<md1 card>`, `[template|mr1]` (mr), `<PATH>` for a
  file in `mt`, `<STRING>` in the toolkit.
- **Every flag and positional has help.** Missing today: `md verify --template`,
  `--key`, `--fingerprint`; `md bytecode --json`; `md vectors --out`; `md
  compile --context`, `--json`, `<EXPR>`; `me sysw wipe --out`; `me sysw show
  <FILE>`. A test over `gui-schema` (or clap's command tree) can enforce this in
  each repo.
- **EXAMPLES** on every user verb. `ms` has them on most verbs; `md` on some; `mk`,
  `mt`, `mr` and most toolkit verbs on none.
- **Root-level options for a default action**: `me` with no subcommand is the
  NFC converter, and its flags (`--in`, `--out`, `--stdout`, `--hex`, …) are root
  options. Every other binary requires a verb. Give the converter a verb (`me
  ndef`) and keep the bare form as an alias.

### CLI-18. Maintenance verbs and `--version` (P3)

- **`gen-man`** exists on `md`, `mk`, `ms` and `mnemonic`, not on `mt`, `me`,
  `mr` or `mr-sitting`. Add it to all, so the installer can drop man pages for
  every binary it installs.
- **`gui-schema`** is needed only where a GUI mirrors the flags (`md`, `mk`, `ms`,
  `mnemonic`). If mr-gui or mnemonic-gui ever drives `mt`, `me` or `mr`, add it
  then.
- **`vectors`** means three things: `md vectors --out DIR` regenerates the corpus
  (maintainer tool), `mk vectors` prints JSON (`--pretty`, `--out DIR` per
  fixture), `ms vectors` prints JSON (`--pretty`, no `--out`). Make it "print the
  corpus as JSON on stdout; `--out DIR` writes one file per fixture; `--pretty`",
  and give `md`'s regenerate mode its own flag (`--regenerate`).
- **`--version`**: `mr`, `mr-sitting` and `mr-gui` print the commit (`mr
  0.1.0-alpha.2 (3cabbbf1)`); the others print the version only. Printing the
  commit everywhere helps support. mnemonic-gui's version gate compares the
  literal `--version` string, so change the parser there first.

### CLI-19. The GUIs (P2)

- **mnemonic-gui** mirrors `md`, `mk`, `ms` and `mnemonic` by hand. Each rename
  above is a schema-mirror change there, and the drift gate checks flag names,
  not values (it cannot see CLI-02's language split or CLI-04's separator). When
  a CLI adds an alias, the mirror should use the canonical name and nothing else.
  Once CLI-01/02 land, it can drop `LANG_MS` and one of its `NETWORKS` lists.
- **refugium-wallet** (`mr-gui`, `refugium-view`) prints command lines for people
  to type: `mnemonic final-word --from phrase=-`, `mnemonic convert --from
  phrase=- --to fingerprint,xpub --path …`, `mr-sitting sit|verify --plan
  --hand-back`, `--seat-check …`. Those strings are a CLI surface too and move in
  lockstep with any rename here. `mr-gui`'s own arguments are parsed by hand:
  it has no `-h`/`--help`, no `-V`, and `--screenshot DIR [SCALE]` takes an
  optional second positional.

### B2. Inventory (verbs per binary)

| binary | verbs |
| --- | --- |
| `md` | encode, decode, verify, inspect, bytecode, vectors, compile, compose, shape-key, descriptor, address, decompose, gui-schema, repair, gen-man |
| `mk` | encode, decode, inspect, verify, vectors, gui-schema, repair, address, derive, gen-man |
| `ms` | derive, encode, decode, hashlock, inspect, verify, vectors, gui-schema, gen-man, repair, split, combine |
| `mt` | encode, decode, verify, inspect |
| `mnemonic` | bundle, verify-bundle, convert, addresses, decode-address, export-wallet, import-wallet, derive-child, electrum-decrypt, final-word, seed-xor {split, combine}, seedqr {encode, decode}, nostr, silent-payment, slip39 {split, combine}, ms-shares {split, combine}, gen-man, gui-schema, repair, inspect, compare-cost, xpub-search {path-of-xpub, account-of-descriptor, address-of-xpub, passphrase-of-xpub}, verify-message, restore, build-descriptor, word-card |
| `mr` | create, upgrade, verify, decode, repair, member, address, current, units, export, root, child, lost-or-stolen, procedure |
| `mr-sitting` | sit, verify, root, child, practice, glossary, kit-write, kit-wipe, erase-sealed |
| `me` | (none: NFC converter), bundle, sysw {pack, wipe, show}, seal, hash |
| `mr-gui` | flags only: `--software-render`, `--diagnostic`, `--check-libs`, `--screenshot DIR [SCALE]`, `--version` |

Global options: `mnemonic` (`--no-auto-repair`, `--allow-argv-secret`), `mr`
(`--network`), `me` (the converter's options). None elsewhere.

Short options in use: `-h`, `-V` everywhere clap is used, and `ms split -k
--threshold`, `-n --shares`. Nothing else, which is a good property to keep:
long flags only, so scripts read the same everywhere.
