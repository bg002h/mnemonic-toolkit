# cycle-prep recon — 2026-06-01 — Theme D: ms K-of-N codex32 shares

**P0 STRICT-GATE recon. Recon ONLY — no implementation, no brainstorm.**
Verifies every claim in the Theme-D pick against the **current `origin/master` source** of `mnemonic-secret`.

## Sync state (Step 1)

- **Repo:** `/scratch/code/shibboleth/mnemonic-secret` (`ms-codec` lib + `ms-cli`; default branch `master`).
- **origin/master SHA:** `4e5266a`
- **Local branch / HEAD:** `master` / `4e5266a` — **clean, 0 ahead / 0 behind** (`git rev-list --left-right --count HEAD...origin/master` → `0  0`). Claims verified against the worktree == origin bytes.
- **Untracked:** `cycle-prep-recon-mnem-language-hint.md` (a prior 2026-05-30 recon doc, SHA `e3d5665`-stamped — see note in Claim 4).
- **Crate versions (current):** `ms-codec = 0.2.1`, `ms-cli = 0.5.1` (`crates/ms-codec/Cargo.toml:3`, `crates/ms-cli/Cargo.toml:3`).
- **Note on doc location:** `MIGRATION.md` lives at the **repo root** (`./MIGRATION.md`), NOT under `design/`. It exists and is git-tracked (`git ls-files` → `MIGRATION.md`).

---

## Per-claim verification (Step 2)

### Claim 1 — single-secret hardcoding — **ACCURATE**

ms1/ms-codec hardcodes BIP-93 threshold=`'0'`, share-index=`'s'`, payload-prefix `0x00`, and the decoder hard-rejects all three deviations. Every cited element is present.

- **Consts** (`crates/ms-codec/src/consts.rs`):
  - `:17` `pub const RESERVED_PREFIX: u8 = 0x00;` — "v0.1 reserved-prefix byte (becomes the v0.2 type discriminator)."
  - `:20` `pub const THRESHOLD_V01: u8 = b'0';`
  - `:23` `pub const SHARE_INDEX_V01: u8 = b's';` ("s" denotes the unshared secret per BIP-93).
- **Emit site** (`crates/ms-codec/src/envelope.rs::package`):
  - `:156-157` `data.push(RESERVED_PREFIX);` then `data.extend_from_slice(payload_bytes);` (prepends `0x00`).
  - `:161-167` `Codex32String::from_seed(HRP, 0, tag.as_str(), Fe::S, &data[..])` — threshold hardcoded `0`, share index hardcoded `Fe::S`.
- **Reject sites** (`crates/ms-codec/src/envelope.rs::discriminate`):
  - `:101-104` `if fields.threshold_byte != THRESHOLD_V01 { return Err(Error::ThresholdNotZero { got: … }) }`
  - `:106-109` `if fields.share_index_byte != SHARE_INDEX_V01 { return Err(Error::ShareIndexNotSecret { got: … }) }`
  - `:131-135` `if payload_with_prefix[0] != RESERVED_PREFIX { return Err(Error::ReservedPrefixViolation { got: … }) }`
