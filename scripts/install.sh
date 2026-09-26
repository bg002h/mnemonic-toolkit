#!/bin/sh
# m-format constellation installer
#
# Installs the four constellation CLIs (mnemonic / md / ms / mk) and the
# mnemonic-gui overlay at the versions pinned in `component_info` below.
#
# DEFAULT: the PREBUILT BINARY attached to each pinned GitHub release,
# checked against that release's SHA256SUMS* file before it is installed.
# A download whose digest does not match, or that no published checksum
# covers, is refused (fail closed). The releases are NOT signed: a checksum
# from the same release proves integrity, not origin (each release's
# VERIFY.txt says the same).
#
# FALLBACK: when a release has no binary for this platform (e.g. FreeBSD,
# or md on a musl-libc x86_64 host), that one component is built from the
# same pinned tag with `cargo install --locked --git <repo> --tag <pin>`,
# and the installer says so on stderr. `--from-source` forces that for
# every component.
#
# crates.io is not used: it lags the pins by several releases (F-676), and
# md-codec / mnemonic-toolkit cannot be published while they depend on
# rust-miniscript master.
#
# THE PIN TABLE IS THE ONE SOURCE OF TRUTH. sibling-pin-check.yml keeps the
# workflows and the manual's install lines in step with it, the Examples
# golden embeds `--list` / `--dry-run` output (regenerate it in the same
# commit), and scripts/install-assets.test.sh checks every platform mapping
# below against the real release assets and their SHA256SUMS files.
#
# Everything installs into <root>/bin, root = --root DIR, else
# $CARGO_INSTALL_ROOT, else ${CARGO_HOME:-~/.cargo} (so the default is
# ~/.cargo/bin, as with cargo). cargo's `install.root` config is not read:
# source builds get --root explicitly, so they land where binary installs do
# and crates_owner() inspects the right records. No system files touched; no sudo.

set -eu

# ── Component table ─────────────────────────────────────────────────────
# `component_info <short-name>` echoes
# `<cargo-package>|<git-url>|<git-tag>|<bin>|<features>`: the package and
# features are used only for a source build; <bin> is the executable name
# inside the release archive and on disk. The version is the tag's suffix
# after the last `-v`. KEEP THE FIVE-FIELD SHAPE: rust.yml and
# sibling-pin-check.yml parse these lines.
component_info() {
    case "$1" in
        mnemonic)
            echo "mnemonic-toolkit|https://github.com/bg002h/mnemonic-toolkit|mnemonic-toolkit-v0.105.1|mnemonic|"
            ;;
        md)
            echo "md-cli|https://github.com/bg002h/descriptor-mnemonic|descriptor-mnemonic-md-cli-v0.20.3|md|cli-compiler"
            ;;
        ms)
            echo "ms-cli|https://github.com/bg002h/mnemonic-secret|ms-cli-v0.20.1|ms|"
            ;;
        mk)
            echo "mk-cli|https://github.com/bg002h/mnemonic-key|mk-cli-v0.13.0|mk|"
            ;;
        mnemonic-gui)
            echo "mnemonic-gui|https://github.com/bg002h/mnemonic-gui|mnemonic-gui-v0.63.0|mnemonic-gui|"
            ;;
        *)
            return 1
            ;;
    esac
}

# ── Release-asset table ─────────────────────────────────────────────────
# `asset_for <short-name> <version> <platform>` echoes the release asset for
# that platform, or returns 1 when the pinned release has none. Read off the
# pinned releases with `gh release view <tag> --repo bg002h/<repo>`; the
# repos name their assets differently, so each is mapped explicitly.
# Platforms: linux-{x86_64,aarch64}-{gnu,musl}, macos-{x86_64,aarch64},
# windows-x86_64. Anything else has no binary.
#   * mnemonic / ms / mk: static musl builds for Linux (they run on glibc
#     and musl hosts alike), macos-{amd64,arm64}, windows-amd64.
#   * md: x86_64 Linux ships only a glibc build (md-<v>-linux-amd64), so a
#     musl x86_64 host builds md from source; aarch64 Linux uses the static
#     musl build.
#   * mnemonic-gui: glibc builds per arch, `v`-prefixed version. The releases
#     also carry static musl GUI builds, but a static binary cannot dlopen the
#     X11/Wayland libraries, so it passes --version and then cannot open a
#     window (measured: "Dynamic loading not supported"). A musl host builds
#     the GUI from source instead, as the pre-F-676 installer did.
asset_for() {
    _n="$1"; _v="$2"; _p="$3"
    case "$_n" in
        mnemonic|ms|mk)
            case "$_p" in
                linux-x86_64-*)  echo "$_n-$_v-x86_64-linux-musl.tar.gz" ;;
                linux-aarch64-*) echo "$_n-$_v-aarch64-linux-musl.tar.gz" ;;
                macos-x86_64)    echo "$_n-$_v-macos-amd64.tar.gz" ;;
                macos-aarch64)   echo "$_n-$_v-macos-arm64.tar.gz" ;;
                windows-x86_64)  echo "$_n-$_v-windows-amd64.zip" ;;
                *) return 1 ;;
            esac ;;
        md)
            case "$_p" in
                linux-x86_64-gnu) echo "md-$_v-linux-amd64.tar.gz" ;;
                linux-aarch64-*)  echo "md-$_v-aarch64-linux-musl.tar.gz" ;;
                macos-x86_64)     echo "md-$_v-macos-amd64.tar.gz" ;;
                macos-aarch64)    echo "md-$_v-macos-arm64.tar.gz" ;;
                windows-x86_64)   echo "md-$_v-windows-amd64.zip" ;;
                *) return 1 ;;
            esac ;;
        mnemonic-gui)
            case "$_p" in
                linux-x86_64-gnu)   echo "mnemonic-gui-v$_v-x86_64-linux.tar.gz" ;;
                linux-aarch64-gnu)  echo "mnemonic-gui-v$_v-aarch64-linux.tar.gz" ;;
                macos-x86_64)       echo "mnemonic-gui-v$_v-x86_64-macos.tar.gz" ;;
                macos-aarch64)      echo "mnemonic-gui-v$_v-aarch64-macos.tar.gz" ;;
                windows-x86_64)     echo "mnemonic-gui-v$_v-x86_64-windows.zip" ;;
                *) return 1 ;;
            esac ;;
        *) return 1 ;;
    esac
}

