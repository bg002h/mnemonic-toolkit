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
            echo "mnemonic-toolkit|https://github.com/bg002h/mnemonic-toolkit|mnemonic-toolkit-v0.34.4|no|"
            ;;
        md)
            echo "md-cli|https://github.com/bg002h/descriptor-mnemonic|descriptor-mnemonic-md-cli-v0.6.0|yes|cli-compiler"
            ;;
        ms)
            echo "ms-cli|https://github.com/bg002h/mnemonic-secret|ms-cli-v0.4.0|yes|"
            ;;
        mk)
            echo "mk-cli|https://github.com/bg002h/mnemonic-key|mk-cli-v0.4.1|yes|"
            ;;
        mnemonic-gui)
            echo "mnemonic-gui|https://github.com/bg002h/mnemonic-gui|mnemonic-gui-v0.10.0|no|"
            ;;
        *)
            return 1
            ;;
    esac
}

ALL="mnemonic md ms mk mnemonic-gui"

# ── Defaults ────────────────────────────────────────────────────────────
ONLY=""
EXCLUDE=""
FORCE=""
DRY_RUN=""
LOCKED="--locked"
FROM_GIT=""

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
        else
            if cargo install $LOCKED $feat_args $FORCE "$pkg"; then
                installed_count=$((installed_count + 1))
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
        else
            if cargo install $LOCKED --git "$url" --tag "$tag" $feat_args $FORCE "$pkg"; then
                installed_count=$((installed_count + 1))
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
