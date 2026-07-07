# POST-IMPL WHOLE-DIFF REVIEW — Cycle B — round 1

**Verdict:** GREEN (0C / 0I)
**Reviewer:** FRESH independent opus execution reviewer (no prior cycle context — cold read against the R0-GREEN SPEC/PLAN).
**Scope:** `git diff afaabee5..HEAD` — 3 commits (P0 `a596e109` `Option<bool>`+provenance param, P1 `750dc195` merge pre-pass, P2 `82334999` docs/transcript). Read-only; no files edited. Persisted verbatim per CLAUDE.md.

## Verdict: GREEN (0C / 0I)

Independently reproduced:
- `cargo test -p mnemonic-toolkit` → **3599 passed, 0 failed, exit 0** (incl. `cli_import_wallet_bitcoin_core` §8 matrix, `cli_import_wallet_roundtrip` restorations, `lint_zeroize_discipline`).
- `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` → exit 0, zero warnings.

## Funds-safety attack vectors (all SAFE)

Key structural insight underpinning regression-safety: **only fixed-single-step entries ever become merge candidates** (`analyze_merge_candidate` returns `None` for bare `/*`, already-multipath `MultiXPub`, hardened steps, non-uniform steps, script-path `tr`, non-xpub keys). Every fixed-step entry was already an exit-2 floor reject in v0.76.0. Therefore the pre-pass can only move a former-reject to either merge-accept or differentiated-reject — it **cannot** break any previously-parsing import. Non-candidate entries pass through byte-identically.

1. **Wrong merge / distinct keys → different wallets** — SAFE. Grouping key (`MergeGroupKey`, bitcoin_core.rs:246-252) is positional/ordered `(DescriptorType, multi_keyword, threshold, Vec<(fp, origin_path, xpub)>)` EXCLUDING the final step. The full xpub string is in the key, so distinct wallets never share a group. A receive/change-*shaped* cross-group pair is a hard exit-2 differentiated reject (bitcoin_core.rs:538-560), never a silent merge. Attempted counterexample `wpkh([FP_A/84'/0'/0']xpubA/0/*)` + `wpkh([FP_B/84'/0'/0']xpubB/1/*)` → exit 2 (confirmed by §8.1 linchpin `core_receive_change_distinct_keys_must_not_merge`).

2. **Global `str::replace("/{step}/*")` cross-contamination** — SAFE. Origins always terminate in `]`; base58 xpubs contain no `/` or `*`; wildcards appear only at use-sites — so the literal `/N/*` matches exactly the use-site step(s) and never an origin digit. Multi-digit disjointness holds (`/1/*` does not match inside `/11/*` — the leading `/` anchors it). Attempted `[fp/84/0]xpub/0/*` (unhardened origin ending in the step digit) → origin fragment is `/0]`, not `/0/*`; untouched.

3. **Partial per-key rewrite (the C1 class)** — SAFE. `analyze_merge_candidate` enforces cond-7 uniformity (bitcoin_core.rs:380-386): every key must carry the *identical* single unhardened step or it returns `None`. The construction is a **global** `.replace` (bitcoin_core.rs:519), so all cosigners are rewritten together; a first-only/partial rewrite is impossible. A within-entry non-uniform split (`K1/0/*, K2/1/*`) is not a candidate → floor reject (§8.11). The §8.10 multisig oracle is a genuine backstop: a partial rewrite would leave a cosigner on the wrong chain and derive mismatched chain-1 addresses.

4. **Script-path `tr` slipping into single-key handling** — SAFE. `d.tap_tree().is_some()` → `None` (bitcoin_core.rs:340) → not a candidate → floor reject (§8.15 `core_tr_scriptpath_pair_does_not_merge`, exit 2). Single-key bip86 `tr` (`tap_tree()==None`) merges like `wpkh`.

5. **Corrupt input checksum silently "repaired"** — SAFE. Each candidate's own BIP-380 checksum is verified *before* consuming (bitcoin_core.rs:435); a corrupt entry is dropped from candidacy and fails closed at `parse_entry`'s re-validation (§8.17).

