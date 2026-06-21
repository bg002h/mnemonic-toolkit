# R0 REVIEW — cycle-11a GUI hygiene (M9 · L12 · L13) — Round 3 (AUTHORITATIVE GATE)

**Spec:** `design/BRAINSTORM_cycle11a_gui_hygiene.md`
**GUI repo:** `mnemonic-gui` `master = 0bbe3e1` (v0.45.0, pins toolkit v0.60.0).
**Toolkit source verified against:** tag `mnemonic-toolkit-v0.60.0` (the pin) + `origin/master` (= `wt-tk-master` worktree, v0.65.0) + tag `mnemonic-toolkit-v0.62.0`.
**This round's mandate:** adversarially verify the round-2 fold's NEW dual-version source claims against AUTHORITATIVE source (not the doc) — the load-bearing-protocol-fact class the project recon discipline exists to catch. History: round-1 I1 = fabricated `:113-118` "supply exactly one" citation; round-2 I2 = residual inaccurate `:289` "key annotation" citation + a v0.60.0-vs-master version contradiction. This is the third pass on the SAME L12 rationale — held to a high bar precisely because two prior folds drifted.

## VERDICT: **GREEN — 0 Critical / 0 Important**

The round-2 fold's dual-version rationale is FACTUALLY CORRECT, SOURCE-VERIFIED, and EMPIRICALLY CONFIRMED on both binaries. The two prior fabrications (`:113-118`, `:289`) are fully purged. No new drift introduced. **The lane may proceed to the plan-doc stage.**

---

## Source + empirical verification of every NEW dual-version claim

