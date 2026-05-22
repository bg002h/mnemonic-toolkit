# Independent oracle: nostr x-only key -> BTC addresses. Pure Python, NO rust-bitcoin.
# secp256k1 + BIP-340 lift_x + BIP-341/86 taptweak + bech32/bech32m + base58check.
import hashlib

P = 2**256 - 2**32 - 977
Gx = 0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798
Gy = 0x483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8
G = (Gx, Gy)

def inv(a, m=P): return pow(a, m-2, m)
def pt_add(p1, p2):
    if p1 is None: return p2
    if p2 is None: return p1
    (x1,y1),(x2,y2)=p1,p2
    if x1==x2 and (y1+y2)%P==0: return None
    if p1==p2: lam=(3*x1*x1)*inv(2*y1)%P
    else: lam=(y2-y1)*inv(x2-x1)%P
    x3=(lam*lam-x1-x2)%P; y3=(lam*(x1-x3)-y1)%P
    return (x3,y3)
def pt_mul(k,pt):
    r=None
    while k:
        if k&1: r=pt_add(r,pt)
        pt=pt_add(pt,pt); k>>=1
    return r
def lift_x(x):
    y2=(pow(x,3,P)+7)%P
    y=pow(y2,(P+1)//4,P)
    if (y*y-y2)%P!=0: raise ValueError("not on curve")
    if y&1: y=P-y         # BIP-340: even y
    return (x,y)
def tagged_hash(tag, msg):
    t=hashlib.sha256(tag.encode()).digest()
    return hashlib.sha256(t+t+msg).digest()
def h160(b): return hashlib.new('ripemd160', hashlib.sha256(b).digest()).digest()

CHARSET="qpzry9x8gf2tvdw0s3jn54khce6mua7l"
def bech32_polymod(v):
    GEN=[0x3b6a57b2,0x26508e6d,0x1ea119fa,0x3d4233dd,0x2a1462b3]; chk=1
    for x in v:
        b=chk>>25; chk=((chk&0x1ffffff)<<5)^x
        for i in range(5): chk^=GEN[i] if (b>>i)&1 else 0
    return chk
def hrp_expand(h): return [ord(c)>>5 for c in h]+[0]+[ord(c)&31 for c in h]
def bech32_encode(hrp,data,const):
    vals=hrp_expand(hrp)+data
    pm=bech32_polymod(vals+[0]*6)^const
    chk=[(pm>>5*(5-i))&31 for i in range(6)]
    return hrp+'1'+''.join(CHARSET[d] for d in data+chk)
def convertbits(data,frm,to,pad=True):
    acc=0;bits=0;ret=[];maxv=(1<<to)-1
    for b in data:
        acc=(acc<<frm)|b;bits+=frm
        while bits>=to: bits-=to;ret.append((acc>>bits)&maxv)
    if pad and bits: ret.append((acc<<(to-bits))&maxv)
    return ret
def segwit_addr(hrp,witver,prog):
    const=1 if witver==0 else 0x2bc830a3  # bech32 vs bech32m
    return bech32_encode(hrp,[witver]+convertbits(list(prog),8,5),const)
B58="123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
def b58check(payload):
    chk=hashlib.sha256(hashlib.sha256(payload).digest()).digest()[:4]
    n=int.from_bytes(payload+chk,'big'); s=''
    while n>0: n,r=divmod(n,58); s=B58[r]+s
    s='1'*(len(payload+chk)-len((payload+chk).lstrip(b'\x00')))+s
    return s

x_hex="7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e"
x=int(x_hex,16)
comp=bytes([0x02])+bytes.fromhex(x_hex)   # even-y 02||x
hash160=h160(comp)
# P2PKH (0x00), P2WPKH (segwit v0), P2SH-P2WPKH (0x05 over redeem hash)
p2pkh=b58check(bytes([0x00])+hash160)
p2wpkh=segwit_addr("bc",0,hash160)
redeem=bytes([0x00,0x14])+hash160
p2sh=b58check(bytes([0x05])+h160(redeem))
# P2TR (BIP-86 key-path): Q = lift_x(x) + tagged_hash("TapTweak", x)*G
Pp=lift_x(x)
t=int.from_bytes(tagged_hash("TapTweak",bytes.fromhex(x_hex)),'big')
Q=pt_add(Pp, pt_mul(t,G))
p2tr=segwit_addr("bc",1,Q[0].to_bytes(32,'big'))
print("P2PKH ", p2pkh)
print("P2WPKH", p2wpkh)
print("P2SH  ", p2sh)
print("P2TR  ", p2tr)
