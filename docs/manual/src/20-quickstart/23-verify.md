# Verifying a bundle

After producing a bundle (and ideally before engraving), confirm it
round-trips. The `mnemonic verify-bundle` subcommand re-derives the
expected card content from the seed and reports per-card pass/fail
plus the overall result.

## The command

Pass the original `--slot @0.phrase=…` plus the cards that came back
from your bundle invocation. (Same canonical test seed as
[Chapter 22](#your-first-bundle); see the DANGER box there.) For a
single-sig BIP-84 mainnet wallet with one ms1, two mk1 strings, and
three md1 strings:

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

- **`--ms1 <STRING>`** takes one ms1 card.
- **`--mk1 <STRING>`** can be repeated; for this BIP-84 wallet the
  bundle emits two mk1 strings, so you pass `--mk1` twice.
- **`--md1 <STRING>`** can be repeated; this wallet emits three md1
  strings.

The same `--network`, `--template`, and `--slot` flags as the
original `bundle` invocation are required so the verifier knows what
*should* match.

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

Each line is a discrete check. The four checks per card type are:

| Check | What it confirms |
|---|---|
| `ms1_decode` / `mk1_decode` / `md1_decode` | The card string parses back to a structured value (BCH checksum verified, internal layout matches the format). |
| `ms1_entropy_match` | The decoded entropy round-trips byte-for-byte from the supplied `--slot @0.phrase=…`. |
| `mk1_xpub_match` | The xpub on the mk1 card equals the xpub re-derived from the seed at the same origin. |
| `mk1_fingerprint_match` | The mk1 card's master fingerprint matches the seed's. |
| `mk1_path_match` | The mk1 card's origin path matches the template's expected derivation path. |
| `md1_wallet_policy` | The md1 card was emitted in wallet-policy mode (vs. legacy descriptor mode). |
| `md1_xpub_match` | The xpub bound in md1 matches the xpub re-derived from the seed. |

Final line `result: ok` means the bundle is internally consistent
and matches the seed. `result: mismatch` would mean one of the
sub-checks failed; the failing line names the cause.

## Why this matters before engraving

Verification protects against three failure modes:

1. **Transcription errors.** If you typed a card wrong from the
   bundle output, `*_decode` fails — the BCH checksum catches it.
2. **Wrong template / network.** If you bundled BIP-84 mainnet but
   typed `--template bip49` here, every `*_match` for the expected-
   xpub-side fails.
3. **Mixed-bundle confusion.** If you passed an mk1 card from a
   different wallet, `policy_id_stub` mismatches via the
   md1↔mk1 cross-binding (covered in the
   [recovery-paths workflow](#recovery-paths-by-damaged-card-scenario)).

Run `verify-bundle` once after every bundle synthesis. If it returns
`result: ok`, you are safe to engrave.
