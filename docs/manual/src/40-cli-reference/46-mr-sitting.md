# `mr-sitting` (mr-sitting) reference

<!-- cspell:ignore handback -->

The offline program of a Refugium sitting: `sit` makes or reads a plan's seeds and checks their plates, `verify` checks one plate against the hand-back, and `root` and `child` do the jobs of `mr root` and `mr child`, `practice` runs a practice sitting on signet from a built-in case, and `glossary` says what each term the sitting uses means. Linux only.

---

## `mr-sitting sit`

### Synopsis

```sh
mr-sitting sit --plan <I1> --hand-back <I2> [--closing-in <I8>] [--closing-out <I9>]
```

### Flags

| Flag | Purpose |
|---|---|
| `--plan <PLAN>` | The plan file (I1) |
| `--hand-back <HAND_BACK>` | The hand-back to write (I2); it must not exist |
| `--closing-in <CLOSING_IN>` | The closing check input (I8); needs `--closing-out`. Optional for every plan: without it, `refugium-closing-in-<plan-id>.txt` in the hand-back's folder |
| `--closing-out <CLOSING_OUT>` | The closing check result to write (I9); needs `--closing-in`. Optional for every plan: without it, `refugium-closing-out-<plan-id>.txt` in the hand-back's folder |
| `-h, --help` | Print help |

### What it asks

**ST-UI-01:**

> press Enter to go on

**ST-NET-02 and ST-NET-03:**

> this computer has a network (\<names\>): making mainnet keys here is refused. To go on anyway, type this sentence:
>
> I understand creating private key material on a networked computer is unwise and has an unnecessarily high probability of causing loss of funds

**ST-NET-04:**

> type the sentence above exactly to go on, or anything else to stop:

**ST-NET-05:**

> are you sure? The better way: run mr-sitting on the mr-sitting-stick, or on an offline computer with its network hardware removed. Type yes to go on here anyway, or anything else to stop:

**ST-BR-01:**

> type this seed's 12 or 24 words, or its ms1 string, on one line or several, then an empty line:

**ST-BR-08:**

> this seed has 12 words. only for a seed whose word 1 came at random; a chosen word 1 leaves 117 random bits: make a 24-word seed instead Was word 1 of this seed random? (y/n)

**ST-M-01:**

> how are this seed's random bits drawn? 1 dice, hand-checkable (selected): more rolls, and each word can be checked by hand from its rolls; 2 dice, hashed: fewer rolls, mixed by SHA-256, so no word can be checked by hand; 3 coin flips; 4 Entropia pills. Press Enter for 1, or type 2, 3 or 4:

**ST-D-01:** (line mode adds " Then an empty line:")

> dice, hand-checkable: roll one die at a time and type each roll (1 to 6) as it lands, until it says stop. 1 gives 00, 2 gives 01, 3 gives 10, 4 gives 11, 5 gives 0, 6 gives 1; \<N\> bits are needed, about \<142 or 151\> rolls. A die that favours a number passes that bias straight into the bits.

**ST-D-09:**

> dice, hashed: type your dice rolls (1 to 6; at least 98, 99 recommended), on as many lines as you like, then an empty line to end:

**ST-D-11:**

> coin flips: type your 252 coin flips (H or T), on as many lines as you like, then an empty line to end:

**ST-D-13a:**

> Entropia pills: first, 7 coin flips (H or T), then an empty line:

**ST-D-13b:**

> then the 22 words that landed face up, one draw at a time, on as many lines as you like, then an empty line:

**ST-D-13c:**

> last, 3 coin flips (H or T), then an empty line:

**ST-PP-01:**

> passphrase. Only \<person\> types it; \<person\> will need it to sign and to check this seed's plate. Empty means none. It is not shown:

**ST-PP-02:**

> passphrase. Only \<person\> types it: exactly the passphrase this seed already uses, or nothing if it has none. A new passphrase here makes another wallet, not this seed's. It is not shown:

**ST-PP-03:**

> the same passphrase again:

**ST-HOT-09:**

> has seed \<L\> made a phone wallet for another card before? (y/n) n: this card records, as \<person\>'s word, that it never has; y: no phone wallet in this sitting

**ST-HOT-06:**

> type \<person\>'s phone wallet seed: the 12 or 24 words of a BIP-85 child of seed \<L\> (index 0 to 15), on one line or several, then an empty line; an empty line alone makes no phone wallet. The card records it with no passphrase of its own:

**ST-HOT-02:**

> \<the error's text\> Try \<12 or 24\> words? (y/n)

