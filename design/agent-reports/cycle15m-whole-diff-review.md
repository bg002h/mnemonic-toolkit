# WHOLE-DIFF REVIEW — cycle-15 Lane M (ms-codec 0.6.0 / ms-cli 0.10.0)

Independent mandatory review. Worktree `wt-cycle15m`, off `mnemonic-secret origin/master = 6f9f60b`. Commits `e52d0cc`/`108e1ea`/`f943166`. Gates the ms-codec 0.6.0 / ms-cli 0.10.0 publish.

## VERDICT: GREEN — 0 Critical / 0 Important

The two highest-risk axes (redacting-Debug no-leak, wire/Payload byte-stability) verified EMPIRICALLY.

### Axis 1 — InspectReport redacting Debug — PASS (empirically proven)
- `payload_bytes` is `Zeroizing<Vec<u8>>` (`inspect.rs:60`); `#[derive(Debug)]` dropped, `Clone` retained (`:43`); hand-rolled `impl fmt::Debug` (`:70-91`) renders `payload_bytes` via `format_args!("[REDACTED; {} bytes]", …len())` — length-only.
- Traced every printed field (`hrp/threshold/tag/share_index/prefix_byte/checksum_valid/kind/language`) — NONE re-encodes entropy (`tag` = the "entr" tag bytes; `language` = wordlist index, structural).
- Dumped a real `{:?}` of a `deadbeef`-sentinel report: `payload_bytes: [REDACTED; 16 bytes]` — no `deadbeef` hex, no `222,173,190,239` decimal dump. Marquee test asserts absence of BOTH forms (sentinel `0xDE,0xAD,0xBE,0xEF`==`222,173,190,239`) + presence of placeholder + structural fields. Non-vacuous.
- `PartialEq` NOT derived. I-1 fix `inspect.rs:164 assert_eq!(*r.payload_bytes, entropy)` (derefs field).

### Axis 2 — Secret-residue completeness — PASS
`decode.rs:85-92` MOVES bytes into `Payload` (no `(*scrubbed).clone()`, no throwaway envelope; negative-anchor `decode_has_no_clone_into_bare_vec` passes). `RepairDetail` Debug dropped / Clone kept / chunk fields `Zeroizing<String>` (`repair.rs:69-73`); `repair_detail_does_not_derive_debug` passes; no `{:?}` consumer. All ms-cli intake/emit wrapped. codex32 `Codex32String` String-leg correctly ENUMERATE-AND-DEFER (`shares.rs:129-138`; FOLLOWUP kept `open` PARTIAL — no false-GREEN; reachable `Vec<u8>` filler + recovered-secret bytes stay `Zeroizing`). FULL RULE Z-DEBUG audit: only `RepairDetail` + `InspectReport` hold Zeroizing fields, both handled; every other `derive(Debug)` (`Payload`, `CorrectionDetail`, clap Args) holds no Zeroizing field.

### Axis 3 — Wire/Payload byte-stability — PASS (proven by cross-tree diff)
Encoded identical vectors on `6f9f60b` vs branch → BYTE-IDENTICAL. `Payload` enum shape unchanged (`Entr(Vec<u8>)`/`Mnem{...}`, bare-by-design). `--json` unchanged (`InspectReportJson.payload_bytes_hex` = `hex::encode(&report.payload_bytes)` via Deref; all `*Json` derive only `Serialize`). Recompile-only pin bump for the downstream toolkit.

### Axis 4 — cargo-fmt deviation — PASS (no introduced churn)
`cargo fmt --all --check` flags 149 locations across files the branch NEVER touched (`cli_derive.rs`, `cli_output_class.rs`, `cli_split.rs`) → pre-existing repo state (default-rustfmt vs committed code), NOT churn. The implementer correctly avoided `cargo fmt --all`. CI clippy-only (`rust.yml:153`, no fmt gate); clippy clean.

### Axis 5 — SemVer / publish / lint / FOLLOWUPs — PASS
ms-codec 0.5.0→0.6.0, ms-cli 0.9.0→0.10.0, pin `=0.6.0`, Cargo.lock, both CHANGELOGs — consistent. Lint: ms-codec 4 rows asserts 4, ms-cli 13 rows asserts 13 (verify row RE-POINTED to `wc_src` at `emit_round_trip_ok`, not appended — old false-green `derived_str` anchor gone). FOLLOWUPs: 6 resolved + #3/#7 PARTIAL-open + new `rust-bitcoin-xpriv-zeroize-upstream`.

### Gates
`cargo test -p ms-codec -p ms-cli`: 371 passed, 0 failed, 5 ignored. `cargo clippy --all-targets -- -D warnings`: exit 0.

## Disposition
GREEN. Clear to publish ms-codec 0.6.0 → ms-cli 0.10.0 (publish ms-codec first, then ms-cli with the `=0.6.0` pin).