# `glibc_floor <short-name> <platform>` echoes the newest GLIBC_x.y symbol
# version the pinned glibc build of that component needs, or nothing when the
# asset is static (every musl build). Measured with `readelf -V` on the pinned
# assets; scripts/install-assets.test.sh re-measures every Linux asset and fails
# if a floor here is wrong or a static asset turns dynamic. A glibc host older
# than the floor gets a source build instead of a binary that cannot start.
glibc_floor() {
    case "$1:$2" in
        md:linux-x86_64-gnu)           echo 2.34 ;;
        mnemonic-gui:linux-x86_64-gnu) echo 2.18 ;;
        mnemonic-gui:linux-aarch64-gnu) echo 2.18 ;;
    esac
}

# The checksum files a release may carry, tried in this order; the first one
# that lists the asset by exact name is the one checked against. None
# listing it is a refusal, not a fallback.
SUMS_FILES="SHA256SUMS.x86_64 SHA256SUMS.aarch64 SHA256SUMS.portable SHA256SUMS"

# Minimum rustc minor for BUILDING the mnemonic-gui overlay from source (its
# --locked deps' MSRV; icu_*@2.2.0 / idna_adapter@1.2.2 / image@0.25.10
# require rustc >= 1.88). The 4 CLIs build on the lower toolkit MSRV
# (rustc >= 1.85). A prebuilt GUI needs no rustc at all. Bump this one line
# when the GUI's dependency MSRV rises. See design/FOLLOWUPS.md
# `install-sh-gui-sibling-pin-staleness-ungated`. Stored as the MINOR
# integer so the guard compares integers, never dotted strings.
GUI_MIN_RUSTC_MINOR=88

ALL="mnemonic md ms mk mnemonic-gui"

# ── Defaults ────────────────────────────────────────────────────────────
ONLY=""
EXCLUDE=""
FORCE=""
DRY_RUN=""
LOCKED="--locked"
FROM_SOURCE=""
ROOT_ARG=""
# v0.73.0 man-page install: after a successful install, each CLI self-emits
# its roff man pages (`<bin> gen-man --out`) into the XDG user manpath. No
# sudo / no system files (preserves the install.sh invariant).
NO_MAN=""
MAN_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/man/man1"

