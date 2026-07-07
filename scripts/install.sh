#!/bin/sh
# m-format constellation installer
#
# Installs the four constellation CLIs (mnemonic / md / ms / mk) and the
# mnemonic-gui overlay. By default, components install from crates.io
# (cargo install <pkg>); the toolkit `mnemonic` binary is the exception
# — it stays on git+tag until its upstream miniscript dependency
# `[patch.crates-io]` is resolved (see `bg002h/mnemonic-toolkit`
# FOLLOWUPS for the blocker). The `--from-git` flag forces git+tag for
# every component, matching the pre-crates.io installer behavior; pins
# below mirror `mnemonic-gui/pinned-upstream.toml`'s `[mnemonic|md|ms|mk].tag`
# fields. When a new toolkit / GUI cycle ships, update the
# `component_info` match arms in lockstep.
#
# All components install into `$CARGO_INSTALL_ROOT` (defaults to
# `~/.cargo/bin`). No system files touched; no sudo required.

set -eu

# ── Component table ─────────────────────────────────────────────────────
# `component_info <short-name>` echoes
# `<cargo-package>|<git-url>|<git-tag>|<cratesio>|<features>` where
# `<cratesio>` is `yes` (published on crates.io; default install path) or
# `no` (must use git+tag), and `<features>` is a comma-separated list of
# Cargo features to enable (empty for default). Short names match the
# installed binary names; the cargo-package is the workspace member name
# (last positional to `cargo install`), distinct from the bin name when
# they differ.
component_info() {
    case "$1" in
        mnemonic)
            echo "mnemonic-toolkit|https://github.com/bg002h/mnemonic-toolkit|mnemonic-toolkit-v0.80.0|no|"
            ;;
        md)
            echo "md-cli|https://github.com/bg002h/descriptor-mnemonic|descriptor-mnemonic-md-cli-v0.11.2|yes|cli-compiler"
            ;;
        ms)
            echo "ms-cli|https://github.com/bg002h/mnemonic-secret|ms-cli-v0.13.2|yes|"
            ;;
        mk)
            echo "mk-cli|https://github.com/bg002h/mnemonic-key|mk-cli-v0.12.0|yes|"
            ;;
        mnemonic-gui)
            echo "mnemonic-gui|https://github.com/bg002h/mnemonic-gui|mnemonic-gui-v0.51.0|no|"
            ;;
        *)
            return 1
            ;;
    esac
}

# Minimum rustc minor for the mnemonic-gui overlay (its --locked deps'
# MSRV; icu_*@2.2.0 / idna_adapter@1.2.2 / image@0.25.10 require
# rustc >= 1.88). The 4 CLIs build on the lower toolkit MSRV
# (rustc >= 1.85). Bump this one line when the GUI's dependency MSRV
# rises. See README.md:33-36 and design/FOLLOWUPS.md
# `install-sh-gui-sibling-pin-staleness-ungated`. Stored as the MINOR
# integer (floor 1.88) so the guard compares integers, never dotted
# strings.
GUI_MIN_RUSTC_MINOR=88

ALL="mnemonic md ms mk mnemonic-gui"

# ── Defaults ────────────────────────────────────────────────────────────
ONLY=""
EXCLUDE=""
FORCE=""
DRY_RUN=""
LOCKED="--locked"
FROM_GIT=""
# v0.73.0 man-page install: after a successful `cargo install`, each CLI
# self-emits its roff man pages (`<bin> gen-man --out`) into the XDG user
# manpath. No sudo / no system files (preserves the install.sh invariant).
NO_MAN=""
MAN_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/man/man1"

# ── Help ────────────────────────────────────────────────────────────────
usage() {
    cat <<EOF
m-format constellation installer

USAGE:
    install.sh [OPTIONS]

INSTALLS (default — all 5):
    mnemonic       CLI: BIP-39 -> 3-card engraving bundle (mnemonic-toolkit)
    md             CLI: descriptor / wallet-policy (descriptor-mnemonic)
    ms             CLI: ms1 BIP-39 entropy codec (mnemonic-secret)
    mk             CLI: mk1 xpub codec (mnemonic-key)
    mnemonic-gui   GUI: cross-platform overlay for the 4 CLIs

SOURCE (default behavior):
    Components on crates.io install via 'cargo install <pkg>' (latest
    published version). The 'mnemonic' (mnemonic-toolkit) binary
    currently stays on git+tag — it's not on crates.io yet (blocked by
    rust-miniscript [patch.crates-io] in the toolkit workspace).
    Override with --from-git below.

