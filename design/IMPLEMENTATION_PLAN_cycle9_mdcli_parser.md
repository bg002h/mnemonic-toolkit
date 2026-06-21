# IMPLEMENTATION PLAN — cycle-9 — md-cli lexer/parser robustness cluster

**Findings:** M5 (funds) + M2 / M10 / M11 + advisory L4 / L19 / L7.
**Repo:** `descriptor-mnemonic` (md-codec descriptor codec + `md` CLI). **All fixes md-cli-side; md-codec UNTOUCHED.**
**Executes:** `design/BRAINSTORM_cycle9_mdcli_parser.md` — **R0-GREEN (0C/0I, round 2)**. Spec R0 reviews: `design/agent-reports/cycle9-spec-r0-round{1,2}-review.md`.
**Status of THIS doc:** DESIGN ONLY (the plan). It carries its OWN mandatory opus-architect **R0 reviewer-loop to 0C/0I BEFORE any implementation** (CLAUDE.md Conventions bullet 1). No code, no implementer dispatch, no tag, no publish until the plan is R0-GREEN.

---

## 0. Source SHA + execution model

### Source SHAs (re-grepped at write-time per the grep-at-write-time rule)

| Repo | SHA | Role |
|---|---|---|
| **descriptor-mnemonic** | **`origin/main` = `836faf87c3d82b119a9f0f5c6589a7db1f8613a4` (`836faf8`)** — md-codec 0.38.0 / **md-cli 0.8.1** | cycle-9 base; **implementation branches off `origin/main`** |
| mnemonic-toolkit | **`origin/master` = `d6398b57`** (manual mirror live here) | L7 manual-mirror lockstep target (§Phase 4) + cycle-9 design-trail home |

> **STALE-LOCAL CAVEAT (load-bearing).** descriptor-mnemonic's LOCAL `main` is behind `origin/main`; `template.rs` grew to **1972 lines**. **DO NOT branch off local `main` and DO NOT trust local line numbers.** Branch off `origin/main` (`836faf8`); all line numbers below are verified against `git show origin/main:<path>` at this write. The bughunt-report citations are pre-cycle-1/2 snapshots — superseded by this table.

### Citation re-verification (against `836faf8`, this write)

| Finding | File@origin/main | Line(s) verified | Live content (excerpt) |
|---|---|---|---|
| **M5** lexer regex | `crates/md-cli/src/parse/template.rs` | **`:55`** | `Regex::new(r"@(\d+)((?:/\d+'?)*)(?:/<([^>]*)>)?(/\*(?:'|h)?)?")` |
| **M5** substitution regex | `…/template.rs` | **`:498`** | `Regex::new(r"@(\d+)((?:/\d+'?)*)(?:/<[0-9;]+>)?(?:/\*(?:'|h)?)?")` |
| **M5** group-3 validator (H13, DO NOT TOUCH) | `…/template.rs` | **`:77-110`** | `;`-split; rejects `'`/`h`-suffixed + non-`u32` alts; accepts bare ints |
| **M5** stitch (`parse_template` `Ok(Descriptor{…})`) | `…/template.rs` | **`:1915-1921`** | `n`@1916, `path_decl`@1917, `use_site_path`@1918 (lexer), `tree`@1919 (substituted), `tlv`@1920 |
| **M2** n-cast `(…+1) as u8` | `…/template.rs` | **`:307-312`** | `let n = (by_i.keys().max().copied()…? as usize + 1) as u8;` |
| **M2** density loop / panic-index | `…/template.rs` | **`:313`** (loop) / **`:320`** (`by_i[&0]`) / **`:324`** (`by_i[&i]`) | bracket-index panics on absent key |
| **M10** classifier `ctx_for_template` | `…/template.rs` | **`:1925-1932`** | `wpkh(`/`pkh(`/`sh(wpkh(`→SingleSig; `else`→MultiSig (`tr(` falls here) |
| **M10** synthetic 2nd consumer | `…/template.rs` | **`:458`** (`fn synthetic_xpub_for(i,ctx)`), called **`:506`** | depth 3 SingleSig / 4 MultiSig — harmless, address-neutral |
| **M10** depth gate | `crates/md-cli/src/parse/keys.rs` | **`:67-77`** | `SingleSig⇒3 / MultiSig⇒4`; rejects mismatched depth |
| **M11** payload copy (no point check) | `…/keys.rs` | **`:78-79`** | `let mut payload=[0u8;65]; payload.copy_from_slice(&bytes[13..78]);` (pubkey = `bytes[45..78]`) |
| **L4** repair advisory | `crates/md-cli/src/cmd/repair.rs` | **`:156-159`** | unconditional `OutputClass::Template` |
| **L19** encode advisory (JSON) | `crates/md-cli/src/cmd/encode.rs` | **`:73-76`** | unconditional `OutputClass::Template` |
| **L19** encode advisory (text) | `…/encode.rs` | **`:110-113`** | unconditional `OutputClass::Template` |
| **L7** stale epilog | `crates/md-cli/src/main.rs` | **`:241`** (`after_long_help` for `Repair`) | "Non-chunked single-string md1 … are rejected with a wire-format error …" |
| **L7** toolkit-manual MIRROR | `mnemonic-toolkit/docs/manual/src/40-cli-reference/42-md.md` | **`:367-379`** @ toolkit `d6398b57` | "### v0.6.0 limitation: chunked-form only" … "rejected with a wire-format error" |
| helper `is_wallet_policy()` | `crates/md-codec/src/encode.rs` | **`:50-52`** (`pub`) | `matches!(&self.tlv.pubkeys, Some(v) if !v.is_empty())` |
| M11 secp256k1 already-dep | `…/template.rs` | **`:461`** | `use bitcoin::secp256k1::{Secp256k1, SecretKey}` → no new Cargo dep |
| md-cli version site | `crates/md-cli/Cargo.toml` | **`:3`** (`version = "0.8.1"`), **`:28`** (`md-codec = { path="../md-codec", version="=0.38.0" }`) | bump `:3` → `0.9.0`; `:28` UNCHANGED |
| root CHANGELOG | `CHANGELOG.md` | crate-prefixed (`## md-cli [0.8.1] — 2026-06-21`) | add `## md-cli [0.9.0]` entry |

