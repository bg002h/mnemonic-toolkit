# `mnemonic final-word` {#mnemonic-final-word}

Given an N-1 word BIP-39 partial phrase, emit the lexicographically
sorted set of wordlist entries that, when appended as the Nth word,
yield a phrase with a valid BIP-39 checksum. Output set size is a
function of N alone: 128 candidates for N=12, 64 for N=15, 32 for
N=18, 16 for N=21, 8 for N=24. Use cases include paper-backup
recovery (a smudged last word), manual seed generation (compute
the only-valid checksum-fixing word for a hand-rolled partial), and
phrase-typo verification (look up whether a written-down last word
appears in the candidate set for the first N-1 words).

:::danger
The worked example below uses
`abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon`
(11 words). This is a known-public test prefix; the canonical
zero-entropy 12-word vector
`abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about`
that completes it has been swept by chain watchers since 2017.
**Never engrave or fund** any phrase derived from this partial.
The 128 candidate words this subcommand emits are themselves
secret-class: any candidate paired with the partial yields a
valid seed phrase. Treat the candidate list as you would a seed
phrase. The §14 Defense-2 cold-node operational warning applies
in full to this subcommand and to every secret-bearing
invocation in the GUI.
:::

> **GUI form:** see [GUI Forms › mnemonic › final-word](#gui-form-mnemonic-final-word).

## Outline {#mnemonic-final-word-outline}

- [`--from`](#mnemonic-final-word-from) — N-1 word partial phrase, inline or via stdin
- [`--language`](#mnemonic-final-word-language) — BIP-39 wordlist (default `english`)
- [`--json-out`](#mnemonic-final-word-json-out) — write JSON envelope to PATH (side-effect; stdout still emits)

## `--from` {#mnemonic-final-word-from}

The N-1 word partial phrase, in `phrase=<value-or->` form. Required
flag, no default. Two value shapes:

- `phrase=<11-or-14-or-17-or-20-or-23 words>` — inline. The partial
  appears in the assembled argv and is therefore visible to other
  processes on the machine via `/proc/<pid>/cmdline` (Linux) or
  equivalent. The CLI emits a stderr advisory recommending the
  stdin form for sensitive input.
- `phrase=-` — read the partial from stdin. The GUI surfaces stdin
  routing through the secret-bearing widget; the partial does not
  appear in argv.

The partial word count must be exactly one of `{11, 14, 17, 20,
23}`. Any other count is refused with `final-word: got K words;
expected one of [11, 14, 17, 20, 23]`. Each word must be a valid
entry in the selected `--language` wordlist; an unknown word at
position I is refused with `final-word: unknown BIP-39 word at
position I (not in selected wordlist; did you pick the right
--language?)`.

The GUI renders this flag as a NodeValueComposite widget — a
single-row text field labelled `--from` with a single composite
node `phrase`. The `?` help-icon next to the row deep-links here.
The widget treats inline values as paste-warn-eligible per [§14
Defense 4](#secret-handling) when the typed/pasted content
exceeds 8 characters.

### `phrase` {#mnemonic-final-word-from-phrase}

The only valid node for `--from`. The value is the partial
N-1 word phrase per the body above. Schema-`secret: true` at the
node level (the BIP-39 phrase is secret-bearing).

## `--language` {#mnemonic-final-word-language}

The BIP-39 wordlist used to interpret the partial AND to choose
candidate words. Optional; defaults to `english`. Ten allowed
values; the wordlist names match the BIP-39 specification verbatim
(note: the `mnemonic` tab uses the names below; the `ms` tab uses
a different naming convention — see [`ms encode`'s
`--language`](#ms-encode-language) for the divergence and the per-language
mapping table).

The GUI renders this flag as a Dropdown widget. The `?` help-icon
next to the dropdown label deep-links here.

### Outline {#mnemonic-final-word-language-outline}

- [`english`](#mnemonic-final-word-language-english)
- [`simplifiedchinese`](#mnemonic-final-word-language-simplifiedchinese)
- [`traditionalchinese`](#mnemonic-final-word-language-traditionalchinese)
- [`czech`](#mnemonic-final-word-language-czech)
- [`french`](#mnemonic-final-word-language-french)
- [`italian`](#mnemonic-final-word-language-italian)
- [`japanese`](#mnemonic-final-word-language-japanese)
- [`korean`](#mnemonic-final-word-language-korean)
- [`portuguese`](#mnemonic-final-word-language-portuguese)
- [`spanish`](#mnemonic-final-word-language-spanish)

### `english` {#mnemonic-final-word-language-english}

The BIP-39 English wordlist (2048 entries). Default if `--language`
is omitted. The canonical worked example below uses this.

### `simplifiedchinese` {#mnemonic-final-word-language-simplifiedchinese}

The BIP-39 Simplified Chinese wordlist (2048 entries; UTF-8 input
required). Note the spelling: this tab joins the qualifier to the
language name as a single token. The `ms` tab uses
`chinese-simplified` (hyphen-separated). The two GUI tabs do NOT
share a single wordlist enum; the divergence is documented at
[`ms encode --language`](#ms-encode-language).

### `traditionalchinese` {#mnemonic-final-word-language-traditionalchinese}

The BIP-39 Traditional Chinese wordlist. Same naming convention as
`simplifiedchinese`.

### `czech` {#mnemonic-final-word-language-czech}

The BIP-39 Czech wordlist.

### `french` {#mnemonic-final-word-language-french}

The BIP-39 French wordlist.

### `italian` {#mnemonic-final-word-language-italian}

The BIP-39 Italian wordlist.

### `japanese` {#mnemonic-final-word-language-japanese}

The BIP-39 Japanese wordlist. Note: the canonical Japanese wordlist
uses ideographic-space (U+3000) word separators in some
implementations; this CLI accepts ASCII-space-separated input
verbatim.

### `korean` {#mnemonic-final-word-language-korean}

The BIP-39 Korean wordlist.

### `portuguese` {#mnemonic-final-word-language-portuguese}

The BIP-39 Portuguese wordlist.

### `spanish` {#mnemonic-final-word-language-spanish}

The BIP-39 Spanish wordlist.

## `--json-out` {#mnemonic-final-word-json-out}

Optional. If provided, writes a versioned JSON envelope to the
specified PATH **in addition to** the plain candidate list on
stdout. The JSON envelope is a side-effect — it does NOT replace
stdout; both outputs are emitted in parallel. Schema:

```json
{
  "schema_version": "1",
  "language": "english",
  "partial_word_count": 11,
  "target_word_count": 12,
  "candidate_count": 128,
  "candidates": ["abandon", "ability", "above", "..."]
}
```

Field order is part of the schema (SHA-pinned in
`tests/cli_final_word_json.rs`). `candidates` is lexicographically
sorted; `candidate_count == candidates.len()`.

On Unix the resulting file inherits the process umask. A
world-readable result (default umask 022 → mode 644) raises a
stderr advisory: `warning: --json-out <PATH> inherits umask (file
may be world-readable, mode 644); consider --json-out /dev/stdout
or chmod 0600 the path before invoking`. The GUI does not
intercept this advisory; it surfaces in the output panel's
stderr region after the run completes.

The GUI renders this flag as a Path text field (no `?` help-icon
per [§33 Option C placement](#help-icons-and-deep-links-into-this-manual)
— Path widgets are not in the `?`-button class set).

## Worked example — fill the form

1. Switch to the **mnemonic** tab and pick **Final Word (BIP-39
   N-1 → candidate Nth words)** from the subcommand selector.
2. In the `--from` row, type or paste the 11-word partial:

   ```text
   phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon
   ```

3. Leave `--language` at its default (`english`) and `--json-out`
   blank.
4. The `Preview:` line should read:

   ```text
   mnemonic final-word --from "phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon"
   ```

5. Click **Run**. Because `--from` carries a `phrase=` node, the
   run-confirm modal appears with the full partial visible. After
   you click **Run** in the modal the subprocess fires.

The output panel renders 128 candidate words on stdout, one per
line, lexicographically sorted, including `about` (the canonical
12-word vector's correct last word). For an N=24 worked example,
substitute a 23-word partial — the output collapses to 8
candidates, including `art` (the canonical 24-word zero-entropy
vector's last word).

## Refusals

| Trigger | CLI message |
|---|---|
| Partial word count not in `{11, 14, 17, 20, 23}` | `final-word: got K words; expected one of [11, 14, 17, 20, 23] ...` |
| Empty partial (0 words after `split_whitespace`) | `final-word: empty partial phrase; need 11/14/17/20/23 words ...` |
| Unknown word at position I | `final-word: unknown BIP-39 word at position I (not in selected wordlist; did you pick the right --language?)` |
| `--from` variant other than `phrase=` | `final-word --from only accepts phrase=<value> or phrase=-` |

The GUI does not pre-validate the partial — it submits the form and
surfaces the CLI's refusal in the output panel's stderr region.

## Advisories

| Trigger | Stderr advisory |
|---|---|
| Inline `--from phrase=<value>` | `warning: secret material on argv (--from phrase=) — pipe via --from phrase=- to avoid /proc/$PID/cmdline exposure` |
| Stdout is a TTY AND candidate set non-empty | `warning: candidate list is secret material — pairing the partial phrase with any candidate yields a valid seed phrase; do not paste this output into untrusted tools` |
| `--json-out PATH` with world-readable file (Unix umask 022 default) | `warning: --json-out <PATH> inherits umask (file may be world-readable, mode 644); consider --json-out /dev/stdout or chmod 0600 the path before invoking` |

The TTY-vs-pipe advisory fires whenever the CLI detects a TTY on
stdout. The GUI's subprocess launcher pipes stdout via
`std::process::Stdio::piped()` (`mnemonic-gui/src/runner.rs`), so
this advisory does NOT fire when invoked through the GUI — but
the candidate-list secrecy concern remains regardless. Treat the
output panel's contents as secret material until you have used
or discarded them.