OPTIONS:
    --only LIST       Install only the comma-separated components
                      (e.g., --only mnemonic,mnemonic-gui)
    --exclude LIST    Skip the comma-separated components
                      (e.g., --exclude mnemonic-gui)
    --no-gui          Alias for --exclude mnemonic-gui
    --cli-only        Alias for --exclude mnemonic-gui
    --from-git        Force git+tag installs for ALL selected
                      components (pinned tags match
                      mnemonic-gui/pinned-upstream.toml). Slower
                      (compiles from source) but reproducible.
    --force           Re-install even if the same version is already
                      installed (cargo install --force)
    --no-locked       Do NOT pass --locked to cargo install
                      (default: --locked, for reproducibility)
    --no-man          Do NOT emit/install man pages after install
                      (default: each installed CLI self-emits its man
                      pages into the XDG user manpath, no sudo)
    --man-dir DIR     Directory to write man pages into
                      (default: \${XDG_DATA_HOME:-\$HOME/.local/share}/man/man1)
    --dry-run         Print the cargo install commands without executing
    --list            Print the component table + sources and exit
    -h, --help        Show this help and exit

EXAMPLES:
    install.sh                            # install all 5 (mixed source)
    install.sh --from-git                 # install all 5 from git+tag
    install.sh --no-gui                   # install 4 CLIs only
    install.sh --only mnemonic-gui        # install GUI only
    install.sh --exclude md,ms,mk         # install mnemonic + GUI
    install.sh --force                    # reinstall all 5
    install.sh --dry-run --only mk        # see what would run

REQUIREMENTS:
    - cargo (Rust toolchain; rustup recommended: https://rustup.rs/)
    - GUI only: rustc >= 1.88 (the mnemonic-gui overlay's dependency
      MSRV; the 4 CLIs build on rustc >= 1.85). On an older toolchain
      the installer auto-skips the GUI with a warning; pass --no-gui to
      skip it explicitly.
    - git (cargo install --git uses git under the hood; --from-git only)
    - C toolchain (some transitive deps build C code: cc, pkg-config)
    - Linux only: a few system libs for the GUI's wgpu/egui graphics
      stack; see the mnemonic-gui README for the distro-specific list.

The binaries install into \$CARGO_INSTALL_ROOT (default: ~/.cargo/bin).
If \$HOME/.cargo/bin is not on your PATH yet, add it to your shell rc:
    fish:  fish_add_path \$HOME/.cargo/bin
    bash:  export PATH="\$HOME/.cargo/bin:\$PATH"
EOF
}

# ── Argument parsing ────────────────────────────────────────────────────
while [ $# -gt 0 ]; do
    case "$1" in
        --only)
            shift; [ $# -gt 0 ] || { echo "--only requires an argument" >&2; exit 2; }
            ONLY="$1"; shift ;;
        --only=*)
            ONLY="${1#*=}"; shift ;;
        --exclude)
            shift; [ $# -gt 0 ] || { echo "--exclude requires an argument" >&2; exit 2; }
            EXCLUDE="$1"; shift ;;
        --exclude=*)
            EXCLUDE="${1#*=}"; shift ;;
        --no-gui|--cli-only)
            EXCLUDE="${EXCLUDE}${EXCLUDE:+,}mnemonic-gui"; shift ;;
        --no-man)
            NO_MAN="1"; shift ;;
        --man-dir)
            shift; [ $# -gt 0 ] || { echo "--man-dir requires an argument" >&2; exit 2; }
            MAN_DIR="$1"; shift ;;
        --man-dir=*)
            MAN_DIR="${1#*=}"; shift ;;
        --from-git|--from-source)
            FROM_GIT="1"; shift ;;
        --force)
            FORCE="--force"; shift ;;
        --no-locked)
            LOCKED=""; shift ;;
        --dry-run)
            DRY_RUN="1"; shift ;;
        --list)
            printf '%-15s %-20s %-12s %-14s %s\n' "COMPONENT" "CARGO_PACKAGE" "DEFAULT" "FEATURES" "GIT_TAG"
            printf '%-15s %-20s %-12s %-14s %s\n' "---------" "-------------" "-------" "--------" "-------"
            for n in $ALL; do
                info=$(component_info "$n") || continue
                pkg=$(echo "$info" | cut -d'|' -f1)
                tag=$(echo "$info" | cut -d'|' -f3)
                cratesio=$(echo "$info" | cut -d'|' -f4)
                features=$(echo "$info" | cut -d'|' -f5)
                if [ "$cratesio" = "yes" ]; then
                    source="crates.io"
                else
                    source="git (only)"
                fi
                printf '%-15s %-20s %-12s %-14s %s\n' "$n" "$pkg" "$source" "${features:-(none)}" "$tag"
            done
            exit 0 ;;
        -h|--help)
            usage; exit 0 ;;
        *)
            echo "unknown option: $1" >&2
            echo "try: install.sh --help" >&2
            exit 2 ;;
    esac
done