### Execution model

- **Single implementer in a worktree off `origin/main`** (NOT parallel re-implementations) — per CLAUDE.md ultracode phase-3 policy.
- **TDD, RED-first** at every phase: write the failing test(s) → confirm RED → implement → confirm GREEN.
- **Per-phase gate = the FULL package suite, not targeted `--test` targets** (memory rule `feedback_r0_review_run_full_package_suite`): each phase ends with
  - `cargo test -p md-cli` (FULL — CLI/parse phases ripple into help/version/argv lints outside any one target), AND
  - `cargo clippy --all-targets -- -D warnings` (CI runs `cargo clippy --workspace --all-targets -- -D warnings` @ `.github/workflows/ci.yml:47`).
- **`cargo fmt` IS REQUIRED in this repo.** CI enforces `cargo fmt --all --check` (`.github/workflows/ci.yml:59`). **This is NOT the toolkit's mlock.rs exemption** — descriptor-mnemonic has no fmt exemption. Run `cargo fmt --all` before each commit and confirm `cargo fmt --all --check` is clean. (Distinct from the toolkit `NEVER cargo fmt mlock.rs` rule, which does not apply to this repo.)
- **Phase order is LOAD-BEARING: 1 → 2 → 3 → 4, with M5 LAST** (spec D13: simplest→hardest; M5 is the funds crux + H13-collision and must land on the M2-bounded `n` with the five H13 guard tests loaded). Phases are **disjoint** (see §Disjointness).
- **Per-phase architect review persists verbatim** to `design/agent-reports/cycle9-phase-N-<round>-review.md` BEFORE the fold-and-commit step (CLAUDE.md). Per-phase reviewer-loop continues until 0C/0I.
- **Stage paths explicitly** (no `git add -A`).
- **Resolved decisions are FINAL** (spec §11 D1-D13). Do NOT re-decide — reference them.

### Disjointness (why these phases don't collide)

| Phase | Function(s) touched | H13 collision? |
|---|---|---|
| **P1** M11 / M10 / L4 / L19 / L7 | `keys.rs:78` (M11), `template.rs:1925-1932` (M10 classifier), `repair.rs:156` + `encode.rs:73,110` (L4/L19), `main.rs:241` (L7) | **NONE** — H13 never touched any of these |
| **P2** M2 | `resolve_placeholders` (`template.rs:307-324`) — the n-cast + density loop | **NO collision** with the M5 regexes; same file, distinct function/logic; M2 lands FIRST so M5 builds on a bounded `n` (spec §3.2) |
| **P3** M5 | `lex_placeholders` (regex `:55`, validator `:77-110`) + `substitute_synthetic` (regex `:498`) + `parse_template` stitch (`:1915-1921`) | **THE H13 home** — touch ONLY suffix-residue text OUTSIDE `<…>`; group-3 capture + validator + strip class are byte-for-byte UNCHANGED (§Phase 3) |

**Why M5 last:** M5 is the only funds finding and the only H13-collision; it lands on the already-bounded `n` (M2 in P2) and is reviewed with all five H13 guard tests + the full suite green. P1/P2 de-risk the file so the M5 diff is minimal and reviewable in isolation.

---

## Phase 1 — independent fixes (no H13 collision)

**Goal:** land the five non-colliding fixes (M11, M10, L4, L19, L7). RED test per finding first. One coherent phase; may be one commit or grouped sub-commits (M11; M10; L4+L19 one commit; L7) — implementer's call, but each finding RED-first.

### P1.a — M11: off-curve xpub admitted at parse (no secp256k1 point check) — spec §3.4, D7

**Mechanism @ `836faf8` (`keys.rs:78-79`):** `parse_key` validates base58check / 78-byte length / version / depth, then `let mut payload=[0u8;65]; payload.copy_from_slice(&bytes[13..78]);` — NO check that the compressed pubkey (`bytes[45..78]`) is a valid secp256k1 point. An off-curve / all-zero pubkey passes intake, lands in the Pubkeys TLV, fails only later at `derive_address`.

**Fix (insert point check BEFORE the copy at `keys.rs:78`):**
```rust
bitcoin::secp256k1::PublicKey::from_slice(&bytes[45..78])
    .map_err(|e| CliError::BadXpub {
        i,
        why: format!("xpub public key is not a valid secp256k1 point: {e}"),
    })?;
```
`bitcoin::secp256k1` is already reachable (`template.rs:461`) — **no new Cargo dep.** A valid BIP-32 serialized xpub's `bytes[45..78]` is a compressed on-curve point, so this rejects only genuinely-corrupt keys.

**RED test (then GREEN):** mutate a real depth-3/4 xpub's pubkey bytes to an off-curve value (e.g. all-zero `bytes[45..78]`), re-base58check-encode → `parse_key` currently passes → after fix rejects with `CliError::BadXpub` ("not a valid secp256k1 point").
**Positive control (stays GREEN):** the real `XPUB_DEPTH4` test fixture (`keys.rs`) and a real depth-3 BIP-86 account xpub still parse.

### P1.b — M10: BIP-86 single-key `tr(@0)` falsely rejected — spec §3.3, D6

