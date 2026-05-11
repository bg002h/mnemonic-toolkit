# Read this first — UNTESTED ALPHA SOFTWARE

**This software has not yet been independently tested, audited, or proven in production.** The m-format constellation is alpha-quality work-in-progress: the four codecs, the four CLIs, the BCH error-correcting math, the BIP-388 wallet-policy emission path, and the cross-card binding invariants have all been authored and reviewed only by the original developer (and AI agents under that developer's supervision).

**Do not use this software to back up significant sums of money at this time. Doing so is tantamount to asking to be rekt.**

The technical manual targets implementers, auditors, and Rust integrators — the audiences who will *find* the bugs that an end-user manual could only document around. Read this manual with the assumption that any wire-format claim, any BCH-math claim, any canonicality rule, and any cross-card invariant *may be wrong*. Cross-check against:

- The per-format BIP drafts (md1 draft in `bg002h/descriptor-mnemonic/bip/`).
- The per-version `design/SPEC_*.md` documents in each repo.
- The Rust reference implementation source (`crates/*/src/`).
- The test vectors and corpus files (`crates/*/tests/vectors/`, `design/CORPUS.md`).

If you find a wrong claim — in this manual, in a BIP draft, in a SPEC, or in the Rust impl — open an issue on the relevant GitHub repo. Cross-implementation work (e.g., re-implementing md1 in another language) is *especially* welcome as a bug-finding mechanism.

## Acceptable uses today

- Disposable amounts only (a few sats, on mainnet or testnet).
- Evaluation, learning, code review.
- Cross-implementation work and conformance testing.
- Reproducing the worked-example transcripts in this manual.
- Integration smoke-testing.

## Unacceptable uses today

- Backing up funds you cannot afford to lose.
- Production wallets without independent external review.
- Any context where "this software lost my money" is not an acceptable outcome.

The status will change with independent review and the accumulation of cross-implementation conformance evidence. Until then, treat every claim in this manual as provisional.
