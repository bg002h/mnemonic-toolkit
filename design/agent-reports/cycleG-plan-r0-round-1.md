# PLAN R0 review — cycleG-zeroization-and-compare-cost-multipath — round 1

**Verdict: GREEN (0 Critical / 0 Important)** — 4 Minor (non-blocking, folded).
**Reviewer:** Fable, per user directive. Plan @ `46b2ec4d` vs live source.
**Dispatched:** 2026-07-09 (Cycle G, plan-R0 round 1). Persisted verbatim per CLAUDE.md.

Every load-bearing claim independently verified live. Migration surface complete, no-wire-change guard real + pinned by existing value-level tests, every cited line accurate.

## Verification
- **Phase decomposition CORRECT / zero file overlap** (grep): P0 = `repair.rs`/`cmd/repair.rs`/`cmd/verify_bundle.rs`/`secret_string.rs`; P1 = `cost/strip.rs` + `tests/cli_bip388_double_star_shorthand.rs`. Order free; P0-first fine.
- **Migration surface — NOTHING OMITTED.** Repo-wide grep `corrected_chunks|original_chunk|corrected_chunk` → consumers ONLY in the 4 P0 files. All §1 sites live-verified: `RepairDetail` @:427-428, `RepairOutcome.corrected_chunks` @:440, `verify_mk1_set` @:978 + `.as_str()` @:1051, producer locals @:1098/1126/1660, `AutoFireRepairJson.corrected_chunks` @:1890 + `AutoFireRepairJsonDetail` @:1895-1898, `RepairJson.corrected_chunks` @cmd/repair.rs:300 (Detail fields @:307-308 correctly `&'a str`, `&r.original_chunk` @:330-331 deref-coerces), verify-bundle compare @:2026-2033. Un-named `repair.rs` sites (e.g. `RepairDetail` construction @:800-804) compile-enforced by the type change.
- **Type prerequisites hold:** `SecretString` derives `Clone` @:22 (producer `.clone()` pushes @:1103/1145/1683 + `RepairOutcome` derive(Clone) work), has `PartialEq`/`Eq` @:46-52 (RepairDetail derive(PartialEq,Eq) @:424 compiles), `Display` @:54 + `Deref<str>` @:32; no `Default`, no `PartialEq<str>` today — the plan's additions are exactly the gaps.
- **P0 no-wire-change guard SUFFICIENT:** no `{:?}` of the outcome types anywhere in `src/` (grep); text emitters `{chunk}` Display (cmd/repair.rs:282-284, repair.rs:~1825), JSON via serde (cmd/repair.rs:319-347, repair.rs:1846+). Wire PINNED: `cli_repair.rs:99-103` asserts chunk VALUES + `cli_auto_repair.rs:307/:424-425` values AND raw key ORDER → a silent redaction fails immediately. `&*`-not-`.as_str()` carried; `no Default` = G0-3; 8-assert count exact (repair.rs:1952/1973/2001/2012/2024/2108/2109/2110).
- **P1 compare-cost IMPLEMENTABLE:** prior-art `derive_address.rs:34-46` exactly is_multipath→into_single_descriptors→is_empty guard→remove(0), then has_wildcard/TryFrom @:48-60; split lands BEFORE `strip.rs:35-42` → wpkh multipath reaches the wrapper match → `UnsupportedWrapper` @:59-63 (UPDATE-not-invert correct). wsh-ACCEPT + equivalence + malformed homed; stale comment strip.rs:21-28 update included.
- **TDD mapped** (§4: 1/2/3/7→P0, 4/5/6→P1, 8→per-phase FULL suite). **Release ritual COMPLETE** (all sites live: Cargo.toml @0.81.0, both READMEs, install.sh:32, gen.sh FATAL @:44; both FOLLOWUP slugs open @:36/:64; the 4 CI workflows exist; NO re-vendor/sibling-pin correct; MINOR v0.82.0). **Guard-rails G-A..G-D sufficient** (missed-site doubly covered: grep + compile-enforcement).

## Minors (non-blocking — FOLDED)
- **M1** — "goldens byte-identical" (plan :24-25/:73) is loose: the pins are parsed-JSON value + raw key-ORDER asserts (`cli_repair.rs:99-103`, `cli_auto_repair.rs:307/:424-425`), NOT raw golden files. They DO catch wire redaction → no gap; cite the assertion sites so the implementer doesn't hunt for golden files.
- **M2** — stale doc-comment `verify_bundle.rs:2014-2015` ("held in `Zeroizing` … §8 risk 6 / G5") becomes false when P0 drops the `Zeroizing` wrap → add to the P0 comment-update list.
- **M3** — `gen.sh` has ~6 `0.81.0` occurrences (`:3/:44/:109/:126/:711/:724`), not one pin → bump GLOBALLY (the "non-version diff empty" check covers it operationally; state global).
- **M4** — plan Source SHA `267f938c` → now `46b2ec4d` (cosmetic; all citations accurate at 46b2ec4d).

**Gate: plan-R0 GREEN — implementation may begin.**
