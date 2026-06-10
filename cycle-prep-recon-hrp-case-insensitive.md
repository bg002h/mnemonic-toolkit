# cycle-prep recon — 2026-06-10 — hrp-classifier-rejects-valid-uppercase-cards (audit M11)

**Origin/master SHA at recon time:** `38db912` (= v0.53.2)
**Local branch:** `master`
**Sync state:** up-to-date (0 ahead / 0 behind)
**Untracked:** `.claude/`, `CONTINUITY.md`, cycle-b-* scripts/decisions, ~25 prior `cycle-prep-recon-*.md`, 2 feature-coverage surveys (all session scratch; none staged)

Slug(s) verified: `hrp-classifier-rejects-valid-uppercase-cards` (FOLLOWUPS.md:17 index line; full finding `design/agent-reports/constellation-audit-2026-06-10-findings.json` id `hrp-classifier-rejects-valid-uppercase-cards`). Drift expectation was "v0.53.2 advisory shifted repair.rs lines" — **measured: NO drift**; the M3 advisory landed inside `resolve_groups` (repair.rs:297-315), *below* `classify_hrp_prefix`, so the cited :106-119 lines are exact.

---

## Per-slug verification

### hrp-classifier-rejects-valid-uppercase-cards

- **WHAT (from FOLLOWUPS.md):** Toolkit card-type prefix probes match lowercase `ms1`/`mk1`/`md1` only, but "the codecs decode all-uppercase strings per BIP-173" — so a valid all-uppercase card is misrouted to a confusing error. Audit fix: lowercase before the HRP probe.

- **Citations:**
  - `crates/mnemonic-toolkit/src/repair.rs:106-119` `classify_hrp_prefix` — **ACCURATE** (zero drift at 38db912). `:107 if s.starts_with("ms1")` / `:109 "mk1"` / `:111 "md1"`, else `:114-117 ToolkitError::UnknownHrp { got: s.to_string(), .. }`. Doc-comment `:103-105` declares the case-sensitivity *deliberately*: "The prefix-match is case-sensitive on the codec convention (lowercase per BIP-93)."
  - `crates/mnemonic-toolkit/src/cmd/restore.rs:1026` `values.iter().all(|v| v.starts_with("mk1"))` — **ACCURATE** (line exact). Context: `--cosigner @N=<mk1|xpub>` cross-check branch (restore.rs:1006-1048).
  - "mk-codec rejects only Mixed case then to_lowercase-normalizes" — **ACCURATE** at the pinned **crates.io mk-codec 0.4.0** (Cargo.lock:693-696): `string_layer/bch.rs:646-651` `decode_string` rejects only `CaseStatus::Mixed` (`:648-649`) then `let s_lower = s.to_lowercase()` (`:651`); unit test `:792` `case_check("MD1QQ") == CaseStatus::Upper`.
  - "the codecs decode all-uppercase strings per BIP-173" (blanket, all three) — **STRUCTURALLY-WRONG for ms-codec**. At pinned **ms-codec 0.4.0** (Cargo.lock:744-747): the codex32 layer *does* accept consistent-uppercase (codex32-0.1.0 `checksum.rs:146-170` `set_check_case` allows `Case::Upper`, rejects mixed with `InvalidCase`; `field.rs:58-67` `CHARS_INV` maps both cases), **but** `envelope.rs:100` compares `fields.hrp != HRP` on the RAW string (`consts.rs:11 HRP = "ms"`) → all-uppercase `MS1…` fails `Error::WrongHrp { got: "MS" }`; even past that, `envelope.rs:112` `share_index_byte != SHARE_INDEX_V01` (`consts.rs:23` = `b's'`) would reject the uppercase `S` index. **ms-codec does NOT decode all-uppercase today.** md-codec 0.35.0 (Cargo.lock:655-658) is accurate and *over*-lenient: `codex32.rs:92-119` `unwrap_string` and `chunk.rs:429-431` `parse_chunk_symbols` per-char `to_ascii_lowercase()` with **no mixed-case rejection** → md-codec accepts uppercase AND mixed-case.
  - "restore.rs:1026 is false for an uppercase mk1 cosigner → falls through to Xpub::from_str → confusing 'xpub parse' message" — **ACCURATE** for a *single* uppercase value (restore.rs:1030-1032 `"--cosigner @{n} xpub parse: {e}"`). For *multiple* uppercase chunks the consequence is restore.rs:1035 `"--cosigner @{n}: multiple values must all be mk1 chunks, or a single xpub"`. Both hard errors; no silent wrong result.