# ── Validate cargo ──────────────────────────────────────────────────────
if ! command -v cargo >/dev/null 2>&1; then
    echo "error: \`cargo\` not found on PATH." >&2
    echo "       install the Rust toolchain first: https://rustup.rs/" >&2
    exit 1
fi

# ── Reject overlap of --only and --exclude ──────────────────────────────
if [ -n "$ONLY" ] && [ -n "$EXCLUDE" ]; then
    echo "error: --only and --exclude are mutually exclusive." >&2
    exit 2
fi

# ── selected: returns 0 if $1 should be installed ───────────────────────
selected() {
    name="$1"
    if [ -n "$ONLY" ]; then
        case ",$ONLY," in
            *",$name,"*) return 0 ;;
            *)           return 1 ;;
        esac
    fi
    if [ -n "$EXCLUDE" ]; then
        case ",$EXCLUDE," in
            *",$name,"*) return 1 ;;
        esac
    fi
    return 0
}

# ── Validate --only / --exclude tokens ──────────────────────────────────
for_each_token() {
    list="$1"
    callback="$2"
    saved_ifs="$IFS"
    IFS=','
    for tok in $list; do
        IFS="$saved_ifs"
        "$callback" "$tok" || return 1
        IFS=','
    done
    IFS="$saved_ifs"
}

validate_token() {
    tok="$1"
    if ! component_info "$tok" >/dev/null 2>&1; then
        echo "error: unknown component '$tok'." >&2
        echo "       valid: $ALL" >&2
        return 1
    fi
}

[ -n "$ONLY" ]    && for_each_token "$ONLY" validate_token
[ -n "$EXCLUDE" ] && for_each_token "$EXCLUDE" validate_token

# ── GUI rustc-MSRV guard ────────────────────────────────────────────────
# The mnemonic-gui overlay needs a newer rustc than the 4 CLIs (its
# --locked deps' MSRV). On an older toolchain, skip the GUI WITH A CLEAR
# WARNING rather than letting `cargo install` raw-exit-101 mid-loop — the
# run still exits 0 with the 4 CLIs installed (matches README.md's
# `--no-gui or upgrade rustc` contract). On any rustc-parse failure we
# FALL THROUGH (attempt the install) — never block a capable user.
# `--dry-run` is exempt so the full 5-component plan prints unchanged.
if selected mnemonic-gui && [ -z "$DRY_RUN" ]; then
    rustc_ver=$(rustc --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -n1)
    rustc_major=$(printf '%s' "$rustc_ver" | cut -d. -f1)
    rustc_minor=$(printf '%s' "$rustc_ver" | cut -d. -f2)
    if [ -n "$rustc_major" ] && [ -n "$rustc_minor" ] && [ "$rustc_major" = "1" ] \
       && [ "$rustc_minor" -lt "$GUI_MIN_RUSTC_MINOR" ] 2>/dev/null; then
        echo "warning: mnemonic-gui needs rustc >= 1.$GUI_MIN_RUSTC_MINOR;" >&2
        echo "         your rustc is $rustc_ver — skipping the GUI this run." >&2
        echo "         The 4 CLIs install normally. Upgrade rustc and re-run" >&2
        echo "         to add the GUI (or this is expected if you only want the CLIs)." >&2
        # Drop mnemonic-gui from the selection on BOTH axes: appending to
        # $EXCLUDE covers the default/`--exclude` set; `selected()` consults
        # $ONLY first, so an `--only mnemonic-gui[,…]` run also needs the
        # token removed from $ONLY (token-exact via for_each_token rebuild,
        # not an unanchored sed — avoids clobbering a future mnemonic-gui-*
        # token). If dropping the GUI empties a previously-set $ONLY (the
        # user asked for ONLY the GUI), substitute a never-matching sentinel
        # so `selected()` installs nothing — an empty $ONLY would otherwise
        # flip to "install all (minus EXCLUDE)".
        EXCLUDE="${EXCLUDE:+$EXCLUDE,}mnemonic-gui"
        if [ -n "$ONLY" ]; then
            NEW_ONLY=""
            drop_gui_token() {
                [ "$1" = "mnemonic-gui" ] && return 0
                NEW_ONLY="${NEW_ONLY:+$NEW_ONLY,}$1"
            }
            for_each_token "$ONLY" drop_gui_token
            [ -z "$NEW_ONLY" ] && NEW_ONLY="__none__"
            ONLY="$NEW_ONLY"
        fi
    fi
fi