**Mechanism @ `836faf8` (`template.rs:1925-1932`):** `ctx_for_template` maps only `wpkh(` / `pkh(` / `sh(wpkh(` → SingleSig(depth-3); **everything else — including key-path `tr(@0/<0;1>/*)` — falls to `else` → MultiSig(depth-4).** The depth gate (`keys.rs:67-77`) then rejects a depth-3 BIP-86 account xpub with "expected depth 4 … got 3". **BIP-86 fact (verified):** a single-key P2TR account xpub is at `m/86'/0'/0'` = **depth 3**. Depth byte is advisory (discarded in derivation) → relaxing the classifier cannot corrupt an address → **pure availability, not wrong-address.**

**Fix (extend the SingleSig branch at `template.rs:1925-1932`):** also classify a **bare single-key key-path taproot** as SingleSig depth-3 — `tr(` whose argument list has **NO top-level `,`** (a comma separates the internal key from a script tree or separates `multi_a` cosigners) **AND NO `{`**. Such a form is `tr(@i[/path])` = BIP-86 single-key P2TR.

**Over-acceptance guard (MUST confirm — spec's noted hazard):** a `tr(` WITH a script tree (`tr(NUMS,{...})`, `tr(@0,{...})`, `tr(NUMS,multi_a(2,@0,@1,@2))`) still routes to MultiSig(depth-4). Scope is **strictly** "`tr(` + exactly one `@i` + no top-level `,` + no `{`". Do NOT relax the depth gate (`keys.rs:67-77`) — that would also loosen depth-4 script-path-tr keys (spec's rejected alternative).

**SECOND consumer (note — harmless, no extra fix):** `ctx_for_template`'s output also feeds `synthetic_xpub_for(i, ctx)` (`template.rs:458`, called `:506`), which builds the throwaway synthetic xpub at depth 3 (SingleSig) / 4 (MultiSig). Flipping bare `tr(@0)`→SingleSig makes its synthetic xpub depth-3 — **harmless/address-neutral** (synthetic discarded after `key_map`; depth advisory). The `tr_key_only` test pins `ScriptCtx::MultiSig` **explicitly** (not via `ctx_for_template`) so it does not break. The implementer must not miss this consumer when editing the classifier.

**RED→GREEN:** `tr(@0/<0;1>/*)` + a depth-3 BIP-86 account xpub via `md encode --key @0=<depth-3 xpub>` currently rejected → now ACCEPTED. Assert `ctx_for_template("tr(@0/<0;1>/*)") == ScriptCtx::SingleSig`.
**Positive control:** `tr(@0)` / `tr(@0/*)` classify SingleSig.
**NEGATIVE (over-acceptance guard):** `tr(@0,{...})`, `tr(NUMS,multi_a(2,@0,@1,@2))` still classify MultiSig → a depth-3 key in those forms still rejects.
**Existing tr tests stay GREEN:** `tr_with_and_v_verify_older_inheritance`, `tr_tap_leaf_bare_pk_on_wire`, `tr_multi_branch_three_leaf_right_unbalanced`, `tr_key_only` (`template.rs:1195` — Minor-4 grep note; the others by name).

### P1.c — L4 + L19: watch-only md1 mislabeled "keyless template (no keys)" — spec §3.5, D8

**Mechanism @ `836faf8`:** three emit sites unconditionally pass `OutputClass::Template` — `repair.rs:156-159`, `encode.rs:73-76` (`--json`), `encode.rs:110-113` (text) — but all three run on **arbitrary** md1, including **wallet-policy** md1 whose Pubkeys TLV carries watch-only xpub/pubkey entries (the natural `md encode --key …` / `md repair` input). Model to mirror: `md address` emits `WatchOnly` soundly because it is guarded to wallet-policy inputs (`address.rs`); `repair`/`encode` accept BOTH classes and must **branch**.

**Fix (ONE branch, THREE sites — fold into one commit):**
```rust
let class = if descriptor.is_wallet_policy() {
    OutputClass::WatchOnly
} else {
    OutputClass::Template
};
crate::output_advisory::emit_output_class_advisory(class, &mut std::io::stderr());
```
`is_wallet_policy()` = `md_codec::Descriptor::is_wallet_policy` (`md-codec/src/encode.rs:50-52`, already `pub`). `OutputClass` lives at `crates/md-cli/src/output_advisory.rs` (crate root `src/`, NOT `src/cmd/`); the `WatchOnly` variant exists (cross-repo byte-parity-locked). **md-codec UNTOUCHED.**

