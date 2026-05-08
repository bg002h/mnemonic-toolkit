# About this Quick Start

This is a hands-on onboarding to the **m-format star** — a family of
checksum-protected backup cards designed for steel engraving. By the
end you will have produced a working multi-card Bitcoin backup, and
you will understand what each card does.

## Audience

You have heard of Bitcoin self-custody — the idea that *you* hold the
keys to your coins, not an exchange — and you want to engrave your
first multi-card backup. No prior Bitcoin background is required.
Everything technical is introduced as you need it.

## Prerequisites

- A Linux or macOS terminal. (Windows works via WSL but is not
  covered here.)
- Basic comfort with shell commands: copying a command, running it,
  reading the output.
- Roughly an hour of uninterrupted time.

You do not need: existing Bitcoin software, a hardware wallet, or
prior cryptography knowledge.

## What you'll have at the end

Three things, depending on how far you read:

- **Part II (single-sig).** A complete steel-engraved bundle for a
  single-signature wallet — the simplest setup that the m-format
  star supports.
- **Part III (multisig).** A 2-of-3 multisig bundle: three independent
  cards that together describe a wallet requiring two signatures of
  three to spend.
- **Part IV (watch-only).** Configurations that import either of the
  above into popular wallet software (Sparrow, Bitcoin Core) for
  monitoring without exposing the secret.

## Reading order

Top to bottom, roughly 90 minutes. Each chapter forward-points to
the next; later chapters assume the earlier ones. Skim if you are
already comfortable with a section's topic, but do not skip out of
order — the worked examples build on each other.

## What this guide is not

This is not the full reference. For exhaustive coverage of every
CLI flag, every supported descriptor type, the BIP-85 child-secret
flow, taproot multisig, or recovery edge cases, see the
**reference manual** (`docs/manual/`) — a 129-page sibling document
that this Quick Start cross-references.

Onward to the foundations.
