# cycle-prep recon — 2026-05-29 — codec test-hardening themes 1/2/3

**Origin SHAs at recon time** (4 repos; note divergent default branches):
- `mnemonic-toolkit` — `master` @ `1349fa2` — up-to-date (0/0)
- `descriptor-mnemonic` (md) — **`main`** @ `ca4591b` — up-to-date vs upstream (0/0)
- `mnemonic-secret` (ms) — `master` @ `c919f4b` — up-to-date (0/0)
- `mnemonic-key` (mk) — **`main`** @ `d9d2ed9` — up-to-date vs upstream (0/0)

**Untracked:** toolkit `.claude/` only.

Themes verified: **(1) property/fuzz harness per codec**, **(2) BCH miscorrection/boundary coverage**, **(3) indel reject-contract oracle**. These are survey FINDINGS (not filed FOLLOWUP slugs) spanning the 3 codec repos + the toolkit (theme 3). Verdict headline: **theme 1 has a STRUCTURALLY-WRONG over-generalization for ms-codec; themes 2 & 3 are ACCURATE with minor line-drift + one toolkit path correction (which was my own grep error, not the survey's).**

---

## Per-theme verification

### Theme 1 — property/fuzz harness for the encode↔decode bijection
- **Claim (synthesis):** "zero proptest/quickcheck/fuzz deps in *any* codec; add a harness to each."
- **md-codec** — **ACCURATE.** `descriptor-mnemonic/crates/md-codec/Cargo.toml` has no proptest/quickcheck/arbitrary/fuzz dep. Needs a harness from scratch: `Arbitrary` for the encodable type + assert `decode(encode(x))==x` and `decode` never panics on arbitrary `&[u8]`. (Confirm exact public Arbitrary target — `Descriptor`/`Node`/`Body` — at brainstorm; lib.rs re-export surface not enumerated here.)
- **mk-codec** — **ACCURATE.** `mnemonic-key/crates/mk-codec/Cargo.toml` has no such dep. Harness target = `KeyCard` (respecting `1≤stubs≤255`, `1..=10` path components, valid secp pubkey/chaincode, both networks).
- **ms-codec** — **STRUCTURALLY-WRONG as stated.** `mnemonic-secret/crates/ms-codec/Cargo.toml:20` already has `proptest = "1"`, and `tests/round_trip.rs:11` already property-tests the bijection across ALL 5 entropy lengths (`round_trip_entr_16/20/24/28/32`). The real ms gap is narrower: `round_trip.rs` has NO **corrupt→correct→decode** property (grep for corrupt/flip/error/decode_with_correction in that file = empty). **Action for brainstorm:** ms theme-1 = EXTEND the existing proptest with a correction property (inject ≤4 random symbol errors → assert recovery + reported positions), NOT "bootstrap proptest."
- **Action for brainstorm spec:** scope theme-1 as md (new harness) + mk (new harness) + ms (extend). Cite SHAs above.

### Theme 2 — BCH correction tested only at low error counts; miscorrection guards untested
- **mk-codec** — **ACCURATE.** Through the public-ish `bch_correct_*` API (`bch.rs:394/452`), tests cover regular {clean,1,2} (`bch.rs:1090/1104/1120`) and long {clean,1} (`bch.rs:1140/1151`). 3-/4-error only at the RAW algorithm layer (`bch_decode.rs:779 four_errors_decode_correctly_long`), never through `bch_correct_*`/`decode()`. **NUANCE (flag for brainstorm):** mk's 5-error test is `five_errors_either_rejects_or_returns_bogus_recovery` (`bch_decode.rs:811`) — mk apparently has NO per-chunk polymod re-verify guard like md/ms; it relies on the **cross-chunk hash** at reassembly to catch bogus per-chunk recovery. So theme-2-for-mk must test the cross-chunk-hash guard, not a (nonexistent) per-chunk re-verify. mk's guard model differs from md/ms.
- **md-codec** — **ACCURATE (line-drift ~1).** `tests/bch_decode.rs` corruption sites: `:151` pos-0, `:174 one_error_at_last_data_symbol` (highest = last DATA symbol, before the 13-symbol checksum), `:193 four_error_t_boundary`, `:224 five_error_too_many`. NO checksum-region corruption, no mixed data+checksum. Defensive re-verify guard at `chunk.rs:559` ("catches pathological 5+-error patterns") + polymod `:564` — described but untested with an aliasing pattern. (Survey cited `chunk.rs:560-570`/`:565`; actual `:559-564`.)
- **ms-codec** — **ACCURATE.** Single fixture `VALID_MS1_12W` (`bch_decode.rs:35`); all correction cells use only the 12-word string (`:64/:76/:102`). Defensive re-verify branch at `decode.rs:231-238` ("catches pathological 5+-error patterns" → `TooManyErrors`) — untested. (Per the ms survey, `decode.rs:237`/the BM-fooled branch is dead under the current corpus.)
- **Common finding:** for md+ms, the miscorrection re-verify guard is the highest-value test target (prove it's load-bearing: disable → a test must go red); plus checksum-region + multi-length errors. For mk, the equivalent is the cross-chunk-hash bogus-recovery guard.
- **Action for brainstorm spec:** refresh line numbers to the SHAs above; treat mk's guard model as distinct from md/ms.

### Theme 3 — codecs are the toolkit's indel oracle but don't test the "no self-correct / verify-or-reject" contract
- **Core claim — ACCURATE.** All 3 codecs have ZERO indel tests/contract (grep indel|insertion|deletion across each codec src+tests: ms=0; md=1 and mk=1, but BOTH are false positives — md `bitstream.rs:215` "extended in-place", mk `gen_mk_vectors.rs:26` "sorts on insertion"). Codec oracle APIs confirmed: `ms_codec::decode.rs:188 decode_with_correction`, `md_codec::chunk.rs:492 decode_with_correction` + `chunk.rs:305 reassemble`.
- **Toolkit anchors — ACCURATE (survey was right; my first grep searched the wrong subdir).** Indel engine = `src/indel.rs` (`IndelOracle` trait `:63`, `recover_indel` `:77`); per-card oracles in `src/repair.rs`: `Ms1IndelOracle` `:884`, `Mk1IndelOracle` `:1001`, `repair_via_ms_codec` `:818`, `repair_via_md_codec` (whole-set atomic). The load-bearing premise is documented at `repair.rs:962`: `md_codec::chunk::reassemble` "does NOT self-correct, so the chunk…" — i.e. the toolkit's ⊆-rule soundness rests on a codec property the codecs never test.
- **Action for brainstorm spec:** correct the file path to `src/repair.rs` / `src/indel.rs` (NOT `cmd/`). Each codec gets a test pinning: any input not within Hamming-≤4 of a valid codeword (i.e. an indel-corrupted / length-changed string) returns `Err`, never a different valid payload. This is the cheapest, highest-leverage-for-toolkit-soundness slice.

---

## Cross-cutting observations
1. **Theme-1 over-generalization (the strict-gate's main catch):** ms-codec already has proptest + a bijection property suite. A brainstorm that says "add proptest to all 3 codecs" would mis-scope ms. Scope ms as "extend with a correction property."
2. **Divergent default branches:** md & mk ship from **`main`**; toolkit & ms from `master`. The ship sequence (and any CI workflow refs) differ per repo — don't assume `master` everywhere.
3. **SemVer / bug-likelihood:** all three themes are **test-only** (theme-1's `proptest` is a `[dev-dependencies]` add → no published-API change). Per project convention, test-only ⇒ no version bump, commit-to-default-branch. **BUT** — per the convergence-suite precedent (the toolkit's F1–F5 turned "test-only" cycles into real PATCHes), **theme 2 is the most likely to surface a real codec bug** (untested miscorrection guards are exactly where silent-miscorrection bugs hide). If a guard test goes red, that codec gets a PATCH/MINOR fix-bump, and the toolkit's git-dep pin to that codec may need a refresh. Plan for the possibility.
4. **Lockstep:** test additions → no GUI schema-mirror, no manual flag-coverage (no clap surface change). The only lockstep risk is the conditional one in obs. 3 (a codec fix-bump → toolkit re-pin → possibly a toolkit `--include-ignored` cross-CLI test). No sibling-codec *companion-FOLLOWUP* needed for tests alone.
5. **mk guard-model asymmetry (obs. under Theme 2):** theme 2 is NOT uniform — md/ms have per-chunk polymod re-verify; mk leans on the cross-chunk hash. A single "test the re-verify guard" spec would be wrong for mk.

---

## Recommended brainstorm-session scope
- **Structure: per-codec cycles, NOT per-theme.** Each codec is its own repo (own branch/ship: md/mk `main`, ms `master`), the BCH guard model differs per codec (obs. 5), and a per-codec cycle keeps the mandatory R0 reviewer-loop in one tree. Three cycles: **mk**, **md**, **ms** — each covering its slice of themes 1+2+3.
- **Sequence by risk (highest first):** **mk** (leanest existing coverage; the `five_errors_…_returns_bogus_recovery` path + the cross-chunk-hash guard are the scariest; also adjacent to the unresolved depth/child seam) → **md** (largest grammar surface; checksum-region + miscorrection-aliasing) → **ms** (smallest delta — already has proptest; extend + multi-length + re-verify-branch).
- **Alternative fast first slice:** theme 3 alone (the indel reject-contract) is tiny per codec (~1 test each) and directly de-risks the toolkit's shipped `repair --max-indel` soundness — viable as a single cross-codec quick win before the larger per-codec property/BCH cycles. Flag in brainstorm.
- **Sizing:** theme-1 harness ≈ 60–120 LOC/codec (md/mk); ms extend ≈ 30 LOC. theme-2 ≈ 5–10 cells/codec. theme-3 ≈ 1–3 cells/codec. Each per-codec cycle is small (≈150–250 LOC of tests).
- **SemVer:** test-only ⇒ no bump (commit to the repo's default branch) UNLESS a guard test goes red (obs. 3) → fix-bump that codec + refresh the toolkit git-dep pin.
- **Mandatory R0 gate** (project standard): each per-codec brainstorm/plan-doc must reach opus-architect 0C/0I before any code — applies even to test-only cycles (cf. the anchor-dangler & sibling-pin CI/test-only cycles, both R0-gated). The mk guard-model asymmetry + the ms over-generalization are exactly what R0 should stress.
- **Inter-theme dependency:** none hard; theme-3 is independent and cheapest. theme-1's property harness, once built per codec, is the natural vehicle to *also* express theme-2's randomized miscorrection-aliasing tests (a proptest that injects 5–8 errors and asserts `Err`-or-original) — so within a per-codec cycle, do theme-1 harness first, then layer theme-2 on it.
