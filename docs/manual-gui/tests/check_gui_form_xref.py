#!/usr/bin/env python3
"""Lint phase: gui-form-xref.

Bidirectional cross-reference parity for the dedicated "GUI Forms"
Part (SPEC §6 — closes the cross-link verification hole I-r1-1). The
61 structural form renders live in the gallery chapters under
`src/75-gui-forms/`; each per-subcommand chapter keeps a one-line
cross-link pointing at its gallery form. Nothing else gates that the
two ends agree — lychee runs `--offline` with no `--include-fragments`
(so it skips bare `#gui-form-*` intra-doc fragments), pandoc/LaTeX do
not hard-fail dangling internal links, and `include-transcript.lua`
fail-closes only on bad `.gui` STEMS, not bad cross-link fragments. So
without this check a typo on any of the ~122 edit sites would ship a
dead link no other gate catches.

The canonical stem list is the set of `*.gui` basenames committed in
`transcripts/gui/` (one per pinned-GUI subcommand form). The check
keys on the FULL filename stem (NOT the first hyphen) so a
multi-segment sub like `mnemonic-xpub-search-account-of-descriptor`
is never mis-split (plan M-r2-3). For every stem `S` the manual must
carry:

  (i)  EXACTLY ONE `{#gui-form-S}` heading anchor across the gallery
       chapters (`src/75-gui-forms/`); and
  (ii) EXACTLY ONE `](#gui-form-S)` cross-link in the subcommand
       chapters (`src/`, EXCLUDING `src/75-gui-forms/`).

Scoping the cross-link count to NON-gallery chapters lets a future
gallery-overview cross-reference list link each form without
false-failing the gate (plan M-1). The tour's bare `include="gui/…"`
fences carry neither a `{#gui-form-*}` anchor nor a `](#gui-form-*)`
link token, so they cannot perturb the counts (SPEC §5 / M-r2-2).

Reverse / orphan direction: every `gui-form-*` anchor or cross-link
token found anywhere in `src/` must map to a real `.gui` stem.

Invocation:

    python3 check_gui_form_xref.py \\
        --transcripts-gui <path-to-transcripts/gui> \\
        --src-dir <path-to-docs/manual-gui/src>

Exits 0 iff every stem has exactly one gallery anchor + exactly one
cross-link and no orphan token exists. Exits 1 on any missing / extra
/ orphan, naming the offending stem(s). Exits 2 on usage error.
"""

from __future__ import annotations

import argparse
import re
import sys
from collections import Counter
from pathlib import Path

MAX_REPORT_LINES = 80

GALLERY_SUBDIR = "75-gui-forms"

# Heading anchors: `## ... {#gui-form-<stem>}`.
ANCHOR_RE = re.compile(r"\{#gui-form-([a-z0-9-]+)\}")
# Cross-links: `](#gui-form-<stem>)`.
LINK_RE = re.compile(r"\]\(#gui-form-([a-z0-9-]+)\)")


def canonical_stems(transcripts_gui: Path) -> list[str]:
    return sorted(p.stem for p in transcripts_gui.glob("*.gui"))


def scan(src_dir: Path) -> tuple[Counter, Counter, Counter, Counter]:
    """Walk src_dir/**/*.md once.

    Returns (gallery_anchor_counts, crosslink_counts,
    all_anchor_tokens, all_link_tokens). The first two are scoped
    (anchors counted only inside the gallery, cross-links only
    outside it); the last two are unscoped, for the orphan check.
    """
    gallery_anchors: Counter = Counter()
    crosslinks: Counter = Counter()
    all_anchor_tokens: Counter = Counter()
    all_link_tokens: Counter = Counter()
    for md_path in sorted(src_dir.rglob("*.md")):
        rel = md_path.relative_to(src_dir)
        in_gallery = len(rel.parts) >= 1 and rel.parts[0] == GALLERY_SUBDIR
        text = md_path.read_text(encoding="utf-8")
        for match in ANCHOR_RE.finditer(text):
            stem = match.group(1)
            all_anchor_tokens[stem] += 1
            if in_gallery:
                gallery_anchors[stem] += 1
        for match in LINK_RE.finditer(text):
            stem = match.group(1)
            all_link_tokens[stem] += 1
            if not in_gallery:
                crosslinks[stem] += 1
    return gallery_anchors, crosslinks, all_anchor_tokens, all_link_tokens


