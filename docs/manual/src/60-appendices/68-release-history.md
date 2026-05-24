# Appendix H — Release history

This appendix is hand-curated for v0.1 of the manual; subsequent
versions will switch to auto-extraction from each repo's
`CHANGELOG.md` (filed as `release-history-auto-extract` in
`docs/manual/FOLLOWUPS.md`).

## mnemonic-toolkit

| Version | Date | Highlights |
|---|---|---|
| `0.8.0` | 2026-05-07 | **BREAKING** composite-edge BIP-38 passphrase split (`--passphrase` for BIP-39, `--bip38-passphrase` for BIP-38). `--passphrase-stdin` for the BIP-38 V3 NULL-byte case. Four non-English Electrum wordlists. `--taproot-internal-key` (`nums` or `@N`) on `export-wallet`. `--descriptor` + `--format bip388` interop. BIP-85 DICE shipped. |
| `0.7.1` | 2026-05-07 | Multi-repo BIP test-vector audit-cycle. ~43 net-new pinned vectors. Minor SPEC erratum corrections. |
| `0.7.0` | 2026-05-06 | New subcommands: `mnemonic export-wallet` (Bitcoin Core, BIP-388 stub, Sparrow / Specter stubs) and `mnemonic derive-child` (BIP-85 entropy / hd-seed / xprv / hex / password-base64 / password-base85). 4 new `convert` node types. |
| `0.6.x` | 2026-05-06 | `mnemonic convert` subcommand; BIP-39 / BIP-38 / WIF / xpriv edges; SLIP-0132 prefix-tolerant input. |
| `0.5.x` | 2026-05-06 | Unified `--slot @N.<subkey>=<value>` input shape. Legacy CLI flag retirement. |
| `0.4.x` | 2026-05-05 | BIP-388 + multi-leaf taproot + schema-4. |
| `0.3.x` | (early) | Descriptor-mode foundation. |
| `0.2.x` | (early) | Multisig foundation. |
| `0.1.x` | (early) | Single-sig foundation. |

## descriptor-mnemonic / md-codec / md-cli

The md1 format. Tracking notable releases:

| Version | Date | Highlights |
|---|---|---|
| `md-codec 0.16.2` | 2026-05-07 | v0.7.1 audit-cycle close-out; BIP-388 388.2 test vector pinned. |
| `md-codec 0.16.0` | 2026-05-03 | CLI extracted to `md-cli` crate; library-only `md-codec`. |
| `md-codec 0.11.x` | 2026-05 | Wire-format cleanup; path dictionaries dropped. |
| `md-codec 0.10.x` | 2026-04-29 | `OriginPaths = 0x36`; per-`@N` divergent-path encoding. |
| `md-codec 0.9.x` | 2026-04-29 | BIP-submission gate cleared; `chunk_set_id` rename. |

## mnemonic-key / mk-codec / mk-cli

The mk1 format. Standalone CLI `mk` shipped in v0.2 alongside
`mk-codec` v0.2.

| Version | Date | Highlights |
|---|---|---|
| `mk-cli 0.2.0`  | 2026-05-08 | Standalone CLI with `encode`, `decode`, `inspect`, `verify`, `vectors` subcommands; `--from-md1` cross-repo policy-id-stub derivation. |
| `mk-codec 0.2.2` | 2026-05-07 | Path-dictionary mirror retirement (post md-codec v0.11). v0.7.1 audit-cycle close-out. |
| `mk-codec 0.2.0` | 2026-04-30 | Stable encoder/decoder surface; path-dictionary v1. |
| `mk-codec 0.1.0` | 2026-04-29 | Initial release, v0.1 wire format. |

## mnemonic-secret / ms-codec / ms-cli

The ms1 format. BIP-93 codex32 directly via `rust-codex32`.

| Version | Date | Highlights |
|---|---|---|
| `ms-codec 0.1.0` + `ms-cli 0.1.0` | 2026-05-03 .. 04 | Initial release: BIP-93 codex32 single-string mode. K-of-N share splitting deferred to v0.2. |
| `ms-codec 0.1.1` | 2026-05-07 | v0.7.1 audit-cycle: extra corpus vectors; BIP-93 §"Test vectors" cross-format pinning. |

## Cross-repo coordination

The four-format star coordinates via the mirror-invariant protocol
documented in `descriptor-mnemonic/CLAUDE.md`: any cross-format
work surfaces in the originating repo's `design/FOLLOWUPS.md` with
a primary entry, and lands a companion entry in each affected
sibling. The retired path-dictionary mirror (md-codec v0.11) was
the last invariant of this kind to close in lockstep across mk1.

For the in-progress and shipped milestones across the four repos,
see each repo's `CHANGELOG.md` directly — the authoritative,
continuously-updated release record. This manual tracks the current
release; the CLI reference mirrors the live `--help` surface.
