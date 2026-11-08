#!/usr/bin/env python3
"""design/spec-vendor-freshness/fill.py -- authoring helper: rewrite every
`<!-- vf:<label> -->` block of design/SPEC_vendor_freshness_shared.md from a
gate.sh log directory (python3 -I fill.py <logs>). It extracts exactly what
gate.sh compares, so a gate run after a fill checks the fill. It does not
decide anything: a block whose source log is missing stops the fill.
NOT covered: prose outside the blocks."""
import hashlib, os, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))
SPEC = os.path.join(HERE, "..", "SPEC_vendor_freshness_shared.md")
PROTO = os.path.join(HERE, "proto")
LOG = sys.argv[1]


def read(p):
    with open(p, encoding="utf-8") as f:
        return f.read()


def test_rows(raw):
    keep = re.compile(r"^(subjects|ok   - |skip - |FAIL - |vendor-freshness.test:|exit )")
    return [l for l in raw.splitlines() if keep.match(l)]


def refusal(repo, rid):
    out, on = [], False
    for l in read(f"{LOG}/test-{repo}.raw").splitlines():
        if re.match(r"^(ok   - |FAIL - |skip - )", l):
            on = l.find("- " + rid + " ") == 5
            continue
        if l.startswith("vendor-freshness.test:"):
            on = False
        if on and l.startswith("       | "):
            body = l[9:]
            if re.match(r"^(::error::|  - |      )", body):
                out.append(body)
    if not out:
        sys.exit(f"fill: no refusal lines for {repo} {rid}")
    return out


def red(repo):
    got = []
    for l in read(f"{LOG}/red-{repo}.raw").splitlines():
        m = re.match(r"^FAIL - ([AG][0-9]+) .*\(exit ([01]); wanted ([01]);", l)
        if m and m.group(2) != m.group(3):
            got.append(f"{m.group(1)}:{m.group(2)}")
    return [" ".join(got)]


def mutant(mid):
    ids = re.findall(r"^FAIL - ([AG][0-9]+) ", read(f"{LOG}/mut-{mid}.raw"), re.M)
    if not ids:
        sys.exit(f"fill: mutant {mid} caused no FAIL row")
    return [" ".join(ids)]


def sha():
    return [f"{hashlib.sha256(open(os.path.join(PROTO, f), 'rb').read()).hexdigest()}  {f}"
            for f in ("vendor-freshness.sh", "vendor-freshness.test.sh")]


def content(label):
    m = re.match(r"^(data|green|test|red)-(.+)$", label)
    if m and m.group(1) == "data":
        return read(f"{PROTO}/data/{m.group(2)}.txt").splitlines()
    if m and m.group(1) == "green":
        return read(f"{LOG}/green-{m.group(2)}.txt").splitlines()
    if m and m.group(1) == "test":
        return test_rows(read(f"{LOG}/test-{m.group(2)}.raw"))
    if m and m.group(1) == "red":
        return red(m.group(2))
    m = re.match(r"^refusal-(.+)-([AG][0-9]+)$", label)
    if m:
        return refusal(m.group(1), m.group(2))
    m = re.match(r"^mutant-(M[0-9]+)$", label)
    if m:
        return mutant(m.group(1))
    if label == "proto-sha256":
        return sha()
    sys.exit(f"fill: unknown block label {label}")


lines = read(SPEC).split("\n")
out, i = [], 0
while i < len(lines):
    out.append(lines[i])
    m = re.match(r"^<!-- vf:(.+) -->$", lines[i])
    if not m:
        i += 1
        continue
    j = i + 1
    assert lines[j].startswith("```"), (i, lines[j])
    k = j + 1
    while not lines[k].startswith("```"):
        k += 1
    out.append(lines[j])
    out.extend(content(m.group(1)))
    out.append(lines[k])
    i = k + 1
open(SPEC, "w", encoding="utf-8").write("\n".join(out))
print("fill: done")
