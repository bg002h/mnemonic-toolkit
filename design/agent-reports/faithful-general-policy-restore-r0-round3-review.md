# R0 Review — faithful general-policy restore (PART 1) — ROUND 3

**Source SHA:** `5d599f7` (live-verified: `restore.rs:839` ms0 / `:841` collapse / `:882` build call; `pipeline.rs:116-121` `taproot_internal_key.ok_or_else` hard-error; `bundle.rs:1036 extract_multisig_threshold -> Option<u8>`; `pipeline.rs:21 k: u8`; md-codec `tree.rs:9 pub struct Node`, `use_site_path.rs:49/:51`).

**Verdict: 🟢 — 0 Critical / 0 Important / 0 Minor (1 cosmetic residual, non-blocking)**

- **NEW-I1 — RESOLVED.** §1.C defines the unified binding explicitly: taproot arm → `(Some(tmpl), Some(ik))`; non-taproot → `(plain_template_from_tree(...), None)`; the descriptor `Some(template)` arm passes the variable `tap_internal_key` (NOT literal `None`), with the explicit NOTE banning literal `None` (cites `pipeline.rs:116-121`). Taproot path byte-for-byte unchanged (current `:882` already passes `tap_internal_key`). `k_opt.expect` holds for both template arms (plain ⇒ `Wsh→MultiKeys{k}`; taproot template only for `multi_a/sortedmulti_a` ⇒ `MultiKeys` ⇒ `Some(k)`).
- **NEW-M1 — RESOLVED.** Types corrected + grep-verified: `Option<u8>`, `k: u8`, `plain_template_from_tree(node: &md_codec::tree::Node, use_site: &md_codec::use_site_path::UseSitePath)`. *Cosmetic residual:* §1.B prose still writes the old `&md_codec::Tree`/`&md_codec::UseSitePath`; the §1.C grep-verified note governs and the compiler catches any literal transcription — not gate-relevant.
- **NEW-M2 — RESOLVED.** Root cause now reads `restore.rs:839`.
- **NEW-M3 — PRESENT.** Single-key N=1 wording note in scope boundary.
- **No new drift.** k-gate move intact (gate plain arm only; general → `faithful_multisig_descriptor`); 11-variant format table (4 faithful incl. BSMS + 7 refusing) unchanged; all four label sites + GUI mandate intact; discriminator untouched; test matrix (13+12 stay-green, both pk-keyed RED shapes) unchanged.

**GREEN. Gate passed — ready for implementation (TDD, RED-first per §Tests).**
