**⚠ Proposed Important (not Minor): fails open, one condition. Verdict CONFIRMED for the silent-ignore; correct behaviour is an OPEN PRODUCT POINT.**

# Triage: `convert --template wsh-sortedmulti --network signet --to xpub` ignores `--path`

Repo mnemonic-toolkit, origin/master 6ccfdebc, worktree tk-worktrees/triage-convert-path. cargo 1.85.0 (matches rust-toolchain).

## Reproduction (binary run; seed = BIP-39 12-word "abandon x11 about", signet)
`mnemonic --allow-argv-secret convert --from phrase=<seed> --template wsh-sortedmulti --network signet --to xpub [..]`, all exit 0:
- no --path:               tpubDCChhoz5Qdrkn7Z7KXawq6Ad6r3A4MUkCoVTqeWxfTkA6bHNJ3CHUEtALQdkNeixNz4446PcAmw4WKcj3mV2vb29H7sg9EPzbyCU1y2merw
- --path "m/48'/1'/0'/2'": identical tpubDCChhoz5...merw
- --path "m/99'/1'/5'":    identical tpubDCChhoz5...merw
- --account 3:             tpubDCChhoz5Qdrksgvco...  (differs: account IS honoured)
Same with `--template bip84`: with and without `--path m/99'/1'/5'` both print tpubDC8msFGeGuwnKG9Up...Q1M. No warning, no refusal, exit 0.

## Cause (real code)
crates/mnemonic-toolkit/src/cmd/convert.rs:1303-1321: for targets Xpub/Xprv/Fingerprint (`needs_derive`) the code calls `derive_bip32_from_entropy(..., template, args.account)`, which uses `template.derivation_path(network, account)` (derive_slot.rs:57). `args.path` is never read on this edge; it is read only for Wif (line ~1371), and refused for `--from wif` (line 950). A path-explicit sibling `derive_bip32_from_entropy_at_path` already exists (derive_slot.rs:66). Affects xpub, xprv and fingerprint targets from phrase/seedqr/entropy, any template (not only wsh-sortedmulti).

## What the docs promise
- `--help` (convert.rs:234-240): "BIP-32 derivation path. Accepts named/hex/literal ... Required for `--to wif` and for `(xpub, address)` derivation when no `--template`". Silent on xpub-from-phrase.
- Manual 41-mnemonic.md:1102: "`--path <PATH>` | derivation path override". "Override" reads as honoured; nothing says it is ignored or refused with --template.
- convert.rs:286 (`--account`): "Used to compute the template-derived path when `--path` is omitted" -- implies `--path` takes precedence over the template path. The code does not do so.

## Test and verdict (CONFIRMED)
RED test, uncommitted: crates/mnemonic-toolkit/tests/triage_convert_path.rs, `convert_template_xpub_path_not_silently_ignored` (public CLI; asserts: with --path, either the command fails or the xpub differs from the no-path one). Standalone file, no all.rs in this crate.
- Unmodified code: FAILED, "--path silently ignored: xpub: tpubDCChhoz5...merw".
- Scratch fix A (refuse `--path` when needs_derive, 3 lines at convert.rs ~1305): passes.
- Scratch fix B (honour literal `m/...` via derive_bip32_from_entropy_at_path): passes.
Both scratch changes reverted (convert.rs byte-identical to origin/master; `git status` shows only the untracked test file). The test is deliberately neutral between A and B.

## OPEN PRODUCT POINT (not decided here)
What should `--path` do with `--template` on `--to xpub|xprv|fingerprint`?
1. Honour it as override (manual wording). Then decide: are named (`bip84`) and hex (`0xNN`) forms accepted as help says, how it interacts with `--account` (conflict? ignored?), and whether fingerprint output is unaffected (it is master fp, path-independent).
2. Refuse it (fail closed, exit 1 BadInput), text to be chosen; then fix help/manual to say so.
3. Accept and warn on stderr (weakest; still prints a wrong-path key).
Also: whether `--to fingerprint` (path-independent) should accept `--path` silently.

## Proposed grade: Important
Likelihood test: Minor only if the tool fails closed, no printed command fails, and the path needs two or more unusual conditions. Here it fails OPEN (exit 0, plausible-looking xpub for a different path than the user typed), the trigger is a single ordinary flag documented as "derivation path override", and a printed result is wrong versus the request (a user building a cosigner/watch-only descriptor at m/48'/... gets the BIP-87 key). Not Critical: no key loss (the seed still derives either key), and a user can notice via account/origin path. Not Minor on all three limbs.

Counts: Critical 0, Important 1 (proposed), Minor 0, Nit 0; verdict CONFIRMED; 1 open product point.
