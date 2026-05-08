#!/usr/bin/env python3
# Source: BIP-93 §"Generating the Checksum"
# (https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki) +
# m-format constellation NUMS-derivation prose for the per-format
# target residues (MK_REGULAR_CONST, MK_LONG_CONST, MD_REGULAR_CONST).
# Transcribed verbatim. See docs/volvelles/verify/ for the codec
# round-trip cross-check.
"""make-volvelle-data.py — BCH paper-computer drift gate + cell-data emitter.

Reads the canonical_vectors table emitted by docs/volvelles/verify/, runs the
BIP-93 polymod against each one, and asserts the result matches the per-format
target residue. On `--check-only` the script is silent on success and exits
non-zero on any divergence (CI gates on this).

Without `--check-only`, after a passing check the script writes per-format
TikZ cell-data fragments at docs/volvelles/{mk-regular,mk-long,md-regular}.cells.tex
for the wheel template to `\\input{}`.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

try:
    import tomllib  # Python ≥ 3.11
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib  # type: ignore[no-redef]


# ---------------------------------------------------------------------------
# BCH constants (verbatim from md-codec / mk-codec source)
# ---------------------------------------------------------------------------

POLYMOD_INIT = 0x23181B3

# Regular code: BCH(93,80,8). 13-symbol checksum. 60-bit residue body.
GEN_REGULAR = (
    0x19DC500CE73FDE210,
    0x1BFAE00DEF77FE529,
    0x1FBD920FFFE7BEE52,
    0x1739640BDEEE3FDAD,
    0x07729A039CFC75F5A,
)
REGULAR_SHIFT = 60
REGULAR_MASK = (1 << REGULAR_SHIFT) - 1

# Long code: BCH(108,93,8). 15-symbol checksum. 70-bit residue body.
GEN_LONG = (
    0x3D59D273535EA62D897,
    0x7A9BECB6361C6C51507,
    0x543F9B7E6C38D8A2A0E,
    0x0C577EAECCF1990D13C,
    0x1887F74F8DC71B10651,
)
LONG_SHIFT = 70
LONG_MASK = (1 << LONG_SHIFT) - 1

# Per-format NUMS-derived target residues.
MK_REGULAR_CONST = 0x1062435F91072FA5C
MK_LONG_CONST = 0x41890D7E441CBE97273
MD_REGULAR_CONST = 0x0815C07747A3392E7

# bech32 alphabet, value-indexed.
ALPHABET = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"
ALPHABET_INV = {c: i for i, c in enumerate(ALPHABET)}


# ---------------------------------------------------------------------------
# Per-format dispatch
# ---------------------------------------------------------------------------

# Each entry: (HRP, GEN, shift, mask, target_const).
FORMATS = {
    "mk1-regular": ("mk", GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK, MK_REGULAR_CONST),
    "mk1-long":    ("mk", GEN_LONG,    LONG_SHIFT,    LONG_MASK,    MK_LONG_CONST),
    "md1-regular": ("md", GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK, MD_REGULAR_CONST),
}

# Output file basename per format (lives under docs/volvelles/).
CELLS_BASENAME = {
    "mk1-regular": "mk-regular",
    "mk1-long":    "mk-long",
    "md1-regular": "md-regular",
}


# ---------------------------------------------------------------------------
# BIP-93 polymod (verbatim)
# ---------------------------------------------------------------------------

def polymod_step(residue: int, value: int, gen: tuple[int, ...], shift: int, mask: int) -> int:
    """One BIP-93 polymod step. See bip-0093 §"Generating the Checksum"."""
    b = residue >> shift
    new_residue = ((residue & mask) << 5) ^ value
    for i, g in enumerate(gen):
        if (b >> i) & 1:
            new_residue ^= g
    return new_residue


def polymod_run(values, gen: tuple[int, ...], shift: int, mask: int) -> int:
    """Run polymod over a sequence of 5-bit values, starting from POLYMOD_INIT."""
    residue = POLYMOD_INIT
    for v in values:
        residue = polymod_step(residue, v, gen, shift, mask)
    return residue


def hrp_expand(hrp: str) -> list[int]:
    """BIP-173 HRP expansion: high-3-bits || [0] || low-5-bits."""
    out = [ord(c) >> 5 for c in hrp]
    out.append(0)
    out.extend(ord(c) & 31 for c in hrp)
    return out


def decode_data_part(data_part: str) -> list[int]:
    """Decode the bech32 data part to 5-bit values; raises on bad chars."""
    out = []
    for i, c in enumerate(data_part):
        v = ALPHABET_INV.get(c.lower())
        if v is None:
            raise ValueError(f"non-bech32 char {c!r} at data-part index {i}")
        out.append(v)
    return out


# ---------------------------------------------------------------------------
# Drift gate
# ---------------------------------------------------------------------------

def check_vector(format_name: str, s: str) -> None:
    """Polymod the canonical vector and assert it hits the per-format target.

    Raises AssertionError or ValueError on any divergence.
    """
    if format_name not in FORMATS:
        raise ValueError(f"unknown format {format_name!r}")
    hrp, gen, shift, mask, target = FORMATS[format_name]

    sep_pos = s.rfind("1")
    if sep_pos < 0:
        raise ValueError(f"{format_name}: no '1' separator in {s!r}")
    actual_hrp = s[:sep_pos]
    data_part = s[sep_pos + 1 :]
    if actual_hrp != hrp:
        raise ValueError(f"{format_name}: expected HRP {hrp!r}, got {actual_hrp!r}")

    symbols = hrp_expand(hrp) + decode_data_part(data_part)
    residue = polymod_run(symbols, gen, shift, mask)
    if residue != target:
        raise AssertionError(
            f"{format_name}: polymod residue {residue:#x} ≠ target {target:#x}"
            f" (input={s!r}). Helper has drifted from the codec."
        )


def run_drift_gate(params_path: Path) -> None:
    """Read the TOML and check every canonical_vectors entry. Raises on failure."""
    with params_path.open("rb") as f:
        params = tomllib.load(f)
    schema_version = params.get("schema_version")
    if schema_version != 1:
        raise ValueError(
            f"{params_path}: unsupported schema_version {schema_version!r} (expected 1)"
        )
    vectors = params.get("canonical_vectors") or []
    if not vectors:
        raise ValueError(f"{params_path}: no [[canonical_vectors]] entries")
    for v in vectors:
        check_vector(v["format"], v["input"])
        # The codec_output field re-confirms the codec round-trip; the gate
        # also verifies the helper agrees with the codec on the post-decode
        # rendering. For v0.1 single-string fixtures, input == codec_output.
        if v["input"] != v["codec_output"]:
            raise AssertionError(
                f"{v['format']}: input != codec_output (helper expects identity"
                f" for v0.1 single-string fixtures): {v!r}"
            )


# ---------------------------------------------------------------------------
# Cell-data emission
# ---------------------------------------------------------------------------

# 32x32 grid layout (per spec §4 wheel-layout table).
GRID_N = 32
RADIAL_PITCH_IN = 0.35   # ≥ 0.35" per spec
ANGULAR_PITCH_DEG = 11.25  # 360 / 32


def emit_cells_tex(format_name: str, out_path: Path) -> None:
    """Write a TikZ fragment with one \\node per (row, col) cell.

    Cell content at (R, C) is the bech32 char of polymod_step(R, C, ...) for
    the format's BCH parameters. The top 5 bits of R are not constrained to
    valid 5-bit values — R ranges 0..31, modeling the "current state's
    selector b" the user lines up — but the math runs over the full 5-bit
    state-and-value space the wheel exposes.
    """
    _hrp, gen, shift, mask, _target = FORMATS[format_name]

    lines = []
    lines.append(f"% Auto-generated by docs/tools/make-volvelle-data.py for {format_name}.")
    lines.append("% Do not hand-edit; regenerate from docs/volvelles/bch-params.toml.")
    lines.append(f"% Format: {GRID_N}x{GRID_N} cells, radial pitch ≥ {RADIAL_PITCH_IN}in,"
                 f" angular pitch {ANGULAR_PITCH_DEG} deg.")
    for r in range(GRID_N):
        for c in range(GRID_N):
            # Place state R into the top 5 bits of the residue so the polymod
            # selector b = R; the input value is C. The result's low 5 bits
            # (mod 32) pick the bech32 character that lands in the cell.
            residue = (r & 0x1F) << shift
            stepped = polymod_step(residue, c, gen, shift, mask)
            ch = ALPHABET[stepped & 0x1F]
            angle_deg = c * ANGULAR_PITCH_DEG
            radius_in = RADIAL_PITCH_IN * (r + 1)
            # Innermost two rings have the tightest arc length per cell;
            # ramp the font down for r=0 (smallest radius) and r=1.
            if r == 0:
                cell_macro = "volvellecellinnermost"
            elif r == 1:
                cell_macro = "volvellecellinner"
            else:
                cell_macro = "volvellecell"
            lines.append(
                f"\\node at ({angle_deg:.4f}:{radius_in:.4f}in)"
                f" {{\\{cell_macro}{{{ch}}}}}; % r={r} c={c}"
            )
    out_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def emit_all_cells(out_dir: Path) -> list[Path]:
    written: list[Path] = []
    for fmt, basename in CELLS_BASENAME.items():
        path = out_dir / f"{basename}.cells.tex"
        emit_cells_tex(fmt, path)
        written.append(path)
    return written


# ---------------------------------------------------------------------------
# Inline self-tests (run on import + as __main__)
# ---------------------------------------------------------------------------

def _self_tests() -> None:
    # polymod_step(0, 0) must produce 0 (zero state, zero input, no GEN XORs).
    assert polymod_step(0, 0, GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK) == 0
    assert polymod_step(0, 0, GEN_LONG, LONG_SHIFT, LONG_MASK) == 0

    # Known values: hrp_expand("md") and hrp_expand("mk").
    # 'm'=0x6D → high3=3, low5=13. 'd'=0x64 → high3=3, low5=4.
    # 'k'=0x6B → high3=3, low5=11.
    assert hrp_expand("md") == [3, 3, 0, 13, 4]
    assert hrp_expand("mk") == [3, 3, 0, 13, 11]

    # POLYMOD_INIT < 2^60: first regular step with value=0 just shifts left 5.
    assert polymod_step(POLYMOD_INIT, 0, GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK) \
        == POLYMOD_INIT << 5


_self_tests()


# ---------------------------------------------------------------------------
# CLI entry point
# ---------------------------------------------------------------------------

def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("--params", type=Path, required=True,
                   help="Path to bch-params.toml emitted by docs/volvelles/verify/.")
    p.add_argument("--check-only", action="store_true",
                   help="Run the drift gate; exit non-zero on divergence. No file output.")
    p.add_argument("--out-dir", type=Path, default=None,
                   help="Where to write per-format .cells.tex (default: parent dir of --params).")
    args = p.parse_args(argv)

    try:
        run_drift_gate(args.params)
    except (AssertionError, ValueError, KeyError, FileNotFoundError) as e:
        print(f"make-volvelle-data: drift gate FAILED: {e}", file=sys.stderr)
        return 1

    if args.check_only:
        return 0

    out_dir = args.out_dir or args.params.parent
    out_dir.mkdir(parents=True, exist_ok=True)
    written = emit_all_cells(out_dir)
    for path in written:
        print(f"wrote {path}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
