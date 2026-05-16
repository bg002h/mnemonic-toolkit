#!/usr/bin/env python3
"""Lint phase: gui-schema-coverage.

Bidirectional anchor parity between mnemonic-gui's SubcommandSchema
source (live-extracted via the sibling extract_gui_schema module) and
the rendered HTML manual.

Per SPEC §2.0a #1 / §2.1 G1 direction A: every SubcommandSchema name,
FlagSchema name, Dropdown variant, NodeValueComposite node, and
TaggedOrIndexed tag has a matching `id="..."` attribute in the
rendered HTML manual.

Per SPEC §2.0a #1 / §2.1 G1 direction B (schema-shaped subset): every
HTML anchor whose ID matches a `<tab>-<sub>` or
`<tab>-<sub>-<rest>` prefix for a known (tab, sub) pair has a
corresponding schema entry. Prose anchors (chapter titles, foundations,
install, etc.) are exempt by construction — only anchors that look
schema-derived are subject to the orphan check.

Per SPEC §2.2 anchor derivation rule:

    anchor(subcommand) = <tab> + "-" + kebab(name)
    anchor(flag)       = anchor(subcommand) + "-" + flag-name-without-leading-dashes
    anchor(variant)    = anchor(flag) + "-" + kebab(variant)

Per SPEC §2.2 kebab rule: lowercase; non-alphanumeric → "-"; collapse
consecutive dashes; strip leading/trailing dashes. The mnemonic-gui
schema names are already lowercase-ascii-kebab-case, so the rule is
effectively identity for subcommand and flag names, but the kebab
normalization matters for variant strings that contain "/" or other
non-alphanumeric characters.

Invocation:

    python3 check_gui_schema_coverage.py \\
        --upstream-root <path-to-mnemonic-gui> \\
        --html <path-to-rendered-html>

Exits 0 iff zero missing AND zero orphan. Otherwise exits 1 with a
capped one-line-per-finding report on stderr. Exits 2 on usage error.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

# Sibling-module import for the extractor.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import extract_gui_schema


MAX_REPORT_LINES = 20


def kebab(value: str) -> str:
    """SPEC §2.2 kebab rule."""
    out = value.lower()
    out = re.sub(r"[^a-z0-9]+", "-", out)
    out = re.sub(r"-+", "-", out)
    return out.strip("-")


def build_expected(inventory: dict) -> tuple[set[str], set[str]]:
    """Compute (expected_anchors, schema_shape_prefixes).

    The shape-prefix set holds every `<tab>-<sub>` string; HTML anchors
    matching one of those prefixes (exact, or followed by "-") are
    subject to the orphan-direction check.
    """
    expected: set[str] = set()
    shapes: set[str] = set()
    for tab_name in sorted(inventory["tabs"].keys()):
        tab = inventory["tabs"][tab_name]
        for sub_name in sorted(tab["subcommands"].keys()):
            sub = tab["subcommands"][sub_name]
            sub_anchor = f"{tab_name}-{kebab(sub_name)}"
            expected.add(sub_anchor)
            shapes.add(sub_anchor)
            for flag in sub["flags"]:
                flag_name = flag["name"].lstrip("-")
                flag_anchor = f"{sub_anchor}-{flag_name}"
                expected.add(flag_anchor)
                for variant in flag["variants"]:
                    expected.add(f"{flag_anchor}-{kebab(variant)}")
    return expected, shapes


def collect_html_ids(html_path: Path) -> set[str]:
    """Pandoc HTML5 emits `<h1 id="...">` directly on the heading; this
    regex collects every `id="..."` regardless of element type per the
    SPEC §2.1 G1 emission rule."""
    if not html_path.is_file():
        return set()
    text = html_path.read_text(encoding="utf-8")
    return set(re.findall(r'id="([^"]+)"', text))


def is_schema_shaped(anchor: str, shapes: set[str]) -> bool:
    """An HTML anchor is schema-shaped if its ID equals or starts with
    `<tab>-<sub>` for a known (tab, sub) pair.

    SPEC §2.3 outline anchors (`<parent>-outline`) are derived from
    schema anchors via the `-outline` suffix and therefore look
    schema-shaped, but they are required by phase-5 outline-coverage
    rather than schema-derived. Exempt them from the orphan check so
    the two phases do not contradict each other."""
    if anchor.endswith("-outline"):
        return False
    for shape in shapes:
        if anchor == shape or anchor.startswith(shape + "-"):
            return True
    return False


def report(missing: list[str], orphans: list[str]) -> None:
    if missing:
        print(
            f"ERROR: gui-schema-coverage: {len(missing)} schema anchor(s) "
            f"missing from HTML build:",
            file=sys.stderr,
        )
        for anchor in missing[:MAX_REPORT_LINES]:
            print(f"  missing: #{anchor}", file=sys.stderr)
        if len(missing) > MAX_REPORT_LINES:
            print(
                f"  ... and {len(missing) - MAX_REPORT_LINES} more",
                file=sys.stderr,
            )
    if orphans:
        print(
            f"ERROR: gui-schema-coverage: {len(orphans)} schema-shaped HTML "
            f"anchor(s) have no schema entry:",
            file=sys.stderr,
        )
        for anchor in orphans[:MAX_REPORT_LINES]:
            print(f"  orphan: #{anchor}", file=sys.stderr)
        if len(orphans) > MAX_REPORT_LINES:
            print(
                f"  ... and {len(orphans) - MAX_REPORT_LINES} more",
                file=sys.stderr,
            )


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--upstream-root",
        type=Path,
        required=True,
        help="Path to mnemonic-gui repo checkout at the pinned tag.",
    )
    ap.add_argument(
        "--html",
        type=Path,
        required=True,
        help="Path to build/m-format-gui-manual.html.",
    )
    args = ap.parse_args()

    if not args.upstream_root.is_dir():
        print(
            f"ERROR: --upstream-root not a directory: {args.upstream_root}",
            file=sys.stderr,
        )
        return 2

    inventory = extract_gui_schema.extract(args.upstream_root)
    expected, shapes = build_expected(inventory)

    if not args.html.is_file():
        print(
            f"WARN: HTML build not found at {args.html}; "
            f"treating as zero-anchor input. Run `make html` first.",
            file=sys.stderr,
        )

    found = collect_html_ids(args.html)
    missing = sorted(expected - found)
    orphans = sorted(
        a for a in (found - expected) if is_schema_shaped(a, shapes)
    )

    if missing or orphans:
        report(missing, orphans)
        return 1

    print(
        f"OK: gui-schema-coverage: {len(expected)} schema anchors "
        f"({len(shapes)} subcommands) all present in HTML"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
