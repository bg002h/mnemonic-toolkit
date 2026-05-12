#!/usr/bin/env bash
# tests/api-surface-coverage.sh
#
# Hint helper (NOT a lint gate; SPEC §4.4). For each of the four Part V
# crates, enumerate the public top-level symbol names exposed at the crate
# root (re-exports + `pub mod` declarations + crate-root `pub fn` / `pub
# struct` / `pub enum` / `pub trait` / `pub const` / `pub type` /
# `pub static`) and grep each one against the relevant Part V chapter at
# `src/50-rust-api/`. Emit a warning row per symbol absent from the chapter.
#
# Approximation, not an AST parse — the enumeration is a deliberate
# best-effort regex over `lib.rs` only. False negatives (symbol missed by
# the regex) and false positives (a symbol the chapter cites only inside
# a code block that happens to use the literal name in a non-`use`
# context) are both possible; warnings are advisory, never blocking.
#
# v0.4: covers md-codec, mk-codec, ms-codec, mnemonic-toolkit. The
# binary-only mnemonic-toolkit crate has no `lib.rs`, so we special-case
# it against the seven serde JSON-envelope items declared at the crate
# root of `src/format.rs` (the surface the Part V chapter actually
# documents).
#
# Exits 0 in all cases. Called from `lint.sh` step 4/6.

set -euo pipefail

# -------- args --------

SRC_DIR=""
for arg in "$@"; do
  case "$arg" in
    SRC_DIR=*)      SRC_DIR="${arg#*=}" ;;
    MD_BIN=*)       : ;; # unused; reserved for future runtime probes
    MK_BIN=*)       : ;;
    MS_BIN=*)       : ;;
    MNEMONIC_BIN=*) : ;;
  esac
done

# Resolve SRC_DIR default from this script's location if not supplied.
if [ -z "$SRC_DIR" ]; then
  HERE="$(cd "$(dirname "$0")" && pwd)"
  SRC_DIR="$(cd "$HERE/../src" && pwd)"
fi

# Derive the sibling-repo workspace root from SRC_DIR
#   .../<workspace>/mnemonic-toolkit/docs/technical-manual/src
# → .../<workspace>
WORKSPACE_ROOT="$(cd "$SRC_DIR/../../../.." && pwd)"

MD_CRATE="$WORKSPACE_ROOT/descriptor-mnemonic/crates/md-codec"
MK_CRATE="$WORKSPACE_ROOT/mnemonic-key/crates/mk-codec"
MS_CRATE="$WORKSPACE_ROOT/mnemonic-secret/crates/ms-codec"
TK_FORMAT="$WORKSPACE_ROOT/mnemonic-toolkit/crates/mnemonic-toolkit/src/format.rs"

CHAP_DIR="$SRC_DIR/50-rust-api"

warnings=0
crates_checked=0

# -------- helpers --------

note() { printf '[api-surface-coverage] %s\n' "$*"; }
warn() { printf '[api-surface-coverage] WARNING — %s\n' "$*" >&2; warnings=$((warnings + 1)); }

# Enumerate public top-level symbols from a `lib.rs`.
#
# Strategy:
#   1. Strip /* ... */ block comments and // line comments.
#   2. Parse every `pub use PATH { a, b as c, ... };` block — emit the
#      brace contents (renames: take the `as <NAME>` suffix), and for the
#      no-brace form `pub use PATH::NAME;` emit the tail segment.
#   3. Emit every `pub mod NAME;` declaration (declarations only; inline
#      `pub mod NAME { ... }` bodies are skipped — they re-export their
#      items through the path above when intended).
#   4. Emit every `pub fn / struct / enum / trait / const / type / static
#      NAME` at line start.
#
# Heuristic, not an AST parse — see file header. Python is used because
# pure-shell regex over multi-line `pub use` blocks is fragile.
enumerate_lib_symbols() {
  local lib="$1"
  python3 - "$lib" <<'PY'
import re, sys
src = open(sys.argv[1]).read()
src = re.sub(r'/\*.*?\*/', '', src, flags=re.S)
src = re.sub(r'//.*', '', src)

names = set()

# pub use ... ;  (greedy until terminating semicolon)
for m in re.finditer(r'pub\s+use\s+([^;]+);', src, flags=re.S):
    body = m.group(1)
    if '{' in body:
        inner = body[body.index('{')+1 : body.rindex('}')]
        for tok in inner.split(','):
            tok = tok.strip()
            if ' as ' in tok:
                tok = tok.split(' as ')[-1].strip()
            tok = tok.strip()
            if tok:
                names.add(tok)
    else:
        tail = body.strip().split('::')[-1].strip()
        if ' as ' in tail:
            tail = tail.split(' as ')[-1].strip()
        if tail:
            names.add(tail)

# pub mod NAME;   (declaration form only)
for m in re.finditer(r'^[ \t]*pub\s+mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;', src, flags=re.M):
    names.add(m.group(1))

# pub <kind> NAME — crate-root items.
for m in re.finditer(
    r'^[ \t]*pub\s+(?:unsafe\s+)?(?:async\s+)?'
    r'(?:fn|struct|enum|trait|const|type|static)\s+'
    r'([A-Za-z_][A-Za-z0-9_]*)',
    src, flags=re.M,
):
    names.add(m.group(1))

for n in sorted(names):
    print(n)
PY
}

