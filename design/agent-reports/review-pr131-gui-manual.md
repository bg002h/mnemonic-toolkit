**⚠ 0C/1I/2M/3N**

# Class A review: tk PR #131 (head 6f0ba802), GUI-manual installer-ahead text

Reviewer: Opus, medium effort. Read-only; scratch work in `/scratch/cache/release/md-0.20.4/review/`.
First-round review (no previous fix commit).

## Method

- Downloaded `md-0.20.{3,4}-linux-amd64.tar.gz` and `SHA256SUMS.portable` from the
  `descriptor-mnemonic-md-cli-v0.20.{3,4}` releases; `sha256sum -c` OK for both; `--version`
  prints `md 0.20.3` / `md 0.20.4`.
- Ran `scripts/install.sh --only md --root <scratch> --no-man --require-signature` from the
  head: `verified signature on SHA256SUMS.portable (minisign, pinned release key RWRUl0DY…)`,
  sha256 verified, installed `md 0.20.4`. The root CHANGELOG's `first_signed` sentence holds.
- Each of the seven md behaviours in 82-binary-and-launch.md lines 68-90 run on both binaries,
  keyless or with the signet tpub of `abandon … about` at m/84'/1'/0'
  (`tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M`):
  - `--fingerprint @0=+f+f+f+f`: 0.20.3 encodes (decode shows `[0f0f0f0f/m]`); 0.20.4 exits 1
    with `md: --fingerprint @0: invalid hex byte: +f`. Matches.
  - `wsh(multi(2,@0/<0;2147483648>/*,@1/<0;1>/*))`: 0.20.3 `md: codec error: varint value
    2147483648 exceeds single-extension range (max 2^29 - 1)`; 0.20.4 `md: template parse error:
    @0 multipath alt `2147483648` is hardened; …`. Matches.
  - `--policy-id-fingerprint`: 0.20.3 prints `policy-id-fingerprint: 0x3ace1082` on stdout;
    0.20.4 on stderr, stdout is the md1 string only. Matches.
  - `--json` with `older(65536)`: 0.20.3 prints a `phrase` (decodes as `older(65536)`); 0.20.4
    `md: codec error: older(65536) is not what consensus enforces: …`. Matches.
  - `--json` on an 8-key `or_d` template with 8 fingerprints: 0.20.3 `md: codec error: payload
    is 101 data symbols; the codex32 regular code caps single strings at 80 (use chunked encoding
    / --force-chunked)`; 0.20.4 prints `chunk_set_id` + `chunks`, identical to the text path's
    strings. Matches (see N1). `--json --force-chunked` on a short card also gives `chunks`
    (42-md.md `--json` row holds).
  - `md inspect` of a keyed card: last stderr line `note: stdout is a keyless descriptor template
    (no keys)` (0.20.3) vs `note: stdout is watch-only — public keys only, cannot spend`
    (0.20.4). Matches.
  - `md decode`: see I1.
- Read the md-cli source at the tag (`crates/md-cli/src/cmd/decode.rs`, `emit_origins_stderr`).

## Important

### I1. `md decode` bullet omits the no-fingerprint condition; a fingerprinted card still prints the note

`docs/manual-gui/src/80-troubleshooting/82-binary-and-launch.md:88-90` says a single key at its
canonical origin "no longer prints the stderr note `key origins carried by this card …` and its
`--path` hint". md 0.20.4 suppresses the note only when the single key has **no** fingerprint
and its origin equals the canonical one (`decode.rs` at the tag:
`expanded.len() == 1 && expanded[0].fingerprint.is_none() && canonical_origin(..) == Some(..)`).
A holder whose card carries the master fingerprint (any card minted with `--fingerprint`) still
gets the note and the `--path` hint on 0.20.4, so the manual describes an output change they will
not see; a holder comparing against the section may conclude the installer left md at 0.20.3.

Reproduction (run as printed; `repro-decode.sh` is in the scratch dir, args are the two binaries):

    /scratch/cache/release/md-0.20.4/review/repro-decode.sh /scratch/cache/release/md-0.20.4/review/md3 /scratch/cache/release/md-0.20.4/review/md4
    fingerprint=none md 0.20.3: origins-note lines=1
    fingerprint=none md 0.20.4: origins-note lines=0
    fingerprint=73c5da0a md 0.20.3: origins-note lines=1
    fingerprint=73c5da0a md 0.20.4: origins-note lines=1

Fix wording (82-binary-and-launch.md lines 88-90; every clause measured above):

    - **`md decode`.** A single key at its canonical origin, on a card that
      carries no fingerprint for it (for example `wpkh` at `m/84'/0'/0'`
      minted without `--fingerprint`), no longer prints the stderr note
      `key origins carried by this card …` and its `--path` hint. A card
      that carries the fingerprint still prints both.

And in `docs/manual-gui/CHANGELOG.md:11` and root `CHANGELOG.md:76-77`, replace "drops the
origins note for a single key at its canonical origin" with "drops the origins note for a single
key at its canonical origin when the card carries no fingerprint for it". (The same imprecision
is in md's own CHANGELOG at the tag; that is md-side and not this PR's to fix.)

## Minor

- M1. `82-binary-and-launch.md:68` "Then `md`, six outputs:" (ruling on the wording asked for).
  The precedent 99310923 counts bullets: "three outputs" over three `ms` bullets that hold five
  changes. Here four bullets hold seven changes, so "six" matches neither reading (it mirrors the
  six-item CHANGELOG list, which merges the two `--json` changes). The "First `ms`, three outputs: /
  Then `md`, … outputs:" split itself is fine and consistent with the precedent. Fix: "Then `md`,
  four outputs:", and in `docs/manual-gui/CHANGELOG.md:11` "the six `md` outputs" -> "the four
  `md` outputs".
- M2. `design/FOLLOWUPS.md:638` owner "the md-cli pin bump that follows the md release" is this
  PR, which leaves the item open, so read literally it is overdue at merge. The GUI page
  (`53-encode.md:34,209-210`) is correct for the GUI's tier (md 0.20.3 prints the fingerprint on
  stdout, measured); the owner is the GUI's re-pin to md >= 0.20.4. Fix: "**Owner:** the
  mnemonic-gui re-pin to md-cli >= 0.20.4 (when `[installer-ahead]` md is deleted)."

## Nits

- N1. `82-binary-and-launch.md:82` quotes "payload is 129 data symbols"; the count depends on the
  template (measured 101). Write "payload is N data symbols" or cut at "payload is …".
- N2. `82-binary-and-launch.md:77` "stdout carries only the md1 string": a chunked card's stdout is
  a `chunk-set-id:` line plus several md1 strings; "only the md1 string(s)" is exact.
- N3. md-side, not this PR: md 0.20.4 `md encode --help` still says `--policy-id-fingerprint`
  prints "after the phrase"; 42-md.md:40 (stderr) is the accurate text.

## Not findings (checked)

- Sections 12/21 cross-references and 82 lines 31-41 ("except `md` and `ms`", "the two it
  installs") read true against `pinned-upstream.toml` and `install.sh`.
- 21-install.md "the 0.20.4 documented here" and 42-md.md `--policy-id-fingerprint` / `--json`
  rows match 0.20.4 output.
- GUI renders stdout and stderr in its output panel (12-relation-to-cli.md:20), so the stderr
  move does not hide the fingerprint from GUI holders.

0 C / 1 I / 2 M / 3 N