# ── Help ────────────────────────────────────────────────────────────────
usage() {
    cat <<EOF
m-format constellation installer

USAGE:
    install.sh [OPTIONS]

INSTALLS (default — all 5, at the versions in --list):
    mnemonic       CLI: BIP-39 -> 3-card engraving bundle (mnemonic-toolkit)
    md             CLI: descriptor / wallet-policy (descriptor-mnemonic)
    ms             CLI: ms1 BIP-39 entropy codec (mnemonic-secret)
    mk             CLI: mk1 xpub codec (mnemonic-key)
    mnemonic-gui   GUI: cross-platform overlay for the 4 CLIs

SOURCE (default behavior):
    The prebuilt binary from each component's pinned GitHub release. Each
    download is checked against the SHA256SUMS file published with that
    release, and refused if the digest does not match or no checksum
    covers it. The releases are not signed: this proves the download is
    the file the release published, not who built it.
    If a release has no binary for this platform, that component is
    built from the same pinned tag with cargo instead (said on stderr).
    The same happens where a binary needs a newer glibc than this host
    has: on Linux the GUI needs glibc >= 2.18 and x86_64 md needs >= 2.34
    (mnemonic, ms, mk, and md on aarch64, are static and run
    on any Linux). A musl host always builds the GUI from source.

OPTIONS:
    --only LIST       Install only the comma-separated components
                      (e.g., --only mnemonic,mnemonic-gui)
    --exclude LIST    Skip the comma-separated components
                      (e.g., --exclude mnemonic-gui)
    --no-gui          Alias for --exclude mnemonic-gui
    --cli-only        Alias for --exclude mnemonic-gui
    --from-source     Build ALL selected components from their pinned
                      git tags with 'cargo install --locked --git
                      <repo> --tag <pin>' instead of downloading
                      binaries. Needs cargo, git and a C toolchain.
    --from-git        Alias for --from-source
    --root DIR        Install into DIR/bin (default:
                      \$CARGO_INSTALL_ROOT/bin, else ~/.cargo/bin).
                      cargo's install.root config is not read.
                      Binaries only: man pages go to --man-dir
                      (turn them off with --no-man).
    --force           Source builds: re-install even if the same version
                      is already installed (cargo install --force).
                      Binary installs always replace the file.
    --no-locked       Source builds: do NOT pass --locked to cargo
    --no-man          Do NOT emit/install man pages after install
                      (default: each installed CLI self-emits its man
                      pages into the XDG user manpath, no sudo)
    --man-dir DIR     Directory to write man pages into
                      (default: \${XDG_DATA_HOME:-\$HOME/.local/share}/man/man1)
    --dry-run         Print what would be downloaded, verified and
                      installed (or built) without doing it
    --list            Print the pin table, with each component's binary
                      for this platform, and exit
    -h, --help        Show this help and exit

ENVIRONMENT:
    MNEMONIC_INSTALL_PLATFORM   Override platform detection, e.g.
                      linux-x86_64-gnu, linux-aarch64-musl, macos-aarch64,
                      windows-x86_64 (anything else means "no binary").
    MNEMONIC_INSTALL_GLIBC      Override the detected glibc version, e.g.
                      2.35 (default: getconf GNU_LIBC_VERSION).

EXAMPLES:
    install.sh                            # install all 5 (release binaries)
    install.sh --no-gui                   # install the 4 CLIs only
    install.sh --only mnemonic-gui        # install GUI only
    install.sh --exclude md,ms,mk         # install mnemonic + GUI
    install.sh --root /opt/m              # install into /opt/m/bin
    install.sh --from-source              # build all 5 from their pinned tags
    install.sh --dry-run --only mk        # see what would run

REQUIREMENTS:
    - curl or wget; tar (unzip on Windows); sha256sum, shasum or openssl
    - Source builds only (--from-source, or a platform with no binary):
      cargo (https://rustup.rs/), git and a C toolchain. The 4 CLIs build
      on rustc >= 1.85; the mnemonic-gui overlay needs rustc >= 1.88, and
      on an older toolchain the installer skips the GUI with a warning.
    - Linux GUI: a few system libs for the wgpu/egui graphics stack; see
      the mnemonic-gui README for the distro-specific list.

If <root>/bin is not on your PATH yet, add it to your shell rc, e.g.:
    fish:  fish_add_path \$HOME/.cargo/bin
    bash:  export PATH="\$HOME/.cargo/bin:\$PATH"
EOF
}

# ── Platform ────────────────────────────────────────────────────────────
detect_platform() {
    if [ -n "${MNEMONIC_INSTALL_PLATFORM:-}" ]; then
        echo "$MNEMONIC_INSTALL_PLATFORM"
        return 0
    fi
    _os=$(uname -s 2>/dev/null || echo unknown)
    _arch=$(uname -m 2>/dev/null || echo unknown)
    case "$_arch" in
        x86_64|amd64) _arch=x86_64 ;;
        aarch64|arm64) _arch=aarch64 ;;
    esac
    case "$_os" in
        Linux)
            # A glibc host answers GNU_LIBC_VERSION even when a musl loader is
            # also installed (Debian's musl package puts one in /lib), so ask
            # glibc first and only then look for musl.
            if getconf GNU_LIBC_VERSION >/dev/null 2>&1; then
                _libc=gnu
            elif ldd --version 2>&1 | grep -qi musl || ls /lib/ld-musl-* >/dev/null 2>&1; then
                _libc=musl
            else
                _libc=gnu
            fi
            echo "linux-$_arch-$_libc" ;;
        Darwin) echo "macos-$_arch" ;;
        MINGW*|MSYS*|CYGWIN*|Windows_NT) echo "windows-$_arch" ;;
        *) echo "$(echo "$_os" | tr '[:upper:]' '[:lower:]')-$_arch" ;;
    esac
}
PLATFORM=$(detect_platform)
case "$PLATFORM" in
    windows-*) EXE=".exe" ;;
    *) EXE="" ;;
esac

field() { echo "$1" | cut -d'|' -f"$2"; }
version_of() { echo "${1##*-v}"; }

# The host's glibc as "major.minor", or empty when unknown (not glibc, or no
# getconf). Only consulted on linux-*-gnu platforms.
HOST_GLIBC="${MNEMONIC_INSTALL_GLIBC:-}"
if [ -z "$HOST_GLIBC" ]; then
    HOST_GLIBC=$(getconf GNU_LIBC_VERSION 2>/dev/null | awk '{print $2}' | grep -oE '^[0-9]+\.[0-9]+' || true)
fi

