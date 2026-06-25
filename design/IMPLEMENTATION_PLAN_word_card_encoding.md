# IMPLEMENTATION PLAN — Engravable Word-Card encoding (`mk1` / `md1`)

- **Status:** Plan-doc — **R0 round-1 folded (2C/3I/5n addressed); round-2 re-dispatch pending.** NOT approved for implementation.
- **Date:** 2026-06-24 (plan-R0 round-1 folds same day)
- **Spec (R0-GREEN):** `design/BRAINSTORM_word_card_encoding_2026-06-24.md` (commit `31109f8e`,
  R0 converged round-4; reviews `design/agent-reports/word-card-r0-round-{1,2,3,4}.md`).
- **Source SHAs (grep-verified at write time):** toolkit `d08b0d51`, md-codec `7764145d`,
  mk-codec `46631c6`, ms-codec `5c0335c`.
- **Verified deps already present:** `crates/mnemonic-toolkit/Cargo.toml:47 sha2 = "0.10"`,
  `:49 bip39 = { version = "2", features = ["all-languages"] }`; workspace members =
  `["crates/mnemonic-toolkit"]` (`Cargo.toml:2`).

> Per `CLAUDE.md` this plan-doc MUST pass an opus architect **R0 review to 0C/0I** before any
> code; the reviewer-loop continues after every fold. This document RESOLVES the spec's §12
> open questions to concrete, R0-checkable decisions. No code until plan-R0-GREEN.

### Plan-R0 round-1 fold log (2026-06-24)

Round-1 verdict RED (2C/3I/5n); full review `design/agent-reports/word-card-plan-r0-round-1.md`.
The reviewer **machine-verified the field/RS/RAID algebra CORRECT** (primitive poly
`x¹¹+x²+1`/`α=x` order 2047 brute-forced; systematic eval-form RS append-only + MDS; RAID
`[n+r,n]` MDS). Folds (all in the concrete sync constants + header self-description):

- **C1** — deletion *pinpoint* is now via the **GLOBAL** non-linear integrity tag (local
  reinsert-test can't: the missing value is a free unknown); local CRC only prunes;
  whole-block-erasure / refuse fallback (§4.3).
- **C2** — checkpoint local-check is now **CRC-5** (`x⁵+x²+1`), uniform `2⁻⁵` substitution
  miss, replacing the integer weighted-sum's ~25% even-slot blind spot (§3, §4.3).
- **I1** — ledger entries are now **2-word exact counts** (0..2047), replacing ×16 (capped
  2032 < 2047) (§4.2).
- **I2** — P0 is now adversarial multi-vector round-trip KATs (multi-chunk mk1, multi-`0x02`
  md1, keyless template) + resolve the NO-BUMP question before P1 (§7).
- **I3** — header carries a **positional, self-CRC'd `GEOM`** block (explicit `payload_len`,
  `t`, `b`) read **before RS**, with closed-form geometry recovery (§4.2).
- Minors: M1 K reconciliation (xpub K=58 incl. tag; spec ladder was payload-only) (§4.1);
  M2 mod-8 aliasing bound KAT (§4.3); M3 array-id target + `P₂` exponent = H1 index (§3);
  M4 padding freeze at P1/P2 (§7); N5 SHA refresh; N6 `error.rs` placement (§6.2); +
  `wc-codec` fuzz target + `recover --json` wire-shape coordination (§9).

---

## 1. What this plan resolves (spec §12 → concrete)

| Spec open-Q | Resolution (this plan) | §ref |
|---|---|---|
| Q1 header bit-layout | concrete field/width table | §4.2 |
| Q2 checkpoint 11-bit split | `marker(3) │ index(3, mod 8) │ CRC-5(5)`; deletion pinpoint via GLOBAL tag; block-erasure/refuse fallback | §4.3 |
| Q3 crate boundary | new `crates/wc-codec` lib in the toolkit workspace; extraction-to-repo deferred | §2 |
| Q4 wordlist | BIP-39 English (confirmed) | §3 |
| Q5 stripe padding | zero-pad each xpub payload to array-wide max byte-length | §4.6 |
| Q6 stop-sign encoding | 2 words: `marker(4) │ count(11) │ checksum(7)` | §4.4 |
| Q7 small-`K` floor | `K<16 ⇒` floor `m_min=12` parity + single degenerate checkpoint | §4.3 |
| Q8 interleaving | DEFERRED (documented non-default lever) | §9 |
| Q9 r≥3 | construction supports it; surfaced stop-points capped at r=2 | §4.6 |
| §9.5 frozen constants | concrete values | §3 |

