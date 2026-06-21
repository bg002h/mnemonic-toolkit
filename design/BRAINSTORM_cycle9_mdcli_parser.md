# BRAINSTORM — cycle-9 — md-cli lexer/parser robustness cluster

**Findings:** M5 (funds), M2, M10, M11 + advisory L4 / L19 / L7.
**Repo:** `descriptor-mnemonic` (the m-format *descriptor* codec + `md` CLI). All fixes are **md-cli-side**; md-codec is UNTOUCHED.
**Status:** DESIGN ONLY — no code. Feeds the mandatory opus-architect **R0 loop to 0 Critical / 0 Important** before ANY implementation.

---

## 0. Source-SHA table — cite these BYTES, not the bug-report snapshots

> **STALE-LOCAL CAVEAT.** The descriptor-mnemonic local `main` is **7 commits behind** `origin/main` and MUST NOT be trusted for line numbers. Every citation below was re-grepped against the **bytes of `origin/main` = `836faf8`** via `git show origin/main:<path>`. The bug-hunt report's citations are pre-cycle-1/cycle-2 snapshots (`template.rs` grew from ~400 to **1972** lines) — they are documented here only as "report said" for traceability.

| Repo | `origin/main` (or `origin/master`) SHA | Role this cycle |
|---|---|---|
| **descriptor-mnemonic** | **`836faf87c3d82b119a9f0f5c6589a7db1f8613a4`** (`836faf8`) | md-codec 0.38.0 / **md-cli 0.8.1** — the cycle-9 base; impl branches off here |
| mnemonic-toolkit | `8d2fe50` (master, recon-time) | manual-mirror lockstep target for **L7** (see §8) |

| Finding | File | LIVE line(s) @ `836faf8` | Report-snapshot (stale) |
|---|---|---|---|
| **M5** lexer regex | `crates/md-cli/src/parse/template.rs` | **`:55`** | `:32-91` / `:40` |
| **M5** substitution regex | `crates/md-cli/src/parse/template.rs` | **`:498`** | `:357-381` / `:365` |
| **M5** stitch site (`parse_template`) | `crates/md-cli/src/parse/template.rs` | **`:1874-1922`** (`Ok(Descriptor{…})` @ `:1915-1921`; field order `n`/`path_decl`/`use_site_path`@`:1918` (from lexer) / `tree`@`:1919` (from substituted) / `tlv`) | — |
| **M2** n-cast (`+1) as u8`) | `crates/md-cli/src/parse/template.rs` | **`:307-312`** | `:188-201` |
| **M2** density loop / panic-index | `crates/md-cli/src/parse/template.rs` | **`:313` (loop)**, **`:320` (`by_i[&0]`)**, **`:324` (`by_i[&i]`)** | `:188-201` |
| **M10** classifier (`ctx_for_template`) | `crates/md-cli/src/parse/template.rs` | **`:1925-1932`** (`tr(` falls to `else` → MultiSig) | `:1792-1799` |
| **M10** depth gate | `crates/md-cli/src/parse/keys.rs` | **`:67-77`** (expected 3 SingleSig / 4 MultiSig) | `:67-77` (accurate) |
| **M11** payload copy (no point check) | `crates/md-cli/src/parse/keys.rs` | **`:78-79`** (`copy_from_slice(&bytes[13..78])`) | `:78-79` (accurate) |
| **L4** repair advisory | `crates/md-cli/src/cmd/repair.rs` | **`:156-159`** (unconditional `OutputClass::Template`) | `:156-159` (accurate) |
| **L19** encode advisory (JSON path) | `crates/md-cli/src/cmd/encode.rs` | **`:73-76`** | `:73-76` (accurate) |
| **L19** encode advisory (text path) | `crates/md-cli/src/cmd/encode.rs` | **`:110-113`** | `:110-113` (accurate) |
| **L7** stale help epilog | `crates/md-cli/src/main.rs` | **`:241`** (`after_long_help` for `Repair`) | `:241` (accurate) |
| **L7** toolkit-manual MIRROR | `mnemonic-toolkit/docs/manual/src/40-cli-reference/42-md.md` | **`:367-379`** (the false "rejected with a wire-format error" prose) | — (cross-repo) |
| helper `is_wallet_policy()` | `crates/md-codec/src/encode.rs` | **`:50-52`** (`matches!(&self.tlv.pubkeys, Some(v) if !v.is_empty())`) | `:50-52` (accurate) |

### The H13-collision constraint — READ BEFORE TOUCHING M5/M2

**M5 and M2 live in the exact functions cycle-1's H13 (`081f61c` + C1 `ddddeff`) rewrote** — `lex_placeholders` (regex `:55`) and `substitute_synthetic` (regex `:498`), plus the `resolve_placeholders` n-computation (`:307-324`). **M10, M11, L4, L19, L7 are in code H13 never touched → no collision there.**

H13's current mechanism (verified @ `836faf8`, `template.rs:55,77-110,197-247`):

- The lexer regex group-3 multipath body is captured **PERMISSIVELY** as `([^>]*)` — *on purpose*, so the strict in-loop validator (`:90-110`) **sees** every present `<…>` body and can REJECT it.
- The in-loop validator rejects (typed `CliError::TemplateParse`): **hardened** alts (`'`/`h`), **malformed double-marker** bodies (`<0'';1>`, `<0'h;1>`, `<0h';1>`), and any non-`[0-9;]` residue; it ACCEPTS bare integer alts `<0;1>` / `<0;1;2>`.
- The substitution regex strip class is `[0-9;]` (NOT the widened `[0-9;'h]` — the C1 revert; widening was the original silencer of the malformed-double-marker collapse).
- H13's guard tests (must stay GREEN — verified present @ `836faf8`): `lex_rejects_hardened_multipath_apostrophe` (`:197`), `lex_rejects_hardened_multipath_h_form` (`:211`), `lex_rejects_mixed_hardened_multipath` (`:221`), `lex_rejects_malformed_double_marker_multipath` (`:228`), and the CLEAN-NEGATIVE `lex_accepts_nonhardened_multipath` (`:247`).

