# BRAINSTORM / SPEC — cycle-14: close L22 (stdin secret read into un-scrubbed `String`)

**Status:** DESIGN ONLY — feeds the mandatory R0 loop (spec R0 → plan → plan R0 → TDD). NO code yet.
**Cycle:** constellation bug-hunt cycle-14 (the secret-memory-hygiene **tail** — collapses to **L22 only**).
**SemVer:** toolkit **MINOR v0.67.0** (off current `v0.66.0`). md/ms/mk codecs + CLIs **NO-BUMP**. GUI: **NO `schema_mirror` impact** (no clap flag added/removed/renamed). NO manual mirror. NO cross-repo (ms-cli Site 5 already `Zeroizing<String>`).
**Source SHA verified against:** `origin/master = 82c61e76` (`design(cycle-14): secret-memory-hygiene tail recon — collapses to L22 only`; toolkit `Cargo.toml` version `0.66.0`). Every citation below was re-grepped via `git show origin/master:<path> | grep -n` at write time. The working tree is on the v0.60.0 own-account branch — working-tree line numbers were NOT trusted.

> **MANDATORY R0 GATE (CLAUDE.md).** No implementation — no code, no implementer dispatch — until this brainstorm/SPEC passes an opus architect R0 review converged to **0 Critical / 0 Important**. Fold → persist the review verbatim to `design/agent-reports/` → re-dispatch → repeat until GREEN. The reviewer-loop continues after every fold (folds introduce drift). The same gate then re-applies to the plan-doc, and per-phase during TDD. R0 reviews MUST run the **full `cargo test -p mnemonic-toolkit` suite** (argv/mlock/zeroize sibling lints fire outside any one `--test` target — MEMORY `feedback_r0_review_run_full_package_suite`).

---

## 1. The finding (L22)

`### - [ ] L22` at `design/agent-reports/constellation-bughunt-2026-06-20.md:850`.

- **repo/class:** toolkit · **D-secret-leak** (tracked Cycle-B) · `w3-tk-slot-input-friendly-1`.
- **bug:** the `@N.<secret>=-` / `--*-stdin` sentinels keep secrets off argv, but the stdin readers materialize the secret into a **bare `String`** (no `Zeroize`/`Drop`). `apply_slot_stdin` stores it in the **persistent** field `SlotInput.value: String`; `read_stdin_passphrase` / `read_stdin_to_string` return a transient bare `String` consumed by the convert/restore/addresses handlers. So the secret lingers un-scrubbed in the heap until natural drop — partly defeating the argv-avoidance. The convert/bundle sites are **mlock-pinned but NOT zeroized** (mlock ≠ scrub — the pin prevents swap-out, it does not clear the buffer on drop).
- **class precedent:** the same residue class as the ms-cli stdin reader (`Zeroizing<String>`, Cycle A), the `Vec<u8>→Zeroizing<Vec<u8>>` field migration (v0.10.1 `ed5a1d9`, FOLLOWUP `resolved-slot-derived-account-zeroizing-field`), the `read_blob → Zeroizing<Vec<u8>>` reader-return migration (v0.33.3, FOLLOWUP `import-wallet-blob-zeroizing`), and the `decrypt_bsms_record → Zeroizing<String>` migration (v0.34.1, FOLLOWUP `bsms-decrypt-record-string-zeroizing`). L22 is the toolkit's **Cycle-B canonical "Site 1"** residual scrub gap — the last open site of the 5-site list **in the toolkit**.

### Severity framing (funds/secret-safety)

**Defense-in-depth (heap residue), NOT a wrong-address / funds-loss bug.** No observable CLI behavior or wire-format change — purely an in-memory hygiene change. `SlotInput` has **no `Serialize`/`Deserialize`** (verified: no serde on `slot_input.rs`), so there is no `--json` wire surface to drift. Best-effort caveat (same as `secret_string.rs`): the EMITTED bytes (stdout/pipe), the allocator-residue of intermediate moves, and third-party carriers (`bip39::Mnemonic`, `Xpriv`) remain out of scope — L22 closes only the OWNED `String` allocation we control.

---

## 2. Verified facts (the load-bearing audit, all vs `82c61e76`)

### 2.1 The two cited symbols

| Symbol | Live location | Current type |
|---|---|---|
| `SlotInput` struct | `slot_input.rs:97` (derive at `:96` = `#[derive(Debug, Clone, PartialEq, Eq)]`) | — |
| `SlotInput.value` | `slot_input.rs:100` `pub value: String` | bare `String` |
| `apply_slot_stdin` | `slot_input.rs:203-232` (reads `let mut buf = String::new()`; `slots[i].value = buf;` at `:225`) | writes bare `String` |
| `read_stdin_passphrase` | `convert.rs:758-769` `pub(crate) fn … -> Result<String, ToolkitError>` (`Ok(buf)` at `:769`) | bare `String` |
| `read_stdin_to_string` | `convert.rs:745-751` `pub(crate) fn … -> Result<String, ToolkitError>` (`Ok(buf.trim()…)` at `:750`) | bare `String` |
| `is_stdin_sentinel` | `slot_input.rs:109-111` — `self.value == "-"` at `:110` | `String == &str` |

`slot_input` is a **private bin `mod`** in normal builds (`main.rs:30` `mod slot_input;`); it is `pub mod` **only under `#[cfg(fuzzing)]`** (`lib.rs:177-178` `#[cfg(fuzzing)] pub mod slot_input;`). There is **no `pub use SlotInput`** anywhere. So `SlotInput` is NOT reachable from the public library API in a normal build — see D3 (m-1): the MINOR bump rests on the v0.10.1 `cfg(fuzzing)`-gated precedent, not on public-API reachability.

