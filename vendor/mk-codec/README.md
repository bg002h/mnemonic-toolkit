# mk-codec

Reference implementation of the **Mnemonic Key (MK)** backup format —
codex32-derived BCH-checksummed strings encoding individual extended
public keys (xpubs) for engraving alongside MD-encoded policy cards.

> **Status:** design-stage skeleton, no implementation yet. The crate
> exists so the public API surface is concretely visible alongside the
> spec and BIP draft, and so the workspace structure is in place when
> implementation work begins. All public functions panic with `todo!()`.

## Design surface

- [`design/SPEC_mk_v0_1.md`](../../design/SPEC_mk_v0_1.md) — wire-format spec
- [`design/DECISIONS.md`](../../design/DECISIONS.md) — rolling decisions log
- [`bip/bip-mnemonic-key.mediawiki`](../../bip/bip-mnemonic-key.mediawiki) — BIP draft skeleton

## Sibling project

- [`bg002h/descriptor-mnemonic`](https://github.com/bg002h/descriptor-mnemonic) — the MD policy-template format and its reference implementation. MK is designed to engrave alongside MD policy cards for foreign-xpub multisig recovery.

## Eventual factoring

Per [`design/DECISIONS.md`](../../design/DECISIONS.md) §D-13, this crate will fork BCH primitives from the sibling [`md-codec`](https://github.com/bg002h/descriptor-mnemonic/tree/main/crates/md-codec) once implementation work begins, and the shared codex32-derived plumbing extracts to a third crate (likely a new sibling repo `mc-codex32`) once both formats are implementation-validated.

## See also

For the Rust API reference, see [docs/MK_CODEC_RUST_API.md](https://github.com/bg002h/mnemonic-key/blob/main/docs/MK_CODEC_RUST_API.md).

## License

MIT License.
