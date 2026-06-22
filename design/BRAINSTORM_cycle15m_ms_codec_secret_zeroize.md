# BRAINSTORM — cycle-15 Lane M: ms-codec / ms-cli secret-memory-hygiene zeroize leg

**Status:** DESIGN ONLY (brainstorm SPEC). Feeds the **mandatory R0 loop** — NO code
until R0 converges to 0 Critical / 0 Important.
**Repo:** `mnemonic-secret` (`ms-codec` library + `ms-cli`).
**Lens:** secret-memory-hygiene is a **first-class bar** — `ms-codec` IS the
BIP-39-entropy codec, the most sensitive crate in the constellation (it owns raw
root entropy + the codex32 K-of-N secret-share intermediates).
**Source SHA audited:** `origin/master` @ `6f9f60b` (HEAD; the sweep itself was
filed against `e80ea3b` "release: ms-cli 0.9.0 — cycle-8"; all citations below
**re-grepped live against `6f9f60b`** per the project citation-decay rule).
**Recon inputs:** `mnemonic-toolkit/design/agent-reports/sweep-keymat-mnemonic-secret.md`
(9 findings) + the 8 NEW FOLLOWUP slugs in `mnemonic-secret/design/FOLLOWUPS.md`.

---

## 0. Live-recon CORRECTIONS to the task framing (READ FIRST — these change the plan)

Three load-bearing assumptions in the cycle brief did **not** survive live verification.
The R0 reviewer should sanity-check these first.

1. **ms-codec is at `0.5.0`, not `0.39.x`.** The brief said "0.39.0 → 0.40.0". Live
   `crates/ms-codec/Cargo.toml:3` = `version = "0.5.0"`; `crates/ms-cli/Cargo.toml:3`
   = `"0.9.0"`. So the SemVer call is **ms-codec MINOR 0.5.0 → 0.6.0** and
   **ms-cli MINOR 0.9.0 → 0.10.0** (still pre-1.0, 0.X is the breaking axis →
   a MINOR bump signals the breaking public-API change). The *reasoning* the brief
   gave (public API change ⇒ MINOR) is correct; only the numbers were stale.

