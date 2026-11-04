#!/usr/bin/env python3
"""Counts for FIX-V5 STOP-1 from corpus-rows-d-final.tsv (stdlib only).

Columns: status, class (free|thresh), rows, L (largest wsh row, WU),
mw (max_weight_to_satisfy, WU), flag, shape. Rows are de-duplicated by shape.
Prints counts and the shapes that break grade ruling 6 only. Does NOT cover:
how the TSV was produced (corpus-scratch.patch, run with V5a applied) or Core.
"""
import collections, pathlib, sys

tsv = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else
                   pathlib.Path(__file__).with_name("corpus-rows-d-final.tsv"))
seen, status = {}, collections.Counter()
for line in tsv.read_text().splitlines():
    f = line.split("\t")
    status[f[0]] += 1
    if f[0] == "OK":
        seen.setdefault(f[6], (f[1], int(f[3]), int(f[4])))
free = {s: v for s, v in seen.items() if v[0] == "free"}
thr = {s: v for s, v in seen.items() if v[0] == "thresh"}
bad_free = {s: v for s, v in free.items() if v[1] != v[2] + 1}
print(f"lines {sum(status.values())} by status {dict(status)}; unique OK shapes {len(seen)}")
print(f"thresh-free {len(free)}: L == mw+1 {len(free) - len(bad_free)}, L != mw+1 {len(bad_free)}")
for s, (_, l, mw) in sorted(bad_free.items()):
    print(f"  free  L={l} mw={mw} L-(mw+1)={l - mw - 1:+d}  {s}")
off = collections.Counter(l - mw - 1 for _, l, mw in thr.values())
print(f"thresh {len(thr)}: offsets L-(mw+1) {dict(sorted(off.items()))}")
for s, (_, l, mw) in sorted(thr.items()):
    if l > mw + 1:
        print(f"  thresh L={l} mw={mw} L-(mw+1)={l - mw - 1:+d}  {s}")