# version_lt A B: A < B, both "major.minor" integers.
version_lt() {
    _a1=${1%%.*}; _a2=${1#*.}; _b1=${2%%.*}; _b2=${2#*.}
    [ "$_a1" -lt "$_b1" ] || { [ "$_a1" -eq "$_b1" ] && [ "$_a2" -lt "$_b2" ]; }
}

# floor_unmet <name>: 0 when this host's known glibc is older than the
# component's floor for this platform. An unknown glibc is NOT unmet: the
# pre-install run check still refuses a binary that cannot start.
floor_unmet() {
    _f=$(glibc_floor "$1" "$PLATFORM")
    [ -n "$_f" ] && [ -n "$HOST_GLIBC" ] && version_lt "$HOST_GLIBC" "$_f"
}

# plan_for <name> -> echoes the asset name, or nothing for a source build.
plan_for() {
    [ -n "$FROM_SOURCE" ] && return 0
    floor_unmet "$1" && return 0
    _i=$(component_info "$1")
    asset_for "$1" "$(version_of "$(field "$_i" 3)")" "$PLATFORM" || true
}

# source_reason <name> <tag>: why a component builds from source (stderr note).
source_reason() {
    if floor_unmet "$1"; then
        echo "note: the $2 $1 binary needs glibc >= $(glibc_floor "$1" "$PLATFORM"); this host has $HOST_GLIBC," >&2
    else
        echo "note: $2 publishes no $1 binary for platform '$PLATFORM';" >&2
    fi
    echo "      building $1 from source at that tag with cargo instead." >&2
}

# ── Argument parsing ────────────────────────────────────────────────────
LIST=""
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
        --root)
            shift; [ -n "${1:-}" ] || { echo "--root requires a non-empty argument" >&2; exit 2; }
            ROOT_ARG="$1"; shift ;;
        --root=*)
            ROOT_ARG="${1#*=}"
            [ -n "$ROOT_ARG" ] || { echo "--root requires a non-empty argument" >&2; exit 2; }
            shift ;;
        --from-source|--from-git)
            FROM_SOURCE="1"; shift ;;
        --force)
            FORCE="--force"; shift ;;
        --no-locked)
            LOCKED=""; shift ;;
        --dry-run)
            DRY_RUN="1"; shift ;;
        --list)
            LIST="1"; shift ;;
        -h|--help)
            usage; exit 0 ;;
        *)
            echo "unknown option: $1" >&2
            echo "try: install.sh --help" >&2
            exit 2 ;;
    esac
done

if [ -n "$LIST" ]; then
    echo "platform: $PLATFORM"
    printf '%-15s %-20s %-14s %-36s %s\n' "COMPONENT" "CARGO_PACKAGE" "FEATURES" "GIT_TAG" "BINARY"
    printf '%-15s %-20s %-14s %-36s %s\n' "---------" "-------------" "--------" "-------" "------"
    for n in $ALL; do
        info=$(component_info "$n") || continue
        asset=$(plan_for "$n")
        printf '%-15s %-20s %-14s %-36s %s\n' "$n" "$(field "$info" 1)" \
            "$(field "$info" 5 | sed 's/^$/(none)/')" "$(field "$info" 3)" \
            "${asset:-(none: builds from source)}"
    done
    exit 0
fi

if [ -n "$ROOT_ARG" ]; then
    ROOT="$ROOT_ARG"
else
    ROOT="${CARGO_INSTALL_ROOT:-${CARGO_HOME:-$HOME/.cargo}}"
fi
BIN_DIR="$ROOT/bin"

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

# ── Which components build from source, and do we have what they need? ──
NEED_CARGO=""
for n in $ALL; do
    selected "$n" || continue
    [ -z "$(plan_for "$n")" ] && NEED_CARGO="${NEED_CARGO:+$NEED_CARGO }$n"
done
if [ -n "$NEED_CARGO" ] && [ -z "$DRY_RUN" ] && ! command -v cargo >/dev/null 2>&1; then
    echo "error: $NEED_CARGO must be built from source here, and \`cargo\` is not on PATH." >&2
    for n in $NEED_CARGO; do source_reason "$n" "$(field "$(component_info "$n")" 3)" 2>&1 | sed -n 1p | sed 's/^note: /       /' >&2; done
    [ -n "$FROM_SOURCE" ] && echo "       (--from-source was given)" >&2
    echo "       Install the Rust toolchain first (https://rustup.rs/), or install" >&2
    echo "       the rest without them: re-run with --exclude $(echo "$NEED_CARGO" | tr ' ' ',')" >&2
    exit 1
fi