- **Callers of `classify_hrp_prefix` (all positional-autodetect paths):**
  - `repair.rs:312` (M3 advisory probe) + `repair.rs:324` (routing) inside `resolve_groups` ← `cmd/inspect.rs:105` and `cmd/repair.rs:111`.
  - `cmd/verify_bundle.rs:1242` (`apply_positional_hrp_autodetect`).
  - `cmd/xpub_search/seed_intake.rs:129` (positional ms1-only intake; `Err(_)` arm `:138-145` gives a clean BadInput without echoing).

- **Consequence of an uppercase `MS1…` card today, per path:**
  - **positional to inspect/repair/verify-bundle:** `UnknownHrp` at repair.rs:324 / verify_bundle.rs:1242 (exit 2) — and **`UnknownHrp`'s Display (error.rs:794-800) echoes the FULL input string** (`"positional argument '{got}' does not begin with…"`). For an uppercase ms1 that is **the full secret echoed to stderr**. Additionally the v0.53.2 M3 advisory at repair.rs:311-315 does **NOT fire** (the `matches!(classify_hrp_prefix(s), Ok(CardKind::Ms1))` probe is case-sensitive) — confirmed security-adjacent miss, worth fixing in the same cycle. (The `--ms1` FLAG advisory at repair.rs:306-310 fires unconditionally regardless of case — only the positional advisory is case-gated.)
  - **typed flag `--ms1 MS1…` (also `--mk1`/`--md1`, and verify-bundle:185-203, seed_intake:106/114):** `validate_flag_hrp` (repair.rs:151-200) has an EXPLICIT case-mismatch branch `:178-187` → `HrpMismatch` with "(HRP case mismatch — lowercase canonical per BIP-93)". This is a *deliberate* v0.24.0 I5-fold rejection, not an accident — the brainstorm must decide whether to keep or relax it (see scope).
  - **`silent-payment --secret MS1…`:** silent_payment.rs:134 probe misses → falls through phrase (1 token, no) / hex (no) → clean refusal `:172-176` ("expected a seed-bearing secret …"); no echo, no panic, but mis-attributed.

- **Action for brainstorm spec:** Keep the toolkit-side fix (probe-only lowercasing) but **split the ms1 story**: mk1 + md1 uppercase become fully functional with the probe fix alone (codecs self-normalize); ms1 uppercase still fails inside ms-codec (`WrongHrp { got: "MS" }` — clean, correctly attributed, no secret echo) until an **ms-codec companion** (`envelope.rs:100/:112` case-normalize per BIP-173) ships and the toolkit pin bumps. Do not claim "uppercase ms1 round-trips" in the toolkit-only release. Cite source SHA `38db912` (toolkit) + crates.io `ms-codec 0.4.0` / `mk-codec 0.4.0` / `md-codec 0.35.0`.

---

## Full case-sensitive HRP-probe sweep (src/, at 38db912)

**PROBES — need case-insensitivity (fix sites):**

