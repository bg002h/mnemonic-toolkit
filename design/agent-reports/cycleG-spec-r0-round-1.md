# SPEC R0 review — cycleG-zeroization-and-compare-cost-multipath — round 1

**Verdict: NOT GREEN (0 Critical / 1 Important / 6 Minor)**
**Reviewer:** Fable architect, per user directive. Verified @ `41afaec1` (= v0.81.0 + the design commit; `git diff --stat f4461c07..HEAD -- crates/` empty).
**Dispatched:** 2026-07-09 (Cycle G, SPEC R0 round 1). Persisted verbatim per CLAUDE.md.

Design sound; citations all ACCURATE. One Important (a test-plan impossibility) + 6 Minors (surface completeness + comment/fixture precision).

## Citations — all ACCURATE
`RepairOutcome` `repair.rs:437-462` (`corrected_chunks: Vec<String>` @:440, `#[derive(Debug,Clone)]` non-redacting, no Serialize); `RepairDetail` `:424-432` (`original_chunk`/`corrected_chunk:String` @:427-428). `SecretString` (`src/secret_string.rs`): `Zeroizing<String>` inner @:23, `Deref<str>` @:32-37, length-only redacting `Debug` @:61-65, TRANSPARENT `Serialize` via `serialize_str` @:67-71, transparent `Display` @:54-58, ONLY `PartialEq<Self>`/`Eq` @:46-52 (no `PartialEq<str>`). `cost/strip.rs::translate_descriptor` `derive_at_index(0)` @:35-37 (drift from filed :26-27 = Cycle C comment block + `expand_literal_double_star` @:29). Prior-art `derive_address.rs:26-66`. JSON via hand-written borrow structs (`RepairJson` `cmd/repair.rs:288-302`; auto-fire `AutoFireRepairJson` `repair.rs:1884-1900`) — no Serialize derive on RepairOutcome/Detail. Text emitters use `Display` (`{chunk}` @cmd/repair.rs:283, repair.rs:1826); NO `{:?}` in any emit path (grep clean). "~11 asserts" nit: 8 string-element compares need PartialEq (`:1952,1973,2001,2012,2024,2108-2110`); 3 are `.len()` (need nothing).

## Ruling (a) no-wire-leak / transparent-Serialize — HOLDS
Both JSON paths `serde_json::to_string` (cmd/repair.rs:344, repair.rs:1871) through serde; `SecretString::serialize`=`serialize_str(&self.0)` byte-identical to String (pinned by `serializes_byte_identically_to_string`; extends to `&[SecretString]` via serde slice impl). Both text paths `Display` (transparent). NO `{:?}` in emitters → nothing redacted on the wire. `RepairJsonDetail.original_chunk`/`corrected_chunk` can stay `&'a str` (field-init deref-coerces); only the two `corrected_chunks:&'a [String]` widen to `&'a [SecretString]`. Only Debug vector = panic/assert failure output (redacting that is the point). Sound; zero wire change.

## Ruling (b) compare-cost SemVer — MINOR
Previously-erroring `--descriptor` input now succeeding = observable capability addition on the public CLI surface; direct precedent v0.78.0 (descriptor-acceptance broadening = MINOR). PATCH counter-precedent (v0.65.1 L24/25) was panic→clean-error, not accept-widening → doesn't control. MINOR standalone; moot for the release number (item 1 forces MINOR v0.82.0).

## IMPORTANT — I1: the existing rejection test CANNOT be "inverted to assert acceptance" — its fixture is `wpkh(...)`, rejected as an unsupported wrapper regardless of multipath
`compare_cost_double_star_rejects_identically_to_explicit_multipath` (`tests/cli_bip388_double_star_shorthand.rs:377-414`) uses `wpkh([FP_A/84'/0'/0']A/**)` (:385-386). Post-fix this STILL fails: multipath split succeeds → `derive_at_index(0)` succeeds → the wrapper match `strip.rs:59-63` rejects `Descriptor::Wpkh` with `UnsupportedWrapper`. So §2's "INVERT … assert it now succeeds with the same cost" is factually impossible with that fixture + internally inconsistent with §4.4 (which correctly uses `wsh`). Left as-is the implementer hits a wall or silently swaps the fixture (losing the `/**`≡`/<0;1>/*` equivalence coverage on the unsupported-wrapper path).
**Fix (rewrite §2/§4.4 test para):** (a) **UPDATE** (rename, don't invert-to-success) the wpkh test — both spellings now fail IDENTICALLY with the NEW `UnsupportedWrapper` error (assert stderr no longer contains "multipath key cannot be a DerivedDescriptorKey") — preserves the equivalence invariant on this surface + pins multipath now getting PAST derivation. (b) **ADD** acceptance tests on a SUPPORTED wrapper — `wsh(...)` multipath (e.g. `wsh(multi(2,…/<0;1>/*,…))` or `wsh(pk(…/<0;1>/*))`) — assert success + cost byte-identical to the single-path `…/0/*` equivalent, + the `/**` equivalence cell.

## Minors
- **M1** — name the SECOND wire struct + more migration surface: `AutoFireRepairJson.corrected_chunks:&'a [String]` (`repair.rs:1890`) + `AutoFireRepairJsonDetail` (:1897-1898); `verify_mk1_set(corrected_chunks:&[String])` (:978) + its `.as_str()` @:1051 (use `&*` — `as_str` may not resolve through `Deref` at MSRV). All compile-caught; list them so they're not surprises.
- **M2** — `verify_bundle.rs:2026-2032` won't compile: `.unwrap_or_default()` needs `SecretString: Default` (absent). Resolution: drop the redundant `Zeroizing` wrap AND the `Option` fallback (the `outcome.repairs.is_empty()` guard @:2020-2025 already guarantees non-empty) → e.g. `outcome.corrected_chunks.first().is_some_and(|c| &**c == expected_ms1)`. Do NOT add a `Default` impl.
- **M3** — stale comments to update same-PR: Cycle C block `strip.rs:21-28` ("rejects ALL /<0;1>/*") + test-file comment `:379-384`.
- **M4** — §4.5 "malformed multipath still errors" needs a concrete fixture hitting the `into_single_descriptors()` error path — inconsistent branch counts across keys (`/<0;1>/*` on one key, `/<0;1;2>/*` on another in one `wsh(multi(...))`). A single-element `/<0>/*` fails at parse, before the new code.
- **M5** — pin impl structure to prior-art EXACTLY: split FIRST (if `is_multipath`), then feed the single descriptor into the EXISTING `has_wildcard`/`TryFrom` branch (as `derive_address.rs:34-60`) — handles the non-wildcard-multipath edge (`…/<0;1>` no trailing `/*`) for free. §2's `if is_multipath {…} else {today}` sketch is ambiguous on that edge.
- **M6 (optional)** — one-line `secret_string.rs` unit test: `Vec<SecretString>` vs `Vec<String>` serialize byte-identically (the exact `RepairJson.corrected_chunks` shape).

## Confirmed
Cost-index-independence (receive vs change differ only in one child index; same-size keys → identical templates/vbytes; `strip.rs:33-34` already asserts index-independence for wildcards; empty-branch guard mirroring `derive_address.rs:38-42` mandated, no panic). Batch independence (zero file overlap; no shared type; no clap surface → no schema_mirror/manual-lint; both slugs open `FOLLOWUPS.md:36,64`; `PartialEq<str>` can't weaken any production compare — none exists).

**Path to GREEN: fold I1 (rewrite §2/§4.4 test para) + the Minors; re-dispatch round 2.**