# ── GUI rustc-MSRV guard (source builds of the GUI only) ────────────────
# The mnemonic-gui overlay needs a newer rustc than the 4 CLIs (its --locked
# deps' MSRV). On an older toolchain, skip the GUI WITH A CLEAR WARNING
# rather than letting `cargo install` raw-exit-101 mid-loop — the run still
# exits 0 with the 4 CLIs installed. On any rustc-parse failure we FALL
# THROUGH (attempt the install) — never block a capable user. `--dry-run`
# is exempt so the full plan prints unchanged. A prebuilt GUI needs no rustc.
if selected mnemonic-gui && [ -z "$DRY_RUN" ] && [ -z "$(plan_for mnemonic-gui)" ]; then
    rustc_ver=$(rustc --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -n1)
    rustc_major=$(printf '%s' "$rustc_ver" | cut -d. -f1)
    rustc_minor=$(printf '%s' "$rustc_ver" | cut -d. -f2)
    if [ -n "$rustc_major" ] && [ -n "$rustc_minor" ] && [ "$rustc_major" = "1" ] \
       && [ "$rustc_minor" -lt "$GUI_MIN_RUSTC_MINOR" ] 2>/dev/null; then
        echo "warning: mnemonic-gui needs rustc >= 1.$GUI_MIN_RUSTC_MINOR to build from source;" >&2
        echo "         your rustc is $rustc_ver — skipping the GUI this run." >&2
        echo "         The 4 CLIs install normally. Upgrade rustc and re-run" >&2
        echo "         to add the GUI (or this is expected if you only want the CLIs)." >&2
        # Drop mnemonic-gui from the selection on BOTH axes: appending to
        # $EXCLUDE covers the default/`--exclude` set; `selected()` consults
        # $ONLY first, so an `--only mnemonic-gui[,…]` run also needs the
        # token removed from $ONLY (token-exact via for_each_token rebuild).
        # If dropping the GUI empties a previously-set $ONLY (the user asked
        # for ONLY the GUI), substitute a never-matching sentinel so
        # `selected()` installs nothing — an empty $ONLY would otherwise flip
        # to "install all (minus EXCLUDE)".
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

# ── Download + verify helpers ───────────────────────────────────────────
# fetch <url> <dest>: 0 on success; non-zero (quietly) on any failure, incl.
# a 404. Only https URLs are ever passed in.
fetch() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL --proto '=https' --tlsv1.2 -o "$2" "$1" 2>/dev/null
    elif command -v wget >/dev/null 2>&1; then
        # BusyBox wget (stock Alpine) has no --https-only. The URL is https and
        # every download is checked against the release's checksum, so plain
        # wget fails closed the same way.
        if wget --help 2>&1 | grep -q -- '--https-only'; then
            wget -q --https-only -O "$2" "$1" 2>/dev/null
        else
            wget -q -O "$2" "$1" 2>/dev/null
        fi
    else
        echo "error: neither curl nor wget is on PATH; cannot download $1" >&2
        return 3
    fi
}

# sha256_of <file>: the lowercase hex digest, or non-zero with no tool.
sha256_of() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d' ' -f1
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | cut -d' ' -f1
    elif command -v openssl >/dev/null 2>&1; then
        openssl dgst -sha256 -r "$1" | cut -d' ' -f1
    else
        return 1
    fi
}

TMP_ROOT=""
PART=""
cleanup() {
    if [ -n "$PART" ]; then rm -f "$PART"; fi
    case "$TMP_ROOT" in /?*) rm -rf "$TMP_ROOT" ;; esac
}
# EXIT alone: a trapped INT/TERM would otherwise run cleanup and CONTINUE.
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

# make_tmp_root: create the private work dir, or exit before touching anything.
# `mktemp -d` alone, then a template under ${TMPDIR:-/tmp} (portable to GNU,
# BSD and BusyBox). An empty or relative result is a refusal: every path below
# is built as "$TMP_ROOT/<name>", which with an empty TMP_ROOT would be /<name>.
make_tmp_root() {
    TMP_ROOT=$(mktemp -d 2>/dev/null) || TMP_ROOT=""
    case "$TMP_ROOT" in /?*) [ -d "$TMP_ROOT" ] && return 0 ;; esac
    TMP_ROOT=$(mktemp -d "${TMPDIR:-/tmp}/mnemonic-install.XXXXXX" 2>/dev/null) || TMP_ROOT=""
    case "$TMP_ROOT" in /?*) [ -d "$TMP_ROOT" ] && return 0 ;; esac
    TMP_ROOT=""
    echo "error: cannot create a temporary directory (mktemp -d failed; TMPDIR=${TMPDIR:-unset})." >&2
    echo "       Nothing was installed. Point TMPDIR at a writable directory and re-run." >&2
    return 1
}