---

## 2. Crate boundary & dependency graph (Q3)

**Decision:** a new **`crates/wc-codec`** library crate, added as a second workspace member,
consumed by `mnemonic-toolkit`. Rationale: keeps the RS/RAID/sync engine isolated and
independently testable (per the spec's isolation principle), reuses the workspace's pinned
`sha2`/`bip39`, and is **extractable to a standalone published repo** later when the sibling
CLIs (`md`/`mk`) adopt a word view. Extraction + crates.io publish is **explicitly deferred**
(documented migration note); for v0.1 it ships in-workspace, `path`-dep'd.

`wc-codec` is **codec-AGNOSTIC** — it operates on `(SourceKind, version, payload: &[u8])`, NOT
on `mk1`/`md1` structure. Dep surface: `sha2` (integrity tag), `bip39` (English wordlist),
`zeroize` (not required — payloads are xpub/descriptor, NOT spending secrets; see §8). No
`miniscript`/codec dep.

**Canonical-payload adapter (toolkit-owned).** The spec (§5.4 / N6) consumes the codec's
**pre-chunking canonical payload bytes**. The toolkit owns `fn canonical_payload(card) ->
(SourceKind, Vec<u8>)`:
- **P0 cross-repo dependency:** mk-codec/md-codec expose a public accessor returning the
  assembled pre-chunking bytecode (mk1) / packed payload (md1) as a deterministic
  `Vec<u8>` — e.g. `mk_codec::Mk1::canonical_payload_bytes()` /
  `md_codec::Md1::canonical_payload_bytes()`. These are additive, NO-BUMP-eligible accessors;
  filed as **companion FOLLOWUPS** in both sibling repos (per `CLAUDE.md` cross-repo rule).
  Round-trip invariant: `decode(m*1) → canonical_payload → wc encode → wc decode →
  canonical_payload → re-encode(m*1)` is byte-identical to the original `m*1` (KAT in P6).

```
bip39 ─┐
sha2 ──┼─► crates/wc-codec (lib) ─► mnemonic-toolkit (bin) ─► uses mk-codec/md-codec accessors
        (RS/RAID/sync/word engine)        (canonical-payload adapter + CLI)
```

---

## 3. Frozen constants (resolves §9.5) — pinned, KAT-locked

All values are **frozen for recoverability**; P1/P2 KATs assert them.

- **Field:** `GF(2¹¹)`, primitive polynomial `p(x) = x¹¹ + x² + 1` (`0x805`). Primitive
  element `α = x` (`0x002`); `ord(α) = 2047 = 23·89`. **KAT:** `α^2047 = 1`, `α^23 ≠ 1`,
  `α^89 ≠ 1` (proves order, since the only proper divisors of 2047 are 23, 89).
- **Symbol ↔ word map:** BIP-39 **English** index `i ∈ 0..2047` is the field element whose
  integer value is `i` (bit₁₀…bit₀, MSB-first to match bech32/codec convention). The wordlist
  is the canonical BIP-39 English list (`bip39` crate, English).
- **RS (value layer):** systematic **evaluation-form** RS. Fixed point sequence
  `βⱼ = α^j` (`j = 0,1,2,…`). Data symbols are placed verbatim at `β₀…β_{K′−1}`
  (systematic); the unique degree-`<K′` polynomial `P` interpolates `(βⱼ, dataⱼ)`; parity
  word `m` = `P(β_{K′+m−1})`. **Append-only** because the `β` sequence is a fixed prefix.
  Encode = Newton/Lagrange interpolation + evaluation; decode = **Gao's algorithm** (partial
  GCD) with erasures handled by puncturing erased coordinates. Length cap `n = K′+m ≤ 2047`.
- **RAID generator:** same `α`. `P₁[c] = Σᵢ xᵢ[c]` (weights `α⁰=1` ⇒ field-add = XOR);
  `P₂[c] = Σᵢ αⁱ·xᵢ[c]` (`i` = stripe index, fixed by `array-id`). `[n+r, n]` MDS;
  `ord(α)=2047 ≥ n_max=32`. **KAT:** recover any `r` of `n+r` erasures.
