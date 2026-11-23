# FIX-tk-printed-text-drift: help and manual examples that fail or mislead as printed (bughunt-1119 bh-tk-I9, I10, I11)

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22. Track: **small** (one Opus review round). Text only: no
behaviour, flag, exit code or refusal changes.
Base: origin/master `b57187b3`. Keys: bh-tk-I9, bh-tk-I10, bh-tk-I11 (all grade I, CONFIRMED). Ruling: **T41**
(architect-bughunt-1119-questions.md L87): "the help follows the code; BIP-85 L398 allows 1 <= rolls <= 2^32-1, so
99 is no spec bound". B8 (README and remedy code count as printed). Sources: `design/agent-reports/bug-hunt-2026-11-19.md`
L61-79; coordinator `agent-reports/bughunt-1119/grade-tk.md` L33-35, L54, L68; `triage-tk-t2.md` L13, L230-243;
`triage-tk-t3.md` L10-18; `xref-tk.md` L23-25.

**Status: complete. No STOP.** Every new holder-visible text is in section 3 and marked PROPOSED (architect
confirms). The exact bytes of every text change are committed as `design/fix-tk-printed-text-drift-texts.patch`
(`git apply` clean at `b57187b3`); section 3 quotes the same texts. The builder applies that patch (or types the same
bytes) and writes the tests of section 6.

## 1. The rule

R-1 (T41). Every printed description of a command (its `--help`, the user manual, the GUI manual) states what the
binary does. Where the binary and the text disagree, the text changes; the binary does not.