# install_binary <name> <bin> <version> <release-base-url> <asset>: download,
# verify against the release's checksum file, extract, run, install. Returns
# non-zero (after saying why on stderr) on ANY failure; never installs an
# unverified file. Runs inside an `if`, where `set -e` is off, so every step
# checks its own status.
install_binary() {
    _name="$1"; _bin="$2"; _ver="$3"; _base="$4"; _asset="$5"
    case "$TMP_ROOT" in
        /?*) [ -d "$TMP_ROOT" ] || { echo "  error: temporary directory $TMP_ROOT is gone" >&2; return 1; } ;;
        *) echo "  error: no temporary directory; refusing to continue" >&2; return 1 ;;
    esac
    _w="$TMP_ROOT/$_name"
    if ! { rm -rf "$_w" && mkdir -p "$_w/x"; }; then
        echo "  error: cannot prepare $_w" >&2
        return 1
    fi

    if ! fetch "$_base/$_asset" "$_w/$_asset"; then
        echo "  error: download failed: $_base/$_asset" >&2
        return 1
    fi

    _want=""; _sums=""
    for _s in $SUMS_FILES; do
        fetch "$_base/$_s" "$_w/$_s" || continue
        # Exact name match on field 2 (`<hex>  <name>` or `<hex> *<name>`).
        # Every digest listed for the name; more than one distinct is refused.
        _want=$(awk -v a="$_asset" '$2 == a || $2 == "*" a { print tolower($1) }' "$_w/$_s" | sort -u)
        if [ -n "$_want" ]; then _sums="$_s"; break; fi
    done
    if [ "$(printf '%s\n' "$_want" | grep -c .)" -gt 1 ]; then
        echo "  error: REFUSED $_asset: $_sums lists it with more than one digest" >&2
        return 1
    fi
    if [ -z "$_want" ]; then
        echo "  error: REFUSED $_asset: no checksum for it in any of" >&2
        echo "         $SUMS_FILES at $_base/" >&2
        return 1
    fi
    if ! _got=$(sha256_of "$_w/$_asset"); then
        echo "  error: no sha256 tool (sha256sum, shasum or openssl) to verify $_asset" >&2
        return 1
    fi
    if [ "$_got" != "$_want" ]; then
        echo "  error: REFUSED $_asset: sha256 mismatch against $_sums" >&2
        echo "         published $_want" >&2
        echo "         download  $_got" >&2
        return 1
    fi
    echo "  verified sha256 $_got ($_sums)"

    case "$_asset" in
        *.zip)
            command -v unzip >/dev/null 2>&1 || { echo "  error: unzip is needed for $_asset" >&2; return 1; }
            unzip -q "$_w/$_asset" -d "$_w/x" || { echo "  error: cannot unpack $_asset" >&2; return 1; } ;;
        *)
            tar -xzf "$_w/$_asset" -C "$_w/x" || { echo "  error: cannot unpack $_asset" >&2; return 1; } ;;
    esac
    _found=$(find "$_w/x" -type f -name "$_bin$EXE")
    if [ -z "$_found" ] || [ "$(printf '%s\n' "$_found" | wc -l | tr -d ' ')" -ne 1 ]; then
        echo "  error: expected exactly one '$_bin$EXE' in $_asset, found: ${_found:-none}" >&2
        return 1
    fi
    # Run it before installing it: a binary that does not start on this host
    # (md's x86_64 Linux build needs glibc >= 2.34) or reports a version other
    # than the pin is refused rather than installed broken. Not done for the
    # Windows GUI build, whose --version output is unverified.
    chmod 755 "$_found"
    case "$PLATFORM:$_bin" in
        windows-*:mnemonic-gui)
            _ran="not run: Windows GUI build" ;;
        *)
            _ran=$("$_found" --version 2>&1) || _ran="(exit $?) $_ran"
            _ran=$(printf '%s\n' "$_ran" | head -n1)
            if [ "$_ran" != "$_bin $_ver" ]; then
                echo "  error: REFUSED $_asset: '$_bin$EXE --version' did not print '$_bin $_ver'" >&2
                echo "         it printed: $_ran" >&2
                echo "         (it does not run on this host, or is not the pinned version)." >&2
                echo "         To build the pinned tag instead, re-run with: --from-source --only $_name" >&2
                return 1
            fi ;;
    esac
    mkdir -p "$BIN_DIR" || { echo "  error: cannot create $BIN_DIR" >&2; return 1; }
    if [ -d "$BIN_DIR/$_bin$EXE" ]; then
        echo "  error: $BIN_DIR/$_bin$EXE is a directory; not replacing it" >&2
        return 1
    fi
    # Copy beside the target, then rename over it, so a failed copy never
    # leaves a truncated binary behind. PART lets the EXIT trap remove the
    # partial copy if the run is interrupted mid-copy.
    PART="$BIN_DIR/.$_bin$EXE.partial.$$"
    if ! { cp "$_found" "$PART" && chmod 755 "$PART" && mv -f "$PART" "$BIN_DIR/$_bin$EXE"; }; then
        rm -f "$PART"; PART=""
        echo "  error: cannot write $BIN_DIR/$_bin$EXE" >&2
        return 1
    fi
    PART=""
    echo "  installed $BIN_DIR/$_bin$EXE ($_ran)"
}

# ── Man-page post-install hook ──────────────────────────────────────────
# After a SUCCESSFUL install, the just-installed CLI self-emits its roff man
# pages into $MAN_DIR via `<bin> gen-man --out`. Excludes the GUI (it has no
# CLI man surface). Short-circuits under --no-man.
#
# The invocation is `||`-guarded so a non-zero `gen-man` (missing
# subcommand, read-only $MAN_DIR, disk full) is NON-FATAL under `set -eu` and
# never aborts an otherwise-working install.
install_man_pages() {
    man_name="$1"
    [ -n "$NO_MAN" ] && return 0
    case "$man_name" in
        mnemonic|md|ms|mk) ;;          # CLIs only — exclude the GUI
        *) return 0 ;;
    esac
    bin="$BIN_DIR/$man_name$EXE"
    if [ -n "$DRY_RUN" ]; then
        echo "  [dry-run] mkdir -p \"$MAN_DIR\" && \"$bin\" gen-man --out \"$MAN_DIR\""
        return 0
    fi
    mkdir -p "$MAN_DIR" 2>/dev/null || true
    "$bin" gen-man --out "$MAN_DIR" 2>/dev/null \
        || echo "warning: man pages skipped for $man_name (needs a $man_name build with gen-man)" >&2
}

