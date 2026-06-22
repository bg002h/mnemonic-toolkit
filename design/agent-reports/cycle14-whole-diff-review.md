# WHOLE-DIFF REVIEW — cycle-14 (close L22 — stdin/@env: secret → SecretString)

Secret-hygiene cycle for a self-custody steel-backup toolkit (first-class hygiene bar). Worktree `wt-cycle14`, off `origin/master = 450b0236` (v0.66.0). Commits `7daf8c69`/`db0bf583`/`e72ff8c2`.

## VERDICT: GREEN (0 Critical / 0 Important)

`cargo test -p mnemonic-toolkit` + `cargo clippy --workspace --all-targets -- -D warnings` GREEN; `=-` stdin + `@env:` channels behaviorally confirmed end-to-end.

### Axis 1 — secret-residue completeness & no-leak (HIGHEST): PASS
- **Coverage:** every owned secret-bearing copy reachable from stdin `=-` / `@env:` is zeroize-on-drop — `SlotInput.value` (`parse_slot_input` ctor `slot_input.rs:189` + `apply_slot_stdin` store `:228`, both `SecretString`); the 3 `@env:` write-backs (`bundle.rs:2633`, `import_wallet.rs:1402`, `verify_bundle.rs:1885`) re-wrap into `SecretString`; P2 convert/restore/addresses handler locals `Zeroizing<String>`. The only remaining bare-`String` copies are the two explicitly-deferred, status-quo-preserving, FOLLOWUP-tracked ones (`phrase_overlays` Vec `.to_string()` `import_wallet.rs:1237`; stdin-reader transient `buf`) — neither NEW residue. The 16 `&*s.value` read-sites only borrow `&str`.
- **Debug-redaction:** `SecretString::Debug` length-only (`secret_string.rs:61-64`). No `{:?}`/`{}`/log/error/panic prints the secret — the `"passphrase: {}"` sites (`restore.rs:582/669/696/1036`) print the applied-flag BOOLEAN; `parse_slot_input` `{:?}` error sites print the raw argv token on parse-FAILURE only (pre-existing). `eq_failure_debug_does_not_leak` proves the `assert_eq!`-panic path is redacted.
- **Plain PartialEq/Eq safe:** only `== "-"` sentinel + test asserts; no secret-vs-secret attacker-observable timing boundary.

### Axis 2 — `TemplateSeed.passphrase: String → Zeroizing<String>` (un-planned deviation): PASS
Genuine residue-closing improvement, not a behavior change; matches the struct's existing `entropy: Zeroizing<Vec<u8>>`. Consumers compile via deref-coercion (`&seed.passphrase` `restore.rs:1370`/`verify_bundle.rs:860` → `MultisigCompletionCtx.passphrase: &'a str` `:1126` — borrow only, no bare-String escape). Never emitted (applied-flag only) → no wire change; empirically confirmed (`@env:` passphrase shifts fingerprint `73c5da0a → d7450d36`).

### Axis 3 — lint floor 35→37 + `bundle_unified.rs` allowlist: PASS
Empirically: 31 distinct `ZEROIZE_ROWS` source_files (now incl. `src/slot_input.rs`) ∪ 6 `NON_ROW_SECRET_FILES` = 37, no overlap. `bundle_unified.rs`'s sole secret-pattern match (`:128`) is inside `#[cfg(test)] mod tests` — a `SlotInput` fixture helper; production owner `slot_input.rs` carries the real row. Allowlisting mirrors the `secret_string.rs` PRIMITIVE precedent; the lint tripwires pass.

### Axis 4 — 16+ `.value.as_str()` → `&*s.value` rewrites: PASS
Each semantically identical (`&*s.value` derefs `SecretString → str` then `&` → `&str`). Spot-checked `Mnemonic::parse_in`, `seedqr::decode`, `normalize_xpub_prefix`, `resolve_ms1_slot`, `Fingerprint::from_str`, `pin_pages_for(.as_bytes())`. No clone introduced, no value swapped. `seedqr.rs:172`/`seed_xor.rs:304` operate on `FromInput` (`.value` stays `String`) — correctly unaffected.

### Axis 5 — scope / behavior / wire / gates: PASS
No CLI/exit/`--json`/wire change (transparent Serialize/Display/Deref). `FromInput.value` STAYS `String` (`convert.rs:133`). Version sweep complete (6 sites + CHANGELOG `^## mnemonic-toolkit [0.67.0]`). L22 ticked without over-claiming `phrase_overlays`. FOLLOWUPs `phrase-overlay-secretstring` + `stdin-reader-transient-buf-zeroizing` filed; Site-1 flipped "SCRUB LEG RESOLVED". `mlock.rs` untouched, no fmt churn, 19-file scope. Pre-existing fuzz break out-of-scope.

### Minor (non-blocking)
- **M1:** the whole-file allowlist of `bundle_unified.rs` could mask a FUTURE production secret allocation (it has production code `:1-117` + the test fixture). Identical to the established `nostr.rs`/`bsms_crypto.rs` precedent; the `non_row_secret_allowlist_…` tripwire keeps it honest. Acceptable as-is; a future cycle could tighten to row-promotion if production secret code lands there. (Fed to the constellation secret-key-material sweep.)

## Disposition
GREEN. Clear to ship toolkit v0.67.0 (closes L22 — the last open bug-hunt finding except the documented won't-fix L16).
