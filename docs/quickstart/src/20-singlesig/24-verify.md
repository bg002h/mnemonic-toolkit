# Verifying the bundle

Run `mnemonic verify-bundle` *before* you engrave. It re-derives the
expected card content from the seed and confirms the bundle you
produced matches.

## The command

Pass the same `--network`, `--template`, and `--slot @0.phrase=…`
as the original `bundle` invocation, plus the cards that came back
in the bundle output:

> **Reminder.** Still the public BIP-39 test phrase — see
> [Generating entropy safely](22-generate-entropy.md).

```sh
mnemonic verify-bundle \
  --network mainnet \
  --template bip84 \
  --slot @0.phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" \
  --ms1 ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f \
  --mk1 mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4 \
  --mk1 mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh \
  --md1 md1zsxdspqqqpm6jzzqqvqz6qu79mg9p2sgfff6p2eph8wftp5uf6gqnlgzqqqnymv0 \
  --md1 md1zsxdspq259s3jnsrcrhnlagpftrf9apnc3m9fy8uqfc85cha4nqnh5k67ey2hzyc \
  --md1 md1zsxdspqjd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nvqhuuyvzgaejah6
```

The flag set:

- **`--ms1 <STRING>`** — the single ms1 card.
- **`--mk1 <STRING>`** — repeat once per mk1 string. BIP-84 emits
  two, so `--mk1` appears twice.
- **`--md1 <STRING>`** — repeat once per md1 string. BIP-84 emits
  three, so `--md1` appears three times.

The `--network`, `--template`, and `--slot` flags must match the
original `bundle` invocation so the verifier knows what to expect.

## Output

```text
ms1_decode: ok decoded successfully
ms1_entropy_match: ok ms1 byte-identical
mk1_decode: ok decoded successfully
mk1_xpub_match: ok xpub matches
mk1_fingerprint_match: ok fingerprint matches
mk1_path_match: ok path matches
md1_decode: ok decoded successfully
md1_wallet_policy: ok wallet-policy mode confirmed
md1_xpub_match: ok 65-byte xpub matches expected
result: ok
```

Each line is a discrete check. `*_decode` lines confirm the BCH
checksum verifies and the card parses. The `*_match` lines confirm
the decoded content equals what the seed and template would produce
at this network. `result: ok` on the final line is the green light.

If something is wrong — a transcription typo, a flag mismatch with
the original bundle, a card from a different wallet — the failing
sub-check names what disagrees and the run exits with a non-zero
status.

## Why this matters before engraving

Verification protects against three failure modes you do *not*
want to discover after stamping steel:

1. **Transcription errors.** A copy-paste typo flips one character.
   The BCH checksum on the affected card catches it; `*_decode`
   fails and the diagnostic names the position.
2. **Wrong template or network.** If you bundled `bip84` mainnet
   but mistakenly invoke `verify-bundle` with `--template bip49`,
   every `*_xpub_match` fails because the verifier re-derives at
   the wrong path.
3. **Mixed cards from different wallets.** The cross-binding via
   `policy_id_stub` means an mk1 from one bundle can't be combined
   with an md1 from another. Mixed cards fail the cross-check.

Run `verify-bundle` once after every bundle synthesis. If
`result: ok`, you are safe to stamp.

Onward: stamp the cards onto steel.
