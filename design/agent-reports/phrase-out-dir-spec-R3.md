# R3 mechanical check — `design/SPEC_convert_phrase_out_dir.md`, `87a8d07..8164979`

Scope: does the fold fix R2's I-1 and m-1..m-4, and does the diff contradict the rest of the spec. Design not re-reviewed. Nothing run.

**Counts: 0 Critical, 0 Important, 1 Minor (pre-existing, not introduced by the diff).**

## R2 findings

| R2 | fixed? | where |
|---|---|---|
| I-1 step-1 write/fsync failure leaves `T` | yes | Step 1 now unlinks `T` on any error after creation and states the invariant "unlinked before `convert` returns on every path; only a kill can leave it". Step 3's false sentence is removed. §8 gains the "step-1 write or fsync failing (seam, e.g. ENOSPC) -> exit 1, no temp left" row. |
| m-1 fallback errno list | yes | Step 4 falls back on any `hard_link` error other than EEXIST, with the reason (exclusive-create is safe whatever the cause). The "any other errno is Io" line is gone. |
| m-2 `O_NOFOLLOW` on Windows | yes | Windows opens with `FILE_FLAG_OPEN_REPARSE_POINT` via `custom_flags`, then the type check refuses it. |
| m-3 `GetLocalTime` cannot fail | yes | "`GetLocalTime` cannot fail. Only if Unix `localtime_r` fails: ..." |
| m-4 windows-sys features and lock | yes | Features `Win32_System_SystemInformation` and `Win32_Foundation` named; "`Cargo.lock` changes; re-vendor and the vendor-freshness gate run". |

## Contradictions introduced by the diff

None. Step numbering (0-5) is intact. Step 4's "after step 3" still holds, since step 3 unlinks `T` on the hard-link error before the fallback. The invariant agrees with the kill note (steps 1 to 3) and with the §8 rows. The Unix/Windows split is consistent with the Windows paragraph at the end of §4. The §8 rows match the new behaviour.

## Minor (pre-existing, found while reading; not from this diff)

- **m-1 `schema_version` contradiction.** §2.3 says `schema_version` is `"2"` only under `--phrase-out-dir` (byte-identical `"1"` without it). But §8's JSON row says "`--json` carries `path` and `already_saved`, no `value`, `schema_version` `"1"`", and §7 says "the JSON v1 extension". Under the flag it is `"2"`. Fix: §8 should say `"2"` with the flag and add a no-flag row asserting `"1"` byte-identical, and §7 should say "the JSON v2 envelope".
- Cosmetic: the status line lacks a comma ("(1C/8I/12m) R1"), and one §3 line is over-long after the edit.
