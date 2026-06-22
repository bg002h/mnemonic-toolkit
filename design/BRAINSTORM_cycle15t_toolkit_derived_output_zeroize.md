# BRAINSTORM SPEC — cycle-15 Lane T: SECRET derived-output zeroize leg (mnemonic-toolkit)

> **DESIGN ONLY.** This is a brainstorm spec for the mandatory R0 architect loop. **NO code is
> written until R0 converges to 0 Critical / 0 Important.** Per CLAUDE.md: implementation MUST NOT
> begin — and no implementer subagent may be dispatched — before this spec (and its plan-doc)
> passes R0 GREEN.

- **Cycle / lane:** cycle-15, Lane T (toolkit). Sibling lanes: Lane M = `mnemonic-secret`
  (ms-codec derived-output/inspect/decode/share-strings), Lane G = `mnemonic-gui` (app-level
  run-holders). This spec is **Lane T only** — it does NOT plan ms-codec or GUI internals.
- **Source SHA (re-pinned live):** `origin/master = 79100a66` —
  *"design(sweep): secret-key-material hygiene sweep — file toolkit FOLLOWUP slugs + 5-repo sweep
  reports"*. This is **one commit AHEAD of the sweep report's audit SHA** (`ddabf5e3`, v0.67.0);
  the sweep commit added only the FOLLOWUP entries + sweep reports, no source change, so every
  cited `src/*.rs` line is byte-identical to the sweep audit. **All `file:line` citations below
  were re-grepped against `79100a66`** (`git show origin/master:<path> | grep -n`) — live, not the
  sweep snapshot. Working worktree: `/scratch/code/shibboleth/wt-tk-master` (NOT the main checkout
  `/scratch/code/shibboleth/mnemonic-toolkit`, which is another instance's workspace).
- **Recon source:** `design/agent-reports/sweep-keymat-toolkit.md` (findings 1, 2, 3).
- **Lens:** secret-memory-hygiene is a **first-class bar** — these `String`s carry root-seed-class
  spending authority (full child seed phrases, WIFs, xprvs, BIP-39 master phrases). The fact that
  the final-stage buffer is already `Zeroizing` does NOT excuse leaving full secret-text copies
  un-scrubbed during the rendering / derivation window.

---

## 0. Scope — three slugs

| # | Slug | Sev | File(s) | What |
|---|------|-----|---------|------|
| 1 | `bip85-derive-child-output-secretstring` | **HIGH/MED** (headline) | `cmd/derive_child.rs`, `bip85.rs` | rendered SPENDABLE BIP-85 child secret (phrase/WIF/xprv/password/hex/dice) returned as bare `String` |
| 2 | `electrum-native-seed-normalize-intermediates-zeroizing` | MED | `electrum.rs` | normalized phrase+passphrase in ~6 bare `String`s across PBKDF2; entropy `clone()`d out of `Zeroizing` |
| 3 | `seedqr-codec-internal-secret-string-zeroizing` | MED/LOW | `seedqr.rs` | SeedQR phrase/digits/entropy-hex built in bare internal `String`s |

Out of scope for THIS spec (filed by the same sweep, deferred to separate slugs/cycles):
`inspect-ms1-payload-husk` (folded into `self-check-ms1-decode-not-zeroizing`, LOW, separate),
`bsms-derive-hmac-key-not-zeroizing` (deliberate-by-doc, flag-for-decision), and
`bundle-unified-whole-file-allowlist-precision` (lint precision). Do NOT scope-creep them in.

---

## 1. The governing rule — raw-`Zeroizing<String>`-Debug-leak (cycle-14 lesson)

This is the **single most important design constraint** and it drives every per-site type call.

- A **bare `zeroize::Zeroizing<String>`** derives a **NON-redacting tuple-struct `Debug`** — i.e.
  `{:?}` on a `Zeroizing<String>` prints `Zeroizing("the-actual-secret")`. Any path that reaches a
  `{:?}` / `panic!` / `assert_eq!` failure message / `#[derive(Debug)]`-on-an-enclosing-struct /
  tracing/log macro will **leak the plaintext secret**. (This is exactly the trap
  `secret_string.rs` test `debug_redacts_the_secret` (`secret_string.rs:117`) was written to prove
  against — *"a raw `Zeroizing<String>` derives a NON-redacting tuple-struct
  Debug (`Zeroizing("secret")`)"*; cf. `eq_failure_debug_does_not_leak` (`secret_string.rs:157`).)
- The in-repo **`SecretString`** newtype (`src/secret_string.rs`, `pub struct
  SecretString(Zeroizing<String>)`, lib.rs:104 → `pub mod secret_string`) solves this:
  - `Zeroize`-on-drop (wraps `Zeroizing<String>`) — scrubs.
  - **length-only redacting `Debug`** (`secret_string.rs:61`) — `{:?}` never leaks the value.
  - `Display` + `Deref<Target=str>` render the value **verbatim** (`secret_string.rs:32,54`) — so
    `writeln!(out, "{x}")` / `format!` text paths are byte-identical.
  - transparent `serde::Serialize` (`secret_string.rs:67`) — `--json` wire-shape byte-identical.
  - `PartialEq`/`Eq` (`secret_string.rs:46,52`) — embeddable in `#[derive(PartialEq, Eq)]` structs.

**The rule, stated for R0:**

> **Use `SecretString` for any secret value that COULD reach a `{:?}` / panic / log / Debug-derived
> enclosing type. Use bare `Zeroizing<String>` ONLY for a value that is provably never
> Debug-printed** (a short-lived internal scratch buffer that is consumed-by-move and never embedded
> in a Debug-derived type, never `{:?}`-formatted, never returned into a Debug-printed struct).

Per-site application is the Resolved-decisions table in §2.

---

## 2. Resolved-decisions table (SecretString vs Zeroizing, per site)

> Each row's type choice follows §1: `SecretString` where the value is rendered/returned/could be
> Debug-printed; `Zeroizing<String>`/`Zeroizing<Vec<u8>>` only for provably-never-Debug scratch.

### Slug 1 — `bip85` derived output (all sites bin-private; see §4 SemVer)

| Site (file:line, `79100a66`) | Current | Decision | Why |
|---|---|---|---|
| `bip85.rs:72` `format_bip39_phrase` return; `:90` `Ok(mnemonic.to_string())` | `Result<String, _>` | **`Result<SecretString, _>`** | rendered child seed phrase; flows to `output` → emitted; mirror `silent_payment`/`nostr` precedent |
| `bip85.rs:103` `format_hd_seed_wif` return; `:121` `Ok(pk.to_wif())` | `Result<String, _>` | **`Result<SecretString, _>`** | HD-seed WIF = full spending authority |
| `bip85.rs:131` `format_xprv_child` return; `:154` `Ok(xprv.to_string())` | `Result<String, _>` | **`Result<SecretString, _>`** | child xprv = full spending authority |
| `bip85.rs:163` `format_hex_bytes` return; `:170` `Ok(hex::encode(&entropy[..n]))` | `Result<String, _>` | **`Result<SecretString, _>`** | raw derived entropy hex |
| `bip85.rs:181` `format_password_base64` return; `:189` | `Result<String, _>` | **`Result<SecretString, _>`** | derived password |
| `bip85.rs:196` `format_password_base85` return; `:204` | `Result<String, _>` | **`Result<SecretString, _>`** | derived password |
| `bip85.rs:222` `format_dice_rolls` return; `:273` `Ok(out.join(","))` | `Result<String, _>` | **`Result<SecretString, _>`** | entropy-derived dice rolls (= seed material) |
| `bip85.rs:268` `out.push(trial.to_string())` (the `Vec<String>` dice scratch) | `Vec<String>` | **`Vec<String>` → wrap the final `out` as the `SecretString` return only**; the per-trial scratch is dice-roll *digits* (1..=N), low-value, consumed-by-`join` — **leave bare** OR wrap `out: Vec<String>` move-into-`SecretString` at `:273`. R0: confirm low-value-digit scratch is acceptable bare. | scratch holds derived bytes but as decimal-die-faces; the joined output is the secret carrier |
| `cmd/derive_child.rs:224` `let output = match … {…}` | `String` | **`SecretString`** | the one emitter local; `format_*` now return `SecretString` so the match arms type-unify |
| `cmd/derive_child.rs:304` `writeln!(stdout, "{output}")` | `Display` of `String` | **unchanged** — `SecretString: Display` renders verbatim | text-path byte-identical (§5) |

**Decision rationale (slug 1):** `SecretString` (NOT bare `Zeroizing`) at every site, because (a) the
value is rendered via `{output}` and (b) `format_*` are `pub(crate)` fns whose return could be
captured in a future Debug context — `SecretString`'s redacting Debug is the safe default. This is a
verbatim mirror of the live precedent `cmd/silent_payment.rs:286-287` (`SecretString::new(hex::…)`)
and `cmd/nostr.rs:235` (`SecretString::new(crate::nostr::wif_for(…))`), both already lint-rowed
(`lint_zeroize_discipline.rs:252,261`).

### Slug 2 — `electrum.rs` normalize intermediates (all sites bin-private; see §4 SemVer)

| Site (file:line, `79100a66`) | Current | Decision | Why |
|---|---|---|---|
| `electrum.rs:79` `normalize_text_electrum(s:&str)->String` return + its internal nfkd/lower/strip/collapse copies | `String` | **`Zeroizing<String>`** | private helper, return consumed-by-move into `norm_*` locals; never `{:?}`-printed → bare `Zeroizing` is safe and avoids a `SecretString` import in a hot path |
| `electrum.rs:98` `let norm_phrase = normalize_text_electrum(phrase)` | `String` | **`Zeroizing<String>`** (auto, once helper returns it) | normalized phrase copy alive across PBKDF2 |
| `electrum.rs:99` `let norm_pp = normalize_text_electrum(passphrase)` | `String` | **`Zeroizing<String>`** | normalized passphrase copy alive across PBKDF2 |
| `electrum.rs:147`/`:149` `let words = …iter().map(normalize_electrum).collect()` (in `phrase_to_entropy`) | `Vec<String>` | **`Vec<Zeroizing<String>>`**, wrapping at the **consumption boundary** — `.map(\|w\| Zeroizing::new(normalize_electrum(&w)))` (each word scrubs). **Do NOT widen `normalize_electrum` itself:** it is imported from `crate::wordlists` (`electrum.rs:12 use crate::wordlists::{normalize_electrum, …}`, defined `pub(crate) fn normalize_electrum(s:&str)->String` at `wordlists/mod.rs:132`) and has **OTHER callers** (`wordlists/mod.rs:88,122`) → widening its return is cross-module scope-creep. `wordlists::normalize_electrum: -> String` stays untouched; the `Zeroizing` wrap lives only at the electrum-side call site. | normalized secret words |
| `electrum.rs:175` `Ok((*acc).clone())` (clones secret entropy OUT of `acc: Zeroizing<Vec<u8>>`) | `Vec<u8>` return + bare clone | **return `Zeroizing<Vec<u8>>`**: `Ok(acc)` (move the `Zeroizing`, no clone). Update the one caller to accept `Zeroizing<Vec<u8>>`. | the headline of this slug — the clone defeats the wrapper |
| `electrum.rs:215` `let phrase = words.join(" ")` (in `entropy_to_phrase`) + its `Ok(phrase)` return | `String` | **KEEP `entropy_to_phrase: -> Result<String, _>`** (do NOT widen the return) — mirror the slug-3 keep-public-`String` philosophy. Wrap ONLY the per-candidate scratch `let phrase = Zeroizing::new(words.join(" "))` so each rejected candidate scrubs; when `validate_seed_version` matches, return by move-out at the `Ok` boundary as a bare `String` (the validated phrase IS the literal return expression). The `words: Vec<&str>` at `:207` are **static-wordlist refs** (`wl[rem].as_str()`), NOT secret → **no wrap**. | reconstructed secret phrase — see **I-1 ripple note** below |
| `electrum.rs:243` `normalize_phrase_for_hmac(s:&str)->String` (electrum-LOCAL helper) + `:244` `let stage1 = normalize_electrum(s)` + `:249` `strip_cjk_internal_whitespace` returns/scratch | `String` | **`Zeroizing<String>`** for the electrum-LOCAL helper return + the `stage1`/CJK scratch — wrap `stage1` at the consumption boundary (`Zeroizing::new(normalize_electrum(s))`), same as `:149`. **`wordlists::normalize_electrum` itself stays `-> String`** (cross-module, other callers — see `:149` row). | normalized-phrase HMAC intermediates |

**Decision rationale (slug 2):** bare `Zeroizing<String>`/`Zeroizing<Vec<u8>>` (NOT `SecretString`)
is correct here — these are **private helper returns and KDF scratch** consumed by move within the
module; none is `{:?}`-printed, returned into a Debug-derived struct, or rendered to the user. The
`:175` `(*acc).clone()` → `acc` move-out is the one funds-relevant fix (it currently copies the
master secret into an un-scrubbed heap `Vec` that outlives `acc`'s scrub). R0 must confirm each
helper's caller chain so a `Zeroizing` return never gets `(*x).clone()`-defeated again downstream.

> **I-1 ripple note — why `entropy_to_phrase`'s return STAYS `String` (do NOT widen).** Widening
> `entropy_to_phrase: -> Result<Zeroizing<String>, _>` breaks `compute_outputs`
> (`cmd/convert.rs:1423`, the sole production caller). That call sits inside the
> `let v = match t { … }` block (`cmd/convert.rs:1300` open, `:1464` close) whose **sibling arms
> produce bare `String`** — `Phrase` (`Mnemonic::…to_string()`, `:1305`), `wif.encrypt_wif`
> (`:1410`), `render_address_from_xpub` (`:1446`) — and every arm value is pushed into
> `out: Vec<(NodeType, String)>` (`:1454`). A `Zeroizing<String>` arm would FAIL match-arm type
> unification → either a hard compile break, or the implementer "fixes" it with `(*phrase).clone()`
> — **re-introducing exactly the clone-out-of-`Zeroizing` anti-pattern slug 2 is fixing** (cf. the
> `:175` `(*acc).clone()` headline). So `entropy_to_phrase` keeps its `String` return; only the
> internal per-candidate scratch is wrapped. **The `phrase_to_entropy` ripple is DIFFERENT and
> STAYS as specified** — its `:175 (*acc).clone()` → `Ok(acc)` move-out + `Zeroizing<Vec<u8>>`
> return are correct (the entropy is *not* type-unified against bare-`Vec<u8>` sibling arms — it
> feeds `hex::encode(&entropy)` (`convert.rs:1751`) which Deref-coerces cleanly), and its test
> `electrum.rs:454` (`assert_eq!(re_bytes, bytes)` on two `Zeroizing<Vec<u8>>`) compiles because
> zeroize 1.8.2 `Zeroizing` derives `PartialEq`.

### Slug 3 — `seedqr.rs` internal secret Strings (**`pub fn` — SemVer-sensitive; see §4**)

| Site (file:line, `79100a66`) | Current | Decision | Why |
|---|---|---|---|
| `seedqr.rs:96` `pub fn decode(input:&str)->Result<String, SeedqrError>` RETURN | `String` | **KEEP `String` return** (do NOT widen) — wrap only internals | changing the **public** return type is a breaking change → would force MAJOR or the MINOR-with-justification debate; the consumer (`cmd/seedqr.rs:261,264`) re-wraps in `Zeroizing` immediately, so the return-move residue is the documented small window |
| `seedqr.rs:98` `let stripped: String` (raw digits scratch) | `String` | **`Zeroizing<String>`** | raw SeedQR digit secret scratch |
| `seedqr.rs:131` `let phrase = words.join(" ")` then returned | `String` | wrap as **`Zeroizing<String>` internally**, `Ok((*phrase).clone())` at the return boundary? **NO** — that re-introduces a clone. **Decision:** keep `phrase` bare ONLY at the final `Ok(...)` move; wrap the *intermediate* `words`/`stripped`. R0: see Open-Q1. | the join result IS the return value; the caller re-wraps |
| `seedqr.rs:141` `pub fn encode` RETURN; `:143` `let words: Vec<String>`; `:155` `let normalized = words.join(" ")`; `:161` `let mut digits: String` (the per-word index string = the SeedQR digit secret return-carrier) | `String` | KEEP return `String`; wrap `words` → **`Vec<Zeroizing<String>>`** (or a single joined `Zeroizing`), `normalized` → **`Zeroizing<String>`**, and `digits` → **`Zeroizing<String>`** scratch, moved out bare at the `Ok(digits)` return (`:171`) | secret phrase copy + SeedQR digit secret |
| `seedqr.rs:182` `pub fn encode_compact` RETURN; `:183` `let words: Vec<String>`; `:192` `normalized`; `:196` `Ok(hex::encode(m.to_entropy()))` | `String` | KEEP return `String`; wrap `words` → **`Vec<Zeroizing<String>>`**, `normalized` → **`Zeroizing<String>`**; the `to_entropy()` `Vec<u8>` → wrap in `Zeroizing` before `hex::encode` | raw entropy hex + normalized phrase |
| `seedqr.rs:205` `pub fn decode_compact` RETURN; `:206` `stripped`; `:208` `let bytes = …` (hex-decoded raw entropy `Vec<u8>`); `:218` `Ok(m.to_string())` | `String` | KEEP return `String`; wrap `stripped` → **`Zeroizing<String>`** and `bytes` → **`Zeroizing<Vec<u8>>`** (decoded raw entropy is the highest-value scratch here) | raw entropy hex scratch + decoded entropy bytes |

**Decision rationale (slug 3):** the public return type is **deliberately NOT widened** — only the
**internal scratch** (`stripped`, `normalized`, intermediate `words`/`digits`, the `to_entropy()`
hex-decoded `bytes`) is wrapped in bare `Zeroizing`. This (a) keeps `pub fn` signatures stable →
**avoids a SemVer escalation from this slug** (the MINOR in §4 is driven entirely by the bip85
lint-row count, not by seedqr API), and (b) is **kept stable to avoid the API break**. The keep-
public-`String` decision STAYS. **Correction (M-2):** the earlier justification claimed the consumer
`cmd/seedqr.rs` "already wraps each return in `Zeroizing` immediately" — that is true ONLY for the
two SeedQR command-emit sites (`cmd/seedqr.rs:194`, `:264`). The OTHER **8** callers of
`seedqr::decode`/`encode`/etc. bind the return to a **bare `String`**: `convert.rs:1259`,
`restore.rs:439`, `restore.rs:867`, `restore.rs:3417`, `addresses.rs:208`, `bundle.rs:552`,
`verify_bundle.rs:1535` (the 8th being a second cmd-path local). Those 8 bare consumer-locals are a
**separate pre-existing residue class, explicitly OUT OF SCOPE for this slug** — widening the public
return to chase them would be the API break this slug rightly avoids. The residue THIS slug closes is
the **internal scratch buffers + the transient return-move**, which the sweep itself scoped as
MED/LOW "one notch lower."

---

## 3. Open questions for R0 — **all four RESOLVED by R0 round 1** (recorded here for the audit trail)

1. **(Slug 3 / I-1) The `join`-into-return clone tension — RESOLVED: keep the public `String` return,
   wrap only intermediate scratch.** For `seedqr::*` AND for `electrum::entropy_to_phrase`, the value
   that is the literal return expression stays a bare `String` moved-out at the `Ok(...)`; only the
   genuinely-intermediate scratch that outlives the return is wrapped in `Zeroizing`. The alternative
   (wrap-then-`(*phrase).clone()`) is **rejected** — it re-introduces exactly the `(*acc).clone()`
   anti-pattern slug 2 fixes. R0 additionally established (I-1) that `entropy_to_phrase` **cannot**
   widen even if we wanted to: its caller `compute_outputs` type-unifies it against bare-`String`
   match arms. The keep-public-`String` rule is uniform across slug 2 (`entropy_to_phrase`) and slug
   3.
2. **(Slug 1) Dice scratch `out.push(trial.to_string())` (`bip85.rs:268`) — RESOLVED: leave the
   per-trial scratch bare.** The per-trial `String`s are decimal die-faces (`1..=sides`); the `join`
   output IS wrapped as the `SecretString` return. (R0 raised no objection; recommendation stands.)
3. **(ms-codec pin — see §6) — RESOLVED: `"0.5"`→`"0.6"` is a SHIP-time recompile-only coordination
   item.** R0 confirmed the prompt's "0.39" is the **md-codec** pin (`Cargo.toml:36`), not ms-codec
   (`ms-codec = "0.5"`, `Cargo.toml:29`). Toolkit references NO `ms_codec::inspect()`/`InspectReport`;
   all `Payload::Mnem` matches use `..` rest patterns → 0.6.0 field ADDITIONS are tolerated. If Lane M
   crosses to `0.6.0`, widen `"0.5"`→`"0.6"` at SHIP + re-resolve lockfiles; this is a recompile-only
   item with no toolkit source change (any rename/variant break is caught by the ship `cargo build` +
   full suite). **Do NOT bump the pin in this lane unless Lane M's release crosses the `"0.5"`
   caret.** (Detail in §6.)
4. **(Slug 2) `phrase_to_entropy` return-type change ripple — RESOLVED: the move fix is correct.**
   `:175` `Ok((*acc).clone())` → `Ok(acc)` changes the bin-private return from `Vec<u8>` to
   `Zeroizing<Vec<u8>>`. R0 confirmed the production consumer `convert.rs:1751 hex::encode(&entropy)`
   Deref-coerces cleanly, and the test `electrum.rs:454` (`assert_eq!(re_bytes, bytes)` on two
   `Zeroizing<Vec<u8>>`) compiles since `Zeroizing` derives `PartialEq` (zeroize 1.8.2). No caller
   does `(*ret).clone()` or embeds it in a Debug-derived struct. (NB: this is `phrase_to_entropy`, a
   DIFFERENT fn from `entropy_to_phrase` in Q1/I-1 — only the former's return widens.)

---

## 4. SemVer — toolkit **MINOR** (0.67.0 → 0.68.0)

**Call: MINOR.** Rationale, per-slug:

- **Slugs 1 (`bip85`/`derive_child`) and 2 (`electrum`):** these modules are **fully bin-private** —
  mounted only as `mod bip85;` / `mod electrum;` in `main.rs` (`main.rs:4,14`); they appear in
  `lib.rs` **nowhere**, not even under the `#[cfg(fuzzing)] pub mod` block (verified: `lib.rs` has
  no `bip85`/`derive_child`/`electrum` mount at all). So their signature changes are
  **internal-only — zero public-API impact**. By themselves these would be NO-BUMP.
- **Slug 3 (`seedqr`):** `seedqr` IS always-`pub mod seedqr;` (`lib.rs:111`) and `decode`/`encode`/
  `encode_compact`/`decode_compact` are `pub fn` (`seedqr.rs:96/141/182/205`). **Because §2 keeps
  their return types as bare `String`, the public API is UNCHANGED** — slug 3 alone is also NO-BUMP.
- **Why MINOR anyway:** the **v0.10.1 precedent** (FOLLOWUPS `resolved-slot-derived-account-
  zeroizing-field`, tag `mnemonic-toolkit-v0.10.1`) established that a **`cfg(fuzzing)`-reachable /
  `pub`-surface secret-type migration ships as a MINOR**, even when the user-facing behavior is
  unchanged, to keep the SemVer signal honest for the `cfg(fuzzing)` lib consumers + downstream
  pin-bumpers. The conservative, precedent-aligned call for a secret-hygiene type-migration cycle
  that touches a `pub mod` (seedqr) — even internals-only — is **MINOR 0.68.0**, matching how
  v0.10.1 was bumped. **R0 decision point:** confirm MINOR vs NO-BUMP. The argument FOR NO-BUMP
  (nothing public changed shape) is real; the argument FOR MINOR (precedent + the cycle co-ships a
  sibling-lane pin that itself may be MINOR, and a hygiene cycle is a meaningful release signal) is
  the recommended default. **Recommend MINOR 0.68.0**; flag for R0 ratification.

**Release version-sites (MEMORY `project_toolkit_release_ritual_version_sites`):** on bump, update
**BOTH READMEs** + `fuzz/Cargo.lock` (silent-drift sites) + the root `Cargo.toml` version + any
`install.sh` self-pin. Re-run full `cargo test -p mnemonic-toolkit` + the fuzz lock sync BEFORE tag.

---

## 5. Behavior / wire / `--json` — UNCHANGED (confirmed)

The printed secret value is **byte-identical**; only the in-memory carrier type changes.

- **Text path:** `SecretString: Display` (`secret_string.rs:54`) and `Deref<Target=str>` (`:32`)
  render the inner string verbatim. `writeln!(stdout, "{output}")` (`derive_child.rs:304`) emits the
  same bytes whether `output: String` or `output: SecretString`. `Zeroizing<String>` likewise
  `Deref`s to `str` for any `{x}` / `.as_str()` use.
- **`--json` path:** `SecretString`'s transparent `serde::Serialize` (`secret_string.rs:67`)
  serializes as the bare string — wire-shape byte-identical. (Note: the schema_mirror gate is
  flag-NAME parity only and does NOT gate `--json` wire-shape per CLAUDE.md — but we assert no
  wire change regardless, via the no-behavior-change RED test in §7.)
- **`derive_child` still emits `OutputClass::PrivateKeyMaterial`** (`derive_child.rs:307`) —
  unchanged; that advisory classification is independent of the carrier type.
- **No CLI flag / subcommand / dropdown-value change** → **no manual mirror update** (CLAUDE.md
  manual-cli-surface-mirror) and **no GUI schema_mirror update** (no clap-surface delta). Confirm in
  R0; both gates are flag-name-scoped and this cycle adds zero flags.

---

## 6. ms-codec sibling-lane pin coordination (Lane M dependency)

- **Live state:** toolkit pins `ms-codec = "0.5"` (Cargo.toml:29), resolving **0.5.0** (root
  `Cargo.lock`). The toolkit consumes `ms_codec::Payload` / `ms_codec::decode` at `inspect.rs:171`
  (`InspectPayload::Ms1 { payload: ms_codec::Payload }`, `inspect.rs:162`) and `bundle.rs:2529`
  (self-check). The `"0.5"` caret admits any `0.5.x`.
- **The dependency to flag (NOT plan here):** Lane M ships an ms-codec MINOR with a public
  `InspectReport` / `Payload` shape change. **IF** Lane M's release stays inside `0.5.x`
  (`0.5.0 → 0.5.1+`), the `"0.5"` caret already admits it → **no toolkit pin edit needed**, but a
  **`Payload`-struct-shape change could still require a small toolkit-side adjustment** at the two
  consume sites if the variant/field shape moved. **IF** Lane M crosses to `0.6.0` (MINOR under
  0.x SemVer = caret-crossing), the toolkit pin `"0.5"` MUST widen to `"0.6"`/`">=0.5, <0.7"` and
  the lockfiles re-resolve — this **rides with Lane T's MINOR at SHIP**.
- **Action for THIS lane:** treat the ms-codec pin bump as a **SHIP-time coordination item, not a
  P-phase deliverable**. At SHIP: (a) read Lane M's actual released ms-codec version; (b) if it
  crosses the caret, bump the pin + re-resolve both lockfiles; (c) `cargo build` + full
  `cargo test -p mnemonic-toolkit` to surface any `Payload`-shape break at `inspect.rs`/`bundle.rs`;
  (d) apply the minimal toolkit-side adjustment if the report-struct shape changed. **Do NOT plan
  ms-codec internals here** (Lane M owns them). See Open-Q3 — the prompt's "0.39 → 0.40" is the
  md-codec version, not ms-codec; R0 must confirm the real Lane M target version.

---

## 7. Per-slug RED tests (written BEFORE impl, per TDD)

> All tests live in the BIN target → `cargo test -p mnemonic-toolkit` (NOT a `--test` target alone;
> per MEMORY `feedback_r0_review_run_full_package_suite`, R0 reviews run the FULL package suite).

### Slug 1 — `bip85` / `derive_child`

1. **Type-level (compile-fence):** a `const _: fn() -> SecretString = || bip85::format_hd_seed_wif(…)`
   style assertion, or a unit test that binds `let _o: SecretString = derive_child …` — RED until
   the return types flip. (Mirror the existing bip85 type rows at `lint_zeroize_discipline.rs:93`.)
2. **Debug-redaction:** construct each `format_*` output and assert `format!("{:?}", out)` does
   **NOT** contain the secret substring and **DOES** contain the redaction marker (mirror
   `secret_string.rs:117 debug_redacts_the_secret`). Proves the chosen `SecretString` (not bare
   `Zeroizing`) — guards the cycle-14 leak class.
3. **No-behavior-change e2e:** run `derive-child` for each application (phrase/wif/xprv/hex/
   password/dice) via the CLI harness; assert stdout bytes are **identical** to a pinned golden
   (pre-change vector) AND `--json` payload is byte-identical. (The known-answer vectors already in
   `bip85.rs:365-374` tests anchor the values.)

**M-1 — EXISTING bip85 KAT asserts need a mechanical deref to stay GREEN.** Flipping the seven
`format_*` returns to `SecretString` breaks the existing in-module known-answer asserts, because
`SecretString` has **no `PartialEq<&str>`**: `assert_eq!(pwd, "dKLoepugzdVJvdL56ogNV")`
(`bip85.rs:384`, `pwd_base64_matches_spec`), `assert_eq!(pwd, "_s\`{TW89)i4\`")` (`:390`,
`pwd_base85_matches_spec`), and `assert_eq!(rolls, "1,0,0,2,0,1,5,5,2,4")` (`:399`,
`dice_d6_10_rolls_matches_spec`). These are a **mechanical deref update** — `assert_eq!(pwd.as_str(),
"…")` (or `assert_eq!(&*pwd, "…")`) — applied as part of the type-flip so the no-behavior-change
suite stays GREEN; they are NOT new RED tests. (Any `.split(',')` / `.parse()` consumers in the
dice/test paths keep working unchanged via `Deref<Target=str>`.)

### Slug 2 — `electrum`

4. **Type-level:** assert `phrase_to_entropy(…)` returns `Zeroizing<Vec<u8>>` (RED: currently
   `Vec<u8>`). Assert (source-grep) that `electrum.rs` wraps the normalize results at the
   consumption boundary — `Zeroizing::new(normalize_electrum(` present at `:149`/`:244` — and that
   `entropy_to_phrase` STILL returns `Result<String, _>` (GUARD, stays GREEN — I-1: the public
   return is NOT widened). Do NOT assert any signature change on `wordlists::normalize_electrum`; per
   M-4 it stays `-> String` (cross-module, other callers) and a guard-grep should confirm
   `pub(crate) fn normalize_electrum(s: &str) -> String` is unchanged in `wordlists/mod.rs`.
5. **No-clone-out-of-Zeroizing:** a source-grep test (mirror `lint_zeroize_discipline` evidence
   anchors) asserting `electrum.rs` contains `Ok(acc)` and NOT `Ok((*acc).clone())` at the
   `phrase_to_entropy` return.
6. **No-behavior-change e2e:** Electrum native-seed import KAT — assert the derived bip32 seed /
   address output is byte-identical to a pinned golden (the existing electrum derivation vectors).

### Slug 3 — `seedqr`

7. **Public-API-stability fence:** assert `seedqr::decode`/`encode`/`encode_compact`/`decode_compact`
   STILL return `Result<String, SeedqrError>` (GUARD: this test must stay GREEN — it proves we did
   NOT widen the public return, i.e. did NOT trigger a SemVer break).
8. **Internal-scratch evidence:** source-grep test asserting `seedqr.rs` scratch is `Zeroizing`-wrapped
   — `stripped`/`normalized`/`digits` as `Zeroizing<String>`, the per-word `words` as
   `Vec<Zeroizing<String>>`, and the `decode_compact` hex-decoded `bytes` as `Zeroizing<Vec<u8>>`
   (M-3; mirror the lint evidence-anchor style).
9. **No-behavior-change e2e:** SeedQR `encode`/`decode`/`encode-compact`/`decode-compact`
   round-trip KAT — output byte-identical to pinned goldens (existing `seedqr.rs:262-511` vectors).

---

## 8. Zeroize-discipline lint-row updates (`tests/lint_zeroize_discipline.rs`)

The lint partitions every secret-bearing `src/*.rs` into **declared rows** (`ZEROIZE_ROWS`) or
**`NON_ROW_SECRET_FILES`** allowlist; `every_secret_bearing_src_file_is_declared_or_allowlisted`
(`:492`) fails if a secret-bearing file is in neither. `SECRET_FILE_FLOOR = 37` (`:462`).
`ZEROIZE_ROWS.len()` is bounded `18..=60` (`:382`).

**I-2 — the count-guard upper bound WILL be breached; we widen it (option b).** The LIVE row count
is already **54** (`grep -c 'ZeroizeRow {' tests/lint_zeroize_discipline.rs` against
`79100a66`), NOT a comfortable headroom — only **6 rows** of slack to the `60` ceiling. This cycle
adds ~6–7 rows (bip85 `format_*` `SecretString` returns, `derive_child` `output`, electrum
norm-scratch + `phrase_to_entropy` move-out, the NEW `src/seedqr.rs` `source_file` row). `54 + 7 =
61 > 60` → `assert!((18..=60).contains(&n))` goes **RED**, breaking the FULL suite. So the prior
claim *"bounded 18..=60 so adding rows is safe"* is **false** as written and is corrected here.

**Resolution (chosen: option b — widen the bound).** Mirroring the documented prior widen (the
inline rationale already records a `36 → 52` widen during the source→declared completeness cycle),
this cycle **widens `(18..=60)` → `(18..=66)`** in `tests/lint_zeroize_discipline.rs:382` and updates
the inline rationale comment to record the cycle-15t bump. Exact lint edit:

```rust
// tests/lint_zeroize_discipline.rs ~:382  (hand-edit; NEVER cargo fmt the toolkit)
assert!(
    (18..=66).contains(&n),                       // was (18..=60)
    "ZEROIZE_ROWS row count = {n}; expected 18..=66 (upper bound widened in cycle-15t \
     toolkit derived-output zeroize: +~7 rows for bip85 SecretString returns, derive_child \
     output, electrum norm-scratch + phrase_to_entropy move-out, and the new src/seedqr.rs \
     source_file row; 54 + 7 = 61, ceiling raised to 66 for headroom). \
     Survey §1 toolkit table is the canonical reference."
);
```

Option (a) — consolidate to land ≤6 new rows and stay ≤60 — is **rejected**: it would force
under-declaring genuinely-distinct secret sites for a cosmetic ceiling, defeating the lint's
completeness purpose. Option (b) is the recommended path and the one this spec adopts.

Updates required (R0 to finalize exact labels/evidence anchors):

- **`bip85.rs`** — add a row for the new `SecretString` returns of the seven `format_*` fns
  (alongside the existing entropy-buffer rows at `:93/:98`). Evidence anchor: a `format_*` signature
  fragment, e.g. `"-> Result<SecretString, ToolkitError>"`.
- **`cmd/derive_child.rs`** — update/add a row for `let output: SecretString` (existing rows at
  `:161/:166`). Evidence: `"SecretString"` or the `output` binding.
- **`electrum.rs`** — extend the existing accumulator rows (`:183/:188`) with rows for the
  `norm_phrase`/`norm_pp` `Zeroizing<String>` locals + the `phrase_to_entropy` `Ok(acc)`
  move-out + the consumption-boundary `Zeroizing::new(normalize_electrum(...))` wraps (`:149`/`:244`)
  + the `entropy_to_phrase` per-candidate `Zeroizing<String>` scratch. **NB (M-4):** do NOT add a row
  claiming a widened `normalize_electrum` return — that fn lives in `crate::wordlists` and stays
  `-> String`; the electrum rows anchor on the `Zeroizing::new(` wrap at the call site, not a helper
  signature change.
- **`seedqr.rs` (the LIB module, `src/seedqr.rs`)** — this file is currently NOT a `ZEROIZE_ROWS`
  `source_file` (only `src/cmd/seedqr.rs` is, at `:310`). After this cycle `src/seedqr.rs` carries
  `Zeroizing` scratch → it becomes secret-bearing-and-declared. **Add a `src/seedqr.rs` row** (or
  confirm it is covered by an existing allowlist entry) so
  `every_secret_bearing_src_file_is_declared_or_allowlisted` stays GREEN. Evidence:
  `"Zeroizing::new"` on the scratch.
- **Floor check:** `SECRET_FILE_FLOOR` likely unchanged (these files already counted as
  secret-bearing). R0: re-run `every_secret_bearing_src_file_is_declared_or_allowlisted` to confirm
  no file falls off; bump `SECRET_FILE_FLOOR` only if a genuinely-new secret-bearing file appears
  (none expected — all three files pre-exist).

> **NEVER `cargo fmt` the toolkit** (mlock.rs is permanently fmt-exempt — MEMORY
> `project_g6_fmt_exemption_and_asymmetric_pin`). Hand-format edits.

---

## 9. Mandatory R0 gate

Per CLAUDE.md: **NO code before R0 GREEN (0C/0I).** This brainstorm spec feeds the architect R0
loop; after R0 converges, a separate plan-doc also passes its own R0 loop; only THEN does a single
implementer subagent execute in a worktree, TDD, with per-phase R0 reviews persisted verbatim to
`design/agent-reports/`, then a mandatory whole-diff adversarial execution review before SHIP.
Reviewer-loop continues after EVERY fold (folds can introduce drift). Persist each review verbatim
BEFORE the fold-and-commit step.

**R0 must specifically ratify:** (a) the §2 per-site SecretString-vs-Zeroizing table — especially
the slug-3 decision to keep public `String` returns and wrap only internals; (b) the §4 MINOR-vs-
NO-BUMP call; (c) the §6 ms-codec pin coordination + Open-Q3 (the prompt's "0.39" = md-codec, not
ms-codec); (d) Open-Q1 (join-into-return clone tension) and Open-Q2/Q4.

---

## Appendix — citation verification log (vs `origin/master 79100a66`)

All re-grepped live this session:
- `bip85.rs`: `format_bip39_phrase:72/90`, `format_hd_seed_wif:103/121`, `format_xprv_child:131/154`,
  `format_hex_bytes:163/170`, `format_password_base64:181/189`, `format_password_base85:196/204`,
  `format_dice_rolls:222/268/273` — all return `Result<String, ToolkitError>`. ✓
- `cmd/derive_child.rs:224` `let output = match`, `:304` `writeln!(stdout, "{output}")`, `:307`
  `OutputClass::PrivateKeyMaterial`. ✓
- `electrum.rs`: `normalize_text_electrum:79`, `electrum_seed_to_bip32_seed:97` + `norm_phrase:98` +
  `norm_pp:99`, `phrase_to_entropy:138` + `words:147` + `acc:161` + `Ok((*acc).clone()):175`,
  `entropy_to_phrase:182` + `phrase = words.join:215`, `normalize_phrase_for_hmac:243`,
  `strip_cjk_internal_whitespace:249`. ✓
- `seedqr.rs`: `pub fn decode:96` + `stripped:98` + `phrase=words.join:131`; `pub fn encode:141` +
  `normalized:155`; `pub fn encode_compact:182` + `normalized:192` + `Ok(hex::encode(m.to_entropy())):196`;
  `pub fn decode_compact:205` + `stripped:206` + `Ok(m.to_string()):218`. ✓
- `cmd/seedqr.rs` consumer re-wraps: `:177/180/194/198/244/247/261/264` all `Zeroizing::new`. ✓
- Module pub-ness: `lib.rs:111 pub mod seedqr` (always-public); `bip85`/`derive_child`/`electrum`
  absent from `lib.rs` (bin-private, `main.rs:4/14`, `mod cmd:6`). ✓
- Precedents: `silent_payment.rs:286-287` + `nostr.rs:235` `SecretString::new`; lint rows
  `lint_zeroize_discipline.rs:252/261`. ✓
- `secret_string.rs`: `pub struct SecretString(Zeroizing<String>):23`, redacting `Debug:61`,
  `Display:54`, `Deref:32`, `Serialize:67`, `PartialEq:46`. ✓
- Lint scaffolding: `ZEROIZE_ROWS:48`, count-bound `18..=60:382`, `SECRET_FILE_FLOOR=37:462`,
  partition test `:492`. ✓
- Pins: `ms-codec = "0.5":29` (NOT 0.39 — that's `md-codec = "0.39":36`); root `Cargo.lock`
  ms-codec resolves `0.5.0`. ✓
- SemVer precedent: `mnemonic-toolkit-v0.10.1` MINOR for pub secret-field-type migration
  (FOLLOWUPS `resolved-slot-derived-account-zeroizing-field`). ✓
