#!/usr/bin/env python3
"""Lint phase: cli-pin-consistency (F-679 fold 1, review I-1).

One source for the CLI versions the GUI manual names: the toolkit installer.

  (a) docs/manual-gui/pinned-upstream.toml's four [manual-gui]
      *-tag-implied tags, and its [mnemonic-gui] tag, must EQUAL the pins in
      scripts/install.sh's component_info table. The installer installs the
      GUI together with these CLIs, so a manual pinned to other CLIs documents
      a combination nobody installs (F-679: ms 0.19.0 broke the GUI's
      `ms verify --phrase`).
      ONE declared exception: a CLI release can ship before the GUI re-pins to
      it. The manual then keeps documenting the GUI's tier, and the file's
      [installer-ahead] table names the installer's newer tag under the same
      key (e.g. `ms-cli-tag-implied = "ms-cli-v0.20.0"`). Each entry must equal
      the installer's pin exactly and be strictly NEWER than the manual's tag,
      and an entry for a CLI the installer does not run ahead on is stale. The
      GUI tag itself can never be ahead. While any entry exists, the manual
      must carry the `{#installer-ahead}` section that tells the reader, and
      it must not carry it otherwise.
  (b) Every CLI version the manual's prose names -- `mnemonic-toolkit-v0.13.0`,
      `ms-cli v0.2.1`, `Pinned: md 0.11.0`, `toolkit 0.104.0`, ... -- must
      equal that CLI's pin in pinned-upstream.toml (or its [installer-ahead]
      tag), UNLESS the exact line is listed in
      tests/cli-version-history.txt as history ("since ms-cli v0.14.0 …",
      the release-history appendix). A pin bump therefore fails on every
      sentence that stated the old pin, and each one must be updated or
      consciously moved to the history list.
  (c) Every history-list entry must still match a line (no stale entries).

Exits 0 when clean, 1 with one line per finding, 2 on usage error.
"""
from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path

CLIS = ("mnemonic", "md", "ms", "mk")
INSTALL_PKG = {"mnemonic": "mnemonic-toolkit", "md": "md-cli", "ms": "ms-cli", "mk": "mk-cli"}
IMPLIED_KEY = {"mnemonic": "toolkit-tag-implied", "md": "md-cli-tag-implied",
               "ms": "ms-cli-tag-implied", "mk": "mk-cli-tag-implied"}

# A CLI name immediately followed by a version. Order matters: longer forms
# first so `descriptor-mnemonic-md-cli-v` is not read as `md-cli-v`.
V = r"(?P<ver>\d+\.\d+(?:\.\d+)?)"
MENTION = re.compile("|".join([
    r"(?P<mnemonic>(?:mnemonic-toolkit-v|mnemonic-toolkit v|\btoolkit v?|Pinned: mnemonic |\bmnemonic v|`mnemonic )" + V.replace("ver", "v1") + ")",
    r"(?P<md>(?:descriptor-mnemonic-md-cli-v|descriptor-mnemonic-md-cli v|\bmd-cli v?|Pinned: md |\bmd v|`md )" + V.replace("ver", "v2") + ")",
    r"(?P<ms>(?:\bms-cli-v|\bms-cli v?|Pinned: ms |\bms v|`ms |\bms (?=\d))" + V.replace("ver", "v3") + ")",
    r"(?P<mk>(?:\bmk-cli-v|\bmk-cli v?|Pinned: mk |\bmk v|`mk )" + V.replace("ver", "v4") + ")",
]))


def install_pins(install_sh: Path) -> dict[str, str]:
    """{cli: version, 'gui': tag} from install.sh's component_info arms."""
    pins: dict[str, str] = {}
    for m in re.finditer(r'echo "([a-z-]+)\|https://[^|"]+\|([^|"]+)\|[^|"]*\|[^"]*"', install_sh.read_text()):
        pkg, tag = m.group(1), m.group(2)
        for cli, p in INSTALL_PKG.items():
            if pkg == p:
                pins[cli] = tag
        if pkg == "mnemonic-gui":
            pins["gui"] = tag
    return pins


def version_of(tag: str) -> str:
    return tag.rsplit("-v", 1)[1]


