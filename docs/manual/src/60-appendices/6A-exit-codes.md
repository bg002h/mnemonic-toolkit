# Appendix J — Exit codes

Every binary in the constellation (`mnemonic`, `md`, `ms`, `mk`, `mt`,
`me`, `mr` and `mr-sitting`) exits with the same codes. They come from
`SPEC_constellation_exit_codes.md` in the mnemonic-engrave repository.
Each chapter's exit-code section lists only the codes its verbs can
produce and points here for their meaning.

## The codes

| Code | Name | Set when | What to do |
|---|---|---|---|
| `0` | OK | Everything requested was done and every check passed. | Nothing. |
| `1` | NO | The input is well-formed, and a comparison against something else came out no: a reference, a phrase, a template, the chain, an expectation. Examples: a verify mismatch, NOT spend-equal, not a member, `--expect` not met, an input the node proves spent. | Read the verdict. |
| `2` | INVALID | An input is wrong for this verb whatever the flags: a bad checksum, an uncorrectable string, a wrong HRP or kind, a newer format, empty input, a malformed value the tool reads, an input that breaks the verb's correctness rules. | Fix or replace the input. |
| `4` | UNVERIFIED | The tool checked part of the result and could not confirm it: a correction with no oracle, a partial decode, a signature accepted leniently. | Verify the result before trusting it. |
| `8` | REFUSED | The tool declined under a safety or policy rule that a flag, network, channel or different destination would waive, or a network mismatch. Examples: a secret on the command line, an unsafe output destination, an output that exists, a limit a flag can raise. | Change how it is run, or pass the override knowingly. |
| `16` | REPAIRED | The tool applied a checksum correction to an input and used the corrected value. | The plate is damaged: re-engrave it. |
| `32` | IO | Something the tool needed from the system failed or is missing: an input file, an output that cannot be written, stdout, a helper program you asked for, a device, a lock, a RAM folder, an environment variable, a feature this build lacks. | Fix the environment. |
| `64` | USAGE | The command line alone is wrong: an unknown flag or verb, a missing argument, a conflict, a bad setting. Nothing ran. Never combined with another code. | Fix the command line. |
| `101` | INTERNAL | A bug, or any failure no other class covers. Never combined. | Report it. |
| `130` | stopped | The operator stopped it, at a prompt or with Ctrl-C. | |
| `128+N` | | The shell reports a process killed by signal N this way. | |

Codes 1 to 63 are **sums** of the six bits 1, 2, 4, 8, 16 and 32, and
each sum is unique. For example `20` is 16 + 4: a correction was
applied and it cannot be verified yet. `18` is 16 + 2: a correction
was applied, but the corrected string is still not a valid card.
`33` is 32 + 1: `mnemonic verify-bundle` found a mismatch and could
not write its report either. (A verb that fails before it writes
anything keeps its own code: an invalid input with stdout closed is
still `2`.) 64 and 101 never combine. Any other code from 65 to 125 is a
bug.

Every binary's root `--help` ends with the same legend:

```text
Exit status: 0 = done, every check passed. 64 = bad command line, nothing
ran. 101 = internal error. 130 = stopped by the operator. Other codes from
128 up = killed by a signal. Otherwise the sum (1 to 63) of: 1 a check said
no; 2 an input is invalid; 4 verify the result before trusting it; 8 refused
(network mismatch, secret on the command line, unsafe output); 16 a
correction was applied; 32 a file, device or helper failed.
```

## How a failure is classed

The binaries ask these questions in order, and the first "yes" decides.
The order is the specification's; it numbers the same questions 0 to 7.

1. Did the operator stop it? Then `130`.
2. Is the command line alone wrong, judged without reading any input,
   file or environment variable? Then `64`. A *setting* (a count, a
   range, an output form, share K and N) that is out of range is a
   usage error. A malformed *input* value is not, even when it arrives
   as an argument (a card, a key, a fingerprint, a template name): that
   is `2`. The one exception is the secret-on-argv guard, which runs
   first and answers `8`.
3. Did something needed from the system fail or go missing? Then `32`.
4. Is it a network mismatch? Then `8`.
5. Is the input well-formed but does it fail to match something it
   should match (a reference you gave, a phrase, a template, a
   policy-id stub, a plan it should belong to, the chain, an `--expect`
   claim)? Then `1`, even when an override flag exists. A limit, cap,
   deadline or delta is not something to match: exceeding one is `8`
   when a flag would waive it, otherwise `2`.
6. Did the tool decline under a safety or policy rule that a flag,
   network, channel or destination would waive? Then `8`.
7. Is the input itself wrong or insufficient for the verb? Then `2`.
8. Anything else is a bug: `101`.

Further rules every binary follows:

- **An override clears what it overrides.** `restore --allow-mismatch`
  exits 0 and still prints the mismatch.
