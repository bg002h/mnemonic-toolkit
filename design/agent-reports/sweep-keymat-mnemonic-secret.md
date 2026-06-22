# Secret-key-material hygiene sweep — `mnemonic-secret` (ms-codec + ms-cli)

**Scope:** SECRET KEY MATERIAL hygiene in the most-sensitive constellation repo —
`ms-codec` encodes/decodes BIP-39 ENTROPY itself (`ms1` card) + the codex32 K-of-N
secret-share intermediates; `ms-cli` is the `ms` CLI intake/emit surface.
**Audited against:** `origin/master` @ `e80ea3b` (`release: ms-cli 0.9.0 — cycle-8`).
**Mode:** RECON/AUDIT ONLY. No fixes, no spec, no source edits. Candidate FOLLOWUP slugs below.
**All `file:line` reproduced against current `origin/master` source.**

---

## CODEC-LIBRARY internal-buffer posture (the headline question)

The codec library **DOES** wrap its OWNED internal entropy buffers in `Zeroizing`
at the encode/decode/share spines (SPEC v0.9.0 §1 item 2, Cycle A):

- `envelope::discriminate` payload buffer — `Zeroizing<Vec<u8>>` (`envelope.rs:169`).
- `envelope::package` / `payload_wire_bytes` encode buffer — `Zeroizing<Vec<u8>>` (`envelope.rs:231,261`).
- `decode()` Entr/Mnem allocation — `Zeroizing::new(...)` (`decode.rs:82,89`).
- `shares::encode_shares` CSPRNG filler — `Zeroizing<Vec<u8>>` (`shares.rs:139`).
- `shares::combine_shares` recovered-secret bytes — `Zeroizing<Vec<u8>>` (`shares.rs:301`).
- Enforced by `crates/ms-codec/tests/lint_zeroize_discipline.rs` (5 rows).

**BUT** there are real residual gaps the lint does not cover: (a) the `decode()`
"scrub" is undone by an immediate `.clone()` into a bare public `Vec` (theater);
(b) the codex32 SHARE STRINGS themselves (`Vec<Codex32String>` / `Vec<String>` /
the `secret_s` full-secret) are held in bare String-backed types with no scrub —
only the recovered `secret` Codex32String is partially tracked (`[obs]
recovered-secret-string-not-zeroized`); (c) the public `inspect()` returns raw
entropy in a `#[derive(Debug)]` struct with a bare `Vec<u8>` field — a leak surface.

So: **the codec mostly zeroizes its `Vec<u8>` entropy buffers, but NOT the
String-backed codex32 share material, and its `decode()` scrub is defeated by a
clone-into-bare-Vec, and `inspect()` exposes raw entropy via a Debug-deriving
public struct.**

---

## CANDIDATE FOLLOWUP SLUGS

### 1. `ms-codec-inspect-report-payload-bytes-bare-and-debug` — NEW
- **Secret type / gap class:** raw BIP-39 entropy `Vec<u8>` / classes 1 (bare buffer) + 2 (Debug leak).
- **Where:** `crates/ms-codec/src/inspect.rs:36-56` (struct), `:48` (`pub payload_bytes: Vec<u8>`), `:34` (`#[derive(Debug, Clone)]`), `:79-84,103` (populated from `c.parts().data()`).
- **What:** `InspectReport` is a PUBLIC codec API struct deriving `Debug` with a bare `pub payload_bytes: Vec<u8>` holding the raw decoded entropy (sans prefix). `{:?}` of an `InspectReport` (or any wrapper deriving Debug over it) prints the FULL entropy; the `Vec<u8>` is never `Zeroizing`, so it lives un-scrubbed until drop. This is the one public codec entry point that hands back raw secret bytes in a Debug-printable, non-scrubbing container.
- **Severity:** **High** (codec-library leak-to-Debug of root entropy + bare buffer; both escalation triggers fire).
- **Why:** The single public codec surface that returns un-wrapped, Debug-printable raw entropy; an upstream `{:?}`/`expect`/log on the report dumps the seed.

