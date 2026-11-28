# SPEC — `mnemonic convert --phrase-out-dir`: save seed words to an owner-only file

Status: DRAFT for Brian's review (2026-11-28). Not implemented. No R0 yet.
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

One new option on `convert`:

```
mnemonic convert --from <node>=<value|-> --to phrase [--to …] --phrase-out-dir <DIR>
```

| flag | meaning |
|---|---|
| `--phrase-out-dir <DIR>` | Write the `phrase` output to a new owner-only file in `DIR` instead of stdout, named by §3. Requires `phrase` among the `--to` targets; refused otherwise. `DIR` must already exist (it is never created). |

Behaviour:

- The `phrase` target is NOT printed to stdout. The other `--to` targets print
  as they do today. (That keeps the words off the terminal, which is the point.)
- On success, stderr gets one line: `saved phrase to <DIR>/<name>`. Under
  `--json`, the `phrase` entry's `value` is replaced by `"path": "<DIR>/<name>"`.
- The stdout private-material warning still fires when another secret target is
  printed, and is silent when the phrase was the only secret output.
- Every input that yields a BIP-39 phrase works: `phrase`, `seedqr`,
  `entropy`, `ms1`. An Electrum phrase is not BIP-39 and has no `phrase`
  output, so it never reaches this path.

## 3. File name

```
seed-<YYYY-MM-DD>-<fp>[-<fp-pass>][-<language>].txt
```

- `<YYYY-MM-DD>`: today's date, **UTC** (see §6, concern C3).
- `<fp>`: the BIP-32 **master fingerprint** of the phrase **with an empty
  passphrase**, 8 lowercase hex characters, the same value `--to fingerprint`
  prints. This is "the hash of the words" (see §6, concern C1 for why not a
  plain SHA-256).
- `-<fp-pass>` (Brian, 2026-11-28: *"if user supplies passphrase use xfp for
  both raw seed and with passphrase applied in filename"*): present only when
  the user supplies a non-empty BIP-39 passphrase through `convert`'s existing
  `--passphrase` / `--passphrase-stdin` / `@env:VAR` channels. It is the master
  fingerprint with that passphrase applied, 8 lowercase hex. Raw seed first,
  passphrase second. The passphrase itself is never written to the file or the
  name. An empty passphrase adds no second fingerprint.
- `-<language>`: present only when the wordlist is not English, using the
  `--language` value (`-japanese`, `-spanish`, …), so the reader knows which
  `--language` to pass back.

Examples (public test seed, all-zero entropy): `seed-2026-11-28-73c5da0a.txt`;
with a passphrase: `seed-2026-11-28-73c5da0a-<fp-pass>.txt`.

Date first so a directory listing sorts by date.

## 4. File contents and mode

- Contents: the words exactly as `convert --to phrase` renders them today,
  without the `phrase: ` label, followed by one `\n`. No other bytes. That is
  what `--slot @N.phrase=- < FILE`, `ms encode --in FILE` and the GUI's
  "Load from file…" all read.
- `--group-size`/`--separator` do not apply to the file (they would break
  re-reading); passing them with `--phrase-out-dir` is refused.
- Written with `mnemonic_io_lib::write_private` (already a dependency, `0.1.3`):
  temp file created `0o600`, synced, renamed into place, directory synced.
- **Never overwrites.** If the name already exists:
  - same bytes → stderr `already saved: <path>`, exit 0, nothing written;
  - different bytes (a fingerprint collision or a hand-edited file) → refuse,
    exit non-zero, nothing written.
  The check-then-rename window is acceptable because the name is derived from
  the contents: a racing writer of the same name writes the same bytes.
- Non-Unix (Windows): no mode bits exist. The file is written and stderr says
  `warning: file permissions not restricted on this platform; protect <path>`.
  (Alternative for Brian: refuse on Windows. See §6, C5.)

## 5. Examples

```sh
# your own entropy (dice, a hardware RNG), 64 hex chars → 24 words
mnemonic convert --from entropy=- --to phrase --phrase-out-dir ~/seeds < my-entropy.hex
# saved phrase to /home/you/seeds/seed-2026-11-28-xxxxxxxx.txt

# words you typed, re-saved in canonical form
mnemonic convert --from phrase=- --to phrase --phrase-out-dir ~/seeds

# an ms1 card back to words
mnemonic convert --from ms1=- --to phrase --phrase-out-dir ~/seeds

# reading it back
mnemonic bundle --network mainnet --template bip84 --slot @0.phrase=- < ~/seeds/seed-2026-11-28-xxxxxxxx.txt
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
the spec follows the ruling (a) until Brian says otherwise.

**C3. Which date.** UTC, because the toolkit has no time-zone dependency and the
result must not depend on the machine's TZ. Near midnight the date can be "wrong"
by a day for the user. The date also records roughly when the user handled the
seed, which is metadata some users would not want. Alternative: local date
(adds a dependency), or a `--no-date` switch.

**C4. A 0600 file is still a plaintext seed on disk.** It survives in backups,
swap-free copies, and filesystem journals, and is readable by root and by any
process running as the user. The help text says so and points at `ms encode` /
engraving as the durable form. Encryption (`age`) is out of scope here.

**C5. Windows.** No `0600`. The spec writes with a warning; refusing is the
stricter choice.

## 7. Lockstep (owed by the build PR)

- Toolkit manual `docs/manual/src/40-cli-reference/41-mnemonic.md` (`convert`
  flag table) and GUI manual `docs/manual-gui`.
- mnemonic-gui `src/schema/mnemonic.rs` gains `--phrase-out-dir` (a directory
  picker), plus its pinned-upstream bump.
- `gui-schema` output picks the flag up automatically.

## 8. Tests (RED first)

- name: `seed-<date>-<fp>.txt` for the all-zero test entropy, with the date
  injected through a test-only seam (no wall clock in tests);
- non-English name suffix; English has none;
- contents: bare words + one `\n`; round-trips through `bundle --slot @0.phrase=-`
  and `ms encode --in`;
- mode `0o600` regardless of a permissive umask (`umask 022`);
- existing identical file → exit 0, untouched (mtime unchanged); differing file
  → refusal, untouched;
- `--phrase-out-dir` without `--to phrase` refused; with `--group-size` refused;
  missing `DIR` refused and not created;
- stdout carries no word of the phrase; other `--to` targets unchanged;
- `--json` shape carries `path`, not `value`.
