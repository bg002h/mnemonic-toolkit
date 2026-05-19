# BIP-129 BSMS Round-1 verification (v0.27.0)

BIP-129 BSMS ("Bitcoin Secure Multisig Setup") specifies a three-round
protocol between cosigners and a coordinator. **Round 1** is the
Signer → Coordinator handshake: each signer sends a 5-line key record
that bundles the cosigner's xpub (with origin annotation) and a
BIP-322 ECDSA signature over the first four lines of the record:

```text
BSMS 1.0                                       # Line 1 — protocol version
<TOKEN_HEX>                                    # Line 2 — hex-encoded session TOKEN
[<fingerprint>/<derivation-path>]<KEY>         # Line 3 — origin annotation + KEY (raw 33-byte pubkey OR xpub)
<DESCRIPTION>                                  # Line 4 — text description, ≤80 chars
<SIGNATURE>                                    # Line 5 — base64 BIP-322 ECDSA signature
```

`<KEY>` carries the BIP-380 `[fingerprint/path]` origin annotation INLINE
on line 3 (e.g., `[deadbeef/48'/0'/0'/2']xpub6E…`). The derivation
path is NOT a separate line; it is embedded in the `[...]` annotation
preceding the KEY.

The signature on line 5 is computed over the **first four lines**
concatenated with `\n` (no trailing newline), then BIP-322-hashed using
the standard Bitcoin signed-message digest. The coordinator's
responsibility is to verify that each signer's signature matches the
KEY on line 3, providing assurance that the recorded xpub really came
from the holder of the corresponding private key (or master seed).
v0.27.0's `mnemonic import-wallet --bsms-round1` verifies these
records.

## When to use this

- **As a coordinator** consolidating Round-1 records before assembling
  the multisig descriptor. Verify each record before trusting its
  xpub.
- **As an auditor** double-checking a recorded coordinator → signer
  exchange. The records are typically retained alongside the Round-2
  output for the cohort's audit trail.
- **As a signer** sanity-checking your own outgoing record before
  sending it to the coordinator (verify-on-emit catches a corrupted
  paste before the bytes leave your machine).

## Standalone verify mode

The simplest use case: you have one or more Round-1 record files and
want to verify them. `--bsms-round1` accepts a file path; pass it
repeating to verify multiple records:

```sh
mnemonic import-wallet \
  --bsms-round1 signer-1.txt \
  --bsms-round1 signer-2.txt \
  --bsms-round1 signer-3.txt
```

Standalone mode (no `--blob`) emits a per-record human-readable
summary on **stdout** showing the verify state for each record:

```text
bsms-round1: 3 record(s) processed
  record[0]: signer_pubkey=02… description="Coordinator 1" token_hex=… verified=true
  record[1]: signer_pubkey=03… description="Coordinator 2" token_hex=… verified=true
  record[2]: signer_pubkey=02… description="Coordinator 3" token_hex=… verified=true
```

Lenient default behavior on **failure** (per record): one **stderr
NOTICE** is emitted and the record's stdout line shows `verified=false`
plus the `failure_reason` field is populated; the toolkit continues
processing remaining records. Strict mode (`--bsms-verify-strict`,
below) flips failure to fatal `BsmsSignatureMismatch` exit 2 on the
first failing record.

## Strict mode (`--bsms-verify-strict`)

For audit / signing-ceremony contexts where any verification failure
is unacceptable, add `--bsms-verify-strict`. Verify failures become
fatal `BsmsSignatureMismatch` (exit 2):

```sh
mnemonic import-wallet \
  --bsms-round1 signer-1.txt \
  --bsms-round1 signer-2.txt \
  --bsms-verify-strict
```

On any record's signature mismatch the toolkit exits 2 with a stderr
template like:

```text
error: BIP-129 Round-1 signature mismatch for record 1
       (signer pubkey 03abcd…): signature does not verify against the supplied key
```

Without `--bsms-verify-strict`, the same condition only emits a
NOTICE and `signature_verified: false`. Pick the mode that matches
your trust model — lenient for "process and report", strict for
"refuse the entire batch on any failure".

## JSON envelope mode

Pass `--json` to emit a structured envelope on stdout, suitable for
piping into other tooling:

