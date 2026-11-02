# Bug report: bg002h/mnemonic-toolkit @ c0a99df

Scope: focused hunt over 35 files of about 40,000 lines in 17 slices: the parsers and codecs, the whole `wc-codec` crate, and the `restore`, `verify_bundle` and `bundle` commands. 17 readers and 17 reviewers read the code, and 16 provers checked the candidates; none of them failed.

**Important caveat:** the workspace would not build in the sandbox. It is pinned to toolchain 1.85.0 and has a git dependency on rust-miniscript, and neither could be fetched offline. So only one bug (#7) was shown with a test that ran. The others were confirmed by a second agent reading the code independently, and their tests were written but never run. The suggested fixes have not been tested except for #7. Test files are under `/tmp/hunt/prover-N/` in the sandbox.

## Confirmed bugs (worst first)

### High

**1. `restore` skips `--expect-xpub` when `--expect-fingerprint` is also given** (confirmed by reading)
- Where: `cmd/restore.rs` `run`, lines 609-627. The checks are an `if let … else if let` chain.
- Input: `restore --from phrase=<seed> --template bip84 --expect-fingerprint 73c5da0a --expect-xpub <wrong xpub>`, for example the right seed at the wrong account.
- Should: exit 4 with MISMATCH (xpub).
- Does: exits 0 with status "verified".
- Test: `prover-1/.../tests/cli_restore.rs::restore_fp_match_but_xpub_mismatch_must_fail`.
- Fix: run the fingerprint check, then the xpub check, and drop the `else`.

**2. `restore` with a keyless single-sig template md1 ignores `--expect-fingerprint`, `--expect-xpub`, `--allow-mismatch` and `--format`** (confirmed by reading)
- Where: `cmd/restore.rs` `run_singlesig_template_completion`, reached from line 348. The function never reads these arguments.
- Input: `restore --md1 <bip84 template> --from phrase=<seed> --expect-fingerprint deadbeef`
- Should: exit 4 with MISMATCH.
- Does: exits 0 and prints the descriptor.
- Test: `prover-8/.../tests/prover8_expect_fp.rs`.
- Fix: add the same mismatch block the normal single-sig path uses, or refuse these flags in template mode.

### Medium

**3. A descriptor with no `/*` wildcard is accepted and encoded as a ranged wallet** (confirmed by reading)
- Where: `parse_descriptor.rs` `lex_placeholders`, around line 180. The `wild` capture group is optional and whether it matched is never stored.
- Input: `parse_descriptor("wpkh(@0/<0;1>)")` or `"wpkh(@0)"`.
- Should: return `Err(DescriptorParse)`. The code's own comments say md1 cannot represent this shape and must refuse it.
- Does: returns Ok, encoded the same as `@0/<0;1>/*`, so the backup describes a different wallet.
- Fix: reject the token when `caps.name("wild").is_none()`.

**4. Electrum normalization misses Hangul Jamo and other CJK ranges, so the seed is wrong** (confirmed by reading, NFKD output checked with Python)
- Where: `electrum.rs` `is_cjk`, line 287.
- Background: NFKD runs first and splits Hangul syllables into U+1100–11FF, which the table doesn't contain. 0x3000–303F, 0x3130–318F, Bopomofo and the fullwidth forms are also missing.
- Input: the passphrase `"한 글"`.
- Should: give the same seed as `"한글"`, because Electrum strips the space.
- Does: keeps the space, so it derives a different seed from Electrum's, with no error.
- Fix: copy Electrum's full `CJK_INTERVALS` list.

**5. Electrum export of single-sig taproot (bip86) writes a plain xpub, which Electrum reads as p2pkh** (confirmed by reading)
- Where: `wallet_export/electrum.rs` `render_slip132_xpub`, line 203.
- Input: `export-wallet --format electrum --template bip86 …`
- Should: refuse, as it already does for taproot multisig.
- Does: writes a "standard" wallet that shows the wrong (legacy) addresses.
- Fix: return an error for `P2tr` in `emit`.

**6. Descriptor-mode `bundle` doesn't check the `[fp/…]@N` annotation for `entropy` and `ms1` slots** (confirmed by reading)
- Where: `cmd/bundle.rs` `bundle_run_unified_descriptor`, around lines 1660-1710. The Phrase arm does this check at lines 1581-1587.
- Input: `--descriptor "wsh(sortedmulti(2,[deadbeef/48'/0'/0'/2']@0,@1))" --slot @0.entropy=00…00 --slot @1.xpub=…`
- Should: fail with an error about the fingerprint mismatch.
- Does: succeeds; the annotation is silently ignored and the bundle records the derived fingerprint `73c5da0a`.
- Fix: copy the Phrase arm's `anno != master_fp` check into both arms.

**7. `wc-codec` sync: `classify_single_insertion` can return a misaligned grid labelled `Aligned`** (**demonstrated**)
- Where: `wc-codec/src/sync.rs`, line 658. The H1 branch accepts the first checkpoint whose marker and index parse, with no CRC check and no check of later checkpoints.
- Input: k=16, `data[i]=(i*37+3)&0xFF`, `data[4]=0x500`. Take `interleave(data)`, flip bit 0 of word 0, then insert the word 7 at index 12.
- Should: either refuse, or erase every wrong position.
- Does: returns Aligned with erasures [0..3], while positions 4–11 are wrong and not erased.
- Test: `prover-11/standalone/repro.rs`. It fails on the current code and passes with the fix.
- Fix: in the H1 branch, after the removal, check that every later checkpoint has the right position, index and crc5; if any fails, return `Refuse(AmbiguousRealignment)`. The fix was run and passes. Note that it also refuses an insertion that comes with real damage in a later block.

**8. `wc-codec` encode accepts `u_slots >= 8`, but the header stores U in only 3 bits** (confirmed by reading)
- Where: `pipeline.rs` `encode_inner`, line 676.
- Input: `EncodeOpts{u_slots: 8, ..}`.
- Should: return `Err(InvalidParams)`, or produce a card that decodes.
- Does: the header records U=0 while 16 ledger words are written, so the card can never be decoded.
- Fix: reject `u_slots > 7`.

**9. Descriptor-mode `bundle` silently drops an unparseable `--slot @N.fingerprint=`** (confirmed by reading)
- Where: `cmd/bundle.rs`, line 1618. The parse result goes through `.ok()`.
- Input: `--slot @0.fingerprint=zzzz` with `@0.xpub`.
- Should: fail with a BadInput error, as template mode does.
- Does: uses the annotation's fingerprint, or 00000000.
- Fix: map the parse error to `ToolkitError::BadInput` and use `?` instead of `.ok()`.

**10. `restore` with a keyless multisig template ignores `--output FILE`** (confirmed by reading)
- Where: `cmd/restore.rs` `emit_completed_multisig` (around line 3070) and `emit_candidate_list` (around line 1598).
- Should: write the output to FILE.
- Does: prints it to stdout, never creates FILE, and exits 0.
- Fix: route the output the same way as line 775 does.

**11. A single-sig descriptor with a hardened use-site passes the template gate** (confirmed by reading)
- Where: `synthesize.rs` `template_admissible`, line 987. The `n==1` branch returns early, before `has_hardened_use_site` is checked.
- Input: `wpkh(@0/<0;1>/*h)` with `--md1-form=template`.
- Should: fail with TemplateFormUnsupportedShape.
- Does: emits a template that can't be restored.
- Fix: move the hardened-use-site check above the `n==1` branch.

### Low

**12. `verify_bundle`: one stray mk1 card blames the wrong cosigner slots** (confirmed by reading)
- Where: `cmd/verify_bundle.rs` `emit_multisig_checks`, line 2777.
- Does: a stray card marks the first unfilled slot XpubNotInPolicy. That slot's correct card can then no longer claim it, so each correct card in turn is reported as XpubNotInPolicy.
- Fix: place the matching cards in one pass first, then mark the leftover slots.

**13. `verify_bundle`: the positional fallback orders cards by chunk_set_id instead of the order the user supplied them** (confirmed by reading)
- Where: around lines 2668-2677 and 2803.
- Does: correct cards get false mk1_*_match failures.
- Fix: group the cards in first-seen order instead of in a BTreeMap.

**14. `verify_bundle`: the multisig `--json` output is missing the `md1_origin_match` row when the md1 doesn't decode** (confirmed by reading)
- Where: around line 3284.
- Does: the check list has a different length on that branch, although the code comment promises a fixed list.
- Fix: push a "skipped" row in the Err arm.

**15. `bip85` dice: a huge `--length` aborts the process** (confirmed by reading, plus a standalone reproduction)
- Where: `bip85.rs` `format_dice_rolls`, line 254.
- Input: `derive-child --application dice --length 2000000000`.
- Does: `Vec::with_capacity` asks for 48 GB and the process aborts with SIGABRT.
- Fix: limit the up-front capacity and check the length in `derive_child`.

## Rejected
- **Canonical-mode guard leaves out entropy+path** (`bundle.rs:1473`). Rejected because this input can never reach the guard: `validate_slot_set` refuses entropy+path earlier.

## Not checked
These 34 suspicions were rated 2 or lower and were not sent to provers:
- **`parse_descriptor`:** multipath `<5>` and `<0;0>` accepted; `unwrap_or(master)` swallows errors; extra key sources silently ignored.
- **`verify_bundle`:** strict versus partial decode of the md1, which skips checks; the origin comparison ignores the master fingerprint; a slice that panics on a short template_id.
- **`restore`:** extra values of a `--account` list silently dropped; `--origin` combined with account.
- **`repair`:** mixed-case mk1 chunks accepted.
- **`bundle`:** seedqr slot not supported in descriptor mode; the `--import-json` overlay uses only phrases.
- **`permutation_search`:** u64 total silently capped; address_index dropped from results.
- **`synthesize`:** stale origin overrides carried over; a template with n==0 panics.
- **`wc-codec`:** `regroup` accepts symbols past the payload length when they are zero; sync deletion re-anchoring has no CRC check; a trailing extra word is refused; `raid` checks the padding only in debug builds; the version field is ignored; u=0 is accepted on decode.
- **`slip39`:** case-sensitive word lookup; Feistel panics on shares longer than 32 bytes; several advisory-message nits.
- **`electrum_decrypt`:** a password file with CRLF line endings breaks decryption.
- **`electrum` and `electrum_crypto`:** Mark-category stripping differs from Electrum's; zero entropy renders as "abandon"; BIE1 input not trimmed.
- **Electrum export:** single-sig ignores the slot's path; k is not checked; cosigner labels are "-1", "-2" and so on.

## Coverage notes
- Readers and reviewers read all of the non-test code. Some skipped test modules, which were out of scope.
- The second readings were partial in places:
  - `verify_bundle.rs`: about 1,080 of 5,019 lines.
  - `restore.rs`: about 700 of 4,733 lines.
  - `bundle.rs`: about 600 of 3,310 lines.
- Files outside the focus (wallet import/export, `convert`, the descriptor builder, and others, about 53,000 lines) were not read.

---

# Parts 2 and 3: wallet import/export and all remaining code

Scope: the 115 files (about 56,000 lines) not read in part 1, split into 45 slices. One run used 45 readers and 45 reviewers to read the code, then 24 provers plus 4 retries to check the candidates. This time provers used standalone `rustc` or Python copies of the code in question, so **most bugs below were demonstrated by a failing test** (the full crate still couldn't build offline). Tests are under `/tmp/hunt2/prover-N/` in the sandbox.

Numbering continues from part 1.

## Confirmed bugs (worst first)

### High

**16. Bitcoin Core import loses the threshold for taproot multisig** (demonstrated)
- Where: `wallet_import/bitcoin_core.rs` `extract_threshold`, line 905. The regex `(?:thresh|multi|sortedmulti)\((\d+)` doesn't match `multi_a(` or `sortedmulti_a(`.
- Input: `tr(K, sortedmulti_a(2, A, B, C))`.
- Should: threshold = Some(2).
- Does: returns None, with no error.
- Test: `prover-12/test_threshold.py`.
- Fix: change the regex to `(?:thresh|multi|sortedmulti)(?:_a)?\((\d+)\s*,`. `bsms::extract_threshold` is documented as mirroring this function, so check it too.

**17. Sparrow import has the same taproot threshold bug** (demonstrated)
- Where: `wallet_import/sparrow.rs` `extract_threshold_local`, line 651. Same cause and fix as #16.
- Test: `prover-4/t.py`.

### Medium

**18. BSMS Round-1 signature check fails on correctly signed records** (confirmed by reading)
- Where: `wallet_import/bsms_round1.rs` `signed_body` / `key_field_to_line`, line 174. Line 3 of the signed text is rebuilt from the parsed key instead of being kept as written.
- What changes in the rebuild:
  - The fingerprint is lowercased (ABCD1234 → abcd1234).
  - `h` becomes `'`.
  - An empty path `[fp/]` turns into `[fp/m]`. This breaks even records that are already in canonical form.
- Should: verify the signature against the bytes the signer actually signed.
- Does: rejects a valid signature with BsmsSignatureMismatch.
- Fix: store the original line 3 text in the record and use it in `signed_body`.

**19. Coldcard multisig import rejects every legacy P2SH (BIP-45, `m/45'`) export** (demonstrated)
- Where: `wallet_import/coldcard_multisig.rs` `coin_type_index`, around line 745. It requires at least 2 path components.
- Input: `Derivation: m/45'` with `Format: P2SH`.
- Should: parse the file.
- Does: fails with "derivation path has only 1 components".
- Test: `prover-21/t.rs`.
- Fix: handle `[45']` before the length check, or take the network from the xpub version.

**20. Sparrow export gives every keystore the same label** (demonstrated)
- Where: `wallet_export/sparrow.rs`, line 129: every keystore gets `label: wallet_name`.
- Does: Sparrow rejects duplicate labels when it imports a multisig wallet.
- Test: `prover-20/t.rs`.
- Fix: number the labels in multisig, e.g. `"{name} {i+1}"`.

**21. BIP-388 export fails for taproot multisig with a NUMS internal key** (demonstrated)
- Where: `wallet_export/pipeline.rs` `descriptor_to_bip388_wallet_policy`, line 208. The bare NUMS internal key has no `/<0;1>/*`, so the conversion refuses it.
- Does: the toolkit can't export its own `--taproot-internal-key nums` output, and `build-descriptor` hits the same refusal.
- Test: `prover-19/t.rs`.
- Fix: skip the NUMS key (or any single key with no origin) in the key loop.

**22. Specter export with `--descriptor` lists 1 device for an N-cosigner wallet** (demonstrated)
- Where: `wallet_export/specter.rs`, line 62: `resolved_slots.len().max(1)`, and that list is empty in descriptor mode.
- Input: `wsh(sortedmulti(2,K1,K2,K3))`.
- Should: 3 devices.
- Does: 1 device, with no error.
- Test: `prover-11/t.rs`.
- Fix: count the keys from the descriptor.

**23. Electrum import mangles wallet names that end in digits** (demonstrated)
- Where: `wallet_import/electrum.rs`, around lines 833-841. The code trims every trailing digit, not just a `-N` suffix.
- Input and result: "Savings 2024" becomes "Savings ", "Vault2" becomes "Vault", and "2024" is dropped entirely.
- Test: `prover-14/t.rs`.
- Fix: strip only a `-<digits>` suffix, using `rsplit_once('-')`.

**24. xpub-search assumes the wrong default path for nested-segwit multisig** (confirmed by reading, plus a standalone model)
- Where: `cmd/xpub_search/descriptor_intake.rs`, around lines 307-311. Only `tr` is special-cased (3'); every other descriptor gets 2'.
- Input: `sh(wsh(sortedmulti(...)))` with no origin annotations.
- Should: BIP-48 script type 1', i.e. `m/48'/c'/a'/1'`.
- Does: assumes `.../2'`, so the search looks at the wrong paths.
- Fix: add an `Sh(Wsh)` arm that returns 1.

**25. `convert` with `--from minikey=@env:VAR` never reads the environment variable** (demonstrated)
- Where: `cmd/convert.rs`, lines 1903 and 1930. These use `is_secret_bearing`, which leaves out MiniKey.
- Does: the literal `@env:K` is passed on and rejected as an invalid mini-key.
- Test: `prover-2/t.rs`.
- Fix: use `is_argv_secret_bearing` in both places.

**26. `convert` phrase→address prints a false "--passphrase ignored" warning** (demonstrated)
- Where: `cmd/convert.rs` `edge_uses_pbkdf2`, around line 977. `Address` is missing from the list of target types.
- Does: the warning says the passphrase is ignored, but the address is in fact derived with it. A user who believes the warning may record the address against the wrong wallet.
- Test: `prover-1/t.rs`.
- Fix: add `NodeType::Address` to the list.

**27. compare-cost corrupts hash arguments when converting taproot leaves** (demonstrated)
- Where: `cost/strip.rs` `inflate_xonly_to_compressed_even_y`, line 217. It puts `02` in front of every 64-hex run, including `sha256(...)` and `hash256(...)` arguments.
- Does: a valid `tr(..., and_v(v:pk(K), sha256(H)))` is rejected with a ContextIncompat error.
- Test: `prover-3/t.rs`.
- Fix: inflate key arguments only.

**28. compare-cost counts a repeated hash twice** (demonstrated)
- Where: `cost/enumerate.rs` `walk_segv0_for_hash_leaves`, line 443. Hashes are not deduplicated, though keys are.
- Input: `or_d(and_v(v:pk(A),sha256(H)),and_v(v:pk(B),sha256(H)))`.
- Should: 2 spending conditions, with the 2^n precheck counting 8.
- Does: 4 conditions, each listed twice, and the precheck counts 16, so it can wrongly report ConditionsTooMany.
- Test: `prover-10/t.rs`.
- Fix: push each hash only on its first occurrence.

**29. Coldcard single-sig import silently turns a non-integer `account` into 0** (demonstrated)
- Where: `wallet_import/coldcard.rs`, line 245. `as_u64()` returns None for `"5"`, -1 or 1.5, and None becomes 0.
- Does: the origin becomes `m/44'/0'/0'`, though the code's own comment says it must never silently rewrite an account index.
- Test: `prover-23/t.rs`.
- Fix: reject a value that is present but not a non-negative integer.

### Low

**30. Coldcard `xfp` with multibyte characters panics** (demonstrated)
- Where: `wallet_import/coldcard.rs` `parse_xfp_hex`, line 406.
- Input: `"aéééb"`, which is 8 bytes long.
- Does: panics on a byte slice that isn't on a character boundary, instead of returning an error.
- Test: `prover-7/t.rs`.

**31. The same parser accepts `+` signs in `xfp`** (demonstrated)
- Input: `"+a+b+c+d"`.
- Does: accepted as `0a0b0c0d`.
- Test: `prover-22/t.rs`.
- Fix for #30 and #31: require `s.bytes().all(|b| b.is_ascii_hexdigit())` after the length check.

**32. xpub-search panics for account values of 2^31 or more** (demonstrated)
- Where: `cmd/xpub_search/candidate_paths.rs`, line 172, and `descriptor_intake.rs` `bip48_default_path`, line 434. Both call `.expect()` on a path that isn't valid.
- Input: `--min-account 2147483648`, alone or with `--number-of-accounts 0`.
- Should: a clean BadInput error.
- Does: panics.
- Tests: `prover-8/repro.rs`, `prover-9b/repro.rs`.
- Fix: check that the account values are below 2^31 when the arguments are parsed.

**33. GUI schema wrongly requires `--template` for `bundle --import-json`** (demonstrated)
- Where: `cmd/gui_schema.rs` `bundle_conditional_rules`, around line 300. The rule doesn't match clap's `required_unless_present_any` in `bundle.rs:29`.
- Test: `prover-5/t.rs`.
- Fix: add `--import-json` to the rule's list.

**34. GUI schema wrongly requires `--template` for verify-bundle** (demonstrated)
- Where: `cmd/gui_schema.rs` `verify_bundle_conditional_rules`, around line 480. The rule misses `--md1`, `--bundle-json` and `--extra-strings`.
- Test: `prover-6/t.rs`.
- Fix: add those three flags to the rule's list.

**35. `cost/translate.rs` sets `concrete_keys` wrong for mixed input** (demonstrated)
- Where: line 48: `concrete_keys = labels.is_empty()`.
- Does: false when abstract labels and hex keys are mixed, though the doc says it should be true if there is any hex key.
- Test: `prover-17/t.rs`.

**36. `friendly.rs` error message shows the wrong length set** (demonstrated with a stand-in for the error type)
- Where: line 118. The `UnexpectedStringLength` message always shows the entr length set.
- Does: for mnem-kind strings it can print "length 50 not in [50, …]".
- Test: `prover-18/t.rs`.
- Fix: print the error's `allowed` field.

## Rejected
- **`export-wallet --threshold 0` not refused** (`export_wallet.rs:675`). Rejected after two provers: miniscript later refuses `multi(0,…)`, so no wallet is ever built and only the error message is vaguer.

## Coverage gaps in parts 2 and 3
- **12 readers and 8 reviewers failed.** I'm not told why agents fail.
- **No agent read these three slices at all:**
  - Slice 13: `cmd/xpub_search/{mod, passphrase_of_xpub, passphrase_search, path_of_xpub, path_search, seed_intake}.rs`.
  - Slice 28: `slot_ms1.rs`, `taproot_override_classify.rs`, `template.rs`.
  - Slice 42: `wallet_import/pipeline.rs`, `wallet_import/sniff.rs`.
- **Read once, by the reviewer only (the reader failed):** slices 1, 9, 10, 11, 21, 22, 27, 30, 36.
- **Read once, by the reader only (the reviewer failed):** slices 16, 19, 20, 33, 44.
- **Read only partly:** `import_wallet.rs` had about 1,400 of 2,920 lines re-read.
- **Not checked:** 76 suspicions rated 2 or lower were not sent to provers. They are listed in the run result and include:
  - u8 count wraparounds at 256 cosigners or slots.
  - Byte-slice panics in `format.rs:269`.
  - A config file losing lines that come before the first section when it is rewritten.
  - Coldcard export filling `xfp` and `deriv` from the wrong source.
  - CRLF in password files.
  - Several mismatches between help text and code.

---

# Follow-up: the three slices left unread in parts 2 and 3

Scope: the 11 files (about 3,800 lines) that no agent read in the part 2/3 run:
- `cmd/xpub_search/{mod, passphrase_of_xpub, passphrase_search, path_of_xpub, path_search, seed_intake}.rs`
- `slot_ms1.rs`, `taproot_override_classify.rs`, `template.rs`
- `wallet_import/pipeline.rs`, `wallet_import/sniff.rs`

Each slice had 1 reader and 1 reviewer, and all 6 agents finished. They read every non-test line; the reviewers also read most or all of the test code.

**Result: no confirmed bugs.** No suspicion was rated 3 or higher, so none was sent to a prover and nothing was rejected.

**Not checked:** these 9 suspicions were rated 2 or lower:
- `passphrase_search.rs:268`: one candidate line that isn't valid UTF-8 aborts the whole search.
- `passphrase_search.rs:274`: blank lines are skipped, so the empty passphrase can never be a candidate.
- `passphrase_search.rs:113`: a candidates file that is empty or only blank lines reports "exhausted" (exit 4) instead of bad input.
- `wallet_import/pipeline.rs:403`: the u8 key counter overflows on the 256th key.
- `wallet_import/pipeline.rs:67`: x-only keys in `multi_a` aren't detected, so the user gets the wrong refusal message.
- `wallet_import/pipeline.rs:292`: only the `@N/**` form of a BIP-388 template is accepted; `@N/<0;1>/*` is refused.
- `template.rs:296`: a bare `sh(multi)` is mapped to BIP-48 script type 1'.
- `template.rs:168`: `n as u8` truncates the key count above 255.
- `taproot_override_classify.rs:115`: a redundant condition, and the doc describes the check inaccurately.

**Overall coverage after this follow-up:** every non-test source file has now been read by at least one agent. Fourteen slices from parts 2 and 3 were still read by only one agent (listed in the part 2/3 coverage section).

---

# Coverage follow-up (second reads)

**Scope:**
- **Second reads** of the 14 part-2/3 slices that only one agent had read.
- **Full reader-plus-reviewer reads** of the three big command files whose second reads in part 1 were only partial:
  - `cmd/verify_bundle.rs` lines 1-4,015
  - `cmd/restore.rs` lines 1-4,446
  - `cmd/bundle.rs` lines 1-2,881

**Coverage:**
- Every range of the three big files now has at least one full read, and most have two. The readers of `verify_bundle.rs` 1341-2680 and `restore.rs` 1-1490 failed (I'm not told why); the reviewers read those ranges in full.
- Of the 14 single-read slices, 12 now have a second read. The readers for **slice 10** (`cmd/nostr.rs`, `cmd/seed_xor.rs`, `cmd/silent_payment.rs`, `cmd/verify_message.rs`) and **slice 33** (`wallet_import/bitcoin_core.rs`) failed, so those two are still read once.

**Result:** 20 suspicions were rated 3 or higher and all 20 were confirmed. One duplicates an existing entry (#14, the missing `md1_origin_match` row), which leaves **19 new bugs**: 14 demonstrated, 5 confirmed by reading. Most were demonstrated with standalone copies or Python models of the logic, because the workspace doesn't build offline. Tests are under `/tmp/hunt-fu/prover-N/` in the sandbox.

## High

**37. `verify-bundle` on a single-sig template never checks the `--ms1` card** (confirmed by reading)
- Where: `cmd/verify_bundle.rs` `verify_singlesig_template`, lines 709-958. `args.ms1` and `expected.ms1` are never used, although the doc says the step "compares the supplied md1/mk1/ms1".
- Input: a template bundle with the correct md1 and mk1 and an `--ms1` from a different seed.
- Should: a failed check and exit 4.
- Does: "OK", exit 0. A wrong seed card passes verification.
- Fix: add an `ms1_match` check that compares against the expected ms1 (or its entropy).

## Medium

**38. `restore` of a taproot `multi_a` template ignores a non-standard use-site path** (confirmed by reading, with a Python model)
- Where: `cmd/restore.rs` `classify_taproot_restore` / `run_multisig`, around line 3247. The Template arm goes to `build_descriptor_string`, which hard-codes `/<0;1>/*` (`pipeline.rs:96`). The wsh path does fall back to a faithful restore for a non-standard use-site; the taproot path has no such check.
- Input: a `tr(NUMS,multi_a(2,@0,@1,@2))` card whose baseline use-site is `/*` or `/<2;3>/*`, with no overrides.
- Does: restores a **different wallet**, with different addresses, and exits 0.
- Test: `prover-5/test_tr_usesite.py`.

**39. `verify-bundle` single-sig `md1_xpub_match` compares only the first pubkey** (demonstrated)
- Where: `verify_bundle.rs` `emit_md1_checks`, line 3364.
- Input: expected `wpkh(@0/<0;1>/*)`; supplied `pkh(...)` or `wpkh(@0/<2;3>/*)` with the same xpub.
- Does: passes. The multisig path's comparison of the tree and use-site path (the H1 fix) is missing here.
- Test: `prover-12/t.rs`.

**40. `restore` multisig template: own accounts use account 0 when the first cosigner is a bare xpub** (demonstrated with a Python model)
- Where: `restore.rs` `complete_multisig_template`, line 1936. The fallback skips the `own_origin_from_family(&fb, acct)` step that the all-own branch applies.
- Does: `--account 1` derives at account 0', so no match is found and nothing explains why; `--account 0,1` derives the same key twice.
- Test: `prover-13/t.py`.

**41. `restore` with `@N=` placement silently ignores `--search-address`** (confirmed by reading)
- Where: `restore.rs`, lines 1828 and 1999-2007. `complete_explicit_assignment` is only given `expect_wallet_id`.
- Does: a swapped cosigner placement prints a different wallet and exits 0, even though the user also gave the true address.
- Fix: check the address, or refuse the combination.

**42. `restore` marks the user's own slot "verified" without checking its fingerprint** (confirmed by reading)
- Where: `restore.rs`, line 4088. Only the 65-byte chain code and pubkey are compared; `acct.master_fingerprint` is never compared with the engraved fingerprint.
- Does: a wrong engraved fingerprint is labelled "← your seed (verified)", and PSBT signers won't recognise that key.

**43. `verify-bundle` crashes on a supplied md1 with more cosigners than expected** (demonstrated)
- Where: `verify_bundle.rs` `emit_multisig_checks`, lines 2762 and 2772. `card_for_cosigner[gi]` and `[slot]` are indexed with no bound check against n.
- Input: a 2-cosigner expected bundle, and a supplied 3-cosigner md1 with an mk1 at slot 2.
- Does: panics with "index out of bounds".
- Test: `prover-4/t.rs`.

**44. `verify-bundle` in descriptor mode ignores the `[fp/path]@N` annotation fingerprint** (demonstrated)
- Where: `verify_bundle.rs` `descriptor_mode_verify_run`, line 1759. It falls back to 00000000, while `bundle` uses `slot_fp.or(anno_fp)`.
- Does: a correct bundle fails verification (exit 4), and `bundle`'s annotation-agreement check (H7) is skipped.
- Test: `prover-18/t.rs`.

**45. `bundle --import-json` labels testnet and signet bundles "mainnet"** (demonstrated)
- Where: `cmd/bundle.rs` `bundle_run_from_import_json`, line 2200. `emit_args` never takes the envelope's network.
- Does: keys are derived for testnet, but the JSON and engraving card say mainnet.
- Test: `prover-16/t.rs`.
- Fix: `emit_args.network = envelope_network`.

**46. `bundle --import-json` silently drops every `--slot` overlay except `@N.phrase=`** (demonstrated)
- Where: `bundle.rs`, line 2070.
- Input: `--slot @1.entropy=…` or `@1.ms1=…`.
- Does: the slot stays watch-only and the command exits 0. This was a rated-2 "not checked" item in parts 2/3; it is now confirmed.
- Test: `prover-22/t.rs`.

**47. An empty `--expect-wallet-id` matches any wallet** (demonstrated with a Python model)
- Where: `verify_bundle.rs` `verify_singlesig_template`, line 891. `hex::decode("")` gives an empty prefix, which matches every id.
- Input: `--expect-wallet-id ""`, for example from an unset shell variable.
- Does: `wallet_id_match` passes, with only an advisory on stderr.
- Test: `prover-17/t.py`.
- Fix: reject an empty prefix. The restore path should be checked for the same logic.

**48. `verify-message` legacy path rejects a valid signature that has a trailing newline** (demonstrated)
- Where: `verify_message.rs` `verify_legacy`, line 62. The BIP-322 path trims the signature; this path does not.
- Input: a P2PKH address, the correct message, and a signature followed by `\n`, e.g. read from a file.
- Does: fails with "not a valid base64 signature".
- Test: `prover-8/t.rs`.
- Fix: `.trim()`.

**49. Coldcard single-sig import treats the top-level master `xpub` as the account xpub** (demonstrated with a Python model)
- Where: `wallet_import/coldcard.rs` `select_dominant_bip`, line 460.
- Input: a blob with only `bip48_*` multisig blocks and a top-level `xpub`.
- Does: emits `pkh([fp/44'/0'/0']<master xpub>/…)`, which gives wrong addresses with no error. The module doc says that field is the master xpub.
- Test: `prover-9/t.py`.

**50. `word-card --from` never groups multi-chunk cards, though its help says it does** (demonstrated)
- Where: `cmd/word_card.rs`, line 197.
- Input: two `--from` values, or two stdin lines, that are chunks of one mk1 card.
- Does: decodes each chunk alone, giving "incomplete chunk set".
- Test: `prover-7/t.rs`.

## Low

**51. `inspect` decodes only the first ms1 and ignores the rest** (demonstrated)
- Where: `cmd/inspect.rs`, line 220.
- Input: `mnemonic inspect <ms1A> <ms1B>`.
- Does: ms1B is never decoded or checked. (`prover-6`)

**52. The mk1 template-stub check ignores an unparseable `--mk1`** (demonstrated)
- Where: `verify_bundle.rs` `check_mk1_template_stubs`, line 1224. `filter_map` drops cards whose chunk-set id can't be read.
- Does: the correct stubs plus a garbage card still pass. (`prover-10`)

**53. Taproot `multi_a` with a repeated key index restores a different leaf** (confirmed by reading)
- Where: `restore.rs`, line 3247.
- Input: `tr(NUMS,multi_a(2,@0,@1,@1))`.
- Does: restored as `multi_a(2,K0,K1)` with no refusal. Standard wallets can't create such a policy.

**54. `bundle --json` multisig `path_family` is always "bip87" in descriptor mode** (demonstrated)
- Where: `bundle.rs`, line 972.
- Does: a BIP-48 descriptor (`m/48h/…`) is reported as bip87. (`prover-15`)

**55. Duplicate:** the run re-confirmed the missing `md1_origin_match` row when the multisig md1 fails to decode. This is the same bug as **#14** above (`prover-19`).

**Not checked:** suspicions rated 2 or lower. The notable ones:
- **`format.rs:335`:** a second place that cuts the descriptor at byte 60 (`&d[..60]`), which can panic in the middle of a multibyte character.
- **`slot_input.rs:154`:** `@+0` and `@00` are accepted as slot indices.
- **`bitcoin_core.rs:81` (export):** `range` and `active` are emitted for un-ranged descriptors, which Bitcoin Core rejects.
- **`bip388.rs:115`:** the `Cosigner(idx)` index is never bounds-checked.
- **`coldcard.rs`:** the network and `deriv` fields are never checked.
- **`error.rs:481`:** the ms_codec `TooManyErrors` error exits 1 instead of 2.
- **`final_word.rs:136`:** the word check skips Unicode normalization.
- **`descriptor_builder/ir.rs:330`:** wrap and key strings are pasted unescaped into miniscript.
- **`verify_bundle.rs`:**
  - A bad `--slot @N.fingerprint` silently becomes 00000000 (`:1758`).
  - The default threshold is 1, while `bundle` defaults to n (`:1458`).
  - Stub slot order is ignored (`:1227`).
- **`restore.rs`:**
  - `--origin m` mislabels the master key (`:968`).
  - The keyless single-sig use-site is ignored (`:1037`).
  - Explicit mode applies no strength check to the wallet-id prefix (`:2934`).
  - Bare xpub cosigners ignore the network (`:2480`).
- **`bundle.rs`:**
  - Envelope ms1 entropy is never checked against the slot's xpub (`:2062`).
  - A slot fingerprint is ignored in the phrase arms (`:1559`).
  - The JSON `descriptor` field echoes the raw input (`:991`).

---

# Closing run: the last single-read slices

**Scope:** slice 10 (`cmd/nostr.rs`, `cmd/seed_xor.rs`, `cmd/silent_payment.rs`, `cmd/verify_message.rs`) and slice 33 (`wallet_import/bitcoin_core.rs`). Each had two independent readers, and **all 4 finished.**

**Result:** 3 suspicions were rated 3 or higher, and **all 3 were confirmed**.

## High

**56. `silent-payment` accepts any xprv as the master key** (confirmed by reading)
- Where: `cmd/silent_payment.rs` `resolve_master_xpriv`, line 126. The depth is never checked.
- Input: `--secret <account-level xprv>`, e.g. the key at m/84'/0'/0'.
- Should: refuse a key that isn't a master (depth 0). The help says "the xprv IS the master".
- Does: derives m/352'/… **underneath the account node** and prints an `sp1` address, with scan and spend keys, that no BIP-352 wallet would derive from the seed. **Funds sent to that address go to keys the user's wallet doesn't recognise**, and there is no warning.
- Fix: `if xpriv.depth != 0 { return Err(...) }`.

## Medium

**57. Bitcoin Core import merges receive and change descriptors that have different scripts** (demonstrated with a Python model)
- Where: `wallet_import/bitcoin_core.rs` `analyze_merge_candidate` / `merge_receive_change_pairs`, lines 387 and 519. The grouping key has no field for the script structure, and the merged result is built from the receive body only.
- Input: `wsh(pk(A/0/*))` with `internal:false`, plus `wsh(pkh(A/1/*))` with `internal:true`.
- Should: not merge, because the doc and SPEC §4.1 say the grouping covers everything except the final derivation step.
- Does: produces one `wsh(pk(A/<0;1>/*))`. **The change script is silently thrown away.**
- Test: `/tmp/hunt-close/prover-9/test_merge.py`.

## Low

**58. Bitcoin Core import error messages name the wrong `descriptors[N]` after a merge** (demonstrated with a Python model)
- Where: `bitcoin_core.rs`, line 227. The loop index over the merged list is passed on as the original index.
- Test: `prover-8/t.py`.

**Not checked:** suspicions rated 2 or lower:
- `cmd/seed_xor.rs:276`: `combine` may accept a single share and echo it back.
- `bitcoin_core.rs:538`: the near-miss check across groups may refuse ordinary multi-wallet exports.

## Final coverage
**Every slice of the toolkit (about 93,000 lines) has now been read by at least two independent agents.** That includes the three big command files (`verify_bundle.rs`, `restore.rs`, `bundle.rs`), each split into ranges and read in full.