- **Not asking for a check is not a failure.** `restore` with no
  reference, or `mt` with no node, exits 0 and says what it did not
  check.
- **Every correction sets 16.** A verb that decoded a damaged card by
  correcting it, and used the corrected value, exits with 16 set even
  when everything else succeeded. A script that wants to accept a
  corrected plate tests `[ $(( rc & ~16 )) -eq 0 ]`.
- **A correction in an incomplete chunk set is 16, not 20.** The rest
  of the set verifies it when it is reassembled. Only a correction with
  no oracle at all (a single string that is not chunked, an `ms1`) is
  20.
- **Output failures add 32.** A stdout write error, a full disk or a
  stdout closed with `>&-` adds 32 to the other bits; a closed pipe
  (the reader went away, `| head`) keeps the verb's own code.
- **The code does not depend on whether stdout is a terminal.**
  `mnemonic`'s auto-repair report on a terminal is advice: the verb
  still failed on an invalid input and exits 2 there and in a pipe.
- **Machine-readable output carries the same number.** A JSON envelope
  with an `exit_code` field holds the process's exit status, a bit sum.

## Reading the code in a script

```sh
mnemonic repair --mk1 "$a" --mk1 "$b"; rc=$?
if   [ "$rc" -eq 0 ];  then echo "clean"
elif [ "$rc" -ge 64 ]; then echo "usage, internal error, operator stop or signal: $rc"
else
  [ $((rc & 1))  -ne 0 ] && echo "a check said no"
  [ $((rc & 2))  -ne 0 ] && echo "an input is invalid"
  [ $((rc & 4))  -ne 0 ] && echo "verify before trusting"
  [ $((rc & 8))  -ne 0 ] && echo "refused"
  [ $((rc & 16)) -ne 0 ] && echo "a correction was applied"
  [ $((rc & 32)) -ne 0 ] && echo "file, device or helper failure"
fi
```

When the order of two failures can change the sum (a network refusal
found before or after a correction), test the bits you care about, not
the exact number.

## Per binary

All eight binaries follow the table above in full. The notes below say
which codes each one can produce in practice; its chapter has the
per-verb detail.

| Binary | Codes it uses | Notes |
|---|---|---|
| `mnemonic` | 0, 1, 2, 4, 8, 16, 18, 20, 22, 32, 64, 101 and sums | [`mnemonic` reference](#mnemonic-reference). `repair`: 16 verified or incomplete-set, 20 unverified (also a corrected `ms1` share), 22 a corrected string that is not a card (a single string that does not decode, an mk1 chunk whose header does not parse, an `ms1` secret share at index `s`); a malformed mk1 set is 2 with nothing printed. `verify-bundle`: 1 mismatch, 2 undecodable card, 4 partial; a corrected card adds 16 (17, 18). A `--slot` key of the other network is 8; input that is not UTF-8 is 2. |
| `md` | 0, 1, 2, 4, 8, 16, 20, 22, 32, 64, 101 and sums | [`md` reference](#md-md-cli-reference). `repair` decode-checks each corrected string (adds 2). `descriptor --verify-against` NOT spend-equal is 1. |
| `ms` | 0, 1, 2, 8, 20, 22, 32, 64, 101 and sums | [`ms` reference](#ms-ms-cli-reference). `repair` applied is 20 (also a corrected K-of-N share); 22 a corrected string that is neither a card nor a share; `inspect` that would not decode is 2. |
| `mk` | 0, 1, 2, 8, 16, 18, 22, 32, 64, 101 and sums | [`mk` reference](#mk-mk-cli-reference). A correction in `decode`, `verify`, `inspect`, `address` or `derive` sets 16. `repair`: 18 a corrected card that fails validation, 22 a corrected string that is not a card (a single string that does not decode, or a chunk whose header does not parse); a malformed set is 2 with nothing printed. |
| `mt` | 0, 1, 2, 8, 16, 32, 64, 101 and sums | No chapter in this manual. A correction in `decode`, `verify` or `inspect` sets 16; a named `--bitcoin-cli` that fails is 32; no node at all stays 0. |
| `me` | 0, 1, 2, 8, 32, 64, 101 and sums | No chapter in this manual. Destination refusals are 8; `sysw pack --expect` not met is 1, and so is a sealed file that fails to open (the AEAD check); every verb's `--help` ends with an `Exit status:` line; a named `ME_PREVIEW_BIN` that is missing is 32. |
| `mr` | 0, 1, 2, 8, 16, 20, 22, 32, 64, 101 and sums | [`mr` reference](#mr-mr-cli-reference). |
| `mr-sitting` | 0, 1, 2, 8, 32, 64, 101, 130, 128+N | [`mr-sitting` reference](#mr-sitting-mr-sitting-reference). A deliberate stop at a prompt is 130. |
