# Constellation install — 2026-08-31

Installed the 4 CLIs + GUI (via `mnemonic-toolkit/scripts/install.sh --from-git`),
`me` and `mt` from their local repos (`cargo install --path ... --locked`), the
`me-preview` Go sidecar, and man pages for the 4 CLIs. All installs land in
`~/.cargo/bin`; man pages in `~/.local/share/man/man1`. Nothing pushed,
committed, or mutated in any repo.

## Deliverable table (all paths verified absolute, never bare names)

| Component | Installed path | `--version` | Man page |
|---|---|---|---|
| mnemonic | `/home/bcg/.cargo/bin/mnemonic` | `mnemonic 0.97.0` | `man -w mnemonic` → `/home/bcg/.local/share/man/man1/mnemonic.1` |
| md | `/home/bcg/.cargo/bin/md` | `md 0.14.0` | `man -w md` → `/home/bcg/.local/share/man/man1/md.1` |
| ms | `/home/bcg/.cargo/bin/ms` | `ms 0.16.0` | `man -w ms` → `/home/bcg/.local/share/man/man1/ms.1` |
| mk | `/home/bcg/.cargo/bin/mk` | `mk 0.13.0` (corrected — see "Corrections" below) | `man -w mk` → `/home/bcg/.local/share/man/man1/mk.1` (regenerated, `.TH mk 1 "mk 0.13.0"`) |
| mnemonic-gui | `/home/bcg/.cargo/bin/mnemonic-gui` | `mnemonic-gui 0.61.0` (corrected — see "Corrections" below) | none (GUI has no CLI man surface; install.sh excludes it by design) |
| me | `/home/bcg/.cargo/bin/me` | `me 0.7.0` | none — `me-cli` ships no `gen-man`/man subcommand, no `man/` dir |
| mt | `/home/bcg/.cargo/bin/mt` | `mt 0.1.0` | none — `mt-cli` ships no `gen-man`/man subcommand, no `man/` dir |

Sub-command man pages were also generated for mnemonic/md/ms/mk (e.g.
`md-encode.1`, `mk-derive.1`, `mnemonic-restore.1`, `ms-combine.1`, ...), all
timestamped 2026-08-31 06:45–06:46, confirming a fresh regeneration rather than
stale leftovers.

Smoke test:
```
$ /home/bcg/.cargo/bin/md --help | head -3
Mnemonic Descriptor (MD) — engravable BIP 388 wallet policy backups

Usage: md <COMMAND>
```

`me-preview` sidecar (built via `GO=<nix go1.25.10> bash scripts/build-preview.sh`,
matching the exact toolchain `preview/go.mod` and release CI pin):
```
/home/bcg/.cargo/bin/me-preview --version  ->  me-preview 0.7.0
```
Installed at `~/.cargo/bin/me-preview`, alongside `me` — `me` discovers the
sidecar via `std::env::current_exe()`'s directory (`crates/me-cli/src/preview.rs`),
not `$PATH`, so it must live next to the `me` binary. This is a byte-exact
version match against the freshly installed `me 0.7.0`, satisfying the gate in
`preview.rs` that refuses a mismatched sidecar.

## MANPATH

No shell config needed. `manpath` already lists `/home/bcg/.local/share/man`
ahead of the system dirs on this box, and `man -w md` / `ms` / `mk` / `mnemonic`
all resolved immediately after the install-hook wrote the pages.

## The `md` alias trap — confirmed live on this box

`command -v md` (and bare `md` in an interactive shell) resolves to a shell
alias `md='mkdir -p'`, NOT the installed binary — `type md` reports "md is an
alias for mkdir -p". The real binary is at `/home/bcg/.cargo/bin/md` and answers
`md 0.14.0`. Every verification above used the absolute path for exactly this
reason. The alias's defining file was not found in `~/.zshrc`/`~/.bashrc`
(likely sourced from an oh-my-zsh plugin or similar); not chased further since
this is a pre-existing shell config issue, not something this task should edit.

## Version drift — reported, not fixed

The installer's pinned component table is stale for two of the five
`install.sh` components, verified against each sibling repo's actual pushed
git tags:

- **mk**: `install.sh` pins `mk-cli-v0.12.0`. The `mnemonic-key` repo's latest
  tag is `mk-cli-v0.13.0` (pushed to `origin`, confirmed via
  `git ls-remote --tags origin`), with 29 more untagged commits after it on
  local HEAD. The box's *previously* installed `mk` was actually `mk-cli v0.13.0`
  built from the local path — running `install.sh --from-git` per this task's
  instructions **downgraded** it to the pinned `v0.12.0`, confirmed in the
  install log: `Replaced package \`mk-cli v0.13.0 (/scratch/.../mnemonic-key/crates/mk-cli)\`
  with \`mk-cli v0.12.0 (https://github.com/bg002h/mnemonic-key?tag=mk-cli-v0.12.0#b0abc867)\``.
- **mnemonic-gui**: `install.sh` pins `mnemonic-gui-v0.59.0`. The `mnemonic-gui`
  repo's latest tag is `mnemonic-gui-v0.61.0` (also pushed to `origin`,
  confirmed the same way), with `v0.60.0` also released in between and 8 more
  untagged commits after `v0.61.0` on local HEAD. The installed GUI is the
  pinned `v0.59.0`, two releases behind.
