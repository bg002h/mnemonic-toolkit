#!/usr/bin/env python3
"""Pre-render mermaid blocks to SHA-256-keyed PDFs for chromium-less builds.

Walks `--src-glob` under `--doc-root`, extracts every fenced ```mermaid```
block via re.MULTILINE | re.DOTALL, hashes its body verbatim with SHA-256,
and renders each block to `<cache-dir>/<sha>.pdf` via mermaid-cli (`mmdc -o
*.pdf`). Skips re-rendering when the cache hit exists (idempotent).

PDF output is xelatex-native: `\\includegraphics{<sha>.pdf}` embeds without
external converters (svg.sty needs inkscape, which we don't ship). The
spec originally locked .pdf; pre-PR Spike B revealed xelatex can't size
SVG without inkscape, so the cache settled on PDF instead.

Writes `<cache-dir>/cache-metadata.toml` with the `mmdc --version` and
the regen timestamp on success.

Pass `--verify` to skip rendering; the helper then exits 0 when every
source block has a cache entry and every cache entry has a source block,
or non-zero with a list of mismatches.

Used by `make figures-cache` (regen) and `make figures-cache-verify`
(lint-time consistency gate) in `docs/manual/Makefile` and
`docs/quickstart/Makefile`.

The byte-alignment contract with the Lua consumer (sha256.lua + the
mermaid-cache-filter.lua skip-mode filter) is: hash the literal text
between (and excluding) the ```mermaid and closing ``` fences, with no
leading or trailing newline appended. The regex captures up to but
excluding the final \\n before the closing fence.
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import pathlib
import re
import subprocess
import sys
import tempfile

MERMAID_FENCE_RE = re.compile(
    r"^```mermaid$\n(.*?)\n^```$",
    re.MULTILINE | re.DOTALL,
)


def extract_blocks(src_files: list[pathlib.Path]) -> list[tuple[pathlib.Path, str, str]]:
    """Return [(source_file, body, sha256_hex)] for every mermaid block found."""
    out: list[tuple[pathlib.Path, str, str]] = []
    for path in src_files:
        text = path.read_text(encoding="utf-8")
        for m in MERMAID_FENCE_RE.finditer(text):
            body = m.group(1)
            sha = hashlib.sha256(body.encode("utf-8")).hexdigest()
            out.append((path, body, sha))
    return out


def mmdc_version() -> str:
    """Return the running mmdc version string (one line)."""
    result = subprocess.run(
        ["mmdc", "--version"], capture_output=True, text=True, check=True
    )
    return result.stdout.strip()


def render_block(body: str, out_path: pathlib.Path) -> None:
    """Render a single mermaid block to PDF via mmdc.

    out_path is expected to end in `.pdf`; mmdc dispatches by extension.
    """
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".mmd", encoding="utf-8", delete=False
    ) as tf:
        tf.write(body)
        tf_path = tf.name
    try:
        subprocess.run(
            ["mmdc", "--input", tf_path, "--output", str(out_path), "--quiet"],
            check=True,
            capture_output=True,
        )
    finally:
        pathlib.Path(tf_path).unlink()


def write_metadata(cache_dir: pathlib.Path, version: str) -> None:
    timestamp = dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    (cache_dir / "cache-metadata.toml").write_text(
        f'mermaid_cli_version = "{version}"\n'
        f'last_regenerated = "{timestamp}"\n',
        encoding="utf-8",
    )


def regen(blocks, cache_dir: pathlib.Path) -> int:
    rendered = 0
    skipped = 0
    for src, body, sha in blocks:
        out_path = cache_dir / f"{sha}.pdf"
        if out_path.exists():
            skipped += 1
            continue
        print(f"  render {sha[:8]}…  ({src.name})", file=sys.stderr)
        try:
            render_block(body, out_path)
            rendered += 1
        except subprocess.CalledProcessError as e:
            print(
                f"ERROR: mmdc failed for block {sha} from {src}:\n{e.stderr.decode(errors='replace')}",
                file=sys.stderr,
            )
            return 2
    print(f"  rendered={rendered}, skipped (cache hit)={skipped}", file=sys.stderr)
    try:
        version = mmdc_version()
    except (FileNotFoundError, subprocess.CalledProcessError) as e:
        print(f"WARN: cannot read mmdc --version ({e}); metadata not written", file=sys.stderr)
        return 0
    write_metadata(cache_dir, version)
    return 0


def verify(blocks, cache_dir: pathlib.Path) -> int:
    expected = {sha for _, _, sha in blocks}
    on_disk = {p.stem for p in cache_dir.glob("*.pdf")}
    missing = expected - on_disk
    orphan = on_disk - expected
    if not missing and not orphan:
        return 0
    if missing:
        print(
            f"ERROR: {len(missing)} cache entries missing — regenerate via `make figures-cache`:",
            file=sys.stderr,
        )
        for sha in sorted(missing):
            sources = [str(s) for s, _, h in blocks if h == sha]
            print(f"  {sha}.pdf  (block from {sources})", file=sys.stderr)
    if orphan:
        print(
            f"ERROR: {len(orphan)} cache entries are orphaned (no source block):",
            file=sys.stderr,
        )
        for sha in sorted(orphan):
            print(f"  {sha}.pdf", file=sys.stderr)
    return 1


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("--doc-root", required=True, type=pathlib.Path)
    p.add_argument("--src-glob", required=True)
    p.add_argument("--cache-dir", required=True, type=pathlib.Path)
    p.add_argument("--verify", action="store_true")
    args = p.parse_args(argv)

    cache_dir = args.doc_root / args.cache_dir
    cache_dir.mkdir(parents=True, exist_ok=True)

    src_files = sorted(args.doc_root.glob(args.src_glob))
    blocks = extract_blocks(src_files)
    print(
        f"  {len(blocks)} mermaid block(s) across {len(src_files)} source file(s)",
        file=sys.stderr,
    )

    if args.verify:
        return verify(blocks, cache_dir)
    return regen(blocks, cache_dir)


if __name__ == "__main__":
    sys.exit(main())
