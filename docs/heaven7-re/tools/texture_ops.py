"""Reference disassembly of Heaven 7's texture-generator VM (lives in .data).

  interpreter : .data:0x44189f   (lodsb opcode -> linear search of operator table)
  op table    : .data:0x4429b8   8-byte entries {u32 id, u32 handler}, 0-terminated
  mask consts : .data:0x441840   MMX RGB565/555 channel masks
"""
from analib import disasm_range, read
import struct
raw=read(0x4429b8,8*40)
ops=[]
for i in range(40):
    a,b=struct.unpack('<II',raw[i*8:i*8+8])
    if a==0: break
    ops.append((a,b))
out=[]
out.append("Heaven 7 texture-generator VM  (all addresses in .data)")
out.append("interpreter: 0x44189f   operator table: 0x4429b8   masks: 0x441840")
out.append(f"{len(ops)} operators\n")
def dump(va,size,title):
    out.append(f"; ===== {title} =====")
    for i in disasm_range(va,size):
        if i.id==0: break
        b=' '.join('%02x'%x for x in i.bytes)
        out.append('%08x  %-20s %-9s %s'%(i.address,b,i.mnemonic,i.op_str))
    out.append("")
dump(0x44189f,0x4b,'interpreter: fetch opcode + table search')
# each handler: disassemble up to the next handler start
starts=sorted(set(h for _,h in ops))
for k,(oid,h) in enumerate(ops):
    nxt=[s for s in starts if s>h]
    size=min((nxt[0]-h) if nxt else 0x80, 0x120)
    dump(h,size,f'operator id {oid:#04x} ({oid})  handler {h:#010x}')
open('texture-ops.txt','w').write('\n'.join(out)+'\n')
print('\n'.join(out[:3])); print('lines:',len(out))
