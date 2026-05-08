# Recovery paths by damaged-card scenario

Cards get scratched, water-damaged, partly-stamped, lost, or
confiscated. This chapter walks through what each failure mode means
for the wallet and how to recover, ordered roughly by frequency.

The single principle behind every scenario: **the seed reconstructs
everything else**. Public-key material (mk1, md1) is derivable from
the seed; only the seed itself is irreplaceable. So recovery paths
fan out from "do you still have a seed?".

```mermaid
flowchart TD
  A{Seed available?} -- yes --> B{Single-sig or<br/>multisig?}
  A -- no --> C{All cosigners' seeds<br/>still available?}
  B -- single-sig --> D[Re-derive mk1 + md1<br/>from the seed]
  B -- multisig --> E[Plus other cosigners'<br/>mk1s + the shared md1]
  C -- yes --> F[Multisig still recoverable<br/>via cosigner cooperation]
  C -- no --> G{Threshold<br/>still met?}
  G -- yes --> H[Spend with cooperating<br/>K cosigners' seeds]
  G -- no --> I[Wallet bricked]
```

## Single-sig wallet — single card lost

**Lost ms1**: re-derive from the seed phrase you remember:

```sh
mnemonic convert \
  --from phrase="<your phrase>" \
  --to ms1
```

Re-stamp.

**Lost mk1**: re-derive from the seed:

```sh
mnemonic convert \
  --from phrase="<your phrase>" \
  --to mk1 \
  --template bip84
```

Re-stamp.

**Lost md1**: derive a new one from xpub:

```sh
xpub=$(mnemonic convert --from phrase="<your phrase>" --to xpub --template bip84)
mnemonic bundle \
  --network mainnet \
  --template bip84 \
  --slot @0.xpub=$xpub
```

The `mk1` and `md1` emitted should match the originals (cross-check
with `verify-bundle` before discarding the originals); re-stamp the
md1 and the mk1 stays intact.

## Single-sig wallet — seed is the only thing left

If you have *only* the seed phrase (no cards), import the phrase
into a wallet that supports BIP-39 directly. Standard wallet apps
(Bitcoin Core, Sparrow, Electrum, hardware wallets) recover from
phrase; the m-format constellation bundle is a *backup*, not a *requirement*.

If you previously used a non-default template (BIP-86 taproot, for
example), specify it on import; the seed alone does not record the
template, only the m-format md1 does.

## Multisig wallet — one cosigner's ms1 lost

The other cosigners still have their seeds; the shared mk1 set and
md1 are intact. Recovery:

1. The cosigner with the missing ms1 re-derives it from their seed
   (if memorised) and re-stamps.
2. If the cosigner cannot recall their seed but the wallet's
   *threshold* remains met by the surviving cosigners, the wallet
   stays spendable. The compromised cosigner is replaced by a fresh
   key in a follow-up "key rotation" — synthesise a fresh seed,
   generate a new bundle replacing the lost cosigner's slot, and
   transfer all funds to the new wallet.

## Multisig wallet — md1 is lost or unreadable

Re-derive from any cosigner's xpub set:

```sh
mnemonic export-wallet \
  --template wsh-sortedmulti \
  --threshold 2 \
  --slot @0.xpub=<xpub-0> \
  --slot @1.xpub=<xpub-1> \
  --slot @2.xpub=<xpub-2> \
  --format bip388
```

Or rebuild the full bundle:

```sh
mnemonic bundle \
  --network mainnet \
  --template wsh-sortedmulti \
  --threshold 2 \
  --slot @0.xpub=<xpub-0> \
  --slot @1.xpub=<xpub-1> \
  --slot @2.xpub=<xpub-2>
```

The cross-binding `policy_id_stub` re-derives identically as long as
the cosigner xpubs and template are unchanged.

## Multisig wallet — partial card damage

The BCH error-correction codec located damage to a small number of
characters. The codec's error position diagnostic identifies *which*
character is wrong:

```sh
mnemonic convert --from ms1=ms10entrsqqQqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f --to phrase
```

Outputs (illustrative):

```text
error: ms1 BCH checksum failed
  position 11: invalid character 'Q' (expected 'q')
```

Manually correct the character (the codec narrows the candidate set
to typically 1-2 characters at the named position). For damage
beyond the codec's correction radius (more than a few errors), the
card is unrecoverable from itself — fall back to re-deriving from
the seed (or other cosigners' cards in multisig).

## Worst-case scenarios

| Scenario | Recoverable? |
|---|---|
| Single-sig, all cards intact | yes (trivially) |
| Single-sig, ms1 only | yes — import phrase into BIP-39 wallet |
| Single-sig, mk1 + md1 only | yes (watch-only); spending requires the seed |
| Single-sig, no ms1 + no seed | no |
| 2-of-3 multisig, 2 cosigners' ms1s + md1 | yes |
| 2-of-3 multisig, 1 cosigner's ms1 + md1 | watch-only only; spending requires another seed |
| 2-of-3 multisig, all 3 cosigners lose ms1 | no — wallet bricked |

## After recovery

Treat the recovered wallet as a *signal* that the original engraving
discipline was compromised. After moving funds:

1. Generate a fresh wallet with new entropy.
2. Stamp a new bundle.
3. Move funds.
4. Destroy the damaged plates carefully (they may still contain
   secrets even though the wallet is empty).
