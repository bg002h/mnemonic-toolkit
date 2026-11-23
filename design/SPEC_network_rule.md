# SPEC: one network-contradiction rule for every tool

Status: spec, r2 (author: Claude Opus 5.5, claude-opus-5-5, 2026-11-23; r1 folds
review r1 slices s1 and s2, fold by Claude Opus 5.5, claude-opus-5-5; r2 folds
the merged round-2 table, coordinator `agent-reports/merge-network-spec-r2.md`
rows M01-M55, fold by Claude Opus 5.5, claude-opus-5-5). Brief:
coordinator `briefs/spec-network-rule.md`. Every decision Brian ruled on
2026-11-23 is binding here (coordinator `memory/network-mismatch-rule.md`);
§9 lists each ruling with the text it changes. Source study: coordinator
`agent-reports/design-network-mismatch-fable.md` (Claude Fable 5.2, folded
with its review at a508afcd8 and 76beb9bde). Where this spec and the study
differ, this spec wins (the study's exit code 6 is 8 here, by ruling).

Machine checks: `scripts/spec-network-rule-gate.sh` re-runs every RED row's
"today" leg against the installed binaries, checks every cite, counts the
text ids and validates the vector table (§7). What it does not cover is in
§13.

Revs this spec reads (heads after `git fetch`, 2026-11-23): mnemonic-toolkit
4553d2da; mnemonic-refugium 34300ff27; descriptor-mnemonic 60956b858;
mnemonic-key af12fea85; mnemonic-secret 601d56850; mnemonic-transaction
88ac6d9e0; mnemonic-engrave 0bd8dfbab; mnemonic-gui 922a0b383;
refugium-wallet b58091825; seedhammer 65717c551. Installed binaries the RED
rows ran on: mnemonic 0.107.1, mr and mr-sitting 0.1.0-alpha.2 (0ce60071),
md 0.20.7, mk 0.16.0, ms 0.20.4, mt 0.2.0, me 0.13.0.

Cites use the form {{repo@rev:path:LINE}} "quote"; the gate checks that the
line holds the quote.

## 1. Scope and terms

The rule binds every CLI of the constellation (`mnemonic`, `md`, `mk`, `ms`,
`mr`, `mr-sitting`, `me`, `mt`), the two GUIs (mnemonic-gui; mr-gui, the
refugium-wallet app) and the SeedHammer firmware.

- **Run network**: the network a run produces output for.
- **Class**: `main` or `test`. Signet, testnet (testnet3 and testnet4) and
  regtest are the test class.
- **Source**: anything in a run's input that says which network it is for
  (§2 N1).
- **Kept artifact**: a card (md1, mk1, ms1, mt1, mr1), a plate, a descriptor
  or key string, an exported wallet file, a hand-back, plan or closing file:
  something the holder kept and now reads. A seed phrase being typed to make
  something new is not a kept artifact. A seed in any form typed now (a
  phrase, an ms1 string, entropy, a SeedQR) is not a kept artifact: it is
  the seed case of N6 (no TX-N). A key with no network (a nostr npub or
  nsec) is a kept artifact: N6 prints TX-N or TX-T for it.
- **Prefix**: each tool's own refusal prefix, as its refusals print today:
  `mnemonic` `error:`, `md` `md:`, `mk` `error:`, `mr` `mr:`, `mr-sitting`
  `mr-sitting:`, `me` `me:`. Warnings start `warning:` in every tool.
- **Network names in texts**: `mr` and `mr-sitting` write `main`, `signet`,
  `testnet`, `regtest`; every other tool writes `mainnet`, `signet`,
  `testnet`, `regtest`. A class is written "mainnet" or "test-network" in
  every tool ("a mainnet key", "a test-network key").

## 2. The rule (normative)

N1 Sources.
- HARD: an extended key's version bytes (xpub/tpub and the SLIP-132
  variants ypub/zpub/Ypub/Zpub = main, upub/vpub/Upub/Vpub = test); an
  address's bech32/bech32m HRP or base58 version byte; a WIF version byte; the
  chain a Bitcoin Core node reports; a network an artifact states in its own
  bytes (mk1's key version; a toolkit import envelope's `bundle.network`; an
  mr hand-back, plan or closing file; a Coldcard export's `chain`; a UR
  crypto-hdkey's use-info network; a GUI profile or saved draft).
- SOFT: the coin type of an origin path, read only from a path of the shape
  `purpose'/coin'/account'...` with purpose in {44, 48, 49, 84, 86, 87, 352,
  270028} and coin in {0, 1}. Every other path (BIP-45 `m/45'/...`, BIP-85
  `m/83696968'/...`, Core's `m/0'/0'/k'`, a bare `m/0'`, `m`, any other
  purpose, any other coin number) is no source: no check, no warning.
- Not a source: a card that states no network (md1, ms1, mt1, a
  path-convention mr1 card; their bytes name none), a nostr key, a seed.

N2 Run network. A verb with `--network` runs on the flag's network if
given, else on mainnet; both count as an assertion (D1). A test-class
source with no `--network` therefore contradicts the run (HARD or SOFT as
the source is) and is refused; no tool infers a test network from a key or
a file. A verb with no `--network` (every `me` and `mt` verb, `mk encode`, `mk derive`, the SeedHammer) has no
assertion: its HARD sources decide the run network; with no HARD source,
coin types that all agree decide the class (coin 1 = test, shown as
"TEST NETWORK" where the tool shows a network); coin types that disagree
with each other are a SOFT contradiction (N4) whose overridden output is
mainnet; with no source at all, N6.

N3 Grain. Two sources compare at the finest grain both can express:
- class (main / test): version bytes, WIF, coin type, a base58 address
  (versions 0 and 5 main; 111 and 196 every test chain, regtest included),
  a Coldcard `chain` (BTC = main; XTN and XRT = test) and a UR use-info
  network (0 = main, 1 = test);
- HRP (bc / tb / bcrt): a bech32 or bech32m address (bc = main, tb = signet,
  testnet3 and testnet4, bcrt = regtest);
- exact name (main, test, testnet4, signet, regtest): Core's chain and a
  stated network that can name every chain (toolkit import envelope; mr
  hand-back, plan or closing file; GUI profile or draft). `--network
  testnet` equals both `test` and `testnet4`.
Within the test class, `--network` picks the chain; a class-grain source
never contradicts signet, testnet or regtest.

N4 Contradictions. A HARD contradiction is any two of {run network, HARD
sources} that differ at their common grain. A SOFT contradiction is a coin
type that differs from the run network or from a HARD source, or (no-flag
tools only) from another coin type.

N5 HARD. Refuse: exit 8, nothing on stdout, no file written or changed. No
override in any tool or GUI. `--accept-coin-type-mismatch` has no effect on
it. mk in --json mode keeps its error envelope on stdout for an exit-8 refusal (kind NetworkMismatch, exit_code 8): that envelope is the one stdout output a refusal may have; the prose line and the machine line go to stderr as in every tool.

N6 No source at all, on a run that reads a kept artifact (or a key with no
network, such as nostr `--pubkey`) and prints something that depends on
the network: with no `--network`, print TX-N (mt: TX-N-MT) once on stderr;
with a test `--network`, print TX-T once on stderr. Exit unchanged. A run
that only derives from a seed typed now (`bundle`, `restore --from
phrase=|ms1=|entropy=|seedqr=`, `ms derive`, `convert --from
phrase=|ms1=|entropy=|seedqr=`, mr `root`/`child`) prints neither (D5).

N7 SOFT. Refuse by default: exit 8, nothing on stdout, nothing written.
With `--accept-coin-type-mismatch` the run goes through: exit 0, output for
the network N2 and the HARD sources give, every path kept as written (never
rewritten), and TX-S-OK printed on stderr once per overridden contradiction.
The flag exists, with that exact name and no value, on every verb that can
meet a SOFT contradiction (§4 "flag" column) and is accepted there with no
effect when there is none (no line printed).

N8 Several at once. Every contradiction is printed, one text line each, in
this order: HARD before SOFT; among HARD, TX-H4C, TX-H4, TX-H4P, TX-H2,
TX-H1, TX-H3, TX-H5; within one text id, by input order (key @0 before
@1). When any HARD contradiction is printed, every SOFT line drops its last
sentence (the one naming `--accept-coin-type-mismatch`), so the flag is
named only when every contradiction is SOFT. When TX-H2 prints, every TX-H1
line of the same run drops its first remedy sentence (the one that says
`rerun with --network ...`) and keeps `Nothing was written.` and its
`export the key again` sentence; no `--network` can pass keys that disagree
with each other (R1.14, R3.8, R7.3). Sources of one class that agree with
each other and disagree with the run are one contradiction, printed once
("keys @0 and @1 are test-network keys (tpub)"; the plural forms are in
§3). When an artifact's stated
network and its keys agree with each other and disagree with the run, only
TX-H4 prints. A coin type that agrees with the version of its own key (the key whose origin path carries it), when that key is named on a HARD line, is not a separate line: only the HARD lines print (V14, R1.14, R6.3). A coin type that disagrees with its own key's version prints TX-S after the HARD lines (V15, R3.6). A coin type that disagrees with both its own key's version and the run (the key agrees with the run) is one contradiction and prints one TX-S in its key-version form, whose machine-line b is that key; the run form is only for a coin type with no key version of its own (V09, V11, R8c.1, R8c.4). When a file contradicts itself, only TX-H4C prints for it: its stated network and its keys are not also compared with the run (V30, V31).

N9 Exit code 8 in every CLI for every network refusal, HARD or SOFT,
including the existing refusals listed in §4 (they keep their meaning and
move to 8). An overridden SOFT run exits 0. `ms` and `mt` never refuse on
the network; 8 is reserved for them and unreachable.

N10 Machine line. When the environment variable `MNEMONIC_NETWORK_JSON` is
`1`, a CLI that exits 8 prints one extra line, the last on stderr, a JSON
object with no spaces outside strings:
`{"kind":"network_mismatch","class":"<hard or soft>","contradictions":[<c>,...]}`
where each `<c>` is
`{"class":"<hard or soft>","text":"<text id>","a":{"what":"<who>","network":"<n>"},"b":{"what":"<who>","network":"<n>"}}`,
`<n>` is one of `main`, `test`, `signet`, `testnet`, `testnet4`, `regtest`
(the grain the comparison used; an HRP is written `main`, `test` or
`regtest`), `<who>` is the text's own name for the source ("key @0",
"key @0's path m/84h/1h/0h", "this run", "cc.json", "plate 3"), and the top
`class` is `soft` only when every contradiction is soft. Without the
variable, no such line. mnemonic-gui sets it on every run (§6.1, Spec D).