R-2 (I9, I11). Every complete command a holder is shown (a `--help` example, a manual ```` ```sh ```` example that
names the command with complete input) runs as printed with exit 0 and prints what the text says it prints.

R-3 (I11). No manual example of these commands passes a secret on argv (`--from <node>=<value>`, `--digits
<value>`): the binary refuses that at exit 2 (`argv_guard`, toolkit#172-#176). The secret goes on stdin with `-`.

Scope (author's call; the architect may narrow it): the three commands the keys name (`compare-cost`,
`derive-child`, `seedqr`) on every printed surface that carries the defect: the two `--help` texts, the manual's
`mnemonic seedqr` section, the manual's BIP-85 chapter `38-bip85-children.md` (it carries the same wrong
`hd-seed` facts as I10 and the same argv form as I11), and the GUI manual's `hd-seed` / `xprv` paragraphs (same
wrong facts). The same argv-form defect elsewhere in the manual and in `crates/mnemonic-toolkit/README.md` is filed,
not fixed (section 8, F1, F2).

## 2. Defect at `b57187b3` (measured; section 5 has the rows)

| Key | Where | As printed | The binary / the spec |
|---|---|---|---|
| I9 | `crates/mnemonic-toolkit/src/cmd/compare_cost.rs:15-16` (`--miniscript` doc, shown by `--help`) | example `or_b(pk(A), pk(B))` | exits 2: `typecheck: fragment «or_b(...)» cannot accept children of types B and B`. `or_b(pk(A),s:pk(B))` exits 0 (it is the manual's transcript `docs/manual/transcripts/41-compare-cost-orb.cmd` and GUI manual `4m-compare-cost.md:76`). |
| I10 | `cmd/derive_child.rs:40-42` | bip39 lengths `12 / 18 / 24` | `derive_child.rs:241` accepts 12, 15, 18, 21, 24 (BIP-85 L159-185 Words Table). |
| I10 | `cmd/derive_child.rs:43-44` | `hd-seed` = "application 32' — derives a 32-byte BIP-32 seed" | `bip85.rs:95-104` derives at `m/83696968'/2'/<index>'` and prints a compressed WIF (BIP-85 L226-240 "HD-Seed WIF, Application number: 2'"). Vector P1 matches the BIP. |
| I10 | `cmd/derive_child.rs:45-46` | `xprv` = "application 32' alt" | `bip85.rs:126-132` derives at `m/83696968'/32'/<index>'` (BIP-85 L250-259). "alt" suggests 32' belongs to hd-seed. |
| I10 | `cmd/derive_child.rs:53-55` | dice "1..=99 each ... REFUSED at runtime" | `derive_child.rs:293-307` runs dice for any rolls >= 1 (u32, so <= 2^32-1); BIP-85 L398 `1 <= rolls <= 2^32 - 1`. 100 rolls exit 0 (P3). |
| I10 (found here) | `cmd/derive_child.rs:59-60` | `rsa-gpg` = "application 828366'" | BIP-85 L369-375: RSA-GPG keys live under `m/83696968'/828365'/...`; there is no 828366'. (Refused either way; text only.) |
| I10 (found here) | `docs/manual/src/30-workflows/38-bip85-children.md:5,13,61,62,67,71-73` | "HD-seed bytes"; "12 / 18 / 24 words; 9 BIP-85 languages"; table `hd-seed` = "64-byte HD-seed bytes (raw)", bip39 `12`, `18`, `24`, dice `1..=10000`; RSA "out-of-scope for v0.8 ... will land in v0.9" | WIF (as above); 5 lengths; 10 languages (`derive_child.rs:352-365`, BIP-85 L120-155); dice >= 1; RSA refused at exit 2 today (v0.107.1). |
| I10 (found here) | `docs/manual-gui/src/40-mnemonic/46-derive-child.md:100-108` | `hd-seed` "Emit a child HD-seed (an xprv at depth 0)"; `xprv` "effectively the same as `hd-seed`" | hd-seed prints a WIF; xprv is a different application (32'). |
| I11 | `docs/manual/src/40-cli-reference/41-mnemonic.md:2867,2878` | `mnemonic seedqr encode --variant compact --from phrase='abandon ... about'`, `mnemonic seedqr decode --variant compact --from seedqr=0000...` | both exit 2 (argv refusal) with nothing on stdout. |
| I11 | `41-mnemonic.md:2870,2925` | `--from phrase='…'`, `--from phrase="abandon ... about"` | the refused argv form, with a placeholder. |
| I11 (found here) | `41-mnemonic.md:2849-2852` | "Both subsubcommands emit an argv-leakage advisory ... Use the stdin form (`-`) to avoid the advisory." | it is a refusal at exit 2, not an advisory (row S3; `--digits <value>` also exits 2, measured). |
| I11 (found here) | `38-bip85-children.md:45-48,84,97,110,123,139` | `--from xprv=<xprv>`, `--from phrase="..."` in the flag list and all five worked examples | exit 2 as printed (B1: the one complete example fails; four are placeholders). |

Root cause (grade-tk.md L54): no gate runs `--help` examples or manual prose commands. Section 6 adds that gate for
these three commands (H1/H2, S1, B1).

## 3. Spec delta (exact texts; all PROPOSED, architect confirms)

The byte-exact change is `design/fix-tk-printed-text-drift-texts.patch` (5 files, 53+/45-). In words:

3.1 `cmd/compare_cost.rs:16`: `` `or_b(pk(A), pk(B))` `` becomes `` `or_b(pk(A),s:pk(B))` `` (the consultant's text,
grade-tk I9). Nothing else in that doc comment changes.

3.2 `cmd/derive_child.rs:40-60` (the `--application` doc, `verbatim_doc_comment`): the `hex`, `password-base64`,
`password-base85` and `rsa` entries are unchanged; the other five become, whitespace-normalized as `--help` prints
them (the patch fixes the line breaks):

- `bip39 BIP-85 application 39' — derives a child BIP-39 mnemonic of `--length` words (12 / 15 / 18 / 21 / 24)`
- `hd-seed BIP-85 application 2' — derives a private key, printed as a compressed WIF, at m/83696968'/2'/<index>' (`--length 0`)`
- `xprv BIP-85 application 32' — derives a child xprv at m/83696968'/32'/<index>' (`--length 0`)`
- `dice BIP-85 application 89101' — derives `--length` rolls (1..=2^32-1) of a `--dice-sides`-sided die`
- `rsa-gpg BIP-85 application 828365' (RSA-GPG key scheme); REFUSED at runtime`

The dice range is written `1..=2^32-1` to match the existing `--dice-sides` text (`2..=2^32-1`, derive_child.rs:103);
the runtime message `valid: rolls >= 1` (derive_child.rs:299) is unchanged. `gen-man` pages follow automatically
(clap renders the same doc comment; no committed man page carries it, grep at `b57187b3`).

3.3 `41-mnemonic.md` `mnemonic seedqr` section:
- L2849-2852 paragraph becomes: "Both subsubcommands refuse a secret supplied inline (`--from seedqr=<value>`, the
  deprecated `--digits <value>`, or `--from phrase=<value>`) at exit 2, before the command line is parsed; see
  [Secret material on argv is REFUSED](#argv-refusal). Pass the value on stdin with `-`."
- L2867: `printf '%s' '<the 12-word abandon...about phrase>' \` newline `  | mnemonic seedqr encode --variant compact --from phrase=-`;
  the `# → 00000000000000000000000000000000   (16 entropy bytes as 32 hex chars)` line stays.
- L2870: the same two-line stdin form (full public phrase, not `'…'`), followed by the unchanged `| xxd -r -p \` and
  `| qrencode -8 ...` lines.
- L2878: `printf '%s' '00000000000000000000000000000000' \` newline `  | mnemonic seedqr decode --variant compact --from seedqr=-`;
  the `# → abandon ... about` line stays.
- L2925: `printf '%s' '<the full phrase>' \` newline `  | mnemonic seedqr encode --from phrase=- | qrencode -o out.png -`.
The placeholders `'…'` and `"abandon ... about"` are replaced by the full public test phrase so each example runs as
printed (the section's other examples already use it).

3.4 `38-bip85-children.md`:
- L5 `HD-seed bytes,` -> `HD-seed WIF keys,`
- L13 `(12 / 18 / 24 words; 9 BIP-85 languages)` -> `(12 / 15 / 18 / 21 / 24 words; 10 BIP-85 languages)`
- L45-48 `--from` bullet -> "the master source, read from stdin: `--from xprv=-` (BIP-85 canonical) or `--from
  phrase=-` (combined with `--passphrase` and `--language`). An inline value is refused (exit 2); see [Secret material
  on argv is REFUSED](#argv-refusal)." (anchor resolves: `make -C docs/manual anchor-check` OK, 9 danglers = baseline.)
- table L61 `| \`bip39\` | new BIP-39 phrase (10 BIP-85-coded languages) | \`12\`, \`15\`, \`18\`, \`21\`, \`24\` |`;
  L62 `| \`hd-seed\` | private key as a compressed WIF | \`0\` (sentinel) |`; L67 `| \`dice\` | deterministic dice rolls | \`1..=2^32-1\` |`.
- L72-73 `out-of-scope for v0.8 pending RUSTSEC-2023-0071 patch and will land in v0.9.` -> `refused (exit 2): the
  \`rsa\` crate carries the unpatched advisory RUSTSEC-2023-0071.` (exit 2 measured: `--application rsa` rc=2.)
- The five worked examples (L83-84, 96-97, 109-110, 122-123, 138-139): `mnemonic derive-child \` + `--from phrase="..."
  \` become `printf '%s' '<the full public phrase>' \` + `  | mnemonic derive-child --from phrase=- \`; the flag lines
  below are unchanged. The chapter already says its examples use the canonical test seed (L77-78), so the four
  `<your master phrase>` / `<phrase>` placeholders become that seed.

3.5 GUI manual `46-derive-child.md:100-108`: hd-seed -> "Emit a child private key as a compressed WIF (BIP-85
application 2', path `m/83696968'/2'/<index>'`). `--length` must be 0. `--network` selects the mainnet (`K…` /
`L…`) or test-network (`c…`) WIF prefix."; xprv -> "Emit a child xprv at depth 0 (BIP-85 application 32', path
`m/83696968'/32'/<index>'`). `--length` must be 0. `--network` selects the prefix." (Anchors unchanged.)

## 4. CHANGELOG, PROPOSED (architect confirms)

Insert after `## [Unreleased]` and its blank line (`CHANGELOG.md:9-10`), above the first existing `###` block; no
version bump:

```
### Help and manual examples now run as printed

- **`compare-cost --help`**: the `--miniscript` example is `or_b(pk(A),s:pk(B))`; the old `or_b(pk(A), pk(B))`
  failed with a Miniscript type error (exit 2).
- **`derive-child --help`**: `hd-seed` is BIP-85 application 2' and prints a compressed WIF at
  `m/83696968'/2'/<index>'` (the help said application 32' and a 32-byte seed); `xprv` is application 32'; `dice`
  runs, for 1..=2^32-1 rolls (the help said 1..=99 and REFUSED); `bip39` takes 12, 15, 18, 21 or 24 words; `rsa-gpg`
  is application 828365'. What the command derives is unchanged.
