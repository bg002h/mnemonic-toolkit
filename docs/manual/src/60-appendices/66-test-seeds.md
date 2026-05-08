# Appendix F — Test seeds and example data

Every worked example in this manual uses public test seeds. The
canonical example is the BIP-39 12-word *all-zeros* seed:

```text
abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
```

This is the standard test vector from the BIP-39 specification.
*Anyone* with a Bitcoin wallet that supports BIP-39 can reproduce
the addresses derived from it.

:::danger
The phrase `abandon abandon abandon … about` — and any other phrase
in this manual — is **public**. Chain watchers continuously sweep
the addresses derived from these seeds; any funds sent to them are
stolen near-instantly. **Never engrave or fund a wallet built from
a test seed.** Generate fresh entropy on an air-gapped device
before doing any real work.
:::

## Why a public test seed

Test seeds make the manual *reproducible*. A reader who runs
`mnemonic bundle --slot @0.phrase="abandon abandon … about"` gets
the same ms1, mk1, md1 strings printed in the manual.
Mismatches surface as a bug or a manual-version drift, not as
"different seeds."

Production seeds, by contrast:

- Should be generated on an air-gapped machine.
- Should never appear in a manual, a script, a chat log, or a
  screenshot.
- Should be derived from hardware sources (dice rolls, hardware
  RNGs in dedicated devices) — software RNGs on networked machines
  have a long history of compromise.

## Other test vectors used in this manual

For multisig examples, the manual uses two other canonical BIP-39
test vectors:

```text
legal winner thank year wave sausage worth useful legal winner thank yellow
letter advice cage absurd amount doctor acoustic avoid letter advice cage above
```

Both are public, both are swept. They are used for cosigner-1 and
cosigner-2 in the multisig walkthroughs (chapter 32) and the
taproot-multisig walkthrough (chapter 33).

## Network-aware test data

Test seeds are network-agnostic — the same entropy produces
addresses on mainnet, testnet, signet, and regtest. The toolkit
takes `--network <NETWORK>` separately; the manual's worked
examples mostly use `--network mainnet` for the addresses to
match what production tooling expects, but readers can substitute
testnet for any of them safely.

## Reading test transcripts

The `docs/manual/transcripts/` directory carries pinned `.out`
transcripts of the worked examples, one per example. Each transcript
was captured against the toolkit binary version named in the
chapter (`mnemonic 0.8.0` for the v0.1 of the manual). The
`make verify-examples` target re-runs each command and diffs against
its transcript; drift is a CI failure.

Updates to a worked example require updating both the chapter prose
and the matching transcript in lockstep.