N11 Out of the rule: verbs with no network in their input or output
(`decode-address`, `verify-message`, `compare-cost`,
`mt verify`, `ms` share and repair verbs, every `repair` and
`inspect`). They gain neither the flag nor a line.

N12 TX-R. A run whose network is a test network, that took seed material,
and that made a kit or wrote secret material, prints TX-R once, as the last
line of the run on stderr (after every other line, advisories included).
Exactly: `mr create` and `mr upgrade` given `--seeds`; `mr-sitting sit` on a
test-network plan in which the holder brought at least one seed (ST-BR);
a `mnemonic` run whose stdout output class is `PrivateKeyMaterial`
({{mnemonic-toolkit@4553d2da:crates/mnemonic-toolkit/src/secret_advisory.rs:80}} "pub enum OutputClass {")
and whose input held a seed, ms1, extended private key or WIF; mr-gui shows
GUI-R (§6.2, Spec D). `md`, `mk`, `ms`, `me`, `mt` and the SeedHammer never print
it. It never changes the exit code.

N13 Stray plates (D35, mr). Every `mr` verb that takes plates (`export`,
`verify`, `address`, `member`; `current` takes no plates) sorts each key
given with `--keys` or `--keys-file` (an mk1 plate or a
`[fingerprint/path]xpub` record alike; "plate" below means either) whose key
version is of the other class than the run: if the seat that plate's path
names under the other class's convention (`270028'/<other coin>'/i'` names
seat i) also has a plate or seed of the run's class, the stray plate is set
aside, TX-P prints once for it and the run goes on (exit unchanged); if that
seat has no key of the run's class, the run refuses: TX-PX for each such
seat (seat order), then TX-PX-END, exit 8, stdout empty. The version check
runs before the path check, so a stray plate gets TX-P or TX-PX, never the
path-mismatch text. A plate whose path names no seat under either
convention keeps today's "no seat of this card" handling. On a card with explicit origin paths (no path-convention flag), a plate names the seat whose origin path equals the plate's path, or equals it with the coin step (the second step, 0' or 1') of the other class. A plate of the other class than the run that names a seat this way is sorted as above (TX-P when the seat also has a key of the run's class, else TX-PX and TX-PX-END, exit 8, stdout empty). TX-P or TX-PX replaces today's "an xpub of another network than <net>: set aside, not seated" line for that plate, and on verify its "no seed and no plate given" line.

## 3. Texts (normative, verbatim)

Placeholders: `<P>` the prefix (§1); `<who>` the tool's own name for the
input, as its messages already use it (descriptor keys `key @i`, 0-based in
key order; flags by flag, e.g. `--target-xpub`, `--cosigner @1's mk1 card`,
`--key @0`; mr records `record k`, plates `plate k`); `<pfx>` the key's
prefix as typed (`xpub`, `tpub`, `zpub`, `vpub`, ...); `<net>` the run
network's name (§1); `<run>` is `this run is on <net> (no --network given)`
when no flag was given and `--network is <net>` when it was. Plural forms:
"keys @0 and @1 are test-network keys (tpub)", "keys @0, @1 and @2 are ...".
<net> in TX-S and TX-S-OK, for a run with no --network whose class comes from a key or a coin type, is mainnet for the main class and a test network for the test class (never a chain name: a tpub names none); mk encode and me print it that way.
`mr` and `mr-sitting` write `--network main` where the others write
`--network mainnet`. In `mr`, on a verb that takes `--height` and `--time` (`create`, `upgrade`, `address`, `member`, `export`, `current`, `lost-or-stolen`), every `rerun with --network <X>` and `Rerun with <remedy>` sentence of TX-H1, TX-H3, TX-H4, TX-S and TX-N ends ` and that network's tip in --height and --time` (TX-PX-END's form); `verify`, `decode` and an `export` run without `--height` keep the bare form. TX-T is unchanged (walk 1).

<who> by arm (exact; plural forms by N8): descriptor and policy keys key @i (BIP-388 keys_info index); export-wallet and bundle slots --slot @N.xpub, --slot @N.wif; restore md1 slot key @N; restore --cosigner @N=<mk1> --cosigner @N's mk1 card (plural --cosigner @0's and @1's mk1 cards, verb "are"); --cosigner @N=<xpub> --cosigner @N's xpub; import-wallet Jade and Coldcard multisig jade key @N and coldcard-multisig key @N (N = 0-based order of the key lines in the file; plural jade keys @0, @1 and @2); Coldcard generic JSON: the section name (bip84); --from xpub=, --from xprv=, --from mk1= (addresses, convert, derive-child): --from xpub, --from xprv, --from mk1; a path flag as the TX-S source (convert --path, restore --origin, --slot @N.path): "<who>'s path <path>" is written "the path <path> given with --path" (--origin, --slot @N.path); md --key @N; md --from-mk1 card <csid>; mk address the mk1 card; mk encode --xpub (one card) and <FILE>:<LINE> for --keys FILE; verify-bundle the mk1 card (one card) and mk1 card @N (several; N = the card's slot); mr: unit 4 (Spec C) (M21) and N13; me: key @i, and in me bundle the bundle form of TX-S.

- **TX-H1** (key against the run). Test key on a main run:
  `<P> network mismatch: <who> is a test-network key (<pfx>), but <run>. Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet.`
  Main key on a test run:
  `<P> network mismatch: <who> is a mainnet key (<pfx>), but --network is <net>. Nothing was written. For a mainnet wallet, rerun with --network mainnet. For a <net> wallet, export the key again with the signer set to a test network.`
- **TX-H2** (key against key): `<P> network mismatch: <who1> is a mainnet key (<pfx1>), but <who2> is a test-network key (<pfx2>). Nothing was written. Every key of one wallet is for one network: export the odd key again.` (`<who1>` is the first mainnet key in input order, `<who2>` the first test key; the line names one of each.)
- **TX-H3** (address): `<P> network mismatch: <who> <address> is a <kind> address, but <run>. Nothing was written. Rerun with <remedy>, or give an address of <net>.` `<address>` is the whole address as given; `<kind>` is `mainnet`, `regtest`, or (tb HRP) `signet or testnet`, or (base58 test version) `test-network`; `<remedy>` is `--network mainnet`, `--network regtest`, `--network signet or --network testnet`, or `--network signet, --network testnet or --network regtest` respectively.
- **TX-H4** (an artifact's stated network against the run): `<P> network mismatch: <file> says network <stated>, but <run>. Nothing was written. Rerun with <remedy>, or use the file made for <net>.` `<file>` is the path as given (or `the hand-back`, `the plan`, `the closing file` where the tool names its inputs that way today); `<stated>` is the stated name, or for a two-valued statement `chain BTC (mainnet)`, `chain XTN (a test network)`, `chain XRT (a test network)`, `use-info 0 (mainnet)`, `use-info 1 (a test network)`; `<remedy>` is `--network <stated name>` for an exact name, `--network mainnet` (mr: `--network main`) for a main-class statement, and `--network signet, --network testnet or --network regtest` for a test-class statement.
- **TX-H4P** (mr-sitting, a file against the plan; mr-sitting `sit` and `verify` have no `--network`, the plan states the run network): `mr-sitting: network mismatch: <file> says network <stated>, but the plan says network <net>. Nothing was written. Use the <hand-back or closing file> written from this plan, or give the plan it was written from with --plan.` (`<file>` as in TX-H4; `<net>` the plan's network name; `<hand-back or closing file>` is `hand-back` or `closing file`, whichever `<file>` is.)
- **TX-H4C** (an artifact that contradicts itself): `<P> network mismatch: <file> says <stated>, but its <section> key is a <class> key (<pfx>). Nothing was written. No --network repairs a file that contradicts itself: export the file again from the <device> with its network set for the wallet's network.` `<section>` is the file's own name for the key's entry (`bip84`, `bip48_2`, ...); every section object of the file that holds an xpub is compared with chain, in the file's key order; the legacy top-level xpub is named xpub; several sections of one class that all disagree with chain are one line, "its bip44, bip49 and bip84 keys are test-network keys (tpub)". The Coldcard parser accepts chain XRT (regtest, test class) in sniff and parse; `<device>` is `Coldcard` for a Coldcard export and `signer` otherwise. For a mr-sitting hand-back the line is `mr-sitting: network mismatch: <file> says network <stated>, but its record @<j> key is a <class> key (<pfx>). Nothing was written. This hand-back does not agree with itself: run mr-sitting sit again from the plan to write a new one.` (`record @<j>` or `hot-key` names the entry: the first that disagrees, in file order.)
- **TX-H5** (WIF): `<P> network mismatch: the WIF key is a <class> key (version byte <n>), but <run>. Nothing was written.` followed by TX-H1's two remedy sentences for the same direction. (`<n>` is 128 or 239.)
- **TX-S** (coin type). Against a key version:
  `<P> network mismatch (coin type): <who>'s path <path> is a <pclass> path (coin type <c>h), but <who> is a <kclass> key (<pfx>). Nothing was written. If the path is a mistake, export the key again at the <kclass> path (coin type <c2>h). To use this key as it is, add --accept-coin-type-mismatch: the output is then for <net> and this warning is printed every time.`
  Against the run (no key version):
  `<P> network mismatch (coin type): <who>'s path <path> is a <pclass> path (coin type <c>h), but <run>. Nothing was written. For a <pclass> wallet, rerun with <remedy>. To use this path on <net> as it is, add --accept-coin-type-mismatch: the output is then for <net> and this warning is printed every time.` (`<remedy>` is `--network mainnet` for a coin-0 path and `--network signet, --network testnet or --network regtest` for coin 1.)
  Path against path (tools with no `--network`):
  `<P> network mismatch (coin type): <who1>'s path <path1> is a mainnet path (coin type 0h), but <who2>'s path <path2> is a test-network path (coin type 1h). Nothing was written. Every key of one wallet is for one network: export the odd key again. To use these paths as they are, add --accept-coin-type-mismatch: the output is then for mainnet and this warning is printed every time.`
  Bundle form (me bundle: the path comes from an md1 card, the key from an mk1 card): `me: network mismatch (coin type): the md1 card's path <path> for key @<i> is a <pclass> path (coin type <c>h), but the mk1 card <fp> for it is a <kclass> key (<pfx>). Nothing was written. Make the md1 card again with the <kclass> path (coin type <c2>h), or the mk1 card again from the key at the <pclass> path. To use these as they are, add --accept-coin-type-mismatch: the output is then for <net> and this warning is printed every time.`
  (`<path>` is `m/` followed by the origin's steps, read from the parsed path, each hardened step written with `h` (`m/84h/1h/0h`); `mr` and `mr-sitting` write `'` instead (`m/84'/1'/0'`, and coin type `<c>'`), as their other texts do; `<pclass>`/`<kclass>` are `mainnet` or `test-network`.) Several sources in one contradiction (N8) are written as one: `<who>` becomes `keys @0 and @1` (`keys @0, @1 and @2`, ...), `<who>'s path <path> is a <pclass> path` becomes `keys @0 and @1's paths <path0> and <path1> are <pclass> paths`, `<who> is a <kclass> key` becomes `keys @0 and @1 are <kclass> keys`, and TX-S-OK takes the same plural; e.g. `error: network mismatch (coin type): keys @0 and @1's paths m/87h/1h/0h and m/87h/1h/0h are test-network paths (coin type 1h), but this run is on mainnet (no --network given). ...` The rest of the line is unchanged; it is one contradiction (N8), whose machine-line (N10) `a.what` is the plural `<who>` with `'s paths` (`keys @0 and @1's paths`).
- **TX-S-OK** (override used): `warning: network mismatch (coin type) accepted with --accept-coin-type-mismatch: <who>'s path <path> is a <pclass> path; the output is for <net>. A signer or recovery tool that looks only at <net> paths will not find coins sent here.`
- **TX-N** (no source, no flag): `warning: nothing here names a network and no --network was given: read as mainnet. For a signet, testnet or regtest wallet, rerun with --network signet, --network testnet or --network regtest.`
- **TX-N-MT** (mt, no node, no source): `warning: no node answered and nothing here names a network: addresses are shown as mainnet. For a transaction on a test network, rerun with --bitcoin-cli naming a script that runs bitcoin-cli for that network (for signet, an executable file (chmod +x) holding the two lines `#!/bin/sh` and `exec bitcoin-cli -signet "$@"`), or set that network in bitcoin.conf (`chain=signet`).`
- **TX-T** (no source, test flag): `warning: this run is on <net>, a test network, because --network says so; this <thing> names no network. A wallet holding real bitcoin is read with --network <main>.` (`<thing>` is `card` for a card or plate, `key` for a key, `descriptor` for a descriptor; `<main>` is `main` in mr, `mainnet` elsewhere.) In mr on a card this reads exactly as walk 1 accepted: `warning: this run is on signet, a test network, because --network says so; this card names no network. A wallet holding real bitcoin is read with --network main.`
- **TX-R** (rehearsal with real seeds): `warning: This rehearsal used your real seeds. Everything it engraved or printed from them is as secret as your real kit: destroy it, do not discard it.`
- **TX-P** (mr stray plate set aside). Test plate on a main run: `warning: plate <k> (<path>) is a test-network key (<pfx>) from a rehearsal; it was not used. Seat @<i> is filled by <filler>.` Main plate on a test run: `warning: plate <k> (<path>) is a mainnet key (<pfx>); it was not used on <net>. Seat @<i> is filled by <filler>.` (`<filler>` is `plate <m>`, `record <m>` or `seed <n>`. When both a plate or record and a seed fill the seat, `<filler>` is the plate or record, the lowest-numbered when several.) In TX-P and TX-PX, `plate <k>` reads `record <k>` when the stray key is a `[fingerprint/path]xpub` record, as mr names that input today. It replaces today's two lines for that plate.
- **TX-PX** (mr seat with only a stray plate): `mr: network mismatch: seat @<i> has no <net> key: plate <k> (<path>) is a <class> key (<pfx>)<origin>.` where `<origin>` is ` from a rehearsal` for a test key on a main run and empty otherwise.
- **TX-PX-END** (closes the TX-PX lines): `mr: Nothing was written. Give each seat named above its <net> plate with --keys or its seed with --seeds; for a wallet on another network, rerun with that --network (and, on a verb that takes --height and --time, that network's tip).`
- **TX-V** (verify-bundle cause line, after its mismatch rows, exit 4 unchanged): `note: every mismatch above is a coin-type or key-version row: the bundle may be for another network than --network <net>; rerun with the network the bundle was made for.`

GUI and device texts (§6, Spec D) are GUI-H, GUI-P, GUI-N, GUI-S0, GUI-R, MG-BANNER,
MG-OVERRIDE, MG-DIALOG, MG-AMBER, SH-S, SH-S-RUN, SH-K, SH-L, SH-RT.

## 4. Per-arm table

"stays" = today's refusal is kept; it moves to exit 8 and the named text.
"flag" = `--accept-coin-type-mismatch`. Today's exit codes are from the RED
rows (§11 of Specs A to D) or the study's recon rows (named "recon").

### 4.1 mnemonic-toolkit (`mnemonic`)
Arms here are built by units in SPEC_network_rule.md (0, 1, 7) and SPEC_network_rule_tk_rest.md (2, 3, 8a, 8b).

| arm | sources | class | today | after | text | exit | flag |
|---|---|---|---|---|---|---|---|
| export-wallet `--descriptor`, key version against run (every format) | key, run | HARD | rc 0, wrong-network output (R1.1-R1.3, R1.7, R1.8, R1.11) | refuse before any format renders | TX-H1 | 8 | no |
| export-wallet `--descriptor`, key against key | keys | HARD | rc 0 (R1.14) | refuse | TX-H2, then TX-H1 without its rerun sentence (N8) | 8 | no |
| export-wallet `--descriptor`, origin coin type | coin, key, run | SOFT | rc 0 (R1.4-R1.6) | refuse; flag lets through | TX-S / TX-S-OK | 8 / 0 | yes |
| export-wallet `--descriptor`, no extended key and no origin coin type (raw public keys) | none | - | rc 0, no line (R1.16, R1.17) | TX-N with no flag, TX-T (`<thing>` = `descriptor`) with a test flag, on every `--format` | TX-N / TX-T | 0 | no |
| export-wallet `--descriptor`, raw public keys with an origin coin type | coin, run | SOFT | rc 0, no line (R1.18) | refuse; flag | TX-S (run form) | 8 / 0 | yes |
| export-wallet `--template`/`--slot` | slot key, run | HARD | rc 2 (recon) | stays | TX-H1 | 8 | no |
| export-wallet `--from-import-json`, envelope network against the run (flag or no flag) | stated, run | HARD | rc 0; `--network` is ignored and the envelope's network wins (R1.13, R1.15) | refuse | TX-H4 (`<file>` = the path given to `--from-import-json`) | 8 | no |
| export-wallet `--from-import-json`, envelope network against its own keys | stated, key | HARD | rc 2 `NetworkMismatch` (cycle-5 S-NET) | stays | TX-H4C (`<section>` = `bundle.descriptor`, `<device>` = `signer`) | 8 | no |
| addresses `--from xpub=`, with or without `--network` | key, run | HARD | flag: rc 1 (R8b.2); no flag on a tpub: rc 0 `tb1` (R8b.1) | refuse | TX-H1 | 8 | no |
| convert xpub to address, with or without `--network` | key, run | HARD | no flag on a tpub: rc 0 `tb1` (R8b.4); flag: rc 2 (recon) | refuse | TX-H1 | 8 | no |
| convert any key source (xpub, xprv, mk1) to any target, key version against the run (flag or no flag; N4 is unconditional, so `--to fingerprint` refuses too) | key, run | HARD | rc 0 with the key's own version, flag ignored (R8a.5, R8a.13-R8a.16) | refuse | TX-H1 (`<who>` = `--from xpub`, `--from xprv`, `--from mk1`) | 8 | no |
| convert `--xpub-prefix` | key, run | HARD | rc 2 (recon) | stays | TX-H1 | 8 | no |
| convert wif to xpub | WIF, run | HARD | rc 2 (recon) | stays | TX-H5 | 8 | no |
| convert phrase/ms1/entropy/seedqr to xpub, xprv, wif, address or bip38 with `--path` | path coin, run | SOFT | rc 0 (R8a.3, R8a.4, R8a.17) | refuse; flag | TX-S | 8 / 0 | yes |
| bundle `--slot @N.xpub=` | key, run | HARD | rc 1 (recon) | stays | TX-H1 | 8 | no |
| bundle `--slot @N.wif=` | WIF, run | HARD | rc 0, a mk1 built with the run's version bytes whatever the WIF's (R8a.10) | refuse before any card | TX-H5 | 8 | no |
| bundle and export-wallet `--template`/`--slot`, a slot's `@N.path=` coin type against its key or the run | coin, key, run | SOFT | rc 0, no line (R8a.11, R8a.12) | refuse; flag | TX-S (key form when the slot has an xpub, run form for a seed slot) | 8 / 0 | yes |
| bundle `--descriptor` | keys, coins, run | HARD + SOFT | rc 0, JSON network from the flag (R8a.1, R8a.2) | as export-wallet `--descriptor` | TX-H1/H2/S | 8 / 0 | SOFT only |
| bundle `--import-json` | envelope, run | HARD | rc 1 (recon) | stays | TX-H4 | 8 | no |
| bundle / restore / convert from a seed, no source | none | - | output for the run | unchanged, no TX-N (N6); TX-R when N12 holds | TX-R | 0 | no |
| restore `--from phrase= --origin <path>` | path coin, run | SOFT | rc 0 (R8a.6) | refuse; flag | TX-S | 8 / 0 | yes |
| restore `--search-address` | HRP, run | HARD | rc 1 (recon) | stays | TX-H3 | 8 | no |
| restore `--md1`, no cosigner | md1 coin types, run | SOFT | rc 0, wrong-network keys (R2.1-R2.3) | refuse; flag | TX-S (run form) | 8 / 0 | yes |
| restore `--md1`, a keyed md1 whose paths have no N1 coin type and no `--cosigner` | none | - | rc 0, no line | TX-N with no flag; TX-T (`<thing>` = `card`) with a test flag; none with `--network mainnet` | TX-N / TX-T | 0 | no |
| restore `--md1` + `--cosigner @N=<mk1>` or bare xpub | key, md1 coin, run | HARD (+SOFT) | rc 0 "cross-checked" (R2.4, R2.7) | refuse | TX-H1 (+TX-S) | 8 | no |
| restore `--expect-xpub` of the other class | key, run | HARD | rc 4, `--allow-mismatch` passes it (recon) | refuse before the expect check; `--allow-mismatch` does not pass it | TX-H1 | 8 | no |
| xpub-search path-of-xpub / passphrase-of-xpub, target key against run | key, run | HARD | rc 4 "no match" (R8a.7) | refuse before searching | TX-H1 | 8 | no |
| xpub-search address-of-xpub, target address against key or run | HRP, key, run | HARD | rc 4 "no match" (R8a.8) | refuse before searching | TX-H3 | 8 | no |
| xpub-search account-of-descriptor, coin type against key | coin, key | SOFT | rc 0 (recon) | refuse; flag | TX-S | 8 / 0 | yes |
| xpub-search account-of-descriptor, descriptor key version against the run (flag or no flag), `--descriptor-from` too | key, run | HARD | rc 4 "no match" (R8a.20) | refuse before searching | TX-H1 (`key @0`) | 8 | no |
| silent-payment from xprv/tprv | key, run | HARD | rc 2 (recon) | stays; TX-R when N12 holds | TX-H1 | 8 | no |
| build-descriptor, keys against run (flag or no flag) | keys, run | HARD | rc 0 with a warning or an inferred preview (R7.1, R7.2, R7.5) | refuse | TX-H1 | 8 | no |
| build-descriptor, key against key | keys | HARD | rc 0 (R7.3) | refuse | TX-H2, then TX-H1 without its rerun sentence (N8) | 8 | no |
| build-descriptor, coin type | coin, key | SOFT | rc 0 (R7.4) | refuse; flag | TX-S | 8 / 0 | yes |
| import-wallet, every parser: key version against the blob's coin type | key, coin | SOFT | rc 2, no override (R3.7) | refuse; flag | TX-S | 8 / 0 | yes |
| import-wallet, blob class against run (flag or no flag) | keys, stated, run | HARD | flag: rc 1 (recon); no flag on a test blob: rc 0 (R3.9) | refuse | TX-H1 / TX-H4 | 8 | no |
| import-wallet coldcard generic JSON, `chain` against key | stated, key | HARD | rc 0 (R3.1-R3.3); with a flag rc 1 naming the wrong coin type (R3.5) | refuse; the flag never repairs it | TX-H4C | 8 | no |
| import-wallet jade / coldcard-multisig | keys, coins, run | HARD / SOFT | rc 2 "coldcard-multisig" (R3.6, R3.8, R3.10) | as the rows above; the text names `jade` for a Jade file | TX-H1/H2/S | 8 | SOFT only |
| verify-bundle, wrong `--network` | cards, run | HARD / SOFT | rc 4 mismatch rows (recon) | refuse before the rows when an mk1 card's key version contradicts the run (TX-H1, `<who>` = `the mk1 card`, exit 8); an md1 coin type against the run is SOFT (TX-S run form, exit 8; with the flag the rows print as today, TX-S-OK); TX-V stays for the remaining case (rows fail and no source contradicts the run) | TX-H1 / TX-S / TX-V | 8 / 4 | SOFT only |
| verify-bundle watch-only slot | key, run | HARD | rc 1 (recon) | stays | TX-H1 | 8 | no |
| derive-child from an xprv or tprv master, key version against the run (flag or no flag), every `--application` (hex, bip39 and password output names no network; N4 is unconditional, so they refuse too) | key, run | HARD | emits a key of the run's version, rc 0 (R8a.18, R8a.19) | refuse; TX-R when N12 holds on a test run | TX-H1 (`--from xprv`) | 8 | no |
| nostr `--pubkey` (no network) | none | - | rc 0, no line (R8a.9) | TX-N with no flag; TX-T with a test flag | TX-N / TX-T | 0 | no |
| nostr `--secret`/`--secret-file`/`--secret-stdin` | none | - | rc 0 | TX-N with no flag; TX-T with a test flag (a key with no network, D5) | TX-N / TX-T | 0 | no |
| decode-address, verify-message, compare-cost, repair, inspect | - | out | - | unchanged (N11) | none | - | no |

Restore md1 rules: a --from seed with a md1 is the same as no --from (a seed typed now does not exempt a md1 coin type: R2.9); a slot whose md1 path coin disagrees with a --cosigner key's version prints TX-S in its key form for that slot (N8, M20), and when that key also disagrees with the run only TX-H1 prints for it (N8: a coin type agreeing with its own key named on a HARD line is no separate line); slots whose coin disagrees with the run and that have no key print one TX-S run-form line per coin class, after every HARD line.

### 4.2 md, mk, ms
Arms here are built by units in SPEC_network_rule_tk_rest.md (8b) and SPEC_network_rule_codecs_gui.md (8d).

| tool, arm | sources | class | today | after | text | exit | flag |
|---|---|---|---|---|---|---|---|
| md address/descriptor/encode `--key`, version against run | key, run | HARD | rc 1 "expected mainnet xpub version" (R8b.3) | refuse | TX-H1 | 8 | no |
| md decompose | key, run | HARD | rc 1 (recon) | stays | TX-H1 | 8 | no |
| md address/descriptor `--from-mk1`/`--from-mk1-file`, an mk1 card's key version against the run (flag or no flag) | key, run | HARD | rc 1, md's own seating text (probed) | refuse; the seating text is replaced | TX-H1 (`--from-mk1 card <csid>`) | 8 | no |
| same, two mk1 cards of different classes | keys | HARD | not compared | refuse | TX-H2 | 8 | no |
| md decompose, origin coin type against key | coin, key | SOFT | rc 0, commands printed without a flag (probed: tpub at 84'/0' with --network signet) | refuse; flag; with the flag every printed md encode and mk encode command carries --accept-coin-type-mismatch (as mr upgrade) | TX-S / TX-S-OK | 8 / 0 | yes |
| md `--key` with an origin coin type | coin, key | SOFT | rc 0 (R8d.1, R8d.2) | refuse; flag | TX-S | 8 / 0 | yes |
| md address/descriptor on a keyed md1 card (key bytes carry no version) with origin coin types | coin, run | SOFT | renders silently (recon) | refuse; flag | TX-S (run form) | 8 / 0 | yes |
| md address/descriptor on a card with no source, no flag | none | - | rc 0; the note "note: network mainnet (--network, default mainnet): ..." on the arms that print it | TX-N in place of the note (R8d.5) | TX-N | 0 | no |
| same, a test `--network` | none | - | rc 0, the note naming the network where printed (R8d.6) | TX-T in place of the note; on md descriptor and on --json too: TX-N / TX-T print once on stderr; the note is not added where it was not printed before (only md address prints it); md encode --json's "network":"mainnet" field for a keyless card is unchanged | TX-T | 0 | no |
| same, `--network mainnet` given | none | - | the note | the note stays as today (Brian 2026-11-20 "md names network") | none | 0 | no |
| md `--path` shortcuts (bip44/48/49/84/86 and their 0xNN spellings) with a test `--network` | shortcut coin, run | - | coin 0 hard-coded (recon) | the shortcut takes coin 1 on a test run, coin 0 on main; no contradiction arises; parse_path takes the run network; coin 1 on signet, testnet and regtest | none | 0 | no |
| mk encode, origin coin type against key | coin, key | SOFT | rc 0 (R8d.3, R8d.4) | refuse; flag | TX-S | 8 / 0 | yes |
| mk encode `--keys`/`--in`, key against key | keys | HARD | rc 0, one card per record (probed) | refuse before minting any card; no flag | TX-H2 (`<FILE>:<LINE>`) | 8 | no |
| mk address, key against run (flag or no flag) | key, run | HARD | flag: rc 64 (R8b.6); no flag on a tpub: rc 0 `tb1` (R8b.5) | refuse | TX-H1 | 8 | no |
| mk address, coin type against key | coin, key | SOFT | rc 0 (recon) | refuse; flag | TX-S | 8 / 0 | yes |
| ms derive | `--network` only | - | path and version follow the flag | unchanged (no contradiction possible) | none | 0 | no |

### 4.3 mr, mr-sitting
Arms here are built by units in SPEC_network_rule_mr.md (4, 5, 8c).

| arm | sources | class | today | after | text | exit | flag |
|---|---|---|---|---|---|---|---|
| create `--keys`, `--hot-key`, `--sibling-keys`, `--bundle-card` keys, version against run | key, run | HARD | `--keys` and `--hot-key` rc 1 (R8c.5, R8c.6; `--hot-key` is checked in create only, hotrun.rs:845); `--sibling-keys` tpub on main: rc 0, kit printed, no line (R8c.7); `--bundle-card` not run | refuse | TX-H1 | 8 | no |
| create `--keys`, key at the other coin's seat path | record path, seat path | not overridable (a path-convention card cannot name another coin) | rc 1 (recon) | today's text kept | existing | 8 | no |
| create `--hot-key`, `--sibling-keys`, `--bundle-card` md1, origin coin type against key or run | coin, key, run | SOFT | not checked: rc 0, no line (R8c.4) | refuse; flag; with the flag each printed command line of the mr verbs that take the flag, and of mk encode, ends with it (R8c.1) (as upgrade) | TX-S / TX-S-OK | 8 / 0 | yes |
| upgrade, key version against run | key, run | HARD | rc 0, kit printed (R4.1-R4.3) | refuse (the check `create` has) | TX-H1 | 8 | no |
| upgrade, md1 coin type against run | coin, run | SOFT | rc 0 + warning (R8c.1) | refuse; flag; with the flag each printed command line of the mr verbs that take the flag, and of mk encode, ends with it (R8c.1) | TX-S / TX-S-OK | 8 / 0 | yes |
| upgrade `--funded-address` | HRP, run | HARD | rc 1 (recon) | stays | TX-H3 | 8 | no |
| create/upgrade `--root-record` hand-back | stated, run | HARD | rc 1 (recon) | stays | TX-H4 | 8 | no |
| create/upgrade `--seeds` on a test network | - | - | no line (R5.5) | TX-R last on stderr (N12) | TX-R | 0 | no |
| root / child, no source | none | - | rc 0 | unchanged (N6: derivation from a root typed now) | none | 0 | no |
| verify/decode/address/member/current/export on a path-convention card, no flag | none | - | rc 0, no line (R5.1-R5.3) | TX-N | TX-N | 0 | no |
| same, a test `--network` | none | - | rc 0, no line (R5.4) | TX-T | TX-T | 0 | no |
| read verbs on an explicit-path card, coin type against run (per key) | coin, run | SOFT | warning, rc 0 (recon) | refuse; flag | TX-S | 8 / 0 | yes |
| pubkeys-TLV card, no flag | none | - | rc 1 `TlvHandoutNetworkNotGiven` | read as mainnet (D1), TX-N; `TlvHandoutNetworkNotGiven` retired (erratum E5) | TX-N | 0 | no |
| verbs taking plates, a stray other-class plate, its seat also filled | key, run | - | rc 0, two warning lines (R8c.2) | TX-P replaces the two lines; output unchanged | TX-P | 0 | no |
| same, the seat filled only by the stray plate | key, run | HARD | rc 1, header lines on stdout (R8c.3) | refuse, stdout empty | TX-PX, TX-PX-END | 8 | no |
| mr-sitting verify: hand-back record kind against its own network line | key, stated | HARD | rc 2 (recon) | stays | TX-H4C | 8 | no |
| mr-sitting sit/verify: hand-back or closing file network against plan | stated, stated | HARD | rc 2: plan-id and plan-sha256 are compared first, so a file from another plan never reaches the network check (R8c.8) | verify: the hand-back's network line is compared with the plan's first, before plan-id and plan-sha256 (ST-HB-02), so a hand-back from a plan on another network gets TX-H4P, exit 8; sit: a closing file of another network read mid-session (closing.rs, I8Error::Differs) keeps its waiting-screen line and the sitting goes on, with the network compared first so the line names the network; a closing file whose own command's --network differs from its network line contradicts itself: TX-H4C in verify, the same screen line in sit | TX-H4P / TX-H4C | 8 | no |
| mr-sitting sit/verify, reading the plan: seat path coin against the plan's network | path, stated | not overridable | rc 2 (recon) | today's text kept | existing | 8 | no |
| mr-sitting sit, test plan, a brought seed | - | - | ST-END-02 last | TX-R after ST-END-02 (N12) | TX-R | 0 | no |
| lost-or-stolen on a path-convention card, no flag | none | - | rc 0, no line; the successor advice prints the seat export path at coin 0 and a `mr create --supersedes` line with no `--network` (R5.7) | TX-N before the first advice line; with a test `--network` TX-T | TX-N / TX-T | 0 | no |
| lost-or-stolen, explicit-path card (positional, `--card`, `--sibling`), coin type against run | coin, run | SOFT | warning, positional card only (lost_or_stolen.rs:583); `--card` and `--sibling` cards' warnings dropped | refuse per card; flag | TX-S | 8 / 0 | yes |
| tip check (`--accept-tip`) | - | not a network check | refuses a main run at a signet height | unchanged; never counted as a network check | - | 1 | - |

### 4.4 me, mt
Arms here are built by units in SPEC_network_rule_codecs_gui.md (6, 8e).

| tool, arm | sources | class | today | after | text | exit | flag |
|---|---|---|---|---|---|---|---|
| me sysw pack, mixed key versions | keys | HARD | rc 3 (R6.3) | refuse | TX-H2 | 8 | no |
| me sysw pack, coin type against key | coin, key | SOFT | rc 0 (R6.1, R6.2) | refuse; flag | TX-S | 8 / 0 | yes |
| me sysw pack / bundle, keyed md1 with no key versions and coin types that disagree | coins | SOFT | rc 0 (recon) | refuse; flag; overridden output mainnet (N2) | TX-S (path form) | 8 / 0 | yes |
| me sysw pack, test-class payload | key or coins | - | network from the key; no network line | unchanged: me prints no network line | none | 0 | no |
| me bundle, mk1 against mk1 | keys | HARD | not compared (recon) | refuse | TX-H2 | 8 | no |
| me bundle, an mk1 card's own origin coin type against its own key | coin, key | SOFT | rc 0 (recon) | refuse; flag | TX-S (key form, `<who>` = `the mk1 card <fp>`) | 8 / 0 | yes |
| me bundle, md1 coin type against mk1 key | coin, key | SOFT | rc 0 (R6.4, R6.5) | refuse; flag | TX-S (bundle form) | 8 / 0 | yes |
| me promotion of a bare testnet key (`PromotionTestnetKey`) | key only | not a network contradiction | rc 3 | unchanged (its text and exit 3) | existing | 3 | no |
| me sysw pack, `key:` record with a test-network xpub (`ComposerRecordError::KeyNetwork`; the composer is mainnet-only) | key | not a network contradiction (N2: the lone key decides the run; the composer cannot take it) | rc 4 | unchanged (its text and exit 4) | existing | 4 | no |
| me sysw pack, `key:` record, coin type against its key | coin, key | SOFT | rc 0 | refuse; flag | TX-S | 8 / 0 | yes |
| mt inspect, a node names a chain (testnet4 included) | Core chain | only source | head: that chain's HRP; installed 0.2.0 treats testnet4 as no node (R8e.3) | unchanged at head; reinstall | none | 0 | no |
| mt inspect, no node | none | - | mainnet, note in the report only (R8e.1, R8e.2) | TX-N-MT on stderr; JSON gains `"network":"mainnet","network_source":"default"` (with a node: the chain and `"node"`) | TX-N-MT | 0 | no |
| mt decode, encode (its report), inspect of mt1 strings or raw bytes, no node | none | - | the report's addresses are mainnet with a note in the report (R8e.1, R8e.5, R8e.6) | TX-N-MT once on stderr; JSON gains the network fields | TX-N-MT | 0 | no |
| mt, a node answered but named no chain or a chain mt does not know (NotSaid, Unrecognised) | none | - | mainnet, the report's own note | unchanged: the report's note stays, no TX-N-MT (a node did answer); JSON network_source "node-unreadable" | none | 0 | no |
| mt verify | - | out | - | unchanged | none | - | no |

me bundle pairs md1 placeholder @i with the mk1 whose origin fingerprint equals the placeholder's; an mk1 or a placeholder with no fingerprint is compared with nothing; one line per pair that differs, merged by N8's plural.

### 4.5 SeedHammer firmware (screens, no exit code)
Arms here are built by units in SPEC_network_rule_codecs_gui.md (8h).

| arm | class | today | after | text |
|---|---|---|---|---|
| address check, candidate of another network | HARD | "Address is for a different network." | stays | existing |
| `bcrt1` candidate | - | "Invalid address." | names regtest | SH-RT |
| mixed key versions | HARD | refuses | stays | existing |
| coin type against key (descriptor, address check, `key:` record, mk1, UR use-info against path, md1 policy address check) | SOFT | accepted silently | warning screen, default Back, hold gesture goes on (D7) | SH-S / SH-S-RUN |
| `key:` record with a tpub | HARD | generic record error | names the reason | SH-K |
| test-network mk1 in the composer list | HARD | silently not offered | listed, not selectable, with a reason | SH-L |
| test-network mk1 as a cosigner in Multisig Build or Multisig Verify | HARD | accepted, version bytes dropped, built with MainNet params | listed, not selectable, with a reason (SH-L's text) | SH-L |

### 4.6 mr-gui and mnemonic-gui
Arms here are built by units in SPEC_network_rule_mr.md (8g) and SPEC_network_rule_codecs_gui.md (8f).

| app, arm | class | today | after | text |
|---|---|---|---|---|
| mr-gui Create, mainnet records | HARD | refused, generic lead | refused with its own lead | GUI-H |
| mr-gui Create/Restore, key or plate at the other coin's path | not overridable | text names `--network main` | refused, no CLI flag named | GUI-P |
| mr-gui Restore, plate of the other network, seat otherwise filled | - | set aside, with mr's two warning lines in the panel | set aside silently in the GUI (the app shows no CLI warning lines); the restore goes on | none |
| mr-gui Restore, plate of the other network, seat filled by nothing else | HARD | refused for a missing seat | refused naming the network | GUI-H |
| mr-gui, node chain against profile; Send to an address of another network; hand-back, closing file or saved draft of another network | HARD | refused (a saved draft silently) | stays; the saved-draft refusal shows GUI-H's second sentence, `This version of Refugium works on signet only.` | existing |
| mr-gui, a card that names no network | - | verified on signet silently | one line under the card on Restore and Balance | GUI-N |
| mr-gui, explicit-path card with coin-0 paths | SOFT | mr's warning lines | refused, no override while signet-only | GUI-S0 |
| mr-gui Create finished from a sitting's hand-back | - | nothing | GUI-R on the finish page | GUI-R |
| mnemonic-gui, any form, CLI exit 8 | - | `exit: 8` label, stderr pane | red banner (§6.1, Spec D) | MG-BANNER |
| mnemonic-gui, CLI exit 8 with the machine line's class `soft` | SOFT | nothing | Override button, typed dialog, one re-run | MG-OVERRIDE, MG-DIALOG, MG-AMBER |

## 5. Machine line examples (N10)

A refused `export-wallet` of R1.4 under `MNEMONIC_NETWORK_JSON=1` ends stderr with:
`{"kind":"network_mismatch","class":"soft","contradictions":[{"class":"soft","text":"TX-S","a":{"what":"key @0's path m/84h/1h/0h","network":"test"},"b":{"what":"key @0","network":"main"}}]}`
R1.14 ends with two contradictions, TX-H2 then TX-H1, and top `"class":"hard"`.

## 6. GUI and device design

GUI and device design (mnemonic-gui, mr-gui, SeedHammer) is in SPEC_network_rule_codecs_gui.md (Spec D) §6.

## 7. Shared vector table

`design/network-rule/vectors.tsv` is part of this spec (normative). Each
repo's unit-0 test reads it (vendored copy, checked by hash against this
file at the pinned rev) and runs every row its tool can express.

Schema (tab-separated, header exact): `id` (V01...); `tool` (`any` or a
tool name); `has_network_flag` (`yes`/`no`); `run_network` (`main`,
`signet`, `testnet`, `regtest`, or `none` = no `--network` given, or the
tool has none); `sources` (`;`-separated, `-` for none: `key=main|test`,
`hrp=bc|tb|bcrt`, `b58=main|test`, `wif=main|test`,
`core=main|test|testnet4|signet|regtest`, `stated=<exact name>`,
`stated2=main|test` (a two-valued statement), `coin=0|1`, `path=other`,
`reads=kept|seed`, `out=secret`, `plate=dup-other|only-other`); `override`
(`0`/`1`: `--accept-coin-type-mismatch` given); `class` (`hard`, `soft`,
`hard+soft`, `none`, `no-source`); `text` (text ids in print order, `,`
separated, `-` for none); `exit` (`0` or `8`). Invariants the gate checks:
hard and hard+soft exit 8 whatever the override; soft exits 8 without the
override and 0 with it; none and no-source exit 0; a row with
`has_network_flag` `no` never names a `--network` remedy. Rows: 61 (the
file).

## 8. Exit-code tables per tool (after)

Exit-code grep revs: mnemonic-toolkit@4553d2da mnemonic-refugium@34300ff27 descriptor-mnemonic@60956b858 mnemonic-key@af12fea85 mnemonic-secret@601d56850 mnemonic-transaction@88ac6d9e0 mnemonic-engrave@0bd8dfbab

Result of the grep (code exit sites, and the toolkit manual's CLI pages,
which hold the exit tables of mnemonic, md, ms, mk, mr and mr-sitting): 8
is used as an exit code by none of them.

| tool | codes today | after |
|---|---|---|
| mnemonic | 0, 1, 2, 3, 4, 5, 64 | + 8 network mismatch; the network refusals that exit 1 or 2 today ({{mnemonic-toolkit@4553d2da:crates/mnemonic-toolkit/src/error.rs:684}} "ToolkitError::NetworkMismatch { .. } => 2,"; {{mnemonic-toolkit@4553d2da:crates/mnemonic-toolkit/src/error.rs:672}} "ToolkitError::ImportWalletNetworkClassMismatch { .. } => 1,"; the `addresses` and bundle-slot refusals) exit 8 |
| md | 0, 1, 2, 5 (repair) | + 8; md's version refusals (exit 1) exit 8 |
| mk | 0, 1, 2, 3, 4, 5, 64 | + 8; `mk address`'s refusal (64, {{mnemonic-key@af12fea85:crates/mk-cli/src/main.rs:87}} "_ => ExitCode::from(64),") exits 8 |
| ms | 0, 1, 2, 4, 64 | 8 reserved, unreachable (no contradiction possible) |
| mr | 0, 1, 2, 5, 101 ({{mnemonic-refugium@34300ff27:crates/mr-codec/src/app/exit_codes.rs:5}} "pub const FAILURE: u8 = 1;") | + 8; every network refusal (exit 1 today) exits 8; `TlvHandoutNetworkNotGiven` is retired (erratum E5) |
| mr-sitting | 0, 1, 2, 101 | + 8; its hand-back and plan network refusals (exit 2) exit 8 |
| me | 0, 2, 3, 4 ({{mnemonic-engrave@0bd8dfbab:crates/me-cli/src/main.rs:484}} "const EXIT_REFUSED: i32 = 3;") | + 8; `MixedNetwork` (3) exits 8; `PromotionTestnetKey` stays 3 ({{mnemonic-engrave@0bd8dfbab:crates/me-cli/src/descriptor/refusal.rs:65}} "PromotionTestnetKey,"); the vocabulary test ({{mnemonic-engrave@0bd8dfbab:crates/me-cli/tests/cli.rs:1111}} "fn the_exit_code_vocabulary_is_one_vocabulary()") gains 8 |
| mt | 0, 1, 2 | 8 reserved, unreachable |

Each tool's manual exit table gains the row: `8` = "network mismatch: a
source of one network contradicts the run's network or another source
(SPEC_network_rule); nothing was written".

## 9. Rulings and errata

Each ruling (Brian, 2026-11-23), with "spec text affected?".
- D1 (no `--network` = mainnet, an assertion): N2. Spec text affected:
  toolkit cycle-5 S-NET's "infer the display network from the keys"
  ({{mnemonic-toolkit@4553d2da:design/BRAINSTORM_cycle5_snet_network_invariant.md:171}} "when `--network` is omitted, **infer**")
  and its tests; erratum E1; mr's `TlvHandoutNetworkNotGiven` refusal
  (Brian 2026-11-23, ruling on review r1 s1 I-7: D1 has no exception);
  erratum E5. The inferring arms in §4 change. Tools with no
  `--network` keep "their sources decide" (the study's Q-default, part of
  the decision Brian ruled).
- Walk 1 / D7 (TX-T): TX-T verbatim; every test-network run of a card naming
  no network, exit 0. Spec text affected: mr spec §7 `--network` gains the
  line (unit 5 of Spec C, mr repo).
- D4 (mr1 cards network-neutral): no card format change. No spec text
  affected.
- Walk 2 (TX-R): N12, TX-R, GUI-R. Spec text affected: mr spec §7 (`create`,
  `upgrade`) and mr-sitting's ST-END texts gain the line (unit 5 of Spec C).
- Walk 3 / D35 (stray plates): N13, TX-P, TX-PX, TX-PX-END. Spec text
  affected: mr spec `export`'s "set aside" and the memory's "exit 6"
  (superseded by D3: 8); erratum E4.
- D8 / D34 (refuse every coin-type read unless the flag; kit carries it):
  N7 and the mr rows. Spec text affected: mr review r1-I-2's warn-only
  reading ({{mnemonic-refugium@34300ff27:design/SPEC_ERRATA_mr_v0_1.md:5316}} "need not be a coin type at all"),
  `card::Warning::CoinType` ({{mnemonic-refugium@34300ff27:crates/mr-codec/src/card.rs:924}} "this card's origin paths carry coin type")
  and `upgrade`'s warning ({{mnemonic-refugium@34300ff27:crates/mr-codec/src/app/create.rs:1740}} "the md1 card's origin paths carry coin type");
  erratum E2.
- D2 (`--accept-coin-type-mismatch` everywhere): N7. No spec text affected
  (new flag).
- D3 (exit 8 everywhere): N9, §8. Spec text affected: the study's "exit 6"
  everywhere and the memory's walk-3 "exit 6"; erratum E4.
- D5 (TX-N only on reading a kept artifact or a no-network key): N6. No spec
  text affected beyond md's note (§4.2, review M-5).
- D6 (typed sentence; mr-gui no override; mr-gui hides flags, mnemonic-gui
  may show them): §6 (Spec D). Spec text affected: the study's R9 rewrite of flags
  into field names is dropped (mnemonic-gui shows the CLI line verbatim).
- D7 (SeedHammer warning screen, default Back, hold to go on): SH-S. No spec
  text affected (firmware handoff).
- P21 (export-wallet `--descriptor`): unit 1, unchanged in substance.

Errata (applied by the unit named; the replacement text is binding):
- E1 (toolkit, unit 7; written now as a banner at the top of
  `design/BRAINSTORM_cycle5_snet_network_invariant.md`): L1's "WARN/DIAGNOSE"
  ({{mnemonic-toolkit@4553d2da:design/BRAINSTORM_cycle5_snet_network_invariant.md:49}} "WARN/DIAGNOSE")
  and §4's "infer ... emit a stderr WARN" are replaced by: build-descriptor
  refuses a key/run or key/key contradiction (exit 8, TX-H1/TX-H2) and a
  coin-type contradiction unless `--accept-coin-type-mismatch` (TX-S); no
  `--network` means mainnet. Why: Brian 2026-11-23, P21 and decision 1 ("No
  network means main net, always"; warnings "must be obnoxiously loud (don't
  exit 0...)").
- E2 (mr, unit 8c of Spec C): mr errata item at SPEC_ERRATA_mr_v0_1.md L5316 and the
  spec's card coin-type warning: "an explicit path whose coin type disagrees
  with `--network` is refused (exit 8, TX-S) unless
  `--accept-coin-type-mismatch`; the path is kept as written; each key is
  compared on its own (a card with mixed coin types is checked per key)".
  Why: Brian decision 8 (a).
- E3 (mr, unit 4 of Spec C): {{mnemonic-refugium@34300ff27:design/SPEC_refugium_mr1.md:19490}} "in `create` only"
  becomes "in `create` and `upgrade`". Why: the study's Q-mr-D7, under
  Brian's HARD ruling for version bytes.
- E4 (study and memory): every "exit 6" for a network refusal reads "exit
  8". Why: Brian decisions 3 and 6.
- E5 (mr, unit 5 of Spec C): the mr spec's refusal of a hand-out that fills a seat
  from the Pubkeys TLV on a path-convention card with no `--network`
  ({{mnemonic-refugium@34300ff27:design/SPEC_refugium_mr1.md:27203}} "a seat from the Pubkeys TLV is refused on a card with the")
  and its code
  ({{mnemonic-refugium@34300ff27:crates/mr-codec/src/network.rs:169}} "Err(Error::TlvHandoutNetworkNotGiven { seats })")
  are retired: such a run is a mainnet run (D1), the card names no
  network, so it prints TX-N and exits 0. Why: Brian 2026-11-23, ruling on
  review r1 s1 I-7 (option b): "No network means main net, always" has no
  exception.

## 10. Decisions this spec makes (logged; none changes what a ruling decided)

- Review M-1: mr compares each key's coin type on its own; mixed coin types
  on one card are per-key SOFT contradictions
  ({{mnemonic-refugium@34300ff27:crates/mr-codec/src/network.rs:174}} "pub fn paths_coin(" returns None on a mix today).
- Review M-2: the flag's reason text no longer counts mr's flags; no spec
  text depends on the count.
- Review M-3: purpose 352 (BIP-352 `m / purpose' / coin_type' / account'`)
  joins the SOFT path list.
- Review M-4: the GUI's signal is the JSON `class` (N10) through an
  environment variable, because only mk has a JSON error envelope, printed on stdout in --json mode (the toolkit's is reserved and unused); a variable keeps holders' stderr free
  of JSON and needs no change to any `--json` stdout shape.
- Review M-5: md's network note stays when `--network mainnet` is given and
  when a source agrees; TX-N or TX-T replaces it when they print.
- Review N-1: every RED row names its fixture directory; the gate runs each
  from there.
- Review N-2: no spec text claims tests cannot pin prose; exit 8 is for the
  GUI and for tests that must not pass for the wrong refusal.
- N8 ordering, plural forms and "one line per contradiction".
- TX-H3 prints the whole address (the study shortened it).
- TX-H4C remedy and its `<device>` word.
- N13 is symmetric (a mainnet plate on a test run) and applies to every mr
  verb that takes plates, not only `export`; the version check runs before
  the path check.
- TX-P keeps the `warning:` prefix of mr's other warnings.
- N12's verb list; mr-sitting prints TX-R only when a seed was brought
  (drawn seeds of a test plan are not real seeds), after ST-END-02
  ({{mnemonic-refugium@34300ff27:crates/mr-sitting/src/session.rs:1144}} "// ST-END-02 (E-SIT-58).");
  ST-END-02 itself is unchanged.
- TX-R carries the `warning:` prefix; Brian's sentence follows it unchanged.
- TX-T's `<thing>` and `<main>` substitutions for tools other than mr.
- TX-N-MT (mt has no `--network`; its remedy names `--bitcoin-cli`).
- TX-V wording (verify-bundle).
- mnemonic-gui: banner, button, dialog wording from the study; paste clears
  the field.
- mr-gui sets aside a stray plate silently when its seat is otherwise
  filled (it shows no CLI warnings).
- SH-S, SH-K, SH-L, SH-RT wording.
- The `--accept-coin-type-mismatch` flag given with nothing to override:
  no effect, no line.
- Fold r1: TX-H4P's wording (the two review slices' texts merged: s2's
  first sentence, s1's remedy); N13 covers `[fingerprint/path]xpub` records
  as plates (the `verify --keys` records row is gone); `convert --from
  entropy=|seedqr=` join N6's seed list beside `ms1=` (§1: a seed in any
  form typed now); fixture `fx/crs.sh` for R8c.4.
- Fold r2: the merged round-2 table (M01-M55; the design rows ruled there)
  applied; new fixtures `fx/d_t1.txt`, `fx/d_x0.txt`, `fx/env_t.json`,
  `fx/env_x.json`, `fx/upx.sh`, `fx/sit_plan.txt`, `fx/sit_hb.txt`,
  `me/d_t_noorigin.txt`, `mt/sheb.sh` for the new RED rows.

## 11. FIX units with RED rows

Rows are run by the gate. Columns: row id; fixture directory under
`design/network-rule/`; stdin file (`-` none); command (bash; `$X` `$T` `$P`
`$T1` `$T2` `$X1` `$X2` `$X1c1` are the gate's public test keys and phrase,
`$W` a scratch directory, `mdargs`/`cos` its helpers); today's exit; an ERE
today's stdout matches (`-` none); an ERE today's stderr matches; an ERE
today's stderr does NOT match; the exit after the unit; the after leg
(binding on the unit's TRACE row). Every toolkit and md/mk unit updates the
toolkit manual (`docs/manual/src/40-cli-reference/`) in lockstep.

### Unit 0: the rule as code (mnemonic-toolkit, then each repo)
The helper grows from
{{mnemonic-toolkit@4553d2da:crates/mnemonic-toolkit/src/network.rs:94}} "pub(crate) fn assert_network_agrees("
to one function that takes the run network (given or default), the list of sources and the flags `reads_kept` (a kept artifact was read) and `secret_out` (N12 holds), and returns the ordered list of lines to print, each with its class (hard, soft, notice), text id (TX-H1 to TX-H5, TX-H4C, TX-H4P, TX-S, TX-N, TX-T, TX-R) and the two sources (N3, N4, N6, N8, N12); `nr0_vectors_tsv_every_row` checks the ids, class and exit of every `any` and `mnemonic` row, V42-V46 and V60 included; the machine line (N10); exit 8 (N9); the
flag (N7, parsed but unused until the arm units). No refusal text or exit code of an existing arm changes in unit 0: the new variant (exit 8) is added beside `NetworkMismatch` (2) and `ImportWalletNetworkClassMismatch` (1), which keep their codes until the unit that owns their arm; `nr0_exit_code_8_is_network_mismatch` asserts the new variant only. Owners of the §4.1 "stays" arms: export-wallet `--template`/`--slot` and both `--from-import-json` rows, unit 1 (R1.12, R1.13, R1.15); every other "stays" arm (convert `--xpub-prefix`, convert wif to xpub, `bundle --slot @N.xpub=`, bundle `--import-json`, restore `--search-address` (multisig template completion only; single-sig restore ignores the flag), silent-payment, verify-bundle watch-only slot, md decompose, `CosignerSpec`), unit 8b (Spec B), with one TRACE leg each in `nr8b_existing_refusals_exit_8_one_wording`; when no site is left, `NetworkMismatch` is deleted.
RED: the vector rows. TRACE rows: `nr0_vectors_tsv_every_row` (reads
vectors.tsv, runs each `any` and `mnemonic` row through the helper; RED:
the helper does not exist), `nr0_vectors_tsv_hash_pinned`,
`nr0_exit_code_8_is_network_mismatch`, `nr0_machine_line_shape`. Each later
repo's unit carries its own `nr0_vectors_tsv_every_row` for its rows. Each repo carries its own copy of the helper (no shared crate) and vendors vectors.tsv as tests/data/network_rule_vectors.tsv, pinned by a sha256 test against design/network-rule/vectors.tsv at the spec's rev; in the toolkit the helper is pub in network.rs so tests/cli_network_rule_vectors.rs can call it. Each part carries nr0_vectors_tsv_every_row, nr0_vectors_tsv_hash_pinned and nr0_machine_line_shape for its own rows. mr's part lands in unit 4 (Spec C): NETWORK_MISMATCH: u8 = 8 in mr-codec/src/app/exit_codes.rs, a Failure constructor with code 8, the N10 line printed by mr's main, and the whole TX-S text with N1's per-key coin read (unit 4 prints TX-S only behind a HARD line, so without its flag sentence, N8; the flag itself is parsed in 8c of Spec C). me's part lands in unit 6 (Spec D) (EXIT_NETWORK = 8, the N10 line on exit 8). md's and mk's land in 8b (Spec B) (mk: a new CliError variant whose exit_code is 8). mt's lands in 8e (Spec D) (the constant only: 8 is unreachable there).

### Unit 1: export-wallet `--descriptor` (P21; mnemonic-toolkit + manual)
| row | dir | stdin | command | today rc | today stdout has | today stderr has | today stderr lacks | after rc | after |
|---|---|---|---|---|---|---|---|---|---|
| R1.1 | fx | - | `mnemonic export-wallet --descriptor "wpkh([73c5da0a/84h/0h/0h]$X/<0;1>/*)" --network testnet --format bitcoin-core-addresses --count 1` | 0 | `tb1qcr8te4k` | - | `network mismatch` | 8 | TX-H1 (main key, `--network is testnet`); stdout empty |
| R1.2 | fx | - | `mnemonic export-wallet --descriptor "wpkh([73c5da0a/84h/1h/0h]$T/<0;1>/*)" --network mainnet --format bitcoin-core-addresses --count 1` | 0 | `bc1q6rz28mc` | - | `network mismatch` | 8 | TX-H1 (`--network is mainnet`); stdout empty |
| R1.3 | fx | - | `mnemonic export-wallet --descriptor "wpkh($T/<0;1>/*)" --format bitcoin-core-addresses --count 1` | 0 | `bc1q6rz28mc` | - | `network mismatch` | 8 | TX-H1 (`no --network given`) |
| R1.4 | fx | - | `mnemonic export-wallet --descriptor "wpkh([73c5da0a/84h/1h/0h]$X/<0;1>/*)" --network mainnet --format bitcoin-core-addresses --count 1` | 0 | `bc1qcr8te4k` | - | `coin type` | 8 | TX-S (key form) naming the flag; with `--accept-coin-type-mismatch`: 0, stdout `bc1qcr8te4k...`, stderr TX-S-OK |
| R1.5 | fx | - | `mnemonic export-wallet --descriptor "wpkh([73c5da0a/84h/0h/0h]$T/<0;1>/*)" --network testnet --format bitcoin-core-addresses --count 1` | 0 | `tb1q6rz28mc` | - | `coin type` | 8 | TX-S; with the flag 0, `tb1q6rz28mc...` |
| R1.6 | fx | - | `mnemonic export-wallet --descriptor "wpkh([73c5da0a/84h/1h/0h]$X/<0;1>/*)" --format bitcoin-core-addresses --count 1` | 0 | `bc1qcr8te4k` | - | `coin type` | 8 | TX-S (key form; key and run agree on mainnet) |
| R1.7 | fx | - | `mnemonic export-wallet --descriptor "wpkh([73c5da0a/84h/1h/0h]$T/<0;1>/*)" --network mainnet --format bip388 --count 1` | 0 | `tpubDC8msF` | - | `network mismatch` | 8 | TX-H1; stdout empty |
| R1.8 | fx | - | `mnemonic export-wallet --descriptor "wpkh($X/<0;1>/*)" --network regtest --format bitcoin-core-addresses --count 1` | 0 | `bcrt1qcr8te4k` | - | `network mismatch` | 8 | TX-H1 (main key, `--network is regtest`) |
| R1.9 | fx | - | `mnemonic export-wallet --descriptor "wpkh([73c5da0a/84h/1h/0h]$T/<0;1>/*)" --network signet --format bitcoin-core-addresses --count 1` | 0 | `tb1q6rz28mc` | - | `network mismatch` | 0 | unchanged (control) |
| R1.10 | fx | - | `mnemonic export-wallet --descriptor "wpkh([73c5da0a/84h/0h/0h]$X/<0;1>/*)" --format bitcoin-core-addresses --count 1` | 0 | `bc1qcr8te4k` | - | `nothing here names` | 0 | unchanged (control; the key is a source, so no TX-N) |
| R1.11 | fx | - | `mnemonic export-wallet --descriptor "wpkh([73c5da0a/84h/1h/0h]$T/<0;1>/*)" --network mainnet --format bitcoin-core --count 1` | 0 | `tpubDC8msF` | - | `network mismatch` | 8 | TX-H1 |
| R1.14 | fx | - | `mnemonic export-wallet --descriptor "wsh(sortedmulti(1,[73c5da0a/48h/0h/0h/2h]$X/<0;1>/*,[00000001/48h/1h/0h/2h]$T/<0;1>/*))" --format bitcoin-core-addresses --count 1` | 0 | `bc1qhptpsd3` | - | `network mismatch` | 8 | TX-H2 then TX-H1 (key @1) without its rerun sentence (N8) |
| R1.12 | fx | - | `mnemonic export-wallet --template bip84 --slot "@0.xpub=$T" --slot @0.fingerprint=00000000 --format electrum --output -` | 2 | - | `key encodes testnet` | - | 8 | TX-H1 (`<who>` = `--slot @0.xpub`, `this run is on mainnet (no --network given)`); stdout empty |
| R1.13 | fx | - | `mnemonic export-wallet --from-import-json env_t.json --network mainnet --format bitcoin-core-addresses --count 1` | 0 | `tb1q6rz28mc` | - | `network mismatch` | 8 | TX-H4 (`env_t.json says network testnet, but --network is mainnet`); stdout empty |
| R1.15 | fx | - | `mnemonic export-wallet --from-import-json env_x.json --network regtest --format bitcoin-core-addresses --count 1` | 0 | `bc1qcr8te4k` | - | `network mismatch` | 8 | TX-H4 (`env_x.json says network mainnet, but --network is regtest`); stdout empty |
| R1.16 | fx | - | `mnemonic export-wallet --descriptor "wpkh(02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5)" --format bitcoin-core-addresses --count 1` | 0 | `bc1qq6hag6` | - | `names a network` | 0 | stdout unchanged; stderr has TX-N |
| R1.17 | fx | - | `mnemonic export-wallet --descriptor "wpkh(02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5)" --network signet --format bitcoin-core-addresses --count 1` | 0 | `tb1qq6hag6` | - | `a test network` | 0 | stdout unchanged; stderr has TX-T (`this descriptor names no network`) |
| R1.18 | fx | - | `mnemonic export-wallet --descriptor "wpkh([73c5da0a/84h/1h/0h]02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5)" --format bitcoin-core-addresses --count 1` | 0 | `bc1qq6hag6` | - | `coin type` | 8 | TX-S run form (`key @0's path m/84h/1h/0h`, `this run is on mainnet (no --network given)`, remedy `--network signet, --network testnet or --network regtest`); stdout empty |

TRACE rows: `nr1_export_wallet_main_key_on_testnet_refused` (R1.1),
`nr1_export_wallet_test_key_on_mainnet_refused` (R1.2, R1.7, R1.11 as legs
over the formats), `nr1_export_wallet_test_key_no_flag_refused` (R1.3),
`nr1_export_wallet_coin_type_soft_refused_and_overridden` (R1.4, R1.5,
R1.6), `nr1_export_wallet_main_key_on_regtest_refused` (R1.8),
`nr1_export_wallet_consistent_controls_unchanged` (R1.9, R1.10),
`nr1_export_wallet_mixed_keys_refused` (R1.14),
`nr1_export_wallet_slot_key_refused_exit_8` (R1.12),
`nr1_export_wallet_import_json_against_run_refused` (R1.13, R1.15; leg: no `--network` on `env_t.json`),
`nr1_export_wallet_raw_keys_no_source_tx_n_tx_t` (R1.16, R1.17),
`nr1_export_wallet_raw_key_coin_type_soft` (R1.18).

### Unit 7: build-descriptor (mnemonic-toolkit + manual; erratum E1)
| row | dir | stdin | command | today rc | today stdout has | today stderr has | today stderr lacks | after rc | after |
|---|---|---|---|---|---|---|---|---|---|
| R7.1 | fx | - | `mnemonic build-descriptor --archetype simple-timelocked-inheritance --key "$T1" --recovery-key "$T2" --older 144 --network mainnet` | 0 | `bc1qsgyzz` | `disagrees with descriptor keys` | - | 8 | TX-H1; stdout empty |
| R7.2 | fx | - | `mnemonic build-descriptor --archetype simple-timelocked-inheritance --key "$X1" --recovery-key "$X2" --older 144 --network testnet` | 0 | `tb1qj7hsd` | `disagrees with descriptor keys` | - | 8 | TX-H1 |
| R7.3 | fx | - | `mnemonic build-descriptor --archetype simple-timelocked-inheritance --key "$X1" --recovery-key "$T2" --older 144` | 0 | `bc1qpst0z` | - | `network` | 8 | TX-H2 then TX-H1 without its rerun sentence (N8) |
| R7.4 | fx | - | `mnemonic build-descriptor --archetype simple-timelocked-inheritance --key "$X1c1" --recovery-key "$X2" --older 144` | 0 | `bc1qj7hsd` | - | `coin type` | 8 | TX-S; with the flag 0 |
| R7.5 | fx | - | `mnemonic build-descriptor --archetype simple-timelocked-inheritance --key "$T1" --recovery-key "$T2" --older 144` | 0 | `tb1qsgyzz` | - | `network mismatch` | 8 | TX-H1 (`no --network given`) |
| R7.6 | fx | - | `mnemonic build-descriptor --archetype simple-timelocked-inheritance --key "$X1" --recovery-key "$X2" --older 144` | 0 | `bc1qj7hsd` | - | `network` | 0 | unchanged (control) |
| R7.7 | fx | - | `mnemonic build-descriptor --archetype simple-timelocked-inheritance --key "$T1" --recovery-key "$T2" --older 144 --json` | 0 | `description_template` | - | `network mismatch` | 8 | TX-H1 (`no --network given`); stdout empty |
| R7.8 | fx | - | `mnemonic build-descriptor --archetype simple-timelocked-inheritance --key "$T1" --recovery-key "$T2" --older 144 --format descriptor` | 0 | `wsh\(or_d\(pk\(` | - | `network mismatch` | 8 | TX-H1; stdout empty |
| R7.9 | fx | - | `mnemonic build-descriptor --archetype simple-timelocked-inheritance --key "$T1" --recovery-key "$T2" --older 144 --emit-spec` | 0 | `schema_version` | - | `network mismatch` | 8 | TX-H1; stdout empty |

TRACE rows: `nr7_build_descriptor_keys_against_run_refused` (R7.1, R7.2,
R7.5), `nr7_build_descriptor_mixed_keys_refused` (R7.3),
`nr7_build_descriptor_coin_type_soft` (R7.4),
`nr7_build_descriptor_consistent_unchanged` (R7.6). The check runs once, after the preset or --spec lowers and the sanity gate passes, before any output, in every output mode: the human view, --format descriptor, --format bip388, --json and --emit-spec (its help sentence "--network is accepted and ignored" becomes "--network is accepted and checked against the keys"). On exit 8 stdout is empty in every mode: --json prints no {diagnostics} envelope for a network refusal (N5). key @i is the key's index in the policy's BIP-388 keys_info (--key keys first, then --recovery-key, then --final-key; for a --spec, the tree's keys in tree order). --accept-coin-type-mismatch is added to build-descriptor and to its GUI schema mirror. The "disagrees with descriptor keys" warning and infer_descriptor_network_kind are deleted. TRACE: nr7_build_descriptor_every_output_mode_refused (R7.7-R7.9).

### Unit 8: the rest (one PR each, in this order)

Units 8a and 8b are in SPEC_network_rule_tk_rest.md (Spec B); 8c and 8g in SPEC_network_rule_mr.md (Spec C); 8d, 8e, 8f and 8h in SPEC_network_rule_codecs_gui.md (Spec D). The order is 8a to 8h as lettered.

## 12. Build impact

1. Unit 0: mnemonic-toolkit `crates/mnemonic-toolkit/src/network.rs`,
   `error.rs` (new variant, exit 8, alphabetical), `main.rs` (machine
   line), `design/network-rule/vectors.tsv` (read by tests); tests:
   `network.rs` unit tests, new `tests/cli_network_rule_vectors.rs`;
   manual 41-mnemonic exit table. Each sibling repo's unit 0 part lands with
   its first arm unit (md and mk in 8b of Spec B, me in 6 of Spec D, mr in 4 of Spec C, mt in 8e of Spec D).
2. Unit 1: toolkit `cmd/export_wallet.rs`; tests that change:
   `tests/cli_network_fail_open.rs` (`e4d_descriptor_concrete_passthrough_ok`
   flips to a refusal), new `tests/cli_network_rule_export_wallet.rs`;
   manual 41-mnemonic `export-wallet`; mnemonic-gui schema mirror for the
   flag.
8. Unit 7: toolkit `cmd/build_descriptor.rs`
   ({{mnemonic-toolkit@4553d2da:crates/mnemonic-toolkit/src/cmd/build_descriptor.rs:431}} "disagrees with descriptor keys"),
   erratum E1; tests: `tests/cli_snet_build_descriptor_network_warn.rs`
   (`l1_tpub_with_network_mainnet_warns_not_rejects` and
   `l1_tpub_network_omitted_infers_testnet_preview` reverse); manual
   41-mnemonic `build-descriptor`.

## 13. Gate

`scripts/spec-network-rule-gate.sh` (run from the repo; `--no-red` skips the
binaries). It does NOT cover: the "after" legs (no code yet); text quality
or whether a decision is right; completeness of the per-arm table; any GUI
or SeedHammer behaviour; the mt testnet4 row against a real node; protocol
facts beyond the cites; a cite quote that also matches with another
meaning; exit tables written as plain numbers outside the toolkit manual's
CLI pages.