- **Manual**: the CompactSeedQR examples and the BIP-85 chapter's examples pass the secret on stdin
  (`--from phrase=-`, `--from seedqr=-`); as printed they put it on argv and were refused (exit 2). The BIP-85
  chapter and the GUI manual now say `hd-seed` prints a WIF; the chapter lists five BIP-39 lengths and ten languages
  and the dice range 1..=2^32-1.
```

No flag changes: the `docs/manual/tests/lint.sh` flag coverage and the `mnemonic-gui` schema mirror are unaffected
(`gui-schema` output carries none of the changed texts, grep at `b57187b3`).

## 5. RED rows (run at `b57187b3`, public test material only)

Command: `cargo build --locked --bin mnemonic && design/fix-tk-printed-text-drift-red.sh target/debug/mnemonic`
(from the repo root). Measured at the base: 14 RED, 3 GREEN (controls); with the patch applied and the binary
rebuilt: 17 GREEN, 0 RED (run once each, 2026-11-22).

| Row | What it runs | At `b57187b3` (RED) | After (GREEN) |
|---|---|---|---|
| H1 | the `e.g., \`...\`` span of `compare-cost --help`, as `compare-cost --miniscript '<span>'` | span `or_b(pk(A), pk(B))`, rc=2, `error: compare-cost: parse error: typecheck: fragment «or_b(pk(03998512...),pk(03866e7f...))» cannot accept children of types B and B` | span `or_b(pk(A),s:pk(B))`, rc=0, first stdout line `Input: or_b(pk(A),s:pk(B))` |
| H2 | every `e.g.,? \`...\`` span in `mnemonic <sub> --help` for every `mnemonic` line of `docs/manual/tests/cli-subcommands.list` | 9 spans; differs from the pinned list in `compare-cost: or_b(pk(A), pk(B))` | exactly the pinned 9 (section 6) |
| D1 | `derive-child --help`, `hd-seed` entry of `--application` | `hd-seed BIP-85 application 32' — derives a 32-byte BIP-32 seed (\`--length 0\`)` | the 3.2 hd-seed text |
| D2 | same, `xprv` entry | `xprv BIP-85 application 32' alt — derives a child xprv directly (\`--length 0\`)` | the 3.2 xprv text |
| D3 | same, `dice` entry | `dice BIP-85 application 89101' — derives \`--length\` dice rolls (1..=99 each \`--dice-sides\`-sided); REFUSED at runtime` | the 3.2 dice text |
| D4 | same, `bip39` entry | `... words (12 / 18 / 24)` | `... words (12 / 15 / 18 / 21 / 24)` |
| D5 | same, `rsa-gpg` entry | `rsa-gpg BIP-85 application 828366' — RSA-GPG; REFUSED at runtime` | the 3.2 rsa-gpg text |
| P1 (control) | BIP-85 master on stdin, `--application hd-seed --length 0 --index 0` | `Kzyv4uF39d4Jrw2W7UryTHwZr1zQVNk4dAFyqE6BuMrMh1Za7uhp` (= BIP-85 L248) | same |
| P2 (control) | same, `--application xprv` | `xprv9s21ZrQH143K2srSbCSg4m4kLvPMzcWydgmKEnMmoZUurYuBuYG46c6P71UGXMzmriLzCCBvKQWBUv3vPB3m1SATMhp3uEjXHJ42jFg7myX` (= BIP-85 L267) | same |
| P3 (control) | dice d6 100 rolls; bip39 15 and 21 words | rolls=100, words=15,21, all rc=0 | same |
| S1 | every logical command naming `mnemonic seedqr` in the ```` ```sh ```` blocks of the `## \`mnemonic seedqr\`` section (to the next `## `), except those with `--json-out` (transcript-backed, write `/tmp`); `| xxd` / `| qrencode` stages cut off | runnable=8 failing=4: L2867 rc=2, L2870 rc=2, L2878 rc=2, L2925 rc=2, each `mnemonic: argument N on ARGV (--from phrase=) is a BIP-39 phrase -- 93 characters. Nothing about it is printed here.` (seedqr: `(--from seedqr=) is a SeedQR digit string -- 32 characters`) | runnable=8 failing=0; L2867's stdout `00000000000000000000000000000000` and L2878's `abandon ... about` equal their `# → ` lines |
| S2 | lines of the same sh blocks matching `--from (phrase\|seedqr)=[^-]\|--digits [^-]` | 2867, 2870, 2878, 2925 | none |
| S3 | the section's prose | contains "emit an argv-leakage advisory ... to avoid the advisory" | contains `(#argv-refusal)`, no "argv-leakage advisory" |
| B1 | every logical command naming `mnemonic derive-child` in the sh blocks of `38-bip85-children.md`, except those containing `<` (the synopsis) | runnable=1 failing=1 (L83-87 rc=2, BIP-39 phrase on argv) | runnable=5 failing=0, all non-empty stdout |
| B2 | lines of those sh blocks matching `--from (phrase\|xprv)=[^-]` | 84, 97, 110, 123, 139 | none |
| B3 | the four exact 3.4 lines present; `HD-seed bytes`, `64-byte HD-seed`, `9 BIP-85`, `land in v0.9`, `1..=10000` absent | 4 missing, 5 present | all present / absent |
| G1 | GUI manual `### \`hd-seed\` {#...-application-hd-seed}` and `### \`xprv\` {#...-application-xprv}` sections, normalized | "Emit a child HD-seed (an xprv at depth 0) ..." | the 3.5 texts |

