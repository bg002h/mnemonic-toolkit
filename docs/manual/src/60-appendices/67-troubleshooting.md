# Appendix G — Troubleshooting matrix

Common failure modes when building, verifying, or recovering an
m-format constellation bundle, ordered by frequency.

## Bundle synthesis fails

| Symptom | Likely cause | Fix |
|---|---|---|
| `error: a value is required for '--template <TEMPLATE>'` | Forgot `--template` (or `--descriptor`) | Pick one of `bip44`/`bip49`/`bip84`/`bip86`/`wsh-multi`/`wsh-sortedmulti`/`sh-wsh-multi`/`sh-wsh-sortedmulti`/`tr-multi-a`/`tr-sortedmulti-a`, or supply `--descriptor`. |
| `error: a value is required for '--threshold <THRESHOLD>'` | Multisig template without `--threshold K` | Pass `--threshold K` (1 ≤ K ≤ N). |
| `error: invalid mnemonic` | Phrase typo, wordlist mismatch, or non-canonical wordlist | Check word spellings; pass `--language <LANGUAGE>` if the phrase isn't English. |
| Bundle synthesis silently produces a different xpub from the wallet you intended | Wrong `--template`, `--account`, or `--network` | Run `mnemonic convert --from phrase=… --to xpub --template <T>` and compare to your wallet's xpub *before* stamping. |

## verify-bundle fails

| Symptom | Likely cause | Fix |
|---|---|---|
| `ms1_decode: error at position N` | Mis-stamped or mis-typed character at position N | Inspect that character against the original digital bundle; correct. |
| `mk1_xpub_match: mismatch` | Wrong cosigner xpub bound to the slot, or wrong template/account/network | Re-derive the expected xpub from the slot's seed; if it differs from the mk1's xpub, the mk1 was bound for a different cosigner. |
| `md1_xpub_match: mismatch` | md1 carries a different cosigner's xpub for the slot than the multisig set expects | Verify each cosigner's slot index; the md1 binds slot `@N` to a specific xpub and the verifier compares to the slot input. |
| `policy_id_stub: mismatch` | Cards from different bundles mixed | Confirm all cards came from a single `mnemonic bundle` invocation. |

## convert / recovery

| Symptom | Likely cause | Fix |
|---|---|---|
| `error: BIP-38 decryption failed` | Wrong `--bip38-passphrase` or wrong source key | Try the alternative passphrase channel (v0.7 used a single `--passphrase`; v0.8 split). |
| Empty stdout from `convert --to phrase` | Source node was an xpub (no privkey to derive phrase from) | Phrases require entropy or seed; xpub is public-only. |
| Wrong-network xpub prefix on output | `--network` mis-set or `--xpub-prefix` mis-set | Pass the matching network; for SLIP-0132 prefixes (zpub, ypub, …) use `--xpub-prefix`. |

## Engraving / stamping

| Symptom | Likely cause | Fix |
|---|---|---|
| Verifier passes on digital bundle but fails on engraved cards | Stamping error (e.g., `0` vs `o`) | Re-decode the engraved card; the BCH error position narrows to the offending character. |
| Plate has a single ambiguous character | Glance bias (e.g., `s` could be `5`) | Use a magnifier; the codex32 / m-format alphabets exclude visually-similar characters but stamping artefacts can still confuse. |
| Plate damaged beyond the BCH correction radius | Physical destruction beyond the codec's repair limit | Re-derive the card from the seed and re-stamp a fresh plate. |

## Wallet-export

| Symptom | Likely cause | Fix |
|---|---|---|
| Bitcoin Core returns "non-canonical descriptor" | Old Bitcoin Core version | Pass `--bitcoin-core-version 24` for Bitcoin Core 24, or upgrade. |
| Sparrow/Specter rejects the file | `--format sparrow` / `--format specter` are accepted by the binary but currently return a deferral stub | Use `--format bip388` (or `--format bitcoin-core`); import the resulting JSON via the wallet's BIP-388 / descriptor-import dialog. |
| Address scan misses recent receives | `--range` too small | Pass a larger `--range`, or re-scan the chain via `bitcoin-cli rescanblockchain`. |

## BIP-85 derive-child

| Symptom | Likely cause | Fix |
|---|---|---|
| `application not in scope` for `rsa` / `rsa-gpg` | These are deferred to v0.9 (RUSTSEC-2023-0071) | Use a different application or wait for v0.9. |
| `--length 0 invalid for application bip39` | `bip39` requires `--length 12`, `18`, or `24` | Use a valid word count. |
| `--dice-sides required for application dice` | DICE app needs the side count | Pass `--dice-sides N` (2..=2^32-1). |

## When in doubt

1. **Re-read the chapter.** Most failures trace to a small flag-set
   discrepancy between the example and the actual command.
2. **Run `--help` directly.** Each CLI's `--help` (`mnemonic` /
   `md` / `ms` / `mk`, and any subcommand) is authoritative for the
   installed version; the CLI-reference chapters (chapter 40) mirror
   the current flag surface.
3. **Run `verify-bundle` on the engraved cards.** It is the
   single most useful diagnostic.
4. **Compare against the canonical test seed.** If a worked example
   from the manual doesn't reproduce, the toolkit version may have
   drifted; check the manual's release-history appendix
   ([Appendix H](#appendix-h-release-history)) for which toolkit
   version this manual was built against.