### 2. `ms-codec-decode-scrub-defeated-by-clone-into-bare-vec` — NEW
- **Secret type / gap class:** entropy `Vec<u8>` / class 1 + 6 (intermediate not effectively scrubbed).
- **Where:** `crates/ms-codec/src/decode.rs:82-83` (Entr) and `:89-90` (Mnem).
- **What:** The "scrub" pattern is `let scrubbed: Zeroizing<Vec<u8>> = Zeroizing::new(data); let p = Payload::Entr((*scrubbed).clone());` — the `.clone()` allocates a FRESH bare `Vec<u8>` that becomes the live public payload and is never scrubbed; the `Zeroizing` only scrubs the original (already-moved-from-an-already-Zeroizing-envelope-buffer) `data`. Net effect: an EXTRA un-scrubbed heap copy of the entropy is created, not eliminated. The lint anchors on `let scrubbed: Zeroizing<Vec<u8>>` so it reads GREEN while the clone defeats the intent.
- **Severity:** **Med** (entropy-in-bare-buffer in the codec library; the clone is a redundant un-scrubbed copy — defense-in-depth, the public boundary is bare by design per OOS-2, but this ADDS an avoidable copy).
- **Why:** A zeroize that visibly does the opposite of its stated purpose (creates a copy) is worse than honest caller-wrap; lint gives false assurance.

### 3. `ms-codec-share-strings-not-zeroized-encode-and-combine` — NEW (broadens tracked `[obs] recovered-secret-string-not-zeroized`)
- **Secret type / gap class:** codex32 share strings + the secret-at-S (full secret) / class 1 + 4 (String-backed copies escaping).
- **Where:** `encode_shares`: `secret_s: Codex32String` (`shares.rs:130`, the FULL secret at index S), `defining: Vec<Codex32String>` (`:136`), `distributed: Vec<String>` (`:148`), `single` (`:115`). `combine_shares`: `parsed: Vec<Codex32String>` x2 (`:195,210`, holds every INPUT share), `secret: Codex32String` (`:281`, the recovered full secret).
- **What:** `Codex32String` is a newtype over `String` (codex32-0.1.0) with NO Drop/Zeroize. Every share string and the secret-at-S string is held bare and dropped without scrub. The existing `[obs] recovered-secret-string-not-zeroized` covers ONLY the recovered `secret` binding in `combine_shares`; the input share vectors (`parsed`), the `secret_s` full-secret in `encode_shares`, and the `.clone()` copies at `from_string(s.clone())` (`:197`) / `c.to_string().to_ascii_lowercase()` (`:213`) are the same class but NOT individually tracked. A `Codex32String` IS secret-equivalent (any share leaks partial secret; secret_s/the recovered secret leak everything).
- **Severity:** **Med** (codec-library secret material in bare String-backed buffers; the secret-at-S and recovered-secret are total-leak-equivalent → arguably High for those two specific bindings).
- **Why:** The whole share pipeline carries secret-equivalent strings un-scrubbed; only one of ~7 such bindings is tracked. Root cause is the dormant-upstream `rust-codex32-zeroize-upstream` (already tracked) — but the per-binding count + the `secret_s`/`parsed` sites should be enumerated so a vendor/fork decision sees the full surface.

### 4. `ms-codec-payload-entr-public-bare-vec` — ALREADY TRACKED (verify, do NOT re-file)
- **Secret type / gap class:** entropy `Vec<u8>` / class 1.
- **Where:** `crates/ms-codec/src/payload.rs:44` (`Entr(Vec<u8>)`), `:55` (`entropy: Vec<u8>`).
- **What:** Public `Payload::Entr(Vec<u8>)` / `Payload::Mnem{entropy}` are bare. Widening to `Zeroizing<Vec<u8>>` is a breaking change; caller-wrap contract is documented (`:19-28`).
- **Tracked as:** `ms-codec-payload-zeroize-public-api` (FOLLOWUPS.md, `open` / OOS-public-payload). **Listed for completeness only.**