**Implementation note (R0-round-1 I-2) — `repair.rs` binding rename.** The decoded `Descriptor` is directly usable at `encode.rs:45` (live `let mut descriptor = parse_template(...)`), but at `repair.rs:118` it is bound underscore-prefixed as `let (_descriptor, details) = match md_codec::decode_with_correction(&str_refs) { … }` — intentionally unused today, so it is NOT in scope as `descriptor` at the advisory site (`:156-159`). The `descriptor.is_wallet_policy()` snippet above therefore requires the `repair.rs` binding be **un-underscored: rename `_descriptor` → `descriptor` AND use it** in the new branch. (An underscore-prefixed var used in an expression, OR an unused non-underscore var, both trip CI's `cargo clippy --workspace --all-targets -- -D warnings`; rename-and-use clears both.) `encode.rs:45` already binds `descriptor` usably — only `repair.rs` needs the rename. Do NOT introduce a second decode.

**RED→GREEN — regression home = existing `crates/md-cli/tests/cli_output_class.rs`** (already asserts `Template`/`WatchOnly` lines + cross-repo `byte_parity_advisory_lines` @ `:23`):
- `md repair` on a **wallet-policy (keyed)** md1 → stderr advisory `WatchOnly`, not `Template`; on a **keyless template** md1 → still `Template`.
- `md encode --key @0=<xpub> …` (both `--json` and text) → `WatchOnly`; keyless `md encode <template>` → `Template`.
- Existing `repair_emits_template` (`:241`), `encode_text_emits_template` (`:74`), `encode_json_emits_template` (`:94`) cover the keyless path; ADD keyed-path assertions alongside (do not delete the keyless ones).
- **Confirm `byte_parity_advisory_lines` (`:23`) stays GREEN** — the fix touches call-sites (which `OutputClass` is passed), NOT the advisory strings, so cross-repo parity is preserved.

### P1.d — L7: stale `md repair --help` epilog — spec §3.6, D9

**Mechanism @ `836faf8` (`main.rs:241`):** the `Repair` `after_long_help` string contains *"Non-chunked single-string md1 … are rejected with a wire-format error — use `md decode` …"*. This is **FALSE** since md-codec **v0.35.0** added single-string auto-dispatch (non-chunked md1 are now repaired). The FOLLOWUP `md-codec-decode-with-correction-supports-non-chunked-md1` is already `RESOLVED in md-codec-v0.35.0` (`descriptor-mnemonic/design/FOLLOWUPS.md:123`).

**Fix:** delete / rewrite the false "INPUT FORMAT … rejected with a wire-format error" sentence in the `:241` epilog. State that BOTH chunked and non-chunked md1 are accepted. **KEEP the "ATOMIC SEMANTICS (multi-chunk)" note** (still accurate). **No FOLLOWUP status flip** (slug already RESOLVED).

**RED→GREEN:** `md repair --help` output no longer contains "rejected with a wire-format error" / "Non-chunked … are rejected" (assert absent); assert the corrected phrasing present.
**Manual mirror is deferred to Phase 4** (cross-repo, toolkit — docs-only, NOT lint-gated; see Phase 4).

**Phase-1 gate:** `cargo fmt --all` → `cargo fmt --all --check` clean → `cargo clippy --all-targets -- -D warnings` clean → **FULL `cargo test -p md-cli` GREEN.** Per-phase architect review → persist verbatim to `design/agent-reports/cycle9-phase-1-<round>-review.md` → fold-loop to 0C/0I → commit (stage explicit paths).

---

## Phase 2 — M2: `@255` placeholder-count u8 overflow → panic — spec §3.2, D5

**Mechanism @ `836faf8` (`template.rs:307-324`):** `let n = (by_i.keys().max().copied()…? as usize + 1) as u8;` (`:307-312`). For max index 255: `(255usize + 1) as u8 == 0`. The density loop `for i in 0..n` (`:313`) becomes `0..0` → skipped. Then `let at0 = by_i[&0];` (`:320`, bracket index) panics `"no entry found for key"` when `@0` is absent (`wpkh(@255/*)`); when `@0` IS present, `n=0` flows silently into the encoder (key count collapses to 0). The lexer accepts `@0..=255` (u8 parse) so `@255` is valid-per-lexer.

**Fix (bound BEFORE the cast; defense-in-depth on the index):**
1. **Primary — bound the max index before `as u8` (`:307-312`).** Compute `max` as the raw value; if `max >= 255` (i.e. `max + 1` would not fit a `u8` placeholder count), return
   `CliError::TemplateParse(format!("placeholder index @{max} out of range: at most @254 (a template may declare at most 255 keys, @0..=@254)"))`.
   After this guard, `(max + 1) as u8` cannot wrap.
2. **Defense-in-depth — checked gets.** Replace bracket indexes `by_i[&0]` (`:320`) and `by_i[&i]` (`:324`) with `by_i.get(&…).ok_or_else(|| CliError::TemplateParse(…))?`. Belt against any future skip-density path.

**This is in `resolve_placeholders`, NOT in the two regexes M5 edits** — no collision with M5 beyond same-file proximity. M2 lands FIRST (this phase) so M5 (P3) builds on a bounded `n`.

**RED→GREEN:** `wpkh(@255/*)` currently **panics** at `by_i[&0]` → after fix returns a typed `CliError::TemplateParse` ("at most @254"). Use `parse_template` / `resolve_placeholders` directly; assert `is_err()` with the range message, **no panic**.
**Boundary:** `@254` still ACCEPTS (dense `@0..=@254`); `@255` rejects. `@0`-present case `wsh(multi(2,@0/*,@255/*))` — no silent `n=0` (rejected at the bound).

**SemVer:** PATCH-class on its own (a previously-panicking input now errors cleanly; no newly-accepted input). Rides the cycle MINOR umbrella.

**Phase-2 gate:** same as Phase 1 (`fmt` → `fmt --check` → `clippy -D warnings` → FULL `cargo test -p md-cli`). Per-phase architect review → `design/agent-reports/cycle9-phase-2-<round>-review.md` → fold-loop 0C/0I → commit (explicit paths).

---

## Phase 3 — M5: post-multipath suffix → divergent card (THE H13-CRITICAL FIX — LAST) — spec §3.1, D1/D2/D3/D4

> **FUNDS-CRITICAL. This phase carries spec §3.1.4 verbatim (below). The whole-diff review's #1 focus.**

### P3.1 — The mechanism (traced @ `836faf8`)

`parse_template` (`:1874-1922`) builds the final `Descriptor` from **two independently-computed views with NO cross-validation**:
```rust
let occs     = lex_placeholders(template)?;              // lexer view
let resolved = resolve_placeholders(&occs)?;             // resolved.use_site_path (multipath here)
let (substituted, key_map) = substitute_synthetic(...)?; // substituted string
let ms_desc  = MsDescriptor::from_str(&substituted)?;    // structural view
let tree     = walk_root(&ms_desc, &key_map)?;
...
Ok(Descriptor {                  // :1915
    n: resolved.n,               // :1916
    path_decl: resolved.path_decl,// :1917
    use_site_path: resolved.use_site_path, // :1918  ← from LEXER
    tree,                        // :1919  ← from SUBSTITUTED
    tlv,                         // :1920
})
```
For `wpkh(@0/<2;3>/0'/*)`: the lexer regex (`:55`) match **ends at `>`** — g3=`2;3`, g4 wildcard fails to match at `/0'/*` (sees `/0`) → matches EMPTY; the trailing `/0'/*` is NEVER consumed. Lexer records `multipath_alts=[2,3]` → `resolved.use_site_path.multipath = Some([2,3])`. Substitution (`:498`) matches the same `@0/<2;3>` span, leaves literal `/0'/*` → `wpkh(XPUB/0'/*)`, which `MsDescriptor::from_str` parses as **single-path, origin `/0'`, NO multipath**. **Result:** `Descriptor { use_site_path.multipath=Some([2,3]), tree=(no multipath, dropped /0') }` → recorded use-site path (chains 2&3) disagrees with the structural tree → **wrong derivation / wrong address, silently, exit 0.**

The `h`-in-origin sub-case (`@0/48h/…`): origin class is `/\d+'?` (apostrophe only, no `h`), so an `h` step is left unconsumed → malformed `XPUBh/…`. Same family; the residue reject covers it.

### P3.2 — DECISION (from spec D1/D2): REJECT (fail-closed), NOT canonicalize

**REJECT the multipath-not-last / unconsumed-suffix form with a typed `CliError::TemplateParse`.**

**Why REJECT, not canonicalize (spec §3.1.2):** BIP-389 *permits* post-multipath path steps (grammar: "the `/<NUM;NUM...>` tuple … Followed by zero or more /NUM... path elements"), so `xpub/<0;1>/0/*` is a *legal BIP-389 descriptor*. **The bug is NOT "BIP-389 forbids it."** It is an **md1 + md-cli `UseSitePath` representability limit** — `make_use_site_path` reads exactly `multipath_alts` + `wildcard_hardened`; there is no field for post-multipath fixed steps, so the form **cannot be faithfully represented**, and the current code silently produces a divergent card (worst funds outcome). Canonicalize would need a wire/`UseSitePath` field → md-codec change → breaks this cycle's "md-codec UNTOUCHED / no toolkit pin" invariant. REJECT (loud, actionable, exit≠0) matches the cycle-1 H13 precedent. Deferred-capability FOLLOWUP `md1-post-multipath-fixed-path-derivation-steps` is filed for genuine future demand. **Do NOT re-decide — D1/D2 are FINAL.**
**Do NOT over-claim in error text:** the reject is an md1/md-cli representability limit, NOT a BIP-389 prohibition (spec §3.1.4 protocol-fact correction).

### P3.3 — The exact change (do BOTH edits)

**Edit 1 — residue reject in `lex_placeholders` (the REAL fix; around `:55`/`:84`).** Detect any unconsumed path residue after the placeholder match. **Preferred form (spec §3.1.3 (i)): a tightened anchor + an explicit residue check in the capture loop.** In the loop, if `caps.get(0).end()` is followed by a `/` + path-like char (an unconsumed `/NUM` / `/NUMh` / `/NUM'` segment that is NOT the start of a new `@i`), return:
`CliError::TemplateParse("@{i}: derivation steps after the multipath group are not representable in md1; the multipath `<…>` must be the final derivation step before the wildcard")`.
This also catches the `h`-in-origin sub-case (the unconsumed `/48h` residue).
**The residue check MUST be placed AFTER the group-3 validator block (`:77-110`)** so that for a fused hardened/malformed body the group-3 validator fires FIRST (ordering guarantee, below). The residue check fires only on path chars OUTSIDE `<…>`.

**Edit 2 — cross-validation in `parse_template` (belt-and-suspenders; spec D4, CHECKABLE):** before constructing the final `Descriptor`, for each `@i` compare the lexer's `occ.multipath_alts.len()` (0 when no `<…>`, else alt-count — `Occurrence.multipath_alts: Vec<u32>`, `template.rs:27`) to the **multipath-step count of the substituted key's `DescriptorPublicKey`** (the substituted descriptor is `MsDescriptor<DescriptorPublicKey>`; `DescriptorPublicKey` carries derivation paths/multipath). **Name the join (Minor-2):** map each substituted key back to its `@i` via the **`key_map: BTreeMap<String,u8>`** (synthetic-xpub-string → `i`) that `substitute_synthetic` returns and `parse_template` binds at `template.rs:1883` — iterate the substituted descriptor's keys, look up `i` via `key_map`, and compare to `occs[i].multipath_alts.len()`. miniscript 13.0.0's `DescriptorPublicKey` exposes the multipath via `MultiXPub` / `is_multipath()` so the step-count is reachable. They must agree, and no substituted key may carry a fixed derivation step the lexer did not record. **Compare against the substituted `DescriptorPublicKey`, NOT `tree`** — `tree` (`walk_root`'s tag/key_index/leaf shape) carries no per-key multipath field (spec Minor-2/D4). On mismatch → `CliError::TemplateParse("internal: lexer/substitution divergence for @{i} — refusing to emit a divergent card")`. This makes any future regex drift fail-closed. (Edit-1 residue reject is the real fix; edit-2 is the checkable drift belt — NOT a vacuous always-true assertion.)

### P3.4 — H13-PRESERVATION ARGUMENT (carried VERBATIM from spec §3.1.4 — MANDATORY)

The M5 change preserves H13 because **it operates on a disjoint region of the input**:

- H13 governs the **content of the `<…>` multipath body** (group-3): permissive capture + strict reject of hardened/malformed alts. M5 governs **path text OUTSIDE `<…>`** — specifically a residual `/NUM…` segment *after* the closing `>` (or an `h`-bearing origin step *before* `<`). These are different spans; M5 adds a check on the suffix, it does **not** relax, widen, or touch the group-3 capture or its validator loop (`:90-110`) or the substitution strip class `[0-9;]` (`:498`).
- **Ordering guarantee:** for a hardened/malformed body like `@0/<0'';1>/0'/*`, H13's group-3 validator (`:90-110`) runs during the same capture iteration and returns the typed "hardened"/"not a bare unsigned integer" error **before** any M5 residue logic would matter — so H13's five guard tests (`:197`, `:211`, `:221`, `:228`, plus accept `:247`) stay GREEN unchanged. The plan MUST run those five tests (plus the full `cargo test -p md-cli` suite per the R0-full-suite memory rule) as a regression gate and assert all pass post-M5.
- **No over-rejection of valid multipath-last:** the canonical `@0/<0;1>/*` has NO post-`>` residue (the `/*` is the wildcard, consumed by group-4), so the M5 residue check does not fire → `lex_accepts_nonhardened_multipath` (`:247`) and `end_to_end_wsh_multi_template_only` (`:1939`) stay GREEN.

**Net:** M5 narrows the accepted grammar (rejects the previously-silently-mis-encoded suffix form) while H13's hardened/malformed reject and the valid-multipath accept are byte-for-byte preserved.

**Protocol-fact correction (for the record):** BIP-389 *allows* post-multipath path steps. The accurate framing: *md1 + md-cli's use-site model cannot represent post-multipath fixed steps, so md-cli rejects the form rather than silently encode a divergent card.* H13's hardened-multipath reject is likewise **not** a BIP-389 rule (BIP-389 permits `h`/`'` in multipath alts) — it is a **watch-only / xpub** constraint (BIP-32 forbids hardened public derivation; md1 cosigner keys are xpubs). Both rejects are correct *for md-cli's domain*; neither is a general BIP-389 prohibition.

**Implementation invariants this phase MUST honor (the review will verify):** (a) residue check placed strictly **AFTER the group-3 block closes at `:110`** (the `if let Some(m)=caps.get(3)` block opens `:77`, the per-alt `.split(';').map(…)` reject loop is `:90-107`, collect `:108`, block closes `:110`) — i.e. insert the residue check after the `?` on the collect / after the block-close brace at `:110`, so it can NEVER land mid-loop (Minor-1 fence); (b) group-3 capture (`([^>]*)` @ `:55`), validator loop (`:90-110`), and substitution strip class (`[0-9;]` @ `:498`) left BYTE-IDENTICAL; (c) `replace_all`/`captures_iter` unanchored semantics not changed to swallow the suffix.

### P3.5 — RED-first tests (M5 — most adversarial coverage)

- **RED→GREEN (the funds case):** `wpkh(@0/<2;3>/0'/*)` currently builds a `Descriptor` with `use_site_path.multipath=Some([2,3])` over a no-multipath tree (assert the divergence today) → after fix **REJECTED** with a typed `CliError::TemplateParse` naming the multipath-not-final cause.
- **H13 REGRESSION GUARD (stay GREEN unchanged):** re-assert all five — `lex_rejects_hardened_multipath_apostrophe` (`:197`), `lex_rejects_hardened_multipath_h_form` (`:211`), `lex_rejects_mixed_hardened_multipath` (`:221`), `lex_rejects_malformed_double_marker_multipath` (`:228`), `lex_accepts_nonhardened_multipath` (`:247`).
- **FUSED ordering test (proves H13 fires first):** `wsh(multi(2,@0/<0'';1>/0'/*,@1/<0'';1>/0'/*))` still errors with the **hardened/malformed** message (H13 before M5), **NOT** the suffix message — i.e. **H13's `<0'';1>` reject STAYS RED/rejected**.
- **POSITIVE CONTROL (stay GREEN):** normal multipath-last `wpkh(@0/<0;1>/*)` and `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))` still parse, multipath preserved, `parse_template` round-trips; `end_to_end_wsh_multi_template_only` (`:1939`) green.
- **`h`-in-origin sub-case:** `wpkh(@0/48h/0h/0h/<0;1>/*)` (unconsumed `48h…` residue) now rejected (today: malformed `XPUBh/…`).
- **Cross-validation belt (D4):** the per-`@i` lexer `occ.multipath_alts.len()` == substituted `DescriptorPublicKey` multipath-step count check — exercised by the positive-control multipath-last cases (asserts the counts agree and no extra fixed step), and a crafted divergence (if reachable) hits the `parse_template` refuse-to-emit path.

**Phase-3 gate:** `cargo fmt --all` → `cargo fmt --all --check` clean → `cargo clippy --all-targets -- -D warnings` clean → **FULL `cargo test -p md-cli` GREEN** (the five H13 guards + the fused ordering test + the positive controls explicitly confirmed GREEN). Per-phase architect review (M5 H13-preservation is the focus) → persist verbatim to `design/agent-reports/cycle9-phase-3-<round>-review.md` → fold-loop to 0C/0I → commit (explicit paths).

---

## Phase 4 — ship (cross-repo paired-PR) — spec §7, §8, D11

### P4.1 — Version bump + CHANGELOG (descriptor-mnemonic)

- **md-cli MINOR → `0.8.1` → `0.9.0`** (M10 widens accepted input — BIP-86 `tr(@0)` now accepted — the additive driver). Edit `crates/md-cli/Cargo.toml:3` `version = "0.9.0"`.
- **md-codec NO BUMP** — stays `0.38.0`. The path-pin `md-codec = { path="../md-codec", version="=0.38.0" }` (`Cargo.toml:28`) is UNCHANGED.
- **Root `CHANGELOG.md`:** add a crate-prefixed `## md-cli [0.9.0] — <date>` entry (the file uses `## md-cli [X.Y.Z]` / `## md-codec [X.Y.Z]` prefixes). SemVer note: MINOR (M10 additive driver); list the 7 fixes (M5 reject / M2 bound / M10 BIP-86 accept / M11 point check / L4+L19 advisory / L7 help prose). md-codec section UNCHANGED.
- **Version-site ritual (check at release):** grep any README self-pin / install snippet that names the md-cli version and bump in lockstep (per `project_toolkit_release_ritual_version_sites`). md-codec sites untouched.
- `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings` + FULL `cargo test -p md-cli` (+ `cargo test --workspace` sanity) all GREEN on the final tree.

### P4.2 — Completion tracking (canonical record = bughunt-report box ticks; NO pre-existing FOLLOWUPS to flip)

> **R0-round-1 I-1 reframe (verified against `origin/main:design/FOLLOWUPS.md` @ `836faf8`):** there are **ZERO pre-filed cycle-9 FOLLOWUP entries** in descriptor-mnemonic to flip. A grep for `multipath-not-last` / `u8-overflow-panic` / `bip86` / `secp256k1-point` / `mislabels-watch` / `epilog-stale` / `w3-mdcli` returns no cycle-9 entries (the single `bip86` hit at `FOLLOWUPS.md:614` is an unrelated renamed-test reference). The only RESOLVED entry is the L7 *underlying* slug `md-codec-decode-with-correction-supports-non-chunked-md1` (`FOLLOWUPS.md:123`), which is **left as-is** (already RESOLVED, no flip). So there is **no "flip 'open' → 'resolved'" of pre-existing entries** — that instruction is dropped. The plan's earlier invented descriptive slug names are reconciled away below in favor of the bughunt-report ids.

**(a) Canonical completion record = the TOOLKIT bughunt-report box ticks (§P4.5).** The authoritative tracking artifact for M5/M2/M10/M11/L4/L7/L19 is ticking the `- [ ]` → `- [x]` boxes in `mnemonic-toolkit/design/agent-reports/constellation-bughunt-2026-06-20.md` (done in §P4.5). The bughunt report is where these seven findings live and are tracked; their canonical ids are:
- M5 = `lexer-substitution-divergence-multipath-not-last` (bughunt `:276`)
- M2 = `placeholder-count-u8-overflow-panic` (bughunt `:225`)
- M10 = `w3-mdcli-01` (bughunt `:733`)
- M11 = `w3-mdcli-04` (bughunt `:748`)
- L4 = `repair-advisory-mislabels-watch-only-as-keyless-template` (bughunt `:332`)
- L19 = `w3-mdcli-03` (bughunt `:780`, sibling of L4)
- L7 = `repair-help-epilog-stale-rejects-nonchunked-claim` (bughunt `:365`)

**(b) OPTIONAL — file NEW RESOLVED entries in descriptor-mnemonic `FOLLOWUPS.md` if repo convention wants an in-repo trail.** This is NOT a required flip of pre-existing entries (none exist). If filed, each NEW entry is created with `Status: resolved <md-cli-v0.9.0 SHA>` in the shipping commit (file-and-resolve in one commit — the normal pattern for a finding first formalized at ship time), and **must cite the bughunt-report id above** so a future reader can cross-reference finding → bughunt-id → ship commit. (Memory rule `feedback_followup_status_discipline` — verify status at decision time, flip in the shipping commit — applies to any such NEW entry.)

**(c) L7 underlying slug — leave it.** `md-codec-decode-with-correction-supports-non-chunked-md1` is already `RESOLVED in md-codec-v0.35.0` (`FOLLOWUPS.md:123`); **no flip, no edit.**

**File NEW deferred-capability slug** `md1-post-multipath-fixed-path-derivation-steps` (M5 deferred; tier `v1+` — needs a wire+`UseSitePath` representation → would force a toolkit pin; until then M5 rejects fail-closed). This is a genuinely-new filing (not a flip).

### P4.3 — Tag + publish

- `git tag descriptor-mnemonic-md-cli-v0.9.0` (precedent `58cc9ec` "release: md-cli 0.8.0 (MINOR)" — md-cli-only release).
- `cargo publish -p md-cli` (md-codec NOT published — NO BUMP; the `=0.38.0` path-pin is already on crates.io).
- **NO toolkit pin** — the toolkit deps `md-codec` (library), never `md-cli` (standalone published binary). No toolkit `Cargo.toml`/`Cargo.lock` change is forced.

### P4.4 — CROSS-REPO docs-only paired edit: toolkit manual L7 mirror (DISCIPLINE, NOT lint-gated)

The toolkit manual `mnemonic-toolkit/docs/manual/src/40-cli-reference/42-md.md:367-379` (@ toolkit `origin/master` `d6398b57`) carries the SAME false prose as L7 — heading "### v0.6.0 limitation: chunked-form only" + "Non-chunked single-string md1 … is rejected with a wire-format error — use `md decode` …". **It MUST be corrected in lockstep with the `main.rs:241` epilog fix** (rewrite the subsection to state both chunked and non-chunked md1 are accepted; it points at the now-RESOLVED FOLLOWUP).

**This edit lands in the TOOLKIT repo** (off toolkit `origin/master`), done with the cycle-9 design-trail commit (this plan + the spec + the per-phase/whole-diff reviews live in the toolkit `design/` tree).

**It is DOCS-ONLY and NOT lint-enforced** (spec I-1 fold, R0-confirmed against live `lint.sh`): `docs/manual/tests/lint.sh`'s only `--help`-consuming step (4/6 flag-coverage) extracts flag **NAMES** (`grep -oE -- '--[a-z…]+'`) and asserts presence — **NO step diffs the `repair` epilog PROSE.** So the lint PASSES whether or not the prose is corrected; the `repair` flag set is unchanged. The edit needs **NO v0.9.0-`md`-binary build and NO `MD_BIN` pin** to pass lint. The lockstep obligation is the **CLAUDE.md manual-mirror paired-PR DISCIPLINE** (human/PR review of the corrected prose), a lagging/human gate — **not** a mechanical lint gate. If cross-repo single-session authoring is infeasible, file a paired sibling toolkit PR and note it in both repos' `FOLLOWUPS.md`; the manual edit stays a required cycle-9 deliverable either way.
**Belt check (M10):** sanity-check the manual's `md encode` accepted-heads wording is not contradicted by M10's now-accepted bare `tr(@0)`. The `tr(@0)` at `42-md.md:166` is a **`# tr(@0)` comment inside an `md compile 'pk(@0)' --context tap` worked EXAMPLE** (Minor-3) — i.e. a `compile` example, NOT the `md encode` accepted-heads prose and NOT part of the L7 epilog edit. The belt check is satisfied (no contradiction with M10's encode-side widening), but it does not by itself prove the encode accepted-heads prose covers bare `tr(@0)` — confirm no new encode prose is needed (expected: none).

### P4.5 — Tick the bughunt report

In `mnemonic-toolkit/design/agent-reports/constellation-bughunt-2026-06-20.md`, tick the seven `- [ ]` → `- [x]` boxes: **M5** (`:274`), **M2** (`:223`), **M10** (`:732`), **M11** (`:747`), **L4** (`:331`), **L7** (`:364`), **L19** (`:779`).

### P4.6 — Lockstep gates that do NOT fire (spec §7, D12)

- **GUI `schema_mirror`** (clap flag-NAME parity): **NOT triggered.** None of the 7 add/rename/remove a clap flag, subcommand, or dropdown value. M10 widens *accepted descriptor-string input* (not a flag); M5/M2/M11 are parse-internal rejects; L4/L19 advisory-text; L7 edits prose inside an existing `after_long_help` (flag names unchanged).
- **Sibling-codec FOLLOWUP companions:** none required — md-codec is untouched.

---

## Mandatory post-implementation gate — independent adversarial whole-diff review (NON-DEFERRABLE)

Per CLAUDE.md ultracode phase-4: after Phase 3 (all code landed), an **independent adversarial execution review over the WHOLE cycle-9 diff** (R0 reviewed plan correctness; this catches implementation-introduced regressions TDD misses). **Funds focus:**

1. **M5 rejects divergent templates AND H13's reject stays intact** — verify (a) `wpkh(@0/<2;3>/0'/*)` and the `h`-in-origin form are REJECTED (no divergent card emitted); (b) the group-3 capture (`:55`), validator loop (`:77-110`), and strip class (`:498`) are BYTE-IDENTICAL to `836faf8`; (c) the residue check is placed strictly AFTER the group-3 validator; (d) the fused `<0'';1>/0'/*` case errors with the **hardened/malformed** message, not the suffix message; (e) all five H13 guard tests + positive multipath-last controls GREEN; (f) the D4 cross-check is the constructible substituted-`DescriptorPublicKey`-count comparison, not a vacuous assertion.
2. **M10 no over-accept** — `tr(@0,{...})` / `tr(NUMS,multi_a(...))` still classify MultiSig(depth-4); the depth gate (`keys.rs:67-77`) is NOT relaxed; the `synthetic_xpub_for` depth flip is address-neutral.
3. **M11 no valid-xpub break** — real depth-3/4 xpubs still parse; only off-curve pubkeys rejected.
4. **M2** — `@254` accepts, `@255` rejects cleanly (no panic, no silent `n=0`).
5. **L4/L19** — keyed→`WatchOnly` / keyless→`Template` at all three sites; `byte_parity_advisory_lines` GREEN.
6. **L7** + manual mirror — false prose gone from both `main.rs:241` and `42-md.md:367-379`.
7. **Full suite** `cargo test -p md-cli` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all --check` GREEN; no schema_mirror / sibling-companion drift.

Persist the review verbatim to `design/agent-reports/cycle9-whole-diff-review-<round>.md`; fold-loop to 0C/0I BEFORE tag/publish. If Agent-API dispatch fails mid-session, flag it explicitly and defer the formal review to API recovery — never silently substitute inline self-review.

---

## Resolved decisions (reference — DO NOT re-decide; spec §11 D1-D13)

D1 M5 REJECT (fail-closed, typed `TemplateParse`) · D2 BIP-389 *permits* post-multipath steps → reject is an md1/`UseSitePath` representability limit, NOT a BIP-389 prohibition · D3 M5 preserves H13 via disjoint regions + group-3-validator-FIRST ordering · D4 cross-validation YES (substituted-`DescriptorPublicKey` multipath count, NOT `tree`) · D5 M2 BOTH (bound `max<=254` + checked `get`) · D6 M10 fix the CLASSIFIER ("`tr(` + one `@i` + no `,` + no `{`"→SingleSig), keep depth gate strict · D7 M11 point check at `keys.rs:78`, no new dep · D8 L4/L19 branch on `is_wallet_policy()`, three sites one commit · D9 L7 rewrite `main.rs:241`, no FOLLOWUP flip · D10 L7 manual lockstep = paired-PR DISCIPLINE, NOT lint-enforced, no `MD_BIN` pin · D11 md-cli MINOR 0.9.0 / md-codec NO BUMP / no toolkit pin / tag + publish · D12 GUI schema_mirror NOT triggered · D13 order M11→M2→M10→M5 then L4+L19+L7 (here grouped P1[M11/M10/L4/L19/L7] → P2[M2] → P3[M5] → P4[ship], M5 LAST).

---

## Plan-doc R0 gate (this doc's own mandatory loop)

This is the IMPLEMENTATION PLAN — **DESIGN ONLY, no code.** Per CLAUDE.md Conventions bullet 1: before ANY implementation (no implementer dispatch, no edits, no tag, no publish), this plan-doc MUST pass the opus-architect **R0 reviewer-loop to 0 Critical / 0 Important** — fold each review → persist verbatim to `design/agent-reports/cycle9-plan-r0-round{N}-review.md` → re-dispatch → repeat until GREEN (the reviewer-loop continues after every fold). The full `cargo test -p md-cli` suite (not targeted targets) gates each per-phase R0 during execution. **The M5 H13-preservation argument (Phase 3 / §P3.4) is the review's primary focus.** No Critical/Important may remain open at any gate (start coding, advance phase, tag, ship).