6. **Vec invariant / double-emit** — SAFE. Remove-both-insert-one-at-first-index (bitcoin_core.rs:567-593); `merged_candidate_idx` + bucket membership guarantee each candidate is consumed at most once; N-pair safe. `--select-descriptor` predicates (`internal != Some(true)` / `!= Some(false)`, mod.rs:431/447) make a merged `None` entry satisfy both filters, emitted once each (§8.5).

7. **Ambiguity / near-miss regressions** — SAFE. 3+-member bucket → NOTICE + no merge (§8.9). The §7 near-miss loop only fires on *unconsumed cross-group* candidates (all of which were already exit-2), so it cannot turn a valid merge or a previously-working import into a new error.

## Oracle integrity — GENUINELY non-tautological
`derive_addresses` (test:146) parses the **original** single-path `/0/*` and `/1/*` strings and asserts `!is_multipath()`; `derive_multipath_chain_addresses` (test:168) parses the **merged** `<0;1>/*` read from `import-wallet --json`'s `bundle.descriptor` and splits via `into_single_descriptors()`. §8.4 (wpkh), §8.10 (3-key `wsh(sortedmulti)`), §8.15 (bip86 `tr`) all compare merged chain-0/chain-1 against the pre-merge originals — anchored on pre-merge truth, never on a re-authored `<0;1>`. The multisig oracle is a real anti-partial-rewrite guard. §8.13 (strengthened) uses distinct already-multipath entries so a shape-based-`None` impl would fail its `bundles=1` select assertions — real teeth. The restored `cli_import_wallet_roundtrip.rs` cells add an end-to-end net: real `export-wallet --format bitcoin-core` split output re-imports and merges, with `roundtrip.semantic_match == true` on the 2-of-2 multisig.

## P2 docs — accurate, no residual false claims
41-mnemonic.md, 45-foreign-formats.md, 39-cross-format-conversion.md now correctly state Core emits the **split** pair (multipath is import-only) and the toolkit **auto-recombines** a same-key pair into one `<0;1>/*` bundle, with the distinct-key refusal and lone-fixed-step-still-rejects documented. The false "Bitcoin Core 25+ emits `<0;1>/*`" claims (incl. the BIP-389 reference) are removed. The error-table row and guard-matrix prose match the code.

**recipe-2 transcript equivalence — CORRECT, not coincidence.** `.cmd` swaps jq/sed hand-combine → native `--format bitcoin-core`; `.out`/`.err` are byte-identical because (a) the fixture carries no `timestamp`/`next`/`next_index`, so the bitcoin-core path emits no dropped-fields NOTICE (silent, like the old descriptor path); (b) the transcript captures only the downstream `bundle` output (import envelope is redirected to a file), which is source-format-independent, plus the seed-mismatch error that depends solely on the declared xpub `xpub6FQya…` — identical in both paths. Native merge and hand-combine both build `<0;1>/*` from the receive body via the same `/0/*`→`/<0;1>/*` rewrite. verify-examples replay (reported green, 62/62) is the empirical confirmation.

## anchor-baseline ratchet — legitimate down-ratchet
`anchor-check.sh` is bidirectionally self-enforcing: it *forbids* removing a still-dangling slug (would flag a new dangler) and *requires* removing a fixed one (exits 1 on "baseline shrunk" otherwise). `#commented-descriptor` is defined at 45-foreign-formats.md:895 and still referenced at :314; the recipe rewrite dropped the cross-file workaround link that made it dangle. With audit green, the removal is correct.

## Cross-phase coherence & collateral — clean
The `Option<bool>` ripple is complete: all readers of `CoreSourceMetadata.internal` migrated (`--json` serde at import_wallet.rs:1859, text-summary `both` at :2269-2274, both select predicates at mod.rs:431/447), verified via grep. Diff touches only `bitcoin_core.rs` + shared `mod.rs`; **no** specter/electrum/coldcard/descriptor/sparrow/bsms parser changed. The select-predicate change is behavior-preserving for all non-Core entries (`source_metadata()==None`) and for passthrough Core entries (`Some(false)`/`Some(true)` equivalence to the old `!internal`/`internal`). Merge path is deterministic (first-seen bucketing, no Date/random).

**GREEN — the release ritual (§10) can proceed.** No Critical or Important findings; nothing to fold.
