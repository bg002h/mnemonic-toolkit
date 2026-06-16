import sys

K = {}
for line in open(".examples-build/keys.env"):
    line = line.strip()
    if not line: continue
    n, val = line.split("=", 1)
    K[n] = val + "/<0;1>/*"   # append the multipath suffix per key

H = "a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad"

def multi(k, keys): return f"multi({k}," + ",".join(keys) + ")"

P1 = f"and_v(v:after(1000000),and_v(v:sha256({H}),{multi(3,[K['K0'],K['K1'],K['K2']])}))"
P2 = f"and_v(v:after(1893456000),and_v(v:sha256({H}),{multi(2,[K['K3'],K['K4'],K['K5']])}))"
P3 = f"and_v(v:older(65535),{multi(2,[K['K6'],K['K7']])})"
P4 = f"and_v(v:older({0x400000 | 61594}),{multi(1,[K['K8'],K['K9'],K['K10']])})"

desc = f"wsh(or_i({P1},or_i({P2},or_i({P3},{P4}))))"
sys.stdout.write(desc)
