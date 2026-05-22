# CONTINUITY — next cycle: import-wallet secret-memory hygiene follow-ons

**Written:** 2026-05-21 (pre-reboot handoff).
**Author context:** Claude (Opus 4.7). Resume from here next session.

## Kickoff command (issue this first when you get back)

```
/cycle-prep bsms-decrypt-record-string-zeroizing import-wallet-plaintext-blob-mlock-pin
```

`/cycle-prep` was lost mid-session and **restored to disk** at `~/.claude/commands/cycle-prep.md` (command) + `~/.claude/skills/cycle-prep/SKILL.md` (skill). Both load at startup, so after the reboot the command above will run the P0 STRICT-GATE recon on both slugs and recommend scope. (If it still errors, I'll reproduce the recon by hand — same procedure.)

## State at handoff (all GREEN — nothing pending on shipped work)

- **toolkit `mnemonic-toolkit-v0.33.3`** SHIPPED — tag `0889af7` pushed, GH release live (not draft); `install-pin-check` + `rust` + `manual` CI all green. Closed FOLLOWUP `import-wallet-blob-zeroizing` (blob → `Zeroizing<Vec<u8>>`).
- Both repos clean + pushed; `master ≡ origin/master`. `mnemonic-gui` correctly stays pinned at toolkit v0.33.2 (v0.33.3 is internal-only — no GUI lockstep).
- Cycle 19 (Electrum BIE1 storage import) fully closed across toolkit (v0.33.0–v0.33.3) + GUI (v0.18.0/v0.18.1).

## Next work — two FOLLOWUP slugs (both in `crates/mnemonic-toolkit/src/cmd/import_wallet.rs`, both secret-memory hygiene, both filed at the v0.33.3 close)

### 1. `bsms-decrypt-record-string-zeroizing`
- **What:** `decrypt_bsms_record` returns a plain `String` (decrypted BSMS record plaintext) — un-zeroized before it is consumed at the BSMS Round-2 reassign (`~:1043`) + the Round-1 decrypt path. Migrate the return type to `Zeroizing<String>`.
- **Sensitivity:** LOW — the BSMS Round-2 plaintext is a watch-only descriptor, not seed/xprv. (That's why it wasn't folded into v0.33.3.)
- **Shape:** function-signature change + minor call-site churn.

### 2. `import-wallet-plaintext-blob-mlock-pin`
- **What:** only the BIE1 decrypt branch calls `mlock::pin_pages_for(&blob)`; the plaintext-import path (`use_encryption:false` Electrum wallet — seed-bearing) + all other formats never pin the blob, so a plaintext seed Vec sits swappable. Pin `&blob` after `read_blob` for ALL formats. **NOTE:** the existing BIE1 pin is itself arm-scoped (dropped at the end of the decrypt arm) — a holistic fix should pin ONCE at the `blob` binding for the whole `run()` scope (covers BIE1 + plaintext + all formats).
- **Sensitivity:** MEDIUM — plaintext seed can be present (orthogonal to the v0.33.3 zeroize-on-drop, which is now done; this is the no-swap leg).

## Expected scope / SemVer (recon will confirm)
- Both touch the SAME file + the SAME theme (secret-memory hygiene around the import-wallet `blob`/decrypt) → likely **ONE cycle**, possibly fold both.
- **SemVer PATCH** (v0.33.3 → v0.33.4) — internal-only, no CLI/wire/GUI/manual/schema surface → **no GUI or manual lockstep** (same as v0.33.3).
- Precedent: the v0.33.3 cycle (`design/PLAN_import_wallet_blob_zeroizing.md` + `design/agent-reports/import-wallet-blob-zeroizing-*`) and `resolved-slot-derived-account-zeroizing-field` (v0.10.1).
- Watch for: `pin_pages_for` is an OWNED guard (`mlock.rs`) whose lifetime = where you bind it; pinning at the `blob` binding (not inside an arm) requires the guard to live for the whole `run()` — bind `let _pin = mlock::pin_pages_for(&blob);` right after the final `blob` value is settled (i.e., AFTER any decrypt reassign) OR re-pin after each reassign. Resolve the pin-lifetime design in the plan-doc.

## Discipline (per CLAUDE.md + project convention)
`/cycle-prep` recon → `design/PLAN_*.md` (SPEC + plan) → **mandatory opus R0 BEFORE impl** → fold to GREEN (R1 if folds change the plan) → implement (regression suite is the gate; type-level zeroize has no runtime assertion, per precedent) → opus end-of-cycle review → split commits → bump + CHANGELOG + install.sh pin → tag + push + GH release → `install-pin-check` CI green → close FOLLOWUP(s) → update memory. Persist all agent reviews to `design/agent-reports/`.
