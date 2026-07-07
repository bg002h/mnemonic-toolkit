# SPEC — concrete-nonranged-xpub-implied-wildcard

**Reject a concrete non-ranged xpub (a `[fp/path]xpub` with NO `/…` derivation suffix) at the substitution
layer, instead of silently ranging it to `xpub/*` — closing a `verify-bundle` false-pass against a materially
different wallet.**

- **Author:** (this session) — single-author design per CLAUDE.md phase-2 convention.
- **Source SHAs (grep-verified):** toolkit `e092f679` (origin/master); md-codec `ef1f3e71` (origin/**main** — sibling default branch is `main`, NOT `master`).
- **FOLLOWUP slug:** `concrete-nonranged-xpub-implied-wildcard` (`design/FOLLOWUPS.md`; surfaced 2026-07-06, Cycle A design decision D1).
- **Recon:** `cycle-prep-recon-concrete-nonranged-xpub-implied-wildcard.md` (2026-07-07, SHA `e092f679`) — verdict CLEAN, funds claim EMPIRICALLY CONFIRMED (6-step repro).
- **Target release:** `mnemonic-toolkit-v0.79.0` (MINOR). md/ms/mk codecs **NO-BUMP**. No GUI/`schema_mirror` impact.
- **Status:** DRAFT — pending opus-architect R0 loop to 0C/0I before any implementation (CLAUDE.md Conventions bullet 1).

---

## §1 — Problem (empirically confirmed, funds)

A concrete non-ranged descriptor `wpkh([fp/84h/0h/0h]xpub…)` — with **nothing after the xpub** (no `/*`, no
`/<a;b>/*`, no derivation at all) — is a valid BIP-380 single-fixed-key descriptor. The toolkit currently
**silently ranges it**: `concrete_keys_to_placeholders` collapses `[fp/path]xpub` → `@N[fp/path]`, and md-codec
encodes every use-site with a wildcard (`wildcard_for`, `to_miniscript.rs:133`, has only `Hardened`/`Unhardened`
arms — md-codec's `UseSitePath` (`use_site_path.rs:49-54`) has NO "no-wildcard" field; this is a deliberate,
already-ruled wire-format choice, `SPEC_cycleA_descriptor_use_site_collapse.md:226`). So the engraved card
carries `wpkh(@0/*)` — a **ranged, multi-address wallet** — for an input that named a single fixed key.

**The funds bug (6-step repro, recon §Funds claim, against the v0.78.0 binary):** `bundle --descriptor
"wpkh([fp]xpub)"` → exit 0 (silently accepted) → card decodes to `wpkh(@0/*)` → `restore` reconstructs a
ranged wallet (3 receive addrs at 0/1/2) → **`verify-bundle --descriptor "wpkh([fp]xpub)" --md1 … --mk1 …`
returns `result: ok` (exit 0)** — a genuine false-pass: verify-bundle re-lexes the original no-wildcard
descriptor through the same collapsing path and blesses a card encoding a materially different wallet. Same
class as Cycle A's C1 false-pass.

## §2 — Why it must be fixed at the substitution layer (not the lexer)

Once `concrete_keys_to_placeholders` emits `@N[fp/path]` text, a **real substituted xpub with no wildcard** and
a **hand-typed bare `@N` keyless template** (the canonical `bundle --md1-form=template` form, v0.60.0) are
BYTE-IDENTICAL to `lex_placeholders` — both build the same `UseSitePath { multipath: None, wildcard_hardened:
false }` (`make_use_site_path`, `parse_descriptor.rs:325-338`). A bare `@N` MUST lex-pass (test
`lex_residue_floor_accepts_bare_at_n_d1_deferred`, `parse_descriptor.rs:1905-1914`, names this exact slug). So
the lexer CANNOT distinguish them. **The provenance signal — "this `@N[fp/path]` came from a real concrete xpub,
not hand-typed" — is known ONLY inside `concrete_keys_to_placeholders`, at the moment of substitution.** That is
where the reject must live. (Recon §Cross-cutting #1: the precise lost signal is *real-key-vs-hand-typed
provenance*, not "wildcard text" — the `/*` text itself survives; it's the origin that's lost.)

## §3 — Scope decision: REJECT (fail-closed), not silently-range

A concrete non-ranged key is **un-representable in md1** (UseSitePath is always wildcarded; §2). Per the Cycle A
precedent (fail-closed reject of un-representable use-site forms), the fix **rejects** it with a funds-framed
message rather than silently encoding a different wallet. Handling it faithfully (a fixed non-ranged key) would
require an md-codec `UseSitePath` "no-wildcard" variant — **explicitly OUT of scope** (deliberate prior ruling,
`SPEC_cycleA:226`; recon §3 confirmed no md-codec/md-cli code path independently builds `UseSitePath` from a
live `Wildcard`, so the entire encode-side surface is toolkit-owned — toolkit-only fix).

## §4 — Current-source anchors (grep-verified @ `e092f679` / md-codec `ef1f3e71`)

| Symbol / site | Location | Role |
|---|---|---|
| `concrete_keys_to_placeholders` | `src/wallet_import/pipeline.rs:330-400` | THE fix site — the per-key `captures_iter` loop; `m = cap.get(0)` @341, `last_end = m.end()` @383; returns `ToolkitError::ImportWalletParse` ("import-wallet: bsms: parse error: …") |
| `descriptor_concrete_to_resolved_slots` | `src/wallet_import/pipeline.rs:406-418` | the `bundle`/`verify-bundle --descriptor` concrete fork; calls `concrete_keys_to_placeholders` @411 and REMAPS its error prefix → `ToolkitError::DescriptorParse` @412-415 (exit 2) |
| `key_regex` | `src/wallet_import/pipeline.rs:38-42` | matches `[fp/path]xpub` span; `m.end()` lands right after the xpub base58 (does NOT consume any trailing `/…`) |
| `make_use_site_path` | `src/parse_descriptor.rs:325-338` | builds byte-identical `UseSitePath` for `@N` vs `@N/*` — why the lexer can't distinguish |
| bare-`@N` accept test (KEEP passing) | `src/parse_descriptor.rs:1905-1914` (`lex_residue_floor_accepts_bare_at_n_d1_deferred`) | pins that a hand-typed bare `@N` template still lex-passes — the fix must NOT break it |
| `wildcard_for` / `UseSitePath` | md-codec `to_miniscript.rs:133`, `use_site_path.rs:49-54` | root cause (no "no-wildcard" representation) — NOT changed (out of scope §3) |
| CHANGELOG deferred-residual line (REPLACE) | `CHANGELOG.md:48` (v0.76.0 entry) | the v0.76.0 "Deferred residual" note naming this slug → replace with a fixed-in-0.79.0 note |

## §5 — The fix (mechanism)

In `concrete_keys_to_placeholders`'s per-key loop, immediately after `let m = cap.get(0)…` (and before emitting
the `@N` substitution), inspect the ORIGINAL descriptor text immediately following the matched concrete key:

```
let tail = &descriptor[m.end()..];
// A concrete key is followed by EITHER a `/` derivation suffix (ranged: `/*`,
// `/<a;b>/*`, or a fixed step `/0/*` which the Cycle-A residue floor rejects
// downstream) OR a use-site terminator (no derivation at all → the bug).
if !tail.starts_with('/') {
    return Err(<funds-framed reject for key @idx>);
}
```

- **Precision:** the check fires IFF the concrete key is NOT immediately followed by `/`. In a well-formed
  descriptor a key expression is followed by either `/…derivation` or a terminator (`)`, `,`, `}`, whitespace,
  `#`, EOF); "no `/`" ≡ "no derivation" ≡ the non-ranged bug. It fires ONLY for a **concrete-key
  (`key_regex`) match** — a hand-typed bare `@N` is not matched by `key_regex` and never reaches this check
  (§2), so the canonical template form is unaffected. (Recon §Cross-cutting #6: the sibling case `xpub/0` — a
  fixed step, no wildcard — is ALREADY caught by the Cycle-A residue floor in `lex_placeholders`; this fix
  closes ONLY the remaining "immediate terminator / no `/` at all" gap.)
- **Error:** reuse `ToolkitError::ImportWalletParse` with the function's existing `"import-wallet: bsms: parse
  error: …"` prefix so the `descriptor_concrete_to_resolved_slots` caller (@411) remaps it to `DescriptorParse`
  (exit 2) uniformly for `bundle`/`verify-bundle`, matching the Cycle-A floor's exit code. Message (funds-
  framed): name the offending `@N`, state that a concrete xpub with no derivation suffix cannot be represented
  in md1 (which always encodes a ranged use-site), and point at the remedy — append `/*` (ranged) or
  `/<0;1>/*` (receive/change multipath) to intend a ranged wallet. **R0 to decide** whether a dedicated
  alphabetically-ordered `ToolkitError` variant is warranted vs. reusing `ImportWalletParse`+remap; default =
  reuse (matches Cycle-A, no new variant).
- **Single choke point:** because `descriptor_concrete_to_resolved_slots` (verify-bundle/bundle concrete fork)
  shares `concrete_keys_to_placeholders` (@411), this one check closes BOTH the encode silent-accept AND the
  verify-bundle false-pass.

## §6 — Test / oracle matrix (TDD-first; the funds anchor is the closed false-pass)

All against the toolkit CLI. Bold = funds oracle.
1. **REJECT `bundle --descriptor "wpkh([fp]xpub)"`** (concrete, no wildcard) → exit 2, message names `@0` +
   the `/*`/`/<0;1>/*` remedy. (Was: silent exit 0 → ranged card.)
2. **REJECT the verify-bundle false-pass (the C1-class anchor):** replay the recon's 6-step repro —
   `verify-bundle --descriptor "wpkh([fp]xpub)" --md1 <ranged card> --mk1 <…>` now REJECTS (exit 2) at
   re-parse, BEFORE card comparison, instead of `result: ok`.
3. **ACCEPT (regression) `wpkh([fp]xpub/*)`** — ranged single-path still bundles/verifies as before.
4. **ACCEPT (regression) `wpkh([fp]xpub/<0;1>/*)`** — multipath still works.
5. **ACCEPT (regression) hand-typed bare `@N` template** — `bundle --md1-form=template` / a literal `wpkh(@0)`
   template still lex-passes + bundles (the fix must NOT touch this path; §2). Pin against
   `lex_residue_floor_accepts_bare_at_n_d1_deferred`.
6. **REJECT multisig with one non-ranged concrete key** — `wsh(sortedmulti(2,[fpA]xpubA,[fpB]xpubB/*))`
   (first key non-ranged) → exit 2 naming `@0`.
7. **Fixed-step still rejects (unchanged floor)** — `wpkh([fp]xpub/0/*)` still rejects via the Cycle-A residue
   floor (confirm the new check doesn't interfere — it sees the `/` and passes through to the floor).
8. **`import-wallet --format descriptor` / `bsms`** carrying a concrete non-ranged key → rejects (same choke
   point; confirm the error surfaces with the right exit).
9. No-op / determinism: existing concrete-descriptor tests stay green.

Full `cargo test -p mnemonic-toolkit` MUST be green per-phase.

## §7 — Lockstep / SemVer / release

- **SemVer MINOR → `v0.79.0`** — a previously-silently-accepted input now hard-rejects (breaking on the
  acceptance surface, not the flag surface); mirrors Cycle A / v0.77.0 / v0.78.0 precedent. Codecs NO-BUMP.
- **Lockstep: none** — no clap flag/subcommand/dropdown change → no manual `40-cli-reference` mirror, no GUI
  `schema_mirror`. (Recon §4 confirmed zero manual/GUI mention of this residual.) If any manual prose or a
  `verify-examples` golden documents a concrete-non-ranged example, re-grep at impl and update; expected none.
- **CHANGELOG:** replace the `CHANGELOG.md:48` "Deferred residual" line's disposition with a `[0.79.0]`
  fixed-in-this-release entry.
- **Release ritual (v0.77.0/v0.78.0 sequence):** Cargo.toml + workspace/fuzz Cargo.lock + both READMEs +
  `scripts/install.sh:32` self-pin (NOT frozen sibling pins) + `.examples-build/gen.sh` version-check +
  embedded strings + regen `Examples.md` + CHANGELOG. Re-vendor N/A (no dep bump). Direct-FF + tag.

## §8 — Risks / R0 focus

1. **The check must fire ONLY for a real concrete xpub, never a hand-typed bare `@N` template** (§2/§6.5) —
   it lives inside the `key_regex` match loop, so structurally it can't see `@N`; R0 confirm.
2. **`starts_with('/')` boundary correctness** — a concrete key is always followed by `/derivation` or a
   terminator; confirm no valid ranged form is mis-rejected (esp. `/<0;1>/*`, `/*`, nested `sh(wpkh(…/*))`)
   and no malformed `/x` slips (downstream floor catches those).
3. **The verify-bundle false-pass is genuinely closed** (§6.2) — reject fires at re-parse before card
   comparison (like Cycle A).
4. **Multisig partial** (§6.6) — one non-ranged key among ranged cosigners still rejects.
5. Error type/exit consistency (ImportWalletParse→DescriptorParse remap, exit 2) across bundle/verify-bundle/
   import-wallet surfaces.

---

*R0 gate: converge to 0C/0I via the opus-architect loop (persisted to `design/agent-reports/`) BEFORE any
implementation, per CLAUDE.md.*