**ST-PL-01b:**

> make plate "\<TITLE\>" now: write these words and the ms1 string on paper, or engrave them on steel, from this screen, under the title "\<TITLE\>" (no plates file: \<why\>). Press Enter when the plate is done

**ST-PL-02:**

> is \<PLATE\> written or engraved correctly? y = yes, check it now; n = no, show it again

**ST-PL-03:**

> check \<PLATE\>: type it from the steel or the paper, not from memory: either its \<24 or 12\> words or its ms1 string (one of the two, not both), then an empty line. \<next\>

**ST-PL-09:**

> type it again (a), engrave it again (e), or skip this check (s)?

**ST-PL-10:**

> skipping leaves \<PLATE\> unchecked: the hand-back marks \<the keys it covers\> verified no, and mr-gui warns until it is checked. To skip, type exactly: plate \<TITLE\> is not checked

**ST-PL-12:**

> this seed was brought with its own backup. Engrave a new plate for it? (y/n)

**ST-PL-14:**

> check this seed's own backup: type it as you keep it, its words or its ms1 string, then an empty line:

**ST-PL-15:**

> \<ST-NAME\>: its passphrase, as \<person\> types it to sign (empty for none; not shown):

**ST-PL-16:**

> \<ST-NAME-04\>: its own passphrase, if it was given one (empty for none; not shown):

**ST-HB-W2:**

> the hand-back could not be written to \<path\> (\<os error\>): fix the drive and press Enter to try again

**ST-CL-07:**

> the closing result could not be written to \<path\> (\<os error\>): fix the drive and press Enter to try again

**ST-END-01:**

> the sitting is done: every plate is checked or its check skipped, and the hand-back is written to \<path\>. Press Enter to end: mr-sitting then wipes the plates file and the seeds in its memory, and clears this screen

## `mr-sitting verify`

### Synopsis

```sh
mr-sitting verify --plan <I1> --hand-back <I2> [--key <L> | --root <n> | --hot]
```

### Flags

| Flag | Purpose |
|---|---|
| `--plan <PLAN>` | The plan file (I1) |
| `--hand-back <HAND_BACK>` | The hand-back (I2) |
| `--key <L>` | Check seed `<L>` [possible values: A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P] |
| `--root <n>` | Check root `<n>` |
| `--hot` | Check the phone wallet seed |
| `-h, --help` | Print help |

### What it asks

**ST-V-01:**

> type the plate as recorded, from the steel or the paper: its 12 or 24 words or its ms1 string, then an empty line:

**ST-PL-15:**

> \<ST-NAME\>: its passphrase, as \<person\> types it to sign (empty for none; not shown):

**ST-PL-16:**

> \<ST-NAME-04\>: its own passphrase, if it was given one (empty for none; not shown):

## `mr-sitting root`

### Synopsis

```sh
mr-sitting root --network <net> --letters <L,…|none> --from coins|dice|pills|dice-hand
```

### Flags

| Flag | Purpose |
|---|---|
| `--network <NETWORK>` | Network: main, signet, testnet or regtest |
| `--letters <L,…>` | The letters this root's plate names, B to P, or `none` [possible values: B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, none] |
| `--from <FROM>` | Where the draws come from; they are read from stdin. `coins`: 237 coin flips; `dice`: Dice, hashed (at least 92 rolls); `pills`: 22 Entropia pill draws; `dice-hand`: Dice, hand-checkable (§9.2): 237 bits from the rolls, no hash |
| `-h, --help` | Print help (see a summary with `-h`) |

### What it asks

