import json, sys

A = "[73c5da0a/87'/0'/0']xpub661MyMwAqRbcFrooZ2966EcDmVX5MoFXZhuJqXTudvJzwBTBfPQSc5JzX52fvS18oqSdEJXJ4kTGRJ76wPWDUSNJsY5JsgVBQoD6KrbdCLL"
B = "[b8688df1/87'/0'/0']xpub661MyMwAqRbcEnFgxHRLx7i1fnjcBPgc71qy8mVkbGXYukNGMK2XFRbAaCLYEJDUufNoBxTNa68i5MYhqmrEkfhjzgHCUEcvJBhXS5bk4RW"
C = "[28645006/87'/0'/0']xpub661MyMwAqRbcEdy4jr5EtEhQBctfscE6a99DGLr2cW4HnnmBsXDoe3odGzRiw3hcRM5wfKcQmb7s5FjdGrR6SrExXmeopaoY9Lk7tQusDjN"
H = "68100fc148a239c4bbf3e6d517a5f4831a803f0603ca834cf790b6703b17bc9d"

def v(sub): return {"wrap": {"w": "v", "sub": sub}}
def older(n): return {"older": n}
def sha(h): return {"sha256": h}
def multi(k, keys): return {"multi": {"k": k, "keys": keys}}
def and_v(a, b): return {"and_v": [a, b]}
def or_i(a, b): return {"or_i": [a, b]}

# 42 days, 63 days, 365 days at 144 blocks/day; 65535 explicit blocks.
D42, D63, D365, B65535 = 42*144, 63*144, 365*144, 65535  # 6048, 9072, 52560, 65535

# P1: after 42d AND 3-of-3 AND secret word
P1 = and_v(v(older(D42)), and_v(v(sha(H)), multi(3, [A, B, C])))
# P2: after 63d AND 2-of-3 AND secret word
P2 = and_v(v(older(D63)), and_v(v(sha(H)), multi(2, [A, B, C])))
# P3: after 65535 blocks AND keys 1 & 2 only
P3 = and_v(v(older(B65535)), multi(2, [A, B]))
# P4: after 365d AND any one of the three keys
P4 = and_v(v(older(D365)), multi(1, [A, B, C]))

root = or_i(P1, or_i(P2, or_i(P3, P4)))
doc = {"schema_version": 1, "wrapper": "wsh", "root": root}
json.dump(doc, sys.stdout, indent=2)
print()
