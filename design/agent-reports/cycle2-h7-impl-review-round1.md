# cycle-2 WS-C / H7 — per-phase implementation review (round 1)

- **Scope:** ACCEPT BIP-380 prefix-form `[fp/path]@N` key-origin via an all-named-group
  conversion of `lex_placeholders`, without regressing cycle-1's H13 hardened-multipath reject.
- **Worktree:** `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/cycle2-h7`
- **Branch / commit:** `fix/cycle2-h7` @ `36095b88`
- **Baseline:** toolkit `origin/master` = `f9467cc5` (cycle-1 post-merge: H12/H1/H13)
- **Plan:** `design/IMPLEMENTATION_PLAN_cycle2_funds_loss_fixes.md` §3 (WS-C/H7) — the PINNED named-group design (§3.4)
- **Method:** EMPIRICAL — built the binary in the worktree and ran the actual `mnemonic`
  bundle/verify-bundle path; did not trust the tests.

---

## VERDICT: **GREEN — 0 Critical / 0 Important**

The named-group conversion is complete (zero numeric accesses remain on `lex_placeholders`
captures), H13 holds empirically for BOTH the bare AND the prefix-annotated forms across the
full cycle-1 reject set, the prefix ACCEPT path carries the origin and fires the per-`@N`
fp cross-checks (xpub-slot AND phrase-arm), and the path-only judgment call rejects loudly
end-to-end in both positions (no silent drop). Scope is clean (only the 4 declared files; no
new error variant; no `verify_bundle.rs` source edit; no version churn). Full suite green;
clippy clean. No fmt churn introduced (pre-existing master deviations only).

---

## Critical

None.

## Important

None.

## Minor

- **M1 (informational, not actionable):** the path-only (no-fp) prefix and suffix forms reject
  with *different* downstream error text — prefix `[/84'/0'/0']@0` → `master fingerprint should
  be 8 characters long`; suffix `@0[/84'/0'/0']` → `base58 encoding error`. Both are loud exit-2
  `ToolkitError::DescriptorParse` rejects from `Descriptor::from_str` (the leaked bracket lands in
  a different position), so this is funds-safe and acceptable. Only the *outcome class* (loud
  reject, no dropped origin) is load-bearing; the message divergence is cosmetic.
- **M2 (pre-existing, out of scope):** standalone `rustfmt --check` reports 4 deviations in
  `parse_descriptor.rs` (lines 1748/1780/1838 — pre-existing cycle-1 *tests*, NOT in any H7 hunk)
  and 1 in `bundle.rs`. PROVEN pre-existing: the same deviations exist on `origin/master`
  (`git show origin/master:… | rustfmt --check` → 4 + 1). H7 introduces ZERO new fmt deviations.
  Consistent with the project's standing manual-format territory (mlock g6 / fmt exemptions).
  No action required for this PR.

---

## Named-group completeness list (MUST-DO #1)

`grep -nE 'caps\[|caps\.get\(|caps\.name\(' parse_descriptor.rs` — every consumer of the
**`lex_placeholders`** captures is BY NAME. ZERO numeric index accesses remain on those captures:

| consumer (line) | access | named group |
|---|---|---|
| `:104` `idx_match` → `i` (index) | `&caps["idx"]` | `idx` |
| `:111` `pfx_fp` | `caps.name("pfx_fp")` | `pfx_fp` |
| `:112` `sfx_fp` | `caps.name("sfx_fp")` | `sfx_fp` |
| `:120` `path_match` | `caps.name("pfx_path").or_else(|| caps.name("sfx_path"))` | `pfx_path` / `sfx_path` |
| `:147` `multipath_alts` (**H13 validator `:146-178`**) | `caps.name("mpath")` | `mpath` |
| `:180` `wildcard_hardened` | `caps.name("wild")` | `wild` |

The H13 strict validator body (`:146-178`) is byte-identical to cycle-1; only its accessor
changed `.get(4)` → `.name("mpath")`. The validator therefore reads the multipath body
**regardless of where the prefix bracket sits** — the named conversion cannot shift it.

The TWO remaining numeric `caps[1]` (`:416`, `:427`) are in the **separate `substitute_synthetic`
strip regex** (`:404`), whose prefix bracket is intentionally NON-CAPTURING `(?:\[…\])?`, so the
sole capturing group `(\d+)` stays at index 1 (the `@N` index). Verified correct: byte-mirrors
the suffix bracket, multipath class stays the C1-narrow `[0-9;]+`. NOT a leftover — by design.