### 5. `ms-cli-inspect-intake-and-entropy-not-zeroized` — NEW
- **Secret type / gap class:** ms1 string + raw entropy / class 1.
- **Where:** `crates/ms-cli/src/cmd/inspect.rs:33` (`let ms1 = read_input(...)` — NOT `Zeroizing`-wrapped), `:34` (`ms_codec::inspect(&ms1)` → bare `InspectReport` with raw `payload_bytes`), `:217,247` (`hex::encode(&report.payload_bytes)`).
- **What:** `ms inspect` is the ONLY ms1-intake command that does NOT wrap its input in `Zeroizing` (contrast decode.rs:42, verify.rs:55, derive.rs:181, repair via read_input). The ms1 string (seed-secret-equivalent) sits in a bare `String`, and the `InspectReport.payload_bytes` raw entropy is bare too (per finding #1). For a v0.1 single-string, `ms inspect` prints the full entropy hex on stdout AND holds it un-scrubbed.
- **Severity:** **Med** (CLI intake of seed-secret-equivalent material in a bare buffer; the lint's 10 rows do not enumerate inspect).
- **Why:** Asymmetry — every sibling intake wraps; inspect is the lone bare hole, and it also handles the bare `payload_bytes` from #1.

### 6. `ms-cli-repair-intake-and-report-strings-not-zeroized` — NEW
- **Secret type / gap class:** ms1 string (seed-secret-equivalent) / class 1 + 4.
- **Where:** `crates/ms-cli/src/cmd/repair.rs:75` (`let original = read_input(...)` — bare `String`), `:89` (`original.clone()`), `:63-70` (`RepairDetail{ original_chunk: String, corrected_chunk: String }` bare), `:90-94` (`corrected_chunk.clone()` + `vec![corrected_chunk]`).
- **What:** `ms repair`'s ms1 input is held in a bare `String`, cloned into `RepairDetail.original_chunk`/`corrected_chunk` (both bare `String`), and the corrected ms1 (a valid, decodable secret string) is collected into `corrected_chunks: Vec<String>` — all un-scrubbed. The whole repair path carries seed-secret-equivalent material in bare buffers.
- **Severity:** **Med** (CLI carries seed-secret-equivalent ms1 strings in bare buffers across multiple copies; not in the lint's 10 rows).
- **Why:** Repair re-emits a fully-valid ms1 (recoverable secret) and never wraps the intermediate strings; multiple bare clones widen the residue window.

### 7. `ms-cli-derive-xpriv-master-not-zeroized` — NEW
- **Secret type / gap class:** master/account `Xpriv` (private key derived from seed) / class 1.
- **Where:** `crates/ms-cli/src/cmd/derive.rs:203` (`Xpriv::new_master(...)`), `:215-217` (`master.derive_priv(...)` → `acct_xpriv`).
- **What:** The seed is `Zeroizing<[u8;64]>` + mlock-pinned (`:200-201`, good), but the derived `master` and `acct_xpriv` `bitcoin::bip32::Xpriv` values hold the root/account PRIVATE keys and have no Drop/Zeroize (rust-bitcoin); they sit bare on the stack/heap until scope-end. Same third-party-blocked class as the tracked `rust-bip39-mnemonic-zeroize-upstream`, but for `Xpriv` rather than `Mnemonic`, and not currently tracked.
- **Severity:** **Med** (live root private key in a bare third-party type; defense-in-depth, upstream-blocked — but `ms derive` is the only place an actual xpriv is materialized).
- **Why:** A root xpriv is as sensitive as the seed; no FOLLOWUP currently names the rust-bitcoin Xpriv zeroize gap (only the bip39 Mnemonic one is tracked).

### 8. `ms-cli-json-output-structs-bare-secret-strings` — NEW (broadens tracked decode-emit obs)
- **Secret type / gap class:** entropy hex / phrase / shares in bare `String` / class 1 + 4.
- **Where:** `crates/ms-cli/src/format.rs` — `EncodeJson.entropy_hex: String` (`:61`), `DecodeJson.entropy_hex` + `.phrase: String` (`:100-101`), `CombineJson.entropy_hex` + `.phrase` + `.ms1` (`:87-90`), `SplitJson.shares: Vec<String>` (`:69`), `InspectReportJson.payload_bytes_hex` (`:129`). Plus each `emit_json`'s `let s = to_string(&json)` buffer (e.g. decode.rs:133, combine.rs:144, split.rs:148).
- **What:** All `--json` emit structs carry secret material (hex entropy, full phrase, full share set, ms1) as bare owned `String`/`Vec<String>` fields, plus the serialized output `String`. None are `Zeroizing`. They are short-lived (build → serialize → drop) but hold plaintext secret in un-scrubbed heap until drop.
- **Severity:** **Low** (STDOUT-LEAK-adjacent: the data goes to stdout by design one syscall later; defense-in-depth only).
- **Tracked partially as:** `ms-cli-decode-emit-zeroize-intermediate` (FOLLOWUPS.md, `open`, decode-only, OOS-decode-stdout). This slug would BROADEN it to encode/combine/split/inspect JSON structs + the struct fields (not just the intermediate String). Flag for dedup; orchestrator may fold into the existing entry.

### 9. `ms-cli-verify-derived-to-string-temp-not-wrapped` — NEW (low-confidence; verify against lint)
- **Secret type / gap class:** BIP-39 phrase `String` / class 1.
- **Where:** `crates/ms-cli/src/cmd/verify.rs:146` (`_mnemonic.to_string()` inside `emit_round_trip_ok`).
- **What:** `emit_round_trip_ok` calls `_mnemonic.to_string()` to count words — a bare temporary `String` containing the FULL phrase, not `Zeroizing`-wrapped, dropped un-scrubbed. (The main compare path at `:92-93` IS wrapped via `derived_str`/`supplied_str`; this success-log temp is a separate, un-wrapped materialization.)
- **Severity:** **Low** (a single short-lived phrase temp on the success path; defense-in-depth).
- **Why:** One un-wrapped full-phrase temp slips past the otherwise-thorough verify zeroize discipline.

---

## VERIFIED-CLEAN (no slug — defensive controls confirmed present)

- **Error Debug/Display leak surface (codec):** `ms_codec::Error` hand-rolls `Debug`→`Display` and peels off the 3 leaky `codex32::Error` String-carrying variants (`InvalidChecksum`/`MismatchedHrp`/`MismatchedId`) + caps `WrongHrp.got` to 4 chars. Red-first no-echo tests present (`error.rs:257-383`). Tracked-and-resolved as `ms-codec-error-display-echoes-input` (0.4.4). **Clean.**
- **ms-cli error leak surface:** `CliError` derives `Debug`, but carried fields are non-secret (indices/lengths/tags); `friendly_bip39` uses `UnknownWord(idx)` (index only, no bad word echoed); `friendly_codex32` drops `InvalidChecksum.string`. **Clean.**
- **stdin intake:** `parse::read_stdin` returns `Zeroizing<String>` + mlock-pins the buffer (`parse.rs:65-82`); `read_phrase_input`/`read_stdin_passphrase` return `Zeroizing<String>`. **Clean** (Cycle-A, verified current).
- **argv hygiene:** `secret_in_argv_warning` advisories on inline `--phrase`/`--hex`/`--passphrase`/`ms1` (derive.rs:131-148); `process_hardening::set_non_dumpable` (PR_SET_DUMPABLE 0). **Clean.**
- **encode/decode/combine/split CLI entropy locals:** all `Zeroizing<Vec<u8>>` with `mem::take`-into-Zeroizing on clap fields. **Clean.**
- **Secret-compare timing (class 7):** verify compares `*supplied_str == *derived_str` (plain `String` eq) and combine guards on share-index `b's'` — neither is a real constant-time-expected secret compare (validity checks over public-on-this-path or checksum-gated material), so no timing-leak slug. **No finding.**

---

## SUMMARY TABLE

| # | slug | class | severity | NEW? |
|---|------|-------|----------|------|
| 1 | `ms-codec-inspect-report-payload-bytes-bare-and-debug` | 1+2 | High | NEW |
| 2 | `ms-codec-decode-scrub-defeated-by-clone-into-bare-vec` | 1+6 | Med | NEW |
| 3 | `ms-codec-share-strings-not-zeroized-encode-and-combine` | 1+4 | Med (High for secret_s/recovered) | NEW (broadens tracked obs) |
| 4 | `ms-codec-payload-zeroize-public-api` | 1 | — | ALREADY TRACKED |
| 5 | `ms-cli-inspect-intake-and-entropy-not-zeroized` | 1 | Med | NEW |
| 6 | `ms-cli-repair-intake-and-report-strings-not-zeroized` | 1+4 | Med | NEW |
| 7 | `ms-cli-derive-xpriv-master-not-zeroized` | 1 | Med | NEW |
| 8 | `ms-cli-json-output-structs-bare-secret-strings` | 1+4 | Low | NEW (broadens tracked decode-emit) |
| 9 | `ms-cli-verify-derived-to-string-temp-not-wrapped` | 1 | Low | NEW (low-confidence) |

**7 NEW candidate slugs** (1 dup listed for completeness; #3 and #8 broaden existing
tracked entries — orchestrator to dedup). Highest severity: **#1 (High)** — public
`inspect()` returns raw entropy in a Debug-deriving, non-scrubbing struct.
</content>
</invoke>