**ST-R-01:** (`--from dice-hand` only; its other prompts are `mr root`'s)

> type your dice rolls (1 to 6) as they land, on as many lines as you like, then an empty line to end: 1 gives 00, 2 gives 01, 3 gives 10, 4 gives 11, 5 gives 0, 6 gives 1; 237 bits are needed, about 142 rolls

## `mr-sitting child`

### Synopsis

```sh
mr-sitting child [--check] --network <net> [--letters <L,…>] [MR1_CARD]... [--card <mr1> …]
```

### Flags

| Flag | Purpose |
|---|---|
| `[MR1_CARD]...` | The card (its mr1 strings), to check the children against |
| `--network <NETWORK>` | Network: main, signet, testnet or regtest |
| `--check` | Check the root's children against the card's seats and print no words (needs a card) |
| `--letters <L,…>` | The letters to print or check, B to P, comma-separated (default: with a card, every letter of its seats but A and the hot seat's; without a card, the letters the plate names) [possible values: B, C, D, E, F, G, H, I, J, K, L, M, N, O, P] |
| `--card <mr1>` | With --check and a card: another card of this set's chain, one card per value (a chunked card's strings in one value, separated by spaces); repeat for each card |
| `-h, --help` | Print help |

`child` asks nothing of its own: its prompts are `mr child`'s.

## `mr-sitting practice`

### Synopsis

```sh
mr-sitting practice [<case>] [--dir <folder>]
```

### Flags

| Flag | Purpose |
|---|---|
| `[CASE]` | The case to run, as `mr-sitting practice` lists it; with no case, the cases are listed and the exit code is 0 |
| `--dir <DIR>` | The folder for the practice files: it must exist and hold none of them (default: a new folder `practice-<case>` here, or `practice-<case>-2`, `-3`, …, made mode 0700); needs a case |
| `-h, --help` | Print help |

`practice` runs on signet only, from a case built into the program; it reads and writes no secret of yours. It writes into its folder `refugium-plan-<plan-id>.txt` (the case's plan, at start), `refugium-handback-<plan-id>.txt`, `refugium-closing-in-<plan-id>.txt` and `refugium-closing-out-<plan-id>.txt`. The session is `sit`'s, with every answer pre-filled and the practice rules of the spec; a banner marks every screen. With no case, `practice` runs no hardening and takes no lock.

### What it prints

**ST-PR-03** (no case; the cases of this build):

```text
practice cases (run one with mr-sitting practice <case>):
  full-1: every kind of seed; passphrases on root 1, seeds B and C and erin's; two passphrases typed differently; a wrong passphrase at root 1's plate check; phone wallet from seed B
  full-2: every kind of seed; passphrases on seed E and dave's; a passphrase typed differently; a wrong passphrase at seed C's plate check; phone wallet from seed B, no plate
  full-3: every kind of seed; passphrases on root 1, seeds E and C and erin's; passphrases typed differently twice; phone wallet from seed C
  full-4: every kind of seed; passphrases on seed B and dave's; passphrases typed differently twice; phone wallet from seed C, no plate
  brought-phone-1: a brought seed with a passphrase and its phone wallet
  brought-phone-2: a brought seed with no passphrase and its phone wallet, no plate
  one-seed: one drawn seed, no phone wallet
```

**ST-PR-01:**

> PRACTICE on signet, case \<case\>: not a real wallet; values are public

**ST-PR-02:**

> PRACTICE on signet, case \<case\>, changed: differs from the stored files

**ST-PR-08:**

> practice files go into \<dir\>: the plan \<file\> now; the hand-back and the closing files as the sitting goes on

**ST-PR-09:**

> [practice: \<value\>; press Enter to take it]

**ST-PR-10:**

> practice: the stored closing file for case \<case\> is now at \<path\>, as mr-gui would write it from this hand-back

**ST-PR-11:**

> practice: this run changed, so no stored closing file fits its hand-back: press s to skip the check and end the practice (line mode: type skip)

**ST-PR-04:**

> mr-sitting: practice case \<case\> is on \<network\>: practice runs on signet only

**ST-PR-05:**

> mr-sitting: \<path\> already exists: practice writes its files into a folder that has none of them; give another --dir

**ST-PR-06:**

> mr-sitting: there is no practice case \<case\>: mr-sitting practice lists them

**ST-PR-07:**

> mr-sitting: the folder \<dir\> does not exist

**ST-PR-12:**

> mr-sitting: could not make \<path\> (\<os error\>): practice writes its files into a folder it can write; give another --dir

ST-PR-04 to ST-PR-07 and ST-PR-12 are exit 2; ST-PR-01 is the banner while the run matches the stored files and ST-PR-02 once it differs.

## `mr-sitting glossary`

### Synopsis

```sh
mr-sitting glossary [<term>]...
```

### Flags

| Flag | Purpose |
|---|---|
| `[TERM]...` | The term, in any case; a space reads as `-`. The words after `glossary` are one term, joined by one space |
| `-h, --help` | Print help |

`glossary` reads no secret and takes no hardening, lock or file. With no term it prints every entry, a header line, then one `<term>: <text>` per line, on stdout (exit 0); with a term it prints that entry alone (exit 0), or ST-G-02 on stderr (exit 2) if there is none. In the sitting, Ctrl-G opens the same entries as an overlay.

**ST-G-02:**

> mr-sitting: there is no glossary entry \<term\>: mr-sitting glossary lists them

The entries, as this build prints them:

```text
mr-sitting glossary: each term and what it means (mr-sitting glossary <term> shows one):
card: the wallet card (its mr1 strings): the wallet's public description, which mr-gui makes from the hand-back. It holds no secret: it records which seed fills each seat and how the wallet spends.
seat: a place on the wallet card for one key, numbered @0, @1 and so on. One seed may fill more than one seat.
seed: the secret a signer makes its keys from, written as 12 or 24 words. Each seed of a wallet card has its own letter, A to P, read from its words.
root: a seed kept only to make other seeds again: its plate, titled KEEP; DO NOT LOAD, regrows the seeds grown from it. A root is never loaded on a signer.
grown: a grown seed is made from a root by BIP-85, so it can be made again from the root's plate (and the root's passphrase, if any) with mr-sitting child.
brought: a brought seed is one its holder already has, typed into the sitting so that its keys go on the wallet card. Its letter is read from its words.
drawn: a drawn seed is a new seed made in the sitting from random bits its holder draws: dice, coin flips or Entropia pills.
hand-checkable: dice, hand-checkable: each roll becomes bits by a fixed table (1 gives 00, 2 gives 01, 3 gives 10, 4 gives 11, 5 gives 0, 6 gives 1), so each word can be checked by hand from its rolls. It needs more rolls than hashed dice.
hashed: dice, hashed: the rolls are mixed by SHA-256 into the seed's bits. Fewer rolls, but no word can be checked by hand.
coins: coin flips: each flip is one bit, H for 1 and T for 0.
pills: Entropia pills: pills drawn one at a time, each landing on a word that gives 11 bits, with a few coin flips for the bits left over.
passphrase: an extra secret typed with a seed's words. The same words with another passphrase make another wallet; empty means none. No plate holds it: its holder keeps it in memory or apart from every plate.
ms1: an ms1 string is the same seed as its words, written another way (codex32, BIP-93): either one alone restores the seed.
plate: a seed's words and its ms1 string, written on paper or engraved on steel under the plate's title. It restores that seed, so it is a secret.
plates-file: the plates file is a copy of the plate being made, for an engraving machine, kept only in this computer's RAM. Each plate replaces the one before, and it is wiped when the sitting ends.
phone-wallet: the phone wallet seed is made from its holder's seed by BIP-85, for a phone wallet app or mr-gui's hot wallet, with a letter no other seed of the wallet card has.
bip85: BIP-85 is a published way to make child seeds from a seed: the same seed always gives the same child at the same number.
hand-back: the hand-back is the file the sitting writes for mr-gui: each seed's public keys and fingerprints, and no secret. mr-gui makes the wallet card from it.
wallet-check: the wallet check (closing check): once mr-gui has made the wallet card, the sitting reads it back and works each seat's key out again from the seeds it holds, proving that each seat this sitting's seeds fill on the wallet card holds that seed's key. It comes before any deposit.
fingerprint: a short public number (8 hex digits) that names a seed with its passphrase: another passphrase gives another fingerprint.
practice: practice mode (mr-sitting practice) runs a stored sitting on signet with every answer filled in from published values. Nothing made there is a real wallet.
```

## The TUI

When `sit` or `practice` runs on a terminal of at least 80 columns and 24 rows, the sitting draws a screen with a step list. The keys:

| Key | On a step screen | In the step list | In an overlay |
|---|---|---|---|
| printable, Backspace, Enter | the answer (a passphrase does not echo) | Enter: go to the chosen step | `y` yes, `n` no (questions) |
| Esc | open the step list | back to the current screen | `n` (questions); close (glossary) |
| Up, Down | nothing | move the choice | glossary: scroll by one row |
| PgUp, PgDn | scroll the body | nothing | glossary: scroll by a page |
| Ctrl-G | the glossary overlay | the glossary overlay | nothing |
| Ctrl-Q, Ctrl-C | the quit question | the quit question | quit question: `n` |
| `s` | the wallet check's waiting screen, while no wallet file is found: skip | nothing | nothing |
| `c` | on the screen that keeps an answer: change the kept answer | nothing | nothing |
| `b` | the dice-hand words screen: toggle | nothing | nothing |

While a closing check's run is in flight, the waiting screen takes only Ctrl-Q, Ctrl-C, Ctrl-G, PgUp and PgDn. On the `Hand-back` screen before the hand-back is written, Enter writes it.

## The wallet check

Every plan ends with a wallet check, not only plans that name a closing file. The sitting waits for the Refugium wallet file that mr-gui writes, verifies it against every seed of the sitting, and writes the closing result. The texts below replace the earlier texts of the same numbers in the `sit` section above. A `<...>` is a fill: `<hot>` is `, and the phone wallet seed is seed <L>'s BIP-85 child` when the hand-back has a `hot` block and empty otherwise, `<closing>` is the words "and the closing result" and the path (after a space) when a closing result file exists, and `<pp>` is the words "and the passphrase" (after a space) when the seed has one.

**ST-W-02:**

> s = Skip: check later, retyping each seed's words from its plate

**ST-W-03:**

> waiting for the Refugium wallet file: \<path\>

**ST-W-04:**

> Refugium wallet file read: verifying it against every seed of this sitting

**ST-W-05:**

> each seat of the Refugium wallet, against this sitting's seeds:

**ST-W-06:**

> the phone wallet seat, against seed \<L\>'s BIP-85 child:

**ST-W-07:**

> Go back to mr-gui, read the hand-back again and press Mint again. This screen then asks you to verify again.

**ST-W-08:**

> ERROR! Seat @\<j\> does not hold \<name\>'s key!
>
> ERROR! Seat @\<j\> is filled by no seed of this sitting and by no key record typed in mr-gui!

**ST-W-12:**

> The wallet check did not run.

**ST-W-L1:**

> wallet check, before any deposit: in mr-gui, look the keys over and press Mint; when mr-gui says the Refugium wallet file is written, press Enter here to verify it, or type skip to check later:

**ST-W-L2:**

> press Enter to try the wallet check again, or type quit to stop:

**ST-CL-02:**

> wallet check skipped: make no deposit until it is done. With the wallet card (its mr1 strings), mr-gui lists the checks to run before any deposit: run them on an offline computer that has mr, typing each seed's words from its plate\<regrow\>; a seed with a passphrase is checked as that list says for it (mr verify's wallet card check takes no passphrase: its holder loads the seed with its passphrase on their Coldcard, checks the fingerprint on the Coldcard's main screen, then, from mr-gui, registers this Refugium wallet on the Coldcard with its Skip Checks setting off), never by typing the seed without its passphrase

**ST-CL-03:**

> wallet check passed: on the wallet card (its mr1 strings), each seat the hand-back gives a seed of this sitting holds that seed's key\<hot\>

**ST-CL-04:**

> mr-gui will not take deposits to this wallet.

**ST-END-01:**

> the sitting is done. It made or read: \<names\>. It wrote the hand-back \<path\> (public keys only, for mr-gui)\<closing\>. Wallet check: \<result\>. Press Enter to end: mr-sitting then wipes the seeds and passphrases from this computer's memory, deletes the plates file, and clears this screen

**ST-END-05:**

> SUCCESS! Your secrets can sign for this Refugium wallet: each seat marked proved below holds the key your seed makes.

**ST-END-07:**

> seat @\<j\>: proved: its key is made from the words\<pp\> of \<ST-NAME\>
>
> seat @\<j\>: proved: its key is made from the words of the phone wallet seed
>
> \<ST-NAME\>: fills no seat itself; seats \<seats\> above are proved from seeds this sitting grew from its words\<pp\>
>
> the phone wallet seed: proved: it is seed \<L\>'s BIP-85 child, as the wallet card records it

**ST-END-10:**

> Next: in mr-gui, finish the Refugium wallet and run its other checks before any deposit\<unchecked\>

**ST-END-12:**

> Press Enter to wipe your secrets from memory and end.

**ST-END-13:**

> seat @\<j\>: not proved here: only its holder's public key was given to mr-gui, so this sitting cannot check the seed behind it. Before any deposit, its holder checks it on their signer:

**ST-END-14:**

> 1. load the seed's words on your Coldcard, with its passphrase if it has one
>
> 2. check that the Coldcard's main screen shows the fingerprint \<fp\> (in either case): another fingerprint means the wrong words or the wrong passphrase
>
> 3. from mr-gui, register this Refugium wallet on the Coldcard, with its Skip Checks setting off (Skip Checks turns every check off): the Coldcard registers it only if the key it makes at m/\<path\> is in the wallet, and otherwise refuses it with a message such as "my key not included", "My key … missing in descriptor." or "… wrong pubkey"
>
> 4. make no deposit to this wallet until the Coldcard has registered it

**ST-END-15:**

> Registering proves only that this wallet holds the key your Coldcard makes at m/\<path\>; it does not check the other seats' keys, or that this key is the one at seat @\<j\>.

**ST-END-02:**

> session over: the seeds and passphrases are wiped from this computer's memory, and the plates file is deleted. Next, in order: (1) \<next\>; (2) keep every plate (steel or paper) that passed its check, each in its own safe place\<unchecked\>; (3) only once the wallet check has passed, destroy every other copy of words or of an ms1 string (a note, a print, a photo; not a plate), and every plate engraved again after a failed check\<wait\>. mr-sitting glossary explains each term

**ST-END-03:**

> mr-sitting: the sitting stopped before the hand-back was written: the seeds and passphrases are wiped from this computer's memory, the plates file is deleted, and nothing is kept. The seeds it made are never used (a brought seed stays as it was): destroy any plate or paper copy made of them, then run the plan again (new draws make new seeds)

## The plates file and its password

The sitting writes the plates file only encrypted, with a plates password it generates: a row `Plates password` before the first plate row, from 6 to 12 BIP-39 words. The file is wiped when the sitting ends.

**ST-W-01c:**

> Your seeds and their secret keys stay in this program's memory, and this program wipes its memory when it ends; the only file this program writes them to is the plates file, encrypted with your plates password, which it wipes when it ends. When you press Enter, this program (mr-sitting) gives mr-gui the PUBLIC keys made from your secrets, in the hand-back file: mr-gui needs them to make your Refugium wallet. After the Refugium wallet is made, this program confirms that your secrets can make the keys needed to sign the spending paths in your wallet. Press Enter to write the hand-back for mr-gui.

**ST-PW-01:**

> plates password. The plates file holds each plate's ms1 string while you make it, encrypted with this password; whatever reads the plates file during this session needs it. The password is words from the BIP-39 English list, drawn at random: you do not choose them, and you will write them down. It is not a seed passphrase and goes on no plate. The file is wiped when the session ends, so the words are needed only until then. How many words, from 6 to 12? More words are stronger. Press Enter for 6, or type 6 to 12:

**ST-PW-02:**

> type a number from 6 to 12, or press Enter for 6

**ST-PW-03:**

> how are the \<n\> words drawn? 0 this computer's random source (selected); 1 dice, hand-checkable; 2 dice, hashed; 3 coin flips; 4 Entropia pills. Press Enter for 0, or type 1, 2, 3 or 4:

**ST-PW-04:**

> type 0, 1, 2, 3 or 4, or press Enter for 0

**ST-PW-05:**

> Entropia pills: type the \<n\> words that landed face up, one draw at a time, on as many lines as you like, then an empty line:

**ST-PW-06:**

> plates password, \<n\> words: write them down now, in this order, and keep the paper until the session ends:

**ST-PW-07:**

> check: type word \<a\> and word \<b\> of the plates password, from your paper, on one line:

**ST-PW-08:**

> those are not words \<a\> and \<b\> of the plates password: here are the words again; write them down as shown

**ST-PW-09:**

> plates password set: the plates file is encrypted with it

**ST-PW-10:**

> the plates password is set for this session and cannot be changed

**ST-PW-11:**

> this computer's random source failed (\<os error\>): choose another way to draw the words

**ST-PL-01:**

> make plate "\<TITLE\>" now: write these words and the ms1 string on paper, or engrave them on steel, under the title "\<TITLE\>". This plate is also in the plates file, \<path\>, encrypted with your plates password (in this computer's RAM; wiped when the session ends). Press Enter when the plate is done

**ST-PL-21:**

> Payload Digest for the engraving machine: \<DIGEST\>

## Exit codes

| Code | Meaning |
|---|---|
| 0 | `sit`: the session ended at ST-END-02. `verify`: exactly one block matched in full. `root`, `child`: as `mr root`, `mr child`. |
| 1 | A refusal of holder material or a check, or the holder stopped: `sit` stopped (ST-END-03, ST-END-04), a mainnet refusal not overridden (ST-NET-06), a brought-seed refusal that ends the session (ST-BR-07, ST-BR-09); `verify` mismatch (ST-V-04, ST-V-05, ST-V-06), or stopped (ST-V-09). |
| 2 | Usage and input files: clap errors; a refused I1, I2 or argument (ST-PLAN-…, ST-FILE-…, ST-HB-02, ST-START-02, ST-START-03, ST-START-04, ST-START-07, ST-START-11, ST-V-07, ST-V-08); `practice`: ST-PR-04 to ST-PR-07 and ST-PR-12; `glossary`: ST-G-02. Always before any secret is read. |
