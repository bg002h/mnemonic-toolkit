#!/usr/bin/env python3
"""Lint phase: tutorial-xref (phase 12).

Bidirectional embed parity for the gui_example tutorial book (SPEC §8).
The tutorial corpus is enumerated by the PINNED mnemonic-gui clone's
`tests/tutorial/manifest-stems.txt` — the single source of truth both
this repo's byte-gates (phases 10/11) and this xref gate read directly
(no toolkit copy to drift). Each committed corpus artifact MUST be
embedded in exactly one place in the tutorial chapters, and every embed
MUST resolve to a manifest artifact:

  - every `<stem>*.png` figure has EXACTLY ONE image embed
    `](../figures/tutorial/<name>.png)` across `tutorial/*.md`; and
  - every `<stem>.{stdout,stderr,exit}.txt` transcript has EXACTLY ONE
    include `include="tutorial/<name>"` across `tutorial/*.md`.

Reverse / orphan direction: every `figures/tutorial/*.png` image token
and every `include="tutorial/*"` token found in `tutorial/*.md` must map
to a manifest artifact.

This is the tutorial analogue of `check_gui_form_xref.py` (which keys on
the `.gui` stem set): it keys on the manifest artifact set instead.

Invocation:

    python3 check_tutorial_xref.py \\
        --manifest <path-to-pinned-clone/tests/tutorial/manifest-stems.txt> \\
        --tutorial-dir <path-to-docs/manual-gui/tutorial>

Exits 0 iff every artifact has exactly one embed and no orphan token
exists. Exits 1 on any missing / duplicated / orphan, naming the
offending artifact(s). Exits 2 on usage error.
"""

from __future__ import annotations

import argparse
import re
import sys
from collections import Counter
from pathlib import Path

MAX_REPORT_LINES = 80

# Image embeds pointing at the tutorial figure corpus (file-relative
# `../figures/tutorial/<name>.png` from tutorial/*.md).
IMG_RE = re.compile(r"figures/tutorial/([A-Za-z0-9._-]+\.png)")
# include-transcript.lua fenced includes: `include="tutorial/<name>"`.
INC_RE = re.compile(r'include="tutorial/([A-Za-z0-9._-]+)"')


def read_manifest(manifest: Path) -> tuple[set[str], set[str]]:
    """Return (png_artifacts, txt_artifacts) from manifest-stems.txt."""
    pngs: set[str] = set()
    txts: set[str] = set()
    for line in manifest.read_text(encoding="utf-8").splitlines():
        name = line.strip()
        if not name:
            continue
        if name.endswith(".png"):
            pngs.add(name)
        elif name.endswith(".txt"):
            txts.add(name)
        else:
            # Unknown artifact class in the manifest — fail loud.
            print(
                f"ERROR: tutorial-xref: manifest artifact is neither .png nor .txt: {name}",
                file=sys.stderr,
            )
            sys.exit(1)
    return pngs, txts


def scan(tutorial_dir: Path) -> tuple[Counter, Counter]:
    img_tokens: Counter = Counter()
    inc_tokens: Counter = Counter()
    for md_path in sorted(tutorial_dir.rglob("*.md")):
        text = md_path.read_text(encoding="utf-8")
        for m in IMG_RE.finditer(text):
            img_tokens[m.group(1)] += 1
        for m in INC_RE.finditer(text):
            inc_tokens[m.group(1)] += 1
    return img_tokens, inc_tokens


def _dump(label: str, items: list[str]) -> None:
    if not items:
        return
    print(f"ERROR: tutorial-xref: {len(items)} {label}:", file=sys.stderr)
    for line in items[:MAX_REPORT_LINES]:
        print(f"  {line}", file=sys.stderr)
    if len(items) > MAX_REPORT_LINES:
        print(f"  ... and {len(items) - MAX_REPORT_LINES} more", file=sys.stderr)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--manifest", type=Path, required=True,
                    help="Path to the pinned clone's tests/tutorial/manifest-stems.txt.")
    ap.add_argument("--tutorial-dir", type=Path, required=True,
                    help="Path to docs/manual-gui/tutorial.")
    args = ap.parse_args()

    if not args.manifest.is_file():
        print(f"ERROR: --manifest not a file: {args.manifest}", file=sys.stderr)
        return 2
    if not args.tutorial_dir.is_dir():
        print(f"ERROR: --tutorial-dir not a directory: {args.tutorial_dir}", file=sys.stderr)
        return 2

    pngs, txts = read_manifest(args.manifest)
    if not pngs and not txts:
        print(f"ERROR: tutorial-xref: no artifacts in {args.manifest}", file=sys.stderr)
        return 2

    img_tokens, inc_tokens = scan(args.tutorial_dir)

    missing_img: list[str] = []
    dup_img: list[str] = []
    for a in sorted(pngs):
        n = img_tokens.get(a, 0)
        if n == 0:
            missing_img.append(f"missing image embed for {a} (expected 1 in tutorial/*.md)")
        elif n > 1:
            dup_img.append(f"{a} embedded {n}x (expected 1)")

    missing_inc: list[str] = []
    dup_inc: list[str] = []
    for a in sorted(txts):
        n = inc_tokens.get(a, 0)
        if n == 0:
            missing_inc.append(f'missing include="tutorial/{a}" (expected 1 in tutorial/*.md)')
        elif n > 1:
            dup_inc.append(f'include="tutorial/{a}" appears {n}x (expected 1)')

    orphan_img = [f"figures/tutorial/{t} maps to no manifest artifact"
                  for t in sorted(img_tokens) if t not in pngs]
    orphan_inc = [f'include="tutorial/{t}" maps to no manifest artifact'
                  for t in sorted(inc_tokens) if t not in txts]

    total = (len(missing_img) + len(dup_img) + len(missing_inc)
             + len(dup_inc) + len(orphan_img) + len(orphan_inc))
    if total:
        _dump("figure embed(s) missing", missing_img)
        _dump("transcript include(s) missing", missing_inc)
        _dump("figure embed(s) duplicated", dup_img)
        _dump("transcript include(s) duplicated", dup_inc)
        _dump("orphan figure embed(s)", orphan_img)
        _dump("orphan transcript include(s)", orphan_inc)
        return 1

    print(
        f"OK: tutorial-xref: {len(pngs)} figures + {len(txts)} transcripts each "
        f"embedded exactly once (0 duplicates, 0 orphans)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
