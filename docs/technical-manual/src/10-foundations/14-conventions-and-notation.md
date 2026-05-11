# Conventions and Notation

This chapter pins the conventions used throughout the rest of the manual. A reader who already knows the constellation can skim it; a reader doing implementer work should read it linearly.

## Bit and byte ordering

All wire-format bit fields in this manual use **big-endian within each multi-bit field**: the most significant bit of a value is the first bit on the wire (lowest bit position within the field's range). For example, an 8-bit field with value `0b10010110 = 0x96` is laid down with bit 7 first, bit 0 last.

Within a multi-byte stream — never relevant for ms1, but relevant for the md1 / mk1 bit-aligned bytecode — bits are accumulated **MSB-first**. A 5-bit symbol from the codex32 alphabet contributes its 5 bits to the bit-aligned payload starting from the most-significant unprotected bit.

GF(32) symbols (= codex32 alphabet characters) are *not* further-decomposed into bits at the BCH layer — the BCH polynomial works over GF(32). The bit-level interpretation applies only at the *payload* layer, after BCH symbols have been decoded back to their integer values.

## The `bN:meaning` field-annotation notation

Every wire-format diagram in this manual annotates each field with the form:

```text
N:meaning
```

where `N` is the field's bit width and `meaning` is a short label. Example from md1 §II.1.1:

```text
4:version | 1:chunked | 1:divergent_paths | 1:reserved | 1:path-decl-present | ...
```

reads: "4 bits of version, then 1 bit of `chunked` flag, then 1 bit of `divergent_paths` flag, …". The bits are laid out left-to-right in the order shown, with the version field's MSB at the start.

Where a field width depends on a runtime value (e.g., `kiw = ⌈log₂(n)⌉` for placeholder indices in md1's multi-family bodies), the notation reads `kiw:key_index` with the *symbolic* width and a footnote or accompanying paragraph defining `kiw` in terms of the policy's `n`.

Concrete bit-by-bit traces in worked examples use `[0]` and `[1]` for individual bit values, grouped by field. Example: `[0100 1 0 0 1]` reads "version = 4 (`0100`), chunked = 1, divergent_paths = 0, reserved = 0, path-decl-present = 1".

## Index marker convention

Every term introduced in this manual that warrants a back-matter index entry is annotated with `\index{}` immediately after its first definitional use, on the same line. The `\index{}` annotation is stripped from the markdown render path (via the `strip-latex-from-md.lua` pandoc filter) and passed through to the PDF render path, where `makeindex` builds a page-numbered alphabetical index (§62).

Every `\index{}` in a chapter must have a matching row in `src/60-back-matter/62-index-table.md`. The `tests/lint.sh` bidirectional check enforces this: a missing row OR a missing source-side marker fails the lint with a direction-specific diagnostic.

When you add a new index marker:

1. Pick a `TERM` consistent with the rest of the manual's index. Lowercase common nouns (`miniscript`, `wire format`); preserve case for proper-noun identifiers (`BIP-32`, `OperatorContextViolation`).
2. Place the `\index{}` after the term's first definitional use, on the same line. Not inside a fenced code block.
3. Add a row to `62-index-table.md`. Each row is a pipe-delimited Markdown table line: the term backtick-wrapped, then a link to the section of first definitional use, e.g. `` `policy_id_stub` `` in column 1 and `[Bundle anatomy](#bundle-anatomy)` in column 2.

Pandoc's slug rules apply: section anchors are lowercase, with non-alphanumerics dropped and runs of whitespace collapsed to a single hyphen. So `# Conventions and Notation` slugs to `#conventions-and-notation`.

## Cross-reference convention

References to other parts of this manual use the form **§II.1.3** (= Part II, chapter 1, section 3) or **§II.1** (= Part II, chapter 1) or **§63** (= back-matter section 63). The parts are:

- §I — Foundations (this Part)
- §II — Wire formats
- §III — Address derivation
- §IV — Bundle formation
- §V — Rust API reference
- §60+ — Back matter

References to external documents use these conventions:

- **BIP citations:** `BIP-93` (= BIP number); `BIP-93 §2.3` (= BIP number, section).
- **Per-version SPECs:** `md-codec SPEC v0.30 §1.4` (= repo's `design/SPEC_v0_30_wire_format.md` section 1.4).
- **Rust crate items:** `md_codec::Error::WireVersionMismatch` (= fully-qualified Rust path); `md-codec` (= crate name only).
- **Git artifacts:** `commit 0c43ca2` (short SHA); `tag md-codec-v0.32.0`.
- **File paths in any repo:** unqualified path is relative to the repo root, e.g., `crates/md-codec/src/tree.rs:122` (with line number).
- **Cross-repo file paths:** prefixed with the repo slug, e.g., `bg002h/descriptor-mnemonic/bip/bip-mnemonic-descriptor.mediawiki`.

## Notational shorthands

A handful of project-specific shorthands appear throughout:

- **`@N`** — a BIP-388 placeholder for cosigner `N` (0-indexed). `@0` is the first cosigner; `@N` (where `N = n`) appears in md1 §II.1 as the NUMS-sentinel discussion (now retired in v0.30).
- **`<0;1>/*`** — BIP-389 multipath; chain 0 for receive, chain 1 for change; `*` is the wildcard non-hardened address index.
- **`kiw`** — *key index width*, the bit width of an `@N` placeholder index field in md1 wire format. `kiw = ⌈log₂(n)⌉` where `n` is the policy's placeholder count.
- **`HRP`** — human-readable prefix (`md`, `mk`, or `ms`), inseparable from the BCH polynomial computation per §I.3.
- **`policy_id_stub`** — 4-byte stub of the canonical wallet-policy preimage hash; carried on each mk1 card, recomputable from md1. Defined fully in §IV.2.

## Code-block conventions

- Rust code samples are valid Rust 2024-edition fragments. Where possible, they compile against the corresponding crate at the version named in §11's version-coverage table.
- Shell-command examples are bash; a leading `$` indicates a prompt; lines without the `$` prefix are command output. `# comment` is a shell comment.
- mediawiki / markdown citations to the md1 BIP draft use the form `bip/bip-mnemonic-descriptor.mediawiki §"Tag table"`.
- Hex byte sequences appear as `0x4a 0x12 0x...`. Hex bit sequences (rare; used in worked encode traces) appear as a continuous string with implicit MSB-first grouping; e.g., `0100 1 0 0 1` is one 4-bit field followed by four 1-bit fields.

## Reading the wire-format diagrams

A typical wire-format diagram looks like:

```text
  Header (5 bits):
    | 4:version | 1:chunked |

  Path-decl (variable):
    | 4:depth | 1:divergent | depth × 32:component | ...

  Tree (variable):
    | 6:tag | body... |

  Section sums: 5 (header) + ... + ... = N bits total
```

The MSB of each field is at the top-left. Fields read top-to-bottom within a row, row-after-row down the diagram. Variable-width fields are annotated as `variable` and explained in adjacent prose.

Section sums at the bottom of each wire-format chapter help cross-implementations verify they're producing the same bit-count for a given input. The md1 §II.1 chapter, in particular, sums every example to the total bit count and shows where the post-payload chunk-header slack lives.

The next Part begins the wire-format chapters proper.
