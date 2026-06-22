# m\* constellation — adversarial funds-safety bug-hunt report

**Scope:** the five m-format constellation repos — `md-codec`/`md-cli`, `mk-codec`/`mk-cli`,
`ms-codec`/`ms-cli`, `mnemonic-toolkit`, `mnemonic-gui`. **EXCLUDES** the seedhammer fork
(`seedhammer*/`, `mnemonic-engrave/`, `shibboleth-wallet/`) and the stale `.claude/worktrees/`
agent-copy trees inside `descriptor-mnemonic`.

**Date:** 2026-06-20.
**Repo HEADs at hunt time:**
- `descriptor-mnemonic` (md) `54dd765` (main)
- `mnemonic-key` (mk) `1279ef9` (main)
- `mnemonic-secret` (ms) `6b28918` (master)
- `mnemonic-toolkit` `8967294d` (feature/bundle-md1-template-multisig)
- `mnemonic-gui` `5ee127c` (master)

**Method** (mirrors the prior `seedhammer-engrave-bughunt.md`, plus refute-by-default refinement):
parallel module-finders fan out across the constellation with a funds-safety lens → **one adversarial
verifier per candidate, prompted to REFUTE by default** (burden of proof on the finding; reachability
traced on the real code path; every load-bearing protocol/crypto/codec fact checked against the
**authoritative spec** — BIP-32/39/48/85, codex32/BIP-93 BCH, SLIP-39 RS1024/GF256, SLIP-132,
descriptor/miniscript — not plausibility) → **second independent skeptic** on every confirmed
critical/high → synthesis + dedup. Bug classes: **(A)** wrong address/descriptor, **(B)** silent
policy-collapse / fidelity loss, **(C)** accepting corrupted data, **(D)** secret leakage / missing
zeroize, **(E)** panic / DoS on valid input.

**Wave 1 result** (`wf_f9c888cc-3af`, 49 agents): 24 raw → **18 confirmed**, 2 downgraded, 4 refuted.
**Wave 2 result** (deep per-file + cross-cutting lenses, `wf_6a7b0def-da1`, 40 agents): 21 raw →
**14 confirmed**, 5 downgraded, 2 refuted. _Appended below as H7–H8 / M6–M7 / L8–L17._
**Wave 3 result** (gap coverage, `wf_ca63bb59-13a`, 36 agents): 18 raw → **15 confirmed** (1 folds
into H3), 3 downgraded-but-real, 0 refuted. _Appended below as H9–H11 / M8–M12 / L18–L23 + 3 downgraded._

**Final round** (`wf_67596cf3-391` + `wf_e7834750-724`, 22 agents): adversarial-decode / taproot /
multipath / timelock lenses (two passes — see note). 10 raw → **4 confirmed** (2 HIGH, 2 LOW),
5 downgraded-but-real, 3 refuted. _Appended below as H12–H13 / L24–L25._

**Wave B re-run** (`wf_e6536114-adb`, 16 agents — the corrected provenance/fingerprint/seed/checksum
lenses): 9 raw → **5 confirmed** (2 HIGH, 2 MEDIUM, 1 LOW), 1 downgraded, 2 refuted. _Appended below as
H14–H15 / M13–M14 / L26._ (The first final-round `args.wave="B"` had silently fallen back to Wave-A;
this run executed the intended Wave-B dimension set.)

**Differential-oracle wave** (`wf_8c03549a-c7c`, 13 agents — EMPIRICAL: real regtest addresses + Bitcoin
Core v27.0 `deriveaddresses` + independent BIP32/secp256k1 derivation): 22 findings → **6 confirmed
empirically, 2 downgraded, 0 refuted, 14 clean-negatives. ZERO new bugs** (validates static
thoroughness). It **escalated 3 findings to CRITICAL with proof** and **cleared several as metadata-only**.
_Appended below; the per-finding severities here SUPERSEDE the static ratings where they differ._

**REVISED confirmed total: 55 unique** — now **3 CRITICAL · 12 HIGH · 14 MEDIUM · 26 LOW** + 5
downgraded-but-real. The 3 CRITICALs (**H12, H1, H13**) are empirically proven wrong-address / false-verdict
funds-loss; they were rated HIGH by the static pass and the differential oracle escalated them.
_Hunt complete — 5 hunting waves + 1 differential-oracle wave, ~185 agents. Wave-3 minikey-persistence folds into H3._

---

## How to use this file

This is the **fix checklist**. Each confirmed item is a `- [ ]` checkbox; tick it (and cite the fixing
commit) when shipped. Keep the refuted/downgraded appendix so the same candidates are not re-hunted from
scratch next cycle.

**Severity tally (Wave 1):** HIGH ×6 · MEDIUM ×5 · LOW ×7.

---

## Confirmed — HIGH

### - [x] H1 · `verify-bundle` only compares the pubkey multiset, never the policy tree/threshold/script-type → false GREEN
<!-- FIXED cycle-1 (toolkit v0.61.0 @f9467cc5) — md1_xpub_match widened to compare tree + use_site_path + use_site_path_overrides (origins excluded per L14, name preserved → no --json change). Report-tick reconciliation in cycle-3 (the cycle-1 checkbox was shipped but never flipped). -->
<!-- See the detailed H1 (→ CRITICAL) entry below. -->
- **repo/class:** toolkit · **B-policy-collapse**
- **id:** `verify-bundle-md1-policy-structure-not-compared`
- **location(s):**
  - `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:2406-2489` (`emit_multisig_checks` md1 block)
  - `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:2521-2638` (`emit_md1_checks` single-sig)
- **bug:** For policy-form bundles the only md1 checks are `md1_decode`, `md1_wallet_policy`
  (`is_wallet_policy()` = merely "pubkeys non-empty"), and `md1_xpub_match` (a **sorted multiset**
  comparison that discards the per-slot index via `.map(|(_, b)| *b).sort()`). Nothing compares the
  supplied md1's `tree` (Tag = Multi/SortedMulti/Wpkh/Pkh/Thresh + threshold *k* + nested
  timelocks/hashlocks/branches), `path_decl`, `use_site_path`, or the full md1 string against the
  expected bundle. `mk1` cards carry no threshold/tag, so the per-cosigner checks cannot catch it
  either. (The keyless single-sig template path *does* do a full `expected.md1 == args.md1` compare at
  `:583`; the policy path does not.)
- **trigger:** `mnemonic verify-bundle --template wsh-sortedmulti --threshold 2 … --md1 <MD1>` where
  `<MD1>` carries the same two pubkeys but `k=1` (or `wsh-multi` vs `wsh-sortedmulti`, or `sh(wsh)` vs
  `wsh`). All checks pass → `result: ok`, exit 0.
- **consequence:** verify-bundle (whose job is "cross-check that a bundle actually reconstructs") issues
  a false GREEN for an md1 that reconstructs a **different** wallet — different spending threshold,
  script type/address, multi vs sortedmulti, or dropped timelocks/hashlocks. Direct funds-safety loss.
- **fix:** Add a policy-structure equality check to both md1 blocks — simplest: `expected.md1 ==
  supplied.md1` (new `md1_policy_match`); or compare decoded `Descriptor` policy-bearing fields (tree
  Tag + *k* + wrapper) and make `md1_xpub_match` index-aware so order-significant `multi`/`multi_a`
  key-order is pinned.
- **spec:** SPEC_mnemonic_toolkit_v0_5 §5.7 (multiset compare); SPEC_bundle_md1_template_multisig
  2026-06-20:111 (key order is consensus-significant for non-sorted shapes); descriptor/miniscript.

### - [x] H2 · GUI runner logs full unmasked argv (seed phrase / entropy / passphrase) to stderr at debug
<!-- FIXED cycle-3 (mnemonic-gui v0.45.0): 600b4dc — runner spawn-log now emits program=%argv[0] + argv_len only, never the cleartext argv. FOLLOWUP gui-runner-debug-logs-unmasked-secret-argv. -->
- **repo/class:** gui · **D-secret-leak**
- **id:** `gui-runner-debug-logs-unmasked-secret-argv`
- **location:** `mnemonic-gui/src/runner.rs:119`
- **bug:** `run_with_stdin` emits `debug!(target: "mnemonic_gui::runner", argv = ?argv, …)`, Debug-
  formatting the entire argv in cleartext. Secret values are assembled directly into argv
  (`invocation.rs:304/339/251`, `--slot @0.phrase=<24 words>`, `import-wallet --passphrase`). The whole
  v0.39.0 secret-mask architecture exists to keep these off cleartext surfaces, but this log was missed
  and gets the **raw** argv.
- **trigger:** `mnemonic-gui --debug` (or `RUST_LOG=…=debug`) + any secret-bearing invocation → phrase /
  entropy / passphrase verbatim on stderr, which users redirect to files / journald captures.
- **consequence:** master secret written cleartext to logs → irreversible fund loss; defeats the GUI's
  own masking + the CLIs' /proc argv-hardening.
- **fix:** Don't log raw argv — drop `argv = ?argv`, or log only `argv.len()`/`argv[0]`, or substitute
  the assembly-time secret mask before logging. Add a regression test asserting no secret appears in
  captured tracing output.
- **spec:** BIP-39 (mnemonic/entropy is the master secret).

### - [x] H3 · `convert --from minikey=<key>` leaks a Casascius mini private key (no mask / no confirm / plaintext persistence)
<!-- FIXED cycle-3 (mnemonic-gui v0.45.0): 8ac983f + 2cc9c9f — new node_type_is_argv_secret (wide SECRET_NODE_TYPES_ARGV = narrow + minikey) routes argv-mask/run-confirm/persist-redact; node-aware composite paste-warn wired into the value widget. No toolkit pin bump. FOLLOWUP gui-minikey-secret-not-masked-in-argv-preview. -->
- **repo/class:** gui · **D-secret-leak**
- **id:** `gui-minikey-secret-not-masked-in-argv-preview`
- **location(s):** `mnemonic-gui/src/secrets.rs:160` (`node_type_is_secret` backed by the **narrow**
  `SECRET_NODE_TYPES`); `mnemonic-gui/src/form/invocation.rs:457` (`emit_one` composite mask bit)
- **bug:** The GUI classifies composite-node secrecy via the toolkit's **persistence-redaction** set
  `SECRET_NODE_TYPES`, which deliberately **excludes `minikey`**. The toolkit ships a separate wider
  `SECRET_NODE_TYPES_ARGV` (= + `minikey`) whose doc-comment explicitly says argv/preview consumers
  must use *it* — the GUI never imports it. So `convert --from minikey=<value>`: (1) not masked in
  preview/confirm modal, (2) secret-run modal never fires, (3) "reveals secret" copy-warning suppressed
  and clipboard silently gets cleartext, (4) the schema's "per-row paste-warn fires" comment is false,
  (5) persisted to `state.json` in plaintext.
- **trigger:** GUI → convert → `--from` = minikey → paste a Casascius mini key → preview / confirm /
  copy buttons / `state.json` all expose it unmasked.
- **consequence:** a Bitcoin private key rendered cleartext on screen, copied without warning, never
  gated, and persisted to disk — shoulder-surf / clipboard-history / on-disk leak → irreversible loss.
- **fix:** Import `SECRET_NODE_TYPES_ARGV` and use it (not the narrow set) for the composite argv mask,
  `should_confirm_run`, and paste-warn/copy-reveal; **and for `redact_for_persistence`** so the private
  key is never written to `state.json`; add the drift-guard token + a unit test; correct the false schema
  comment at `schema/mnemonic.rs:918`.
- **spec:** `mnemonic_toolkit::secret_taxonomy` (`SECRET_NODE_TYPES` vs `SECRET_NODE_TYPES_ARGV`);
  `NodeType::is_argv_secret_bearing`.
- **Wave 3 re-confirmed** the on-disk leak independently (`w3-gui-minikey-persist-plaintext`): the
  plaintext private key is written to `~/.config/mnemonic-gui/state.json` by the autosave timer
  (`main.rs:362-366`) and on exit (`main.rs:1138`), and reloaded next launch — via
  `persistence.rs:94-99` (`redact_for_persistence`) keying on the narrow set (`secrets.rs:34,160-162`).
  This is the highest-severity facet of H3 — a spendable key at rest (default 0644), surviving restarts.

### - [x] H4 · `ms derive` panics (`unreachable!`) on a valid non-English (mnem) ms1 string
<!-- FIXED cycle-8 (ms-cli v0.9.0 @e80ea3b, tag ms-cli-v0.9.0, crates.io) — derive routes a Mnem ms1 through the shared `payload_entropy_and_language` helper using the WIRE language byte (CliLanguage::from_code), not the --language flag → no panic + CORRECT fingerprint (French 7d53dc37, not the wrong English 73c5da0a a naive --language patch produces). effective_lang threaded to all label sites. ms-codec NO-BUMP. -->
- **repo/class:** ms-cli · **E-panic-dos**
- **id:** `ms-derive-mnem-payload-panic`
- **location:** `mnemonic-secret/crates/ms-cli/src/cmd/derive.rs:185`
- **bug:** `match payload { Payload::Entr(b) => …, _ => unreachable!("ms-codec v0.1 decodes only
  Payload::Entr") }` — stale: ms-codec is v0.4.4 and `decode()` returns `Payload::Mnem { language,
  entropy }` for any ms1 made from a **non-English** BIP-39 phrase. Siblings `ms decode`/`ms combine`
  handle both; only `derive` falls into the `_` arm and panics. Secondary latent correctness bug:
  derive builds the mnemonic from the CLI `--language` (default english) not the **wire** language byte,
  so a naive panic-fix would compute a **wrong** fingerprint/xpub for non-English seeds (seed = PBKDF2
  over the language-specific sentence).
- **trigger:** `ms encode --phrase "<12 Japanese words>" --language japanese` → `ms derive <ms1>` →
  abort.
- **consequence:** user with a non-English ms1 card cannot recover their fingerprint/account xpub — tool
  aborts on valid input mid-recovery.
- **fix:** match **both** `Entr` and `Mnem`, and use the **wire** language byte
  (`CliLanguage::from_code`) to build the mnemonic; keep `_ => unreachable!` only as the
  `#[non_exhaustive]` future-variant guard.
- **spec:** BIP-39 (language → sentence → seed); ms-codec `decode.rs:78-95`.

### - [x] H5 · `ms verify` panics (`unreachable!`) on a valid non-English (mnem) ms1 string
<!-- FIXED cycle-8 (ms-cli v0.9.0 @e80ea3b) — verify routes the Ok((tag,payload)) arm through the same wire-language helper (both Err arms incl. exit-3 ReservedTagNotEmittedInV01 preserved verbatim); --language Option-ized (no spurious default note). -->
- **repo/class:** ms-cli · **E-panic-dos**
- **id:** `ms-verify-mnem-payload-panic`
- **location:** `mnemonic-secret/crates/ms-cli/src/cmd/verify.rs:64`
- **bug:** Same stale v0.1 assumption as H4 — `Ok((_, _)) => unreachable!("ms-codec v0.1 only decodes
  to Payload::Entr")`, but `Payload::Mnem` is reachable. Also the `--phrase` round-trip compare uses the
  CLI `--language` not the wire language.
- **trigger:** `ms encode --phrase "<non-English phrase>" --language <lang>` → `ms verify <ms1>` → abort.
- **consequence:** the safety-check command itself DoS's on a valid non-English backup card.
- **fix:** extract entropy from both variants; honor the wire language for the round-trip; keep
  `unreachable!` only as the genuine future-variant guard.