def version_key(tag: str) -> tuple[int, ...]:
    return tuple(int(x) for x in version_of(tag).split("."))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--manual-dir", required=True, help="docs/manual-gui")
    ap.add_argument("--install-sh", required=True, help="scripts/install.sh")
    a = ap.parse_args()
    man = Path(a.manual_dir)
    errs: list[str] = []

    pins = install_pins(Path(a.install_sh))
    missing = [c for c in (*CLIS, "gui") if c not in pins]
    if missing:
        print(f"ERROR: cli-pin-consistency: cannot read pins for {missing} from {a.install_sh}", file=sys.stderr)
        return 2

    # (a) pinned-upstream.toml == installer, or a declared [installer-ahead]
    toml = tomllib.loads((man / "pinned-upstream.toml").read_text())
    ahead = toml.get("installer-ahead", {})
    for key in ahead:
        if key not in IMPLIED_KEY.values():
            errs.append(f"pinned-upstream.toml [installer-ahead] {key}: not one of {sorted(IMPLIED_KEY.values())}")
    manual_tag: dict[str, str] = {}
    for cli in CLIS:
        key = IMPLIED_KEY[cli]
        got = toml["manual-gui"][key]
        manual_tag[cli] = got
        if key in ahead:
            decl = ahead[key]
            if got == pins[cli]:
                errs.append(f"pinned-upstream.toml [installer-ahead] {key} = {decl} is stale: install.sh pins {pins[cli]}, "
                            f"the same as [manual-gui]; remove the entry")
            elif decl != pins[cli]:
                errs.append(f"pinned-upstream.toml [installer-ahead] {key} = {decl}, but install.sh pins {pins[cli]}")
            elif version_key(decl) <= version_key(got):
                errs.append(f"pinned-upstream.toml [installer-ahead] {key} = {decl} is not newer than [manual-gui] {got}")
        elif got != pins[cli]:
            errs.append(f"pinned-upstream.toml [manual-gui] {key} = {got}, but install.sh pins {pins[cli]}")
    if toml["mnemonic-gui"]["tag"] != pins["gui"]:
        errs.append(f"pinned-upstream.toml [mnemonic-gui] tag = {toml['mnemonic-gui']['tag']}, but install.sh pins {pins['gui']}")

    # (b) prose mentions
    hist_file = man / "tests" / "cli-version-history.txt"
    history: dict[tuple[str, str], bool] = {}
    for raw in hist_file.read_text().splitlines():
        if not raw.strip() or raw.startswith("#"):
            continue
        path, _, line = raw.partition("\t")
        history[(path, line)] = False
    want = {cli: version_of(manual_tag[cli]) for cli in CLIS}
    also = {cli: version_of(ahead[IMPLIED_KEY[cli]]) for cli in CLIS if IMPLIED_KEY[cli] in ahead}
    files = sorted([*man.glob("src/**/*.md"), *man.glob("tutorial/*.md")])
    checked = 0
    ahead_section = []
    for f in files:
        rel = f.relative_to(man).as_posix()
        for n, line in enumerate(f.read_text().splitlines(), 1):
            if "{#installer-ahead}" in line:
                ahead_section.append(f"{rel}:{n}")
            for m in MENTION.finditer(line):
                cli = next(c for c in CLIS if m.group(c))
                ver = next(v for v in (m.group("v1"), m.group("v2"), m.group("v3"), m.group("v4")) if v)
                checked += 1
                if ver == want[cli] or ver == also.get(cli):
                    continue
                key = (rel, line)
                if key in history:
                    history[key] = True
                    continue
                errs.append(f"{rel}:{n}: names {cli} {ver} ({m.group(0).strip()!r}), but the pin is {want[cli]}; "
                            f"update it, or if the line is history, add it to tests/cli-version-history.txt")
    if ahead and len(ahead_section) != 1:
        errs.append(f"[installer-ahead] is declared, so the manual needs exactly one {{#installer-ahead}} section "
                    f"telling the reader; found {len(ahead_section)} {ahead_section}")
    if not ahead and ahead_section:
        errs.append(f"{ahead_section[0]}: an {{#installer-ahead}} section, but pinned-upstream.toml declares no "
                    f"[installer-ahead]; remove the section")
    # (c) stale history entries
    for (path, line), used in history.items():
        if not used:
            errs.append(f"tests/cli-version-history.txt: entry matches no line naming an off-pin version: {path}: {line[:80]!r}")

    if errs:
        print(f"ERROR: cli-pin-consistency: {len(errs)} finding(s):", file=sys.stderr)
        for e in errs:
            print(f"  {e}", file=sys.stderr)
        return 1
    ahead_note = "".join(f"; installer ahead: {c} {also[c]}" for c in CLIS if c in also)
    print(f"OK: cli-pin-consistency: pinned-upstream.toml agrees with install.sh "
          f"({', '.join(f'{c} {want[c]}' for c in CLIS)}, {pins['gui']}{ahead_note}); "
          f"{checked} version mention(s) in {len(files)} files, {len(history)} history line(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
