# Stamping and recovering a 2-of-3 wallet

Each cosigner walks away from the bundle with **five plates**: their
own ms1 plus the three shared mk1s plus the one shared md1. This
chapter covers the per-cosigner plate set and the recovery
quick-table — what is still spendable, what is watch-only, and what
is bricked across the common damage scenarios.

## Per-cosigner plate set

> **Reminder.** Examples below still reference the public BIP-39
> test phrases used in [chapter 32's bundle](32-bundle.md) — see the
> DANGER box there for why they must never back a real wallet.

After stamping, every cosigner holds the same shape of set. Cosigner
N's plates:

| Plate | Held by | Engraved with |
|---|---|---|
| **Red (ms1)** | only cosigner N | their *own* ms1 (1 string) |
| **Blue × 3 (mk1)** | every cosigner | the three mk1 cards (2 strings each) |
| **Green (md1)** | every cosigner | the shared md1 (3-4 strings) |

The mk1 plates are public xpub material — distributing copies across
all three cosigners is harmless and useful (any cosigner can
reconstruct the full wallet's watch-only side from their own copy).
The md1 plate is also public; it carries no secrets, only the
wallet policy. The single plate that must be guarded is the cosigner's
own red ms1.

The stamping discipline from [chapter 25](../20-singlesig/25-stamp.md)
applies per plate: stamp, re-decode through `mnemonic verify-bundle`
with the steel-read strings, and re-strike on any sub-check failure
*before* moving to the next plate. The BCH error-position diagnostic
names the failing card and the character offset.

For geographic separation: each cosigner stores their own red
plate in their primary safe, and the blue/green plates in a
secondary location (bank box, attorney, off-site vault). With three
cosigners times five plates, the wallet is engraved across at least
six independent locations.

## Recovery quick-table

The single rule: **each cosigner's seed reconstructs that
cosigner's own ms1 and mk1**, and the md1 (the wallet policy) is
derivable from any one of the three xpubs plus the bundle's
template. So recovery scenarios fan out from "how many seeds
remain accessible, and is the threshold still met?".

| Lost / damaged | Wallet status | Recovery path |
|---|---|---|
| One cosigner's red ms1 | spendable (other two cosigners meet threshold) | The lost cosigner re-derives their ms1 from their seed via `mnemonic convert --from phrase=… --to ms1` and re-stamps. |
| One blue mk1 plate | spendable | The cosigner whose mk1 was lost re-derives their own from their seed via `mnemonic convert --from phrase=… --to mk1`; the mk1 carries no secret, but each is bound to one specific cosigner's xpub. |
| The green md1 plate | spendable | Re-derive from the three xpubs via `mnemonic export-wallet --template wsh-sortedmulti --threshold 2 --slot @N.xpub=…` (one slot per cosigner; no seeds needed). |
| Two cosigners' ms1s + the md1 still readable | spendable | The threshold is 2; any two surviving seeds spend the wallet. |
| Only one cosigner's ms1 + md1 readable | watch-only only | One seed cannot meet the 2-of-3 threshold; the wallet receives but does not spend. Move funds out via the surviving cosigners' coordination if any of their seeds can still be restated. |
| All three cosigners lose their ms1s and forget their seeds | bricked | No threshold-meeting subset of seeds survives. Funds are unrecoverable. |

The "watch-only only" row is the row to study most carefully: one
seed is sufficient to reconstruct the full *public* side of the
wallet (addresses, balances, transaction history) but not to sign.
Recovering from that state requires at least one more cosigner to
restate their seed, after which the surviving two meet threshold and
can spend.

For the long form of these scenarios — including what to do after
recovery, key rotation, and partial card damage within a single
plate — see the reference manual's
**Recovery paths by damaged-card scenario** workflow chapter.

## Onward

You have produced a 2-of-3 multisig bundle, stamped its seven
plates across three cosigners, and learned the recovery quick-table.
[Part IV — watch-only](../40-watch-only/41-singlesig-watch-only.md)
covers importing the public side of the wallet (xpubs + policy) into
Bitcoin Core, Sparrow, and Specter so addresses can be generated and
balances watched without ever exposing a seed.