---

## H13 non-regression — EMPIRICAL result (MUST-DO #2)

Built `target/debug/mnemonic` in the worktree (cargo build OK). Fed each input through the
real `mnemonic bundle --descriptor … --slot @0.xpub=<valid xpub> --network mainnet`:

| input | exit | stderr (message) |
|---|---|---|
| **baseline** `wpkh(@0[deadbeef/84'/0'/0']/<0;1>/*)` | **0** | (bundle emitted — valid form still accepts) |
| **H13-A bare** `wpkh(@0/<0';1'>/*)` | **2** | `@0 multipath alternative `0'` is hardened; … un-restorable` |
| **H13-B prefix** `wpkh([deadbeef/84'/0'/0']@0/<0';1'>/*)` | **2** | `@0 multipath alternative `0'` is hardened; … un-restorable` |
| bare `<0h;1h>` | **2** | `@0 multipath alternative `0h` is hardened; …` |
| prefix `<0h;1h>` | **2** | `@0 multipath alternative `0h` is hardened; …` |
| bare malformed `<0'';1>` | **2** | `@0 multipath alternative `0''` is hardened; …` |
| prefix malformed `<0'';1>` | **2** | `@0 multipath alternative `0''` is hardened; …` |
| bare mixed `<0;1'>` | **2** | `@0 multipath alternative `1'` is hardened; …` |