# cargo_root <cargo args...>: run cargo with `--root "$ROOT"` (as ONE
# argument) after the subcommand. Always explicit, so cargo installs into, and
# keeps its records in, the same root this script inspects and reports; a
# cargo `install.root` config would otherwise send the build somewhere else.
cargo_root() {
    _sub="$1"; shift
    cargo "$_sub" --root "$ROOT" "$@"
}

# crates_owner <bin>: which cargo package, per cargo's own records in $ROOT,
# owns $BIN_DIR/<bin>. Prints exactly one of:
#   none              no cargo record in $ROOT claims it (neither .crates.toml
#                     nor .crates2.json exists, or .crates.toml parses and no
#                     package lists it, and .crates2.json does not name it)
#   pkg <package-id>  .crates.toml lists it under <package-id>, e.g.
#                     "fake-mk 0.1.0 (path+file:///src/fake-mk)"
#   unknown           the records exist but cannot be read with certainty
#                     (malformed, unreadable, or .crates2.json without
#                     .crates.toml, or the two files disagree)
# .crates.toml is cargo's v1 record: a `[v1]` table of
#   "<name> <version> (<source>)" = ["<bin>"]
# with a package of several bins written as a multi-line array, one
# "<bin>", per line and a closing `]` (as cargo writes it). Any line that is
# not one of those shapes makes the whole file "unknown".
crates_owner() {
    _cb="$1"; _toml="$ROOT/.crates.toml"; _json="$ROOT/.crates2.json"
    if [ ! -e "$_toml" ]; then
        if [ -e "$_json" ]; then echo unknown; else echo none; fi
        return 0
    fi
    [ -r "$_toml" ] || { echo unknown; return 0; }
    _co=$(awk -v b="$_cb" -v be="$_cb$EXE" '
        # item <token>: one array element, e.g. "mk" or "mk", ; claims b?
        function item(it) {
            gsub(/^[ \t]+|[ \t]+$/, "", it)
            sub(/,$/, "", it)
            gsub(/[ \t]+$/, "", it)
            if (it == "") return
            if (it !~ /^"[^"]*"$/) { bad = 1; return }
            it = substr(it, 2, length(it) - 2)
            if (it == b || it == be) {
                if (owner != "" && owner != id) bad = 1
                owner = id
            }
        }
        BEGIN { bad = 0; hdr = 0; inarr = 0; owner = "" }
        inarr {
            line = $0; gsub(/^[ \t]+|[ \t]+$/, "", line)
            if (line == "]") { inarr = 0; next }
            if (line ~ /^"[^"]*",?$/) { item(line); next }
            bad = 1; next
        }
        /^[ \t]*$/ { next }
        /^\[v1\][ \t]*$/ { if (hdr) bad = 1; hdr = 1; next }
        /^"[^"]*" = \[/ {
            if (!hdr) { bad = 1; next }
            # index(), not a regex: mawk 1.3.4 misses /" = \[.*$/ when the
            # line ends at the bracket (a multi-line array header).
            id = substr($0, 2); id = substr(id, 1, index(id, "\" = [") - 1)
            rest = $0; sub(/^"[^"]*" = \[/, "", rest); gsub(/[ \t]+$/, "", rest)
            if (rest == "") { inarr = 1; next }          # multi-line array
            if (rest !~ /\]$/) { bad = 1; next }
            sub(/\]$/, "", rest)
            n = split(rest, items, ",")
            for (i = 1; i <= n; i++) item(items[i])
            next
        }
        { bad = 1 }
        END {
            if (bad || !hdr || inarr) print "unknown"
            else if (owner == "") print "none"
            else print "pkg " owner
        }' "$_toml" 2>/dev/null) || _co=unknown
    [ -n "$_co" ] || _co=unknown
    if [ "$_co" = none ] && [ -e "$_json" ]; then
        # .crates.toml says nobody owns it; .crates2.json must not disagree.
        if [ ! -r "$_json" ] || grep -Eq "\"$_cb(\\.exe)?\"" "$_json" 2>/dev/null; then
            _co=unknown
        fi
    fi
    echo "$_co"
}

# ── Install loop ────────────────────────────────────────────────────────
if [ -z "$DRY_RUN" ]; then
    for n in $ALL; do
        selected "$n" || continue
        if [ -n "$(plan_for "$n")" ]; then make_tmp_root || exit 1; break; fi
    done
fi
installed_count=0
failed_count=0
echo "m-format constellation installer"
echo "install root: $ROOT (binaries in $BIN_DIR)"
echo "platform: $PLATFORM"
if [ -n "$FROM_SOURCE" ]; then
    echo "source: pinned git tags, built with cargo (--from-source)"
else
    echo "source: pinned GitHub release binaries, sha256-verified"
fi
echo

