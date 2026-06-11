# R0 Review — CI/test hygiene Cycle B (PolicyNode-kind macro; Part A only) — ROUND 2 (GREEN)

**Source SHA:** `aa3e46d`. Re-review after re-scoping to Part A only (Part B split out) + folding I1/I2/m1.

**Verdict: 🟢 GREEN — 0 Critical / 0 Important.** Implementation ready.

## Confirmations
- **(a) Internal consistency / E0201:** the macro emits `impl PolicyNode { pub fn kind() }` as a SEPARATE impl block; deleting the manual `kind()` (ir.rs:209-229) is mandatory + explicitly stated (E0201 fires on duplicate method names across impl blocks). `render()` + others stay in the existing impl. `schema.rs` imports `NODE_KINDS` from `super::ir` — resolves identically to the macro-generated `pub const NODE_KINDS`; `grammar_matches_node_kinds_hand_list` is also de-vacuified (NODE_KINDS now complete). All 17 PolicyNode variants are single-field tuple variants → `PolicyNode::$variant(..)` matches each.
- **(b) De-vacuification chain** correctly stated: new variant → macro `kind()` non-exhaustive → COMPILE error → extend macro → NODE_KINDS grows → `node_kinds_cover_enum` (samples==NODE_KINDS) RUNTIME-fails until the sample is added. REMINDER match kept (belt-and-suspenders compile-time forcing of the sample visit); the test is NOT redundant.
- **(c) RED-proof** sound: either add a throwaway variant (→ node_kinds_cover_enum REDs) OR drop a macro entry (→ kind() compile-fails). Both are valid forcing proofs.
- **(d) Part B deferral** correct: the 19-file allowlist is unaudited; verify_bundle.rs/ms_shares.rs likely need new canonical ZEROIZE_ROWS; blanket-allowlisting would encode a false "transient" record. Split to its own cycle; slug stays OPEN.

No remaining findings. Ready for implementation.
