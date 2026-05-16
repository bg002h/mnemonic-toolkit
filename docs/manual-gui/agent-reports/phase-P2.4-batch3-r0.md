# Phase P2.4 batch 3 (Track M — 20-install) — R0 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1` (mnemonic-toolkit)
**Scope:** §3.2 P2.4 batch 3 — `docs/manual-gui/src/20-install/21-linux.md` (NEW, 150 LOC), `22-macos.md` (NEW, 123 LOC), `23-windows.md` (NEW, 126 LOC), `.cspell.json` (+6 words).

**Verdict:** **ITERATE 3C / 1I / 0N / 1n.**

The three install chapters are well-organised, the platform-defense line citations land within ±2 lines of the actual sites, the wayland-keepalive narrative is byte-faithful to `main.rs:107-131`, the `WGPU_BACKEND` backend selection is verifiable against `egui-wgpu` + `wgpu-types`, the rustup install command matches the CLI manual, and all 6 cspell additions appear in prose. **But three classes of factual error will mislead users:** (1) every chapter pins "Rust 1.77+" while the actual MSRV in `mnemonic-gui/Cargo.toml:5` is `rust-version = "1.85"`, off by 8 minor versions; (2) all three Path C prebuilt-binary blocks invert the asset-name component order (`-${OS}-${ARCH}` in prose vs. `-${ARCH}-${OS}` produced by `.github/workflows/build.yml:20,23,27,30,33`) AND hedge "when published" while assets DO exist on `mnemonic-gui-v0.3.0`'s release (5 of them); (3) three of the four cited FOLLOWUP IDs are fictional — `gui-macos-notarisation`, `gui-macos-code-signing`, `gui-windows-code-signing` are NOT in `mnemonic-gui/FOLLOWUPS.md`. The real IDs are `gui-code-signing-mac-developer-id` (covers both signing+notarisation in one bullet at lines 152-156) and `gui-code-signing-windows` (line 157-161). A fourth FOLLOWUP ID (`gui-os-snapshot-secret-occlusion` at 21-linux:148) is **resolved-and-closed** in v0.2 Phase B.2; the Linux gap is tracked by the suffixed `gui-os-snapshot-secret-occlusion-linux` (FOLLOWUPS.md:162).

No inherited-filter regression. `wrap-long-code.lua`, `mermaid-cache-filter.lua`, `primer-box.lua` all format-gate correctly per batch-1/batch-2 follow-up fixes.

---

## Critical

### C-1 — Rust MSRV pin is off by 8 minor versions (1.77 claimed vs 1.85 actual)

**Where:**
- `src/20-install/21-linux.md:13` — "The `rust-toolchain.toml` in the `mnemonic-gui` repo pins Rust 1.77+."
- `src/20-install/22-macos.md:16` — "Rust 1.77+ required."
- `src/20-install/23-windows.md:18` — "Rust 1.77+ required."

**Why:** `/scratch/code/shibboleth/mnemonic-gui/Cargo.toml:5` declares `rust-version = "1.85"`. A user on Rust 1.78-1.84 hits an opaque `cargo install --locked` failure. There is no `rust-toolchain.toml` in the GUI repo; the 21-linux:12-13 narrative cites a non-existent file.

**Fix:** drop the `rust-toolchain.toml` reference; rephrase to "`mnemonic-gui`'s `Cargo.toml` declares MSRV 1.85" in all three chapters.

### C-2 — Path C prebuilt-binary asset names invert the ARCH/OS component order; stale "(when published)" hedge

**Where:** all three Path C blocks (21-linux:72, 22-macos:54-57, 23-windows:58).

**Why:** `.github/workflows/build.yml:18-33` canonicalises asset suffixes as `${ARCH}-${OS}.{tar.gz|zip}` (`x86_64-linux.tar.gz`, `aarch64-linux.tar.gz`, `x86_64-windows.zip`, `x86_64-macos.tar.gz`, `aarch64-macos.tar.gz`). The published `mnemonic-gui-v0.3.0` release confirms 5 assets in the actual form. Users following the manual look for files that don't exist.

**Fix:** swap to `${ARCH}-${OS}` order; drop "(when published)" hedge; replace with "Every `mnemonic-gui-v*` release attaches per-architecture assets" framing.

### C-3 — Three cited FOLLOWUP IDs do not exist; one cited ID is resolved-not-tracking

**Where + actual evidence in `/scratch/code/shibboleth/mnemonic-gui/FOLLOWUPS.md`:**

1. **22-macos:74** cites `gui-macos-notarisation` and `gui-macos-code-signing`. Neither exists. The actual entry (FOLLOWUPS.md:152-156) is `gui-code-signing-mac-developer-id` — single bullet covering BOTH signing and notarisation.
2. **23-windows:71** cites `gui-windows-code-signing`. Does not exist. The actual entry (FOLLOWUPS.md:157-161) is `gui-code-signing-windows`.
3. **21-linux:148** cites `gui-os-snapshot-secret-occlusion` as tracking the Linux gap. That unsuffixed entry is in the "Resolved in v0.2" section (FOLLOWUPS.md:180-186). The Linux-specific tracker is `gui-os-snapshot-secret-occlusion-linux` (FOLLOWUPS.md:162-170).

**Fix:** correct all 4 citations.

---

## Important

### I-1 — 22-macos overstates "AirPlay screen-mirror" defense scope

`NSWindowSharingType::NSWindowSharingNone` affects per-window capture APIs; whole-display AirPlay mirroring goes through a different path. The 22-macos:90 bullet asserts blanket coverage that the API doesn't provide.

**Fix:** soften to acknowledge the per-window-vs-whole-display distinction.

---

## Nit

### n-1 — `gui-glow-wayland-loop-broken` is resolved (v0.1.1), not actively tracked

`21-linux.md:98-100` primer-block says "tracks the underlying egui issue" — but the FOLLOWUP is in the resolved section (FOLLOWUPS.md:275). Rephrase to "resolved entry".

---

## Final verdict

**ITERATE 3C / 1I / 0N / 1n.**

Three Critical findings are factual install-instruction errors. After C-1/C-2/C-3 fold + I-1 soften + n-1 rephrase, batch 3 is ready for R1. The plan §3.5 per-batch reviewer-LOCK gate (0C/0I) requires the three Critical fixes before promotion to batch 4.
