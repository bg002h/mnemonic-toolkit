#!/usr/bin/env python3
"""Lint phase: outline-coverage.

Per SPEC §2.1 G2 + §2.3, every subcommand section with at least two
flags MUST begin with an `### Outline {#<sub>-outline}` heading whose
immediate body contains exactly N top-level bullets (N = flag count).
Every Dropdown / NodeValueComposite / TaggedOrIndexed flag section
with at least two variants MUST begin with a
`#### Outline {#<flag>-outline}` heading whose body contains exactly
V top-level bullets (V = variant count).

The lint reads markdown source (not rendered HTML) so it can fire
before the pandoc build step. The anchor convention is
`<anchor-of-parent-section>-outline`; the heading text itself is the
literal string "Outline" by convention but only the anchor is gated.

Bullet counting is strict: only column-0 `-` or `*` lines between the
target heading and the next heading are counted. Indented sub-items
(continuation lines, nested lists) are intentionally excluded — the
outline shape is a flat link list per SPEC §2.3.

Fenced code blocks (``` or ~~~) are skipped when scanning for
headings: a code sample that contains literal `## title {#anchor}`
must not be miscounted as an outline target.

Invocation:

    python3 check_outline_coverage.py \\
        --upstream-root <path-to-mnemonic-gui> \\
        --src-dir <path-to-docs/manual-gui/src>

Exits 0 iff every expected outline exists at the right anchor with
the right bullet count. Exits 1 on any failure. Exits 2 on usage error.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import extract_gui_schema


MAX_REPORT_LINES = 20

# `## Heading text {#anchor-id}` — level captured but not used; anchor required.
HEADING_RE = re.compile(r"^(#+)\s+.*?\s*\{#([a-z0-9-]+)\}\s*$")
# Top-level bullets only — leading whitespace fails the match. SPEC §2.3
# requires a flat link list, not nested.
BULLET_RE = re.compile(r"^[-*]\s+")

ENUMERATED_KINDS = frozenset({"Dropdown", "NodeValueComposite", "TaggedOrIndexed"})


def kebab(value: str) -> str:
    """SPEC §2.2 kebab rule."""
    out = value.lower()
    out = re.sub(r"[^a-z0-9]+", "-", out)
    out = re.sub(r"-+", "-", out)
    return out.strip("-")


def expected_outlines(inventory: dict) -> list[tuple[str, int, str]]:
    """Returns [(outline-anchor, expected-bullet-count, kind-label), ...]
    for every (subcommand, flag) pair whose cardinality is >= 2."""
    out: list[tuple[str, int, str]] = []
    for tab_name in sorted(inventory["tabs"].keys()):
        tab = inventory["tabs"][tab_name]
        for sub_name in sorted(tab["subcommands"].keys()):
            sub = tab["subcommands"][sub_name]
            sub_anchor = f"{tab_name}-{kebab(sub_name)}"
            if len(sub["flags"]) >= 2:
                out.append(
                    (f"{sub_anchor}-outline", len(sub["flags"]), "subcommand-outline")
                )
            for flag in sub["flags"]:
                if (
                    flag["kind"] in ENUMERATED_KINDS
                    and len(flag["variants"]) >= 2
                ):
                    flag_name = flag["name"].lstrip("-")
                    flag_anchor = f"{sub_anchor}-{flag_name}"
                    out.append(
                        (
                            f"{flag_anchor}-outline",
                            len(flag["variants"]),
                            "flag-outline",
                        )
                    )
    return out


def scan_markdown(src_dir: Path) -> dict[str, int]:
    """Walk src_dir/**/*.md and return {anchor: top-level-bullet-count}
    for every heading that carries an explicit `{#anchor}` tag."""
    found: dict[str, int] = {}
    if not src_dir.is_dir():
        return found
    for md_path in sorted(src_dir.rglob("*.md")):
        lines = md_path.read_text(encoding="utf-8").splitlines()
        in_code = False
        i = 0
        while i < len(lines):
            stripped = lines[i].lstrip()
            if stripped.startswith("```") or stripped.startswith("~~~"):
                in_code = not in_code
                i += 1
                continue
            if in_code:
                i += 1
                continue
            match = HEADING_RE.match(lines[i])
            if match:
                anchor = match.group(2)
                bullets = _count_bullets_until_next_heading(lines, i + 1)
                found[anchor] = bullets
            i += 1
    return found


def _count_bullets_until_next_heading(lines: list[str], start: int) -> int:
    bullets = 0
    in_code = False
    j = start
    while j < len(lines):
        stripped = lines[j].lstrip()
        if stripped.startswith("```") or stripped.startswith("~~~"):
            in_code = not in_code
            j += 1
            continue
        if in_code:
            j += 1
            continue
        if HEADING_RE.match(lines[j]):
            break
        if BULLET_RE.match(lines[j]):
            bullets += 1
        j += 1
    return bullets


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--upstream-root",
        type=Path,
        required=True,
        help="Path to mnemonic-gui repo checkout at the pinned tag.",
    )
    ap.add_argument(
        "--src-dir",
        type=Path,
        required=True,
        help="Path to docs/manual-gui/src.",
    )
    args = ap.parse_args()

    if not args.upstream_root.is_dir():
        print(
            f"ERROR: --upstream-root not a directory: {args.upstream_root}",
            file=sys.stderr,
        )
        return 2
    if not args.src_dir.exists():
        print(
            f"ERROR: --src-dir not found: {args.src_dir}",
            file=sys.stderr,
        )
        return 2

    inventory = extract_gui_schema.extract(args.upstream_root)
    expected = expected_outlines(inventory)
    found = scan_markdown(args.src_dir)

    missing: list[str] = []
    mismatched: list[str] = []
    for anchor, expected_count, kind in expected:
        if anchor not in found:
            missing.append(
                f"  missing: #{anchor} ({kind}; expects {expected_count} bullets)"
            )
        elif found[anchor] != expected_count:
            mismatched.append(
                f"  mismatch: #{anchor} ({kind}) expects {expected_count} bullets, "
                f"got {found[anchor]}"
            )

    if missing or mismatched:
        if missing:
            print(
                f"ERROR: outline-coverage: {len(missing)} expected outline(s) "
                f"missing from markdown source:",
                file=sys.stderr,
            )
            for line in missing[:MAX_REPORT_LINES]:
                print(line, file=sys.stderr)
            if len(missing) > MAX_REPORT_LINES:
                print(
                    f"  ... and {len(missing) - MAX_REPORT_LINES} more",
                    file=sys.stderr,
                )
        if mismatched:
            print(
                f"ERROR: outline-coverage: {len(mismatched)} outline(s) have "
                f"wrong bullet count:",
                file=sys.stderr,
            )
            for line in mismatched[:MAX_REPORT_LINES]:
                print(line, file=sys.stderr)
            if len(mismatched) > MAX_REPORT_LINES:
                print(
                    f"  ... and {len(mismatched) - MAX_REPORT_LINES} more",
                    file=sys.stderr,
                )
        return 1

    print(
        f"OK: outline-coverage: {len(expected)} outlines all present with "
        f"correct bullet count"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