### Claim 1 — v0.60.0 (the pin) is a SUFFIX-ONLY lexer that ACCEPTS the double-origin. **CONFIRMED.**
- `git show mnemonic-toolkit-v0.60.0:…/parse_descriptor.rs`: the lexer `Regex::new(` is at file line **69**, regex string at **70** → spec's `:69-70` is **exact**.
- The regex is `r"@(\d+)(?:\[([0-9a-fA-F]{8})((?:/\d+(?:'|h)?)*)\])?(?:/<([0-9;]+)>)?(/\*(?:'|h)?)?"` — it begins with `@(\d+)`; the optional origin bracket sits IMMEDIATELY AFTER `@N`. There is NO prefix-bracket alternative. A leading `[fp]` before `@N` is genuinely not part of the pattern; `captures_iter` skips it. Positional captures 1–5 only — `pfx_fp`/`sfx_fp` named groups do NOT exist at v0.60.0.
- Whole-file grep of v0.60.0 `parse_descriptor.rs` for "supply exactly one" / double-origin refusal: **ZERO hits.** v0.60.0 has no double-origin guard. (Confirms the round-1 I1 string never existed at the pin.)
- **Empirical (v0.60.0 binary `/tmp/mnemonic-v0600/bin/mnemonic`, `--version` = `mnemonic 0.60.0`):**
  - `gui-schema --classify-descriptor "wpkh(@0[deadbeef/84'/0'/0']/<0;1>/*)"` → `canonical`, exit 0 (single suffix-form accepted — the L12 form).
  - `…"wpkh([deadbeef/84'/0'/0']@0/<0;1>/*)"` → `canonical`, exit 0 (prefix-form).
  - `…"wpkh([deadbeef/84'/0'/0']@0[cafef00d/84'/0'/0']/<0;1>/*)"` → **`canonical`, exit 0** (DOUBLE-ORIGIN ACCEPTED — exactly the spec's pin claim).

### Claim 2 — master / v0.62.0+ named-group lexer REFUSES the double-origin. **CONFIRMED.**
- `git show origin/master:…/parse_descriptor.rs` (grep -n): `Regex::new(` at 97, named-group regex string `(?P<pfx_fp>…)…(?P<sfx_fp>…)` at **98** → spec's `:98` (line 117/D7) and `:97-98` (line 91) are **exact**. Named groups `pfx_fp` (:111), `sfx_fp` (:112), refusal `if pfx_fp.is_some() && sfx_fp.is_some()` at **:113**, refusal string `"…double-origin is ambiguous — supply exactly one"` at **:116** → spec's `:113-116` is **exact** (and the byte-exact string matches the spec quote at line 117/D7).
- The **`mnemonic-toolkit-v0.62.0` tag** carries the identical named-group lexer + refusal at the same lines → the spec's "shipped by H7 commit `36095b88` … v0.62.0+" attribution is correct.
- H7 commit confirmed: `36095b88 fix(cycle2-h7): ACCEPT BIP-380 prefix-form [fp/path]@N key-origin (funds-safety)`.
- **Empirical (master binary `wt-tk-master/target/release/mnemonic`, `--version` = `mnemonic 0.65.0`):**
  - single suffix-form → `canonical`, exit 0 (still accepted, as claimed).
  - prefix-form → `canonical`, exit 0.
  - DOUBLE-ORIGIN → `error: @0 carries BOTH a prefix … and a suffix … ambiguous — supply exactly one`, **exit 2** (REFUSED at parse — exactly the spec's master claim).

### Claim 3 — the dual-version LOGIC is sound; no WRONG-result scenario. **CONFIRMED.**
- GUI Canonical-classification ⇒ it PINS `--account` to `PinValue(0)` (it does not forward user `--account N`).
- v0.60.0 `cmd/bundle.rs:1401`: `if !is_non_canonical && args.account != 0 { … DESCRIPTOR_WITH_NONZERO_ACCOUNT … }`. The guard fires ONLY on `account != 0`. The pin value IS 0 ⇒ guard is false ⇒ no error; and **0 is the toolkit's descriptor-mode default** (account index encoded in the `@i` origin path — confirmed by `bundle --help`: "BIP-32 account index (default 0)"). So pinning 0 is a NO-OP relative to descriptor semantics; it can never OVERRIDE the account in the origin path.
- Outcome lattice for "GUI Canonical + `--account 0` pin": **{correct (v0.60.0, account-0 default no-op), refused-at-parse (v0.62.0+ double-origin only)}** — never silently-wrong-address. D7's "don't tighten the GUI regex" decision survives both versions.

### Claim 4 — no residual fabrication / contradiction. **CONFIRMED CLEAN.**
- Spec grep for `:289` / "key annotation" / "tr([fp": **ZERO hits** (round-2 I2 residue fully purged).
- "supply exactly one" appears ONLY at lines 117 + 219 (D7), BOTH correctly anchored to **master / v0.62.0+** — never to the pin. No misattribution.
- Version-anchoring is internally consistent across §3.2 lines 91/115/117/119, the FOLLOWUP slug (line 203), and D7 (line 219): uniformly "v0.60.0 ACCEPTS (pin fine) / master+v0.62.0+ REFUSES at parse (pin moot) / benign either way." No contradiction (the round-2 I2 contradiction is resolved).

### Claim 5 — M9 and L13 SOUND and untouched. **CONFIRMED.**
- The fold was L12-scoped; M9 (§3.1 / D1–D4) and L13 (§3.3 / D8–D9, D11–D12) bodies are structurally unchanged from the round-1-confirmed-sound state.
- Spot re-verify of L13 anchors at the pin: v0.60.0 `cmd/convert.rs:58` = `Self::Seedqr => "seedqr"` (present, index-1 ordering holds). Seedqr-intro commit `5f0b7b45` = "v0.31.6 — SeedQR --from unification" (the round-1-corrected M-min-2 SHA; v0.31.6 < v0.60.0 ⇒ no-pin-bump conclusion intact).

---

## MINOR (non-blocking — do NOT gate; note for plan-doc citation-lift hygiene)

- **m-1 (path shorthand in the L12 problem-statement, line 91 — NOT in the fold under review):** the spec cites `bundle.rs:194` / `bundle.rs:1396-1407` / `gui_schema.rs:1318-1325`, but the real toolkit paths are `src/cmd/bundle.rs` and `src/cmd/gui_schema.rs`. The line numbers and the verbatim error string resolve correctly (`cmd/bundle.rs:194` = `DESCRIPTOR_WITH_NONZERO_ACCOUNT` byte-exact; `cmd/bundle.rs:1398` = `canonical_origin(&…tree).is_none()` within the cited `:1396-1407`; `cmd/gui_schema.rs:1320` = `canonical_origin(&desc.tree).is_some()` within the cited `:1317-1325`, spec says `:1318-1325` — ~2-line merge-decay, code genuinely present). The `bundle.rs`/`gui_schema.rs` basenames are unambiguous (single file each in the toolkit). These are toolkit-master citations the plan-doc should re-grep with the `src/cmd/` prefix per the citation-lift discipline. Confirmed SOUND in rounds 1–2; does not block.

## Path forward

GREEN at the spec gate. Persist this review verbatim (done — this file). No fold required (Minor m-1 is a plan-doc-stage citation-hygiene note, not a spec blocker). **The lane proceeds to the implementation plan-doc, which runs its OWN R0 loop** — and when it lifts the `parse_descriptor.rs` / `cmd/bundle.rs` / `cmd/gui_schema.rs` citations it MUST re-grep them against current `origin/master` with the `src/cmd/` prefix and record the source SHA.