- **spec:** BIP-39; ms-codec `decode.rs:88-94`.

### - [x] H6 · md1 single-string encode/decode run outside the BCH(93,80,8) regular-code domain (no length cap)
<!-- FIXED cycle-4 (md-codec v0.38.0 @836faf8, crates.io; toolkit v0.62.1 @0b39709c pins it) — encode-side 80-DATA-symbol cap at wrap_payload (Error::PayloadTooLongForSingleString) + decode-side 93-CODEWORD caps (M4 ChunkSymbolCountOutOfRange + I1 StringSymbolCountOutOfRange). Out-of-domain md1 now fail-closed rejected on encode/repair/inspect/restore. FOLLOWUP encode-no-regular-code-length-cap. -->
- **repo/class:** md-codec · **C-corrupt-accept**
- **id:** `encode-no-regular-code-length-cap` (encode side) — companion to M4 (decode side)
- **location(s):** `md-codec/src/codex32.rs:67` (`wrap_payload`); `md-codec/src/encode.rs:136`
  (`encode_md1_string`); + decode side `md-codec/src/bch_decode.rs:284` (`chien_search`), `:403`
  (`decode_regular_errors`)
- **bug:** codex32's regular code is BCH(93,80,8) — only defined to 93 5-bit symbols. `wrap_payload`
  emits `data_symbols.len()+13` with **no** check that data ≤ 80; the polymod runs at any length so an
  over-length string is produced and even round-trip-verifies. Default `md encode` (no `--force-chunked`)
  uses this path, so any descriptor whose payload exceeds ~67 data symbols yields an out-of-code single
  string. Auto-chunking (`SINGLE_STRING_PAYLOAD_BIT_LIMIT=320`) exists to prevent this but is **opt-in**.
- **trigger:** `md encode <2-of-3 template> --key @0=<xpub> --key @1=<xpub> --key @2=<xpub>` → payload
  ~1587 bits → ~331 symbols (~3.5× the 93 cap); no error. Then `md repair <that md1>` runs the BCH
  decoder with `len > 93`.
- **consequence:** (1) emitted card is not transcribable/correctable under the regular code all steel
  tooling assumes — silently breaks the engravable-backup invariant; (2) on repair, Chien roots **alias**
  (a single error at pos 100 in a 331-symbol word yields aliased roots at 7/100/193/286, confirmed
  empirically) → for multi-error patterns a correction can be applied at the **wrong** aliased position
  while still zeroing the residue, passing re-verify and decoding to a wrong-but-valid descriptor
  (wrong addresses).
- **fix:** hard length guard in `wrap_payload`/`encode_md1_string` rejecting data > 80 symbols /
  codeword > 93 (`Error::PayloadTooLongForSingleString`) — encoders needing more MUST chunk. **And**
  (defense-in-depth, separate change) reject `data_with_checksum_len > 93` at the top of
  `decode_regular_errors`/`chien_search`.
- **spec:** BIP-93/codex32 regular code length 93; generator order 93 (test `beta_has_order_93_regular`).

---

## Confirmed — MEDIUM

<!-- FIXED cycle-13 (toolkit v0.66.0) — import-wallet decodes the real BIP-32 account from the single-sig origin into bundle.account (was hardcoded 0); export re-emits m/.../<account>'. Multisig (per-slot origins) unaffected. Whole-diff review GREEN. -->
### - [x] M1 · `export-wallet --from-import-json` drops the BIP-32 account for single-sig (origin forced to account 0)
- **repo/class:** toolkit · **B-policy-collapse**
- **id:** `from-import-json-singlesig-account-lost-wrong-origin-path`
- **location(s):** `export_wallet.rs:825`; `wallet_export/electrum.rs:111`; `wallet_export/coldcard.rs:201`;
  `wallet_export/sparrow.rs:256`; root cause `cmd/import_wallet.rs:1547` (`account: 0` literal)
- **bug:** `import-wallet --json` hardcodes `bundle.account: 0` for every wallet; the inverse path feeds
  `EmitInputs.account = 0`. Single-sig template-requiring emitters (coldcard/electrum/sparrow) rebuild
  the origin purely from `template.origin_path_str(network, account)` and ignore `resolved_slots[0].path`
  (which carries the real `m/84'/0'/5'`). So a wallet imported at account 5 is re-emitted as
  `m/84'/0'/0'`. Xpub is still the real account-5 key (addresses correct), but the declared origin no
  longer matches the key. Multisig unaffected.
- **trigger:** import a single-sig descriptor at non-zero account (`wpkh([fp/84'/0'/5']xpub…)`) `--json`
  then `export-wallet --from-import-json … --format electrum|coldcard|sparrow` → derivation `m/84'/0'/0'`.
- **consequence:** origin/derivation metadata no longer matches the xpub → a HW wallet/coordinator
  relying on the declared path (PSBT key-origin matching, account discovery) may fail to recognize/sign,
  stranding recovery. Fidelity/availability loss, not wrong-address.
- **fix:** decode the real account from the origin into `bundle.account` at import (`:1547`), or make the
  single-sig emitters honor `resolved_slots[0].origin_path_bare()` when present. Add a non-zero-account
  round-trip test.
- **spec:** BIP-32 key-origin; BIP-44/49/84 account at path index 2.

