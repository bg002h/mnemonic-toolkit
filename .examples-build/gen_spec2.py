import json, sys

K = {}
for line in open(".examples-build/keys.env"):
    line = line.strip()
    if not line: continue
    name, val = line.split("=", 1)
    K[name] = val

H = "a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad"

def v(sub): return {"wrap": {"w": "v", "sub": sub}}
def older(n): return {"older": n}
def after(n): return {"after": n}
def sha(h): return {"sha256": h}
def multi(k, keys): return {"multi": {"k": k, "keys": keys}}
def and_v(a, b): return {"and_v": [a, b]}
def or_i(a, b): return {"or_i": [a, b]}

# Leaf 1 - ABSOLUTE block height + 3-of-3 + secret word
P1 = and_v(v(after(1000000)), and_v(v(sha(H)), multi(3, [K["K0"], K["K1"], K["K2"]])))
# Leaf 2 - ABSOLUTE time (unix) + 2-of-3 + secret word
P2 = and_v(v(after(1893456000)), and_v(v(sha(H)), multi(2, [K["K3"], K["K4"], K["K5"]])))
# Leaf 3 - RELATIVE blocks + both keys
P3 = and_v(v(older(65535)), multi(2, [K["K6"], K["K7"]]))
# Leaf 4 - RELATIVE time (bit22 | 61594 units ~= 365 days) + any 1 of 3
P4 = and_v(v(older(0x400000 | 61594)), multi(1, [K["K8"], K["K9"], K["K10"]]))

root = or_i(P1, or_i(P2, or_i(P3, P4)))
doc = {"schema_version": 1, "wrapper": "wsh", "root": root}
json.dump(doc, sys.stdout, indent=2)
print()