2. **The toolkit does NOT consume `ms_codec::inspect()` / `InspectReport`.** The brief
   flagged "the toolkit's `inspect-ms1`/`self-check-ms1` sites use `ms_codec::Payload`"
   and to "flag that the toolkit pin-bump must absorb any `InspectReport`/`Payload`
   shape change." Live grep over `wt-tk-master/crates/`:
   - `ms_codec::inspect` / `InspectReport` / `InspectKind`: **0 hits.** The toolkit's
     own `inspect-ms1` path is `crates/mnemonic-toolkit/src/cmd/inspect.rs:171
     let (tag, payload) = ms_codec::decode(chunks[0])?;` — it goes through
     **`decode()` → `Payload`**, NOT `inspect()`.
   - `ms_codec::Payload` / `ms_codec::decode` / `decode_with_correction`: **many hits**
     (`slot_ms1.rs`, `language.rs`, `cmd/inspect.rs`, `cmd/ms_shares.rs`,
     `cmd/convert.rs`, `synthesize.rs`, `repair.rs`, …).
   ⇒ **The `InspectReport` reshape (slug #1) is invisible to the toolkit.** The ONLY
     cross-repo coupling is the **`Payload` shape**, and every design in this spec
     keeps `Payload` byte-for-byte unchanged (slug #4 `ms-codec-payload-zeroize-public-api`
     stays deferred). The downstream-toolkit-pin flag therefore narrows: a Lane-T
     pin bump to ms-codec 0.6.0 needs only a recompile-and-test; **no source change**
     in the toolkit is forced by this cycle. (Still: pin-bump + CI is mandatory because
     `decode()`'s clone-removal — slug #2 — is on a hot toolkit path, even though it is
     ABI-transparent.)

3. **This repo's CI does NOT enforce `cargo fmt --all --check`.** The brief asserted it
   does ("contrast the toolkit's mlock.rs exemption"). Live scan of
   `.github/workflows/{rust,fuzz-smoke}.yml`: **no `cargo fmt` / `rustfmt` step.** What
   IS gated: **`cargo clippy --all-targets -p ms-cli -- -D warnings`** (`rust.yml:143-153`).
   `rust-toolchain.toml` pins channel `1.85.0` with `components = ["rustfmt", "clippy"]`
   (fmt is *available* but not *gated*). **Implication for the spec:** the implementer
   MUST run `cargo clippy --all-targets -- -D warnings` clean (a `Zeroizing` wrap can
   trip `clippy::redundant_clone` / needless-borrow lints), and SHOULD run
   `cargo fmt` for hygiene, but there is **no fmt CI gate to break** and **no
   mlock.rs-style fmt-exemption** in this repo (the toolkit's mlock exemption is a
   *toolkit-only* g6 rule; ms has its own `mlock.rs` but no cross-repo fmt-sync
   constraint). Keep the diff `cargo fmt`-clean as a courtesy, gate on clippy.

> R0 note: if the reviewer has out-of-band knowledge that a fmt gate is *intended*
> but missing, that is a separate FOLLOWUP, not in scope here.

---

## 1. The guiding rule (the cycle-14 trap, restated as a HARD design invariant)

**RULE Z-DEBUG — never let a derived `Debug` reach secret bytes.**
`Zeroizing<T>`'s own derived `Debug` is **non-redacting**: it forwards to `T::fmt`,
so `{:?}` of a `Zeroizing<Vec<u8>>` prints the raw bytes. Wrapping a field in
`Zeroizing` gives **scrub-on-drop but NOT Debug-redaction.** Therefore, for ANY
struct or field that (a) derives `Debug` and (b) transitively holds secret bytes:

- **EITHER** drop the `#[derive(Debug)]` and hand-roll a redacting `Debug`
  (precedent in-repo: `ms_codec::Error` peels its 3 String-carrying `codex32::Error`
  variants + caps `WrongHrp.got` to 4 chars, with red-first no-echo tests —
  `error.rs:257-383`, tracked-resolved `ms-codec-error-display-echoes-input`);
- **OR** route the secret through a redacting newtype whose `Debug` prints a fixed
  placeholder (e.g. `SecretBytes([REDACTED; N])`).

This rule is the spine of slug #1 and is asserted by a **Debug-redaction RED test**
(§5). It is the same class cycle-14 hit; we encode it once and reuse it.

---

## 2. Scope — the 8 slugs (resolutions)

| # | slug | sev | layer | resolution (this cycle) |
|---|------|-----|-------|--------------------------|
| 1 | `ms-codec-inspect-report-payload-bytes-bare-and-debug` | **High** | codec | **FIX — Design A (see §3): redact Debug + `Zeroizing` field.** |
| 2 | `ms-codec-decode-scrub-defeated-by-clone-into-bare-vec` | Med | codec | **FIX — drop the `.clone()`, move bytes straight into `Payload` + tighten lint.** |
| 3 | `ms-codec-share-strings-not-zeroized-encode-and-combine` | Med (High for `secret_s`/recovered) | codec | **PARTIAL — wrap the reachable `Vec<u8>`-shaped intermediates in `Zeroizing`; the `Codex32String` String-buffers are upstream-blocked → enumerate + defer to the vendor/fork decision.** |
| 4 | `ms-codec-payload-zeroize-public-api` | — | codec | **OUT OF SCOPE — stays deferred** (breaking; keeps `Payload` shape stable for the toolkit). Listed for completeness. |
| 5 | `ms-cli-inspect-intake-and-entropy-not-zeroized` | Med | cli | **FIX — wrap `read_input` result in `Zeroizing<String>`.** |
| 6 | `ms-cli-repair-intake-and-report-strings-not-zeroized` | Med | cli | **FIX — `Zeroizing` the intake + corrected chunks + `RepairDetail` fields.** |
| 7 | `ms-cli-derive-xpriv-master-not-zeroized` | Med | cli | **PARTIAL — minimize `Xpriv` lifetimes; file the rust-bitcoin `Xpriv`-zeroize upstream-blocked FOLLOWUP analogue (cannot fully close).** |
| 8 | `ms-cli-json-output-structs-bare-secret-strings` | Low | cli | **FIX (defense-in-depth) — hold secret-bearing JSON fields + the serialized output `String` in `Zeroizing`.** |
| 9 | `ms-cli-verify-derived-to-string-temp-not-wrapped` | Low | cli | **FIX — wrap the `to_string()` word-count temp in `Zeroizing` (or count off the already-wrapped `derived_str`).** |

(The brief lists 8 slugs; #4 is the "already-tracked, listed for completeness, do
NOT re-file" entry, and #9 is the 8th NEW slug — so this is the full set.)

All citations below verified against `origin/master` @ `6f9f60b`.

---

## 3. KEY DESIGN DECISION — `InspectReport` entropy exposure (slug #1, the headline)

### The current surface (verified)
- `crates/ms-codec/src/inspect.rs:34` — `#[derive(Debug, Clone)]`
- `:35` — `#[non_exhaustive]`
- `:36` — `pub struct InspectReport`
- `:48` — `pub payload_bytes: Vec<u8>` (raw decoded entropy, prefix stripped; for
  `Mnem` it is `[lang_byte, entropy…]`)
- `:50-55,68-78` — populated from `c.parts().data()`
- `lib.rs:46,56` — `pub mod inspect;` + `pub use inspect::{inspect, InspectKind, InspectReport};`
  ⇒ `InspectReport` is a **public, re-exported** type. The struct is already
  `#[non_exhaustive]`, which constrains how external code constructs it (they
  can't `InspectReport { .. }`-literal it) but **not** how they read public fields
  or `{:?}` it.

### Three candidate exposures (the brief's a/b/c)

**Design A — `Zeroizing<Vec<u8>>` field + hand-rolled redacting `Debug` [RECOMMENDED].**
- `payload_bytes: Zeroizing<Vec<u8>>` → scrub-on-drop.
- Drop `#[derive(Debug)]`; hand-impl `Debug` that prints
  `payload_bytes: [REDACTED; N]` (length-only) and forwards all the
  *non-secret* fields (`hrp`, `threshold`, `tag`, `share_index`, `prefix_byte`,
  `checksum_valid`, `kind`, `language`) verbatim. Per RULE Z-DEBUG, the `Zeroizing`
  wrap alone is NOT enough — its Debug is non-redacting — so the hand-impl is
  **mandatory, not optional**.
- **Trait consequences:**
  - `Debug`: hand-rolled redacting impl (the whole point).
  - `Clone`: `Zeroizing<Vec<u8>>: Clone` ✓ → keep `#[derive(Clone)]` *minus* Debug,
    i.e. `#[derive(Clone)]` + manual `impl fmt::Debug`.
  - `PartialEq`: NOT currently derived (verified: only `Debug, Clone`) — so no new
    constraint. Tests compare individual fields, not whole-struct eq. **Keep it underived.**
    (If a future need arises, `Zeroizing<Vec<u8>>: PartialEq` ✓.)
- **Field access ergonomics:** `Zeroizing<Vec<u8>>` `Deref`s to `Vec<u8>`, so existing
  readers — `ms-cli/src/cmd/inspect.rs:160,166,217,247` `report.payload_bytes.len()` /
  `hex::encode(&report.payload_bytes)` — compile **unchanged** (`&report.payload_bytes`
  coerces via `Deref`; `.len()` via auto-deref). This is the smallest blast radius.
- **SemVer:** changing a public field's type `Vec<u8>` → `Zeroizing<Vec<u8>>` is a
  **breaking change** for any external code that *constructs* (blocked by
  `#[non_exhaustive]` already) or *moves out / binds by value* the field
  (`let v: Vec<u8> = report.payload_bytes;` would now be `Zeroizing<Vec<u8>>`). Plus
  removing the derived `Debug`'s transparency is observable. ⇒ **ms-codec MINOR.**
  Toolkit impact: **none** (toolkit doesn't touch `InspectReport`, §0.2).

**Design B — redacting `SecretBytes` newtype.**
- `pub struct SecretBytes(Zeroizing<Vec<u8>>);` with `impl Debug` redacting,
  `Deref<Target=[u8]>` (or `Vec<u8>`), `Clone`. `payload_bytes: SecretBytes`.
- Pros: a reusable redacting type the share-leg (#3) and JSON leg (#8) could share.
- Cons: a **bigger** public-API surface (new exported type + its trait impls), and
  every reader site (`.len()`, `hex::encode(&…)`) needs `Deref` to line up the same
  way A does — net no ergonomic win over A but more API to commit to and version.
- SemVer: same MINOR. **Not recommended for THIS cycle** (over-engineering for one
  field); revisit if #3/#8 want a shared newtype later. Capture as a possible
  follow-up consolidation, not a blocker.

**Design C — remove the public field; expose entropy via a `-> Zeroizing<Vec<u8>>` accessor.**
- `InspectReport` drops `payload_bytes` from its public fields; add
  `pub fn payload_bytes(&self) -> Zeroizing<Vec<u8>>` (clones-on-demand into a
  Zeroizing) or `pub fn payload_bytes(&self) -> &[u8]`.
- Pros: cleanest "no secret in a public Debug-able field" story.
- Cons: **larger breaking change** (field removal — every reader rewrites
  `report.payload_bytes` → `report.payload_bytes()`), and an accessor returning
  `&[u8]` still lets `{:?}` of the returned slice leak (doesn't *itself* solve the
  Debug class), while an owned-`Zeroizing` accessor re-introduces a clone. The
  derived `Debug` on the *struct* would also need handling for any other future
  secret field. **Not recommended** — A solves the leak with the smallest break.

### DECISION: **Design A.** Smallest blast radius, kills both the Debug leak (class 2)
and the bare-buffer (class 1) at the public boundary, keeps `Clone`, leaves
`PartialEq` underived, and (via `Deref`) needs **zero** changes at the existing
ms-cli reader sites. SemVer = ms-codec MINOR. The redacting `Debug` is mandatory
(RULE Z-DEBUG).

> **OPEN QUESTION for R0 (Q1):** the `Mnem` payload's `language` byte is *also* the
> first byte of `payload_bytes` (`inspect.rs:62`). Redacting `payload_bytes` in Debug
> is right; but should the *redacted* Debug still surface `language: Some(0)` (already
> a separate non-secret field — language index is not secret)? Proposed: yes, keep
> `language`/`prefix_byte`/`kind` visible (they're structural, not secret), redact only
> the raw bytes. Confirm no diagnostic field implicitly re-encodes entropy.

---

## 4. Per-slug fix sketches (codec + cli)

### #2 — `decode()` clone-into-bare-Vec (codec) — verified `decode.rs:78-95`
Current (`:82-83` Entr, `:89-90` Mnem):
```
let scrubbed: Zeroizing<Vec<u8>> = Zeroizing::new(data);
let p = Payload::Entr((*scrubbed).clone());   // ← extra un-scrubbed heap copy
```
The `.clone()` allocates a FRESH bare `Vec` that becomes the live `Payload`; the
`Zeroizing` only scrubs the (already-moved-from) `data`. Net: an EXTRA un-scrubbed
copy, and the lint reads GREEN because it anchors on the `let scrubbed:`.
**Fix:** move the bytes straight into the public `Payload` (which is bare-by-design
per slug #4 — so the honest move is strictly *fewer* copies than the clone):
```
let p = Payload::Entr(data);   // data already came from a Zeroizing envelope buffer
p.validate()?;
```
(and symmetrically for `Mnem { language, entropy }`.) This is **byte-identical wire
behavior** — `Payload::Entr(Vec<u8>)` shape unchanged, only the internal copy count
drops. **Lint change:** the `decode.rs` row currently anchors on
`let scrubbed: Zeroizing<Vec<u8>>` (`tests/lint_zeroize_discipline.rs:51`); once we
*remove* that wrap the row's evidence must change. Re-anchor the row's intent — either
delete the now-meaningless `decode` row (the scrub it asserted was theater) or
re-point it at a real invariant (e.g. assert decode.rs does **not** contain
`(*scrubbed).clone()` — a *negative* anchor proving the theater is gone). **Lean:
replace with a negative anchor + a runtime test** (see §5 #2). Update the row-count
assertion (`:81` "expected 5") accordingly.

### #3 — share strings not zeroized (codec) — verified `shares.rs`
- `encode_shares`: `secret_s: Codex32String` (`:130`, FULL secret-at-S),
  `defining: Vec<Codex32String>` (`:136`), `distributed: Vec<String>` (`:148`),
  single-string `single` (`:115`).
- `combine_shares`: `parsed: Vec<Codex32String>` ×2 (`:195,210`, every INPUT share),
  `secret: Codex32String` (`:281`, recovered full secret), clone copies at
  `from_string(s.clone())` (`:197`) and `c.to_string().to_ascii_lowercase()` (`:213`).
- **Root cause:** `Codex32String` is a `String`-newtype in `codex32-0.1.0` with **no
  Drop/Zeroize** — and that crate is **DORMANT** (tracked
  `rust-codex32-zeroize-upstream` + `codex32-upstream-dormant-vendor-vs-accept-decision`).
  We **cannot** wrap a `Codex32String`'s internal String in `Zeroizing` without
  vendoring/forking the crate (out of scope this cycle).
- **What we CAN do this cycle (in scope):** any **`Vec<u8>`-shaped** intermediate the
  share spine materializes (e.g. the CSPRNG `filler` and `secret.parts().data()`
  byte buffers — already `Zeroizing`, `shares.rs:139` + lint row 5) stays wrapped;
  confirm no NEW bare `Vec<u8>` entropy buffer is introduced. The `String`-backed
  `Codex32String`/`Vec<String>` bindings are **enumerated** (this slug's job per its
  FOLLOWUP) and **deferred** to the vendor/fork decision — we do NOT pretend to scrub
  them. **This slug PARTIALLY resolves:** keep the FOLLOWUP `open` (root cause
  external), flip its description to "enumerated; bound to vendor/fork decision";
  the `secret_s` / recovered-`secret` "arguably High" bindings get a code-comment
  marker + lifetime-minimization (drop as early as possible). **No false GREEN.**

> **OPEN QUESTION for R0 (Q2):** should this cycle pull the trigger on the
> **vendor/fork codex32** decision (`codex32-upstream-dormant-vendor-vs-accept-decision`)
> to actually close #3's String leg, or hold? Recommendation: **hold** — vendoring
> the BCH/Shamir primitives is a large, separately-gated decision (own maintenance of
> secret-sharing math); this cycle is in-memory hygiene of *our* code. Flag, don't fold.

### #5 — `ms inspect` bare intake (cli) — verified `inspect.rs:33-34,217,247`
`let ms1 = read_input(args.ms1.as_deref())?;` (bare `String`) — the lone ms1-intake
command NOT wrapping in `Zeroizing` (contrast decode/verify/derive/repair). **Fix:**
`let ms1: Zeroizing<String> = Zeroizing::new(read_input(...)?);` (or change
`read_input` callers uniformly). `report.payload_bytes` redaction/scrub is covered by
#1's Design A. Add an ms-cli lint row asserting the inspect intake wrap.

### #6 — `ms repair` bare intake + report chunks (cli) — verified `repair.rs`
`:75 let original = read_input(...)` bare; `:65-66 RepairDetail{ original_chunk:
String, corrected_chunk: String }`; `:89 original.clone()`; `:90/:94
corrected_chunk.clone()` + `vec![corrected_chunk]`. The corrected ms1 is a **valid,
decodable secret string**. **Fix:** wrap `original` in `Zeroizing<String>`; hold
`RepairDetail`'s `original_chunk`/`corrected_chunk` as `Zeroizing<String>` (note: the
`emit_json` borrows them by `&str` at `:225-226` — `Zeroizing<String>` Derefs to
`String` Derefs to `str`, so the borrow sites are fine); wrap the `corrected_chunks:
Vec<String>` accumulator's elements or the vec in `Zeroizing`. **Watch:** `RepairDetail`
derives — check for a `#[derive(Debug)]` on it (per RULE Z-DEBUG, a Debug over
`Zeroizing<String>` chunk fields would leak; redact if present).

### #7 — derived `Xpriv` not zeroized (cli) — verified `derive.rs:217-235`
`:217 seed: Zeroizing<[u8;64]>` + `:218 mlock pin` (good); `:220 master =
Xpriv::new_master(...)`; `:232-233 acct_xpriv = master.derive_priv(...)`. `bitcoin::
bip32::Xpriv` has **no Zeroize** (rust-bitcoin) → upstream-blocked, same class as the
tracked `rust-bip39-mnemonic-zeroize-upstream` but no FOLLOWUP names `Xpriv`.
**This cycle (partial):** minimize `Xpriv` lifetimes (drop `master` as soon as
`acct_xpriv` + `master_fp` are taken; scope `acct_xpriv` tightly); add a code comment
marking the bare-secret. **File** a NEW upstream-blocked FOLLOWUP
`rust-bitcoin-xpriv-zeroize-upstream` (the `Xpriv` analogue) so it's tracked.
**Cannot fully close** — note in the FOLLOWUP status.

### #8 — `--json` emit structs bare secret strings (cli) — verified `format.rs`
`EncodeJson.entropy_hex` (`:61`), `DecodeJson.entropy_hex`+`.phrase` (`:100-101`),
`CombineJson.entropy_hex`+`.phrase`+`.ms1` (`:86-89`), `SplitJson.shares: Vec<String>`
(`:69`), `InspectReportJson.payload_bytes_hex` (`:129`); plus each `emit_json`'s
`let s = to_string(&json)` serialized buffer. **Fix (defense-in-depth, Low):** hold the
secret-bearing owned fields in `Zeroizing<String>` / `Zeroizing<Vec<String>>` and wrap
the serialized output `String` before `println!`. **RULE Z-DEBUG watch:** these are
`#[derive(Serialize)]` structs; if any also `#[derive(Debug)]`, redact. Note these are
**borrowing** structs (`EncodeJson<'a>`, etc., several fields are `&'a str`) — for the
`&str`-borrow fields the secret is owned *upstream* (the wrapped local), so the wrap
belongs at the owner, not the borrow; only the genuinely-OWNED fields
(`entropy_hex: String`, `shares: Vec<String>`, `phrase: Option<String>`) take the
`Zeroizing`. Broadens tracked `ms-cli-decode-emit-zeroize-intermediate`.

### #9 — verify success-log `to_string()` temp (cli) — verified `verify.rs:170`
`fn emit_round_trip_ok` (`:169`) calls `_mnemonic.to_string()` (`:170`) to count words
— a bare full-phrase temp. (Main compare path `:116-118` IS wrapped via
`derived_str`/`supplied_str`.) **Fix:** `let s: Zeroizing<String> =
Zeroizing::new(_mnemonic.to_string()); let word_count = s.split_whitespace().count();`
— or pass the already-wrapped `derived_str` into `emit_round_trip_ok` and count off it
(zero new materialization, preferred). Add/extend the ms-cli verify lint row.

---

## 5. RED-test sketches (TDD; written BEFORE impl, per project gate)

**Type-level + behavior, per slug. The marquee is the Debug-redaction test (#1).**

- **#1 (marquee) — Debug-redaction proof.** Build an `InspectReport` from a known
  ms1 whose entropy hex is a fixed sentinel (e.g. `payload_bytes = [0xDE,0xAD,0xBE,0xEF,…]`).
  `let dbg = format!("{report:?}");`
  - `assert!(!dbg.contains("deadbeef"));` and `assert!(!dbg.to_lowercase().contains("de, ad"))`
    / no element-wise `222, 173, 190, 239` Vec-Debug leak.
  - `assert!(dbg.contains("REDACTED"));` (or the chosen placeholder).
  - `assert!(dbg.contains("hrp"));` (non-secret fields still present).
  RED on `#[derive(Debug)]`; GREEN on the hand-rolled redacting impl. (Mirror the
  `error.rs:257-383` no-echo test pattern.)
- **#1 — type-level scrub-on-drop / `Zeroizing` field.** A `cfg(test)` compile assertion
  that `payload_bytes` is `Zeroizing<Vec<u8>>` (e.g. a fn taking
  `&Zeroizing<Vec<u8>>` fed `&report.payload_bytes`), or a `static_assertions`-style
  trait check. Confirms the field type changed.
- **#1 — Deref-compat regression.** Assert existing reader shape still compiles:
  `let _ = hex::encode(&report.payload_bytes); let _ = report.payload_bytes.len();`
  (proves Design A's `Deref` keeps the ms-cli sites green).
- **#2 — no-clone / single-copy.** A `decode()` round-trip equality test
  (entropy in == entropy out) — must stay GREEN (behavior unchanged). Plus a **lint**
  test: the `decode.rs` source contains NO `(*scrubbed).clone()` (negative anchor),
  and the canonical lint-row list no longer asserts a theater scrub. (Memory-level
  "no extra copy" isn't directly observable in safe Rust; the negative source anchor +
  the round-trip is the proof we can write.)
- **#3 — enumeration / no-regression.** Round-trip `encode_shares`→`combine_shares`
  equality stays GREEN; a `Vec<u8>`-buffer lint row (filler/`parts().data()`) stays
  anchored. (No String-scrub test — honestly deferred; do NOT write a passing test that
  implies the `Codex32String`s are scrubbed.)
- **#5/#6/#9 — ms-cli lint rows.** Extend `crates/ms-cli/tests/lint_zeroize_discipline.rs`
  with rows: inspect-intake wrap (`src/cmd/inspect.rs` evidence
  `Zeroizing::new(read_input`), repair intake + chunk fields, verify success-log
  wrapped. RED first (no anchor), GREEN after the wrap. Bump the row-count assertion
  (`:102` "expected 10").
- **#7 — FOLLOWUP-presence + lifetime.** No type-level test possible (`Xpriv` is
  upstream-bare); assert the NEW FOLLOWUP slug exists (a doc/lint presence check) and
  a code comment marks the site. Behavior round-trip (derive → xpub) stays GREEN.
- **#8 — JSON wire unchanged + (if Debug) redaction.** Snapshot the `--json` output
  bytes for encode/decode/combine/split/inspect — MUST be byte-identical pre/post
  (the wrap is in-memory only). If any JSON struct derives `Debug`, add a no-echo test.
- **Wire-format guard (whole-cycle).** A test asserting `ms1` `encode`/`decode` of the
  full vector set is byte-identical pre/post this cycle — the contract that **NOTHING**
  in this cycle touches the on-wire bytes (purely in-memory hygiene + the report-struct
  shape). This is the cross-cutting safety net.

---

## 6. SemVer / publish / lint notes

- **ms-codec:** `0.5.0 → 0.6.0` (**MINOR**). Public API break = `InspectReport.payload_bytes`
  type change + dropped derived `Debug`. `Payload` shape **UNCHANGED** (slug #4 stays
  deferred). Wire bytes UNCHANGED.
- **ms-cli:** `0.9.0 → 0.10.0` (**MINOR**). Re-pins ms-codec `=0.6.0`
  (`crates/ms-cli/Cargo.toml:20` currently `ms-codec = { path = "../ms-codec",
  version = "=0.5.0" }` → bump to `=0.6.0`). CLI-surface behavior unchanged
  (zeroize is invisible to users); no clap flag/subcommand/dropdown change ⇒
  **no manual-mirror / no gui-schema-mirror update** required for this cycle
  (confirm: no `--help` text changes).
- **Publish chain (lockstep):** ms-codec → **crates.io** → bump ms-cli pin to `=0.6.0`
  → ms-cli → **crates.io**. (Standard ms two-step; see the release ritual.)
- **Lints to update (RED→GREEN this cycle):**
  - `crates/ms-codec/tests/lint_zeroize_discipline.rs` — re-anchor/replace the `decode.rs`
    row (#2), keep the share `Vec<u8>` row (#3); update the row-count assertion (`:81`).
  - `crates/ms-cli/tests/lint_zeroize_discipline.rs` — add inspect-intake (#5),
    repair-intake + chunk-field (#6), verify success-log (#9) rows; bump row-count (`:102`).
  - **No new fmt lint** (none exists). **clippy `-D warnings` must stay clean** — watch
    `clippy::redundant_clone` (we're *removing* a clone in #2, good) and any
    needless-borrow the `Zeroizing` wraps introduce.
- **Toolchain:** pinned `1.85.0` (`rust-toolchain.toml`). Run `cargo fmt` for hygiene
  (no gate) + `cargo clippy --all-targets -- -D warnings` (gated) before tag.

---

## 7. Cross-repo / downstream flags

- **`mnemonic-toolkit` (Lane T — SEPARATE):** consumes ms-codec via `ms_codec::Payload`
  + `decode()` / `decode_with_correction()` — **NOT `inspect()` / `InspectReport`**
  (verified, §0.2). ⇒ The `InspectReport` reshape is **toolkit-invisible**; the
  `Payload` shape is **unchanged**. **Implication:** the Lane-T pin bump to ms-codec
  `0.6.0` is a **recompile-and-test-only** bump — NO toolkit source change forced.
  Still mandatory (it's a dep bump on a hot path; #2's clone removal is ABI-transparent
  but on the live decode spine) — pin-bump + full `cargo test -p mnemonic-toolkit`
  before any toolkit tag. Toolkit's own zeroize cycle (cycle-14 / Lane T) is independent.
- **`descriptor-mnemonic` (md-codec/md-cli):** does **NOT** consume ms-codec (grep: 0
  hits). No impact.
- **`mnemonic-key` (mk-codec/mk-cli):** does **NOT** consume ms-codec (grep: 0 hits).
  No impact.
- **`mnemonic-gui`:** consumes the **toolkit** binary, not ms-codec directly. The
  schema-mirror gate is clap-flag-name parity; this cycle adds **no** ms-cli flag.
  No GUI action this cycle (and ms-cli isn't a GUI schema-mirror target anyway).
- **FOLLOWUP status discipline:** in the shipping commit, flip #1/#2/#5/#6/#8/#9
  → `resolved` (vX.Y); #3 → keep `open` (root-cause external; description updated to
  "enumerated, bound to vendor/fork"); #7 → keep `open` + the NEW
  `rust-bitcoin-xpriv-zeroize-upstream` filed `open` (upstream-blocked). Verify
  "open" at decision time (tracking lags code).

---

## 8. Wire-format invariant (explicit confirmation)

**The `ms1` on-wire encode/decode byte stream is UNTOUCHED by this entire cycle.**
Every change is either (a) in-memory lifetime/scrub hygiene (`Zeroizing` wraps, clone
removal) or (b) the **shape** of the in-process `InspectReport` struct / `--json`
emit structs — none of which is the BCH-checksummed wire payload. The §5 wire-format
guard test enforces this. `Payload` (the toolkit-shared type) is also byte-stable.

---

## 9. MANDATORY R0 GATE (project hard-gate)

**NO code before R0 GREEN (0 Critical / 0 Important).** This brainstorm SPEC enters the
opus-architect R0 loop now: dispatch → fold findings → **persist the review verbatim to
`design/agent-reports/`** → re-dispatch → repeat until 0C/0I (the reviewer-loop
continues after EVERY fold; folds themselves can introduce drift). Only then does the
plan-doc (also R0-gated) and per-phase TDD execution begin. Proceeding past this gate
with any open Critical/Important is prohibited.

### Open questions surfaced for R0
- **Q1 (slug #1):** redacted-`Debug` field visibility — keep `language`/`prefix_byte`/
  `kind` visible (structural, non-secret), redact only raw bytes? Confirm no
  diagnostic field re-encodes entropy. (Recommendation: yes, keep them.)
- **Q2 (slug #3):** pull the codex32 vendor/fork trigger this cycle to close #3's
  `String` leg, or hold and only enumerate? (Recommendation: **hold** — large
  separately-gated decision; this cycle is our-code hygiene.)
- **Q3 (framing):** the brief's stale assumptions (ms-codec `0.39`→ actually `0.5`;
  toolkit consumes `inspect()`→ actually only `Payload`/`decode()`; CI fmt gate →
  actually clippy-only). Confirm the R0 reviewer agrees with the §0 corrections, since
  they change the SemVer numbers, the downstream flag, and the gate story.

---

## Resolved-decisions table

| # | Decision | Choice | Rationale |
|---|----------|--------|-----------|
| D1 | `InspectReport` entropy exposure | **Design A** — `payload_bytes: Zeroizing<Vec<u8>>` + hand-rolled redacting `Debug` | smallest blast radius; kills class-1 (bare) + class-2 (Debug leak); `Deref` keeps ms-cli readers green; `Clone` kept, `PartialEq` underived |
| D2 | Redacting `SecretBytes` newtype (Design B) | **deferred** | over-engineering for one field this cycle; reconsider if #3/#8 want a shared type |
| D3 | Remove field + accessor (Design C) | **rejected** | larger break; accessor doesn't itself solve the Debug class |
| D4 | `RULE Z-DEBUG` | **adopted as hard invariant** | `Zeroizing`'s derived Debug is non-redacting (cycle-14 trap); every Debug-over-secret needs a hand-impl or redacting newtype |
| D5 | `decode()` clone (#2) | **remove `.clone()`, move into `Payload`** + re-anchor/replace lint row | the public boundary is bare-by-design (#4); the move is strictly fewer copies than the theater clone |
| D6 | Share `Codex32String` strings (#3) | **enumerate + defer (PARTIAL)**; wrap only reachable `Vec<u8>` intermediates | root cause is dormant-upstream `Codex32String` (no Drop); honest non-close, no false GREEN |
| D7 | `Xpriv` zeroize (#7) | **lifetime-min + file `rust-bitcoin-xpriv-zeroize-upstream` (PARTIAL)** | rust-bitcoin `Xpriv` is upstream-bare; cannot fully close |
| D8 | JSON structs (#8) | **`Zeroizing` owned secret fields + serialized output** | defense-in-depth (Low); STDOUT-adjacent |
| D9 | `Payload` shape (#4) | **stays deferred — UNCHANGED** | keeps the toolkit-shared type byte-stable; breaking change out of scope |
| D10 | SemVer | **ms-codec 0.5.0→0.6.0 MINOR; ms-cli 0.9.0→0.10.0 MINOR** | public `InspectReport` break + ms-cli pin re-release |
| D11 | CI gate | **clippy `-D warnings` (gated); fmt run-but-ungated** | NO `cargo fmt --check` CI exists; clippy is the real gate |
| D12 | Wire format | **UNCHANGED — guarded by a byte-identity test** | purely in-memory + report-struct-shape hygiene |
| D13 | Manual / GUI schema mirror | **no update needed** | no clap flag/subcommand/help change |

---

*Authored for cycle-15 Lane M. Citations verified live against `origin/master` @
`6f9f60b`. DESIGN ONLY — feeds the mandatory R0 loop; no code until 0C/0I GREEN.*
