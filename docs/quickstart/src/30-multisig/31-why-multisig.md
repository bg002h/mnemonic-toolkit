# Why multisig

Single-sig is one seed, one signer, one point of failure. Multisig
splits signing authority across several independent cosigners, so
losing — or compromising — any one of them does not lose the wallet.
This chapter motivates the 2-of-3 layout the rest of Part III walks
through.

## When single-sig isn't enough

A BIP-84 single-sig wallet is recovered by anyone who reads the seed
phrase. That is its whole security model: the phrase is the wallet.
The model breaks under any of these:

- **Compromise.** Someone photographs the steel plate, sees the
  phrase on a connected screen, or coerces it out of the holder.
- **Single-point loss.** Fire takes the only plate; the holder
  forgets where they hid it; the holder dies without sharing it.
- **Insider trust.** A custodian, family member, or business partner
  who handles the phrase can drain the wallet unilaterally.

There is no defense against any of these in single-sig. The
[Stamping the steel plates](../20-singlesig/25-stamp.md) ceremony
helps with *durability* (geographic separation of `ms1`/`mk1`/`md1`)
but not with the underlying "one seed = one signer" structure.

## What 2-of-3 means

A *K-of-N* multisig wallet has `N` cosigners, any `K` of whom can
sign together to spend. The remaining `N − K` cosigners contribute
nothing to a given spend. For 2-of-3:

- Three cosigners, each holding their own independently-generated
  seed.
- Any two of them, cooperating, can spend.
- A single cosigner, alone, cannot spend.

The wallet's address is derived from *all three* cosigner xpubs plus
the spending rule, so the receive side is unchanged from the user's
perspective — they hand out one address as usual. The signing side
is what differs: a spend transaction collects two of the three
cosigners' signatures, and the network accepts it.

## Air-gapped vs. coordinated

Two operational modes for producing the bundle:

- **Coordinated.** One laptop sees all three phrases briefly during
  the bundle pass. Faster; trusts the coordinator's machine for the
  duration of the synthesis.
- **Air-gapped.** Each cosigner runs a `convert` step on their own
  machine to derive *only* their xpub; the coordinator builds a
  watch-only bundle from the three xpubs (no secrets ever leave
  any cosigner's machine); each cosigner separately derives their
  own ms1 locally.

Part III walks through the **coordinated** flow because it is the
shorter, copy-pasteable shape suited to a Quick Start. The full
air-gapped variant is in [Part IV — watch-only multisig](../40-watch-only/42-multisig-watch-only.md)
and the reference manual's multisig workflow chapter.

## Why 2-of-3 specifically

The "2-of-3" choice is the canonical small-team layout for two
reasons:

- **1-of-N defeats the purpose.** Any single cosigner spending alone
  is identical to single-sig with extra steps. The whole point of
  multisig is that no individual key is sufficient.
- **K = N loses the recovery property.** A 3-of-3 wallet is bricked
  the moment any one cosigner is unavailable. With 2-of-3, losing
  one cosigner is survivable: the other two still meet threshold.

Larger layouts (3-of-5, 4-of-7) extend the same logic; the toolkit
caps at `1 ≤ K ≤ N ≤ 16`. For most personal and small-business
setups, 2-of-3 is the sweet spot: one cosigner per geography or per
trust domain, with one redundant slot.

Onward: produce the seven-card 2-of-3 bundle.
