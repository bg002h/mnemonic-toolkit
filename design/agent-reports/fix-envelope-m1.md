# Fix: envelope M1 (card-only origin-path mismatch), PR #138

Decision from code: E2 compares the decoded card's `origin_path` with `mk1_origin_path(&card_xpub, &descriptor_path)` (synthesize.rs:128, the form every emit site uses). The first attempt used the descriptor xpub's depth and was refused by the released-v0.27.0 fixture `envelope_v0_27_0_as_released.json` (its descriptor key @1 is depth 3, its card carries a depth-4 xpub with the full 4-component path). Keying on the card's own xpub depth accepts that card and every current-form card, and refuses the reviewer's card. Same E2 line, no new text. No manual/CHANGELOG edit: CHANGELOG already says the cards must match "in number, key, fingerprint and origin path", which is now true.

## RED (edff64cf, on ae93943d source)
`envelope_mk1_origin_path_only_mismatch_refused` FAILED: `descriptor: stderr=note: stdout is watch-only ... left: Some(0) right: Some(1)`. Honest control `envelope_honest_depth_3_and_5_origins_pass` (depth-4 xpub with depth-3 origin; depth-4 xpub with depth-5 origin; depth-3 xpub with depth-4 origin; bundle exits 0, no export format raises E2) passes both before and after. Output: /scratch/cache/m1-red.txt.

## GREEN (f2fb51ca)
cli_import_json_envelope_consistency: 16 tests run, 16 passed (/scratch/cache/m1-green.txt).

## Proof script (scripts/prove-envelope-consistency.sh, new case "mk1[0] card path only differs", bundle + 6 export formats)
- pre-change 52a0424b: 4 passed, 17 failed (21 cases)
- ae93943d: 19 passed, 2 failed (21 cases); the 2 are the new case (bundle, export)
- head: 21 passed, 0 failed (21 cases)

## Gate (head before FOLLOWUPS/report commit)
- cargo fmt --all -- --check: clean
- cargo clippy --locked --all-targets -- -D warnings: clean
- cargo nextest run --locked --workspace: 4267 run, 4267 passed, 38 skipped (/scratch/cache/m1-gate.txt)
- proof script: 21/21

## Mutation gate (mutation-gate.sh <wt> ae93943d, head 924245cc)
`mutation-gate 924245cc vs ae93943d: caught 9, MISSED 0, timeout 0, unviable 0 (pass 1 exit 0)`. No survivors, no extra rows needed.

C / I / M / N: 0 / 0 / 0 / 0
