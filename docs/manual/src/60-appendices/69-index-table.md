# Appendix I — Index of terms

The PDF render emits a true page-numbered alphabetical index (built
by `makeindex` from `\index{}` markers throughout the source). The
markdown render emits this curated `Term → §section` table instead,
since markdown viewers have no notion of page numbers.

The two indexes are kept in lockstep by the bidirectional consistency
check in `tests/lint.sh`: every `\index{TERM}` marker in the source
must have a matching row here, and vice versa. Adding a marker
without adding the row (or vice versa) fails the lint.

| Term | Section |
|---|---|
| `m-format star` | [About this manual](#about-this-manual) |