| Site | What it gates | Uppercase consequence today | After probe-fix (original passed to codec) |
|---|---|---|---|
| `repair.rs:107-111` `classify_hrp_prefix` | inspect/repair/verify-bundle positionals; seed_intake positional; M3 advisory | `UnknownHrp` + **full-string stderr echo** (secret for ms1); advisory misses | mk1/md1 decode; ms1 → ms-codec `WrongHrp("MS")` until companion; advisory fires |
| `repair.rs:312` (advisory probe, same fn) | "positional ms1" secret-argv advisory | does not fire | fires (fixed by the same function change) |
| `cmd/restore.rs:1026` | `--cosigner @N=mk1\|xpub` | single: "xpub parse" error; multi: ":1035 must all be mk1 chunks" | mk-codec decodes (works end-to-end) |
| `cmd/xpub_search/target_intake.rs:24` `resolve_target_xpub` | `--target-xpub` mk1-vs-SLIP-0132 dispatch (P1/P4 + address-of-xpub:187) | falls to SLIP-0132 `normalize_xpub_prefix` → confusing xpub-prefix error | mk-codec decodes |
| `cmd/xpub_search/address_of_xpub.rs:178` | guard skipping multisig-prefix detection for mk1 | uppercase mk1 enters `detect_multisig_prefix` (benign — `Ypub/Zpub/Upub/Vpub` don't match `MK1…`) then fails at :187 | consistent skip; comment :179-180 needs rewording |
| `cmd/xpub_search/descriptor_intake.rs:155` `detect_shape` | md1-shape detection for `--descriptor` | uppercase MD1 mis-detected as LiteralXpub → descriptor parse error | md1 route; md-codec decodes (works end-to-end) |
| `cmd/silent_payment.rs:134` | secret-kind dispatch | falls to clean refusal :172-176 (mis-attributed) | ms-codec `WrongHrp("MS")` until companion |
| `repair.rs:178` `validate_flag_hrp` case-mismatch branch | typed `--ms1/--mk1/--md1` flags (repair/inspect/verify-bundle/seed_intake) | DELIBERATE explicit rejection w/ good message (I5 fold) | **decision needed** — see scope |

**Cosmetic (optional):** `cmd/xpub_search/seed_intake.rs:204-213` `classify_hrp_str` — display-only helper for one error message; uppercase → `"<unknown>"` in the message text.

**NOT probes — leave alone:**
- **Already case-tolerant today (no probe in path):** `restore --md1` (restore.rs:816-817 → `md_codec::chunk::reassemble`, codec lowercases — uppercase MD1 restore WORKS today); `convert --from ms1|mk1|md1` (kind declared by flag, value straight to codec — uppercase mk1/md1 work, uppercase ms1 fails in ms-codec); toolkit BCH repair engine `repair.rs:574` (`parse_chunk` already `to_lowercase()`s — in-repo precedent for normalizing).
- **Emission/display — must stay lowercase:** `cmd/convert.rs:65-66,84-85,217-218` (shape-name enum, clap value strings); `cmd/repair.rs:303-305` (CardKind→str display); `cmd/ms_shares.rs:454` (output shaping); `verify_bundle.rs:2809` (check-NAME filter `"ms1_decode"`); `bundle.rs:2586` (check-name `"mk1_xpub_binding"`).
- **Sentinel probes, not HRP:** `import_wallet.rs:288`, `ms_shares.rs:136,146` (`starts_with("@env:")`).
- **Test fixtures/asserts on emission:** `synthesize.rs:994-1727`, `friendly.rs:460`, `tests/cli_auto_repair.rs:20-26` etc. — untouched.
- No `starts_with("MS1"/"MK1"/"MD1")`, no `== "ms1"`-style literal HRP equality on card data found anywhere else in src/.

**Mixed-case edge (post-fix, probe lowercased, ORIGINAL passed to codec):**
- ms1 mixed → codex32-0.1.0 `Error::InvalidCase` (checksum.rs:158) via `Codex32String::from_string` → ms-codec `Error::Codex32` → toolkit friendly mapper ("ms1 codex32: …"). Clean typed error, no panic.
- mk1 mixed → mk-codec `Error::MixedCase` (bch.rs:648-649). Clean.
- md1 mixed → **md-codec ACCEPTS** (no mixed check; per-char lowercase). Not an error — codec-side BIP-173 leniency, not a toolkit bug. Note as md-codec observation (optional companion `md-codec-accepts-mixed-case-bip173-leniency`); characterization-test it, don't "fix" toolkit-side.

---

## Cross-cutting observations

1. **Zero line drift** — unusual: the v0.53.2 M3 advisory landed *inside* `resolve_groups` (repair.rs:297-315), below `classify_hrp_prefix`, so both audit citations are exact at 38db912.
2. **The blanket codec claim is wrong for ms-codec** (primary-source verified at the pinned crates.io versions, not git checkouts — codecs moved to registry deps: md-codec 0.35.0 / mk-codec 0.4.0 / ms-codec 0.4.0 per Cargo.lock). The audit verified mk-codec only and generalized. ms-codec's envelope layer (envelope.rs:100, :112) is case-sensitive past codex32 → toolkit-only fix cannot make uppercase ms1 decode; needs an ms-codec companion + pin bump.
3. **Secret-echo hardening rider:** `UnknownHrp` Display (error.rs:794-800) interpolates the FULL unrecognized positional into the error. Today an uppercase ms1 secret hits exactly this. The probe fix removes that specific trigger, but any near-miss secret-ish positional still echoes — recommend truncating `got` (e.g. to the pre-`1` prefix) in the same cycle.
4. **Design tension the brainstorm must resolve explicitly:** `validate_flag_hrp`'s case-mismatch branch (repair.rs:175-187) deliberately REJECTS uppercase on typed flags (I5 fold, "lowercase canonical per BIP-93"). A probe-only fix would create an inconsistent surface (positional `MK1…` accepted, `--mk1 MK1…` rejected). Recommend relaxing the case-mismatch branch to accept consistent-case values in the same cycle, making "codecs are the authority on case" the uniform rule. The doc-comments repair.rs:103-105 and :169-170 and address_of_xpub.rs:179-180 must be rewritten either way.
5. The toolkit surface is ALREADY case-inconsistent today: `restore --md1 MD1…` and `convert --from mk1/md1 <UPPER>` work (no probe), while inspect/repair/verify-bundle/xpub-search reject. The fix is a consistency restoration, not a behavior novelty.
6. BIP-173 note for the spec: uppercase-only QR alphanumeric-mode encoding is the canonical *reason* uppercase cards exist in the wild — steel-engraved/QR'd cards may legitimately come back uppercase.

---

## Recommended brainstorm-session scope

**Cycle: single toolkit PATCH (v0.53.3), plus one sibling companion FOLLOWUP.**

1. **Fix shape: probe-only lowercasing; pass the ORIGINAL string to the codecs** (they self-normalize mk/md and are the acceptance authority). Do NOT normalize-at-intake (lowercasing a mixed-case string before the codec would case-launder input the codecs deliberately reject — mk `MixedCase`, ms `InvalidCase`).
2. **Sites (~40-80 LOC impl):** `repair.rs:106-119` `classify_hrp_prefix` (one change fixes inspect/repair/verify-bundle positionals + the M3 positional advisory at :312 + seed_intake:129); `restore.rs:1026`; `target_intake.rs:24`; `address_of_xpub.rs:178`; `descriptor_intake.rs:155`; `silent_payment.rs:134`; relax `validate_flag_hrp` repair.rs:175-187 case-mismatch branch (consistency, per observation 4 — architect to ratify); optional cosmetic `seed_intake.rs:204-213`. Rider: truncate `UnknownHrp.got` echo (error.rs:794-800). Doc-comment rewrites at repair.rs:103-105/:169-170, address_of_xpub.rs:179-180.
3. **Tests (~150-250 LOC):** uppercase MK1 end-to-end through inspect + restore `--cosigner` + `--target-xpub`; uppercase MD1 through inspect + xpub-search `--descriptor`; uppercase MS1 positional → advisory FIRES + ms-codec `WrongHrp` (no full-string echo) — flip the assertion when the companion lands; mixed-case ms1/mk1 → clean codec-attributed errors (no panic); md1 mixed-case accepted (characterization of codec leniency); `--ms1 MS1…` typed-flag behavior per the validate_flag_hrp decision.
4. **Companion (sibling repo, mirrored FOLLOWUPS entries):** `mnemonic-secret` — ms-codec `envelope.rs` case-normalize (`extract_wire_fields`/`discriminate` compare lowercased hrp + share-index) so all-uppercase ms1 decodes per BIP-173; then toolkit pin bump closes the ms1 leg. Optional md-codec observation FOLLOWUP for mixed-case leniency.
5. **SemVer:** PATCH v0.53.3 — error-path/acceptance behavior only; **no clap flag-name change → no GUI `schema_mirror` lockstep; no manual flag-coverage lockstep**. Manual makes no lowercase-only intake claim (grepped `uppercase|case-sensitive|case mismatch|BIP-93` across docs/manual/src — no case-acceptance prose for cards); optional one-line case-tolerance note in 41-mnemonic.md inspect/repair section + the :2863 advisory row already matches post-fix behavior.
6. **Ordering:** toolkit PATCH first (self-contained win for mk1/md1 + advisory + echo); ms-codec companion second; pin-bump fast-follow flips the ms1 uppercase test green. Mandatory R0 gate before any implementation.
