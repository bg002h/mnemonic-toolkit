# Provenance — the journey address lists

The four `journey_*.txt` files are byte-copies of the operator-journey capture
in the sibling repo `mnemonic-engrave`, taken 2026-08-22:

| fixture | source (`mnemonic-engrave/design/journeys/out/rcw/…`) | sha256 at copy time |
| --- | --- | --- |
| `journey_wsh_receive.txt` | `wsh/receive.txt` | `e16a8835b4fd912ebd2b64851fc79c35e216d001d8a08d780dbf563526c2ada4` |
| `journey_wsh_change.txt` | `wsh/change.txt` | `2045113cc39e57e445290828c93024360ee79cb9ecab74f22ba85fca913fccca` |
| `journey_tr_receive.txt` | `tr/receive.txt` | `3718045f98d3cd01a0190121fb119dfd8292d80f0af7605ff1f70633a1c8d4cc` |
| `journey_tr_change.txt` | `tr/change.txt` | `b2f62884d67c08e2f35970dcedd94b943cfe5222687b806d1908ea6ac7871dfe` |

**Why they are worth copying.** These five-per-chain lists are the fourth
independent derivation of the same wallet: the SeedHammer II device, a BIP-129
BSMS canary, the journey capture, and now
`export-wallet --format bitcoin-core-addresses`. Disagreement is a defect in
the newest implementation until proven otherwise.

The two descriptors these addresses derive from are NOT duplicated here — they
already live at `tests/fixtures/export_wallet_allow/rcw_{wsh,tr}_descriptor.txt`
and were verified byte-identical to
`mnemonic-engrave/design/journeys/out/rcw/{wsh,tr}/descriptor.txt` on
2026-08-22.