```sh
mnemonic import-wallet \
  --bsms-round1 signer-1.txt \
  --bsms-round1 signer-2.txt \
  --json
```

Output (formatted):

```json
{
  "source_format": "bsms-round1",
  "bsms_round1_verifications": [
    {
      "index": 0,
      "signer_pubkey": "02abc…",
      "description": "Cosigner 1",
      "token_hex": "deadbeef…",
      "signature_verified": true,
      "failure_reason": null
    },
    {
      "index": 1,
      "signer_pubkey": "03def…",
      "description": "Cosigner 2",
      "token_hex": "deadbeef…",
      "signature_verified": true,
      "failure_reason": null
    }
  ]
}
```

The `failure_reason` field is `Some(<string>)` on lenient-mode failures
(strict mode exits early before emitting the envelope).

## Combined with Round-2 import

When `--bsms-round1` is supplied alongside `--blob` (a BSMS Round-2
blob), both runs proceed and the Round-1 verifications propagate
into every emitted envelope's `bsms_round1_verifications` field:

```sh
mnemonic import-wallet \
  --blob round-2-coordinator-output.bsms \
  --format bsms \
  --bsms-round1 signer-1.txt \
  --bsms-round1 signer-2.txt \
  --bsms-round1 signer-3.txt \
  --json
```

This is the canonical full-audit flow: ingest the Round-2 output,
verify each Round-1 record against the same session, and emit a
single envelope carrying the synthesized bundle + per-record Round-1
verification state.

## Verification protocol details

The toolkit verifies BIP-322 ECDSA recoverable signatures against
the **xpub's own embedded pubkey** (bytes 45–78 of the serialized
xpub), per BIP-129 §Specification → Round 1. The signed body is the
**first four lines of the record** joined by `\n` (no trailing
newline); the BIP-322 legacy-format digest is computed over that
body via the canonical Bitcoin signed-message hash:

```text
body   = "<line 1>\n<line 2>\n<line 3>\n<line 4>"
digest = SHA-256(SHA-256("\x18Bitcoin Signed Message:\n" || varint(len(body)) || body))
```

(Implementation: `crates/mnemonic-toolkit/src/wallet_import/bsms_round1.rs::signed_body`,
plus `bsms_verify::verify_round1_signature`.)

The signature is base64-decoded into a 65-byte recoverable signature
(`[recovery-id: 1] [r: 32] [s: 32]`), recovered, and compared against the
declared pubkey. Mismatch produces `BsmsSignatureMismatch`.

The verifier accepts both raw-pubkey KEYs (line 3 carries a 33-byte
compressed secp256k1 pubkey hex inside the `[fingerprint/path]` origin
annotation) and xpub KEYs (line 3 carries a base58 xpub inside the
origin annotation); pubkey extraction uses the xpub's OWN embedded
pubkey, NOT any child-derived key.

## Limitations (v0.27.0)

- **No standalone `--bsms-round2` verifier.** Round-2 verification
  is implicit in `import-wallet --format bsms`'s parse + canonicalize
  step (the blob's BIP-380 checksum + the toolkit's round-trip
  canonicalizer assert structural integrity). The optional BIP-129
  Round-2 6-line signature field (`<SIGNATURE>` on line 6) is preserved
  in `bundle.bsms_audit.signature` for downstream tooling but is not
  cryptographically verified by `import-wallet` itself (FOLLOWUP
  `bsms-round2-signature-verify` for v0.28+).
- **No multi-record stdin intake.** `--bsms-round1 -` is rejected
  with a clear error message in v0.27.0; supply a file path per
  record. Stdin support (one record per blob, separated by a sentinel)
  is FOLLOWUP work.
- **BIP-322 recoverable signatures only.** The bare ECDSA BIP-322
  "signed message" form; not the script-aware BIP-322 v0.7 form. BIP-129
  predates BIP-322 v0.7 and uses the recoverable-ECDSA shape exclusively.

## Related

- [`mnemonic import-wallet`](../40-cli-reference/41-mnemonic.md#mnemonic-import-wallet)
  — full flag reference incl `--bsms-round1` / `--bsms-verify-strict`
- [Cross-format wallet conversion](./39-cross-format-conversion.md)
  — Round-2 ingest + envelope-mediated emit