for name in $ALL; do
    if ! selected "$name"; then
        printf 'skip     %s\n' "$name"
        continue
    fi
    info=$(component_info "$name")
    pkg=$(field "$info" 1)
    url=$(field "$info" 2)
    tag=$(field "$info" 3)
    bin=$(field "$info" 4)
    features=$(field "$info" 5)
    asset=$(plan_for "$name")

    if [ -n "$asset" ]; then
        base="$url/releases/download/$tag"
        printf 'install  %s (release %s: %s)\n' "$name" "$tag" "$asset"
        if [ -n "$DRY_RUN" ]; then
            echo "  [dry-run] download $base/$asset"
            echo "  [dry-run] verify its sha256 against the release's SHA256SUMS file; refuse on mismatch or no entry"
            case "$PLATFORM:$bin" in
                windows-*:mnemonic-gui) ;;
                *) echo "  [dry-run] run '$bin$EXE --version'; refuse unless it prints '$bin $(version_of "$tag")'" ;;
            esac
            echo "  [dry-run] install $BIN_DIR/$bin$EXE"
            installed_count=$((installed_count + 1))
            install_man_pages "$name"
        elif install_binary "$name" "$bin" "$(version_of "$tag")" "$base" "$asset"; then
            installed_count=$((installed_count + 1))
            install_man_pages "$name"
        else
            echo "  FAILED" >&2
            failed_count=$((failed_count + 1))
        fi
        continue
    fi

    # ── source build from the pinned tag ──
    if [ -z "$FROM_SOURCE" ]; then
        source_reason "$name" "$tag"
    fi
    # An existing $BIN_DIR/<bin> decides whether cargo needs --force. Ask
    # cargo's own records in $ROOT who owns it (crates_owner), never infer
    # from the package name:
    #   none     no record claims it: the file a binary install of this
    #            script copied in. cargo refuses to overwrite an untracked
    #            file, so pass --force.
    #   this pkg cargo already tracks it as $pkg: no --force; cargo upgrades
    #            it or says it is already installed.
    #   another  a DIFFERENT package owns it: refuse this component and name
    #            the owner. Only a user-supplied --force overrides.
    #   unknown  records unreadable or inconsistent: no --force (fail safe);
    #            cargo itself then refuses if the file is in the way.
    src_force="$FORCE"
    if [ -e "$BIN_DIR/$bin$EXE" ]; then
        owner=$(crates_owner "$bin")
        case "$owner" in
            none)
                if [ -z "$src_force" ]; then
                    src_force="--force"
                    echo "note: replacing $BIN_DIR/$bin$EXE, which no cargo record in $ROOT claims (--force)." >&2
                fi ;;
            "pkg $pkg "*) ;;
            pkg\ *)
                owner_id=${owner#pkg }
                if [ -z "$FORCE" ]; then
                    printf 'install  %s (source: %s)\n' "$name" "$tag"
                    echo "  error: $BIN_DIR/$bin$EXE belongs to cargo package '$owner_id', not $pkg;" >&2
                    echo "         not replacing it. To replace it, re-run with --force (this" >&2
                    echo "         deletes that package's $bin), or first run:" >&2
                    echo "           cargo uninstall --root \"$ROOT\" ${owner_id%% *}" >&2
                    echo "  FAILED" >&2
                    failed_count=$((failed_count + 1))
                    continue
                fi
                echo "note: --force given: replacing $BIN_DIR/$bin$EXE, owned by cargo package '$owner_id'." >&2 ;;
            *)
                if [ -z "$FORCE" ]; then
                    echo "note: cargo's records in $ROOT could not be read with certainty; not forcing." >&2
                    echo "      If cargo refuses because $BIN_DIR/$bin$EXE exists, check what owns it." >&2
                fi ;;
        esac
    fi
    if [ -n "$features" ]; then
        feat_args="--features $features"
    else
        feat_args=""
    fi
    printf 'install  %s (source: %s)\n' "$name" "$tag"
    # shellcheck disable=SC2086
    # $LOCKED / $src_force / $feat_args intentionally unquoted: each is empty
    # or literal flag tokens. --root goes through cargo_root, which passes the
    # directory as ONE argument (a space in it once split it in two: cargo
    # installed into the first half and tried to install a crate named after
    # the second).
    if [ -n "$DRY_RUN" ]; then
        echo "  [dry-run] cargo install --root \"$ROOT\" $LOCKED --git $url --tag $tag $feat_args $src_force $pkg"
        installed_count=$((installed_count + 1))
        install_man_pages "$name"
    elif cargo_root install $LOCKED --git "$url" --tag "$tag" $feat_args $src_force "$pkg"; then
        installed_count=$((installed_count + 1))
        install_man_pages "$name"
    else
        echo "  FAILED" >&2
        failed_count=$((failed_count + 1))
    fi
done

echo
if [ "$failed_count" -gt 0 ]; then
    echo "$installed_count installed, $failed_count failed." >&2
    exit 1
fi
echo "$installed_count installed."
if [ -z "$DRY_RUN" ] && [ "$installed_count" -gt 0 ]; then
    case ":$PATH:" in
        *":$BIN_DIR:"*) ;;
        *)
            echo
            echo "warning: $BIN_DIR is not on your PATH, so the commands below will"
            echo "         not be found yet. Add it to your shell rc, e.g.:"
            echo "           bash/zsh: export PATH=\"$BIN_DIR:\$PATH\""
            echo "           fish:     fish_add_path $BIN_DIR" ;;
    esac
fi
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
