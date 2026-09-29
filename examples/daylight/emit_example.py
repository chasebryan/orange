"""Emit a self-contained Orange Horizon encryption vector; public fixture only."""
from pathlib import Path
import json
from daylight import CORE, IV, array, literals, packed

MAGIC=b'DLTHV1A'
ROOT=bytes(range(32))
PLAINTEXT=b'Daylight Horizon runs in Orange.'
HEADER={'nonce':'000000000000000000000001',
        'authorization':{'authorization_tag':'0123456789abcdef'*4},
        'plaintext_len':len(PLAINTEXT)}


class Builder:
    def __init__(self):
        self.lines=[]
        self.n=0
    def bind(self,ty,expr):
        name=f'v{self.n}'; self.n+=1
        self.lines.append(f'    let {name}: {ty} = {expr};')
        return name
    def sha(self,data):
        n=len(data)
        data=list(data)+['0x80']+['0']*((55-n)%64)+literals((n*8).to_bytes(8,'big'))
        h=array(IV)
        for i in range(0,len(data),64):
            h=self.bind('Word[32]^8',f'sha_compress({h},{packed(data[i:i+64],"big")})')
        return byte_exprs(h,32,'big')
    def hmac(self,key,data):
        key=list(key)+['0']*(64-len(key))
        inner=self.sha([f'({x} ^ 0x36)' for x in key]+list(data))
        return self.sha([f'({x} ^ 0x5c)' for x in key]+inner)


def byte_exprs(name,size,endian='little'):
    parts=[]
    for i in range(size):
        shift=8*(i%4 if endian=='little' else 3-i%4)
        w=f'{name}[{i//4}]'
        parts.append(f'(({w} >> {shift}) as Word[8])' if shift else f'({w} as Word[8])')
    return parts


def render():
    header=json.dumps(HEADER,sort_keys=True,separators=(',',':'),ensure_ascii=True).encode()
    prefix=MAGIC+len(header).to_bytes(4,'little')+header
    info=b'DAYLIGHT-HORIZON-ALPHA-AEAD-KEY:'+MAGIC+b':'+HEADER['authorization']['authorization_tag'].encode()
    b=Builder()
    salt=b.sha(literals(MAGIC+header))
    prk=b.hmac(salt,[f'root[{i}]' for i in range(32)])
    key=b.hmac(prk,literals(info+b'\x01'))
    k=b.bind('Word[32]^8',packed(key))
    nonce=packed(literals(bytes.fromhex(HEADER['nonce'])))
    stream=b.bind('Word[32]^16',f'chacha_block({k},1,{nonce})')
    c=b.bind(f'Word[8]^{len(PLAINTEXT)}',array([f'p[{i}] ^ {e}' for i,e in enumerate(byte_exprs(stream,len(PLAINTEXT)))]))
    otk=b.bind('Word[32]^16',f'chacha_block({k},0,{nonce})')
    r=b.bind('Word[64]^5',f'poly_r({array([f"{otk}[{i}]" for i in range(4)])})')
    mac=literals(prefix)+['0']*(-len(prefix)%16)+[f'{c}[{i}]' for i in range(len(PLAINTEXT))]+['0']*(-len(PLAINTEXT)%16)
    mac+=literals(len(prefix).to_bytes(8,'little')+len(PLAINTEXT).to_bytes(8,'little'))
    h='[0,0,0,0,0]'
    for i in range(0,len(mac),16):
        h=b.bind('Word[64]^5',f'poly_step({h},{r},{packed(mac[i:i+16])},0x1000000)')
    tag=b.bind('Word[32]^4',f'poly_finish({h},{array([f"{otk}[{i}]" for i in range(4,8)])})')
    result=literals(prefix)+[f'{c}[{i}]' for i in range(len(PLAINTEXT))]+byte_exprs(tag,16)
    if len(result)>256: raise ValueError('fixture exceeds Orange array capacity')
    source=CORE.read_text().rstrip()[:-1]
    source+='\n  // Fixed public test context only. Real Horizon uses the adapter and fresh nonces.\n'
    source+=f'  spec seal_fixture(root: Word[8]^32, p: Word[8]^{len(PLAINTEXT)}) -> Word[8]^{len(result)} {{\n'
    source+='\n'.join(b.lines)+'\n    '+array(result)+'\n  }\n'
    source+=f'  spec example() -> Word[8]^{len(result)} {{ seal_fixture({array(literals(ROOT))},{array(literals(PLAINTEXT))}) }}\n}}\n'
    return source


if __name__=='__main__':
    Path(__file__).with_name('daylight-horizon.or').write_text(render())