### 2.2 The zeroize-1.8.2 trait reality — **corrects the recon's load-bearing claim**

`zeroize = "1.8"` resolves to **1.8.2** (`Cargo.lock`). In that version (`src/lib.rs:622-623`):

```rust
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Zeroizing<Z: Zeroize>(Z);
```

with `impl Deref for Zeroizing<Z> { type Target = Z; … }` (`:660`) — i.e. it derefs to the **inner type** (`String`), NOT to `str`.

Therefore, against the recon/task premise *"`Zeroizing` has no `Eq`; its `Debug` is redacted"*:

- **FALSE — `Zeroizing<String>` IS `Eq` and `PartialEq`** (the derive delegates to `String: Eq`). The `#[derive(Eq)]` on `SlotInput` would **still compile** with `value: Zeroizing<String>` — **no derive needs dropping**.
- **FALSE — `Zeroizing`'s `Debug` is the derived tuple-struct `Debug`, which is NON-redacting.** `format!("{:?}", Zeroizing::new("secret".to_string()))` prints `Zeroizing("secret")` — it **leaks the secret** into any `{:?}` (panic messages, `assert_eq!` failure output, `{slot:?}`/`{e:?}` logging). The hygiene snag is **inverted**: raw `Zeroizing<String>` compiles but *re-introduces a (debug-only) leak vector* the bare `String` already had.

These two facts are the spine of the design decision in §3. **R0 must re-verify them against the resolved `zeroize` version** (re-grep `Cargo.lock` + `~/.cargo/.../zeroize-1.8.*/src/lib.rs:622`).

### 2.3 `Zeroizing<String>` is NOT a drop-in at the cited sites