> **HARD CONSTRAINT (this cycle's top risk):** the M5 fix MUST NOT regress any of those five tests, MUST NOT let a hardened/malformed multipath body through, and MUST NOT break the accept of valid `<0;1>`. The M5-preservation argument in §3.1 is mandatory and is the focus of the post-impl whole-diff review.

---

## 1. Finding summary — ALL REPRODUCE @ `836faf8`

| # | Class | One-line | Repro? |
|---|---|---|---|
| **M5** | B — policy-collapse / wrong-derivation (FUNDS) | post-multipath suffix (`@0/<2;3>/0'/*`) → lexer truncates at `>`, substitution keeps `/0'/*` literal → `Descriptor` carries `multipath=Some([2,3])` over a tree with none → divergent derivation, exit 0 | **YES** |
| **M2** | E — panic/DoS | `@255` → `(255+1) as u8 == 0` → density loop `0..0` skipped → `by_i[&0]` bracket-index panic (`@0` absent) or silent `n=0` (`@0` present) | **YES** |
| **M10** | availability — false-reject | `tr(@0/<0;1>/*)` BIP-86 single-key → classifier routes `tr(` to MultiSig(depth-4) → depth-3 BIP-86 account xpub rejected | **YES** |
| **M11** | C — corrupt-input accept | `parse_key` copies `bytes[45..78]` with NO `PublicKey::from_slice` point check → off-curve xpub admitted, deferred to derive | **YES** |
| **L4** | D — privacy/doc | `md repair` unconditionally labels output "keyless template" even for watch-only (keyed) md1 | **YES** |
| **L19** | D — privacy/doc | `md encode` (both JSON + text paths) same mislabel as L4 with `--key` | **YES** |
| **L7** | doc drift | `md repair --help` epilog falsely says non-chunked md1 "rejected with a wire-format error" (md-codec v0.35.0 now repairs them) | **YES** |

---

## 2. Per-finding fix design

### 3.1 — M5 (THE FUNDS CRUX): post-multipath suffix → divergent card

#### 3.1.1 — The exact mechanism (traced @ `836faf8`)

`parse_template` (`:1874-1922`) builds the final `Descriptor` from **two independently-computed views** with **no cross-validation**:

```
let occs     = lex_placeholders(template)?;              // :1880  → lexer view
let resolved = resolve_placeholders(&occs)?;             // :1881  → resolved.use_site_path (multipath here)
let (substituted, key_map) = substitute_synthetic(...)?; // :1883  → substituted string
let ms_desc  = MsDescriptor::from_str(&substituted)?;    // :1884  → structural view
let tree     = walk_root(&ms_desc, &key_map)?;           // :1886
...
Ok(Descriptor {                                          // :1915
    n: resolved.n,                                       // :1916
    path_decl: resolved.path_decl,                       // :1917
    use_site_path: resolved.use_site_path,               // :1918  ← from LEXER
    tree,                                                 // :1919  ← from SUBSTITUTED
    tlv,                                                  // :1920
})
```

For input `wpkh(@0/<2;3>/0'/*)`:

- **Lexer (`:55`)** `@(\d+)((?:/\d+'?)*)(?:/<([^>]*)>)?(/\*(?:'|h)?)?`:
  - g1 = `0`; g2 = `` (empty — `/<` is not `/\d`); g3 = `2;3`; **g4 wildcard tries to match at `/0'/*` → fails (`/\*` needs `/`+`*`, sees `/0`) → matches EMPTY**.
  - The overall match **ends at `>`**. The trailing `/0'/*` is **never consumed**. Lexer records `multipath_alts=[2,3]`, `origin_path=None`, `wildcard_hardened=false`. → `resolved.use_site_path.multipath = Some([2,3])`.
- **Substitution (`:498`)** `@(\d+)((?:/\d+'?)*)(?:/<[0-9;]+>)?(?:/\*(?:'|h)?)?`: matches the same `@0/<2;3>` span, replaces it with a synthetic xpub, **leaves the literal `/0'/*`** → `wpkh(XPUB/0'/*)`.
- `MsDescriptor::from_str("wpkh(XPUB/0'/*)")` parses a **single-path key, origin `/0'`, NO multipath** → `tree` has no multipath.
- **Result:** `Descriptor { use_site_path.multipath = Some([2,3]), tree = (no multipath, dropped /0') }`. The emitted md1's recorded use-site path (chains 2 & 3) disagrees with its structural tree → **wrong derivation / wrong address, silently, exit 0.** Confirmed: grep of `template.rs` finds NO `mismatch`/`cross`/`leftover`/`residual` cross-check, and the only positional lexer test (`origin_path_extracted` `:170`) puts the multipath **last** — the not-last case is **untested**.

The `h`-in-origin sub-case (`@0/48h/…`) is the same family: origin class is `/\d+'?` (apostrophe only, no `h`), so an `h` step is left unconsumed → malformed `XPUBh/…`. The §3.1.3 fix covers it (any unconsumed `/…` after the placeholder span → reject).

#### 3.1.2 — DECISION: REJECT (fail-closed), NOT canonicalize. **RECOMMENDED.**

**Decision: REJECT the multipath-not-last / unconsumed-suffix form with a typed `CliError::TemplateParse`.**

**BIP-389 justification (verified against the spec text, not plausibility):** BIP-389 *does* permit path steps after the multipath tuple — the grammar is "the `/<NUM;NUM...>` tuple ... **Followed by zero or more /NUM... path elements**." So `xpub/<0;1>/0/*` is a *legal BIP-389 descriptor*. **The bug is therefore NOT "BIP-389 forbids it."** The bug — and the reason REJECT is the right call — is a **representability limitation of md1 + the md-cli use-site model**: md-cli's `UseSitePath` and md1's wire format model the multipath as the **single final pre-wildcard derivation step** (`make_use_site_path` reads exactly the lexer's `multipath_alts` + `wildcard_hardened`; there is no field for post-multipath fixed steps). A descriptor with fixed derivation steps *after* the multipath **cannot be faithfully represented**. The current code does not *refuse* it — it **silently produces a divergent card** (the worst funds outcome).

Given that, the choice is:

- **(a) canonicalize** — extend BOTH regexes with a post-multipath capture group so lexer == substitution and the suffix is faithfully carried. **REJECTED** because: (1) it requires a *new* representation (md1 wire + `UseSitePath` would need a post-multipath fixed-path field) — that is a wire/format change, far outside a md-cli-only robustness cycle, and would force an md-codec change (breaking the "md-codec UNTOUCHED / no toolkit pin" property of this whole cycle); (2) the multipath-not-last form is **non-canonical and rare** — BIP-389's canonical examples put `<…>` as the final step before `/*`, and every md-cli/toolkit emit path produces multipath-last; (3) silently *accepting* an exotic shape is exactly the failure mode H13 and cycle-1 fought (never silently encode an un-restorable / divergent card).
- **(b) REJECT (fail-closed)** — detect any unconsumed path residue after the placeholder span and error with a typed message. **CHOSEN.** Simpler, funds-safe, matches the cycle-1 H13 precedent (reject the un-representable rather than silently encode it), and keeps md-codec / wire / toolkit-pin all untouched. A user who genuinely needs post-multipath fixed steps gets a *loud, actionable* error rather than a wrong card — and that capability can be a future FOLLOWUP (`md1-post-multipath-fixed-path-derivation-steps`, §9) if real demand appears.

#### 3.1.3 — The exact change

The defect is **unconsumed input after a placeholder match**. Two complementary edits (do BOTH — the residue check is the real fix, the cross-validation is belt-and-suspenders):

1. **Residue reject in `lex_placeholders` (`template.rs`, around `:55`/`:84`).** After the regex matches each placeholder, verify nothing legal-looking was left dangling. Two viable forms (architect to pick at plan time; both fail-closed):
   - **(i) Anchor the wildcard/terminator.** Tighten the trailing portion so that after an optional multipath the regex requires either the wildcard `(/\*(?:'|h)?)?` **immediately** OR a placeholder-terminator (`,`, `)`, end). Then, in the capture loop, if `caps.get(0).end()` is followed by a `/` + path-like char (i.e. an unconsumed `/NUM`/`/NUMh`/`/NUM'` segment that is NOT part of a new `@i`), return `CliError::TemplateParse("@{i}: derivation steps after the multipath group are not representable in md1; the multipath `<…>` must be the final derivation step before the wildcard")`. This also catches the `h`-in-origin sub-case (the unconsumed `/48h` residue).
   - **(ii) Whole-span validation.** Keep the permissive group-3 capture (H13 needs it), but ADD a check: reconstruct the matched placeholder text and assert the original template's placeholder span contains no trailing `/`-segment between the matched end and the next placeholder/closing token; reject otherwise.

   > **Preferred:** (i) — a tightened anchor + an explicit residue check in the loop. It composes with H13: H13's permissive `([^>]*)` group-3 + strict validator is **untouched**; the new check fires only on path chars *outside* `<…>` (the suffix), so a hardened/malformed body still hits H13's validator FIRST and still rejects with the "hardened" message. State this explicitly in the plan.

2. **Cross-validation in `parse_template` (`:1874-1922`), belt-and-suspenders.** Before constructing the final `Descriptor`, assert the **lexer view agrees with the substituted structural view**. The `tree` built by `walk_root` does NOT carry a per-key "multipath alts" field to diff against the lexer view directly, so the comparison must be made against the **substituted `DescriptorPublicKey`(s)** that `MsDescriptor::from_str` parsed (which DO carry derivation paths / multipath), NOT against `tree`'s tag/key_index/leaf shape. **Concrete, constructible check:** for each `@i`, compare the lexer's `occ.multipath_alts.len()` (0 when no `<…>`, else the alt-count) to the multipath-step count of the substituted key's `DescriptorPublicKey` (the count of `/<…>` multipath tuples on that key's derivation path); they must agree, and no substituted key may carry a fixed derivation step the lexer did not record. On mismatch → `CliError::TemplateParse("internal: lexer/substitution divergence for @{i} — refusing to emit a divergent card")`. This makes ANY future regex drift fail-closed rather than silently emit a divergent card. (Cheap; defensive; funds-priority. The residue reject in edit-1 is the real fix; this is the checkable belt against drift, not a vacuous always-true assertion.)

#### 3.1.4 — H13-PRESERVATION ARGUMENT (mandatory)

The M5 change preserves H13 because **it operates on a disjoint region of the input**:

- H13 governs the **content of the `<…>` multipath body** (group-3): permissive capture + strict reject of hardened/malformed alts. M5 governs **path text OUTSIDE `<…>`** — specifically a residual `/NUM…` segment *after* the closing `>` (or an `h`-bearing origin step *before* `<`). These are different spans; M5 adds a check on the suffix, it does **not** relax, widen, or touch the group-3 capture or its validator loop (`:90-110`) or the substitution strip class `[0-9;]` (`:498`).
- **Ordering guarantee:** for a hardened/malformed body like `@0/<0'';1>/0'/*`, H13's group-3 validator (`:90-110`) runs during the same capture iteration and returns the typed "hardened"/"not a bare unsigned integer" error **before** any M5 residue logic would matter — so H13's five guard tests (`:197`, `:211`, `:221`, `:228`, plus accept `:247`) stay GREEN unchanged. The plan MUST run those five tests (plus the full `cargo test -p md-cli` suite per the R0-full-suite memory rule) as a regression gate and assert all pass post-M5.
- **No over-rejection of valid multipath-last:** the canonical `@0/<0;1>/*` has NO post-`>` residue (the `/*` is the wildcard, consumed by group-4), so the M5 residue check does not fire → `lex_accepts_nonhardened_multipath` (`:247`) and `end_to_end_wsh_multi_template_only` (`:1939`) stay GREEN.

**Net:** M5 narrows the accepted grammar (rejects the previously-silently-mis-encoded suffix form) while H13's hardened/malformed reject and the valid-multipath accept are byte-for-byte preserved.

**Protocol-fact correction (for the spec record):** the bug-report and recon framed this as "BIP-389 multipath must be last." That is **imprecise** — BIP-389 *allows* post-multipath path steps. The accurate framing (used above) is: *md1 + md-cli's use-site model cannot represent post-multipath fixed steps, so md-cli rejects the form rather than silently encode a divergent card.* H13's hardened-multipath reject is likewise **not** a BIP-389 rule (BIP-389 permits `h`/`'` in multipath alts) — it is a **watch-only / xpub** constraint (BIP-32 forbids hardened public derivation; md1 cosigner keys are xpubs). Both rejects are correct *for md-cli's domain*; neither is a general BIP-389 prohibition. Do not over-claim in any user-facing error text.

---

### 3.2 — M2: `@255` placeholder-count u8 overflow → panic

**Mechanism @ `836faf8` (`template.rs:307-324`):** `n = (by_i.keys().max().copied()...? as usize + 1) as u8;` (`:307-312`). For max index 255: `(255usize + 1) as u8 == 0`. The density loop `for i in 0..n` (`:313`) becomes `0..0` → skipped. Then `let at0 = by_i[&0];` (`:320`, bracket index) panics `"no entry found for key"` when `@0` is absent (`wpkh(@255/*)`); when `@0` IS present, `n=0` flows silently into the encoder (key count collapses to 0). The lexer accepts `@0..=255` (`caps[1].parse::<u8>()` @ `:60`; `parse_index` @ `keys.rs:83-88` also u8) so `@255` is valid-per-lexer.

**Fix (bound before the cast; defense-in-depth on the index):**
1. **Primary — bound the max index before `as u8` (`:307-312`).** Compute `max` as the raw value; if `max > 254` (i.e. `max + 1` would not fit a `u8` placeholder count, or — equivalently — reject `max >= 255`), return `CliError::TemplateParse("placeholder index @{max} out of range: at most @254 (a template may declare at most 255 keys, @0..=@254)")`. After this guard, `(max + 1) as u8` cannot wrap.
2. **Defense-in-depth — checked gets.** Replace the bracket indexes `by_i[&0]` (`:320`) and `by_i[&i]` (`:324`) (and `:386`/`:391` if reachable) with `by_i.get(&…).ok_or_else(|| CliError::TemplateParse(...))?`. Belt-and-suspenders against any future path that skips the density loop.

**SemVer:** PATCH-class on its own — no input is *newly accepted*; a previously-**panicking** input now errors cleanly. (Rides the cycle's MINOR umbrella, see §7.)

**H13/M5 interaction:** M2's guard sits between the dense-check (`:313`) and the n-cast it protects (`:307-312`); it is in `resolve_placeholders`, NOT in the two regexes M5 edits. No collision with M5 beyond same-file proximity — but the plan should sequence M2 before M5 so the M5 work lands on the bounded `n`.

---

### 3.3 — M10: BIP-86 single-key `tr(@0)` falsely rejected

**Mechanism @ `836faf8`:** `ctx_for_template` (`template.rs:1925-1932`) maps only `wpkh(` / `pkh(` / `sh(wpkh(` → `SingleSig`(depth-3); **everything else, including key-path `tr(@0/<0;1>/*)`, falls to the `else` → `MultiSig`(depth-4).** `parse_key`'s depth gate (`keys.rs:67-77`) then rejects a depth-3 BIP-86 account xpub with `"expected depth 4 ... got 3"`. **BIP-86 fact (verified against the spec):** a single-key P2TR account xpub is at `m/86'/0'/0'` = **depth 3** (purpose'/coin_type'/account'). The depth byte is advisory (discarded in derivation per the recon) → relaxing the classifier cannot corrupt an address → **pure availability**, not wrong-address.

**Fix (classify bare single-key `tr(` as SingleSig depth-3; `template.rs:1925-1932`):** extend the SingleSig branch to also match a **bare single-key key-path taproot** — `tr(` whose body contains **exactly one `@i` and NO script tree** (no internal `,` and no `{`). Concretely: `head.starts_with("tr(")` AND the `tr(...)` argument list has no top-level `,` (a comma separates the internal key from a script tree, or separates `multi_a` cosigners) AND no `{`. Such a form is `tr(@i[/path])` = BIP-86 single-key P2TR → SingleSig(depth-3).

**Over-acceptance guard (MUST confirm — the recon's noted hazard):** a `tr(` WITH a script tree (`tr(NUMS,{...})`, `tr(@0,{...})`, `tr(NUMS,multi_a(2,@0,@1,@2))`) still routes to MultiSig(depth-4) — its keys are multisig-context (depth-4) keys. The classifier scope is **strictly** "`tr(` + exactly one `@i` + no `,` + no `{`". Test BOTH directions: `tr(@0/<0;1>/*)` now ACCEPTS a depth-3 xpub (positive); a `tr(@0,{...})` / `tr(NUMS,multi_a(...))` still routes MultiSig so a genuinely depth-4 key is still required and a wrong-depth key in those forms still rejects (negative control). Also keep the existing `tr(...)` tests green (`tr_with_and_v_verify_older_inheritance` `:1240`, `tr_tap_leaf_bare_pk_on_wire` `:1804`, etc.).

> **Alternative considered + rejected:** relax the depth gate at `keys.rs:67-77` to accept "3 OR 4" for taproot. Rejected — it would *also* loosen the depth-4 script-path-tr keys (over-acceptance), losing a real intake check. Fixing the **classifier** is surgical; the gate stays strict.

**SECOND `ctx_for_template` consumer (note for the implementer — harmless/unaffected):** the `ctx` returned by `ctx_for_template` feeds TWO call-sites, not one — (1) `parse_key`'s depth gate (`encode.rs:34` `let ctx = ctx_for_template(...)` → `:38` `parse_key(k, ctx, ...)`), the depth check this fix targets; AND (2) `substitute_synthetic → synthetic_xpub_for(i, ctx)` (`template.rs:458` `fn synthetic_xpub_for(i: u8, ctx: ScriptCtx)`, called at `:506`), which builds the throwaway synthetic xpub at depth 3 (SingleSig) / depth 4 (MultiSig). Flipping bare `tr(@0)` to SingleSig therefore ALSO makes its synthetic xpub depth-3. **This is harmless and address-neutral:** the synthetic is discarded after `key_map` substitution, the depth byte is advisory (discarded in derivation per the recon), and the existing `tr_key_only` test pins `ScriptCtx::MultiSig` *explicitly* (not via `ctx_for_template`) so it does not break. The implementer must not miss this second consumer when editing the classifier, but no extra fix is needed there.

**SemVer:** this WIDENS accepted input (`tr(@0)` BIP-86 now accepted where it was rejected) → additive behavior → **the MINOR driver for the whole md-cli release** (see §7).

---

### 3.4 — M11: off-curve xpub admitted at parse (no secp256k1 point check)

**Mechanism @ `836faf8` (`keys.rs:78-79`):** `parse_key` validates base58check / 78-byte length / version / depth, then `let mut payload = [0u8; 65]; payload.copy_from_slice(&bytes[13..78]);` — **no check that the compressed pubkey is a valid secp256k1 point.** Layout: `bytes[13..45]` = chaincode (32B), `bytes[45..78]` = compressed pubkey (33B). An off-curve / all-zero `bytes[45..78]` passes intake, is copied into the Pubkeys TLV, and only fails later at `derive_address`. (Grep confirms the file imports only `bitcoin::base58` — no point check anywhere.)

**Fix (insert point check before the copy; `keys.rs:78`):**
```
bitcoin::secp256k1::PublicKey::from_slice(&bytes[45..78])
    .map_err(|e| CliError::BadXpub { i, why: format!("xpub public key is not a valid secp256k1 point: {e}") })?;
```
`bitcoin::secp256k1` is **already a reachable dep** — `template.rs:461` already `use bitcoin::secp256k1::{Secp256k1, SecretKey}`, so **no new Cargo dependency**. **secp256k1 fact (verified):** a valid BIP-32 serialized xpub's `bytes[45..78]` is a compressed point on secp256k1 (BIP-32 serialization) — so this rejects only genuinely-corrupt keys.

**Classification:** TRIVIAL→**FORMAL lane** — it adds a REJECT of previously-accepted input (a behavior change), so it goes through the formal TDD lane (failing test first). PATCH-class on its own (rejecting nonsense is not a feature add); rides the MINOR umbrella from M10.

---

### 3.5 — L4 + L19: watch-only md1 mislabeled "keyless template (no keys)"

**Mechanism @ `836faf8`:** three emit sites unconditionally pass `OutputClass::Template`:
- `repair.rs:156-159` (`md repair`).
- `encode.rs:73-76` (`md encode --json` path) and `encode.rs:110-113` (`md encode` text path).

`OutputClass::Template`'s advisory text is `"note: stdout is a keyless descriptor template (no keys)"` (`crates/md-cli/src/output_advisory.rs` — crate root `src/`, NOT `src/cmd/`). But all three run on **arbitrary** md1 — including **wallet-policy** md1 whose Pubkeys TLV carries xpub/pubkey entries (watch-only material; the natural `md encode --key …` / `md repair` input). The model to mirror: `md address` emits `WatchOnly` and is sound because it is guarded to wallet-policy inputs only; `repair`/`encode` accept BOTH classes and so must **branch**.

**Fix (ONE branch, THREE sites — fold into one commit):** at each of the three sites, choose the class from `descriptor.is_wallet_policy()`:
```
let class = if descriptor.is_wallet_policy() { OutputClass::WatchOnly } else { OutputClass::Template };
emit_output_class_advisory(class, &mut stderr());
```
`is_wallet_policy()` is `md_codec::Descriptor::is_wallet_policy` (`md-codec/src/encode.rs:50-52`, `matches!(&self.tlv.pubkeys, Some(v) if !v.is_empty())`) — already public, already in scope (these commands hold a decoded `Descriptor`). md-codec UNTOUCHED.

**SemVer:** PATCH (advisory-text correctness; no flag/behavior change).

---

### 3.6 — L7: stale `md repair --help` epilog (+ toolkit-manual mirror)

**Mechanism @ `836faf8` (`main.rs:241`):** the `Repair` `after_long_help` string contains *"Non-chunked single-string md1 ... are rejected with a wire-format error — use `md decode` for read-only inspection of those."* This is **false** since md-codec **v0.35.0** added single-string auto-dispatch (`chunk.rs:604-632`: `if strings.len() == 1 { ... chunked_flag == 0 → decode_md1_string(...) }`) — non-chunked md1 are now repaired. The FOLLOWUP `md-codec-decode-with-correction-supports-non-chunked-md1` is already **`Status: RESOLVED in md-codec-v0.35.0`** (verified in `descriptor-mnemonic/design/FOLLOWUPS.md:123`); only the help text + manual drifted.

**Fix:** delete / rewrite the false sentence in the `:241` epilog (state that both chunked and non-chunked md1 are accepted; keep the ATOMIC-SEMANTICS multi-chunk note, which is still accurate). **No FOLLOWUP status flip needed** (the slug is already RESOLVED).

**LOCKSTEP — toolkit manual mirror (verified, cross-repo; paired-PR DISCIPLINE, NOT lint-gated):** the toolkit manual `docs/manual/src/40-cli-reference/42-md.md:367-379` mirrors this exact prose under "### v0.6.0 limitation: chunked-form only" — *"Non-chunked single-string md1 ... is rejected with a wire-format error — use `md decode` ..."*. **This is the same false claim and MUST be corrected in lockstep** (delete/rewrite that subsection; it points at the now-RESOLVED FOLLOWUP). This is required by the **CLAUDE.md manual-mirror paired-PR discipline** (any md-cli help-surface prose change mirrors into the manual), **NOT** by `docs/manual/tests/lint.sh` — that lint's only `--help`-consuming step (flag-coverage) extracts flag **NAMES** (`grep -oE -- '--[a-z…]+'`) and checks presence; NO step diffs the `repair` epilog PROSE, so the lint passes whether or not the false prose is corrected, and the `repair` flag set is unchanged either way. It must therefore be done **by discipline** (human/PR review of the corrected prose), with **no v0.9.0-`md`-binary build and no `MD_BIN` pin** needed for lint to pass. See §8 for the paired sequencing.

---

## 7. SemVer / publish / lockstep

| Item | Call |
|---|---|
| **Per-finding** | M5 PATCH-class-reject · M2 PATCH · M11 PATCH (FORMAL-lane reject) · L4/L19/L7 PATCH · **M10 MINOR (widens accepted input)** |
| **md-cli aggregate** | **MINOR → `0.8.1` → `0.9.0`** (M10 is the additive driver) |
| **md-codec** | **NO BUMP** — stays `0.38.0`; all 7 fixes are md-cli-side. md-cli path-deps `md-codec = { path = "../md-codec", version = "=0.38.0" }` (already on crates.io). |
| **toolkit pin** | **NONE.** The toolkit depends only on `md-codec` (library), **never on `md-cli`**. md-cli is a standalone published binary. No toolkit `Cargo.toml`/`Cargo.lock` change is forced by this cycle. (The toolkit-manual L7 edit in §8 is a docs-only paired commit, NOT a pin bump.) |
| **Tag + publish** | `git tag descriptor-mnemonic-md-cli-v0.9.0` → `cargo publish -p md-cli`. Precedent: `58cc9ec` "release: md-cli 0.8.0 (MINOR)" (md-cli-only release). |
| **README/version sites** | bump md-cli version in `crates/md-cli/Cargo.toml` (+ any README self-pin / install snippet that names the md-cli version — check at release per the version-site ritual). md-codec sites untouched. |

### Lockstep gates — what fires, what doesn't

- **GUI `schema_mirror`** (clap flag-NAME parity): **NOT triggered.** None of the 7 add/rename/remove a clap flag, subcommand, or dropdown value. M10 widens *accepted descriptor-string input* — it is NOT a new flag. M5/M2/M11 are parse-internal rejects. L4/L19 are advisory-text. L7 deletes help **prose inside an existing `after_long_help`** — clap flag NAMES are unchanged → no schema-mirror delta.
- **Manual mirror** (`docs/manual/src/40-cli-reference/`): **TRIGGERED by L7 only — by paired-PR DISCIPLINE, NOT by the lint.** The flag-coverage lint (`docs/manual/tests/lint.sh`) extracts flag **NAMES** only from `--help` and checks presence; it does NOT diff the `repair` epilog **prose**, so it passes regardless of the L7 prose edit (the `repair` flag set is unchanged). The lockstep obligation is the CLAUDE.md manual-mirror **discipline**: L7 changes `md repair`'s `--help` epilog prose, and `42-md.md:367-379` carries the mirrored false prose → must be corrected in lockstep (§8), enforced by human/PR review, NOT a mechanical gate. M10 does NOT change `md encode --help` text (the classifier is internal; `md encode`'s help/examples already list `tr(@0)` at `42-md.md:166` — no new prose needed, but the plan should sanity-check the manual's `md encode` accepted-heads wording is not contradicted). M2/M5/M11/L4/L19 change no help text → no manual delta.
- **Sibling-codec FOLLOWUP companions:** **none required** — md-codec is untouched, so no `descriptor-mnemonic ↔ toolkit/ms/mk` companion `FOLLOWUPS.md` mirror is needed for the *code*. (L7's FOLLOWUP is already RESOLVED across companions; this cycle only corrects the stale prose those entries point at.)

---

## 8. Cross-repo sequencing for L7 (md-cli help + toolkit manual)

Because the help-text source (`md-cli/src/main.rs:241`) and its manual mirror (`mnemonic-toolkit/docs/manual/.../42-md.md`) live in **two repos**, the L7 edit is a **paired change — by CLAUDE.md manual-mirror DISCIPLINE, NOT a lint-enforced gate**:

1. **descriptor-mnemonic** (this cycle's branch off `836faf8`): rewrite the `Repair` `after_long_help` epilog at `main.rs:241`.
2. **mnemonic-toolkit** (paired docs commit off `origin/master`): rewrite `docs/manual/src/40-cli-reference/42-md.md:367-379` to match. This is a **docs-only** toolkit commit — NOT a pin bump, NOT a code change.
3. **The lockstep is discipline-only, not gate-enforced.** `docs/manual/tests/lint.sh`'s only `--help`-consuming step is flag-coverage, which extracts flag **NAMES** (`grep -oE -- '--[a-z…]+'`) and asserts presence; **NO step diffs the `repair` epilog PROSE** against the manual. So the lint passes whether or not the false `rejected`-prose is corrected — there is **no automated lockstep gate** on epilog prose, and the L7 edit needs **no v0.9.0-`md`-binary build and no `MD_BIN` pin** to pass lint (the `repair` flag set is unchanged → flag-coverage is unaffected). The manual `42-md.md:367-379` false-`rejected` prose is corrected in lockstep with the `main.rs:241` help-epilog fix purely by **paired-PR mirror discipline** (human/PR review of the corrected prose); it is NOT gate-enforced, so it must be done by discipline. (The flag-coverage lint still passes trivially — fine to keep running it — but it does NOT prove the prose match.)

> If cross-repo authoring in one session is infeasible, file the toolkit manual edit as a **paired sibling PR** and note it in both repos' `FOLLOWUPS.md` — the preferred path is to land both together. Either way the manual edit stays a required deliverable of this cycle (paired-PR discipline), even though no lint gates it.

---

## 9. Per-finding tests (TDD — RED-first; FORMAL lane for M5/M2/M10/M11, batch for L4/L19/L7)

Each FORMAL-lane finding gets a **failing test first**, then the fix turns it GREEN; the H13/positive-control guards stay GREEN throughout. Run the **FULL `cargo test -p md-cli` suite** at each R0 (per the "R0 must run the full package suite" memory rule), not just targeted `--test` targets — CLI/parse phases ripple into help/version/argv lints outside any one target.

**M5 (funds — most adversarial coverage):**
- RED→GREEN: `wpkh(@0/<2;3>/0'/*)` (multipath-not-last) currently builds a `Descriptor` with `use_site_path.multipath = Some([2,3])` over a no-multipath tree (assert the divergence today) → after fix is **REJECTED** with a typed `CliError::TemplateParse` naming the multipath-not-final cause.
- H13 REGRESSION GUARD (must stay GREEN unchanged): re-assert all five — `lex_rejects_hardened_multipath_apostrophe`, `lex_rejects_hardened_multipath_h_form`, `lex_rejects_mixed_hardened_multipath`, `lex_rejects_malformed_double_marker_multipath`, `lex_accepts_nonhardened_multipath`. Add a fused case proving ordering: `wsh(multi(2,@0/<0'';1>/0'/*,@1/<0'';1>/0'/*))` still errors with the **hardened/malformed** message (H13 fires before M5), not the suffix message.
- POSITIVE CONTROL (must stay GREEN): normal multipath-last `wpkh(@0/<0;1>/*)` and `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))` still parse, multipath preserved, `parse_template` round-trips.
- `h`-in-origin sub-case: `wpkh(@0/48h/0h/0h/<0;1>/*)` (unconsumed `48h…` residue) now rejected (today: malformed `XPUBh/…`).
- Cross-validation belt: a synthetic test that forces a lexer/substitution divergence (if reachable via a crafted input) hits the `parse_template` cross-check and refuses to emit.

**M2:**
- RED→GREEN: `wpkh(@255/*)` currently **panics** at `by_i[&0]` → after fix returns a typed `CliError::TemplateParse` ("at most @254"). (Use `parse_template`/`resolve_placeholders` directly; assert `is_err()` with the range message, no panic.)
- `@254` boundary still ACCEPTS (dense `@0..=@254`); `@255` rejects. Also `wpkh(@255/*)` with `@0` present (`wsh(multi(2,@0/*,@255/*))`) — no silent `n=0` (rejected at the bound).

**M10:**
- RED→GREEN: `tr(@0/<0;1>/*)` + a depth-3 BIP-86 account xpub via `md encode --key @0=<depth-3 xpub>` currently rejected ("expected depth 4 ... got 3") → now ACCEPTED. Assert `ctx_for_template("tr(@0/<0;1>/*)") == ScriptCtx::SingleSig`.
- POSITIVE CONTROL: `tr(@0)` / `tr(@0/*)` classify SingleSig.
- NEGATIVE (over-acceptance guard): `tr(@0,{...})`, `tr(NUMS,multi_a(2,@0,@1,@2))`, `tr(50929b74...,multi_a(...))` still classify MultiSig → a depth-3 key in those forms still rejects; existing tr-script-tree tests (`tr_with_and_v_verify_older_inheritance`, `tr_tap_leaf_bare_pk_on_wire`, `tr_multi_branch_three_leaf_right_unbalanced`) stay GREEN.

**M11:**
- RED→GREEN: an xpub with valid base58check/version/length/depth but an **off-curve** (e.g. all-zero) `bytes[45..78]` currently passes `parse_key` → now rejected at parse with `CliError::BadXpub` ("not a valid secp256k1 point"). Construct the bad xpub by mutating a real depth-3/4 xpub's pubkey bytes to an off-curve value, re-base58check-encoding.
- POSITIVE CONTROL: the real `XPUB_DEPTH4` (`keys.rs:95`) and a real depth-3 BIP-86 xpub still parse.

**L4 / L19** (regression home = existing `crates/md-cli/tests/cli_output_class.rs`):
- Extend the existing `tests/cli_output_class.rs` (which already asserts the `Template`/`WatchOnly` advisory lines, incl. the cross-repo `byte_parity_advisory_lines`): `md repair` on a **wallet-policy** (keyed) md1 → stderr advisory is `WatchOnly`, not `Template`; on a **keyless template** md1 → still `Template`.
- `md encode --key @0=<xpub> …` (both `--json` and text paths) → `WatchOnly`; keyless `md encode <template>` → `Template`. (Capture stderr; assert the advisory string.)
- Confirm `byte_parity_advisory_lines` stays GREEN — the fix touches call-sites (which `OutputClass` is passed), NOT the advisory strings themselves, so cross-repo parity is preserved.

**L7:**
- `md repair --help` output no longer contains "rejected with a wire-format error" / "Non-chunked ... are rejected" (assert the false phrase is absent; assert the corrected phrasing present).
- Manual lockstep (toolkit) — **discipline, not a lint gate:** the corrected prose is present in `docs/manual/src/40-cli-reference/42-md.md:367-379` (human/PR-verified — there is NO lint step that diffs the epilog prose). The flag-coverage `lint.sh` still passes trivially (the `repair` flag set is unchanged), but that is NOT proof of the prose match and needs **no v0.9.0-`md` binary / `MD_BIN` pin**.

---

## 10. FOLLOWUP slugs

| Slug | Repo(s) | Tier | Note |
|---|---|---|---|
| `md-cli-lexer-substitution-divergence-multipath-not-last` (M5) | descriptor-mnemonic | resolve THIS cycle | flip to `resolved <md-cli-v0.9.0 SHA>` in the shipping commit |
| `placeholder-count-u8-overflow-panic` (M2) | descriptor-mnemonic | resolve THIS cycle | flip on ship |
| `tr-singlekey-bip86-depth3-false-reject` (M10) | descriptor-mnemonic | resolve THIS cycle | flip on ship |
| `parse-key-missing-secp256k1-point-check` (M11) | descriptor-mnemonic | resolve THIS cycle | flip on ship |
| `repair-encode-advisory-mislabels-watch-only-as-keyless-template` (L4+L19) | descriptor-mnemonic | resolve THIS cycle | one fix, three sites |
| `repair-help-epilog-stale-rejects-nonchunked-claim` (L7) | descriptor-mnemonic (+ toolkit manual companion) | resolve THIS cycle | paired toolkit-manual edit; underlying FOLLOWUP already RESOLVED |
| `md1-post-multipath-fixed-path-derivation-steps` (M5 DEFERRED capability) | descriptor-mnemonic | `v1+` | NEW — if real demand for BIP-389 post-multipath fixed steps appears, design a wire+`UseSitePath` representation (md-codec change → would force a toolkit pin). Until then, M5 rejects fail-closed. |

> **Status discipline (memory rule):** verify each slug's "open" status at decision time and flip it **in the shipping commit**, not after. Run `scripts/followup-reconcile.sh` if present.

---

## 11. RESOLVED DECISIONS (no open questions)

| # | Question | Decision | Rationale |
|---|---|---|---|
| D1 | M5: canonicalize vs reject the multipath-not-last / suffix form? | **REJECT (fail-closed), typed `TemplateParse`** | md1 + md-cli use-site model cannot represent post-multipath fixed steps; canonicalize would need a wire/`UseSitePath` change (md-codec bump + toolkit pin) — out of scope; matches cycle-1 H13 precedent (never silently encode a divergent card). |
| D2 | Is multipath-not-last illegal under BIP-389? | **NO — BIP-389 permits post-multipath path steps.** Frame the reject as an md1/md-cli **representability** limit, not a BIP-389 prohibition. | Verified against BIP-389 text ("Followed by zero or more /NUM... path elements"). Corrects the recon's "must be last" framing. |
| D3 | How does M5 preserve H13? | M5 checks path text **outside** `<…>` (suffix residue); H13's group-3 permissive capture + strict validator + substitution strip class `[0-9;]` are **untouched**; H13's validator fires first for hardened/malformed bodies. Five guard tests + full suite stay GREEN. | Disjoint input regions; ordering guarantee; §3.1.4. |
| D4 | M5: also add cross-validation in `parse_template`? | **YES (belt-and-suspenders), concretized:** for each `@i`, compare the lexer's `occ.multipath_alts.len()` to the multipath-step count of the **substituted `DescriptorPublicKey`** (NOT `tree` — `tree` carries no per-key multipath field); mismatch (or any substituted-key fixed step the lexer didn't record) → refuse to emit. | Constructible (checks the substituted key's derivation-path count, not a vacuous always-true assertion); makes any future regex drift fail-closed; cheap; funds-priority. The residue reject (edit-1) is the real fix; D4 is the checkable drift belt. |
| D5 | M2: bound the index or just checked-get? | **BOTH** — bound `max <= 254` before the `as u8` cast (primary) + checked `by_i.get` (defense-in-depth). | The bound is the real fix; the get guards future skip-density paths. |
| D6 | M10: fix the classifier or relax the depth gate? | **Classifier** (`ctx_for_template`) — scope to "`tr(` + exactly one `@i` + no `,` + no `{`" → SingleSig depth-3. | Surgical; keeps the depth gate strict so script-path-tr (depth-4) keys aren't over-accepted. BIP-86 account xpub = depth 3 (verified). |
| D7 | M11: where to put the point check + new dep? | `keys.rs:78`, before the copy; `bitcoin::secp256k1::PublicKey::from_slice(&bytes[45..78])`. **No new dep** (already used at `template.rs:461`). | bytes[45..78] = compressed pubkey; valid BIP-32 xpub key is on-curve (verified). |
| D8 | L4/L19: branch on what? | `descriptor.is_wallet_policy()` (md-codec `encode.rs:50-52`) → `WatchOnly` if true else `Template`, at all three sites; fold into one commit. | Mirrors the sound `md address` model; md-codec untouched. |
| D9 | L7: delete or rewrite; FOLLOWUP flip? | Rewrite the false sentence at `main.rs:241`; **no FOLLOWUP status flip** (slug already RESOLVED in md-codec-v0.35.0). | Underlying support shipped; only prose drifted. |
| D10 | L7 manual lockstep? | **YES — paired edit to toolkit `42-md.md:367-379`, required by CLAUDE.md manual-mirror DISCIPLINE, NOT lint-enforced.** The flag-coverage lint extracts flag NAMES only — NO step diffs the `repair` epilog prose — so the lint passes regardless of the prose edit; the edit needs no v0.9.0-`md` build and no `MD_BIN` pin. Corrected by human/PR review. | The manual mirrors the false prose (verified @ `42-md.md:367-379`); the obligation is paired-PR discipline (a lagging/human gate), not a mechanical lint gate (which checks names, not prose). |
| D11 | SemVer aggregate + pins? | **md-cli MINOR 0.9.0** (M10 driver); **md-codec NO BUMP**; **NO toolkit pin**; tag `descriptor-mnemonic-md-cli-v0.9.0` + `cargo publish -p md-cli`. | Toolkit deps md-codec only, never md-cli; all fixes md-cli-side. |
| D12 | GUI schema_mirror lockstep? | **NOT triggered** — no clap flag/subcommand/dropdown add/rename/remove. | M10 widens descriptor-string input, not a flag; rest are parse-internal / advisory / help-prose. |
| D13 | Implementation ordering? | **M11 → M2 → M10 → M5** (simplest→hardest; M5 last with the five H13 tests loaded), then **L4+L19** (one commit) + **L7** (paired). Single release. | M5 is the funds crux + H13-collision; lands on the M2-bounded `n`. |

---

## 12. Mandatory R0 gate

This is a **brainstorm spec — DESIGN ONLY, no code.** Per CLAUDE.md Conventions (first bullet): before ANY implementation (no implementer dispatch, no edits, no tag, no publish), this spec + the subsequent plan-doc MUST pass the **opus-architect R0 reviewer-loop to 0 Critical / 0 Important** — fold each review's findings → persist the review **verbatim** to `design/agent-reports/` → re-dispatch → repeat until GREEN. The reviewer-loop continues **after every fold** (folds can introduce drift). The full `cargo test -p md-cli` suite (not targeted `--test` targets) gates each per-phase R0. **The M5 H13-preservation argument (§3.1.4) is the review's primary focus.** No Critical/Important may remain open at any gate (start coding, advance phase, tag, ship).
