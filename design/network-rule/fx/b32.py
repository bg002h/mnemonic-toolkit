import hashlib,hmac,sys
P=2**256-2**32-977;N=0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
G=(0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798,0x483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8)
def add(a,b):
    if a is None:return b
    if b is None:return a
    if a[0]==b[0] and (a[1]+b[1])%P==0:return None
    l=(3*a[0]*a[0]*pow(2*a[1],-1,P) if a==b else (b[1]-a[1])*pow(b[0]-a[0],-1,P))%P
    x=(l*l-a[0]-b[0])%P;return (x,(l*(a[0]-x)-a[1])%P)
def mul(k,p=G):
    r=None
    while k:
        if k&1:r=add(r,p)
        p=add(p,p);k>>=1
    return r
def ser(p):return bytes([2+(p[1]&1)])+p[0].to_bytes(32,'big')
B58='123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz'
def b58c(b):
    b+=hashlib.sha256(hashlib.sha256(b).digest()).digest()[:4];n=int.from_bytes(b,'big');s=''
    while n:n,r=divmod(n,58);s=B58[r]+s
    return '1'*(len(b)-len(b.lstrip(b'\0')))+s
def h160(b):return hashlib.new('ripemd160',hashlib.sha256(b).digest()).digest()
def xpub(words,path,ver):
    seed=hashlib.pbkdf2_hmac('sha512',words.encode(),b'mnemonic',2048)
    I=hmac.new(b'Bitcoin seed',seed,'sha512').digest();k=int.from_bytes(I[:32],'big');c=I[32:]
    fp=h160(ser(mul(k)))[:4];pfp=b'\0'*4;depth=0;idx=0
    for s in path.split('/'):
        hard=s.endswith("'");i=int(s.rstrip("'"))+(0x80000000 if hard else 0)
        data=(b'\0'+k.to_bytes(32,'big') if hard else ser(mul(k)))+i.to_bytes(4,'big')
        I=hmac.new(c,data,'sha512').digest();pfp=h160(ser(mul(k)))[:4]
        k=(int.from_bytes(I[:32],'big')+k)%N;c=I[32:];depth+=1;idx=i
    x=bytes.fromhex(ver)+bytes([depth])+pfp+idx.to_bytes(4,'big')+c+ser(mul(k))
    return fp.hex(),b58c(x)
if __name__=='__main__':
    w,path,ver=sys.argv[1:4];fp,x=xpub(w,path,ver);print(f"[{fp}/{path.replace('m/','')}]{x}")