### - [x] M2 · Placeholder index 255 overflows `n` to 0 → BTreeMap-index panic (and silent wrong `n` when `@0` present)
<!-- FIXED cycle-9 (md-cli v0.9.0 @1a4b322) — placeholder index bounded ≤254 before the (N+1) as u8 cast + checked get; @255 panic → typed TemplateParse reject; @254 still accepts. -->
- **repo/class:** md-cli · **E-panic-dos**
- **id:** `placeholder-count-u8-overflow-panic`
- **location:** `descriptor-mnemonic/crates/md-cli/src/parse/template.rs:188-201`
- **bug:** `n = (by_i.keys().max() as usize + 1) as u8`; the lexer accepts `@0..=255`. Max index 255 →
  `256 as u8 == 0`: the `0..n` density check is skipped, and `by_i[&0]` panics ("no entry found for
  key") when `@0` absent (e.g. `wpkh(@255/*)`). If `@0` *is* present, no panic but `n=0` silently flows
  into the encoder (key count collapses to 0).
- **trigger:** `md encode --template 'wpkh(@255/*)'` (also `md address`/`md verify`) → panic at `by_i[&0]`.
- **consequence:** clean panic on valid-per-lexer input; or silent corruption of the encoded key count.
- **fix:** bound max index before the `as u8` cast (reject > 254 with a typed `CliError::TemplateParse`);
  replace `by_i[&0]` with checked `.get(&0).ok_or(...)`.
- **spec:** BIP-388 `@N` placeholders; md1 encodes key count `n` as a u8-derived field.

<!-- FIXED cycle-10 (md-codec v0.39.0 @8c73b4d, crates.io) — derive_address chain-gate widened to MAX alt-count over baseline + every per-@N override (None→1); provably fail-closed (per-key use_site_to_derivation_path still rejects). Whole-diff review GREEN. Toolkit pin-bump → 0.65.2 pending (after cycle-11b 0.65.1). -->
### - [x] M3 · `derive_address` chain gate reads baseline use-site only → valid change addresses unreachable on None-baseline + Some-override
- **repo/class:** md-codec · **B-policy-collapse**
- **id:** `derive-chain-gate-baseline-only-ignores-overrides`
- **location:** `descriptor-mnemonic/crates/md-codec/src/derive.rs:108-122`
- **bug:** The chain pre-flight bounds the allowable chain solely from `self.use_site_path.multipath`
  (the descriptor baseline), ignoring `self.tlv.use_site_path_overrides`. Per the legal D5(b) mix
  (`validate.rs:112-148`), a descriptor may have a `None` baseline (`bare /*`) plus a per-`@N` override
  carrying `Some(<0;1>)`. Faithful reconstruction yields two single descriptors (chain 0/1), but because
  baseline multipath is `None` the gate takes `else if chain != 0` and rejects every `chain != 0` with
  `ChainIndexOutOfRange { alt_count: 0 }`.
- **trigger:** 2-of-2 `wsh(multi(@0,@1))` with baseline `bare /*` + override `[(1, <0;1>)]`:
  `derive_address(chain=1)` errors though the faithful change address is real & fundable (verified
  end-to-end; chain-1 address unreachable).
- **consequence:** change-chain (and any non-zero chain) addresses for the overridden key become
  underivable by md-codec → funds received there are invisible to the restoring tool. Not wrong-address
  (chain 0 correct; it errors rather than substituting).
- **fix:** bound chain from the max alt-count across baseline **and** every override entry (treat `None`
  as 0); allow `chain in 0..alt_count`. Per-key resolution already does the right thing, so widening the
  gate is sufficient.
- **spec:** BIP-388 multipath substitution; `<a;b>` semantics; md-codec `validate.rs` D5(b).

### - [x] M4 · `decode_regular_errors`/`chien_search` accept `len > 93` → aliased error positions (decode-side companion to H6)
<!-- FIXED cycle-4 (md-codec v0.38.0 @836faf8, crates.io) — typed Error::ChunkSymbolCountOutOfRange at the decode_with_correction boundary + None-floors at decode_regular_errors/chien_search tops (reject len>93 before the unbounded loop). Paired-with-but-independent-of H6 (encode-cap vs decode-cap). FOLLOWUP chien-search-unbounded-length. -->
- **repo/class:** md-codec · **C-corrupt-accept**
- **id:** `chien-search-unbounded-length`
- **location(s):** `md-codec/src/bch_decode.rs:284` (`chien_search`), `:403` (`decode_regular_errors`)
- **bug:** `chien_search` iterates `0..data_with_checksum_len` evaluating `Lambda(beta^-d)` with no upper
  bound; BETA has order 93, so for `len > 93` positions `d` and `d+93` alias. Reachable via `md repair`
  on an over-93-symbol md1 (`decode_with_correction → parse_chunk_symbols` has no length cap;
  `md-cli repair.rs` feeds user strings straight in).
- **trigger:** `md repair <over-length md1>` (as produced by default `md encode` per H6, or hand-crafted).
- **consequence:** decoder sound only for `len ≤ 93`; for longer words aliased Chien roots can mis-locate
  errors; in multi-error patterns a wrong-position correction that still zeroes the residue passes
  re-verify → wrong descriptor on repair output. **Independent code change from H6** (a hand-crafted
  over-length md1 fed to `md repair` bypasses the encoder entirely).
- **fix:** reject `data_with_checksum_len > 93` at the top of `decode_regular_errors`/`chien_search`.
- **spec:** BIP-93/codex32 regular code length 93; generator order 93.

### - [x] M5 · Lexer/substitution regexes disagree when a multipath group is not last → use-site path ≠ parsed descriptor
<!-- FIXED cycle-9 (md-cli v0.9.0 @1a4b322, tag descriptor-mnemonic-md-cli-v0.9.0, crates.io) — a non-final <a;b> multipath (e.g. wpkh(@0/<2;3>/0'/*)) is now REJECTED (was: descriptor carried multipath over a single-path tree → WRONG address); fail-closed, an md1/UseSitePath representability limit (BIP-389 permits the form, md1 can't represent it). H13's hardened/malformed reject PRESERVED byte-identical (validator fires first; fused-test mutation-proven). md-codec NO-BUMP, no toolkit pin. -->
- **repo/class:** md-cli · **B-policy-collapse**
- **id:** `lexer-substitution-divergence-multipath-not-last`
- **location:** `descriptor-mnemonic/crates/md-cli/src/parse/template.rs:32-91`, `:357-381`
- **bug:** Lexer (line 40) and substitution (line 365) both capture the origin path as `((?:/\d+'?)*)`
  *before* the multipath, and the multipath as `/<…>` + optional wildcard. For `wpkh(@0/<2;3>/0'/*)` the
  lexer captures multipath `[2,3]` but substitution emits `wpkh(XPUB/0'/*)` — a single-path key with
  origin `/0'`, **no** multipath. Structure (from substituted string) and use-site (from lexer) are then
  stitched together despite describing different shapes. Also `h`-style hardened markers in the origin
  (`@0/48h/…`) aren't matched, emitting malformed `XPUBh/…`.
- **trigger:** `md encode --template 'wpkh(@0/<2;3>/0'\''/*)'` accepted; emitted md1 records use-site
  `<2;3>`+wildcard while the structural descriptor has no multipath and a dropped `/0'`.
- **consequence:** fidelity loss on malformed/exotic (non-BIP-389-canonical) templates — encoded backup's
  derivation ≠ what the user typed; fails silently instead of erroring. Lower likelihood (non-canonical
  ordering).
- **fix:** anchor the placeholder grammar so the multipath is only valid as the final pre-wildcard
  component; reject leftover path chars with a typed error; cross-validate lexer view vs substituted
  descriptor before emitting.
- **spec:** BIP-389 (multipath appears once, final derivation component before wildcard); BIP-388.

---

## Confirmed — LOW

### - [x] L1 · `build-descriptor` human view derives the first address with `--network` (default mainnet), no xpub-network cross-check
<!-- FIXED cycle-5 (toolkit v0.63.0 @bad8a3fb, tag mnemonic-toolkit-v0.63.0) — S-NET: build-descriptor now WARNs (stderr, exit 0 — deliverable is network-agnostic) on a --network/keys preview disagreement via `infer_descriptor_network_kind`; not a hard reject. -->
- **repo/class:** toolkit · **A-wrong-address** (display) · `build-descriptor-wrong-network-address-display`
- **location:** `crates/mnemonic-toolkit/src/cmd/build_descriptor.rs:476-485`
- **bug:** `emit_human()` uses `args.network.unwrap_or(Mainnet)` for `derive_receive_addresses`, never
  checking the descriptor xpubs' own SLIP-132/BIP-32 network bytes. A testnet-`tpub` descriptor with no
  `--network` prints `first receive address (Mainnet): bc1…`. The real deliverables (canonical descriptor
  / bip388 output) are network-agnostic & correct; only the display label/address HRP is wrong.
- **fix:** walk `vp.descriptor.for_each_key`, read each `Xpub::network`; infer display network from keys
  when `--network` omitted, or diagnose/refuse on disagreement.
- **spec:** SLIP-132/BIP-32 version bytes; BIP-173/350 HRP (`bc` vs `tb`).

### - [x] L2 · Electrum multisig network inferred from BIP-48 coin-type only, not cross-checked vs cosigner xpub prefix
<!-- FIXED cycle-5 (toolkit v0.63.0 @bad8a3fb) — S-NET: electrum-multisig parser now cross-checks each cosigner xpub's version vs the coin-type network via `assert_network_agrees` → NetworkMismatch (exit 2). -->
- **repo/class:** toolkit · **A-wrong-address** · `electrum-multisig-network-from-cointype-not-xpub-prefix`
- **location:** `crates/mnemonic-toolkit/src/wallet_import/electrum.rs:698-718`
- **bug:** `build_multisig_descriptor` decides network solely from the BIP-48 coin-type child; the
  SLIP-132 xpub prefix is used only for `variant_class` and normalized away without asserting agreement.
  (The **single-sig** path *does* derive network from the xpub prefix — the two paths disagree.) A
  testnet-`Vpub` set with coin-type `0'` is imported as `network = Bitcoin` while the descriptor derives
  testnet addresses.
- **fix:** assert every cosigner's neutralized xpub network matches the coin-type-derived network; else
  `ImportWalletParse` (mirror the single-sig `network_from_xpub_neutral` cross-check).
- **spec:** SLIP-132; BIP-48 coin-type at path index 1.

### - [x] L3 · Coldcard single-sig `account` silently truncated via `as u32`; legacy-xpub fallback bakes it into the origin path
<!-- FIXED cycle-5 (toolkit v0.63.0 @bad8a3fb) — ride-along (firewalled from the network helper): coldcard single-sig account now `u32::try_from` → REJECTS (ImportWalletParse, exit 2) on >u32::MAX instead of silently truncating. RED drives the legacy top-level-xpub fixture. -->
- **repo/class:** toolkit · **A-wrong-address** · `coldcard-singlesig-account-u32-truncation-legacy-fallback`
- **location:** `crates/mnemonic-toolkit/src/wallet_import/coldcard.rs:237-241`
- **bug:** `account = obj["account"].as_u64().map(|n| n as u32).unwrap_or(0)` wraps a `>u32::MAX` JSON
  value silently; in the legacy top-level-xpub fallback it's interpolated into
  `format!("m/{purpose}'/{coin}'/{raw_account}'")`, producing a wrong origin annotation.
- **fix:** reject (or error-saturate) an account that doesn't fit `u32` instead of truncating; typed
  `ImportWalletParse` naming the field.
- **spec:** BIP-32 child indices are u32.

### - [x] L4 · `md repair` always emits the "keyless template (no keys)" advisory even when the md1 carries watch-only pubkeys
<!-- FIXED cycle-9 (md-cli v0.9.0 @1a4b322) — is_wallet_policy()-gated label: keyed md1 → WatchOnly, keyless stays Template (repair.rs; _descriptor→descriptor binding fixed for clippy -D warnings). -->
- **repo/class:** md-cli · **D-secret-leak** (privacy) · `repair-advisory-mislabels-watch-only-as-keyless-template`
- **location:** `descriptor-mnemonic/crates/md-cli/src/cmd/repair.rs:156-159`
- **bug:** `md repair` unconditionally emits `OutputClass::Template` ("stdout is a keyless descriptor
  template"), but it operates on arbitrary md1 including wallet-policy md1 whose Pubkeys TLV carries
  65-byte pubkey entries (and which auto-chunk, a natural repair input). `md address` correctly emits
  `WatchOnly`; only `repair` mislabels key-bearing output as keyless.
- **fix:** choose the advisory from `descriptor.is_wallet_policy()` — `WatchOnly` when true, `Template`
  when false.
- **spec:** output-class advisory contract; `is_wallet_policy()`.

### - [x] L5 · `ms-cli CliError::Codex32` wraps the raw `codex32::Error`, bypassing ms-codec's sanitizing Debug (latent secret leak)
<!-- FIXED cycle-8 (ms-cli v0.9.0 @e80ea3b) — hand-rolled CliError Debug delegates to sanitized kind()+message() so codex32::Error's `string` (which echoes the secret ms1) doesn't leak; mutation-proven (forcing the echo turns the L5 test RED). Latent (no production {:?} on CliError), defensively closed. -->
- **repo/class:** ms-codec/ms-cli · **D-secret-leak** (latent) · `ms-cli-clierror-codex32-bypasses-sanitized-debug`
- **location:** `mnemonic-secret/crates/ms-cli/src/error.rs:20`
- **bug:** `#[derive(Debug)] enum CliError` carries `Codex32(codex32::Error)` directly, not through
  ms-codec's hand-rolled sanitizing Debug. `codex32::Error::InvalidChecksum { string }` carries the full
  secret ms1 string; any future `{:?}`/`unwrap`/`expect`/`panic` on this variant leaks it. Production
  path uses `Display`/`message()` (safe today) — latent.
- **fix:** hand-roll `CliError`'s Debug to delegate to sanitized Display, or stop carrying the raw
  `codex32::Error`. Add a no-echo test.
- **spec:** BIP-93/codex32 (ms1 data-part is the secret); ms-codec sanitized-Debug contract.

<!-- FIXED cycle-10 (md-codec v0.39.0 @8c73b4d, crates.io) — added the existing DivergentPathCountMismatch len==n guard (mirroring expand_per_at_n) before the reorder index; n_keys bound before the &mut borrow. -->
### - [x] L6 · `canonicalize_placeholder_indices` indexes Divergent `path_decl` without a `len == n` guard (library-reachable panic)
- **repo/class:** md-codec · **E-panic-dos** · `canonicalize-divergent-path-decl-unchecked-len-panic`
- **location:** `descriptor-mnemonic/crates/md-codec/src/canonicalize.rs:206-219`
- **bug:** Non-identity permutation branch does `old_paths[inverse[new_idx]]` for `new_idx in 0..n` with
  no `old_paths.len()==n` check (unlike `expand_per_at_n` which returns
  `Error::DivergentPathCountMismatch`). Not reachable from decoded wire (`PathDecl::read` always reads
  exactly `n`), but reachable by a library consumer who constructs a `Descriptor` with a short Divergent
  vector + non-canonical tree, then calls `encode_payload`/`compute_wallet_policy_id`.
- **fix:** add the same length guard before the reorder.
- **spec:** md1 spec §3.4 origin-path-decl divergent mode.

### - [x] L7 · `md repair --help` epilog claims non-chunked single-string md1 are rejected, but they are now repaired (doc drift)
<!-- FIXED cycle-9 (md-cli v0.9.0 @1a4b322) — stale "rejected with a wire-format error" prose removed from main.rs:241; the toolkit-manual MIRROR (docs/manual/src/40-cli-reference/42-md.md) corrected in lockstep (paired-PR docs discipline — not lint-gated) in THIS commit (non-chunked repaired since md-codec v0.35.0). -->
- **repo/class:** md-cli · **other** · `repair-help-epilog-stale-rejects-nonchunked-claim`
- **location:** `descriptor-mnemonic/crates/md-cli/src/main.rs:241`
- **bug:** Epilog says non-chunked single md1 are "rejected with a wire-format error", but
  `decode_with_correction` gained v0.35.0 single-string auto-dispatch (`chunk.rs:599-617`) and now
  repairs them. UX/doc drift; tracked under FOLLOWUP `md-codec-decode-with-correction-supports-non-chunked-md1`.
- **fix:** update the epilog and reconcile the FOLLOWUP status.

---

## Appendix — downgraded / refuted (do not re-hunt from scratch)

| status | repo · dim | finding | location | why |
|---|---|---|---|---|
| downgraded→low | toolkit · verify-bundle | `md1_xpub_match` sorted-multiset drops slot→key binding; for unsorted `multi`/`multi_a` cannot alone distinguish a key-order permutation | `verify_bundle.rs:2428-2444` | Real, but **subsumed by H1's fix** (make the match index-aware). Order is consensus-significant for non-sorted shapes. |
| downgraded→low | toolkit · xcut-secret | `DerivedAccount` `#[derive(Debug)]` would print raw entropy/xpriv **if** ever Debug-formatted | `derive.rs:22` | `entropy` is `Zeroizing<Vec<u8>>` (its Debug redacts) and there's no live `{:?}` site; latent only. |
| refuted | md-cli · template-parse | multipath alt ≥ 2³¹ accepted & encoded as invalid unhardened child index | `template.rs:62-73,220-233` | Load-bearing claim disproven on the real path — not a bug. |
| refuted | ms-codec · bch | `decode_with_correction` runs BCH on arbitrary-length data before a length gate | `decode.rs:237-256` | Divergence real but **not exploitable** (BCH correction self-limits; re-verify fail-safes). Defense-in-depth nicety only. |
| refuted | toolkit · bundle/perm | `ChainScope::Change` collapses to receive chain (bit 0) in `AddressRange::flatten` | `permutation_search.rs:232-240` | **Both** load-bearing claims about the live decode path are FALSE (`chain` is not decoded as `address_index & 1`). Not a bug. |
| refuted | toolkit · restore | restore `--md1` of a taproot `multi_a` card ignores wire tap-leaf key order → wrong descriptor | `cmd/restore.rs:1215` → `wallet_export/pipeline.rs:113-156` | Premise false: a faithful md1 card cannot carry a non-identity `multi_a` leaf order divergent from the slot table. Not a bug. |

---

## Cross-cutting themes (Wave 1)

1. **Stale-version assumptions baked into match arms / comments.** The two highest-severity panics (H4,
   H5) both carry `unreachable!("ms-codec v0.1 …")` while the codec is v0.4.4 and now returns
   `Payload::Mnem` for every non-English seed. Same flavor in the `md repair` help epilog (L7). Decode-
   result enums and CLI text drift out of lockstep with the codecs they consume.
2. **Network/account metadata decided from one source without cross-checking the authoritative one.**
   `build-descriptor` (L1), Electrum multisig (L2), Coldcard single-sig (L3), and `from-import-json`
   single-sig (M1) all pick network/account from a secondary signal (CLI flag, BIP-48 coin-type, JSON
   field) and never assert agreement with the SLIP-132 xpub version bytes or the descriptor origin path.
   Addresses stay correct (derived from key bytes); origin/network **fidelity** degrades.
3. **Secret-class set confusion in the GUI.** The persistence-redaction set (narrow) is reused for
   argv/preview/clipboard classification → leaks `minikey` (H3); the runner debug log was never wired to
   the mask at all (H2). Both defeat a deliberate masking architecture by missing one consumer.
4. **Partial verification / partial validation as false assurance.** `verify-bundle` compares a
   necessary-but-insufficient property (pubkey multiset) and reports success (H1); the BCH decoder runs
   outside its proven-sound domain (H6/M4). The recurring failure mode is checking too little and
   reporting OK.
5. **Unguarded arithmetic / indexing on values that are valid-per-parser.** Placeholder index 255 wraps
   `n` to 0 and panics (M2); Divergent `path_decl` indexed without a `len==n` guard (L6); account
   `as u32` truncation (L3). Each is a missing bound between an over-permissive lexer/parser and a
   downstream consumer that assumes the invariant already holds.

---

# Wave 2 — deep per-file + cross-cutting-lens pass

**Tally:** HIGH ×2 · MEDIUM ×2 · LOW ×10. None re-tread Wave 1 ground (seeded with Wave 1 verdicts).

## Confirmed — HIGH (Wave 2)

### - [x] H7 · Prefix-form `[fp/path]@N` origin annotation silently ignored → origin path dropped + fingerprint guard bypassed
<!-- FIXED cycle-2 (toolkit v0.62.0): 36095b88 — lex_placeholders ACCEPTs the BIP-380 prefix form; all-named capture groups keep cycle-1's H13 reject intact. FOLLOWUP descriptor-prefix-form-origin-annotation-ignored. -->
- **repo/class:** toolkit · **B-policy-collapse** · `w2-tk-synth-parse-01`
- **location(s):** `parse_descriptor.rs:60-140` (`lex_placeholders` regex `:69-71`); `parse_descriptor.rs:319`
  (`substitute_synthetic` strip); `cmd/bundle.rs:1369-1370,1569-1626` (bypassed fp cross-check);
  `cmd/verify_bundle.rs` (shared lexer)
- **bug:** `lex_placeholders` matches only the **suffix** form `@N[fp/path]`. The manual documents the
  BIP-380 **prefix** form `[fp/path]@N` as the canonical per-`@N` override ("takes precedence"). For the
  prefix form the lexer captures `@N` with `fingerprint_anno=None`, `origin_path_anno=None`, so (1) the
  origin path is silently **dropped** (slot xpub built at the master/default path), and (2) the per-`@N`
  master-fingerprint cross-check (a funds-safety guard) is skipped. `substitute_synthetic` only strips
  the suffix bracket, so the prefix text never reaches md1/mk1. Affects bundle **and** verify-bundle.
- **trigger:** `bundle --slot @0.phrase=<seed> --descriptor "wpkh([deadbeef/84'/0'/0']@0/<0;1>/*)"` exits
  0 with `origin_path:null`; the prefix form with a *wrong* fp yields md1/mk1 byte-identical to a bare
  `@0` — both fields discarded. (The suffix form correctly errors on fp mismatch.)
- **consequence:** a user following the manual's `[fp/path]@N` override gets a backup derived at the
  **wrong path** (silent default-path fallback) — wallet watches a different address set; funds at
  unexpected derivations or an apparently-empty restore. fp sanity guard bypassed; verify-bundle can't
  catch it (shared lexer).
- **fix:** accept **both** annotation positions in `lex_placeholders` (capture a leading
  `[<8hex>(/path)?]` before `@N`, populate the anno fields identically; mirror the strip), **or** reject
  the prefix form with an explicit `DescriptorParse` error and fix the manual. Add prefix-vs-suffix
  fp-mismatch + identical-md1/mk1 round-trip tests.
- **spec:** BIP-380 key-origin (`[fingerprint/path]KEY` — origin is a **prefix**); manual
  §41-mnemonic "non-canonical" point 2.

### - [x] H8 · `--md1-form=template` drops the BIP-39 wordlist language → non-English seed re-emits as English → wrong master seed
<!-- FIXED cycle-2 (toolkit v0.62.0): 53787cbb — run_language threaded into synthesize_template_descriptor; template ms1 emit uses unwrap_or(run_language). FOLLOWUP template-form-md1-drops-bip39-wordlist-language. -->
- **repo/class:** toolkit · **B-policy-collapse** · `w2-tk-bundle-template-emit-01` · _highest-impact funds-loss this hunt_
- **location(s):** `synthesize.rs:1265` (template ms1 emit hardcodes English); `synthesize.rs:486-488`
  (call drops `run_language`); `synthesize.rs:1158-1162` (fn sig lacks language param); `synthesize.rs:547`
  (keyed path correctly uses `run_language`)
- **bug:** the keyed path emits ms1 with `emit_lang = c.language.unwrap_or(run_language)`; the keyless
  **template** path (`synthesize_template_descriptor`) is invoked **without** `run_language` and hardcodes
  `unwrap_or(English)`. For any non-English seed under `--md1-form=template`, the ms1 card decodes as
  "english (default)" instead of carrying the true wordlist. Because this toolkit derives the master seed
  from the regenerated **phrase** via PBKDF2 (not raw entropy), re-encoding under a different wordlist
  yields a **different master seed**. Also affects the multisig template path.
- **trigger:** `bundle --template bip84 --language spanish --slot @0.phrase='ábaco … abierto'
  --md1-form=template` → `ms decode` of the emitted ms1 reports "english (default)" and reconstructs the
  English phrase. Master fp diverges (all-zero entropy: `1b6aef92` spanish vs `73c5da0a` english).
- **consequence:** silent wordlist-language loss in the ms1 backup → faithful restore reconstructs a
  different phrase → different seed → wrong keys/addresses; funds engraved under the template descriptor
  are not recoverable from the card.
- **fix:** thread `run_language` into `synthesize_template_descriptor` and replace the hardcoded English
  fallback at `:1265` with `c.language.unwrap_or(run_language)`. Add a non-English template-form ms1
  round-trip test.
- **spec:** SPEC §5.8 ms1 emission + ms-mnem wire-language-preservation invariant (keyed path enforces it,
  template path regressed it); BIP-39 (seed = PBKDF2 over NFKD phrase → wordlist is seed-load-bearing).

## Confirmed — MEDIUM (Wave 2)

### - [x] M6 · `combine_shares()` silently reconstructs a WRONG secret from an inconsistent same-id share set
<!-- FIXED cycle-4 (ms-codec v0.5.0 @44ac71f, crates.io; toolkit v0.62.1 @0b39709c pins it) — combine_shares now truncates to k, recovers from the first k shares, and verifies every EXTRA supplied share lies on that polynomial via interpolate_at(k_set, idx) → Error::InconsistentShareSet. Beyond-BIP-93 defense-in-depth; valid exactly-k + all-consistent combines bit-identical. ms-cli + toolkit exit-2 arms (silent lockstep, explicit). Exactly-k mixed pair intrinsically undetectable (out of scope per BIP-93). FOLLOWUP w2-ms-slip39-gf256-1. -->
- **repo/class:** ms-codec · **C-corrupt-accept** · `w2-ms-slip39-gf256-1`
- **location(s):** `ms-codec/src/shares.rs:186-270` (`combine_shares`; `interpolate_at` at `:263`);
  `ms-codec/src/envelope.rs:192-220` (`dispatch_payload` — only probabilistic backstop)
- **bug:** `combine_shares` validates only per-string BCH, no-share-at-`s`, count ≥ k, and distinct
  indices, then Lagrange-interpolates over **all** supplied shares. codex32's `interpolate_at` checks only
  that shares agree on hrp/id/threshold/length — **not** that the points lie on one degree-(k-1)
  polynomial — and `combine_shares` neither truncates to exactly k nor verifies extras. So any shares
  sharing the same 4-char (20-bit) id with distinct indices are interpolated even if from **different
  secrets**. codex32/BIP-93 K-of-N has **no digest share** (unlike SLIP-39), so nothing else catches it;
  `dispatch_payload`'s prefix-byte check is only a ~255/256 filter.
- **trigger:** split secret A and secret B as 2-of-3 with the **same** id/threshold/length;
  `combine_shares([A_share, B_share])` returns B's secret with no error. Same-id collisions arise from the
  20-bit id space (birthday-bound at scale) or an attacker crafting a valid-checksum share. Reachable via
  `ms combine`.
- **consequence:** a user believing they recover secret A silently obtains an unrelated secret (or
  ~1/256 garbage that still parses), derives keys for the wrong wallet → irreversible loss. Or an attacker
  substitutes one same-id share to steer reconstruction.
- **fix:** interpolate over exactly k shares, then verify **every** remaining supplied share lies on the
  reconstructed polynomial (`interpolate_at(set, idx) == supplied`); reject with a new
  `Error::InconsistentShareSet`. Document that codex32 K-of-N carries no digest share.
- **spec:** BIP-93/codex32 K-of-N Shamir over GF(32) (plain Lagrange, no integrity share); codex32-0.1.0
  `interpolate_at` checks only hrp/id/threshold/length.

<!-- FIXED cycle-13 (toolkit v0.66.0) — bundle --json derives the real threshold K via extract_multisig_threshold (was cosigner count N) in descriptor/--import-json mode; path_family sub-claim already-fixed pre-cycle. --json wire-value (GUI paired-PR). Whole-diff review GREEN. -->
### - [x] M7 · `bundle … --json` reports `multisig.threshold = N` (cosigner count) instead of the real K
- **repo/class:** toolkit · **B-policy-collapse** · `w2-tk-bundle-template-emit-02`
- **location(s):** `cmd/bundle.rs:915,922` (`threshold = args.threshold.unwrap_or(n)`); `:924`
  (`path_family` hardcoded `bip87`); `:1263` (card path correctly uses `extract_multisig_threshold`)
- **bug:** in descriptor / `--import-json` / concrete-descriptor mode `args.threshold` is always `None`,
  so the `--json` emitter falls back to the cosigner **count** `n` rather than reading the real K from the
  descriptor tree. The engraving **card** path does it right (`extract_multisig_threshold`); the JSON path
  never calls it. md1 wire is correct (encodes K); only JSON metadata is wrong. `multisig.path_family` is
  likewise hardcoded to `bip87` even for BIP-48.
- **trigger:** `bundle --descriptor 'wsh(sortedmulti(2,…))' … --json` → `.multisig.threshold` = 3 for a
  2-of-3 (card says "2 of 3").
- **consequence:** a consumer reading SPEC §5.3 `multisig.threshold` gets N-of-N → wrong spending policy
  (and wrong descriptor for unsorted multi). Consumers using the embedded descriptor/md1 unaffected.
- **fix:** derive threshold in the JSON branch via `extract_multisig_threshold(&tree)` as the card does;
  derive `path_family` from the real origin paths (or omit in descriptor mode).
- **spec:** SPEC §5.3 (JSON `multisig{threshold,cosigner_count,path_family}`); §5.5 (card already correct).

## Confirmed — LOW (Wave 2)

<!-- FIXED cycle-13 (toolkit v0.66.0) — all-own multisig-template completion substitutes network.coin_type() into the synthesized own origin's coin component; testnet/signet/regtest all-own now restore (was hardcoded 0' → silent NO-MATCH). Fail-safe (never wrong address); md-codec NO-BUMP. Whole-diff review GREEN. -->
### - [x] L8 · Multisig-template completion hardcodes mainnet coin-type 0' → testnet/signet/regtest all-own wallets unrestorable
- **repo/class:** toolkit · **A-wrong-address** · `w2-tk-restore-deep-01`
- **location(s):** `cmd/restore.rs:1342-1346` (consumed `:1377-1382,1411-1416`);
  `md-codec/src/canonical_origin.rs:61,70` (hardcoded coin 0'); `synthesize.rs:1195-1197`
- **bug:** all-own multisig-template completion (no `--cosigner`/`--origin`) builds the own origin from
  `canonical_origin(&d.tree)`, which hardcodes coin 0' (`m/48'/0'/acct'/{2,1}'`); only the account
  component is substituted. But bundles emit cosigner origins via `network.coin_type()` (=1 for
  non-mainnet). So `restore --network testnet` derives every own key at `m/48'/0'/…` → never matches →
  silent NO-MATCH. The in-code comment at `:1379` even asserts `m/48'/coin'/…`.
- **fix:** substitute `network.coin_type()` into the coin component of the canonical fallback (mirror the
  bundle emitter). Add a testnet all-own round-trip test. Fail-safe today (NO-MATCH, no wrong address).
- **spec:** BIP-48 coin-type (mainnet 0' / testnet 1').

<!-- FIXED cycle-13 (toolkit v0.66.0) — hoisted the has_hardened_use_site + unrestorable-taproot-override refusals into the shared complete_multisig_template core (covers restore + verify-bundle uniformly); precise early refusal instead of an opaque downstream error. Whole-diff review GREEN. -->
### - [x] L9 · `run_multisig_template_completion` omits the hardened-use-site / taproot-override refusals `run_multisig` applies
- **repo/class:** toolkit · **B-policy-collapse** · `w2-tk-restore-deep-02`
- **location(s):** `cmd/restore.rs:1159-1585` (no guards) vs `:2581-2594,2639-2646` (`run_multisig`
  guards); `synthesize.rs:317,320,323` (use_site preserved verbatim)
- **bug:** `run_multisig` refuses `has_hardened_use_site` and an unrestorable `taproot_override_card`
  before reconstruction; the parallel keyless-template path carries neither. Low reachability on
  legitimate cards (named templates are non-hardened, taproot never reaches this path) and downstream
  watch-only derive fail-safes to NO-MATCH — so today it's a missing **early actionable refusal** (opaque
  error instead of precise message) + path inconsistency, latent if a future bundle form emits a hardened
  canonical multisig template.
- **fix:** apply the same `has_hardened_use_site` / `taproot_override_card` guards at the top of the
  completion path; consider refusing templates carrying `origin_path_overrides`.

### - [x] L10 · BSMS network inferred from coin-type only, no xpub-version cross-check
<!-- FIXED cycle-5 (toolkit v0.63.0 @bad8a3fb) — S-NET: BSMS parser cross-checks each xpub version vs coin-type network via `assert_network_agrees` → NetworkMismatch (exit 2). -->
- **repo/class:** toolkit · **A-wrong-address** · `w2-tk-msimport-03`
- **location:** `wallet_import/bsms.rs:386-413` (`network_from_origins`/`coin_type_from_path`),
  `:249,297` (first-address)
- **bug:** distinct BSMS-parser instance of the Wave-1 Electrum coin-type pattern — network from BIP-48
  coin-type child, never cross-checked against cosigner xpub version bytes. Inconsistent/edited blob →
  wrong-network address. Fix: assert xpub network consistent across cosigners and with coin-type; else
  `ImportWalletParse`. · **spec:** BIP-32 xpub `0x0488B21E` / tpub `0x043587CF`.

### - [x] L11 · `convert --from wif --to xpub` uses `--network` (default mainnet), ignoring the WIF's embedded network
<!-- FIXED cycle-5 (toolkit v0.63.0 @bad8a3fb) — S-NET: convert wif→xpub now extracts the WIF's OWN NetworkKind (pk.network) and cross-checks vs --network via `assert_network_agrees` → NetworkMismatch (exit 2); escape is --network testnet. -->
- **repo/class:** toolkit · **A-wrong-address** · `w2-xcut-02`
- **location:** `cmd/convert.rs:1480-1491` (Wif arm), `:1217` (`network = args.network.unwrap_or(Mainnet)`)
- **bug:** the sentinel xpub's network is set from `--network` (default mainnet), discarding the parsed
  `pk.network`; a testnet WIF → `xpub…` (mainnet) instead of `tpub…`. Sentinel is flagged non-derivable;
  blast radius limited. Fix: derive network from `pk.network`, or error on disagreement.

<!-- FIXED cycle-11a (mnemonic-gui v0.46.0 @1999323, PR #14) — the three single-key canonicity regexes now accept the suffix-origin form @N[fp/path]; benign over-acceptance of double-origin (v0.60.0 accepts / v0.62.0+ refuses-at-parse); is_match only (no capture renumber). Whole-diff review GREEN. -->
### - [x] L12 · GUI canonicity regex misclassifies `@N[fp/path]` descriptors → lifts the `--account` pin
- **repo/class:** gui · **A-wrong-address** · `w2-gui-cond-02`
- **location:** `mnemonic-gui/src/form/conditional.rs:99-126,136-141,238-245`
- **bug:** the GUI's textual `classify_descriptor_canonicity` places the origin bracket **before** `@N`
  (`[fp]@N`), but the toolkit's grammar writes it **after** (`@N[fp/path]`). All four toolkit-canonical
  `@N[fp/path]` forms classify as NonCanonical → GUI lifts the `--account` pin → `--account N` reaches the
  toolkit, which classifies the same descriptor Canonical and hard-errors `DESCRIPTOR_WITH_NONZERO_ACCOUNT`.
  Confusing error, not silent wrong address. Fix: accept `@N[fp/path]` in the regex, or call
  `gui-schema --classify-descriptor` (the toolkit's own classifier).

<!-- FIXED cycle-11a (mnemonic-gui v0.46.0 @1999323, PR #14) — split CONVERT_FROM_NODES (14, seedqr@1) for --from vs CONVERT_TO_NODES (13, seedqr-free) for --to, matching the toolkit NodeType::as_str / --to PossibleValuesParser asymmetry; zero schema_mirror drift. -->
### - [x] L13 · GUI `convert --from` dropdown missing the valid `seedqr` node type
- **repo/class:** gui · **B-policy-collapse** (coverage) · `w2-gui-cond-01`
- **location:** `mnemonic-gui/src/schema/mnemonic.rs:130-144` (`NODE_TYPES`), false "exact mirror" comment
  `:123-129`; authoritative `cmd/convert.rs:54-72` (`seedqr` at index 1)
- **bug:** `NODE_TYPES` (shared by `--from`/`--to`) omits `seedqr`, so `convert --from seedqr=<digits>` is
  unreachable from the GUI though the CLI supports it. The schema-mirror gate checks flag **names**, not
  dropdown **values**, so it didn't catch the drift. Fix: add `seedqr` at index 1; use a `--to`-restricted
  set (toolkit rejects `--to seedqr`); extend the conditional-drift test to dropdown values.

<!-- FIXED cycle-10 (md-codec v0.39.0 @8c73b4d, crates.io) — compute_wallet_policy_id canonical-fills an elided (empty-components) in-memory origin via canonical_origin(&d.tree) so elided hashes identically to explicit; decoded wires unaffected; in-memory-only (not on the md1 wire) → MINOR. -->
### - [x] L14 · `WalletPolicyId` is NOT stable across origin-path elision (contradicts its doc-invariant)
- **repo/class:** md-codec · **B-policy-collapse** · `w2-md-canon-1`
- **location:** `md-codec/src/identity.rs:106-113` (false doc), `:172-240` (`compute_wallet_policy_id`),
  `canonicalize.rs:420-474` (`canonical_origin` used only as error-gate)
- **bug:** the doc claims the id is "stable across origin elision", but `compute_wallet_policy_id` hashes
  the per-`@N` path verbatim and never canonical-fills an elided empty path. `wpkh(@0)` with empty
  `path_decl` vs explicit `m/84'/0'/0'` → **different** ids for the same wallet → false non-matches if a
  consumer dedups/matches engravings by id (e.g. mk1 cosigner stub binding). Fix: make the doc honest
  (id is origin-significant) **or** implement canonical-fill. · **spec:** spec v0.13 §5.3.

<!-- FIXED cycle-10 (md-codec v0.39.0 @8c73b4d, crates.io) — compute_wallet_descriptor_template_id now canonicalizes placeholder ordering on a clone first (mirrors policy-id); identity fast-path leaves canonical inputs unchanged; in-memory-only. -->
### - [x] L15 · `compute_wallet_descriptor_template_id` doesn't canonicalize placeholder ordering (asymmetry vs policy-id)
- **repo/class:** md-codec · **B-policy-collapse** · `w2-md-canon-3`
- **location:** `md-codec/src/identity.rs:71-104` vs `:172-177`
- **bug:** the WDT-id hashes raw placeholder indices with no canonicalization, while
  `compute_wallet_policy_id` canonicalizes a clone first. `wsh(multi(2,@1,@0))` vs `(@0,@1)` → different
  WDT-ids. Not CLI-reachable (decoder gates inputs), but a library asymmetry. Fix: canonicalize a clone
  before hashing, or document the precondition. · **spec:** spec §8.1 / §6.1.

### - [ ] L16 · LP4-ext varint cannot encode BIP-32 child numbers ≥ 2²⁹ (valid descriptors fail to encode)
- **repo/class:** md-codec · **E-panic-dos** (graceful) · `w2-md-tlv-bitstream-varint-02`
- **location:** `md-codec/src/varint.rs:15-42` (cap at `l_high>15`); `origin_path.rs:23,28-31`;
  `use_site_path.rs:28-32`
- **bug:** `write_varint` caps at 29 bits (`VarintOverflow` for value ≥ 2²⁹), but `PathComponent.value`
  /`Alternative.value` carry BIP-32 child numbers up to 2³¹−1 (doc says "u31 effective range"). A child
  index in [2²⁹, 2³¹−1] makes encoding an otherwise-valid descriptor fail. Graceful error (no panic/wrong
  address) but a fidelity/availability gap. Fix: extend the varint to the full 31-bit range, or
  enforce/document the ceiling at the parse boundary. · **spec:** BIP-32 child numbers 0..2³¹−1.

<!-- FIXED cycle-10 (md-codec v0.39.0 @8c73b4d, crates.io) — de-vacuified: the test now builds a genuinely ELIDED empty path_decl wpkh(@0) and asserts it hashes identically to the explicit m/84'/0'/0' form (the RED→GREEN gate for L14; confirmed RED by temporarily reverting L14). -->
### - [x] L17 · Test `walletpolicyid_stable_across_origin_elision` is vacuous (masks L14)
- **repo/class:** md-codec · **other** · `w2-md-canon-2`
- **location:** `md-codec/src/identity.rs:571-588` (+ fixture `:385-419`)
- **bug:** both operands carry an explicit `Shared(BIP84)` `path_decl`; the "override" is byte-identical
  to the baseline, so the test never exercises the elided empty-path form it claims to — it's why L14's
  false invariant passes CI. Fix: rewrite to build a genuinely elided `path_decl` and assert the real
  (currently differing) behavior. · **spec:** spec v0.13 §5.3.

## Appendix — Wave 2 downgraded / refuted

| status | repo · dim | finding | location | why |
|---|---|---|---|---|
| downgraded→low | md-codec · varint | `read_varint` accepts non-canonical LP4-ext encodings (wire malleability) | `varint.rs:45-56` | Real (no minimality check on decode) but decode-only malleability; no wrong-value/panic. Defense-in-depth. |
| (4 more downgraded/refuted) | — | see full run output `wd2vwrqrc.output` | — | 5 downgraded + 2 refuted total; none funds-critical. |

## Cross-cutting themes (Wave 2)

1. **Network/coin-type provenance discarded for an out-of-band signal** (L8, L10, L11 + Wave-1 L2/L3) —
   trusting a coin-type child or `--network` over the authoritative network byte embedded in the key
   material. Standardize on a fail-closed "network from key material, cross-check coin-type" rule.
2. **The keyless `--md1-form=template` path is a second-class citizen** that regresses guards/fidelity
   present on the keyed path (H8 language drop, L9 missing refusals, M7 threshold-collapse). Worth a
   structural audit of `synthesize_template_descriptor` / `run_multisig_template_completion` vs their
   keyed twins.
3. **The BIP-380 origin annotation has two positional forms and the codebase disagrees with itself** —
   the toolkit drops the prefix form (H7), the GUI only recognizes the prefix form (L12). Both are
   hand-rolled regexes; both should delegate to the tree-based `canonical_origin`.
4. **Identity/canonicalization invariants the code doesn't uphold, hidden by a vacuous test** (L14, L15,
   L17). Library-API/doc gaps gated off the CLI today; latent traps for future md-codec consumers.
5. **Inherited codex32 integrity gap** (M6) — no digest share, so the combiner must verify cross-share
   polynomial consistency itself; it doesn't.

---

# Wave 3 — gap-coverage pass (files Wave 1/2 didn't deeply hit)

**Tally:** net-new HIGH ×3 · MEDIUM ×5 · LOW ×6 (+1 high folded into H3) + 3 downgraded-but-real.
Seeded with W1+W2 verdicts; 0 refuted.

> **Dedup:** Wave 3's `w3-gui-minikey-persist-plaintext` (HIGH, plaintext private key in `state.json`)
> is the same root cause as **H3** — H3's sub-point (5) already named the persistence leak. Folded into
> **H3** below (locations + fix expanded); not counted as a new item.

## Confirmed — HIGH (Wave 3)

### - [x] H9 · `import-wallet --network` mislabels heterogeneous-network entries (class-check on `first()`, rebind to ALL)
<!-- FIXED cycle-5 (toolkit v0.63.0 @bad8a3fb) — axis-1: the import-wallet --network class-check is now PER-ENTRY (reads each parsed entry's own network before the iter_mut rebind, was first()-only) → ImportWalletNetworkClassMismatch (exit 1). Distinct axis from the exit-2 NetworkMismatch xpub-version check (intentional two-axis coexistence). RED via --format bitcoin-core (only multi-entry parser). -->
- **repo/class:** toolkit · **A-wrong-address** · `w3-tk-iw-01`
- **location(s):** `cmd/import_wallet.rs:1191-1209` (guard vs rebind), `:1544` (per-entry emit);
  `wallet_import/bitcoin_core.rs:444-450` (per-descriptor coin-type)
- **bug:** the `--network` override resolves the coin-type class from **`parsed.first()`** only and uses
  it to gate the cross-class refusal, but then rebinds `p.network` for **every** entry via `iter_mut()`.
  A Bitcoin Core `listdescriptors` blob derives each entry's network independently (coin-type 0→Bitcoin,
  1→Testnet); cross-entry agreement is **not** enforced. So a heterogeneous `[Bitcoin, Testnet]` Vec where
  `first()` matches the override passes the guard and **all** entries (including the other-class one) get
  silently relabeled. Key material is untouched, so `bundle.network` contradicts the xpub/tpub prefix.
- **trigger:** blob `[testnet tpub desc, mainnet xpub desc]`, `import-wallet --format bitcoin-core … --json
  --network mainnet` → the testnet entry is emitted `bundle.network: mainnet` (baseline correctly says
  testnet). No warning.
- **consequence:** a consumer trusting `bundle.network` derives addresses under the wrong HRP/network. Worst
  case `[mainnet,testnet]+--network mainnet`: a testnet descriptor presented as mainnet, inviting real
  funds to addresses with no spendable mainnet key.
- **fix:** compute each entry's coin-type; refuse (or per-entry-gate) when entries span both classes — the
  guard and the rebind must operate on the **same per-entry** network, not `first()` for the guard and all
  for the write.
- **spec:** SPEC wallet-import signet/regtest disambiguation (override honored only within one coin-type
  class); BIP-44/SLIP-132 coin-type 0=mainnet/1=testnet; bech32 HRP network-distinct.

### - [x] H10 · Unsorted `multi(...)` silently exported as BIP-67 `sortedmulti` to Coldcard/Jade/Electrum → wrong addresses
<!-- FIXED cycle-2 (toolkit v0.62.0): 29b39723 — typed ExportWalletUnsortedMultisigUnsupported (exit 2) in the emit_payload chokepoint; PURE REFUSAL. FOLLOWUP export-wallet-unsorted-multi-silent-sortedmulti-coercion (+ open export-wallet-direct-descriptor-unsorted-multi-generic-refusal). -->
- **repo/class:** toolkit · **A-wrong-address** (B→A) · `w3-tk-export-2`
- **location(s):** `wallet_export/coldcard.rs:258-369`; `wallet_export/jade.rs:43-46`;
  `wallet_export/electrum.rs:131-191`; `cmd/export_wallet.rs:122-128` (dispatch accepts unsorted)
- **bug:** Coldcard and Electrum multisig file formats have **no** sorted-vs-unsorted field — both vendors
  **always** apply BIP-67 `sortedmulti` on reconstruction. The toolkit accepts unsorted `wsh-multi` /
  `sh-wsh-multi` (where literal key order is consensus-significant) for these formats and emits with no
  refusal/warning. So a user's unsorted multisig is reconstructed as sortedmulti, deriving **different**
  addresses whenever the per-index derived pubkeys aren't already in BIP-67 order.
- **trigger:** `export-wallet --format electrum --template wsh-multi --threshold 2 --slot @0.xpub=… --slot
  @1.xpub=…` (also coldcard-multisig / jade).
- **consequence:** the exported watch-only wallet watches sorted-multisig addresses ≠ the user's actual
  unsorted addresses; funds sent to the real addresses don't appear, generated receive addresses are wrong.
- **fix:** refuse unsorted `wsh-multi`/`sh-wsh-multi` for electrum/coldcard/jade (point to a faithful
  format: descriptor/bitcoin-core/bip388/sparrow), or gate behind an explicit `--allow-sortedmulti-coercion`
  with a loud warning.
- **spec:** BIP-67 (these vendor formats are sortedmulti-only); miniscript `multi` ≠ `sortedmulti` script.

<!-- FIXED cycle-13 (toolkit v0.66.0) — export emits a per-cosigner Derivation: line read from each cosigner's OWN sorted slot on divergent origins (sorted is the only reachable divergent case); shared line only when all agree; refuses rather than emitting m/0'/0'. Jade inherits via delegation. Independent whole-diff review GREEN (sorted-slot no-scramble verified). -->
### - [x] H11 · Coldcard/Jade multisig export collapses divergent cosigner paths to a wrong global `m/0'/0'`
- **repo/class:** toolkit · **B-policy-collapse** · `w3-tk-export-1`
- **location(s):** `wallet_export/coldcard.rs:324-336`; `wallet_export/jade.rs:46`;
  `wallet_import/coldcard_multisig.rs:38-49` (per-cosigner shape proves a faithful form exists)
- **bug:** `emit_coldcard_multisig_text` writes a single global `Derivation:` line only when **all**
  cosigner origins are identical; if they **diverge** it silently falls back to the literal placeholder
  `m/0'/0'` and emits **no** per-cosigner `Derivation:` lines — even though the format (and the toolkit's
  own import parser) supports per-cosigner `Derivation:` overrides. Divergent paths are legitimate
  (collaborative custody: cosigner A at account 0, B at account 7).
- **trigger:** `export-wallet --format coldcard-multisig … --slot @0.path=m/48'/0'/0'/2' --slot
  @1.path=m/48'/0'/7'/2'` → both exported under one `Derivation: m/0'/0'`; re-import corrupts both origins.
- **consequence:** the config declares the wrong BIP-32 path for every divergent cosigner. A Coldcard
  derives a different xpub at `m/0'/0'` and either refuses to register the wallet or registers one whose
  origins don't reproduce the real keys — breaks co-sign / fund recognition.
- **fix:** when paths diverge, emit a per-cosigner `Derivation:` line before each `<XFP>: <xpub>`; use a
  shared line only when all agree; refuse rather than emit `m/0'/0'`.
- **spec:** Coldcard multisig format (shared + per-cosigner `Derivation:` supported); BIP-48/32.

## Confirmed — MEDIUM (Wave 3)

### - [x] M8 · `build-descriptor` accepts a key carrying an extra derivation suffix → silently derives a deeper subtree
<!-- FIXED cycle-7 (toolkit v0.65.0 @20514561, tag mnemonic-toolkit-v0.65.0) — gate.rs::check_secret_key now REJECTS (else-after-xprv-screen) a key whose post-[origin]-strip xpub body contains '/' (the extra-derivation-suffix class) via existing DiagnosticKind::SchemaField (exit 2, no --json delta). Was: silently accepted → builder appended /<0;1>/* → DEEPER/WRONG subtree (mutation-proven). Covers all 4 key fields + nested recursion + both intake paths; no over-rejection (bare/[origin]/SLIP-132 still build). -->
- **repo/class:** toolkit · **A-wrong-address** · `w3-tk-descbuild-key-extra-path-suffix-silent`
- **location:** `descriptor_builder/ir.rs:218-227` (appends `/<0;1>/*`), `:22-23` (account-level contract);
  `descriptor_builder/gate.rs:347-360` (only an xprv screen exists)
- **bug:** the renderer unconditionally appends `/<0;1>/*` to every key, but nothing enforces the
  documented account-level contract — a key like `[fp/48h/0h/0h/2h]xpub.../5` is accepted and rendered as
  `xpub.../5/<0;1>/*`, deriving from the `…/5` subtree, not the account level. (A trailing `*` is caught;
  a fixed extra index isn't.)
- **trigger:** `build-descriptor --key '[…]xpub…/5' …` → emits `pk([…]xpub…/5/<0;1>/*)`, no warning.
- **consequence:** the engraved descriptor commits to a derivation the user didn't intend → funds at/only
  recoverable from different addresses; a copy/paste slip is unrecoverable-by-inspection (validates cleanly).
- **fix:** add a step-1 check that, after stripping the `[origin]` prefix, rejects any key whose xpub body
  is followed by extra `/`-segments; the renderer owns the `/<0;1>/*` suffix.
- **spec:** BIP-388 (key info items are account-level xpubs).

<!-- FIXED cycle-11a (mnemonic-gui v0.46.0 @1999323, PR #14) — TreeNode::zeroize_keys recursive walk (key + keys[i] + all children; hex public, excluded) wired into zeroize_form_state via state.tree.as_mut(). Whole-diff review enumerated every field, confirmed no missed secret + heap-zeroized + RED non-vacuous. -->
### - [x] M9 · GUI exit zeroize sweep skips `state.tree` → private keys typed into the descriptor builder never scrubbed
- **repo/class:** gui · **D-secret-leak** · `w3-gui-tree-key-not-zeroized-on-exit`
- **location:** `mnemonic-gui/src/secrets.rs:278-310` (`zeroize_form_state`); `schema/mod.rs:324-333`
  (`FormState.tree`); `form/tree_model.rs:81,89` (`TreeNode.key/.keys` plain String)
- **bug:** `zeroize_form_state` (documented to zero "every secret-class buffer") iterates
  values/slots/positionals/secret_widgets but **never** `state.tree`. The tree builder stores key material
  in plain `String`s rendered via `text_edit_singleline`; a user can enter an xprv/WIF/hex private key
  there. Persistence is defended (fail-closed allowlist), but the in-memory String is never zeroized.
  Distinct from the tracked allocator-residue caveat (here zeroize is never called at all).
- **fix:** recursively walk `state.tree` in the exit sweep, zeroizing each node's `key`/`keys`; ideally
  store tree keys in a zeroizing buffer. Add a test.

### - [x] M10 · BIP-86 single-key taproot `tr(@0)` falsely rejected (depth gate treats all `tr(...)` as depth-4 multisig)
<!-- FIXED cycle-9 (md-cli v0.9.0 @1a4b322) — bare keypath tr(@0)/tr(@0/<0;1>/*) now classified SingleSig (depth-3 BIP-86 accepted); script-path tr(...,{...})/multi_a stay MultiSig (no over-accept, depth gate kept strict). is_bare_keypath_tr (no top-level comma/brace). -->
- **repo/class:** md-cli · **E-panic-dos** (false reject) · `w3-mdcli-01`
- **location:** `md-cli/src/parse/template.rs:1792-1799` (`ctx_for_template`); `parse/keys.rs:67-77` (depth gate)
- **bug:** `ctx_for_template` maps only `wpkh(`/`pkh(`/`sh(wpkh(` to `SingleSig` (depth 3); every other head
  — including key-path `tr(@0/<0;1>/*)` — falls through to `MultiSig` (depth 4). A real BIP-86 account xpub
  is depth 3 (`m/86'/0'/0'`), so `parse_key` rejects it ("expected depth 4 … got 3"). The taproot shape is
  otherwise fully supported; the depth byte is advisory (discarded), so relaxing the gate can't corrupt
  addresses.
- **trigger:** `md encode "tr(@0/<0;1>/*)" --key @0=<real BIP-86 depth-3 xpub>` → exit 1.
- **consequence:** users can't encode/derive for the most common modern taproot wallet (BIP-86 single-sig)
  with its real xpub.
- **fix:** classify single-`@i` `tr(` as SingleSig (depth 3), or relax the MultiSig depth check to 3-or-4
  (depth never participates in derivation).
- **spec:** BIP-86 (depth-3 account); BIP-388.

### - [x] M11 · `parse_key` accepts an off-curve xpub (no secp256k1 point check); failure deferred to derive time
<!-- FIXED cycle-9 (md-cli v0.9.0 @1a4b322) — keys.rs adds a secp256k1 PublicKey::from_slice point check → off-curve xpub rejected at parse (BadXpub); real depth-3/4 xpubs still parse. -->
- **repo/class:** md-cli · **C-corrupt-accept** · `w3-mdcli-04`
- **location:** `md-cli/src/parse/keys.rs:33-80` (esp. `:78-79` payload copy, no point validation)
- **bug:** `parse_key` validates base58check/length/version/depth then blindly copies `bytes[13..78]`
  (chaincode‖pubkey) without checking `bytes[45..78]` is a valid compressed secp256k1 point. An off-curve
  (e.g. all-zero) pubkey passes, encodes into the Pubkeys TLV, and only fails later at `derive_address`.
- **fix:** validate via `PublicKey::from_slice(&bytes[45..78])` (or `Xpub::decode`) at intake; return
  `BadXpub`. Low (clean failure, unlikely accidental). · **spec:** BIP-32 (`bytes[45..78]` is a valid point).

<!-- FIXED cycle-12 (mk-cli v0.10.1 @df7c2eb) — repair.rs lowercases the mk1 prefix so all-uppercase input no longer re-emits a mixed-case string mk decode rejects; whole-diff review GREEN -->
### - [x] M12 · `mk repair` emits an INVALID mixed-case mk1 string for all-uppercase input (even when clean)
- **repo/class:** mk-cli · **other** (broken artifact) · `w3-mk-cli-repair-verify-1`
- **location:** `mk-cli/src/cmd/repair.rs:97-147` (`reconstruct_corrected`); `cmd/mod.rs:97` (no case-normalize)
- **bug:** `reconstruct_corrected` splices the original-cased prefix (`MK`) with lowercase data symbols from
  `ALPHABET`; input is never case-normalized (decode accepts all-uppercase). So an uppercase card yields
  `MK1qpslfap…` — **mixed case**, invalid per bech32/codex32. Uppercase is the canonical QR-friendly form,
  so this is a normal input class. Feeding the output back to `mk decode` errors "mixed case".
- **consequence:** `mk repair`'s entire purpose — a usable, re-ingestable string — produces a permanently
  invalid card for uppercase input, even with exit 0 (no correction needed). Fails loud on next decode.
- **fix:** normalize case before re-emitting (lowercase the prefix, or uppercase the data when input is
  upper). Add a re-decode round-trip test. · **spec:** BIP-173/93 (entirely-lower XOR entirely-upper).

## Confirmed — LOW (Wave 3)

<!-- FIXED cycle-13 (toolkit v0.66.0) — Electrum import accepts null root_fingerprint (→ 00000000 + NOTICE) / null derivation (→ script-type from SLIP-132 prefix, canonical origin synthesized + NOTICE; key-origin metadata only, never affects address derivation). Protocol fact verified vs live Electrum keystore.py. Whole-diff review GREEN. -->
### - [x] L18 · Electrum import hard-refuses valid wallets with null `root_fingerprint`/`derivation` (watch-only xpub imports, re-saved exports)
- **repo/class:** toolkit · **E-panic-dos** (false reject) · `w3-tk-electrum-import-deep-01`
- **location:** `wallet_import/electrum.rs:513-531` (singlesig), `:796-813` (multisig)
- **bug:** both paths require `root_fingerprint`/`derivation` to be non-null strings (`.as_str().ok_or_else`).
  Electrum emits these as JSON `null` for watch-only "use a master key" (xpub-import) wallets, older
  wallets, and — per the tracked round-trip quirk — toolkit-emitted files once re-saved by Electrum. The
  blob sniffs positive then bails. Fix: treat null as unknown-origin (`00000000` fp + purpose inferred from
  the SLIP-132 prefix) with a NOTICE. · **spec:** Electrum `keystore.dump()`; BIP-380 `00000000`.

### - [x] L19 · `md encode` emits "keyless template (no keys)" advisory even when `--key` embeds watch-only xpubs
<!-- FIXED cycle-9 (md-cli v0.9.0 @1a4b322) — same is_wallet_policy() gate at encode.rs json+text sites (sibling of L4). -->
- **repo/class:** md-cli · **D-secret-leak** (privacy) · `w3-mdcli-03` _(sibling of L4)_
- **location:** `md-cli/src/cmd/encode.rs:73-76,110-113`; `output_advisory.rs:35`
- **bug:** both emit paths call `emit_output_class_advisory(OutputClass::Template, …)` unconditionally; with
  `--key` the card is wallet-policy (embeds xpubs = watch-only material). Understates sensitivity — same
  bug class as L4 (`md repair`). Fix: branch on `is_wallet_policy()` → `WatchOnly`/`Template` (as `md
  address` does).

<!-- FIXED cycle-12 (mk-cli v0.10.1 @df7c2eb) — threshold corrected to 93 + "mk1".len() so a 96-symbol data-part labels "long" per mk-codec bch_code_for_length (96..=108); display-only, no funds/wire path -->
### - [x] L20 · `classify_code_variant` off-by-one mislabels a 96-symbol long-code mk1 chunk as "regular"
- **repo/class:** mk-cli · **other** (display) · `w3-mk-cli-repair-verify-2`
- **location:** `mk-cli/src/cmd/mod.rs:131-140`; authoritative `mk-codec/src/string_layer/bch.rs:117-124`
- **bug:** `s.len() <= 96+len("mk1")` (≤99) → "regular", but a long-code minimum data-part of 96 gives total
  99, so a 96-symbol long chunk is mislabeled. Display/JSON only. Fix: threshold `≤ 93+len("mk1")` (≤96), or
  classify via `bch_code_for_length`. · **spec:** BIP-93 (regular data-part 14..=93, long 96..=108).

### - [x] L21 · `convert (phrase|entropy)→bip38` with only `--passphrase` silently encrypts with an EMPTY passphrase
<!-- FIXED cycle-11b (toolkit v0.65.1 @af7100ff) — composite (seedqr|phrase|entropy)→bip38 with an unset --bip38-passphrase now REFUSED (ConvertRefusal, exit 2) at the Bip38=> sub-arm head inside the Seedqr|Phrase|Entropy=> outer arm (position-based, covers all THREE sources incl. seedqr). Predicate tests the in-scope bip38_passphrase.is_none() (NOT .is_empty()), so --bip38-passphrase "" still encrypts. Direct (wif↔bip38) edges' --passphrase fallback left as-is. Manual prose (edge table + --bip38-passphrase row) same commit. FOLLOWUP convert-composite-bip38-empty-passphrase-refusal (resolved). -->
- **repo/class:** toolkit · **D-secret-leak** · `w3-tk-convert-01`
- **location:** `cmd/convert.rs:1366` (empty fallback), `:932` (guard satisfied by `--passphrase`),
  `:1502,1522` (asymmetric direct edges fall back to `--passphrase`)
- **bug:** on the composite `→bip38` edge, `--passphrase` feeds only BIP-39 PBKDF2; the BIP-38 Scrypt layer
  uses `--bip38-passphrase`, which falls back to `""` when unset. The presence guard is satisfied by
  `--passphrase` and the "ignored" warning is suppressed, so the BIP-38 key is encrypted with an **empty**
  passphrase silently — asymmetric with the direct wif↔bip38 edges. A user migrating from v0.7 (dual-purpose
  `--passphrase`) produces an effectively-unprotected ciphertext.
- **fix:** refuse when `--bip38-passphrase` unset on this arm (funds-safe default), or emit a loud warning.
- **spec:** BIP-38 (Scrypt over the passphrase) vs BIP-39 (PBKDF2) — distinct layers.

### - [x] L22 · `apply_slot_stdin` reads a stdin secret into an unscrubbed `String` (partly defeats the `@N.<secret>=-` argv-avoidance)
<!-- FIXED cycle-14 (toolkit v0.67.0 @db0bf583) — SlotInput.value (incl. the parse_slot_input ctor + apply_slot_stdin store) is now SecretString (Zeroizing<String> inner, length-only redacting Debug), so both the stdin `=-` AND the @env: write-back (bundle / import-wallet / verify-bundle) secret-residue paths scrub on drop and never re-leak via {:?}. The convert / restore / addresses handler-scope passphrase / --from / BIP-38 locals are wrapped in Zeroizing<String> (restore's TemplateSeed.passphrase field too). The stdin readers stay bare String (D1 — flipping them would make 14 already-wrapping callers illegal Zeroizing<Zeroizing<String>>). mlock pins preserved. No wire/behavior change. The downstream phrase_overlays Vec is NOT yet scrubbed (it copies the phrase via .to_string() — status-quo-preserving, no NEW residue) — deferred to FOLLOWUP phrase-overlay-secretstring. Whole-diff review pending (PE). -->
- **repo/class:** toolkit · **D-secret-leak** (tracked Cycle-B) · `w3-tk-slot-input-friendly-1`
- **location:** `slot_input.rs:203-232,96-101`; `cmd/convert.rs:747` (`read_stdin_passphrase` bare String)
- **bug:** the `=-` sentinel keeps secrets off argv, but `apply_slot_stdin` reads stdin into a plain
  `String` and stores it in `SlotInput.value` (no Zeroize/Drop), so the secret lingers unscrubbed/swappable
  in heap. Same residue class as the ms-cli stdin reader fixed in Cycle A. Already tracked as Cycle-B target
  #1 (`secret-memory-hygiene-cycle-b`). Fix: `Zeroizing<String>` + `ZeroizeOnDrop` when Cycle-B lands.

### - [x] L23 · `ecies_decrypt_message` panics (not `InvalidScalar`) on a zero privkey (latent public-API)
<!-- FIXED cycle-7 (toolkit v0.65.0 @20514561) — explicit zero-scalar → Err(EciesDecryptError::InvalidScalar) before mul_tweak().expect(); reuses the existing variant. Latent (sole caller derive_storage_eckey already rejects zero), defensively closed. -->
- **repo/class:** toolkit · **E-panic-dos** (latent) · `w3-tk-electrum-crypto-01`
- **location:** `electrum_crypto.rs:345-351`; `:309-311` (sole in-tree caller is safe)
- **bug:** `Scalar::from_be_bytes` accepts 0 (≤ n−1), so the `InvalidScalar` guard passes; then
  `mul_tweak` returns `Err(InvalidTweak)` for a zero tweak and hits `.expect(...)` → panic, instead of the
  typed `EciesDecryptError::InvalidScalar`. Not CLI-reachable (the only caller reduces mod n and rejects
  zero); latent public-API misuse. Fix: reject zero scalar before `mul_tweak`, or `map_err` the result.

## Wave 3 — downgraded-but-real (track, lower severity than filed)

| sev | repo · dim | finding | location | note |
|---|---|---|---|---|
| **MEDIUM** (filed high) **✓ FIXED cycle-6 (toolkit v0.64.0 @8d2fe505)** | toolkit · descriptor-builder | **decaying-multisig decay-ordering compares raw BIP-68 operands without normalizing the 512-sec unit flag → a recovery quorum can silently unlock BEFORE the primary timelock** | `descriptor_builder/archetype.rs:305-317` (`validate_params`) | The only producer-level guard for "tiers unlock progressively later" compares raw `u32` `older`/`recovery_older` (clap raw `Option<u32>`) with no BIP-68 unit (blocks vs 512-sec) normalization. Real funds-relevant — **worth a real fix.** **D-decay-rel — FIXED:** new `older_unit_value` helper classifies BIP-68 bit-22; `validate_params` REFUSES cross-unit `--older`/`--recovery-older` pairs (un-orderable offline) + non-strict same-unit order → `Diagnostic{Param}` exit 2. |
| low (filed low) | md-codec · chunk | `chunk::split` sizes chunks by raw payload bits, ignoring the 37-bit per-chunk header in the codex32 length budget | `md-codec/src/chunk.rs:219-289` | Header is 37 bits exactly; budget arithmetic omits it. |
| low (filed medium) | md-cli · keys/format | Named `--path` shortcuts (bip44/48/49/84/86) hardcode mainnet coin-type 0'; `--network testnet` still records 0' | `md-cli/src/parse/path.rs:25-34`, `cmd/encode.rs:46-49` | md-cli sibling of the network-provenance theme; `parse_path` called without network. |

## Cross-cutting themes (Wave 3)

1. **Fidelity loss / wrong-material at format boundaries** — 3 of 4 highs (H9 network mislabel, H10
   unsorted→sorted coercion, H11 path-collapse) are silent transformations in the toolkit's export/import
   emitters where the emitted artifact's network/order/path no longer matches the key material. Discipline:
   emitters must validate per-key invariants against the **actual** material, never `first()` or a silent
   collapse.
2. **Secret-at-rest / in-memory hygiene gaps** concentrated in the GUI (minikey→`state.json` plaintext via
   the narrow allowlist [→H3]; tree-builder private keys skipped by the exit zeroize sweep [M9]) plus a
   tracked toolkit stdin-residue item (L22). Fail-closed on value-dependent secret composites
   (`SECRET_NODE_TYPES_ARGV`).
3. **Input-contract rigidity vs laxity** — md-cli falsely rejects valid BIP-86 taproot (M10) and accepts
   off-curve xpubs (M11); build-descriptor accepts stray derivation suffixes (M8); Electrum import
   over-rejects null-fingerprint wallets (L18). Tighten where it's lax, loosen where it's wrong.
4. **The network-provenance cluster keeps recurring** — H9 + the downgraded md-cli named-path item join
   Wave-1 L2/L3 and Wave-2 L8/L10/L11. A single fail-closed "network from key bytes, cross-check coin-type"
   rule across all import/export/convert/build paths would close the whole family.

---

# Final round — adversarial / taproot / multipath / timelock (Wave-A lenses, 2 passes)

**Tally:** HIGH ×2 · LOW ×2 + 2 new downgraded-but-real. Saturation reached on these lenses (low yield,
mostly siblings of known patterns) — _except_ the taproot BIP-48 origin bug, a real new funds-loss item.

## Confirmed — HIGH (final round)

### - [x] H12 · Descriptor-mode default origin engraves BIP-48 script-type `2'` (P2WSH) for taproot multisig instead of `3'` (P2TR) → wrong re-derived keys/addresses
<!-- FIXED cycle-1 (toolkit v0.61.0 @f9467cc5) — taproot-aware default-origin (reuse bip48_script_type() → 3' for Tag::Tr, 3 sites incl. descriptor_intake). Report-tick reconciliation in cycle-3 (shipped but checkbox never flipped). -->
<!-- See the detailed H12 (→ CRITICAL) entry below. -->
- **repo/class:** toolkit · **A-wrong-address** · `w4a-taproot-bip48-default-script-type` · _new funds-loss_
- **location(s):** `cmd/bundle.rs:2210-2235` (`compute_default_origin_path`, hardcoded `value:2` at `:2231`),
  `:1397-1398,1444-1445` (call site); `cmd/verify_bundle.rs:1373` (mirror reproduces the wrong path);
  `cmd/xpub_search/descriptor_intake.rs:324,345` (`bip48_default_path(network, account, 2)`)
- **bug:** `compute_default_origin_path` unconditionally builds `m/48'/<coin>'/<account>'/2'` for every
  non-canonical multisig descriptor whose cosigner keys lack a `[fp/path]` origin — **without inspecting the
  tree**. md-codec's `canonical_origin` returns `None` for `tr(@N, TapTree)` (`canonical_origin.rs:56`), so
  taproot multisig **always** enters this branch and is annotated with the segwit `2'` instead of taproot
  `3'`. The toolkit already knows the right mapping (`template.rs:231-235 bip48_script_type()` → `Some(3)`
  for `TrMultiA|TrSortedMultiA`) and the **template-mode** path uses it correctly — only descriptor-mode is
  wrong, producing two divergent origins for the same taproot wallet.
- **trigger:** `bundle --descriptor "tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))" --slot @0.xpub=… --slot
  @1.xpub=… --network mainnet --account 0` → CLI prints "defaulting origin path … to m/48'/0'/0'/2'" and the
  emitted mk1/md1 record `@ m/48'/0'/0'/2'` on a taproot script.
- **consequence:** the exported watch-only taproot multisig carries `[fp/48'/coin'/account'/2']` origins; a
  coordinator (Coldcard/Sparrow/Jade) re-derives each cosigner xpub from the **P2WSH** path instead of
  **P2TR** → different pubkeys, different taproot output, different address — a wallet no participant can
  co-sign for (funds-loss). verify-bundle mirrors the same wrong path, so it can't catch it.
- **fix:** make the default-path script-type taproot-aware — pass `is_taproot = (d.tree.tag == Tag::Tr)`
  into `compute_default_origin_path`, emit `3'`/`2'`/`1'` via `CliTemplate::bip48_script_type()` as the
  single source of truth; apply identically to `verify_bundle.rs:1373` and
  `descriptor_intake.rs:324,345` (mirror lockstep); fix the baked-in `value:2` taproot test fixtures.
- **spec:** BIP-48 `m/48'/coin'/account'/script_type'` (1'=P2SH-P2WSH, 2'=P2WSH, **3'=P2TR**); the toolkit's
  own `template.rs:231-235` already encodes `3'` for taproot.

### - [x] H13 · Hardened multipath alternatives (`<0h;1h>` / `<2';3'>`) silently dropped at template lex → single-path key (md-cli + toolkit mirror)
<!-- FIXED cycle-1 (toolkit v0.61.0 @f9467cc5 + md-cli v0.8.0 @58cc9ec) — lexer REJECTS hardened/malformed multipath via typed error (md-cli + toolkit lockstep; hardened-from-xpub is impossible). Report-tick reconciliation in cycle-3 (shipped but checkbox never flipped). -->
<!-- See the detailed H13 (→ CRITICAL) entry below. -->
- **repo/class:** md-cli (+ toolkit mirror) · **B-policy-collapse** · `w4a-mp-2`
- **location(s):** `md-cli/src/parse/template.rs:40` (`lex_placeholders` multipath body class `[0-9;]+`),
  `:220-233` (hardcoded `hardened:false` at `:225`), `:365` (canonicity probe shares the class);
  **toolkit mirror:** `mnemonic-toolkit/src/parse_descriptor.rs:70,227-230` (same defect, verifier
  downgraded the toolkit instance to low but it shares the root cause)
- **bug:** the multipath capture `(?:/<([0-9;]+)>)?` can't match the hardened markers `'`/`h`, and because
  the group is optional, `@0/<0h;1h>/*` lexes to just `@0`; `make_use_site_path` then hardcodes
  `hardened:false`, yielding `multipath: None`. So `md encode "wsh(multi(2,@0/<0h;1h>/*,@1/<0h;1h>/*))"`
  emits an md1 with a bare single-path key — the hardened multipath is **unrepresentable and silently
  dropped** at encode time.
- **consequence:** silent policy-collapse — the engraved card records a single-path non-wildcard key instead
  of the hardened-multipath wallet; restore derives at the wrong path with no error.
- **fix:** extend the capture to `[0-9;'h]+`, parse the per-alternative hardened marker into
  `Alternative.hardened` (replace the hardcoded `hardened:false`), update the canonicity regex identically;
  or reject hardened-multipath bodies with a clear error. **Fix the md-cli and toolkit mirrors in lockstep**
  (m-format mirror invariant); companion FOLLOWUPS in both repos.
- **spec:** BIP-389 (multipath permits hardened indices); md-codec `use_site_path.rs Alternative.hardened`
  carries the per-alternative hardened bit this lexer can never populate.

## Confirmed — LOW (final round)

### - [x] L24 · `verify-bundle` descriptor-mode `--slot @N.path` loop indexes `new_paths[idx]` unbounded → OOB panic for `idx ≥ n`
<!-- FIXED cycle-11b (toolkit v0.65.1 @03017917) — mirrored bundle.rs:1373-1388's exact-coverage `max(idx+1) != n` gate into verify_bundle.rs (iterating args.slot), after validate_slot_set and before the canonicity probe. OOB panic (new_paths[idx] at :1435) → clean DescriptorParse (exit 2); also catches the under-n case (bundle.rs parity). Standalone gate carries an S-VERIFY fold note. FOLLOWUP verify-bundle-bundle-rs-descriptor-mode-dedup (OPEN — folds the gate into the shared descriptor-mode binding fn when the S-VERIFY dedup lands). -->

- **repo/class:** toolkit · **E-panic-dos** · `w4a-1`
- **location:** `cmd/verify_bundle.rs:1374-1393` (`new_paths` built with exactly `n`), `:1425` (write, no
  `idx<n` guard), `:1456-1462` (range-checked loop runs **after**); `slot_input.rs:249-267`
  (`validate_slot_set` can't range-check — `n` unknown at that layer)
- **bug:** guard-asymmetry — `bundle.rs:1373-1388` has an unconditional `index+1.max() != n` gate before its
  override loop; the hand-copied `verify_bundle.rs` mirror **omits** it, so `new_paths[*idx as usize]` with a
  user `--slot @N` index ≥ n panics (OOB). Gated behind a phrase/seedqr/ms1-bearing slot + non-canonical
  descriptor. Operator-misuse, not attacker input; abort, no funds/secret impact.
- **fix:** add the `bundle.rs` index-vs-n gate before the path-override loop (clean `ToolkitError`).
  **Structural:** deduplicate the `bundle.rs ↔ verify_bundle.rs` descriptor-mode binding into one shared
  function so guard-drift (this + H1's class) can't recur.

### - [x] L25 · `import-wallet` keyed/keyless classifier blind to raw x-only taproot keys → routes to the "keyless" error message
<!-- FIXED cycle-11b (toolkit v0.65.1 @3bc84d0c) — has_any_key_token extended with ADDITIVE position-aware anchors `(?:tr|pk|pk_k|pk_h)\([0-9a-fA-F]{64}`, matching a 64-hex x-only key only in a taproot KEY position (not as a bare token, so sha256/hash256/ripemd160/hash160 64-hex args stay keyless). The 66-hex `02/03` compressed-key alternation is unchanged. Origin-less tr(<xonly>,...) now re-routes from "keyless script" to the correct "must carry a key origin" message; both arms still Err. FOLLOWUP import-classify-xonly-position-aware (resolved). -->

- **repo/class:** toolkit · **other** · `w4a-taproot-01`
- **location:** `wallet_import/pipeline.rs:53-60` (`has_any_key_token` regex), `:160-180`
  (`classify_descriptor_form` `(false,false)` arm)
- **bug:** `has_any_key_token` matches xpub-family + `02/03`-prefixed compressed pubkeys but **not** bare
  32-byte x-only (BIP-340/341) keys, so a `tr(<xonly>, pk(<xonly2>))` descriptor with no origin annotation
  is misclassified, surfacing the wrong error. Benign — both classifier arms reject the descriptor anyway,
  so it only affects the error message. **fix:** replace regex key-sniffing with structural descriptor
  parsing for taproot x-only positions.

## Final round — downgraded-but-real (track)

| sev | repo · dim | finding | location | note |
|---|---|---|---|---|
| low (filed med) | md-codec · adversarial-decode | multi-chunk `reassemble` cross-chunk integrity is only a 20-bit `chunk_set_id` (no per-set content hash) → a swapped chunk from a different descriptor is accepted at ~2⁻²⁰, vs sibling **mk-codec's deliberate 32-bit SHA-256** cross-chunk hash | `md-codec/src/chunk.rs:305-389` (check `:378-386`), `:175-179` (`derive_chunk_set_id`) | Defense-in-depth asymmetry with mk-codec; worth aligning. |
| low **✓ FIXED cycle-6 (toolkit v0.64.0 @8d2fe505)** | toolkit · timelock | decaying-multisig **tier-3 absolute `after(T)` is never validated against the decay invariant** → a past height makes the last-resort key immediately spendable | `descriptor_builder/archetype.rs:305-317`, `gate.rs:306-324` | Distinct facet of the W3 decaying-multisig item (relative `older` there; absolute `after` here). Real funds-relevant; pair the fix. **D-decay-abs — FIXED:** `validate_params` REFUSES a PAST absolute `after(N)` via static BIP-65 past-floors (height + unix-time, strict `<`, monotone-safe) → `Diagnostic{Param}` exit 2. Canon fixtures migrated `after(500000)`→`after(4000000)`. |

_(Refuted in the final round: encode walker `k as u8` threshold truncation — not reachable, masked by the
5-bit wire n-cap; + the Wave-A refutals already noted.)_

---

# Wave B (re-run) — provenance / fingerprint / seed-derivation / checksum

**Tally:** HIGH ×2 · MEDIUM ×2 · LOW ×1. **4 of 5 are network-provenance defects** — the single most
recurring theme of the whole hunt.

## Confirmed — HIGH (Wave B)

<!-- FIXED cycle-13 (toolkit v0.66.0) — depth-gated master-fp matrix: depth>0 + no supplied XFP now REFUSES (master fp unrecoverable from an account key — was silently substituting the account fp); supplied XFP at depth>0 accepted without spurious warning; xpub.fingerprint() treated as master only at depth 0. Jade inherits. Independent whole-diff review GREEN (no account-fp-as-master leak). -->
### - [x] H14 · `coldcard-multisig` import uses the account xpub's own fingerprint (depth>0) as the master fingerprint → wrong `[fp/path]` on every cosigner
- **repo/class:** toolkit · **A-wrong-address** · `w4b-fp-01` (merges `w4b-fp-02`)
- **location(s):** `wallet_import/coldcard_multisig.rs:358-360` (`computed_fp = xpub.fingerprint()`),
  `:363-399` (5-row truth table — Row 4 silent substitute, Row 2 spurious warning), `:415`
  (`[fp/path]` format), `:936-957` (test fixtures pin FP to `xpub.fingerprint()`, masking both)
- **bug:** a BIP-380 key-origin needs the **master** fingerprint (`HASH160(pubkey)[:4]` at depth 0), but
  `xpub.fingerprint()` is the **account** key's own identifier at `m/48'/0'/0'/2'`. With no depth guard:
  (Row 4, no XFP — the accepted older-firmware/third-party shape) it **silently** substitutes the account
  fingerprint as the master fp; (Row 2, real master XFP supplied) it can essentially never equal
  `xpub.fingerprint()` at depth>0, so a "disagrees with computed fingerprint" warning fires on **every**
  cosigner of **every** authentic export. (`json_envelope.rs:383-399` does the same substitution but emits
  a loud NOTICE; coldcard-multisig Row 4 is silent.) child→parent fp recovery is one-way.
- **trigger:** `import-wallet --format coldcard-multisig <file>` on the `Derivation: m/48'/0'/0'/2'` + bare
  `<xpub>` shape → cosigner origins become `[<account-xpub-fp>/48'/0'/0'/2']` instead of `[<master-fp>/…]`.
- **consequence:** wrong master fp on every cosigner → coordinators/signers matching by master fp (PSBT
  hints, "not my key" checks) fail to recognize the device; the engraved/re-exported backup carries a fp
  that doesn't identify the signing seed. Plus false "internally inconsistent" warnings erode warning signal.
- **fix:** only treat `xpub.fingerprint()` as a master fp when the cosigner xpub is itself depth 0; at
  depth>0 with no XFP, **refuse** (master fp unrecoverable from an account xpub) rather than substitute; at
  depth>0 with a supplied XFP, accept it as authoritative without the disagreement warning. Fix the SPEC
  §11.4.1 note + test fixtures (use realistic master fps ≠ `xpub.fingerprint()`).
- **spec:** BIP-32 (master/parent fp = HASH160(pubkey)[:4], child→parent one-way); BIP-380 key-origin;
  rust-bitcoin `Xpub::fingerprint()` = current key's identifier.

### - [x] H15 · 7 import parsers derive network from the BIP-48 coin-type, never cross-checking the xpub/tpub version bytes → wrong-network accept/mislabel
<!-- FIXED cycle-5 (toolkit v0.63.0 @bad8a3fb) — axis-2 STRUCTURAL ANCHOR: new shared `pipeline::assert_slots_network_agrees` wires `assert_network_agrees` (xpub's OWN NetworkKind vs coin-type NetworkKind, 2-way Main/Test) at ALL 7 import parsers (descriptor/specter/sparrow/bitcoin-core/bsms/coldcard-multisig/electrum) → NetworkMismatch (exit 2). No-op when no asserted network (originless). Wires the formerly-dead variant. -->
- **repo/class:** toolkit · **A-wrong-address** · `w4b-1` _(the network-provenance cluster, root instance)_
- **location(s):** `wallet_import/descriptor.rs:168-213` (`network_from_origins`/`coin_type_from_path`);
  `specter.rs:370-397`; `sparrow.rs:591-633`; `bitcoin_core.rs:430-475`; `bsms.rs:386-430`;
  `coldcard_multisig.rs:678-707`; `electrum.rs:698-716`; `pipeline.rs:109-122`; `slip0132.rs:66-103`
  (`normalize_xpub_prefix` only rejects unknown prefixes); `mod.rs:508-515` (`validate_watch_only_resolved`)
- **bug:** every descriptor-bearing import parser computes network **solely** from the BIP-48 coin-type
  (`comps[1]==0'`→Bitcoin, `1'`→Testnet). The cosigner xpub is decoded but `normalize_xpub_prefix` only
  swaps/rejects SLIP-132 prefixes — a canonical mainnet `xpub`/testnet `tpub` passes unchanged and
  `Xpub.network` (the true network) is never compared to the coin-type-derived network. So
  `wpkh([fp/84'/1'/0']xpub…)` (mainnet key on testnet path) is accepted as `network = Testnet`, and the
  inverse as Mainnet; addresses render under the wrong HRP/version while the key is the other network.
- **trigger:** `import-wallet --format descriptor|specter|sparrow|bitcoin-core|bsms|coldcard-multisig|electrum`
  on a blob whose coin-type disagrees with the xpub/tpub version bytes.
- **consequence:** the bundle's network label and rendered/round-tripped addresses contradict the key's true
  network → user watches the wrong chain; corrupted/hand-edited foreign exports accepted without complaint.
- **fix:** after decoding each xpub, cross-check `xpub.network` against the coin-type-derived network and
  reject on mismatch — **the dead `ToolkitError::NetworkMismatch` variant already exists for this.** (NB the
  *bundle synthesis* path already does this via `synthesize.rs:771-783` `CosignerSpec`; the **import**
  parsers don't — port the same invariant.) Closes H15 + M13 + M14 with one shared rule.
- **spec:** SLIP-132 (`xpub 0x0488B21E` mainnet / `tpub 0x043587CF` testnet); BIP-44/48 coin-type registry.

## Confirmed — MEDIUM (Wave B)

### - [x] M13 · `export-wallet --from-import-json` takes network from the envelope JSON string, no cross-check vs the descriptor's xpub version bytes
<!-- FIXED cycle-5 (toolkit v0.63.0 @bad8a3fb) — S-NET: export-wallet --from-import-json now cross-checks each decoded xpub's network vs the envelope's declared bundle.network via `assert_network_agrees` → NetworkMismatch (exit 2). -->
- **repo/class:** toolkit · **A-wrong-address** (C→A) · `w4b-4`
- **location:** `cmd/export_wallet.rs:711-712` (`network = cli_network_from_str(envelope.bundle.network)`),
  `:824`; `wallet_import/json_envelope.rs:149-154,339-410,483-495` (only the BIP-380 checksum is validated);
  `slip0132.rs:108-111` (`apply_xpub_prefix` **overwrites** version bytes)
- **bug:** the `--from-import-json` export trusts the envelope's `network` string entirely; the only
  integrity check on the descriptor is its checksum, not its xpub network. A hand-edited
  `{"network":"mainnet", …"descriptor":"wpkh([fp/84'/1'/0']tpub…)"}` exports a mainnet-labeled file
  containing testnet keys — and `apply_xpub_prefix` overwrites the version bytes, so emitters re-emit
  wrong-network SLIP-132 xpubs.
- **fix:** cross-check the descriptor/cards' `Xpub` NetworkKind against `envelope.bundle.network` and reject
  (reuse `NetworkMismatch`). · **spec:** SLIP-132; the envelope's own threat model already guards the
  checksum but not the network.

### - [x] M14 · `convert --xpub-prefix` re-emits with the `--network` version family without checking the xpub's own network
<!-- FIXED cycle-5 (toolkit v0.63.0 @bad8a3fb) — S-NET: convert --xpub-prefix now cross-checks the input xpub's own network vs the --network family via `assert_network_agrees` → NetworkMismatch (exit 2), preventing a cross-network prefix re-emit. -->
- **repo/class:** toolkit · **A-wrong-address** · `w4b-5`
- **location:** `cmd/convert.rs:1100-1113` (`apply_xpub_prefix(&xpub, prefix, network)` with
  `network = args.network.unwrap_or(Mainnet)`), `:921-926` (guard checks **presence** of `--network`, not
  agreement); `slip0132.rs:197-211` (`swap_target_for` keys purely on the CLI arg)
- **bug:** `convert --from xpub=<testnet tpub> --to xpub --xpub-prefix zpub --network mainnet` emits a
  mainnet `zpub` whose decoded key is the **testnet** account key — the version bytes advertise the wrong
  network. The only guard requires `--network` to be present, not to match the xpub.
- **fix:** verify `xpub.network == args.network.network_kind()` before applying a non-default prefix; refuse
  on mismatch (the SPEC's "user responsibility" note is the design choice being challenged — fail-closed is
  warranted for a steel-backup tool). · **spec:** SLIP-132 version-byte table.

## Confirmed — LOW (Wave B)

### - [x] L26 · `ms combine --to entropy` silently drops the mnem wordlist-language (asymmetric vs the toolkit, which warns)
<!-- FIXED cycle-8 (ms-cli v0.9.0 @e80ea3b) — ms combine --to entropy now emits a non-English-wordlist advisory (stderr-only, no --json/exit change; entropy is correct, language is re-encode metadata); --to ms1 preserves the language byte. -->
- **repo/class:** ms-cli · **B-policy-collapse** · `w4b-sdl-01`
- **location:** `ms-cli/src/cmd/combine.rs:91-117` (payload routing), `:157-175` (`emit_entropy` — hex only)
- **bug:** when `ms combine` recovers a `Payload::Mnem{language,entropy}` (non-English wordlist) and the user
  picks `--to entropy`, it emits only the raw entropy hex with **no** language advisory — the wire language
  byte is extracted then discarded. The toolkit's equivalent `mnemonic ms-shares combine --to entropy` emits
  `non_english_seed_advisory` for exactly this; ms-cli has no such helper. Entropy hex is correct; the gap is
  the missing advisory.
- **consequence:** a user recording only the entropy hex (no language note) and later recovering with
  English-default BIP-39 software derives a different seed → different wallet → apparent loss.
- **fix:** mirror the toolkit — emit a stderr advisory naming the recovered wordlist language on the
  `--to entropy` arm (port `non_english_seed_advisory` into ms-cli). · **spec:** BIP-39 (seed = PBKDF2 over
  the language-specific sentence — wordlist language is load-bearing).

## Wave B — appendix (downgraded / refuted)

| status | finding | location | why |
|---|---|---|---|
| downgraded (partial) | bundle/export xpub slots accept an xpub whose network disagrees with `--network` | `bundle.rs:586-621`, `export_wallet.rs:449-478` | **Bundle half REFUTED** — `synthesize_unified`/`synthesize.rs:771-783` already cross-checks (`CosignerSpec`). The export `--descriptor` residue overlaps **M13**. |
| refuted | "`ToolkitError::NetworkMismatch` is dead → no cross-check anywhere" | `error.rs:265-269`, `network.rs:1-4` | FALSE — the bundle synthesis path **does** enforce the cross-check (via `CosignerSpec`, not `NetworkMismatch`). The variant is unused, but the *import/export/convert* paths are the real gap (H15/M13/M14), not "everywhere". |
| refuted | `recanonicalize_descriptor` recomputes the #checksum instead of verifying | `wallet_import/roundtrip.rs:231-263` | Matches an accepted recompute pattern; not a tamper vector in context. Not a bug. |

---

# Per-repo fix index (route to owners)

**mnemonic-toolkit** (the integration crate, most findings): H1, H3, H7, H8, H10, H11, H12, H14, H15 ·
M1, M7, M8, M9, M13, M14 · L1, L2, L3, L8, L9, L10, L11, L21, L22, L23, L24, L25 + the decaying-multisig
tier-3 `after()` downgraded item. _Cross-cutting fixes:_ (a) one fail-closed "xpub network MUST agree with
asserted network/coin-type" invariant across import/export/convert (closes H15/M13/M14 + Wave-1/2 network
items); (b) deduplicate `bundle.rs ↔ verify_bundle.rs` descriptor-mode binding (closes H1-class drift, L24,
and the H12 mirror); (c) make `--md1-form=template` mirror the keyed path (H8/L9/M7).

**descriptor-mnemonic (md-codec / md-cli):** H6, H13 · M4, M10, M11 · L4, L6, L7, L14, L15, L16, L17, L19,
L20-sibling + downgraded chunk-budget / named-path-coin items. _Funds-critical:_ H6/M4 (BCH out-of-domain),
H13 (hardened-multipath drop — **lockstep with the toolkit mirror**).

**mnemonic-key (mk-codec / mk-cli):** M12 (mk repair mixed-case), L20 (variant off-by-one).

**mnemonic-secret (ms-codec / ms-cli):** H4, H5 (non-English `unreachable!` panics) · M6 (Shamir wrong-secret
combine) · L5, L26 (language advisory). _Funds-critical:_ M6.

**mnemonic-gui:** H2 (unmasked argv debug log), H3 (minikey leak — argv + persistence) · M9 (tree key not
zeroized) · L12, L13.

---

## Status: hunt complete

5 waves (~170 agents, ~18M subagent tokens): broad (18 dims) → deep-core (16) → gap (13) → adversarial/
taproot/timelock (×2 passes, 8) → provenance/fingerprint/seed/checksum (4). Refute-by-default verification
with a second skeptic on every confirmed crit/high; protocol/crypto facts spec-checked. The static pass
reported **no criticals** — but the **differential-oracle wave overturned that**: H12, H1, and H13 are now
empirically-proven **CRITICAL** (real diverging addresses, key-level proof, full repro). The remaining
high-impact items still require a non-default override / non-English seed / vendor-format export / tampered
input, and several static HIGH/MEDIUMs proved **metadata-only** (addresses correct) under the oracle. Each
`- [ ]` item is a fix checklist entry — tick it with the fixing commit when shipped.

---

# Differential-oracle wave — EMPIRICAL results (`wf_8c03549a-c7c`)

**Method:** 4 derivation slices. Each built the tools, stood up its own regtest `bitcoind` (v27.0, unique
datadir/port), constructed intended wallets, derived ground-truth addresses via Core `deriveaddresses` +
independent pure-Python BIP32/BIP39/secp256k1, ran the toolkit's bundle→restore / import→export /
build-descriptor paths, and **diffed actual addresses**. **6 confirmed, 2 downgraded, 0 refuted, 14
clean-negatives, 0 NEW.** Severities here SUPERSEDE the static ratings.

## EMPIRICALLY-PROVEN CRITICAL (escalated from static HIGH)

### - [x] H12 (→ CRITICAL) · descriptor-mode taproot multisig derives cosigner keys at BIP-48 `2'` not `3'` — every address diverges
<!-- FIXED cycle-1 (toolkit v0.61.0 @f9467cc5) — taproot-aware compute_default_origin_path (3' for Tag::Tr via bip48_script_type(), incl. Descriptor::Tr at descriptor_intake). Report-tick reconciliation in cycle-3. -->
- proof: descriptor-mode bundle prints "defaulting origin path … to m/48'/0'/0'/2'"; the **mk-decoded
  cosigner xpub is byte-for-byte the independent `2'` BIP32 derivation, ≠ `3'`** → keys live in the wrong
  subtree (not just a label). Core `deriveaddresses`: intended `2-of-2 tr(NUMS,multi_a)` receive[0]
  `bcrt1p20ad3q3errr7h4p06j6vxj7sygppgnvjylnyyejy8nuz77jxgv5qmqyk3v` (`3'`) vs toolkit
  `bcrt1pe8q8h9a67gq6fpuycxu8zuskwg6e93vu4qryfekclcj7lly8atqsjv2ww7` (`2'`) — **all 6 receive + change[0]
  differ**; also reproduced for `sortedmulti_a`.
- location: `cmd/bundle.rs::compute_default_origin_path` (hardcoded `2`, no taproot inspection); mirrored in
  `verify_bundle.rs` + `xpub_search/descriptor_intake.rs`. `template.rs::bip48_script_type()` already
  returns `3` for `TrMultiA/TrSortedMultiA` and template-mode is correct — only descriptor-mode is wrong.
- consequence: any BIP-48 coordinator (Sparrow/Coldcard/Jade) re-derives at `3'` → different keys, output,
  address at every index → coins unspendable by any participant. _Fix elevated to top of the program._

### - [x] H1 (→ CRITICAL) · `verify-bundle` returns `result: ok` (exit 0) for a wallet that reconstructs DIFFERENTLY
<!-- FIXED cycle-1 (toolkit v0.61.0 @f9467cc5) — emit_multisig_checks md1 compare widened to expected.tree == desc.tree && use_site_path == && tlv.use_site_path_overrides == (origins excluded per L14). Report-tick reconciliation in cycle-3. -->
- proof: engraved a real `wsh(sortedmulti(2,A,B,C))` bundle, then verify-bundle GREEN-lit (exit 0): (a)
  `sortedmulti(1,…)` 1-of-3 **anyone-spends**, (b) `multi(2,…)` **unsorted**, (c) `sh(wsh(sortedmulti(2)))`
  **P2SH-nested** — addresses `bcrt1q3nh67k…` / `bcrt1qja7rvt…` / `2MypJZ…` all ≠ the real
  `bcrt1qea2gkg…`. Clean-negative: a genuinely-wrong cosigner xpub → `result: mismatch` exit 4 (proves the
  gate is **structurally blind**, not always-green).
- location: `verify_bundle.rs:2406-2489` — `md1_xpub_match` is a sorted-pubkey-multiset compare; tree
  Tag/threshold/wrapper never compared. Fix: `expected.md1 == supplied.md1` (the keyless single-sig path at
  `:583` already does this). _verify-bundle blindness compounds H12/H10/H13 — it's the safety net that
  should catch all three structural-drift classes and doesn't._

### - [x] H13 (→ CRITICAL) · hardened multipath `<0h;1h>`/`<2';3'>` silently collapsed to bare `/*` → wrong addresses (md-cli + toolkit)
<!-- FIXED cycle-1 (toolkit v0.61.0 @f9467cc5 + md-cli v0.8.0 @58cc9ec) — lex_placeholders / template lexer REJECT hardened+malformed multipath via typed error (lockstep). Report-tick reconciliation in cycle-3. -->
- proof: `md encode` exits 0 silently; `md decode` of the md1 (both md-cli **and** the toolkit
  `bundle --descriptor` md1) returns the collapsed `wsh(multi(2,@0/*,@1/*))`. `md address` renders
  `bcrt1qq0kxm9…` = Core's bare-key derivation, while the intended hardened wallet is `bcrt1q5tgwjk…`. Core
  **rejects** `<0';1'>` ("not a valid uint32") → the correct behavior is to ERROR, not collapse.
- location: md-cli `parse/template.rs:40,220-233` (`[0-9;]` class can't match `'`/`h`; hardcoded
  `hardened:false`) + toolkit mirror `parse_descriptor.rs:70,227-230`. **Fix both in lockstep.**

## Empirically confirmed (severity stands)

- **H10 (HIGH)** — full export→reimport proven: unsorted `wsh(multi(2,A,B,C))` exported to electrum
  (field-less) re-imports as `wsh(sortedmulti(2,…))`; receive[0] `bcrt1qja7rvt…` (unsorted) ≠
  `bcrt1qea2gkg…` (sorted), **4 of 6 indices diverge**. `descriptor`/`sparrow` formats correctly preserve
  `multi`. _Fix: refuse unsorted multi for electrum/coldcard/jade._
- **H12-crossmode (HIGH, new facet of H12)** — same seeds/family: template-mode emits `3'` (correct,
  `bcrt1p20ad3q…`), descriptor-mode emits `2'` (`bcrt1pe8q8h9…`); `--multisig-path-family bip48` is rejected
  in descriptor-mode (no escape hatch). One toolkit, two non-cosignable wallets. Same fix as H12.
- **H15 (→ MEDIUM, corrupt-input-only)** — mainnet xpub (`0488b21e`) on a `84'/1'/0'` path imports as
  `network=testnet` → witness program `c0cebcd6…` rendered `tb1qcr8te4…` instead of true `bc1qcr8te4…`.
  Legit network-consistent round-trips are a **clean-negative** (correct), so this only bites corrupt/edited
  blobs — empirically MEDIUM, not HIGH. (`descriptor.rs`/`sparrow.rs` network-from-coin-type gap.)
- **Decaying-multisig (the 2 W3/final downgraded items) — both empirically reproduced:** tier-3 absolute
  `after(T)` never validated (past height → last-resort key immediately spendable); decay-ordering compares
  raw BIP-68 operands without the 512-sec unit flag (`--recovery-older 4194305` unlocks BEFORE `--older 145`).

## Empirically DEMOTED to metadata-only (addresses correct — NOT wrong-address)
The oracle proved these reproduce their static mechanism but the derived **addresses are identical**, so
they are fidelity/PSBT-matching/availability bugs, not wrong-address funds-loss:
- **H14** — coldcard-multisig uses each account xpub's own depth-4 fp as the master fp, but addresses are
  identical (the xpub is correct). Real (breaks PSBT device-matching) but **not** wrong-address.
- **H11** — coldcard/jade path-collapse to `m/0'/0'`: origins corrupted, watch-only addresses unchanged.
- **M7** — `bundle --json` threshold=N: metadata-only (embedded descriptor + md1 correct).
- **M1 / L3** — `--from-import-json` account→0 / coldcard `as u32` account truncation: origin metadata wrong,
  addresses correct.
- **M3** — chain-gate: change addresses **underivable** (fail-closed), not wrong-address.
- **M10** — BIP-86 depth-3 taproot **false-reject** (availability), not wrong-address.

## Clean-negatives worth keeping (raise confidence / scope the fixes)
- multi_a tap-leaf order **is** preserved & order-significant through bundle→restore; BIP-86 single-key uses
  correct `86'/0'/0'`; export template-mode correctly emits `3'` — so **H12 is specifically the
  descriptor-mode default-origin bug**, not broader taproot breakage.
- non-hardened `<0;1>` multipath is wire-correct for receive AND change → **H13 is hardened-specific**.
- legitimate network-consistent single-sig round-trips match the oracle exactly across all script
  types/accounts/networks → **H15 is corrupt-input-only**.

_(2 downgraded findings counted in stats but not detailed in the wave output; non-critical.)_