- **md, ms, mnemonic**: no drift — each repo's latest pushed tag matches the
  installer's pin exactly (`descriptor-mnemonic-md-cli-v0.14.0`,
  `ms-cli-v0.16.0`, `mnemonic-toolkit-v0.97.0`).

Per this task's instruction, this is reported and not fixed — `install.sh`'s
`component_info` table was not edited.

## GUI MSRV guard — installed, after a re-run from a neutral directory

The first `install.sh --from-git` run (from inside `mnemonic-toolkit/`) skipped
the GUI: that repo pins `rust-toolchain.toml` to channel `1.85.0` (the 4 CLIs'
MSRV), and rustup's directory-based override made `rustc --version` report
`1.85.0` inside that checkout — below the GUI's `>= 1.88` guard, so `install.sh`
correctly warned and excluded it (`ALL="mnemonic md ms mk mnemonic-gui"` minus
GUI). That is not "this box's toolchain refuses" in the sense the task meant,
though: the box's default toolchain (active outside any MSRV-pinned repo) is
`rustc 1.97.0-nightly`, and `1.88.0` is separately installed via rustup. Re-running
`install.sh --only mnemonic-gui --from-git` from `$HOME` (no toolchain file in
scope) picked up `rustc 1.97.0-nightly` and installed the GUI cleanly at the
pinned `v0.59.0`. So the GUI was **not** skipped in the final state — flagging
this only because the instruction anticipated a skip-with-reason path that
didn't end up applying, and the reason for the first attempt's skip is worth
recording (a repo-local `rust-toolchain.toml`, not a genuine MSRV gap on the
box).

## Corrections — mk and mnemonic-gui re-installed at their actual latest tags

The user asked for the *latest* versions, and the "Version drift" section above
showed two components had landed behind `install.sh`'s stale pins. Corrected
after the fact, outside `install.sh` (its `component_info` table was **not**
edited, per instruction):

Both tag names were verified against the real GitHub remotes before use, not
trusted from spelling — `git ls-remote --tags origin` inside each local
checkout confirmed:
```
mnemonic-key   refs/tags/mk-cli-v0.13.0            78b6e26c...
mnemonic-gui   refs/tags/mnemonic-gui-v0.61.0      b41da516...
```

**mk**, installed at its actual latest pushed tag:
```
$ cargo install --locked --git https://github.com/bg002h/mnemonic-key --tag mk-cli-v0.13.0 mk-cli
    Replaced package `mk-cli v0.12.0 (...tag=mk-cli-v0.12.0#b0abc867)`
        with `mk-cli v0.13.0 (...tag=mk-cli-v0.13.0#0feaaaa9)` (executable `mk`)
$ /home/bcg/.cargo/bin/mk --version
mk 0.13.0
```
Man page content **is** version-dependent (the `.TH` header line embeds the
version string, and `mk-encode.1` grew from 1961 to 2342 bytes — new content,
not just the header), so it was regenerated:
```
$ /home/bcg/.cargo/bin/mk gen-man --out /home/bcg/.local/share/man/man1   # exit 0
$ grep '^.TH' /home/bcg/.local/share/man/man1/mk.1
.TH mk 1  "mk 0.13.0"
$ man -w mk
/home/bcg/.local/share/man/man1/mk.1
```

**mnemonic-gui**, installed the same way v0.59.0 was — from `$HOME`, so the
`mnemonic-toolkit`-local `rust-toolchain.toml` (pinned to 1.85.0, below the
GUI's 1.88 MSRV) does not shadow the box's real default toolchain:
```
$ cd /home/bcg && rustc --version
rustc 1.97.0-nightly (52b6e2c20 2026-04-27)
$ cargo install --locked --git https://github.com/bg002h/mnemonic-gui --tag mnemonic-gui-v0.61.0 mnemonic-gui
    Replaced package `mnemonic-gui v0.59.0 (...tag=mnemonic-gui-v0.59.0#0390ce20)`
        with `mnemonic-gui v0.61.0 (...tag=mnemonic-gui-v0.61.0#82fc3f81)` (executables `gui-render`, `mnemonic-gui`)
$ /home/bcg/.cargo/bin/mnemonic-gui --version
mnemonic-gui 0.61.0
```
No man page to regenerate — the GUI has no CLI man surface (unchanged from
the original install).

**This confirms `install.sh`'s existing FOLLOWUPS entry.** The mk and
mnemonic-gui pins going stale relative to their siblings' actual latest pushed
tags, unnoticed until an install run, is exactly the failure mode named in
`mnemonic-toolkit/design/FOLLOWUPS.md:517`,
`install-sh-gui-sibling-pin-staleness-ungated` — "install.sh's GUI/sibling
pins silently drift (no CI gate, unlike the toolkit self-pin)". That entry was
read to confirm it exists and matches; it was **not** edited, and `install.sh`
itself was **not** edited — both corrections above were done as direct
`cargo install --git ... --tag ...` invocations outside the installer, per
instruction to report the staleness rather than fix the installer.

## Nothing mutated

No `git add`/`commit`/`push` was run in any repo. `install.sh` and
`design/FOLLOWUPS.md` were both read-only for this whole session — not edited.
This report file itself is untracked in `mnemonic-toolkit` — left for the
user's review, not committed.