- **Error variants** (`crates/ms-codec/src/error.rs`): `:18 ThresholdNotZero{got:u8}`, `:23 ShareIndexNotSecret{got:char}`, `:44 ReservedPrefixViolation{got:u8}`. (The claim's "or similar" for the prefix error is exactly `ReservedPrefixViolation`.)

The reject machinery is also negative-tested: `crates/ms-codec/tests/negative.rs:51 rule_3_threshold_not_zero_rejected`, `:63 assert!(matches!(decode(&s), Err(Error::ThresholdNotZero{..})))`.

**Action for brainstorm:** the three hardcoded values + three rejects are the exact seam K-of-N must relax. v0.1 reject on threshold≠0 / share≠'s' becomes v0.2 dispatch (prefix-byte-gated). `package` gains the `Threshold` param (per migration invariant #4). All edits confined to `envelope.rs` + `consts.rs` + `error.rs` per the §10 isolation comment.

### Claim 2 — K-of-N is UNBUILT — **ACCURATE**

No `encode_shares`/`combine_shares` public API. The full `pub fn` surface of ms-codec is:
`encode` (`encode.rs:16`), `decode` + `decode_with_correction` (`decode.rs:27,188`), `inspect` (`inspect.rs:34`), the BCH helpers (`bch.rs:73,82,96,109`, `bch_decode.rs:403`), and `Payload`/`Tag` methods. **Zero share-encoding / share-combining functions.**

- **`Payload` has exactly ONE kind** (`crates/ms-codec/src/payload.rs:29-44`): `#[non_exhaustive] pub enum Payload { Entr(Vec<u8>) }`. `PayloadKind` (`:11-14`) likewise has only `Entr`. Doc-comment `:7-8`: "Future kinds (Mnem, Seed, Xprv) will arrive in v0.2+."
- **The grep "K-of-N"-ish hits are exactly the two non-share categories the claim said to exclude:**
  1. **ms-codec's reject machinery** — `ThresholdNotZero` plumbing in `error.rs` / `envelope.rs` / `inspect.rs` (NOT a share API; it's the v0.1 single-secret guard from Claim 1).
  2. **Upstream rust-codex32 error rendering** — `crates/ms-cli/src/codex32_friendly.rs:31-56` maps `codex32::Error::{InvalidThreshold, InvalidThresholdN, MismatchedThreshold, ThresholdNotPassed, …}` to friendly strings. These render the **upstream** crate's K-of-N errors; ms-cli itself never *invokes* a share API. (`codex32_friendly.rs` doc: "Friendly human-readable messages for `codex32::Error` variants.")
  3. **BIP-93 conformance test vectors** — `crates/ms-codec/tests/bip93_inline_vectors.rs:75 vector_2_k_of_2_share_s_recovers_secret`, `:102 vector_3_k_of_3_share_s_canonical`. These **parse** k-of-2/k-of-3 `s`-shares directly via raw `Codex32String::from_string` (the upstream crate) — NOT through any ms-codec share API — and only assert the recovered secret bytes. They prove the *upstream* crate handles shares; ms-codec exposes none of it.
- **ms-cli subcommands today** (`crates/ms-cli/src/cmd/`): `decode, derive, encode, inspect, repair, vectors, verify`. **No `share`, no `combine`.**

**Action for brainstorm:** K-of-N is genuinely greenfield at the ms-codec public API. The good news (de-risking): the *underlying* `rust-codex32 =0.1.0` crate already implements share split/combine (the BIP-93 vectors exercise it through `Codex32String`). The v0.2 work is largely (a) exposing/wrapping that upstream capability behind `encode_shares`/`combine_shares` with the prefix-byte discriminator and anti-collision `id` handling, and (b) the CLI/GUI/manual surface — NOT re-implementing Shamir over GF(32).

### Claim 3 — the locked v0.2 wire migration — **ACCURATE (with a documented EXTRA hardening beyond the claim)**

The reserved prefix `0x00` is documented as promoting to a type discriminator (`0x01`=entr-share, `0x02..`=future) in the `envelope.rs` seam. All FOUR invariants are present **verbatim in BOTH** `SPEC_ms_v0_1.md §5` and root `MIGRATION.md`.

- **SPEC §5** = `design/SPEC_ms_v0_1.md:212-226` ("§5. v0.1 → v0.2 Migration Contract"):
  - **(a) reserved-prefix / forward-readability** — `:216` invariant #1 (`0x01 = entr`, `0x00` → v0.1 fallback).
  - **(b) grouping** — `:218` invariant #2 (gate on prefix byte BEFORE treating `id` as a share-group key; `0x00`→v0.1 path, `0x01`→entr-group-by-id, `≥0x02`→kind-specific path).
  - **(c) anti-collision** — `:220` invariant #3 (refuse `id` ∈ `RESERVED_TAG_TABLE`; re-roll random `id`s ≈ 1/209 715; hard-error on deterministic-derivation collision).
  - **(d) API back-compat** — `:222` invariant #4 (`encode_shares(tag, Threshold::ZERO, &[p])` wire-bit-identical to `encode(tag,&p)`; `encode` becomes a thin wrapper; `Threshold` is v0.2-introduced, no v0.1 public symbol).
- **MIGRATION.md** (`./MIGRATION.md:11-23`) restates all four identically (invariants 1-4 at `:15,17,19,21`).
- **envelope.rs seam comments** corroborate the dispatch design: `:1-5` ("THE v0.2-MIGRATION SEAM … `discriminate()` adds prefix-byte dispatch, `package()` gains the `Threshold` parameter"); `:88-90` ("In v0.2 this function gains prefix-byte dispatch (`0x00`→v0.1 entr fallback, `0x01`→v0.2 entr-share path, `0x02..`→kind-specific)"); `:145-146` ("In v0.2 this function gains a `Threshold` parameter … and the prefix byte becomes the type discriminator").
- **EXTRA (beyond the claim, a plus):** SPEC §5 `:224` mandates a **prefix-byte registry table** in the v0.2 SPEC (`0x00 = v0.1-entr`, `0x01 = v0.2-entr-share`, `0x02..0xFF = unallocated, claim-via-PR`) "so two future v0.2+ kinds cannot race for the same byte." This is a hardening the claim under-stated, added in SPEC revision r3 (`:413`). MIGRATION.md `:23` carries the same "distinct prefix-byte value … claim-via-PR" requirement in prose. No invariant is MISSING or WEAKER than the claim; the contract is *stronger* than stated.

**Action for brainstorm:** invariant #4's bit-identity requirement is a hard SHA-pin regression constraint — the v0.2 SPEC/plan MUST include a fixture asserting `encode_shares(tag, ZERO, &[p]) == encode(tag,&p)` byte-for-byte across all 5 entr lengths. The registry table (§5 `:224`) is a mandatory deliverable of the v0.2 SPEC itself.

### Claim 4 — the `mnem` slug rides the SAME migration — **ACCURATE**

`design/FOLLOWUPS.md:340-347` `### \`mnem-wordlist-language-hint-on-wire\``:
- **`:346` Status:** `open`. ✓ (matches claim)
- **`:347` Tier:** `v0.2-feature`. ✓ (matches claim)
- **`:345` Scope note (verbatim):** "**NOT an independent small fix** — `mnem` rides the **v0.2 prefix-byte migration** (`0x00`/`0x01` discriminator, SPEC §1.3 `:24-29`), the same framing K-of-N share encoding and the `seed`/`xprv`/`prvk` kinds all require. **Sequence WITH the ms-v0.2 cycle, not standalone.**" ✓

**K-of-N itself has NO FOLLOWUP slug** — confirmed by exhaustive slug enumeration (`grep -nE '^### ' design/FOLLOWUPS.md`): 30+ slugs, none for share/threshold/K-of-N encoding. K-of-N lives purely as **SPEC §5 (migration contract) + §8 deferred-table tier-v0.2** (`SPEC_ms_v0_1.md:275` "K-of-N share encoding for `entr` | v0.2 deliverable … via the prefix-byte migration contract (§5) | ms-codec v0.2"). **The survey was correct: no slug; SPEC-§5/tier-v0.2 scope only.**

**Note (doc cross-reference):** the untracked `cycle-prep-recon-mnem-language-hint.md` (in the *secret* repo) is a prior 2026-05-30 recon of the same `mnem` slug, stamped against an older SHA `e3d5665`. Its cited FOLLOWUPS line numbers still match current `4e5266a` exactly (`:340-347`) — no citation decay. That doc independently reached the same scope conclusion (option B = the ms-v0.2 prefix-byte migration is the real fix and overlaps Theme D).

### Claim 5 — cross-repo lockstep surface — **ACCURATE**

- **ms-codec / ms-cli versions:** `0.2.1` / `0.5.1` (Cargo.toml `:3` each).
- **ms-cli IS on crates.io (vs toolkit tag-only):** Strong indirect confirmation — ms-cli pins `ms-codec = { path="../ms-codec", version = "=0.2.1" }` (crates.io-style exact pin, `crates/ms-cli/Cargo.toml:20`), and the **toolkit consumes ms-codec as a plain crates.io dep** `ms-codec = "0.2.1"` (`mnemonic-toolkit/crates/mnemonic-toolkit/Cargo.toml:20` — NO `git`/`path`). That dep form only resolves if ms-codec is published. Per MEMORY (`project_ms_derive_v0_5_0_shipped`), ms-cli v0.5.0 was published to crates.io. *(Direct `crates.io` API calls failed from this sandbox — network-restricted; not a contradiction.)*
- **toolkit consumes ms-cli via git-tag pin** (the binary, vs the library crates.io pin): `.github/workflows/manual.yml:88` `cargo install --git https://github.com/bg002h/mnemonic-secret --tag ms-cli-v0.5.0 ms-cli`. A dedicated `sibling-pin-check.yml` gates this tag (`:8,57` — "sibling pins … md-cli|ms-cli|mk-cli"). So the toolkit pins **ms-codec (lib, crates.io) + ms-cli (binary, git tag)** — exactly as the claim states.
- **GUI mirrors ms** via `mnemonic-gui/src/schema/ms.rs` — **present and checkable here** (`/scratch/code/shibboleth/mnemonic-gui/src/schema/ms.rs`). Per-subcommand `SubcommandSchema` with `*_FLAGS` consts for `inspect/encode/decode/verify/vectors/derive` + `LANG_MS` dropdown enum (`ms.rs:16,32,58,116,151,194,212`). A new `ms share`/`ms combine` ⇒ two new `SubcommandSchema` entries + their `*_FLAGS` consts, caught by `schema_mirror`.
- **Manual chapter** = `mnemonic-toolkit/docs/manual/src/40-cli-reference/43-ms.md` — **exists** (8.6 KB).

**SemVer + lockstep chain for a K-of-N (+ optionally `mnem`) cycle:**
New public ms-codec API (`encode_shares`/`combine_shares` + `Threshold` type) ⇒
1. **ms-codec MINOR** `0.2.1 → 0.3.0` (pre-1.0 SemVer: 0.X = breaking axis; new wire prefix `0x01` + new public types = minor). Publish to crates.io.
2. **ms-cli MINOR** `0.5.1 → 0.6.0` — new `ms share` / `ms combine` subcommands; re-pin `ms-codec = "=0.3.0"`. Publish + tag `ms-cli-v0.6.0`.
3. **Manual** — new flag/subcommand rows in `43-ms.md` (mirror invariant; `make -C docs/manual lint`).
4. **GUI** — new `SubcommandSchema`s in `mnemonic-gui/src/schema/ms.rs` (paired-PR; `schema_mirror` is a lagging gate).
5. **toolkit re-pin** — `ms-codec = "0.3.0"` in `mnemonic-toolkit/Cargo.toml:20` **+** `--tag ms-cli-v0.6.0` in `manual.yml:88` (+ any other workflow sites; `sibling-pin-check.yml` gates). Whether the toolkit *exposes* share/combine in its own CLI (`mnemonic ...`) is a scoping decision (see below) and would add a toolkit MINOR + its own GUI/manual lockstep.

---

## Cross-cutting observations

1. **The migration was designed up-front and is locked — this is the rare cycle where the wire-format contract precedes the implementation.** All four invariants are dual-sourced (SPEC §5 + MIGRATION.md) and triangulated by inline `envelope.rs` seam comments, and SPEC §5 is *stronger* than the pick claimed (mandatory prefix-byte registry table). The brainstorm does NOT get to redesign the framing; it implements §5 verbatim. R0's job is to catch deviations FROM §5, not to re-litigate it.

2. **Architecturally low-ripple in the codec.** The §10 isolation note (`envelope.rs:1-5`) means only `envelope.rs` (`discriminate` dispatch + `package` `Threshold` param), `consts.rs` (registry table / `0x01` discriminator), `payload.rs` (likely no change — shares are still `Entr` bytes, just split), and `error.rs` (new variants, alphabetical-ordering convention per toolkit CLAUDE.md applies to *toolkit* error.rs, not ms-codec — but ms-codec's `Error` is `#[non_exhaustive]` so additive variants are non-breaking) change. The rest of the crate is untouched by design.

3. **Upstream does the hard crypto.** `rust-codex32 =0.1.0` already splits/combines shares (proven by `bip93_inline_vectors.rs` parsing k-of-2/k-of-3 via `Codex32String`). The v0.2 cycle is a *wrapping + discriminator + CLI/GUI* exercise, not a Shamir-over-GF(32) reimplementation. This materially de-risks the codec layer; the risk concentrates in (a) the anti-collision `id` re-roll logic, (b) the grouping/dispatch gate correctness (invariant #2 — the misgrouping footgun), and (c) the `combine_shares` recovery surface + its error taxonomy.

4. **`mnem` and K-of-N share ONE migration but are SEPARABLE deliverables on it.** Both ride the `0x00`/`0x01` prefix-byte discriminator. K-of-N = `0x01` entr-share. `mnem` = a *new prefix value* (`0x02`+, per the registry) carrying entropy+language. They are independent payload kinds on the same framing — you can ship the migration + K-of-N first and `mnem` as a follow-on `0x02` allocation, OR bundle them. K-of-N does NOT depend on `mnem` and vice-versa; the only shared prerequisite is the prefix-byte framing itself (which both need and neither can ship without).

5. **`seed`/`xprv`/`prvk` are explicitly OUT** of this migration contract (SPEC §5 `:224`, MIGRATION.md `:23`): they overflow BIP-93 brackets and need a *separate* sub-format design. Do not let scope creep pull them in.

6. **Two prior cycle-prep saves on the constellation were mis-sized by the feature survey** (per the mnem recon doc's cross-cutting note): `bip85` no-op flags (already resolved v0.8) and `mnem` standalone (a big-cycle-in-disguise). Theme D (K-of-N) is the opposite — correctly sized by the survey as "the biggest unshipped capability," and this recon confirms it: it IS large, but the framing is pre-locked, lowering design risk.

---

## Recommended brainstorm-session scope

### Verdict on the pick
Theme D is **real, large, and well-scoped**. Every claim is ACCURATE; the only deltas are *strengthenings* (SPEC §5 has a mandatory registry table the pick under-stated; MIGRATION.md lives at repo root not `design/`). Nothing in the pick was over-stated. No structural errors.

### K-of-N alone vs K-of-N + mnem together
- **Recommend: K-of-N + the prefix-byte migration as ONE ms-v0.2 cycle; `mnem` as a fast-follow on the SAME framing (separate phase or separate patch cycle).** Rationale: the migration framing (`0x00`/`0x01` discriminator + registry table + grouping/anti-collision gates) is the expensive, design-locked part and MUST land for either feature. K-of-N is its first consumer (`0x01`). Once the framing + registry ship, `mnem` (`0x02` + 4-bit language nibble) is a small additive allocation — cleaner as its own focused phase/cycle than bloating the K-of-N R0 surface. Doing `mnem` *first* would mean building the whole migration to ship a 4-bit language hint, then K-of-N second pays the framing cost again — wrong order. **Migration + K-of-N first; `mnem` rides behind it.**

### Rough LOC sizing (codec + CLI + GUI + manual)
- **ms-codec:** ~150-300 LOC (envelope dispatch + `Threshold` type + `encode_shares`/`combine_shares` wrappers over upstream + anti-collision `id` + new error variants + the bit-identity + grouping + round-trip test matrix). Upstream does the crypto, so this is mostly plumbing + tests. **M.**
- **ms-cli:** ~200-350 LOC (two new subcommands `ms share` / `ms combine` + clap + JSON envelopes + friendly errors + gui-schema reflection + tests). **M.**
- **GUI:** ~80-150 LOC (two `SubcommandSchema`s + `*_FLAGS` + any dropdown enums). **S.**
- **Manual:** new `43-ms.md` rows + a worked share/combine example + transcript. **S.**
- **toolkit:** re-pin only (S) UNLESS the toolkit exposes `mnemonic share`/`mnemonic combine` in its own CLI — that would add a toolkit MINOR + GUI schema + manual chapter + a new toolkit subcommand (M). **Recommend scoping the toolkit-CLI-surface question explicitly at brainstorm** (is K-of-N a `mnemonic`-level verb, or ms-cli-only with toolkit consuming via library?).

### SemVer
ms-codec `0.2.1 → 0.3.0` (MINOR); ms-cli `0.5.1 → 0.6.0` (MINOR). `mnem` follow-on = another ms-codec MINOR (`0.3 → 0.4`) when it lands (new `0x02` kind).

### Full lockstep chain (in order)
ms-codec (publish crates.io) → ms-cli (publish + tag) → manual rows → GUI schema (paired-PR; lagging gate) → toolkit re-pin (`ms-codec` crates.io ver **+** `ms-cli` git tag in `manual.yml` + sibling-pin-check). If toolkit grows its own share/combine verbs: + toolkit GUI schema + toolkit manual.

### Inter-item dependencies / ordering
1. **Prefix-byte migration framing + registry table** (the hard, design-locked spine — MUST be first; SPEC §5 verbatim).
2. **K-of-N (`0x01` entr-share)** — first consumer of the framing; `encode_shares` + `combine_shares` + grouping/anti-collision gates.
3. **ms-cli `share`/`combine`** — depends on (2).
4. **GUI + manual** — depend on (3)'s flag surface.
5. **toolkit re-pin** — depends on (1)-(3) published/tagged.
6. **`mnem` (`0x02`)** — fast-follow, depends only on (1)'s framing being live; independent of (2)-(5).

### One cycle or multi-phase arc?
**One ms-v0.2 *cycle*, internally multi-phase** (framing → codec K-of-N → ms-cli → surface lockstep), with **`mnem` as a deliberate fast-follow cycle** on the now-live framing. Do NOT try to land K-of-N + mnem + the surface in a single monolithic R0 — phase it (framing/codec/cli/surface) per the per-phase TDD + reviewer-loop discipline.

---

## Gate note
The **mandatory opus R0 gate** (project CLAUDE.md, "NO code before GREEN 0C/0I") applies to whatever brainstorm spec + implementation plan-doc follow this recon — at brainstorm-spec level, plan-doc level, AND per-phase. R0 is never skipped or deferred. Because the wire contract is pre-locked in SPEC §5, R0's specific job here is to verify the implementation does not deviate from the four invariants (esp. invariant #2 grouping-gate correctness and invariant #4 bit-identity) and that the registry table is delivered.
