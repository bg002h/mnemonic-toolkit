# codex32 paper-computer reference (bundled)

This directory bundles the upstream codex32 hand-computation PDF for offline access by ms1 hand-decoding users.

- **File**: `2023-03-07--color.pdf`
- **Title**: "Codex 32: A Shamir Secret Sharing Scheme"
- **Authors**: Leon Olson Curr & Pearlwort Snead (text); Micaela Paez (cover + volvelle illustrations); M. Lutfi' As'ad (illuminated letters + inline illustrations); Arri Isak Beck (editor + producer).
- **Copyright**: © 2022 Blockstream. Licensed under the MIT License — see `LICENSE` in this directory or page 2 of the PDF.
- **Upstream URL**: https://www.secretcodex32.com/docs/2023-03-07--color.pdf
- **Retrieved**: 2026-05-08
- **SHA-256**: `9156c7ccf7dbf7fa5eb183af45296bfa132c348bd4583f737c1741482b92c236`
- **Revision** (from PDF page 2): `2303-1-8822ef51`

## Why this is bundled here

ms1 (HRP `ms`) is the m-format constellation's secret-material card and uses BIP-93 codex32 directly via `rust-codex32` (identical generator polynomial, identical target residue `MS32_CONST = 0x10ce0795c2fd1e62a`, identical alphabet). The codex32 paper-computer toolkit — Checksum Table, Worksheet, Addition wheel, dice de-biasing worksheet — therefore works for ms1 strings unmodified. Bundling the PDF here gives offline access without depending on `secretcodex32.com` staying online.

The other m-format cards (mk1 with HRP `mk`; md1 with HRP `md`) use forked BCH plumbing (different target residues for cryptographic format-separation) and are public material (xpub + origin metadata; wallet policy template) where hand-decodability is not load-bearing. No per-format paper-computer is shipped for those cards.

See `docs/manual/src/60-appendices/65-bch-codex-primer.md` §"Hand-decodability" for the operational guidance on using this PDF with ms1.
