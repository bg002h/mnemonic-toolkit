#!/usr/bin/env python3
"""Extract a JSON inventory of mnemonic-gui's SubcommandSchema entries.

Reads `<MANUAL_GUI_UPSTREAM_ROOT>/src/schema/{mnemonic,md,ms,mk}.rs`
and emits a JSON document with this shape:

    {
      "tabs": {
        "mnemonic": {
          "subcommands": {
            "convert": {
              "flags": [
                {"name": "--from", "kind": "NodeValueComposite", "variants": ["phrase", "entropy", ...]},
                {"name": "--to", "kind": "Dropdown", "variants": ["mainnet", ...]},
                {"name": "--passphrase", "kind": "Text", "variants": []},
                ...
              ],
              "repeating_flags": ["--slot", "--to", ...],
              "has_dropdown_or_composite": true
            },
            ...
          }
        },
        "md": {...}, "ms": {...}, "mk": {...}
      }
    }

The regex-over-source approach is justified by the schema files'
stylized const-decl shape (4 files × ~1000 LOC, no `#[cfg(...)]`
gating, no macro-generated entries). If that ever changes, migrate to
`cargo run --bin extract_gui_schema` using the `syn` crate.

Invocation:

    python3 extract_gui_schema.py --upstream-root ../mnemonic-gui [--out inventory.json]

Reads tag-pinned source IF the upstream-root checkout is at the tag in
docs/manual-gui/pinned-upstream.toml. The lint phase that consumes
this script's output is `tests/lint.sh::gui-schema-coverage`.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path


TAB_FILES = ("mnemonic", "md", "ms", "mk")


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _collect_named_slices(src: str) -> dict[str, list[str]]:
    """Find every `pub const NAME: &[&str] = &["a", "b", ...];` decl.

    Returns mapping NAME -> [values].
    """
    out: dict[str, list[str]] = {}
    # Match: `const NAME: &[&str] = &[ "v1", "v2", ... ];` (multiline).
    pattern = re.compile(
        r"const\s+([A-Z][A-Z0-9_]+)\s*:\s*&\[&str\]\s*=\s*&\[\s*([^\]]+)\]\s*;",
        re.MULTILINE | re.DOTALL,
    )
    for match in pattern.finditer(src):
        name = match.group(1)
        body = match.group(2)
        # Extract "...".
        values = re.findall(r'"([^"]*)"', body)
        out[name] = values
    return out


def _collect_subcommands(src: str) -> list[tuple[str, str]]:
    """Find every SubcommandSchema entry. Returns [(name, flag-array-const-name)].

    Matches: `SubcommandSchema { name: "convert", ..., flags: CONVERT_FLAGS, ... }`
    """
    out: list[tuple[str, str]] = []
    # The arrays are top-level static lists like:
    #   pub static SUBCOMMANDS: &[SubcommandSchema] = &[
    #       SubcommandSchema { name: "bundle", ..., flags: BUNDLE_FLAGS, ...},
    #       ...
    #   ];
    pattern = re.compile(
        r'SubcommandSchema\s*\{\s*name:\s*"([^"]+)"[^}]*?flags:\s*([A-Z][A-Z0-9_]+)',
        re.MULTILINE | re.DOTALL,
    )
    for match in pattern.finditer(src):
        out.append((match.group(1), match.group(2)))
    return out


def _collect_flag_array(src: str, array_name: str) -> list[dict]:
    """Parse a FlagSchema array by name. Returns list of {name, kind, variants, repeating}."""
    # Find the array decl: `const NAME: &[FlagSchema] = &[ ... ];`
    decl_re = re.compile(
        rf"const\s+{re.escape(array_name)}\s*:\s*&\[FlagSchema\]\s*=\s*&\[\s*(.*?)\];\s*$",
        re.MULTILINE | re.DOTALL,
    )
    match = decl_re.search(src)
    if not match:
        return []
    body = match.group(1)

    # Split into FlagSchema { ... } blocks. Brace-depth-aware split.
    flags: list[dict] = []
    blocks = _split_flagschema_blocks(body)
    for block in blocks:
        # Extract name.
        name_match = re.search(r'name:\s*"([^"]+)"', block)
        if not name_match:
            continue
        name = name_match.group(1)
        # Extract kind.
        kind, variants = _classify_kind(block)
        # Extract repeating.
        repeating = bool(re.search(r"repeating:\s*true", block))
        flags.append(
            {
                "name": name,
                "kind": kind,
                "variants": variants,
                "repeating": repeating,
            }
        )
    return flags


def _split_flagschema_blocks(body: str) -> list[str]:
    """Split a FlagSchema array body into per-block strings."""
    blocks: list[str] = []
    depth = 0
    start = -1
    for i, ch in enumerate(body):
        if ch == "{":
            if depth == 0:
                start = i
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0 and start >= 0:
                blocks.append(body[start : i + 1])
                start = -1
    return blocks


def _classify_kind(block: str) -> tuple[str, list[str]]:
    """Classify FlagKind and extract variants (for Dropdown / NodeValueComposite /
    TaggedOrIndexed). Returns ("kind", ["v1", "v2", ...]).
    """
    # Dropdown(CONST) — variants are values of CONST (resolved by caller).
    m = re.search(r"FlagKind::Dropdown\(\s*([A-Z][A-Z0-9_]+)\s*\)", block)
    if m:
        return ("Dropdown", [m.group(1)])  # caller resolves
    # NodeValueComposite(CONST)
    m = re.search(
        r"FlagKind::NodeValueComposite\s*\{\s*nodes:\s*([A-Z][A-Z0-9_]+)\s*\}", block
    )
    if m:
        return ("NodeValueComposite", [m.group(1)])  # caller resolves
    # NodeValueComposite(&[...])
    m = re.search(
        r'FlagKind::NodeValueComposite\s*\{\s*nodes:\s*&\[\s*([^\]]+)\]\s*\}', block
    )
    if m:
        values = re.findall(r'"([^"]*)"', m.group(1))
        return ("NodeValueComposite", values)
    # TaggedOrIndexed(&["nums"])
    m = re.search(r'FlagKind::TaggedOrIndexed\(\s*&\[\s*([^\]]+)\]\s*\)', block)
    if m:
        values = re.findall(r'"([^"]*)"', m.group(1))
        return ("TaggedOrIndexed", values)
    # Other kinds (Text, Number, Boolean, Path, Range, Timestamp).
    m = re.search(r"FlagKind::([A-Z][A-Za-z]+)", block)
    if m:
        return (m.group(1), [])
    return ("Unknown", [])


def extract(upstream_root: Path) -> dict:
    schema_dir = upstream_root / "src" / "schema"
    if not schema_dir.is_dir():
        raise SystemExit(f"schema dir not found: {schema_dir}")

    out = {"tabs": {}}
    for tab in TAB_FILES:
        rs = schema_dir / f"{tab}.rs"
        if not rs.is_file():
            raise SystemExit(f"schema file not found: {rs}")
        src = _read(rs)
        named_slices = _collect_named_slices(src)
        subcommands = _collect_subcommands(src)

        tab_entry = {"subcommands": {}}
        for sub_name, flag_array_name in subcommands:
            flags = _collect_flag_array(src, flag_array_name)
            # Resolve named-slice references in variants.
            for flag in flags:
                if flag["kind"] in ("Dropdown", "NodeValueComposite") and flag["variants"]:
                    first = flag["variants"][0]
                    if first in named_slices:
                        flag["variants"] = named_slices[first]
            tab_entry["subcommands"][sub_name] = {
                "flags": flags,
                "has_dropdown_or_composite": any(
                    f["kind"] in ("Dropdown", "NodeValueComposite", "TaggedOrIndexed")
                    for f in flags
                ),
                "repeating_flags": [f["name"] for f in flags if f["repeating"]],
            }
        out["tabs"][tab] = tab_entry
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--upstream-root",
        type=Path,
        required=True,
        help="Path to mnemonic-gui repo checkout at the pinned tag.",
    )
    ap.add_argument(
        "--out",
        type=Path,
        help="Output path. If unset, JSON is written to stdout.",
    )
    args = ap.parse_args()
    data = extract(args.upstream_root)
    out_str = json.dumps(data, indent=2, sort_keys=True)
    if args.out:
        args.out.write_text(out_str + "\n", encoding="utf-8")
    else:
        sys.stdout.write(out_str + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