def _dump(label: str, items: list[str]) -> None:
    if not items:
        return
    print(f"ERROR: gui-form-xref: {len(items)} {label}:", file=sys.stderr)
    for line in items[:MAX_REPORT_LINES]:
        print(f"  {line}", file=sys.stderr)
    if len(items) > MAX_REPORT_LINES:
        print(f"  ... and {len(items) - MAX_REPORT_LINES} more", file=sys.stderr)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--transcripts-gui",
        type=Path,
        required=True,
        help="Path to docs/manual-gui/transcripts/gui (the canonical stem list).",
    )
    ap.add_argument(
        "--src-dir",
        type=Path,
        required=True,
        help="Path to docs/manual-gui/src.",
    )
    args = ap.parse_args()

    if not args.transcripts_gui.is_dir():
        print(
            f"ERROR: --transcripts-gui not a directory: {args.transcripts_gui}",
            file=sys.stderr,
        )
        return 2
    if not args.src_dir.is_dir():
        print(
            f"ERROR: --src-dir not a directory: {args.src_dir}",
            file=sys.stderr,
        )
        return 2

    stems = canonical_stems(args.transcripts_gui)
    if not stems:
        print(
            f"ERROR: no *.gui files under {args.transcripts_gui}",
            file=sys.stderr,
        )
        return 2
    stem_set = set(stems)

    gallery_anchors, crosslinks, all_anchor_tokens, all_link_tokens = scan(
        args.src_dir
    )

    missing_anchors: list[str] = []
    dup_anchors: list[str] = []
    missing_links: list[str] = []
    dup_links: list[str] = []
    for stem in stems:
        n_anchor = gallery_anchors.get(stem, 0)
        if n_anchor == 0:
            missing_anchors.append(
                f"missing: {{#gui-form-{stem}}} (expected 1 in src/{GALLERY_SUBDIR}/)"
            )
        elif n_anchor > 1:
            dup_anchors.append(f"{{#gui-form-{stem}}} appears {n_anchor}x (expected 1)")
        n_link = crosslinks.get(stem, 0)
        if n_link == 0:
            missing_links.append(
                f"missing: ](#gui-form-{stem}) (expected 1 outside src/{GALLERY_SUBDIR}/)"
            )
        elif n_link > 1:
            dup_links.append(f"](#gui-form-{stem}) appears {n_link}x (expected 1)")

    orphan_anchors = [
        f"{{#gui-form-{stem}}} maps to no .gui stem"
        for stem in sorted(all_anchor_tokens)
        if stem not in stem_set
    ]
    orphan_links = [
        f"](#gui-form-{stem}) maps to no .gui stem"
        for stem in sorted(all_link_tokens)
        if stem not in stem_set
    ]

    total = (
        len(missing_anchors)
        + len(dup_anchors)
        + len(missing_links)
        + len(dup_links)
        + len(orphan_anchors)
        + len(orphan_links)
    )
    if total:
        _dump("gallery anchor(s) missing", missing_anchors)
        _dump("cross-link(s) missing", missing_links)
        _dump("gallery anchor(s) duplicated", dup_anchors)
        _dump("cross-link(s) duplicated", dup_links)
        _dump("orphan gallery anchor(s)", orphan_anchors)
        _dump("orphan cross-link(s)", orphan_links)
        return 1

    print(
        f"OK: gui-form-xref: {len(stems)} forms each with 1 gallery anchor "
        f"+ 1 cross-link (0 duplicates, 0 orphans)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