# Enumerate v0.8 JSON-envelope item names from mnemonic-toolkit's
# binary-only crate. The Part V chapter documents the seven serde-bearing
# items at the crate root of `format.rs`; we check those specifically
# rather than every `pub` item in the binary tree.
#
# The seven are: MsField (pub type), MkField (pub enum), CosignerEntry,
# MultisigInfo, BundleJson, VerifyBundleJson, VerifyCheck.
enumerate_toolkit_envelope() {
  printf '%s\n' \
    BundleJson \
    CosignerEntry \
    MkField \
    MsField \
    MultisigInfo \
    VerifyBundleJson \
    VerifyCheck
}

# Greater-than-zero exit if `sym` is absent from `chapter`. The check is
# whole-word grep with the symbol treated literally (no regex
# metacharacter splash).
check_symbol_present() {
  local sym="$1" chapter="$2"
  grep -qwF "$sym" "$chapter"
}

# Check one crate against its chapter.
#   $1 — crate-label (e.g. md-codec)
#   $2 — chapter basename inside $CHAP_DIR (e.g. 51-md-codec-api.md)
#   $3 — newline-separated symbol list
#
# Uses globals `warnings` and `crates_checked` (no piped subshell).
check_crate() {
  local label="$1" chapter_name="$2" symbols="$3"
  local chapter="$CHAP_DIR/$chapter_name"
  if [ ! -f "$chapter" ]; then
    warn "$label: chapter file missing at $chapter"
    return
  fi
  if [ -z "$symbols" ]; then
    warn "$label: no public symbols enumerated (check that the crate exists)"
    return
  fi
  local total=0 missing=0
  while IFS= read -r sym; do
    [ -z "$sym" ] && continue
    total=$((total + 1))
    if ! check_symbol_present "$sym" "$chapter"; then
      warn "$label: \`$sym\` missing from $chapter_name"
      missing=$((missing + 1))
    fi
  done <<<"$symbols"
  local covered=$((total - missing))
  note "$label: $total public symbols"
  note "$label: $covered/$total covered in $chapter_name"
  crates_checked=$((crates_checked + 1))
}

# -------- run --------

# md-codec
if [ -f "$MD_CRATE/src/lib.rs" ]; then
  md_syms="$(enumerate_lib_symbols "$MD_CRATE/src/lib.rs")"
  check_crate "md-codec" "51-md-codec-api.md" "$md_syms"
else
  warn "md-codec: $MD_CRATE/src/lib.rs not found; skipping"
fi

# mk-codec
if [ -f "$MK_CRATE/src/lib.rs" ]; then
  mk_syms="$(enumerate_lib_symbols "$MK_CRATE/src/lib.rs")"
  check_crate "mk-codec" "52-mk-codec-api.md" "$mk_syms"
else
  warn "mk-codec: $MK_CRATE/src/lib.rs not found; skipping"
fi

# ms-codec
if [ -f "$MS_CRATE/src/lib.rs" ]; then
  ms_syms="$(enumerate_lib_symbols "$MS_CRATE/src/lib.rs")"
  check_crate "ms-codec" "53-ms-codec-api.md" "$ms_syms"
else
  warn "ms-codec: $MS_CRATE/src/lib.rs not found; skipping"
fi

# mnemonic-toolkit (binary-only; check JSON-envelope items).
if [ -f "$TK_FORMAT" ]; then
  note "mnemonic-toolkit: 7 JSON envelope types (binary-only crate)"
  tk_syms="$(enumerate_toolkit_envelope)"
  check_crate "mnemonic-toolkit" "54-mnemonic-toolkit-api.md" "$tk_syms"
else
  warn "mnemonic-toolkit: $TK_FORMAT not found; skipping"
fi

# -------- summary --------

note "OK ($crates_checked crates checked; $warnings warnings)"
exit 0