## 6. Tests

New file `crates/mnemonic-toolkit/tests/cli_printed_text_drift.rs`, one test per row, named exactly (TRACE):
`h1_compare_cost_help_example_runs`, `h2_help_eg_examples_inventory`,
`d1_derive_child_help_hd_seed_is_application_2`, `d2_derive_child_help_xprv_is_application_32`,
`d3_derive_child_help_dice_is_runnable_with_bip85_range`, `d4_derive_child_help_bip39_lists_five_lengths`,
`d5_derive_child_help_rsa_gpg_is_application_828365`, `p1_hd_seed_bip85_vector_application_2`,
`p2_xprv_bip85_vector_application_32`, `p3_dice_100_rolls_and_bip39_15_21_run`,
`s1_manual_seedqr_commands_run_as_printed`, `s2_manual_seedqr_no_inline_secret`,
`s3_manual_seedqr_argv_paragraph_says_refused`, `b1_manual_bip85_commands_run_as_printed`,
`b2_manual_bip85_no_inline_secret`, `b3_manual_bip85_application_facts`, `g1_gui_manual_hd_seed_and_xprv_facts`.

Each asserts its row's GREEN condition in section 5, as the RED script computes it:
- Help text: run `env!("CARGO_BIN_EXE_mnemonic")` with `--help`, collapse whitespace runs to one space. A D-row entry
  starts at the line (inside the `--application` block, i.e. after the line beginning `--application ` and before the
  next line beginning `--`) whose first token is the app name followed by two spaces, and runs to the next such line
  or a blank line. D rows assert equality with the 3.2 strings.