**Result: H13 (the C1 guard) HOLDS for BOTH the bare and the prefix-annotated forms across the
entire cycle-1 reject set.** The cycle-1 typed error fires every time — no silent collapse to a
bare `/*`. The prefix alternation did NOT detach the H13 validator from the multipath body.
(Unit tests `lex_prefix_hardened_multipath_still_rejects_h13` + the pre-existing
`lex_rejects_hardened_multipath_h_form` / `lex_rejects_malformed_double_marker_multipath` all
pass — but the table above is the binary's own runtime behavior, not the test.)

---

## Prefix ACCEPT correctness — EMPIRICAL (MUST-DO #3)

- **(a) origin carried:** `wpkh([deadbeef/84'/0'/0']@0/<0;1>/*)` → exit 0; stderr shows
  `fingerprint: deadbeef` + `origin path: m/84'/0'/0'` (origin NO LONGER dropped).
- **(c) prefix ≡ suffix byte-identical:** prefix vs `wpkh(@0[deadbeef/84'/0'/0']/<0;1>/*)`
  stdout is `diff`-IDENTICAL (md1 + mk1 byte-for-byte equal).
- **(b) xpub-slot fp cross-check (`bundle.rs:1665`):**
  - WRONG `--slot @0.fingerprint=cafef00d` vs prefix `deadbeef` → exit 2:
    `--slot @0.fingerprint specifies cafef00d but descriptor @0 annotation specifies deadbeef`.
  - MATCHING `--slot @0.fingerprint=deadbeef` → exit 0.
  - suffix-form oracle: same wrong fp → same refusal. **The `:1665` check compares prefix-fp vs
    `--slot @N.fingerprint=` and refuses on mismatch — confirmed at the xpub-slot site.**
- **RED→GREEN proven:** `origin/master` (`bundle.rs:1650-1654`) had `.or(anno_fp)` with NO equality
  check — the slot value silently won, the prefix-anno fp was silently dropped. The pre-H7
  binary (suffix-only regex, faithfully reproduced) emits `fingerprint: 00000000` for the prefix
  form (origin dropped, exit 0). The fix is a genuine funds-safety RED→GREEN discriminator.
- **BONUS — phrase-arm cross-check (`bundle.rs:1617`) ALSO fires:** the corrected pre-existing
  test exposed it. `[deadbeef/…]@0` + TREZOR_24 phrase now → exit 2:
  `--slot @0.phrase derives master fingerprint 5436d724 but descriptor @0 annotation specifies
  deadbeef`. So BOTH origin-cross-check arms (xpub slot AND phrase) honor the prefix annotation.

---

## Path-only judgment call — VERDICT: **CORRECT / funds-safe** (MUST-DO #4)

The implementer asserts `[/84'/0'/0']@0` (path, no fp) is rejected at `Descriptor::from_str`
(end-to-end), NOT at the lexer (the 8-hex fp is mandatory in BOTH grammars, so the bracket
leaks downstream). Verified empirically via the real CLI:

| input | exit | stderr |
|---|---|---|
| prefix `wpkh([/84'/0'/0']@0/<0;1>/*)` | **2** | `descriptor parse failed: master fingerprint should be 8 characters long` |
| suffix `wpkh(@0[/84'/0'/0']/<0;1>/*)` | **2** | `descriptor parse failed: base58 encoding error` |

**BOTH positions reject loudly (exit 2, `DescriptorParse`); NEITHER is silently accepted with a
dropped origin.** Any loud reject is funds-safe, and critically there is NO residual H7 bug
(no exit-0-with-origin-dropped). The layer (downstream `Descriptor::from_str`, not the lexer)
and the divergent message text are acceptable (see M1). **No residual H7 bug.**

---

## Over-reject + corrected-test integrity (MUST-DO #5)

- **No legit form wrongly fails:** bare `wpkh(@0/<0;1>/*)` → 0; plain `wpkh(@0/0/*)` → 0;
  suffix `wpkh(@0[deadbeef/84'/0'/0']/0/*)` → 0. All 17 `parse_descriptor::tests::lex_*` pass —
  including the pre-existing cycle-1 suffix/bare/full-annotation tests — proving the named
  conversion preserved suffix/bare parsing.
- **Both-positions (d):** `wpkh([deadbeef/…]@0[cafef00d/…]/<0;1>/*)` → exit 2 with the NEW typed
  `…carries BOTH a prefix `[fp/path]@0` and a suffix `@0[fp/path]`…ambiguous…supply exactly one`.
- **Corrected pre-existing test `cli_mode_violations_v0_5::descriptor_without_template_accepted`
  — NOT a made-to-pass hack.** The rewrite swaps the annotated `wpkh([deadbeef/…]@0/<0;1>/*)`
  for the bare `wpkh(@0/<0;1>/*)`. PROVEN necessary: the ORIGINAL annotated descriptor + the
  test's TREZOR_24 phrase now (correctly) REFUSES — `master fingerprint 5436d724 but descriptor
  @0 annotation specifies deadbeef`. The original test was inadvertently *exercising the very
  silent-drop bug H7 fixes* (`deadbeef` was a fake fp that pre-H7 was ignored). The bare rewrite
  preserves the test's stated intent (MODE acceptance — descriptor without template) and was
  verified to accept (exit 0). The rewrite is a correct consequence of the fix, not a cover-up.

---

## Scope & hygiene

- **Files changed (exactly the declared 4):** `parse_descriptor.rs`, `cmd/bundle.rs`,
  `tests/cli_bundle_h7_prefix_origin.rs` (new, 6 integration tests), `tests/cli_mode_violations_v0_5.rs`.
- **No new `ToolkitError` variant** (`error.rs` untouched — reuses `DescriptorParse`). ✓
- **No `verify_bundle.rs` source edit** (verify-bundle coverage is test-only via the shared
  `lex_placeholders`, as the plan specified). ✓
- **No version / Cargo.toml churn.** ✓
- **clippy** `-p mnemonic-toolkit --bins --tests` → exit 0, no warnings. ✓
- **Full suite** `cargo test -p mnemonic-toolkit` → 0 failed across all targets (1031 unit incl.
  the 6 new lex tests; 6 H7 integration; 6 mode-violation incl. the corrected test). ✓
- **Tests are real RED→GREEN discriminators:** the H7 integration file uses real
  `cargo_bin("mnemonic")` end-to-end runs, computes the master fp from the binary itself (not
  hardcoded), and pins every arm against a suffix-form oracle; comments document pre-H7 behavior
  ("Before H7 the prefix form exited 0"). Independently confirmed by reproducing the pre-H7 bug
  on the suffix-only baseline binary.
- **fmt:** no churn introduced (M2 — pre-existing master deviations only).

---

## Bottom line

**GREEN, 0 Critical / 0 Important.** The highest-risk change in cycle-2 is empirically sound:
H13 is non-regressed for bare AND prefix forms, the prefix origin is accepted + cross-checked on
both arms, path-only rejects loudly with no dropped origin, and the corrected test is a faithful
consequence of the fix. Ship.
