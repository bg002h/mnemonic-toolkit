# `mr-sitting` (mr-sitting) reference

The offline program of a Refugium sitting: `sit` makes or reads a plan's seeds and checks their plates, `verify` checks one plate against the hand-back, and `root` and `child` do the jobs of `mr root` and `mr child`. Linux only.

---

## `mr-sitting sit`

### Synopsis

```sh
mr-sitting sit --plan <I1> --hand-back <I2> [--closing-in <I8> --closing-out <I9>]
```

### Flags

| Flag | Purpose |
|---|---|
| `--plan <PLAN>` | The plan file (I1) |
| `--hand-back <HAND_BACK>` | The hand-back to write (I2); it must not exist |
| `--closing-in <CLOSING_IN>` | The closing check input (I8); needs `--closing-out` |
| `--closing-out <CLOSING_OUT>` | The closing check result to write (I9); needs `--closing-in` |
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

> how are this seed's random bits drawn? 1 dice, hand-checkable (selected); 2 dice, hashed; 3 coin flips; 4 Entropia pills. Press Enter for 1, or type 2, 3 or 4:

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

> passphrase. Only \<person\> types it: the passphrase this seed already uses, or nothing if it has none. It is not shown:

**ST-PP-03:**

> the same passphrase again:

**ST-HOT-09:**

> has seed \<L\> made a phone wallet for another card before? (y/n) n: this card records, as \<person\>'s word, that it never has; y: no phone wallet in this sitting

**ST-HOT-06:**

> type \<person\>'s phone wallet seed: the 12 or 24 words of a BIP-85 child of seed \<L\> (index 0 to 15), on one line or several, then an empty line; an empty line alone makes no phone wallet. The card records it with no passphrase of its own:

**ST-HOT-02:**

> \<the error's text\> Try \<12 or 24\> words? (y/n)

**ST-PL-01:**

> engrave plate \<TITLE\> now from the plates file (its title: \<TITLE\>), or write these words and the ms1 string on paper. Press Enter when the plate is done

**ST-PL-01b:**

> no plates file: engrave plate \<TITLE\> from this screen (its title: \<TITLE\>), or write these words and the ms1 string on paper. Press Enter when the plate is done

**ST-PL-02:**

> did plate \<TITLE\> engrave correctly? y = yes, check it now; n = no, show its payload again

**ST-PL-03:**

> check plate \<TITLE\>: type it as engraved, from the steel or the paper, not from memory: its \<24 or 12\> words or its ms1 string, then an empty line:

**ST-PL-09:**

> type it again (a), engrave it again (e), or skip this check (s)?

**ST-PL-10:**

> skipping leaves plate \<TITLE\> unchecked: the hand-back marks \<the keys it covers\> verified no, and mr-gui warns until it is checked. To skip, type exactly: plate \<TITLE\> is not checked

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

**ST-CL-01:**

> phone wallet check, before any deposit: in mr-gui, make the card from the hand-back now. When mr-gui says the closing file is written, type ready; to end now and run this check later from seed \<L\>'s backup, type later:

**ST-CL-07:**

> the closing result could not be written to \<path\> (\<os error\>): fix the drive and press Enter to try again

**ST-END-01:**

> every plate is checked or skipped and the hand-back is written. Press Enter to end the session: the plates file is wiped and the screen cleared

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

## Exit codes

| Code | Meaning |
|---|---|
| 0 | `sit`: the session ended at ST-END-02. `verify`: exactly one block matched in full. `root`, `child`: as `mr root`, `mr child`. |
| 1 | A refusal of holder material or a check, or the holder stopped: `sit` stopped (ST-END-03, ST-END-04), a mainnet refusal not overridden (ST-NET-06), a brought-seed refusal that ends the session (ST-BR-07, ST-BR-09); `verify` mismatch (ST-V-04, ST-V-05, ST-V-06), or stopped (ST-V-09). |
| 2 | Usage and input files: clap errors; a refused I1, I2 or argument (ST-PLAN-…, ST-FILE-…, ST-HB-02, ST-START-02, ST-START-03, ST-START-04, ST-START-07, ST-START-08, ST-START-09, ST-START-11, ST-V-07, ST-V-08). Always before any secret is read. |