- H2 pinned list (subcommand: span): `compare-cost: or_b(pk(A),s:pk(B))`, `convert: --to mk1`, `export-wallet:
  bip84-0`, `gen-man: mnemonic-seed-xor-split.1`, `restore: --account 0,1,2,3`, `restore: m/84'/0'/7'`, `restore: 2h`,
  `verify-bundle: m/84'/0'/7'`, `verify-bundle: 2h`. Compare as sorted sets. Only the first is a runnable command
  (H1 runs it); the rest are flag or value fragments. The failure message says: "a --help text gained or lost an
  `e.g.` example; if it is a runnable command add a run row like h1, then update this list".
- S1/B1 extractor: read the markdown from `CARGO_MANIFEST_DIR/../../docs/...`; inside ```` ```sh ```` blocks only;
  skip lines whose trimmed text starts with `#`, except a `# → <v>` line right after a command, which sets that
  command's expected first stdout line to `<v>` cut at the first run of two spaces; join a line onto the previous
  when the previous ends with `\` or the line's trimmed text starts with `|`; a blank line or block end ends a
  command. Keep commands containing `mnemonic seedqr` (S1, skipping `--json-out`) or `mnemonic derive-child` (B1,
  skipping `<`). Cut each command at its first `| xxd` or `| qrencode` and strip trailing `\` and whitespace. Run
  with `bash -o pipefail -c`, `PATH` = the binary's directory prepended. Assert rc 0, non-empty stdout, the `# → `
  value when present, and the count: S1 = 8, B1 = 5 (a count change means an example was added or turned into a
  placeholder: update deliberately).
