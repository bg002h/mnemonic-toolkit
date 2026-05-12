# Release History

_This appendix tracks the technical manual's own release cuts. The prior release history of the four sibling repos is not in scope for this manual's coverage; each row below corresponds to a `tech-manual-vX.Y.Z` tag on the toolkit repo. New rows are added during the next cut's back-matter accretion phase._

| Date | Manual cut / version | One-line summary |
|---|---|---|
| 2026-05-11 | `tech-manual-v0.1.0` | First releasable cut: Parts I (Foundations) + II (Wire formats: md1 / mk1 / ms1) + back-matter skeleton; 97pp PDF. |
| 2026-05-11 | `tech-manual-v0.2.0` | Part III (Address derivation: descriptor → miniscript → address; shape coverage; network & SLIP-0132) added; back-matter accreted; 119pp PDF. |
| 2026-05-11 | `tech-manual-v0.3.0` | Part IV (Bundle formation: bundle anatomy; anti-collision invariants; future shares) added; back-matter accreted (glossary 73 entries, index 200 rows, BIP cross-reference §IV.* updated); 145pp PDF. |
| 2026-05-11 | `tech-manual-v0.4.0` | Part V (Rust API reference: md-codec / mk-codec / ms-codec / mnemonic-toolkit) added; 92/92 public-symbol coverage gate; standalone-consumer worked-example crate (4 transcripts); back-matter accreted (glossary 96 entries, index 530 rows, BIP cross-reference 20 BIPs); 242pp PDF. |
| 2026-05-12 | `tech-manual-v1.0.0` | **v1.0 release.** Back-matter polish + architect sign-off. Bibliography completed across Parts III-V (+13 entries, 20 BIPs cited). Troubleshooting expanded to full 101-Error-variant coverage across all four crates (md 43 / mk 22 / ms 10 / toolkit 26). Glossary 107, index 540. v0.8.x drift folded into §V.4.3.8 (vendor-emitter sub-modules + `ELECTRUM_SEED_VERSION_PIN`). All SPEC §7 A1–A11 acceptance criteria green. |
| 2026-05-12 | `tech-manual-v1.1.0` | `export-wallet` drift fold. Added §V.4.5.9 (eight vendor-format output-shape sub-sub-sections: `bitcoin-core` / `bip388` / `coldcard` / `jade` / `sparrow` / `specter` / `electrum` / `green`) and §V.4.5.10 (8×8 format×shape compatibility matrix with 7 footnotes enumerating per-emitter refusal sources). Glossary 112 (+5 `pub(crate)` symbol entries), index 545. Troubleshooting's 4 `ExportWallet*` rows pointer-refined. Three reviewer rounds (r1 0C/2I/0L/1N → r2 0C/4I → r3 0C/0I). Zero new FOLLOWUPs. |