- **Integrity tag:** `SHA-256(canonical_payload)` truncated to the top `t` bits, default
  **`t = 44`** (4 words; residual `≤ 2⁻⁴⁴`), min `t = 33` (3 words). NON-LINEAR; a linear
  (BCH/CRC/XOR) tag is **forbidden** in-codeword (spec C1/NEW-C1).
- **Checkpoint:** marker `0b101` (3b) + block-index mod 8 (3b) + **CRC-5 local-check**,
  generator `x⁵+x²+1` (5b) — uniform single-substitution miss `≤ 2⁻⁵` (§4.3, C2 fix).
  **Stop-sign marker** `0b1111`; **ledger marker** `0b1110` — all three distinct so the word
  classes never alias.
- **array-id:** top 22 bits of `SHA-256(concat ordered master-fingerprints)`; collision
  target `≤ 2⁻²²` across a user's wallets (a *matching aid only*, never the integrity check).
  **The `P₂` stripe exponent `i` = header `H1`'s `index-in-array` field** (§4.2), NOT
  array-id.
- **Stripe padding (frozen at P1/P2, M4):** zero-pad each xpub payload to the array-wide max
  byte-length before the 8→11 regroup (§4.6).

---

## 4. Wire layout (bit-exact)

A Word-Card string is an ordered word sequence:

```
[ HEADER ][ d d d │C│ d d d │C│ … data interspersed with checkpoints … ][ INTEGRITY ][ PARITY tail … ][ STOP-SIGN ]
   §4.2        §4.1 payload + §4.3 checkpoints              §4.5 tag      §4.1 RS parity   §4.4
```

All of `{header, payload-data, checkpoints, integrity}` are the **RS message** `K′`; the
parity tail is the appendable RS redundancy. (Checkpoints sit *inside* `K′` — spec C2.)

### 4.1 Payload (Layer A)
`canonical_payload` bytes ‖ integrity-tag bits, regrouped **8→11** MSB-first into `K`
symbols. `K = ceil((8·payload_len + t) / 11)`. Final symbol low-bit-padded; pad bits = 0
(decode asserts).

