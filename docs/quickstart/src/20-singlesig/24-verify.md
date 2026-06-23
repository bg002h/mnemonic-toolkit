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
  --md1 md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np \
  --md1 md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d \
  --md1 md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn
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

```{.text include="qs-24-verify.out" lines="2-"}
```

Each line is a discrete check. `*_decode` lines confirm the BCH
checksum verifies and the card parses. The `*_match` lines confirm
the decoded content equals what the seed and template would produce
at this network. `result: ok` on the final line is the green light.

If something is wrong — a transcription typo, a flag mismatch with
the original bundle, a card from a different wallet — the failing
sub-check names what disagrees and the run exits with a non-zero
status.

:::primer
When a `*_decode` failure includes a *position* — for instance
`mk1_decode: error at position 47` — the number is a 0-based index
into the card string and pinpoints the single character whose
checksum disagrees. This is the BCH code's locator output: a
single-character error can be detected and located precisely. So
"position 47" means look at character 47 of the failing card,
re-stamp it, and re-run the check. You don't have to guess which
character broke; the diagnostic names it.
:::

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