- P rows use the BIP-85 master `xprv9s21ZrQH143K2LBWUUQRFXhucrQqBpKdRRxNVq2zBqsx8HVqFk2uYo8kmbaLLHRdqtQpUm98uKfu3vca1LqdGhUtyoFnCNkfmXRyPXLjbKb`
  on stdin (`--from xprv=-`). The triage tests (`triage-tk-t2.md` L230-243, `triage-tk-t3.md` L25-110) are a
  starting point; their names do not carry over.

Existing tests that change: none. Checked at `b57187b3`: no test or source outside the two doc comments contains
`or_b(pk(A), pk(B))`, `application 32' — derives a 32-byte`, `1..=99`, `828366'` or `REFUSED at runtime` for dice
(`grep -rn` over `crates/`); no test reads `38-bip85-children.md`, the seedqr section or `46-derive-child.md`;
`readme_dice_kat_parity.rs` reads only `crates/mnemonic-toolkit/README.md`, which this FIX does not touch.

Source files this FIX changes (for build ordering): `crates/mnemonic-toolkit/src/cmd/compare_cost.rs`,
`crates/mnemonic-toolkit/src/cmd/derive_child.rs` (doc comment L40-60 only),
`crates/mnemonic-toolkit/tests/cli_printed_text_drift.rs` (new), `docs/manual/src/40-cli-reference/41-mnemonic.md`
(L2849-2925), `docs/manual/src/30-workflows/38-bip85-children.md`, `docs/manual-gui/src/40-mnemonic/46-derive-child.md`,
`CHANGELOG.md`; `design/fix-tk-printed-text-drift-red.sh` and `-texts.patch` (this branch). Overlaps:
`fix/tk-env-dash-stdin-order` (#26) changes `cmd/derive_child.rs` code (L126-145) and `41-mnemonic.md` near L79-82
and `CHANGELOG.md` [Unreleased]: no shared lines, but build after it or rebase (the patch carries no CHANGELOG hunk; its hunks
apply by context, so re-run `git apply --check` after rebasing). `fix/tk-derive-child-depth` and `fix/tk-coldcard-dup-headers` are
merged into `b57187b3`'s history.

## 7. Gate (builder, one run to a file, then grep)

    export PATH=$HOME/.cargo/bin:$PATH; cargo --version   # 1.85.0
    cargo +1.95.0 fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings \
      && cargo nextest run --locked --workspace --build-jobs 2 -j 2 > gate.log 2>&1; grep -E '^\s+(PASS|FAIL)|Summary' gate.log | tail -5
    cargo build --locked --bin mnemonic && design/fix-tk-printed-text-drift-red.sh target/debug/mnemonic > red.log; grep -cE '^[A-Z][0-9]+ GREEN' red.log   # 17
    make -C docs/manual lint MNEMONIC_BIN=$PWD/target/debug/mnemonic MD_BIN=md MS_BIN=ms MK_BIN=mk && make -C docs/manual anchor-check

Plus every other job of `.github/workflows/rust.yml`, `manual.yml`, `manual-gui.yml` (needs
`MANUAL_GUI_UPSTREAM_ROOT` = a mnemonic-gui checkout at the pinned tag; CI sets it) and `examples.yml`, locally or
green on the PR head. Author's pre-check with the patch applied: `make -C docs/manual lint` OK, `anchor-check` OK
(9 danglers = baseline); the GUI-manual lint was not run locally (no pinned mnemonic-gui checkout). Not covered by
this gate: the rendered PDF/HTML layout, the generated man pages (same doc comments, not diffed), the `xxd` and
`qrencode` stages of the QR examples (not run), every manual section other than the three named, and the README.

## 8. FOLLOWUPS

Closes (no `design/FOLLOWUPS.md` line names bh-tk-I9/I10/I11; grep at `b57187b3`): two old-format entries of
`docs/manual/FOLLOWUPS.md`: `derive-child-dice-out-of-scope-docstring` (L89-96, the help's dice REFUSED text) and
`dice-length-cap-undocumented` (L98-104, resolved per T41 by dropping the `1..=10000` bound). The coordinator
records those closures after merge; this FIX does not edit either FOLLOWUPS file.

Files, after the PR merges (direct-push; not in the PR), proposed lines in the report
`agent-reports/fix-doc-bh-tk-printed-text-drift.md`: F1 the manual-wide argv-form example sweep and a prose-command
gate; F2 the crate README's derive-child examples; F3 `derive-child --language` applying to the master phrase.