**Canonical mk1-xpub numbers (M1 — supersede the spec's *illustrative* ladder):** payload
73 B, `t=44` ⇒ `K = ceil((584+44)/11) = 58`; `b = round(√58) = 8`;
`checkpoints = ceil(58/8) = 8`; `K′ = 58 + 8 (+ header) `. The spec §6.4 ladder used
payload-only `K=54 / K′=62` (pre-integrity-tag); this plan's `K=58` (tag-inclusive) is the
implementation truth, and all phase KATs use it. The spec ladder stands as illustrative.

### 4.2 Header (Q1) — fixed prefix + appendable ledger
- **`H0`** (1 word, 11 bits): `version(4) │ source-kind(2: 00=mk1,01=md1) │ has-raid(1) │
  reserved(4)=0`.
- **`H1`** (1 word, present iff `has-raid`): `n−1(5: 1..32) │ role(3: 0=solo,1=data,2=parityA,
  3=parityB) │ index-in-array(3: 0..n−1 or parity index)`.
- **`array-id`** (2 words, present iff `has-raid`): top 22 bits of
  `SHA-256(concat of the n ordered cosigner master-fingerprints)`.
- **`GEOM`** (geometry — I3 fix; read **POSITIONALLY before RS**): explicit `payload_len`
  (2 words, up to 2¹⁶ B) `│ t(6) │ stride b(4)`, followed by a **`header-CRC`** word (CRC-11
  over all positional header words). The cold decoder reads GEOM by position, verifies
  header-CRC, then derives the whole geometry **in closed form with NO post-RS dependency:**
  `K = ceil((8·payload_len + t)/11)`; `checkpoints = ceil(K/b)` (or 1 if `K<16`);
  `K′ = |header| + K + checkpoints`; `m_present = words_present − K′ − |stop-sign| −
  |ledger|`. **header-CRC fail ⇒ refuse-and-report** (the header is a few words the human
  re-verifies); the header is also inside the big RS for correction on a clean re-read. This
  breaks the chicken-and-egg: geometry never depends on a successful RS pass.
- **`recorded-length LEDGER`** (append-only, front-anchored, §6.3): each entry is a **2-word**
  `marker(4: 0b1110) │ cumulative-count(11: 0..2047) │ checksum(7)` — **exact**, reaching the
  2047 cap (I1 fix; the prior ×16 capped at 2032 < 2047). Authoritative recorded length =
  **max** over all ledger entries AND stop-signs (all exact 11-bit). A new 2-word entry is
  appended on each stop/upgrade (front grows ~2 words/upgrade — acceptable).

### 4.3 Checkpoints (Layer B, Q2) — C1/C2 fold
- Inserted after every `b` payload-data words, `b = round(√K)`. Count `≈ √K`.
- **11-bit split:** `marker(3)=0b101 │ block-index(3, mod 8) │ local-check(5)`.
- **`local-check(5) = CRC-5`** over the block's payload-word bits, generator `x⁵+x²+1`
  (primitive). Uniform single-substitution miss `≤ 2⁻⁵≈3.1%` — **C2 fix**, replacing the
  integer `Σ(k+1)·wₖ mod 32` (even-weighted slots had a ~25% blind spot). Its role:
  (a) flag a corrupt block (→ convert a known-bad block to a cheap erasure), (b) **prune**
  indel candidate positions, (c) aid recognition.
- **Recognition / realignment:** marker + **≥2-checkpoint mod-8 index continuity** + CRC;
  ambiguity ⇒ **refuse-and-report**. mod-8 index aliasing is safe: a false +1-mod-8
  continuity needs ≥`8·b` consecutive words destroyed = the whole-payload refuse case (M2 —
  asserted by a P3 KAT).
- **Indel pinpoint is GLOBAL, not local (C1 fix).** A block-length anomaly (`b∓1`) flags a
  deletion/insertion. The local reinsert-test **cannot** pinpoint a deletion — the missing
  value is a free unknown, so a single local check validates every candidate slot (reviewer-
  measured 0% unique). Instead, for a **single** in-block deletion: reinsert a placeholder
  erasure, enumerate the `≤b` candidate gap positions **pruned by the CRC-5**, and **validate
  each by the GLOBAL RS-decode + non-linear integrity tag (§4.5)** — wrong alignments fail at
  `≤ 2⁻ᵗ`, so a unique alignment recovers the deletion at **cost 1 erasure**.
- **Fallback / bound:** ≥2 indels in one block, an ambiguous/too-large candidate search, or a
  run ⇒ **whole-block erasure (cost ≤ `b`)**; if the global candidate-alignment budget
  (capped per-decode) is exceeded ⇒ **refuse-and-report**. This is the spec §6.1 fallback,
  faithful; the honest budget (spec §9(b)) holds: 1/word *when globally located*, else ≤`b`.
- **Small-`K` (Q7):** `K < 16` ⇒ a single degenerate checkpoint (no interspersing) and a
  **parity floor `m_min = 12`** (correct 6 / detect 12). The √K rule is skipped.

### 4.4 Stop-sign (Q6) + truncation
- **2 words:** `marker(4)=0b1111 │ cumulative-word-count(11: 0..2047) │ checksum(7) =
  SHA-256(all-preceding-words)[0..7]`. The marker is distinct from checkpoint/ledger markers.
- Decoder takes the **highest-count** stop-sign as authoritative; earlier ones = ordinary
  words. **Truncation flag** iff `words-present < max(ledger entries, highest stop-sign
  count)` — both **exact 11-bit counts reaching the 2047 cap** (I1), so a near-cap top
  truncation is never missed (spec §6.3 / §8 step 1).

### 4.5 Integrity tag (Layer C, §4.1 placement)
`t`-bit `SHA-256(canonical_payload)[0..t]`, regrouped with the payload (§4.1) ⇒ RS-protected,
checked **post-correction** (§8 step 5). Catches an RS miscorrection at `≤ 2⁻ᵗ`.

### 4.6 RAID stripes (Layer D, Q5/Q9)
- Each xpub `canonical_payload` zero-padded to the array-wide **max byte-length**; striped
  column-wise as `GF(2¹¹)` symbols (after the same 8→11 regroup). `P₁`/`P₂` per §3.
- Each Recovery plate is a full Word-Card string (`role = parityA|parityB`). Surfaced
  stop-points capped at **r=2**; the construction admits r≥3 (not surfaced).

---

## 5. Algorithms

- **Encode (per string):** `canonical_payload ‖ tag → 8→11 regroup → data symbols → insert
  checkpoints → interpolate P → emit β-evaluations for the requested parity tier → prepend
  header+ledger → append stop-sign → map symbols→BIP-39 words`.
- **Decode (per string) — two-pass (spec §8):** (1) read the **positional `GEOM` header**,
  verify header-CRC (fail ⇒ refuse), derive `(K, t, K′, b, m_present)` in closed form; read
  the ledger + highest stop-sign, set the truncation flag; (2) **sync pass** — recognize
  checkpoints (marker + ≥2-checkpoint index continuity + CRC), classify blocks (trichotomy);
  for a single in-block deletion enumerate gap candidates **validated by the GLOBAL RS+tag**
  (step 5), else whole-block-erase; rebuild the full grid; (3) **RS pass** — Gao decode
  (errors + erasures), refuse if weight `> ⌊m/2⌋ +` erasure budget; (4) re-verify sync vs
  corrected symbols; (5) strip checkpoints/header, 11→8 regroup, **recompute SHA-256 tag and
  require equality** (miscorrection guard — also the oracle for step-2 alignment search);
  (6) RAID reconstruct if assembling an array.
- **RAID reconstruct:** gather plates by `array-id`; ≤ r missing ⇒ solve the `[n+r,n]`
  Vandermonde system for the missing stripes; each = a full Word-Card string.

---

## 6. Public API & toolkit surface

### 6.1 `wc-codec`
```rust
pub enum SourceKind { Mk1Xpub, Md1Descriptor }
pub struct EncodeOpts { pub parity_words: u16, pub integrity_bits: u8 /*=44*/, pub raid: Option<RaidRole> }
pub fn encode(kind: SourceKind, payload: &[u8], opts: &EncodeOpts) -> Vec<&'static str>;
pub struct Decoded { pub kind: SourceKind, pub payload: Vec<u8>, pub repair: RepairReport, pub truncated: bool }
pub fn decode(words: &[&str]) -> Result<Decoded, WcError>;
pub fn raid_encode(payloads: &[&[u8]], r: u8) -> Result<Vec<Vec<&'static str>>, WcError>;
pub fn raid_reconstruct(plates: &[Option<&[&str]>]) -> Result<Vec<Vec<u8>>, WcError>;
```
`WcError` variants **alphabetical**. No secret material (xpub/descriptor only) ⇒ no zeroize
requirement (§8).

### 6.2 toolkit CLI (lockstep surface)
- **`mnemonic word-card`** — emit a Word Card. Flags: `--from <mk1|md1|@slot>`,
  `--parity-tier <words|pct>`, `--raid <0|1|2>`, `--integrity-bits`, `--json`,
  `--group-size`/`--separator` (reuse display convention).
- **`mnemonic recover`** (extend) — accept word-card input; report repair + truncation; emit
  the recovered `m*1` / xpub / descriptor.
- New `ToolkitError::WordCard(wc_codec::WcError)` — placed **alphabetically among the
  post-v0.27.2 sorted variants** in `error.rs` (the pre-v0.27.2 block is not yet sorted —
  tracked by `error-rs-retroactive-alphabetical-sort`; do not interleave with it) + its
  `Display` / `exit_code` / `kind` arms (N6).

---

## 7. Phased build (TDD; per-phase R0; single-subagent-per-phase)

Each phase: **tests written first**, full `cargo test -p` suite per R0 (memory:
full-package-suite), per-phase opus review persisted to `design/agent-reports/` before fold.

- **P0 — codec accessors (cross-repo, I2).** Add `canonical_payload_bytes()` to
  mk-codec/md-codec + companion FOLLOWUPS. **Adversarial round-trip KATs:** multi-chunk mk1
  (cross-chunk hash), multi-`0x02`-TLV md1 (TLV ordering), keyless-template md1 — each must
  satisfy `assemble∘disassemble = id` byte-identically. **Resolve NO-BUMP here:** if the
  accessor must canonicalize (re-sort TLV / recompute a hash) it is a PATCH, not NO-BUMP —
  decide before P1.
- **P1 — `wc-codec` scaffold + field + symbol map + padding (M4).** `GF(2¹¹)` primitivity KAT
  (`α^2047=1, α^23≠1, α^89≠1`), BIP-39 English map, 8↔11 regroup, **freeze the stripe
  zero-padding rule here** so P5 is self-contained. RED→GREEN.
- **P2 — systematic evaluation-form RS.** encode/decode (Gao), errors+erasures, append-only
  prefix-extensibility. KATs: round-trip, correct `⌊m/2⌋`, erase `m`, prefix validity,
  refuse-beyond-budget.
- **P3 — sync layer.** checkpoints (marker/index/**CRC-5**), trichotomy, realignment,
  **global-validated single-deletion recovery + whole-block-erasure fallback**. KATs:
  del/ins/sub/run/deleted-checkpoint/**compound (deleted checkpoint + adjacent data
  deletion)**/refuse-on-ambiguity; **CRC-5 single-substitution detection at the `2⁻⁵` floor
  (C2)**; **mod-8 aliasing bound (M2)**; **single in-block deletion uniquely recovered via the
  GLOBAL tag, not local pinpoint (C1)**.
- **P4 — integrity tag + GEOM header + stop-sign + exact ledger + full pipeline.** KATs: full
  round-trip; **miscorrection caught by the tag**; **near-2047 top-truncation flagged (I1)**;
  **cold-decode-from-words-only via positional GEOM (I3)**; truncation flag on lost newest
  tail; deliberate-stop NOT flagged; ledger durability.
- **P5 — RAID r=1/r=2.** striping/reconstruct; recover any r of n+r; `P₁` append-only
  invariance; lone-parity-plate privacy KAT.
- **P6 — toolkit integration.** canonical-payload adapter (P0 accessors); `word-card` +
  `recover` CLI; `ToolkitError::WordCard`; **GUI `schema_mirror`** update;
  **`docs/manual/src/40-cli-reference/`** mirror; **binary-identical** doc output (fixed
  seeds); version-sites (§8); a **`wc-codec` fuzz target** (encode/decode round-trip +
  corrupt-input no-panic); the new **`word-card`/`recover --json` wire-shape** is NOT
  schema_mirror-gated (names-only) ⇒ coordinate GUI consumers via the paired-PR rule. KAT:
  `m*1 → word-card → recover → m*1` byte-identical.
- **Post-impl:** mandatory independent adversarial whole-diff review (spec §"post-impl"),
  persisted, re-dispatched to GREEN before tag.

---

## 8. Lockstep, version-sites, hygiene

- **GUI** `mnemonic-gui/src/schema/mnemonic.rs` (`schema_mirror`, paired-PR rule) — new
  `word-card` subcommand + flags + any dropdown VALUE enums.
- **Manual** `docs/manual/src/40-cli-reference/` (`docs/manual/tests/lint.sh`).
- **`ToolkitError`** new variant + arms **alphabetical**.
- **Docs** every CLI-output block **binary-generated/identical** (fixed seeds;
  `verify-examples`).
- **Release version-sites:** `CHANGELOG.md` (tag-gated `changelog-check`), **both** READMEs,
  `fuzz/Cargo.lock`, `scripts/install.sh` sibling pins, man-pages (`gen-man`). Re-run full
  suite + fuzz build before tag.
- **Cross-repo companions:** P0 codec accessors → FOLLOWUPS in mk-codec/md-codec; if/when
  `wc-codec` is extracted to its own repo, register it in `CLAUDE.md`'s constellation list.
- **Hygiene/funds-safety:** payloads are xpub/descriptor (public-ish) — **no spending
  secret**, so the zeroize bar is not triggered; **xpub privacy** is respected (RAID lone
  parity leaks nothing; no secret in logs). The custody bar = no-silent-miscorrection (tag) +
  truncation-flag + refuse-on-ambiguity, all KAT-locked.

## 9. Deferred / non-goals
- **Interleaving** (Q8) — documented lever, not built (single-codeword suffices at `n ≤
  ~300`).
- **r ≥ 3** RAID stop-points (construction supports; not surfaced).
- **Custom confusion-minimized wordlist** — BIP-39 English chosen; revisit only with
  engraving-confusion data.
- **`wc-codec` extraction to a standalone repo + crates.io** — after `md`/`mk` adopt it.

## 10. Next step
**Mandatory plan-doc R0 review to 0C/0I** (own loop; fold→persist→re-dispatch). No code until
plan-R0-GREEN. Versioning at build: toolkit MINOR (new `word-card` surface); `wc-codec` 0.1.0
(in-workspace); codec accessors NO-BUMP/PATCH.
