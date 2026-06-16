import sys

K = {}
for line in open(".examples-build/keys.env"):
    line = line.strip()
    if not line: continue
    n, val = line.split("=", 1)
    K[n] = val + "/<0;1>/*"

H = open(".examples-build/H.txt").read().strip()

def ma(k, keys): return f"multi_a({k}," + ",".join(keys) + ")"

# tapscript leaves (multi_a, not multi)
L1 = f"and_v(v:after(1000000),and_v(v:sha256({H}),{ma(3,[K['K0'],K['K1'],K['K2']])}))"
L2 = f"and_v(v:after(1893456000),and_v(v:sha256({H}),{ma(2,[K['K3'],K['K4'],K['K5']])}))"
L3 = f"and_v(v:older(65535),{ma(2,[K['K6'],K['K7']])})"
L4 = f"and_v(v:older({0x400000 | 61594}),{ma(1,[K['K8'],K['K9'],K['K10']])})"

variant = sys.argv[1] if len(sys.argv) > 1 else "2leaf"
if variant == "1leaf":      # single tap leaf: or_i chain
    tree = f"or_i({L1},or_i({L2},or_i({L3},{L4})))"
elif variant == "2leaf":    # depth-1: two leaves, two tiers each
    tree = f"{{or_i({L1},{L2}),or_i({L3},{L4})}}"
elif variant == "4leaf":    # depth-2: one tier per leaf (balanced)
    tree = f"{{{{{L1},{L2}}},{{{L3},{L4}}}}}"
sys.stdout.write(f"tr(NUMS,{tree})")
