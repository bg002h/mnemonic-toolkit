#!/bin/sh
# m-format constellation installer
#
# Installs the four constellation CLIs (mnemonic / md / ms / mk) and the
# mnemonic-gui overlay at the pinned-tag set that mnemonic-gui v0.3.0
# expects from sibling repos. Pins below are kept in lockstep with
# `mnemonic-gui/pinned-upstream.toml`'s `[mnemonic|md|ms|mk].tag` fields;
# when a new toolkit / GUI cycle ships, update the `component_info`
# match arms.
#
# All components install via `cargo install --git --tag` into
# `$CARGO_INSTALL_ROOT` (defaults to `~/.cargo/bin`). No system files
# touched; no sudo required.

set -eu

# ── Component table ─────────────────────────────────────────────────────
# `component_info <short-name>` echoes `<git-url>|<tag>|<cargo-package>`
# (tab-/pipe-separated). The short names match the installed binary
# names. The cargo-package is the workspace member name (last positional
# to `cargo install`), distinct from the bin name when they differ.
component_info() {
    case "$1" in
        mnemonic)
            echo "https://github.com/bg002h/mnemonic-toolkit|mnemonic-toolkit-v0.13.0|mnemonic-toolkit"
            ;;
        md)
            echo "https://github.com/bg002h/descriptor-mnemonic|descriptor-mnemonic-md-cli-v0.5.0|md-cli"
            ;;
        ms)
            echo "https://github.com/bg002h/mnemonic-secret|ms-cli-v0.2.1|ms-cli"
            ;;
        mk)
            echo "https://github.com/bg002h/mnemonic-key|mk-cli-v0.3.1|mk-cli"
            ;;
        mnemonic-gui)
            echo "https://github.com/bg002h/mnemonic-gui|mnemonic-gui-v0.3.0|mnemonic-gui"
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

OPTIONS:
    --only LIST       Install only the comma-separated components
                      (e.g., --only mnemonic,mnemonic-gui)
    --exclude LIST    Skip the comma-separated components
                      (e.g., --exclude mnemonic-gui)
    --no-gui          Alias for --exclude mnemonic-gui
    --cli-only        Alias for --exclude mnemonic-gui
    --force           Re-install even if the same tag is already
                      installed (cargo install --force)
    --no-locked       Do NOT pass --locked to cargo install
                      (default: --locked, for reproducibility)
    --dry-run         Print the cargo install commands without executing
    --list            Print the component table + pinned tags and exit
    -h, --help        Show this help and exit

EXAMPLES:
    install.sh                            # install all 5
    install.sh --no-gui                   # install 4 CLIs only
    install.sh --only mnemonic-gui        # install GUI only (assumes
                                          # siblings already on PATH)
    install.sh --exclude md,ms,mk         # install mnemonic + GUI
    install.sh --force                    # reinstall all 5
    install.sh --dry-run --only mk        # see what would run

REQUIREMENTS:
    - cargo (Rust toolchain; rustup recommended: https://rustup.rs/)
    - git (cargo install --git uses git under the hood)
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
        --force)
            FORCE="--force"; shift ;;
        --no-locked)
            LOCKED=""; shift ;;
        --dry-run)
            DRY_RUN="1"; shift ;;
        --list)
            printf '%-15s %-50s %s\n' "COMPONENT" "TAG" "CARGO_PACKAGE"
            printf '%-15s %-50s %s\n' "---------" "---" "-------------"
            for n in $ALL; do
                info=$(component_info "$n") || continue
                tag=$(echo "$info" | cut -d'|' -f2)
                pkg=$(echo "$info" | cut -d'|' -f3)
                printf '%-15s %-50s %s\n' "$n" "$tag" "$pkg"
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
echo

for name in $ALL; do
    if ! selected "$name"; then
        printf 'skip     %s\n' "$name"
        continue
    fi
    info=$(component_info "$name")
    url=$(echo "$info" | cut -d'|' -f1)
    tag=$(echo "$info" | cut -d'|' -f2)
    pkg=$(echo "$info" | cut -d'|' -f3)
    printf 'install  %s (%s)\n' "$name" "$tag"
    # shellcheck disable=SC2086
    # $LOCKED / $FORCE intentionally unquoted: they are either empty or
    # a single literal flag with no whitespace. Quoting would pass an
    # empty positional which `cargo install` rejects.
    if [ -n "$DRY_RUN" ]; then
        echo "  [dry-run] cargo install $LOCKED --git $url --tag $tag $FORCE $pkg"
        installed_count=$((installed_count + 1))
    else
        if cargo install $LOCKED --git "$url" --tag "$tag" $FORCE "$pkg"; then
            installed_count=$((installed_count + 1))
        else
            echo "  FAILED" >&2
            failed_count=$((failed_count + 1))
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
