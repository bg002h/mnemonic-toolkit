# SPEC — `mnemonic convert --phrase-out-dir`: save seed words to an owner-only file

Status: DRAFT; Brian's rulings of 2026-11-28 folded (§3, §6); R0 round 1 (1C/8I/12m, `design/agent-reports/phrase-out-dir-spec-R0.md`) folded. Not implemented.
Baseline: mnemonic-toolkit `master` at `d72a18b`.

## 1. Why

Brian, 2026-11-28: *"We have intentionally excluded our utilities from creating
entropy for seed phrases. But writing our words to a 0600 file would be useful.
Use the hash of the words in the name as well as put the date in the file name."*

Today a user who wants their words in a file redirects `convert`'s stdout. That
has three defects, all measured on `d72a18b` with the all-zero public test
entropy:

1. The file holds `phrase: abandon … about`. The label is a 13th token, so
   `mnemonic bundle --slot @0.phrase=- < seed.txt` refuses it ("BIP-39 word count
   13 invalid"). The user must strip it with `sed`.
2. The file's mode is whatever the umask gives (usually `0644`). Nothing warns.
3. The user picks the name, so nothing ties the file to the wallet it holds.

mnemonic-gui is about to load secret fields from a file (draft PR 64). This
gives users a correct file to load.

**Out of scope, by Brian's ruling:** generating entropy. This verb only writes
words the user already has (typed, from their own entropy, from an ms1 card,
from `final-word`, …). It never draws randomness.

## 2. Surface

Two new options on `convert`:

```
mnemonic convert --from <node>=<value|-> --to phrase [--to …] --phrase-out-dir <DIR> [--name-with-passphrase-fp]
```

| flag | meaning |
|---|---|
| `--name-with-passphrase-fp` | clap `requires = "phrase_out_dir"`. Adds the passphrase-applied fingerprint to the file name (§3). Needs a non-empty passphrase (§2.4). Its help says it gives anyone holding the file an offline passphrase checker (§6 C2). |
| `--phrase-out-dir <DIR>` | Write the `phrase` output to a new owner-only file in `DIR` instead of stdout, named by §3. Requires `phrase` among the `--to` targets. `DIR` must already exist and is never created. |

### 2.1 Inputs

Every input that yields a BIP-39 phrase: `phrase`, `seedqr`, `entropy`, `ms1`.
An Electrum phrase is not BIP-39 and has no `phrase` output.

**`phrase` → `phrase` (R0 C-1).** Today `(Phrase, Phrase)` is refused as the
"identity barrier" (`convert.rs:681-734`, exit 2). It is admitted **only when
`--phrase-out-dir` is present**: the phrase is parsed and re-rendered from its
entropy (canonical spelling, single spaces), which is the point of saving a
typed phrase. Without the flag the refusal stays byte-identical. RED rows cover
both sides.

**English/French ambiguity (R0 I-6).** `Mnemonic::to_entropy()`
(`convert.rs:1358-1365`) re-detects the language and panics on a phrase made
only of words English and French share. ms-cli fixed this as F-579
(`entropy_in_language`). The build ports that fix to the toolkit's `phrase` and
`seedqr` arms first (Rust-primary convergence; files a toolkit FOLLOWUP that it
closes), since this feature makes both routes headline ones.

**Auto-repair (R0 m-11).** When an ms1 primary short-circuits to the repair path
(exit 5), no file is written. A repaired string is not a verified seed.

### 2.2 Order of effects (R0 I-5)

1. Parse and convert everything; compute the file name and bytes.
2. Perform the write (or reach the "already saved" verdict, §4). Any failure
   here exits before a single stdout byte is written.
3. Print the other `--to` targets in target order, then advisories.

`--to phrase` given more than once writes one file and prints nothing for any
of them.

### 2.3 What is printed

- The `phrase` target is NOT printed to stdout. Other targets print as today.
- stderr: `saved phrase to <path> (fingerprint <fp>, without passphrase)`, or
  `already saved: <path>`.
- `--json` (additive v1 extension, `schema_version` stays `"1"`): the phrase
  entry is `{"node":"phrase","path":"<path>","already_saved":<bool>}`, with
  `value` omitted, only under `--phrase-out-dir`. Every other shape is
  unchanged. Recorded in the manual and CHANGELOG as a v1 extension.
- The stdout private-material warning is computed from what is emitted
  (verified by R0), so it is silent when the phrase was the only secret output.
- `--group-size`/`--separator` are accepted. They only ever group ms1/mk1
  (`convert.rs:1268-1275`), so `--to phrase,ms1 --group-size 5` saves the
  words and prints a grouped card. The file is never grouped (R0 I-4).

### 2.4 Passphrase and the warning (R0 I-2)

- Without `--name-with-passphrase-fp`, a passphrase plays no part in the file
  or its name. Where today's `edge_uses_pbkdf2` warning
  (`convert.rs:1047-1058`) says `--passphrase ignored on this edge`, it still
  says so.
- With the flag, the passphrase is used for `<fp-pass>` and that warning is
  suppressed. The flag with an absent or empty passphrase is refused (§2.6).
- Readback with a passphrase uses `@env:` (`bundle` allows one stdin, so
  `--slot @0.phrase=-` and `--passphrase-stdin` cannot be combined).

### 2.5 Refusals and exit codes (R0 I-7)

| refusal | error | exit |
|---|---|---|
| `--name-with-passphrase-fp` without `--phrase-out-dir` | clap `requires` | 64 |
| `--phrase-out-dir` without `phrase` in `--to` | `BadInput` | 1 |
| `--name-with-passphrase-fp` with no or empty passphrase | `BadInput` | 1 |
| `DIR` missing, not a directory, or `-` | `BadInput` naming the flag | 1 |
| an entry already at the name that is not the same bytes (§4) | new `ToolkitError::PhraseFileExists` (alphabetical placement) | 1 |
| any other I/O failure while writing | `Io` | 1 |
| `phrase`→`phrase` without `--phrase-out-dir` | existing `ConvertRefusal` (unchanged) | 2 |

If the summable-bit exit-code plan lands first, these move with it; naming the
variant keeps that one decision.

## 3. File name

```
seed-<YYYY-MM-DD>-<fp>[-<fp-pass>][-<language>].txt
```

- `<YYYY-MM-DD>`: today's date in the machine's **local time zone** (Brian,
  2026-11-28: *"use local time when available"*). On Unix: `libc::time` +
  `localtime_r` (`libc` is already a direct dependency; `mnemonic` is
  single-threaded and no non-test code calls `setenv`, so this is sound). On
  Windows, or if `localtime_r` fails: the UTC date by civil-from-days
  arithmetic, and stderr says `note: local time zone unavailable; file date is
  UTC`. No new dependency (R0 m-1). Tests inject the date through a seam.
- `<fp>`: `master_fingerprint(PBKDF2(words written to the file, ""))`, 8
  lowercase hex, computed with the same NFKD and derivation function that
  `--to fingerprint` uses, so the two agree when no passphrase is given. It
  needs no `--template` and no `--network` (a master fingerprint depends on
  neither) and is computed for every input, the `ms1` arm included, which
  derives nothing today (R0 I-2). This is "the hash of the words" (§6 C1).
- `-<fp-pass>` (Brian: *"if user supplies passphrase use xfp for both raw seed
  and with passphrase applied in filename"*, then *"Behind a flag"*): only with
  `--name-with-passphrase-fp`; the same function with the resolved passphrase.
  Raw first, passphrase second. The passphrase never appears in the file or the
  name.
- `-<language>`: present when the words written are not English. The language
  is the one the words are actually in: for an ms1 `Mnem` card, the card's wire
  language (which wins over `--language`, `convert.rs:1742-1757`); otherwise
  `--language` (R0 I-3). Spelled as the toolkit's `--language` value
  (`japanese`, `spanish`, `simplifiedchinese`, …), so it can be passed back to
  `mnemonic` as is. ms-cli spells the Chinese lists `chinese-simplified` /
  `chinese-traditional`; the manual says so next to the readback example.

Examples (public test seed, all-zero entropy): `seed-2026-11-28-73c5da0a.txt`;
with the flag and a passphrase: `seed-2026-11-28-73c5da0a-<fp-pass>.txt`.
(73c5da0a verified by R0 against the BIP-32 definition.)

Date first so a directory listing sorts by date.

## 4. File contents and writing

- Contents: the words as rust-bip39 renders them (ASCII space between words,
  Japanese included; both readers NFKD-normalise and split on whitespace, so do
  not change it to U+3000, R0 m-12), then one `\n`. No other bytes.
- Buffers holding the words or a compared file's bytes are
  `Zeroizing<Vec<u8>>` and mlock-pinned like `primary_value`
  (`convert.rs:957-966`) (R0 m-10).
- **Never replaces anything (R0 I-1).** `mnemonic_io_lib::write::write_private`
  replaces and follows symlinks, so it is NOT used. Instead, in the toolkit:
  1. create `DIR/.<name>.<pid>.tmp` with `create_new(true)` and mode `0o600`,
     write, fsync;
  2. `fs::hard_link(temp, DIR/<name>)` (atomic; fails with EEXIST on any
     existing entry and does not follow a symlink there), then unlink the temp
     and fsync `DIR`;
  3. where hard links are unsupported (FAT/exFAT sticks, some SMB/FUSE mounts):
     unlink the temp and open `DIR/<name>` with `create_new(true)` and `0o600`
     directly (`O_CREAT|O_EXCL`, which also refuses a dangling symlink).
  A follow-up hoists this into mnemonic-io-lib as `write_private_new` (a
  cross-repo change with a publish; not in this build).
- **An entry already at the name** (EEXIST, examined with `symlink_metadata`,
  never `metadata`/`exists`):
  - a regular, non-symlink file of at most 1 KiB whose bytes equal the new
    bytes → `already saved`, exit 0, the file untouched (no chmod, no write);
  - anything else (different bytes, a symlink, FIFO, device, directory, an
    oversized file) → `PhraseFileExists`, nothing written. A FIFO is never
    opened.
- A kill between steps 1 and 2 leaves `.<name>.<pid>.tmp` (0600, holding the
  words). The manual says so; the name carries the fingerprint, so it can be
  found (R0 m-3).
- `DIR` edges (R0 m-4): a symlink to a directory is followed (stated in help);
  `DIR` group- or world-writable → stderr warning (others could delete the
  file or plant names); `DIR` under `/dev/fd` needs no special case (the joined
  name is not numeric).
- Saving the same seed on another day writes a second file. If a
  `seed-*-<fp>*.txt` already exists in `DIR`, stderr notes it (R0 m-9).
- Non-Unix (Windows): no mode bits. The file is written and stderr says
  `warning: file permissions not restricted on this platform; protect <path>`
  (ruled, §6 C5).

## 5. Examples

```sh
# your own entropy (dice, a hardware RNG), 64 hex chars → 24 words
mnemonic convert --from entropy=- --to phrase --phrase-out-dir ~/seeds < my-entropy.hex
# saved phrase to /home/you/seeds/seed-2026-11-28-xxxxxxxx.txt (fingerprint xxxxxxxx, without passphrase)

# words you typed, re-saved in canonical form
mnemonic convert --from phrase=- --to phrase --phrase-out-dir ~/seeds

# an ms1 card back to words, and the card printed grouped for engraving
mnemonic convert --from ms1=- --to phrase --to ms1 --group-size 5 --phrase-out-dir ~/seeds

# with the passphrase fingerprint in the name
PP='…' mnemonic convert --from phrase=- --to phrase --phrase-out-dir ~/seeds \
  --passphrase @env:PP --name-with-passphrase-fp

# reading it back
mnemonic bundle --network mainnet --template bip84 --slot @0.phrase=- < ~/seeds/seed-2026-11-28-xxxxxxxx.txt
PP='…' mnemonic bundle --network mainnet --template bip84 --passphrase @env:PP \
  --slot @0.phrase=- < ~/seeds/seed-2026-11-28-xxxxxxxx-yyyyyyyy.txt
mnemonic bundle … --language japanese --slot @0.phrase=- < ~/seeds/seed-2026-11-28-xxxxxxxx-japanese.txt
ms encode --in ~/seeds/seed-2026-11-28-xxxxxxxx.txt
```

## 6. Concerns for Brian

**C1. What a hash in the file name gives away.** A file name leaks far more
easily than file contents: `ls`, shell history, file-manager "recents", desktop
search indexes, cloud-sync and backup metadata, screenshots of a folder. Two
consequences:

- *It links the file to a wallet.* With the master fingerprint, anyone who sees
  the name can match it to the `[73c5da0a/84'/0'/0']` origin in a descriptor,
  PSBT, md1/mk1 card or coordinator export. That tells them which file to steal.
  Any content hash links the same way once they hold the seed; the fingerprint
  links it *before* they hold it.
- *It is an offline guess-checker.* Someone who knows most of the words (a
  photo with a few words unreadable, a shoulder-surfed partial) can test guesses
  against the name without touching the chain, even for an unfunded wallet. A
  full SHA-256 of the words is the worst case here: fast, and with no false
  positives. The 32-bit fingerprint is better on both counts: each guess costs
  PBKDF2 (2048 rounds of HMAC-SHA512), and 32 bits gives false matches once the
  guess space passes about 4 billion. It adds nothing a descriptor does not
  already publish. That is why this spec uses it.

  If the link-to-wallet point outweighs the convenience, the alternative is a
  name with only the date and a counter (`seed-2026-11-28-1.txt`), and the
  fingerprint printed on stderr for the user to note down. Recommended: keep the
  fingerprint; it is the identifier every other tool here already shows.

**C2. RULED 2026-11-28: both fingerprints when a passphrase is given.** This
fixes the mismatch (a passphrase wallet's file now carries the fingerprint its
descriptors show). One new risk follows, and the build must state it in `--help`:
**the second fingerprint lets anyone who holds the file test passphrase guesses
offline.** The file holds the words, the name holds the target, so a thief needs
nothing else: no chain data, no xpub. The passphrase's job is to protect a stolen
seed, and this name hands the thief a checker for it. Cost per guess is one
PBKDF2 (2048 rounds); 32 bits means a passphrase weaker than about 32 bits is
found exactly. (An xpub or descriptor stored next to the file already gives the
same check, and a used wallet's addresses give a slower one.) Options: (a) as
ruled; (b) keep the second fingerprint off by default and add it with
`--name-with-passphrase-fp`; (c) print it on stderr only. Recommended: (b), but
RULED (b), "Behind a flag": `--name-with-passphrase-fp`.

**C3. RULED 2026-11-28: local date when available, else UTC with a note.**
The date still records roughly when the user handled the seed, which is metadata.

**C4. A 0600 file is still a plaintext seed on disk.** It survives in backups,
swap-free copies, and filesystem journals, and is readable by root and by any
process running as the user. The help text says so and points at `ms encode` /
engraving as the durable form. Encryption (`age`) is out of scope here.

**C5. RULED 2026-11-28: Windows writes with a warning, not a refusal.**

## 7. Lockstep (owed by the build PR)

- Toolkit manual `docs/manual/src/40-cli-reference/41-mnemonic.md` (`convert`
  flag table from :1120; both flags; the JSON v1 extension; the leftover-temp
  note; the ms-cli Chinese spelling), and `make -C docs/manual lint`.
- GUI manual `docs/manual-gui/src/40-mnemonic/44-convert.md` (flag index :33-48).
- mnemonic-gui `src/schema/mnemonic.rs` `CONVERT_FLAGS` gains both flags;
  `--phrase-out-dir` is `Path { stdio_sentinel: false }`, as for `gen-man --out`
  (no new `FlagKind`). `src/form/conditional.rs` (`convert`, :460-471) shows
  `--name-with-passphrase-fp` only when `--phrase-out-dir` is set. Then the
  pinned-upstream bump and `cargo test --test schema_mirror`.
- CHANGELOG and version bump; toolkit FOLLOWUPS for F-579 convergence (closed by
  the build) and for hoisting `write_private_new` into mnemonic-io-lib.

## 8. Tests (RED first)

- name: `seed-<date>-<fp>.txt` for the all-zero test entropy, date injected;
  no `--template` needed; same name from `entropy`, `phrase`, `seedqr` and `ms1`
  inputs of the same seed;
- language suffix: none for English; `-japanese` for Japanese entropy with
  `--language japanese`; for a Japanese ms1 Mnem card, `-japanese` with no
  `--language` and also with `--language spanish`; fingerprint follows the
  written words;
- passphrase: without the flag → one fingerprint and the "ignored" warning
  present; with it → both, raw first, warning absent; flag with empty or absent
  passphrase → exit 1; flag without `--phrase-out-dir` → exit 64; the passphrase
  never appears in the file or name;
- `phrase`→`phrase`: with `--phrase-out-dir` writes the canonical words; without
  it the existing refusal, byte-identical, exit 2;
- local time unavailable → UTC date and the note (through the seam);
- contents: bare words + one `\n`; round-trips through `bundle --slot @0.phrase=-`
  (English, and Japanese with `--language japanese`); ms-cli's reader emulated in
  a unit test (split on whitespace, join with one space, `parse_in`), since no
  toolkit test runs `ms`;
- mode `0o600` under `umask 000`;
- existing identical file → exit 0, untouched (mtime and mode unchanged);
  differing file, symlink (dangling and live), FIFO and directory at the name →
  exit 1 `PhraseFileExists`, nothing written, FIFO never opened;
- hard-link-unsupported fallback path through a seam;
- `DIR` missing, a file, or `-` → exit 1, nothing created; world-writable `DIR`
  → warning;
- write failure → no stdout bytes at all (order of effects);
- `--to phrase,phrase` → one file; `--to phrase,ms1 --group-size 5` → file
  ungrouped, card grouped;
- stdout carries no word of the phrase; `--json` carries `path` and
  `already_saved`, no `value`, `schema_version` `"1"`;
- ms1 auto-repair exit 5 → no file;
- Windows warning: the platform decision is a pure function tested on Linux
  (CI is Ubuntu-only), the real Windows run is a manual row.