- **`is_stdin_sentinel` (`:110`):** `self.value == "-"` is `String == &str`. `Zeroizing<Z>` only derives `PartialEq<Zeroizing<Z>>` — there is **no `PartialEq<str>`/`PartialEq<&str>`** impl. So `self.value == "-"` **breaks**; needs `self.value.as_str() == "-"` (or `*self.value == "-"`).
- **convert.rs `as_deref()` (`:990-991`):** `effective_passphrase.as_deref()` relies on `Deref<Target = str>` to yield `Option<&str>`. `Zeroizing<String>: Deref<Target = String>`, so `Option<Zeroizing<String>>::as_deref()` yields `Option<&String>`, and `.unwrap_or("")` (a `&str`) **type-mismatches**. Needs `.as_deref().map(String::as_str)` or keep the convert locals as `Option<String>`.
- **convert.rs repair `vec![primary_value.clone()]` (`:1025`):** the `if`-arm builds `Vec<String>` via `split_whitespace().map(|s| s.to_string())` (`:1020-1023`); if `primary_value: Zeroizing<String>`, `.clone()` is `Zeroizing<String>` and the two arms no longer unify to `Vec<String>`. Needs `(*primary_value).clone()` or `primary_value.to_string()`.
- **`SlotInput.value` write-back sites (the `@env:` channel) — 3 sites, each itself L22 secret residue:** `resolve_env_var_sentinel(&s.value, …)? -> Result<String, _>` (`env_sentinel.rs:59`) resolves a `@env:VAR` sentinel to the **actual secret phrase** and stores it back into `s.value` (gated on `s.subkey.is_secret_bearing()`). When `s.value: SecretString` this `s.value = <String> ;` no longer type-checks (`String ≠ SecretString`) AND, critically, it is a path that **materializes a secret into the field via the `@env:` channel** — bare `String` residue that the bare-stdin path's sibling. The three sites: `bundle.rs:2629`, `import_wallet.rs:1396`, `verify_bundle.rs:1883`. Each needs `s.value = SecretString::new(resolve_env_var_sentinel(&s.value, &flag)?);`. **This is the key I-1 finding: the field-wrap also closes the `@env:` write-back residue, not just the stdin `=-` path — it strengthens L22.**
- **`import_wallet.rs:1233` phrase-overlay clone:** `let phrase_overlays: Vec<(u8, String)> = args.slot.iter().filter(|s| s.subkey == SlotSubkey::Phrase).map(|s| (s.index, s.value.clone())).collect();` (`:1229-1234`). With `value: SecretString`, `.clone()` returns `SecretString ≠ String` → the tuple no longer unifies with the annotated `Vec<(u8, String)>` (won't compile), AND it clones the **secret seed phrase** out of the scrubbing newtype into a bare `Vec<(u8, String)>`. Minimal fix: `.map(|s| (s.index, s.value.to_string()))` (restores compile). **Open question for the plan:** whether the `phrase_overlays` Vec should itself hold `SecretString` (i.e. `Vec<(u8, SecretString)>`) so the overlay does not re-leak — non-trivial because `apply_seed_overlay`'s signature consumes `&[(u8, String)]`; the plan must decide (minimum: fix the compile + do not regress the existing hygiene).

#### `SlotInput.value` (MIGRATING → `SecretString`) vs `FromInput.value` (NOT migrating — stays `String`)

The `.value` field name is shared by **two distinct structs** and the census must not conflate them:

- **`SlotInput.value` (`slot_input.rs:100`) — MIGRATES to `SecretString`.** All `s.value` sites on a `SlotInput` are in scope: the field reads (§2.6, deref-absorbed), the `apply_slot_stdin` store (D2), the 3 `@env:` write-backs, and the `:1233` overlay clone above. These are the sites that **break** and need edits.
- **`FromInput.value` (`convert.rs:131` — a SEPARATE `pub struct FromInput { pub node, pub value: String }`) — stays `String`, does NOT migrate.** The `from.value`/`primary.value`/`f.value`/`sh.value` and seed_xor `share`/`sh.value` sites that compare `== "-"` (e.g. `addresses.rs:116`, `convert.rs:837,868,1810`, `derive_child.rs:129,139`, `final_word.rs:67`, `ms_shares.rs:266`, `restore.rs:327,792`, `seed_xor.rs:294`) are all on `FromInput` (or its plain-`String` value), NOT `SlotInput` — they **do NOT break** and are **out of L22 scope** (already handled by their own `Zeroizing<String>` locals where applicable). Treat `FromInput.value` as a non-migrating field throughout.

### 2.4 Reader-return-type change is NOT "mostly no-op deref" — it touches ~26 sites with real edits

Full census of `read_stdin_passphrase` / `read_stdin_to_string` call sites (the recon's "~30"):

- **14 sites ALREADY wrap** `Zeroizing::new([crate::cmd::convert::]read_stdin_*(…))` (re-grepped vs `82c61e76`: `git grep -nE 'Zeroizing::new\((crate::cmd::convert::)?read_stdin_(passphrase|to_string)'` = **14**, not the recon's "~16"): `derive_child.rs:140`, `electrum_decrypt.rs:115`, `final_word.rs:68`, `import_wallet.rs:2300`, `ms_shares.rs:267,374`, `seed_xor.rs:172,302`, `seedqr.rs:178,245`, `silent_payment.rs:242` (fully-qualified `Zeroizing::new(crate::cmd::convert::read_stdin_passphrase(…))`), `slip39.rs:304,424,607`. If the reader returns `Zeroizing<String>`, these become `Zeroizing::new(Zeroizing<String>)` = `Zeroizing<Zeroizing<String>>` — a **type error**. Each needs the now-redundant outer `Zeroizing::new(…)` **removed** (14 edits).
- **~12 sites consume bare into an annotated `String`/`Option<String>`** via `if/else`: `addresses.rs:150,160`; `convert.rs:854,862,869`; `restore.rs:400,411,830,840,1292,1300,3045,3053`. These have explicit `let x: String = if stdin { read_stdin_*()? } else { …String… }` annotations; the `else` arm yields `String` (from `resolve_env_var_sentinel`/`String::new()`/`.clone()`). Changing the reader return type forces, per site: drop/flip the `: String` annotation **and** wrap the `else` arm in `Zeroizing::new(…)` so the `if/else` unifies (exactly the v0.34.1 `decrypt_bsms_record` `if/else`-unify pattern).

**Conclusion:** "flip the reader return type once; deref absorbs the rest" — the recon's headline — is **materially wrong**: it is ~26 real call-site edits (14 un-wrap + ~12 annotate-and-rewrap), several with downstream `as_deref()`/`Vec<String>` friction (§2.3). The fix-shape decision (§3) is driven by this.

### 2.5 Test consumers requiring `SlotInput: PartialEq + Debug`

`slot_input.rs` test module has **~15** `assert_eq!(parse_slot_input("…").unwrap(), slot(idx, sub, "…"))` comparisons (`:475,482,493,512,520,527,534,541,548,558,581`, …) plus `slot(…)` builders (`:379`). `assert_eq!` on whole `SlotInput` values requires **both `PartialEq`** (the comparison) **and `Debug`** (the failure-print). So whichever field type we pick, `SlotInput` MUST retain working `PartialEq` + `Debug`. No `==` on `SlotInput` outside tests; no `{slot:?}` logging of a populated `SlotInput` in non-test source (grep: the only `SlotInput`-typed values flow through `detect_bundle_mode`/`validate_*` by `&` and into `&s.value` field reads — see §2.6).

### 2.6 `SlotInput.value` operation sites — reads (Deref-absorbed) vs writes/clones (break)

**Important scope correction (I-1).** The earlier framing "all non-test `.value` operations compile unchanged (auto-deref)" was WRONG — it conflated `SlotInput.value` (migrating → `SecretString`) with `FromInput.value` (separate `String` field, §2.3 disambiguation). Splitting the `SlotInput.value` sites by operation kind:

**(a) Field READS — deref-absorbed, compile unchanged.** All non-test `&slot.value` reads pass into `&str`/`&String`-accepting fns: `DerivationPath::from_str(&p.value)` (`bundle.rs:610,1530,1694`; `verify_bundle.rs:1461,1585`; `restore.rs`), `Fingerprint::from_str(&s.value)` (`bundle.rs:1671`; `verify_bundle.rs:1579`), `normalize_xpub_prefix(&m.value)` (`bundle.rs:634`), and `bundle.rs:2103` `Some((s.index as usize, &s.value))` (the `&value` is consumed as `&str`/`&String` downstream). With a `Deref<Target=str>`-bearing field type these compile **unchanged** (auto-deref `&SecretString → &str`). This is the one place the recon's "deref absorbs reads" claim holds — but only for **field reads**.

**(b) Field WRITES / CLONES — BREAK; these are NEW edit sites (the I-1 omission).** `SlotInput.value` is also *written* and *cloned*, and those do not deref-absorb:

| Site | Current code (vs `82c61e76`) | Edit needed | Note |
|---|---|---|---|
| `slot_input.rs:225` (`apply_slot_stdin`) | `slots[i].value = buf;` (`buf: String`) | `slots[i].value = SecretString::new(buf);` | the stdin `=-` path — D2 already covers this |
| `bundle.rs:2629` | `s.value = resolve_env_var_sentinel(&s.value, &flag)?;` | `s.value = SecretString::new(resolve_env_var_sentinel(&s.value, &flag)?);` | **`@env:` write-back — secret residue; the wrap CLOSES it** |
| `import_wallet.rs:1396` | same `@env:` write-back | `s.value = SecretString::new(resolve_env_var_sentinel(&s.value, &flag)?);` | **`@env:` residue** |
| `verify_bundle.rs:1883` | same `@env:` write-back | `s.value = SecretString::new(resolve_env_var_sentinel(&s.value, &flag)?);` | **`@env:` residue** |
| `import_wallet.rs:1233` | `.map(\|s\| (s.index, s.value.clone()))` into `Vec<(u8, String)>` (`:1229-1234`, filtered to `SlotSubkey::Phrase`) | `.map(\|s\| (s.index, s.value.to_string()))` (min); plan to decide if `phrase_overlays` should be `Vec<(u8, SecretString)>` | `.clone()` → `SecretString ≠ String` (won't compile) + clones the secret phrase out of the scrubbing newtype |

`resolve_env_var_sentinel -> Result<String, _>` (`env_sentinel.rs:59`), so the `@env:` write-backs need an explicit `SecretString::new(…)` re-wrap. **The three `@env:` write-backs are themselves L22 residue** — the sentinel resolves `@env:VAR` to the actual secret phrase and stores a bare `String` into the field — so wrapping the field closes them too, strengthening the L22 fix beyond the stdin path. None of these sites is a `FromInput.value` site (those stay `String`, §2.3).

---

## 3. Resolved design decisions

### D1 — Fix shape: **persistent field first; do NOT flip the shared reader return type**

**Decision:** wrap the **persistent secret-bearing field** `SlotInput.value` (the thing L22 actually names — the value that *lingers*) and the **three convert.rs handler locals** that are mlock-pinned-but-unscrubbed (`effective_passphrase`, `effective_bip38_passphrase`, `primary_value`). **Leave `read_stdin_passphrase` / `read_stdin_to_string` returning bare `String`.**

**Justification:**
- The readers' own transient `buf` is a short-lived stack-rooted local; the **lingering** secret L22 names is the one stored in the **persistent field** (`SlotInput.value`) and the **handler-scope locals** (pinned for the whole `run()`). Those are the residue that matters.
- Flipping the shared `pub(crate)` reader return type is the **highest-churn, highest-risk** option (~26 edits, §2.4) for the **least** marginal benefit (the already-wrapping 14 callers are *already scrubbed*; flipping the return type only *moves* their wrap, net-zero hygiene). It would also create `Zeroizing<Zeroizing<String>>` foot-guns and `as_deref()`/`Vec<String>` friction. The v0.33.3 `read_blob` precedent worked because **all** its read sites were bare and deref-absorbed; here 14/26 are the opposite (already-wrapped), so the precedent inverts.
- **`SlotInput.value` write/clone edit sites that ride along with the field migration (NEW — I-1 fold):** wrapping the field forces 4 additional `SlotInput.value`-operation edits beyond the `apply_slot_stdin` store (full table in §2.6(b)): the 3 `@env:` write-backs (`bundle.rs:2629`, `import_wallet.rs:1396`, `verify_bundle.rs:1883` → `s.value = SecretString::new(resolve_env_var_sentinel(…)?)`) and the `import_wallet.rs:1233` overlay clone (`.value.clone()` → `.value.to_string()`). The three `@env:` write-backs are **themselves L22 secret residue** (a secret materialized into the bare field via the `@env:` channel), so the field-wrap *closes* them — L22 is closed on both the stdin `=-` path AND the `@env:` path. The `:1233` clone is a compile-break (`SecretString ≠ String`) that also currently copies the secret phrase into a bare `Vec<(u8, String)>`; the plan decides whether the overlay Vec should hold `SecretString` (min: `.to_string()` to restore compile without regressing hygiene). These are `SlotInput.value` sites; the structurally-similar `FromInput.value == "-"` sites do NOT migrate (§2.3).
- **Coverage that source-wrapping the field+locals does NOT reach, and how we close it:** the already-wrapping callers (§2.4 list) are *already* `Zeroizing` — no action, no regression. The non-wrapping bare-`String` readers feed exactly into (a) `SlotInput.value` via `apply_slot_stdin` — closed by wrapping the field; (b) the three convert.rs locals — closed by wrapping those locals; (c) `restore.rs`/`addresses.rs` locals — these flow into `Zeroizing<Vec<u8>>` entropy via `resolve_seed_entropy`/derivation and are mlock-pinned; the residual bare-`String` `passphrase`/`from_value` locals there are **in-scope for an explicit local wrap in the same cycle** (cheap: each is a single `let x: Zeroizing<String> = if … else …`, no return-type change, no caller fan-out). **R0 decision point:** include the restore/addresses local wraps in scope (recommended — completes Site 1 uniformly) vs. defer them to a `restore-addresses-stdin-local-zeroizing` FOLLOWUP (acceptable — they are pinned and short-lived). Default: **include** (they are the same `if/else`-unify edit, no fan-out).

So the fix is **fix-at-the-owned-allocation**, not fix-at-the-shared-reader. The reader stays `String`; each *owner* of a lingering secret wraps at its binding.

### D2 — The `Eq, Debug` snag: **a `SecretString`-style newtype with hand-impl'd `PartialEq`/`Eq` + redacting `Debug`** (the key decision)

The three candidates from the task, re-evaluated against the §2.2 zeroize-1.8.2 reality:

| Option | `PartialEq`/`Eq` on `SlotInput` | `Debug` leak? | Verdict |
|---|---|---|---|
| **(a) raw `Zeroizing<String>` + keep derives** | ✓ compiles (`Zeroizing<String>: Eq`) | **✗ LEAKS** — derived `Debug` prints `Zeroizing("secret")` into `assert_eq!` failures / any `{:?}` | **Reject** — re-introduces a debug-only secret-leak the hygiene cycle should not ship |
| **(b) reuse existing `SecretString` as-is** | **✗ breaks** — `SecretString` has only `#[derive(Clone)]`, no `PartialEq`/`Eq` → ~15 `assert_eq!` + `is_stdin_sentinel` fail | redacting ✓ | **Reject as-is** — needs trait additions anyway |
| **(c) explicit zeroize-at-scope-end, keep `String`** | ✓ | n/a | **Reject** — `apply_slot_stdin` stores the value into a struct that outlives the fn; there is no single scope-end to scrub; mlock ≠ scrub (recon). Does not scrub on all drop paths. |

**Chosen: a dedicated `value`-field type that is `Zeroizing<String>` inside but carries the two trait impls `SlotInput` needs.** Two equivalent realizations — pick **D2-i** unless R0 prefers reuse:

- **D2-i (preferred): extend the existing `SecretString` newtype** (`src/secret_string.rs`, already `Zeroizing<String>` inner, `Deref<Target=str>`, redacting `Debug`, `Display`, transparent `Serialize`, `Clone`) with:
  - `impl PartialEq for SecretString { fn eq(&self, o) -> bool { self.0 == o.0 } }` — **plain (non-constant-time) compare**. Justification: `SlotInput` equality is used **only in tests** (`assert_eq!` of parse output vs an expected literal) and in `is_stdin_sentinel`'s `== "-"` check (a public-sentinel test, not a secret-vs-secret authentication compare). There is **no security boundary** where a timing oracle on `SlotInput`/`SecretString` equality leaks anything (the compared value is either a test fixture or the literal `"-"`). A constant-time compare here would be cargo-cult; plain `String` `eq` is correct and keeps the existing zero `subtle`-dependency. **(R0 must confirm no security-sensitive `SecretString` equality consumer exists — grep `SecretString` `==`/`PartialEq` consumers; today there are none.)**
  - `impl Eq for SecretString {}` (total equality holds — `String: Eq`).
  - `Debug` stays the **existing length-only redaction** (`SecretString(<N chars redacted>)`) — so `assert_eq!` failures and any `{slot:?}` print the redaction, never the secret. This is the strict improvement over option (a).
  - Then `SlotInput.value: SecretString`. **Keep `#[derive(Debug, Clone, PartialEq, Eq)]` on `SlotInput`** — all four now satisfied by `SecretString` (no derive dropped, no hand-impl on `SlotInput`).
  - `is_stdin_sentinel`: `self.value == "-"` → add `impl PartialEq<str> for SecretString` (so `self.value == *"-"`/`&*self.value == "-"`), **or** rewrite as `&*self.value == "-"` (Deref to `str`). Prefer the rewrite (no extra impl, explicit).
  - **Wire-shape invariant preserved:** `SecretString`'s `Serialize` is transparent (byte-identical to `String`), and `SlotInput` is not serialized anyway — so no `--json` change either way.

- **D2-ii (fallback): a new local newtype** identical to D2-i but private to `slot_input.rs`, if R0 objects to widening `SecretString`'s public surface (it is a `pub` type in a `pub mod`). Adding `PartialEq`/`Eq` to a `pub` type is **additive** (no breakage) — so D2-i's surface growth is harmless. Default to **D2-i** (reuse beats a second newtype; one redaction definition, one serialize-transparency test).

**Trait-impl consequences, spelled out:**
- `PartialEq`/`Eq`: **plain (structural) `String` compare**, NOT constant-time — justified above (test-only + public-sentinel equality, no auth boundary).
- `Debug`: **redacting** (length-only), strictly better than the bare-`String` status quo (which would print the secret) and than raw `Zeroizing` option (a).
- `Clone`: already present (deep-clones the `Zeroizing<String>`, each copy scrubs on drop).
- `Display`/`Deref<Target=str>`/`Serialize`: unchanged — text-path and (absent) wire-path rendering identical.

**`apply_slot_stdin` change:** `slots[i].value = buf;` (`:225`) where `buf: String` → `slots[i].value = SecretString::new(buf);` (wrap once at the persistent-field write). The transient `buf` itself: optionally `Zeroizing::new` it too for the strip-newline window, but it is consumed-by-move into `SecretString::new` immediately, so the wrap-at-store suffices.

### D3 — SemVer: **MINOR v0.67.0**

**MINOR is correct by the v0.10.1 precedent, NOT by public-API reachability** (m-1 fold). `SlotInput` is NOT reachable from the public library API in a normal build: `slot_input` is a **private bin `mod`** (`main.rs:30`); it is `pub mod` **only under `#[cfg(fuzzing)]`** (`lib.rs:177-178`), and there is **no `pub use SlotInput`**. So the earlier "reachable from the public API" rationale was false and is dropped. The bump is MINOR because this change is **structurally identical to the v0.10.1 `cfg(fuzzing)`-gated `derive`/`synthesize` public-field type change** (`Vec<u8> → Zeroizing<Vec<u8>>`), which took a **MINOR** per the FOLLOWUPS record (`resolved-slot-derived-account-zeroizing-field`). A `cfg(fuzzing)`-exposed `pub` field's type change is given a MINOR by that established convention even though it is not reachable in a normal build. The readers stay `pub(crate)` `String` (D1) → no public-signature change there. **Conclusion unchanged: MINOR v0.67.0.**

NO codec bump (md/ms/mk untouched). NO GUI `schema_mirror` (no clap flag added/removed/renamed; `SlotInput` is not a flag or dropdown enum). NO manual mirror (no CLI-surface flag change). NO cross-repo (ms-cli Site 5 done).

### D4 — Zeroize-lint rows (`tests/lint_zeroize_discipline.rs`)

The lint has **two** gates that this change trips:
1. **DECLARED→evidence** (`every_canonical_zeroize_row_has_evidence_anchor`): each `ZEROIZE_ROWS` row needs an evidence substring in its `source_file`.
2. **SOURCE→declared completeness** (`every_secret_bearing_src_file_is_declared_or_allowlisted`): every `src/*.rs` matching `SECRET_PATTERNS` (`Zeroizing::new(`, `SecretString::new(`, `: Zeroizing<`, `: SecretString`) MUST be a `ZEROIZE_ROWS.source_file` or in `NON_ROW_SECRET_FILES`.

Wrapping `SlotInput.value: SecretString` makes `src/slot_input.rs` newly match `: SecretString` (and `SecretString::new(` in `apply_slot_stdin`) → it becomes secret-bearing → **a new row is MANDATORY** (else gate 2 fails). Add:

```rust
// ---- slot_input.rs (v0.67.0 — L22: stdin secret no longer lingers in a bare String) ----
ZeroizeRow {
    label: "SlotInput value field is SecretString (Zeroizing<String> inner) — L22",
    source_file: "src/slot_input.rs",
    evidence: &["pub value: SecretString", "SecretString::new"],
},
```

If §3-D1's convert/restore local wraps land in files already declared, no new file-rows are needed there (`convert.rs`, `restore.rs`, `addresses.rs` are all already `ZEROIZE_ROWS.source_file`s) — but add **labeled documentation rows** per the cycle's "anchor relabel + new row" directive so the L22 sites are auditable:

```rust
ZeroizeRow {
    label: "convert stdin-secret handler locals (effective_passphrase / bip38 / primary_value) wrap in Zeroizing<String> — L22",
    source_file: "src/cmd/convert.rs",
    evidence: &["Zeroizing<String>", "Zeroizing::new"],
},
```
(`convert.rs` already has Zeroizing rows; this row's evidence already matches — it is documentation, harmless, and pins the L22 site. R0 may collapse it into an existing convert row label instead.)

**Floor / count guards:**
- `SECRET_FILE_FLOOR = 35` (`lint_zeroize_discipline.rs:452`) is a `>=` floor; adding `slot_input.rs` makes the partition 36 → still passes. **Bump the floor `35 → 36` (lint:452)** in the same cycle (hygiene: the floor should track the true partition so a *future* drop is caught at the new level; the comment says "deleting a secret-bearing file is a conscious choice" — adding one is too). R0 to confirm the new partition count by running the scan. **The floor edit is the `SECRET_FILE_FLOOR` constant at lint:452 — do NOT confuse it with the stale doc comment.**
- **Pre-existing stale comment (NOT this cycle's bug — m-3):** the doc comment at `lint_zeroize_discipline.rs:370` says "Loose bound (24..=35)" while the actual assert at lint:375 is `(18..=60)`. This is a stale comment from before the bound was widened; it is **not** related to the `SECRET_FILE_FLOOR=35` floor and **not** this cycle's edit. Do not touch it (out of scope); just don't let the coincidental "35" mislead the floor bump.
- `canonical_zeroize_list_has_expected_row_count` bound is `(18..=60)` (lint:375) — +1/+2 rows stays inside. No bound edit needed.
- `NON_ROW_SECRET_FILES` non-empty tripwire (`non_row_secret_allowlist_is_non_empty_…`) — untouched; `slot_input.rs` goes to a ROW, not the allowlist.

**Sibling lints to keep green** (run FULL suite in R0): `lint_safety_first_party_mlock.rs` (the existing mlock pins on the convert/bundle `String`s must stay — D1 does not remove them), `lint_argv_secret_flags.rs` (no flag surface touched, but a CLI-adjacent type change can ripple — MEMORY `feedback_r0_review_run_full_package_suite`), the `secret_string.rs` `serializes_byte_identically_to_string` / `debug_redacts_the_secret` tests (must stay green after adding `PartialEq`/`Eq`).

---

## 4. RED-test sketches (TDD — tests before impl)

All tests live in the **BIN target** → `cargo test -p mnemonic-toolkit`. NEVER `cargo fmt` the toolkit (mlock.rs exempt).

1. **T1 — type-level field assertion (compile-fences the fix).** In `slot_input.rs` tests: a function that requires `SlotInput.value: SecretString` to type-check, e.g.
   ```rust
   fn _assert_value_is_secret_string(s: &SlotInput) -> &SecretString { &s.value }
   ```
   plus an assertion that `SecretString` is the `Zeroizing`-backed type (`fn _assert_zeroizing<T: zeroize::ZeroizeOnDrop>(){}` is not callable on `SecretString` directly — instead assert via the lint row + a `Deref<Target=str>` round-trip). RED before the field migration.
2. **T2 — `SecretString` trait additions.** In `secret_string.rs` tests: `assert_eq!(SecretString::new("a".into()), SecretString::new("a".into()))` and `assert_ne!(…, SecretString::new("b".into()))` (RED until `PartialEq`/`Eq` land); keep the existing `debug_redacts_the_secret` GREEN (proves option-(a)'s leak is avoided — `format!("{:?}", SecretString::new("supersecret".into()))` must NOT contain the secret).
3. **T3 — `is_stdin_sentinel` still works.** `parse_slot_input("@0.phrase=-").unwrap().is_stdin_sentinel()` is `true`; `@0.xpub=-` is `false`. RED if the `== "-"` rewrite regresses.
4. **T4 — no-behavior-change regression (end-to-end stdin `=-`).** Drive `apply_slot_stdin` with a `Cursor` over `b"correct horse\n"` on a `@0.phrase=-` sentinel slot; assert the resulting `slot.value` Derefs to `"correct horse"` (newline stripped) — identical to today. Plus a CLI-level `bundle --slot @0.phrase=-` smoke (stdin pipe) producing the same output as the pre-change binary. Proves the `=-` path still works end-to-end (purely in-memory change).
   - **T4b — `@env:` write-back regression (the 3 new sites, I-1).** Set an env var to a phrase, drive `resolve_env_sentinels` (bundle / import-wallet / verify-bundle) on a secret-bearing `@N.phrase=@env:VAR` slot, and assert the resolved `s.value` Derefs to the env-var phrase — byte-identical to today (the wrap is purely in-memory). Compile-fences the `s.value = SecretString::new(resolve_env_var_sentinel(…)?)` edit at `bundle.rs:2629`, `import_wallet.rs:1396`, `verify_bundle.rs:1883`. RED before those re-wraps land (the bare `s.value = <String>` won't compile against `SecretString`).
   - **T4c — `import_wallet.rs:1233` phrase-overlay clone.** Drive the `phrase_overlays` collection on a `SlotSubkey::Phrase` slot and assert the overlay still carries the phrase (Deref / `.to_string()`), unchanged from today. Compile-fences the `.value.clone() → .value.to_string()` (or `Vec<(u8, SecretString)>`) edit. RED before it lands.
5. **T5 — `assert_eq!` parse tests still pass.** The existing ~15 `assert_eq!(parse_slot_input(…).unwrap(), slot(…))` must stay GREEN (proves `SlotInput: PartialEq + Debug` survived the field-type change via `SecretString`'s new impls). The `slot()` helper changes to `value: SecretString::new(value.to_string())`.
6. **T6 — drop-scrub (best-effort, if feasible).** A `Zeroizing`-on-drop behavioral test is hard to assert reliably in safe Rust (post-drop read is UB). Prefer the **type-level** guarantee (T1 + lint row) as the authoritative evidence, mirroring how v0.10.1/v0.33.3 asserted via the lint anchor rather than a post-drop memory probe. Document this in the SPEC: scrub is structurally guaranteed by `Zeroizing<String>`'s `Drop`, evidenced by the type + lint, not by a flaky memory-inspection test.
7. **T7 — lint GREEN.** `every_secret_bearing_src_file_is_declared_or_allowlisted` + `every_canonical_zeroize_row_has_evidence_anchor` pass with the new row(s); `secret_files.len() >= 36`.

---

## 5. Out of scope

- **`OOS-reader-return-type-flip`:** flipping `read_stdin_passphrase`/`read_stdin_to_string` to `Zeroizing<String>` — rejected (D1); the already-wrapping 14 callers make it net-zero-benefit, high-churn. If a future cycle wants the transient `buf` scrubbed at the reader, it is a separate focused change (wrap `buf` in-fn as `Zeroizing<String>`, return `.to_string()`), filed as FOLLOWUP `stdin-reader-transient-buf-zeroizing` if R0 wants it tracked.
- **`OOS-third-party-carriers`:** `bip39::Mnemonic` / `Xpriv` interiors — covered by `lint_safety_third_party_blocked.rs` + SPEC §3 OOS, unchanged.
- **`OOS-emitted-bytes`:** stdout/pipe/terminal residue — same allocator-residue limit documented in `secret_string.rs`.
- **`OOS-restore-addresses-local-wraps` (conditional):** if R0 elects to defer the restore/addresses bare-`String` `passphrase`/`from_value` locals (D1 default = include), file `restore-addresses-stdin-local-zeroizing`.

---

## 6. FOLLOWUP disposition

At SHIP (integrated **v0.67.0**), in the shipping commit (MEMORY `feedback_followup_status_discipline` — flip status in the shipping commit, verify "open" at decision time):

- **Tick L22** in `design/agent-reports/constellation-bughunt-2026-06-20.md` — `### - [ ] L22` (`:850`) → `### - [x] L22` + a `<!-- FIXED cycle-14 (toolkit v0.67.0 @<sha>) — SlotInput.value + convert handler locals are SecretString (Zeroizing<String> inner, redacting Debug); both the stdin `=-` AND the `@env:` write-back (bundle/import-wallet/verify-bundle) secret-residue paths scrubbed on drop. mlock pins preserved. No wire/behavior change. Whole-diff review GREEN. -->` note. **Re-grep `:850` at ship time** (the report mutates; anchors decay).
- **Flip the Cycle-B Site-1 status** in `design/FOLLOWUPS.md`: the `secret-memory-hygiene-cycle-b` Site-1 residual scrub gap (`:1211` Site 1 list) — annotate that Site 1's *Zeroizing* leg is now complete (the mlock leg was done at v0.10.0; L22 closed the scrub gap at v0.67.0). The canonical 5-site list is now fully closed in the toolkit (Sites 2-4 at v0.10.x, Site 5 in ms-cli, Site 1 scrub at v0.67.0).
- **Cite, do NOT re-open**, `resolved-slot-derived-account-zeroizing-field` (v0.10.1) and `import-wallet-blob-zeroizing` (v0.33.3) / `bsms-decrypt-record-string-zeroizing` (v0.34.1) as the **precedents** this fix mirrors (same playbook, `String` instead of `Vec<u8>`).
- **Post-cycle FOLLOWUP burndown OFFER** (MEMORY `feedback_post_cycle_followup_burndown`): after ship, enumerate any newly-filed slugs (`stdin-reader-transient-buf-zeroizing`, `restore-addresses-stdin-local-zeroizing` if filed) + effort, AskUserQuestion all-or-select.

---

## 7. Resolved-decisions table

| # | Decision | Resolution | Why / consequence |
|---|---|---|---|
| **D1** | Fix shape | **Wrap the persistent field + convert handler locals (+restore/addresses locals); do NOT flip the shared reader return type** | The lingering secret L22 names is the field + pinned locals; flipping the `pub(crate)` reader is ~26 edits, net-zero benefit (14 callers already wrap — §2.4 m-2), and creates `Zeroizing<Zeroizing<…>>` + `as_deref()`/`Vec<String>` friction. Field/local wrapping covers every owner; already-wrapping callers unaffected. Wrapping the field also forces 4 `SlotInput.value` write/clone edits (§2.6(b), I-1): the 3 `@env:` write-backs (`bundle.rs:2629`/`import_wallet.rs:1396`/`verify_bundle.rs:1883` → `SecretString::new(resolve_env_var_sentinel(…)?)`, **themselves L22 secret residue the wrap closes**) + the `import_wallet.rs:1233` overlay clone (`.clone()`→`.to_string()`). `FromInput.value` does NOT migrate (§2.3). |
| **D2** | `Eq`/`Debug` snag (KEY) | **`SlotInput.value: SecretString`** — extend the existing `secret_string.rs` newtype with **plain (non-CT) `PartialEq`/`Eq`** + keep its **redacting `Debug`**; keep `#[derive(Debug,Clone,PartialEq,Eq)]` on `SlotInput` | zeroize-1.8.2 `Zeroizing<String>` IS `Eq` but its `Debug` LEAKS (corrects the recon). `SecretString` redacts but lacked `Eq`/`PartialEq`. Plain compare is correct: equality is test-only + the public `"-"` sentinel — no auth/timing boundary. `is_stdin_sentinel` `== "-"` → `&*self.value == "-"`. |
| **D3** | SemVer | **MINOR v0.67.0** (off v0.66.0) | MINOR by the **v0.10.1 precedent** (m-1), NOT public-API reachability: `slot_input` is a private bin `mod` (`main.rs:30`), `pub mod` only under `#[cfg(fuzzing)]` (`lib.rs:177-178`), no `pub use SlotInput`. The change is structurally identical to v0.10.1's `cfg(fuzzing)`-gated `derive`/`synthesize` field-type change (`Vec<u8>→Zeroizing<Vec<u8>>` → MINOR per FOLLOWUPS). Readers stay `pub(crate) String` → no public-sig change. No codec/GUI/manual/cross-repo. |
| **D4** | Zeroize-lint | **+1 mandatory row (`slot_input.rs` → `SecretString`)** + 1 doc row (`convert.rs`); **bump `SECRET_FILE_FLOOR` 35→36 (lint:452)** | `slot_input.rs` newly matches `: SecretString` → SOURCE→declared gate REQUIRES a row. Floor is `>=` so 36 passes 35, but bump to track the true partition. The floor edit is the constant at lint:452 — NOT the stale "(24..=35)" doc comment at lint:370 (a pre-existing bug vs the live `(18..=60)` assert, m-3, out of scope). Run FULL `-p` suite in R0. |
| **D5** | Safety framing | **Defense-in-depth (heap residue), NOT funds/wrong-address** | No observable CLI/wire change (no serde on `SlotInput`). Best-effort caveat per `secret_string.rs`. RED = type-level field assertion + lint rows + no-behavior-change `=-` end-to-end regression. |

---

## 8. Recommended phase structure (single-subagent-per-phase TDD, per CLAUDE.md)

- **Cycle-prep / brainstorm + SPEC** (this doc) → **R0 to GREEN** (full `-p` suite; verify zeroize-1.8.2 traits, the 28-site census, the lint partition count).
- **Plan-doc** — re-grep every call site + line number at write time vs current `origin/master`; pin the exact D1-scope (restore/addresses include vs defer); **R0 to GREEN**.
- **P1 (RED→GREEN)** — extend `SecretString` (`PartialEq`/`Eq`; keep redacting `Debug`); `SlotInput.value: SecretString`; `apply_slot_stdin` wrap; `is_stdin_sentinel` rewrite; `slot()` test helper; the `.value` field-READ sites compile unchanged (Deref, §2.6(a)); **the 4 `SlotInput.value` write/clone edit sites (§2.6(b), I-1):** the 3 `@env:` write-backs (`bundle.rs:2629`, `import_wallet.rs:1396`, `verify_bundle.rs:1883` → `SecretString::new(resolve_env_var_sentinel(…)?)`) + `import_wallet.rs:1233` overlay clone (`.value.clone()`→`.value.to_string()`, or decide `Vec<(u8, SecretString)>`) — T4b/T4c fence these; the ~15 `assert_eq!` stay green. + the mandatory lint row + floor bump (lint:452). **Affected files this phase:** `secret_string.rs`, `slot_input.rs`, `bundle.rs`, `import_wallet.rs`, `verify_bundle.rs`, `tests/lint_zeroize_discipline.rs`.
- **P2 (RED→GREEN)** — convert.rs handler locals (`effective_passphrase`/`bip38`/`primary_value`) → `Zeroizing<String>`/`SecretString` with `as_deref()`/`vec![…]` fixups; restore/addresses locals (if in scope). + doc lint row. **Affected files this phase:** `convert.rs`, `restore.rs`, `addresses.rs` (if in scope).
- **PE (mandatory non-deferrable whole-diff adversarial review)** — full `cargo test -p mnemonic-toolkit` + clippy; bump `v0.67.0`; update BOTH READMEs + `scripts/install.sh` self-pin + `fuzz/Cargo.lock` (release-ritual version-sites, NOT gate-enforced — MEMORY `project_toolkit_release_ritual_version_sites`); tick L22 + flip Cycle-B Site-1 status **in the shipping commit**.
