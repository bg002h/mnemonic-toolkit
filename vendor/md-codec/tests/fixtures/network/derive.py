#!/usr/bin/env python3
"""Derive the F-672 non-mainnet goldens WITHOUT md-codec's network code.

Input: a MAINNET descriptor md rendered (mainnet output is unchanged by F-672,
and proven byte-identical before/after on the whole corpus). Output: the same
descriptor under test-network version bytes. Every `xpub` token is base58check
decoded, its 4 version bytes replaced by 0x043587CF (BIP-32 testnet public;
signet and regtest share it), and re-encoded; the BIP-380 checksum is then
recomputed with the reference algorithm. Nothing else in the text may change.

usage: derive.py < mainnet-descriptor > test-network-descriptor
"""
import hashlib, re, sys

B58 = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"

def b58decode(s):
    n = 0
    for c in s:
        n = n * 58 + B58.index(c)
    raw = n.to_bytes((n.bit_length() + 7) // 8, "big")
    raw = b"\0" * (len(s) - len(s.lstrip("1"))) + raw
    body, check = raw[:-4], raw[-4:]
    assert hashlib.sha256(hashlib.sha256(body).digest()).digest()[:4] == check, s
    return body

def b58encode(body):
    raw = body + hashlib.sha256(hashlib.sha256(body).digest()).digest()[:4]
    n, out = int.from_bytes(raw, "big"), ""
    while n:
        n, r = divmod(n, 58)
        out = B58[r] + out
    return "1" * (len(raw) - len(raw.lstrip(b"\0"))) + out

def to_tpub(m):
    body = b58decode(m.group(0))
    assert body[:4] == bytes.fromhex("0488B21E") and len(body) == 78
    return b58encode(bytes.fromhex("043587CF") + body[4:])

# BIP-380 reference checksum.
INPUT_CHARSET = "0123456789()[],'/*abcdefgh@:$%{}IJKLMNOPQRSTUVWXYZ&+-.;<=>?!^_|~ijklmnopqrstuvwxyzABCDEFGH`#\"\\ "
CHECKSUM_CHARSET = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"
GENERATOR = [0xf5dee51989, 0xa9fdca3312, 0x1bab10e32d, 0x3706b1677a, 0x644d626ffd]

def polymod(c, val):
    c0 = c >> 35
    c = ((c & 0x7ffffffff) << 5) ^ val
    for i in range(5):
        if (c0 >> i) & 1:
            c ^= GENERATOR[i]
    return c

def checksum(s):
    c, cls, clscount = 1, 0, 0
    for ch in s:
        pos = INPUT_CHARSET.find(ch)
        assert pos != -1, ch
        c = polymod(c, pos & 31)
        cls = cls * 3 + (pos >> 5)
        clscount += 1
        if clscount == 3:
            c = polymod(c, cls)
            cls, clscount = 0, 0
    if clscount:
        c = polymod(c, cls)
    for _ in range(8):
        c = polymod(c, 0)
    c ^= 1
    return "".join(CHECKSUM_CHARSET[(c >> (5 * (7 - i))) & 31] for i in range(8))

main = sys.stdin.read().strip()
body, _, cs = main.partition("#")
assert checksum(body) == cs, "input checksum does not verify"
out = re.sub(r"xpub[1-9A-HJ-NP-Za-km-z]{100,112}", to_tpub, body)
assert "xpub" not in out
print(out + "#" + checksum(out))
