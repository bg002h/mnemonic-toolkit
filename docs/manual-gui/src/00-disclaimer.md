# Read this first — UNTESTED ALPHA SOFTWARE

**This software has not yet been independently tested, audited, or
proven in production.** The m-format constellation is alpha-quality
work-in-progress: the four codecs, the four CLIs, the BCH
error-correcting math, the BIP-388 wallet-policy emission path, the
cross-card binding invariants, AND the `mnemonic-gui` overlay that
wraps them have all been authored and reviewed only by the original
developer.

**Do not use this software to back up significant sums of money at
this time. Doing so is tantamount to asking to be rekt.**

## Acceptable uses today

- Disposable amounts only (a few sats, on mainnet or testnet).
- Evaluation, learning, code review.
- Reproducing the published worked-example transcripts.
- Integration smoke-testing.
- GUI-affordance UX feedback to the developer.

## Unacceptable uses today

- Production multisig wallets covering meaningful balances.
- Inheritance plans, legacy-estate setups.
- Any wallet you would be unhappy to lose entirely.

## Why this page is here

The format itself — the wire encoding, the BCH math, the BIP-388
mapping, the cross-card cryptographic binding — is intended to
mature through external review and independent implementation. The
GUI is a thin overlay over the CLIs and inherits both their bugs and
their alpha-software caveat; on top of that, the GUI itself has its
own OS-integration concerns (window-screenshot occlusion, clipboard
secret-handling, paste-warn confirmation flows) that have not been
externally reviewed either. Reference implementations carry no
warranty and no guarantee of correctness, ever — but the situation
is especially acute today, before any independent audit has happened.
When the project reaches a stable release with documented external
review, this page will be replaced with a stability-claim page.

Until then: assume bugs. Read the source. Verify your engraving with
your own re-implementation if it matters.