# ── Man-page post-install hook ──────────────────────────────────────────
# After a SUCCESSFUL `cargo install`, the just-installed CLI self-emits its
# roff man pages into $MAN_DIR via `<bin> gen-man --out`. Excludes the GUI
# (it has no CLI man surface). Short-circuits under --no-man.
#
# PRECONDITION: only sibling builds that carry `gen-man` emit pages. The
# md/ms/mk siblings default to crates.io-latest (not the pinned git tag); a
# freshly-published toolkit may run against a sibling whose crates.io-latest
# still lacks `gen-man` during the rollout window — that is tolerated, NOT
# required. The invocation is `||`-guarded so a non-zero `gen-man` (missing
# subcommand, read-only $MAN_DIR, disk full) is NON-FATAL under `set -eu` and
# never aborts an otherwise-working install.
install_man_pages() {
    man_name="$1"
    [ -n "$NO_MAN" ] && return 0
    case "$man_name" in
        mnemonic|md|ms|mk) ;;          # CLIs only — exclude the GUI
        *) return 0 ;;
    esac
    bin="${CARGO_INSTALL_ROOT:-$HOME/.cargo/bin}/$man_name"
    if [ -n "$DRY_RUN" ]; then
        echo "  [dry-run] mkdir -p \"$MAN_DIR\" && \"$bin\" gen-man --out \"$MAN_DIR\""
        return 0
    fi
    mkdir -p "$MAN_DIR" 2>/dev/null || true
    "$bin" gen-man --out "$MAN_DIR" 2>/dev/null \
        || echo "warning: man pages skipped for $man_name (needs a $man_name build with gen-man)" >&2
}

# ── Install loop ────────────────────────────────────────────────────────
installed_count=0
failed_count=0
echo "m-format constellation installer"
echo "install root: ${CARGO_INSTALL_ROOT:-$HOME/.cargo/bin}"
if [ -n "$FROM_GIT" ]; then
    echo "source: git+tag (--from-git)"
else
    echo "source: crates.io (default; mnemonic-toolkit stays on git+tag)"
fi
echo

for name in $ALL; do
    if ! selected "$name"; then
        printf 'skip     %s\n' "$name"
        continue
    fi
    info=$(component_info "$name")
    pkg=$(echo "$info" | cut -d'|' -f1)
    url=$(echo "$info" | cut -d'|' -f2)
    tag=$(echo "$info" | cut -d'|' -f3)
    cratesio=$(echo "$info" | cut -d'|' -f4)
    features=$(echo "$info" | cut -d'|' -f5)
    if [ -n "$features" ]; then
        feat_args="--features $features"
    else
        feat_args=""
    fi

    # shellcheck disable=SC2086
    # $LOCKED / $FORCE / $feat_args intentionally unquoted: each is
    # either empty or a sequence of literal CLI flag tokens with no
    # embedded whitespace. Quoting would pass an empty positional which
    # `cargo install` rejects.
    if [ "$cratesio" = "yes" ] && [ -z "$FROM_GIT" ]; then
        printf 'install  %s (crates.io: %s)\n' "$name" "$pkg"
        if [ -n "$DRY_RUN" ]; then
            echo "  [dry-run] cargo install $LOCKED $feat_args $FORCE $pkg"
            installed_count=$((installed_count + 1))
            install_man_pages "$name"
        else
            if cargo install $LOCKED $feat_args $FORCE "$pkg"; then
                installed_count=$((installed_count + 1))
                install_man_pages "$name"
            else
                echo "  FAILED" >&2
                failed_count=$((failed_count + 1))
            fi
        fi
    else
        printf 'install  %s (git: %s)\n' "$name" "$tag"
        if [ -n "$DRY_RUN" ]; then
            echo "  [dry-run] cargo install $LOCKED --git $url --tag $tag $feat_args $FORCE $pkg"
            installed_count=$((installed_count + 1))
            install_man_pages "$name"
        else
            if cargo install $LOCKED --git "$url" --tag "$tag" $feat_args $FORCE "$pkg"; then
                installed_count=$((installed_count + 1))
                install_man_pages "$name"
            else
                echo "  FAILED" >&2
                failed_count=$((failed_count + 1))
            fi
        fi
    fi
done

echo
if [ "$failed_count" -gt 0 ]; then
    echo "$installed_count installed, $failed_count failed." >&2
    exit 1
fi
echo "$installed_count installed."
echo
echo "verify:"
echo "    mnemonic --version       md --version"
echo "    ms --version             mk --version"
echo "    mnemonic-gui --version"
if [ -z "$NO_MAN" ] && [ "$installed_count" -gt 0 ]; then
    echo
    # Cross-platform man hint — printed only when at least one CLI was actually
    # installed (a run where everything was --exclude'd installs no binaries and
    # emits no man pages, so the hint would be misleading). man-db on many Linux
    # pre-seeds ~/.local/share/man so `man <cli>` resolves immediately; but
    # older man-db builds, distros that strip the XDG default, and macOS/BSD
    # man do NOT auto-read it. The `-M` fallback is always correct.
    echo "man pages installed to $MAN_DIR;"
    echo 'if "man <cli>" does not find them, run: man -M "'"$MAN_DIR"'" <cli>'
fi
