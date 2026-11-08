# Making a lettered seed

A lettered seed is a 24-word seed whose first word falls in the block of words
that its letter names. Refugium's Create wizard (Build my own, step 4) gives
each seed on a card a letter, A to P, and the card names the seed by that
letter. This page shows how to make such a seed by hand, with coins, dice or
Entropia pills, so that no program ever chooses a word for you.

The block of a letter is 128 consecutive words of the BIP-39 English list.
The wizard shows the 128 words of the block, each with its 7-digit binary
number, under "Your seeds".

## Word 1: 7 coin flips

Flip a coin 7 times. Heads is 1, tails is 0, and the first flip is the leftmost
digit. The 7 digits are the number of your first word in the letter's block.
Look it up in the wizard's list: `0000000` is the first word of the block,
`1111111` the last.

## Words 2 to 23: 22 random words

Each of these words is 11 bits of randomness. Never choose them yourself.

| Source | How to make one word |
|---|---|
| Entropia pills | Shake the bag, take one pill, toss it in your hand, take only the word facing up (never the other side), and put the pill back. 22 draws in all. |
| Coins | Flip a coin 11 times, heads 1 and tails 0, first flip leftmost. |
| Dice | Roll a die. A 1, 2, 3 or 4 gives the 2 bits 00, 01, 10 or 11; a 5 gives the bit 0 and a 6 the bit 1. Write the bits left to right and roll until you have 11. If the last roll gives 2 bits where only 1 is needed, keep its first bit and drop the second: every word starts with fresh rolls. |

Coins and dice give 11 binary digits. Read them as a number from 0 to 2047:
from left to right the digits are worth 1024, 512, 256, 128, 64, 32, 16, 8, 4, 2
and 1. Number 0 is the first word of the BIP-39 English list (`abandon`) and 2047
the last (`zoo`); on a list numbered 1 to 2048, look up the number plus 1.

## Word 24: 3 coin flips

The last word holds a checksum, so it cannot be chosen freely. Put words 1 to 23
in a file and ask `mnemonic final-word` for the words that can end the seed:

```sh
mnemonic final-word --from phrase=- --language english < seed-B.txt
```

It prints 8 words, sorted. Flip a coin 3 times (heads 1, tails 0, first flip
leftmost). The number, 0 to 7, picks your last word: 0 is the first word
listed, 7 the last. See [`mnemonic final-word`](#mnemonic-final-word) for the
refusals.

## The signer

Use a signer that can load this wallet's descriptor. For a Coldcard, that means EDGE firmware (6.6.1X or later on a Mk4 or Mk5, 6.6.1QX or later on a Q).
