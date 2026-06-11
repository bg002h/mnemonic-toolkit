# SPEC — CI/test hygiene Cycle B: de-vacuify the PolicyNode-kind coverage (macro single-source)

**Cycle:** toolkit NO-BUMP (test only) · **Source SHA:** `aa3e46d` · **Recon:** `cycle-prep-recon-ci-test-hygiene-cluster.md`.
**Resolves:** `policynode-grammar-coverage-vacuous-on-joint-omission`.
**RE-SCOPED (R0-r1 split):** Part B (`hand-frozen-lint-canons-no-completeness`) is SPLIT OUT — R0-r1 I3 showed its 19-file "transient" allowlist is unaudited and that `verify_bundle.rs`/`ms_shares.rs` likely need NEW canonical ZEROIZE_ROWS (a real coverage gap, not a blanket allowlist). That audit is its own cycle; see §Deferred.

No binary/wire/CLI change → **no `schema_mirror` / manual / GUI / sibling / version bump / CHANGELOG** (NO-BUMP test-only convention).

---

## PART A — single-source `kind()` + `NODE_KINDS` via a variant macro (de-vacuify the coverage test)

### Problem (verified @ `aa3e46d`)
`node_kinds_cover_enum` (`ir.rs:345-355`) asserts `all_variant_samples()` tags == `NODE_KINDS` — both HAND-LISTS. `kind()` (`ir.rs:209-229`) is a parallel hand-maintained variant→str match; `NODE_KINDS` (`ir.rs:36-54`) a parallel hand-list. A new `PolicyNode` variant added to the enum + `kind()` but omitted from BOTH `NODE_KINDS` and `all_variant_samples` passes vacuously (the test compares two lists that both omit it). The fn/test docs already name this ("needs a variant-enumerator macro").

### Design — `declare_policy_node_kinds!` (precedent: `declare_node_type_variants!` at `cmd/convert.rs:1767`)
Generate `NODE_KINDS` + `kind()` from ONE `(Variant => "kind")` list:
```rust
macro_rules! declare_policy_node_kinds {
    ( $( $variant:ident => $kind:literal ),* $(,)? ) => {
        /// External tag per PolicyNode variant. Macro-generated from the single
        /// variant list below — kept complete-by-construction (a new variant
        /// makes `kind()`'s match non-exhaustive → compile error → the author
        /// MUST extend the macro input, which AUTOMATICALLY grows NODE_KINDS).
        pub const NODE_KINDS: &[&str] = &[ $( $kind ),* ];
        impl PolicyNode {
            pub fn kind(&self) -> &'static str {
                match self { $( PolicyNode::$variant(..) => $kind ),* }
            }
        }
    };
}
declare_policy_node_kinds!(
    Pk => "pk", Pkh => "pkh", Multi => "multi", Sortedmulti => "sortedmulti",
    Older => "older", After => "after", Sha256 => "sha256", Hash256 => "hash256",
    Hash160 => "hash160", Ripemd160 => "ripemd160", AndV => "and_v", OrD => "or_d",
    OrI => "or_i", OrB => "or_b", Andor => "andor", Thresh => "thresh", Wrap => "wrap",
);
```
The generated `kind()` match is EXHAUSTIVE → a new variant not in the macro = compile error → forces a macro-input addition → `NODE_KINDS` grows automatically → `node_kinds_cover_enum` (samples == NODE_KINDS) now FAILS until the sample is added → **non-vacuous.** The grammar cross-check (`schema.rs grammar_matches_node_kinds_hand_list`) is also de-vacuified (NODE_KINDS now complete).

**Implementation notes (R0-r1 folds):**
- **DELETE the manual `kind()` body at `ir.rs:209-229` (I1)** — the macro generates `impl PolicyNode { pub fn kind() }`; leaving both is `error[E0201]: duplicate definitions with name 'kind'`. The macro's `impl PolicyNode { fn kind }` is a SEPARATE impl block; `render()` + the other methods stay in the existing manual `impl PolicyNode`. Move the manual `kind()` doc-comment onto the macro/its invocation.
- **DELETE the manual `pub const NODE_KINDS` at `ir.rs:36-54`** — the macro generates it (move its doc onto the macro invocation).
- **KEEP the REMINDER match inside `all_variant_samples` (`ir.rs:309-333`) (I2)** — it is NOT redundant. The macro forces a COMPILE error at `kind()` (→ extend the macro list → NODE_KINDS grows); the REMINDER match independently forces a COMPILE-time visit to `all_variant_samples` to add the SAMPLE (the macro change alone defers the sample obligation to the RUNTIME `node_kinds_cover_enum` failure). Belt-and-suspenders: keep both the REMINDER match AND `node_kinds_cover_enum` (the test is NOT redundant — it's what de-vacuifies).
- All 17 PolicyNode variants are single-data-field tuple variants (`Pk(_)`…`Wrap(_)`), so `PolicyNode::$variant(..)` matches each. A future UNIT variant would fail the `(..)` pattern — an acceptable extra forcing function; note it in the macro doc.
- **Precedent note (m1):** `declare_node_type_variants!` (`convert.rs:1767`) generates a const VALUE array (unit variants only); this macro generates a `match self` METHOD because PolicyNode carries data — same forcing pattern, different output shape.

### Part A tests
- `node_kinds_cover_enum` (unchanged assertion) is now NON-VACUOUS. **Verify** by a scratch-mutation: temporarily add a fake variant to the enum + macro + `kind()` but NOT to `all_variant_samples` → confirm `node_kinds_cover_enum` REDs (it didn't before for a both-omitted variant). Restore. (If adding a throwaway enum variant is too invasive, instead temporarily DROP one entry from the macro list and confirm `kind()` fails to compile — proving the exhaustiveness forcing — then restore.)
- All existing ir.rs/schema.rs grammar tests stay green (the macro output is byte-equivalent to the current `kind()` + `NODE_KINDS`).

---

## Deferred — Part B (`hand-frozen-lint-canons-no-completeness`) → its own cycle

R0-r1 I3 found the proposed zeroize-lint source→declared file-scan needs a 19-file "transient" allowlist, and that the classification is NOT done: a spot-check shows `cmd/verify_bundle.rs` (`Zeroizing::new` on passphrase + entropy) and `cmd/ms_shares.rs` (14 owned-secret sites) carry real owned secrets with NO canonical `ZEROIZE_ROW` — blanket-allowlisting them would write a FALSE "transient" record (worse than the status quo). The real Part B work is: audit all 19 unlisted secret-bearing files, PROMOTE the genuine owned-secret sites (≥ verify_bundle, ms_shares) to canonical ZEROIZE_ROWS (an actual coverage improvement), allowlist only the genuinely crypto-internal/pass-through ones, THEN add the scan. That audit deserves its own cycle-prep + SPEC + R0 — file FOLLOWUP note that `hand-frozen-lint-canons-no-completeness` remains open pending it.

---

## Ritual
NO version bump / CHANGELOG (NO-BUMP test-only). FOLLOWUPS resolve `policynode-grammar-coverage-vacuous-on-joint-omission`; leave `hand-frozen-lint-canons-no-completeness` OPEN with a note that it's split to its own audit cycle. Stage paths explicitly. Mandatory R0 gate to 0C/0I; persist reviews to `design/agent-reports/`.

## Non-goals
Auto-deriving the PolicyNode SAMPLES (data-carrying — only `kind()`/`NODE_KINDS` are macro-single-sourced); the zeroize-lint completeness scan (Part B, split out); any binary/wire change.
