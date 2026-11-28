# R0 review — `design/SPEC_convert_phrase_out_dir.md`

Reviewer: R0 architect (adversarial). Branch `claude/project-thread-oyjycn` at
`ef47be8` (spec), code baseline master `d72a18b`. Binary measured:
`target/debug/mnemonic` 0.107.1 (= crate version at `d72a18b`). Only the
all-zero-entropy public test vector was run.

Settled and not re-argued: no entropy generation; fingerprint as the hash;
passphrase fingerprint behind `--name-with-passphrase-fp`; local date with UTC
fallback; Windows warns.

**Verdict: NOT GREEN — 1 Critical, 8 Important, 12 Minor.**

---

## Critical

### C-1. `--from phrase=` → `--to phrase` is a refused edge; the spec's headline input does not work

Spec §2 ("Every input that yields a BIP-39 phrase works: `phrase`, `seedqr`,
`entropy`, `ms1`"), §1 ("typed, … from `final-word`"), and §5 example 2
(`mnemonic convert --from phrase=- --to phrase --phrase-out-dir ~/seeds`) all
assume `(Phrase, Phrase)` is an edge. It is not:

- `crates/mnemonic-toolkit/src/cmd/convert.rs:681-734` `is_supported_direct_edge`
  has no `(Phrase, Phrase)`; the comment at `:692-697` names it the
  "`(Phrase, Phrase)` identity barrier".
- `classify_edge` (`:737-796`) falls to `refusal_one_way` (`:792-794`).
- Measured: `echo "<all-zero phrase>" | mnemonic convert --from phrase=- --to phrase`
  → `error: --to phrase is cryptographically unrecoverable from --from phrase (one-way derivation barrier)`, exit 2.

The most common use (words the user typed, or `final-word` output piped in)
therefore exits 2. The spec has to make a decision, not leave it to the build:
(a) admit `(Phrase, Phrase)` only when `--phrase-out-dir` is present (the
canonicalising re-render is real work: `parse_in` + `from_entropy_in`), with
the one-way-barrier message unchanged otherwise; or (b) admit it
unconditionally (a behaviour change on an existing refusal, with its own
CHANGELOG and manual line); or (c) drop `phrase` from the supported inputs and
fix §1/§2/§5. (a) keeps every existing refusal byte-identical. Whatever is
chosen needs RED rows for both sides (with and without the flag).

---

## Important

### I-1. `write_private` replaces files and follows a dangling symlink; "never overwrites" is not what it does

Spec §4 says the file is written with `write_private` and "Never overwrites",
with a check-then-rename window called acceptable "because the name is derived
from the contents". The library contract, `vendor/mnemonic-io-lib/src/write.rs`:

- `:79-106`, the doc: "creating **or replacing** it"; "Symlinks are resolved on
  the last component and the target is replaced". `write_atomic` ends in
  `fs::rename(temp, fin)` (`:472`), which replaces with no EEXIST.
- **Dangling symlink.** `classify` (`:336-371`): `fs::metadata(path)` follows
  the link, so a dangling link returns NotFound and `replace = false`;
  `resolve_final` (`:306-333`) walks the link to its target ("NotFound ends the
  walk … which is the final name"); the temp is then renamed onto the
  **link's target**. A caller pre-check made with `Path::exists()` /
  `fs::metadata` also says "absent" for a dangling link. So a symlink planted
  at `DIR/seed-<date>-<fp>.txt` sends the seed file wherever it points. The
  name is predictable: the date plus a fingerprint that every descriptor
  shows (spec §6 C1 says so itself). It needs write access to `DIR`, so the
  danger is a shared or synced `DIR`, not `~/seeds`.
- **Non-regular file at the name.** A FIFO, a device or a directory gives
  `classify → None → write_direct` (`:96-104`, `:120-140`), which opens it in
  place. The spec's "same bytes?" pre-read on a FIFO blocks for ever.
- **TOCTOU argument.** "A racing writer of the same name writes the same bytes"
  holds only for a benign racer. The race is with an attacker who swaps the
  name for a symlink between the check and the rename, and the rename then
  goes through the link to its target, not just onto the name.
- **Spelling.** `mnemonic_io_lib::write_private` does not exist at the crate
  root (`vendor/mnemonic-io-lib/src/lib.rs:61-84`). The item is
  `mnemonic_io_lib::write::write_private`, as at `src/cmd/seedqr.rs:309`.

Required: the spec must name a no-replace mechanism. For example: create the
temp with `create_new` and `0o600`, write and fsync it, then `fs::hard_link(temp,
final)` (atomic, EEXIST, does not follow a symlink at the destination), then
unlink the temp. Where `link` is unsupported (FAT and exFAT USB sticks, a likely
destination for this file, and some SMB or FUSE mounts), fall back to
`OpenOptions::create_new(true).mode(0o600)` on the final name. `O_CREAT|O_EXCL`
fails on any existing entry, a dangling symlink included, so this gives no
replace and no TOCTOU. Use `symlink_metadata` (never `metadata`/`exists`) for
the "already there" branch. Compare bytes only when the entry is a regular
non-symlink file under a small size cap (for example ≤ 1 KiB, since the longest
24-word phrase is well under that), and refuse any other kind of entry. If this
belongs in mnemonic-io-lib as a `write_private_new`, say so: the library holds
mechanism, so it is a cross-repo change and needs a 0.1.4 publish, a
`Cargo.lock` change and a re-vendor.

### I-2. The fingerprint for the name has no derivation path in `convert` today; it collides with `--template`, the passphrase and the warning logic

The spec says `<fp>` is "the same value `--to fingerprint` prints". Measured and
read:

- `--to fingerprint` needs `--template` (`convert.rs:1391-1400`; measured:
  `--to phrase --to fingerprint` without `--template` → "--template is required
  for --to fingerprint", exit 1). Master derivation runs only when
  `Xpub|Xprv|Fingerprint` is a target (`:1372-1374`). The name's fingerprint
  must be computed with no `--template`, whatever the targets are. The spec
  has to say this, or the build will either require `--template` for
  `--phrase-out-dir` or reuse the template path.
- `--to fingerprint` prints the **passphrase-applied** fingerprint
  (`pbkdf2_passphrase` goes into `derive_bip32_from_entropy`, `:1379-1405`;
  measured with `--passphrase @env:PP` and PP=TREZOR, it prints `b4e3f5ed`, not
  `73c5da0a`). So "the same value `--to fingerprint` prints" is false whenever a
  passphrase is given. The raw `<fp>` needs a separate derivation with `""`.
- The `ms1` arm (`:1734-1785`) derives nothing and has no passphrase plumbing.
  The fingerprint for an ms1 input is new code on that arm.
- **Warning contradiction.** `edge_uses_pbkdf2` (`:1047-1058`) is false for
  `--to phrase` alone and for every `ms1` primary, so `--passphrase X` prints
  `warning: --passphrase ignored on this edge` (measured). Under
  `--name-with-passphrase-fp` the passphrase *is* used. Without that flag, and
  with `--phrase-out-dir`, it really is ignored for the name. The spec must
  say which case suppresses the warning: with the flag only.
- Network independence (the master fingerprint does not depend on
  `--network`) should be stated, so the build does not thread `--network`
  into the name.

Specify: `fp = master_fingerprint(PBKDF2(words_in_output_language, ""))`, and
`fp_pass` the same with the resolved passphrase, using the same NFKD and
derivation function `--to fingerprint` uses, so the two agree byte for byte
when no passphrase is given. Add RED rows for no `--template`, for an ms1
primary, and for warning present or absent.

### I-3. Language: an ms1 `Mnem` card's wire language wins over `--language`; the suffix and fingerprint must follow it

Spec §3: the `-<language>` suffix "using the `--language` value". For an ms1
input that is wrong:

- `convert.rs:1742-1757`: `Payload::Mnem` binds `payload_lang` from the wire.
  `--language` is ignored for it. `Payload::Entr` uses `--language`.
- Measured: a Japanese card (made with `--to ms1 --language japanese`) read
  with `--from ms1=- --to phrase` and **no** `--language` prints Japanese
  words. Under the spec's rule the file gets **no suffix**, so it reads as
  English. With `--language spanish` the words are still Japanese, so the
  spec's rule would add `-spanish` to a Japanese file.
- The fingerprint depends on the language, because PBKDF2 runs over the words.
  The spec must derive `<fp>` from the words actually written
  (`payload_lang`), not from `args.language`.
- Suffix spelling: the toolkit's `--language` values are
  `clap(rename_all = "lower")` (`src/language.rs:9-23`), so
  `simplifiedchinese` and `traditionalchinese` (measured: `chinese-simplified`
  is rejected). ms-cli spells them `chinese-simplified` and
  `chinese-traditional` (`mnemonic-secret/crates/ms-cli/src/language.rs:9-12,
  51-63`). The spec promises the suffix tells "which `--language` to pass back",
  and §5 names `ms encode --in` as a reader. Pick one spelling and document the
  other tool's spelling, or choose a suffix that both accept. Today no single
  spelling of the Chinese lists is accepted by both tools.

### I-4. Refusing `--group-size`/`--separator` rests on a false premise and blocks a real workflow

Spec §4: they "would break re-reading", so passing them is refused. They never
touch `phrase`: grouping applies only to `Ms1 | Mk1` (`convert.rs:1268-1275`).
The manual says so too (`docs/manual/src/40-cli-reference/41-mnemonic.md:1154`:
"Display only; `--json` and non-card outputs … stay unbroken"). Measured:
`--to phrase --group-size 4` prints the phrase unbroken. The refusal blocks
`--to phrase,ms1 --group-size 5 --phrase-out-dir D` (save the words, print the
engraving-grouped card), which is the natural pairing. It also needs
`ArgMatches::value_source`, because `group_size` defaults to 0 and `separator`
to space (`:354-361`), and the `ConvertArgs` derive cannot tell an explicit
default from an omission. Drop the refusal and the matching §8 row, and state
that the file is never grouped.

### I-5. Output ordering, partial output and the `--json` shape are unspecified

- **Order.** Today outputs print in target order (`convert.rs:1239-1283`), then
  the advisory (`:1288-1294`). The spec does not say when the file is written.
  If `--to ms1,phrase` prints `ms1:` and then the write fails, the user has
  half the output and a non-zero exit. Specify: compute everything, decide and
  perform the file write (or the "already saved" verdict) **before** any stdout
  byte, then print. `--json` needs the path before the envelope is written in
  any case.
- **Duplicates.** `--to phrase,phrase` is accepted and prints twice
  (measured). Say what happens: one write, or refuse the duplicate.
- **JSON.** `ConvertJsonEntry { node, value: String }` (`:464-468`), under
  `schema_version: "1"` (`:1255`). Replacing `value` with `path` is not
  additive: a consumer that reads `to[i].value` breaks. Either bump
  `schema_version` to `"2"` and document it (manual and CHANGELOG), or keep the
  change additive, for example `{"node":"phrase","path":"…"}` with `value`
  omitted only under the new flag, recorded as a v1 extension. Also state the
  "already saved" JSON form (for example `"already_saved": true`). Today no
  known GUI consumer parses `convert --json` (grep of `mnemonic-gui/src`), but
  the hazard list makes JSON shapes a contract, so the spec must make the
  decision itself.

### I-6. A pre-existing panic is reachable on the new path (cross-repo F-579 not converged)

`convert.rs:1358-1359` (seedqr) and `:1363-1365` (phrase) call
`Mnemonic::to_entropy()`. The vendored rust-bip39 2.2.2
(`vendor/bip39/src/lib.rs:574-588`) re-detects the language with
`language_of_iter(..).unwrap()`, which returns `Err(AmbiguousLanguages)`
(`:417`) for a phrase drawn entirely from the 100-word English/French
intersection, even under `--language english`. The result is a panic (exit
101). ms-cli found and fixed this as F-579
(`mnemonic-secret/crates/ms-cli/src/cmd/encode.rs:153-167`,
`entropy_in_language`). The toolkit has no F-579 entry (grep of
`design/FOLLOWUPS.md`). Not run here, because the brief limits test data to the
all-zero vector; the evidence is the code path. This feature adds
`--from phrase=` and `--from seedqr=` as headline routes for writing a
canonical file. The spec should either fold the convergence fix in (parse
with a known language and take indices from `language.find_word`) or file a
FOLLOWUP and note the hazard. Per the Rust-primary rule, the toolkit is now
behind ms-cli.

### I-7. Refusals have no named error variant or exit code, so the §8 rows cannot assert anything

"refused" and "exit non-zero" appear throughout §2, §4 and §8. In the binary
today: clap-level `requires`/`conflicts_with` give **64** (measured with an
unknown flag; the remap is at `convert.rs:985`), `BadInput` and `Io` give
**1**, and `ConvertRefusal` gives **2** (`src/error.rs:641-701`). The choice
changes what users and the GUI see. The spec should fix it per refusal:

| refusal | suggested |
|---|---|
| `--name-with-passphrase-fp` without `--phrase-out-dir` | clap `requires` → 64 |
| `--phrase-out-dir` without `--to phrase` | `BadInput` → 1, or a `ConvertRefusal` → 2 (pick one) |
| `--name-with-passphrase-fp` with an empty or absent passphrase | `BadInput` → 1 |
| `DIR` missing or not a directory | `BadInput` → 1 (not raw `Io`, so the message names the flag) |
| differing file at the name | a named variant (alphabetical, per CLAUDE.md), stated code |

On the "summable bit-flag exit codes" direction in the brief: no such plan was
found at `d72a18b`. mnemonic-io-lib's `exit.rs:1-25` publishes no integers by
design. That makes it more important that the spec names the *variant*, so a
later remap has one decision to move, not an ad-hoc `BadInput` string.

### I-8. Lockstep list (§7) is incomplete

- It lists only `--phrase-out-dir` for the mnemonic-gui mirror.
  `--name-with-passphrase-fp` is a second clap flag and must also go into
  `mnemonic-gui/src/schema/mnemonic.rs` `CONVERT_FLAGS` (`:1314`). Otherwise
  the `schema_mirror` gate fails at the next pin bump, the lagging indicator
  the CLAUDE.md case study warns about.
- "a directory picker": the GUI has no directory `FlagKind`
  (`mnemonic-gui/src/schema/mod.rs:181-183`, only `Path { stdio_sentinel }`).
  Its precedent `gen-man --out <DIR>` uses `Path { stdio_sentinel: false }`
  (`mnemonic.rs:4369-4381`). Either say "`Path { stdio_sentinel: false }`, as
  for `gen-man`" or own a new `FlagKind` (GUI work, MSRV-separate repo).
- GUI conditionals: `mnemonic-gui/src/form/conditional.rs:460-471` (`convert`)
  should show `--name-with-passphrase-fp` only when `--phrase-out-dir` is set.
  Name it.
- Manual: the `convert` flag table is at `41-mnemonic.md:1120` onward, and the
  GUI manual page is `docs/manual-gui/src/40-mnemonic/44-convert.md` (its flag
  index at `:33-48`). Also owed: the `--json` shape change (I-5), CHANGELOG
  and version bump, and the `make -C docs/manual lint` flag-coverage run.

---

## Minor

- **m-1 Local-time claim is unverified and probably wrong for this binary.**
  §3 says `time`'s `local-offset` "would always fall back". It refuses only when
  the process is multithreaded. `mnemonic` spawns no threads (no
  `thread::spawn`/`Builder` in `src/` outside tests; `main.rs` is a plain
  `fn main() -> ExitCode`), so it would succeed on Linux. Neither `time`,
  `chrono` nor `jiff` is in `Cargo.lock`. `libc 0.2` is already a direct
  dependency (`crates/mnemonic-toolkit/Cargo.toml:129`), so `libc::time` +
  `localtime_r` gives the local date on Unix with no new dependency. It is sound
  here because no thread calls `setenv`: the `set_var` calls are in
  `#[cfg(test)]` code (`passphrase_input.rs:825-862`, `env_sentinel.rs:137-163`).
  The UTC fallback is days-from-epoch civil arithmetic. Windows would need a
  direct `windows-sys` dependency (already vendored, 0.61.2) for
  `GetLocalTime`, or UTC with the note. Fix the sentence. A new dependency
  needs `cargo vendor` and the vendor-freshness gate.
- **m-2 Wording:** "the same value `--to fingerprint` prints" holds only with no
  passphrase (see I-2). Say "the master fingerprint with an empty passphrase
  (what `--to fingerprint` prints without `--passphrase`)".
- **m-3 Temp file left behind.** `write_atomic` removes the temp on error
  (`write.rs:416-420`), but SIGINT or SIGKILL between create and rename leaves
  `<name>.<pid>.<n>.tmp` (`:375-384`), mode 0600 and holding the words. With
  the I-1 hard-link design the window is the same. Document it in the help or
  manual (the leftover name contains the fingerprint, so the user can find it).
- **m-4 `DIR` edge cases to pin:** `DIR` is a regular file (`join` gives
  ENOTDIR from the open or rename; refuse with a flag-naming message);
  `DIR = "-"` (refuse, or document it as a directory literally named `-`);
  `DIR` is a symlink to a directory (follow it, and say so); `DIR` itself
  group- or world-writable (others can plant names or delete the file; at
  least warn); `DIR = /dev/fd` or `/proc/self/fd` (the joined name is not an
  fd alias because `seed-…` is not numeric, so no special case, but say so).
- **m-5 umask row.** `umask 022` does detect a 0666 create, but `umask 000`
  is the stronger probe (it shows the mode comes from the code, not the
  umask). Also assert the existing-file branch never chmods or touches the
  existing file.
- **m-6 Windows row is untestable in CI.** CI's test matrix is
  `os: [ubuntu-latest]` (`.github/workflows/rust.yml:126-128`). Put the
  "permissions not restricted" decision behind a pure function with a
  platform parameter and test that function, or mark the row as manual.
- **m-7 `ms encode --in FILE` round-trip is untestable in-repo.** No toolkit
  test invokes `ms` (no `MS_BIN` or `cargo_bin("ms")` under
  `crates/mnemonic-toolkit/tests/`). Either emulate ms-cli's reader exactly
  (`ms-cli/src/parse.rs:133-141`: split on whitespace and join with a single
  space, then `parse_in`) in a unit test, or move the row to the cross-repo
  manual-example corpus. The `bundle --slot @0.phrase=-` round-trip is
  testable and was measured to work with a trailing `\n`, English and
  Japanese (Japanese needs `--language japanese`; without it the bundle exits
  with "unknown BIP-39 word at position 0").
- **m-8 Readback examples (§5)** need a passphrase case. `--slot @0.phrase=-`
  and `--passphrase-stdin` cannot be combined (single stdin:
  `cmd/bundle.rs:3032-3037`), so the passphrase must come through `@env:`.
  The non-English case also needs `--language` on both readers.
- **m-9 Same seed, different day.** The name check sees only today's name. A
  re-run tomorrow writes a second plaintext copy. Consider a note when a
  `seed-*-<fp>*.txt` already exists in `DIR`, or state that this is accepted.
- **m-10 Secret buffers.** The file bytes, and the existing file's bytes read
  for the "same bytes" comparison, should be `Zeroizing<Vec<u8>>` and
  mlock-pinned like `primary_value` (`convert.rs:957-966`). Say so; it is the
  repo pattern.
- **m-11 Auto-repair short circuit.** With an ms1 primary at a TTY, a decode
  failure can short-circuit with exit 5 (`convert.rs:1101-1140`) before any
  write. State that no file is written on that path (repair is not
  verification).
- **m-12 Japanese separator.** The file uses an ASCII space. rust-bip39's
  `Display` writes `" "` (`vendor/bip39/src/lib.rs:646-655`, measured), not
  U+3000. Both readers NFKD-normalise and split on whitespace, so this is
  correct. Record it so nobody "fixes" it to U+3000.

---

## Claims verified as true

- §1 defects 1-2. The redirected file holds `phrase: …`; bundle refuses it with
  "BIP-39 word count 13 invalid …" (measured). The redirected file is
  `-rw-r--r--` under umask 0022 (measured).
- Fingerprint fact. BIP-32: the key identifier is HASH160(serP(K)) of the
  public key, and the fingerprint is its first 32 bits. For the all-zero
  12-word seed with an empty passphrase it is `73c5da0a` (measured with
  `convert --to fingerprint --template bip84` and in the bundle header).
- Electrum has no `phrase` output: `(ElectrumPhrase, Phrase)` is refused at
  `convert.rs:770-775`.
- `seedqr` and `entropy` inputs project to `phrase` (`:698`, `:686`). `ms1`
  does too (`:719`), with the Mnem caveat in I-3.
- `bundle` and `ms encode --in` both accept the bare words plus `\n`
  (`read_stdin_to_string` trims, `convert.rs:802-813`; ms-cli
  `normalize_phrase`, `parse.rs:138-141`).
- The stdout private-material advisory is computed from the emitted
  `outputs` (`convert.rs:1288-1294`). Taking the phrase out of `outputs`
  makes it silent as §2 says, provided the build removes it there and does not
  merely skip printing it.
