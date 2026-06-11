# SPEC — CI/test hygiene: pin the g6 sibling ref + strengthen the in-crate synthesize sanity tests

**Cycle:** toolkit NO-BUMP (CI + test only) · **Source SHA:** `5fc805f` (v0.53.8) · **Recon:** `cycle-prep-recon-ci-test-hygiene-cluster.md`.
**Resolves:** `g6-invariant-sibling-master-not-pin` (Part A) + `synthesize-incrate-presence-not-correctness` (Part B). Two independent NO-BUMP items, one commit. (The cluster's other two — `policynode-grammar-coverage-vacuous-on-joint-omission`, `hand-frozen-lint-canons-no-completeness` — need enumerator machinery + are already documented-with-mitigation; deferred to an optional Cycle B.)

No binary/wire/CLI change → **no `schema_mirror` / manual / GUI / sibling-codec / version-bump.**

---

## PART A — pin the g6-invariant sibling checkout to the canonical tag

### Problem (verified @ `5fc805f`)
The `g6-invariant` CI job (`.github/workflows/rust.yml:200-229`) checks out `mnemonic-secret` at `ref: master` (`:219`) to byte-compare `crates/ms-cli/src/mlock.rs` against the toolkit's inline copy. The constellation pins `ms-cli-v0.7.0` (`scripts/install.sh:38`, the sibling-pin-check source of truth) — so the invariant runs against an UNPINNED ref, which can drift away from the version the toolkit actually targets.

**De-risked:** the sibling's `crates/ms-cli/src/mlock.rs` is BYTE-IDENTICAL between `master` and `ms-cli-v0.7.0` (verified: empty `git diff master ms-cli-v0.7.0 -- crates/ms-cli/src/mlock.rs`), so pinning passes today — this fix is pre-emptive (prevents a future master-drift from spuriously red/green-ing the job).

### Design
Change `rust.yml:219` `ref: master` → `ref: ms-cli-v0.7.0`.

**Drift coverage — RESOLVED to option (a) dynamic-read (R0-r1 I2):** make the g6 ref a DERIVED value from `scripts/install.sh` (the single source of truth), so there is no second pin to drift. After the own-checkout (`:211-214`, path `mnemonic-toolkit`), add a step that extracts the ms-cli tag MIRRORING the `sibling-pin-check.yml:60-62` parser (so an install.sh format change breaks BOTH together → caught), and FAILS LOUD on an empty tag (else an empty `ref:` makes `actions/checkout` default to the repo's default branch = master, silently re-introducing the bug):
```yaml
- name: Resolve pinned ms-cli tag from install.sh
  id: pin
  working-directory: mnemonic-toolkit
  run: |
    tag=$(grep -oE 'echo "[a-z-]+\|https://[^"]+\|[^"|]+\|[^"|]*\|[^"]*"' scripts/install.sh \
      | sed -e 's/^echo "//' -e 's/"$//' \
      | awk -F'|' '$1=="ms-cli"{print $3; exit}')
    if [ -z "$tag" ]; then echo "::error::g6: could not resolve ms-cli pin from scripts/install.sh"; exit 1; fi
    echo "tag=$tag" >> "$GITHUB_OUTPUT"
- name: Checkout sibling (mnemonic-secret)
  uses: actions/checkout@v5
  with:
    repository: bg002h/mnemonic-secret
    ref: ${{ steps.pin.outputs.tag }}
    path: mnemonic-secret
```
"No drift" is precise here: the g6 ref equals install.sh's ms-cli pin by construction; the residual risk (install.sh line-format change) is SHARED with `sibling-pin-check`'s own parser, so it fails loudly in CI rather than silently. (Confirmed: g6 is the ONLY `ref: master` sibling checkout in the workflows.)

### Part A test/verification
The g6 job itself is the verification — it runs `mlock_g6_invariant` against the pinned ref. A GREEN g6 job on the resulting push confirms byte-equality holds at `ms-cli-v0.7.0` (already locally verified identical to master). `actionlint` (if available) validates the YAML; otherwise confirm the workflow parses.

---

## PART B — strengthen the in-crate synthesize sanity tests (presence → correctness)

### Problem (verified @ `5fc805f`)
`synthesize.rs`'s in-crate tests assert only PRESENCE/syntax: `full_bundle_emits_three_cards` (`:980-1000`) and `watch_only_bundle_omits_ms1` (`:1002-1013`) check `ms1.starts_with("ms1")`, `mk1.iter().all(|s| s.starts_with("mk1"))`, `md1…starts_with("md1")`, non-empty. A regression emitting a syntactically-valid but WRONG card (wrong key/network/entropy) passes. (Round-trip correctness IS covered by `cli_self_check` / verify-bundle / golden vectors — these in-crate tests are light smoke; this strengthens them in-place.)

### Design (R0-r1 I1 — NARROWED to the one genuine gap)
**The only missing round-trip is ms1→entropy.** `cross_binding_holds_round_trip` (`synthesize.rs:1046-1071`) ALREADY decodes mk1 → `assert_eq!(decoded_mk1.xpub, xpub)` (+ `origin_fingerprint`) and md1 → policy-id-stub binding + `is_wallet_policy()` on the same `fixture_full`. So mk1/md1 correctness is covered; adding them again is redundant. **DO NOT add mk1/md1 asserts** (avoid misleading duplicate coverage).

Add ONLY the ms1→entropy decode to `full_bundle_emits_three_cards` (`:980-1000`), keeping the existing presence asserts (they still smoke the HRP):
```rust
// ms1 must round-trip to the input entropy (correctness, not just "starts_with ms1").
let (_, payload) = ms_codec::decode(&bundle.ms1[0]).unwrap();
match payload {
    ms_codec::Payload::Entr(b) => assert_eq!(b, entropy, "ms1 must decode to the input entropy"),
    other => panic!("expected an Entr ms1 payload (English fixture), got {other:?}"),
}
```
(R0-r1 I3: the English `fixture_full` entropy emits a `Payload::Entr(Vec<u8>)`; `entropy` is the plain `Vec<u8>` from `fixture_full` — `Vec<u8> == Vec<u8>` compare, no zeroize obligation in the test, m4.) `watch_only_bundle_omits_ms1` needs NO new card-decode (it has no ms1; mk1/md1 covered by cross_binding) — leave it (its `!any_secret_bearing` + no-ms1 asserts are the point).

### Part B test/verification (R0-r1 I3 — RED-proof direction)
The strengthened test IS the deliverable. **RED-proof:** the assert is `assert_eq!(b, entropy)` where `entropy` is the PRE-synthesis input. To confirm non-vacuity, temporarily change the EXPECTED side (e.g. `assert_eq!(b, vec![0xFFu8; 16])`) — NOT the synthesis input — and confirm it REDs; then restore. (Perturbing the synthesis input would compare a wrong card to a wrong expectation and could falsely pass.)

---

## Ritual
NO version bump (no binary change). **NO CHANGELOG entry (R0-r1 m5):** the project convention is that NO-BUMP CI/test-only commits do NOT get a CHANGELOG entry (per the prior friendly-mapper/chunk_mk1 NO-BUMP cycle). FOLLOWUPS resolve both slugs. No README/install.sh self-pin change (the version is unbumped; install.sh's ms-cli pin is unchanged). Stage paths explicitly. Mandatory R0 gate to 0C/0I before code; persist reviews to `design/agent-reports/`.

## Non-goals
The enumerator-machinery slugs (policynode grammar / lint completeness — Cycle B); any binary/wire change; re-syncing mlock.rs (it's already byte-equal at the pin).
