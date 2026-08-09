"""Emit texture-programs.txt: the intro's actual texture programs plus a focused
disassembly of only the operators they use (7 of 26)."""
from analib import read, disasm_range
import struct, json
d=json.load(open('texprog_dump.json'))
entry=d['entry']; prog=d['prog']
raw=read(0x4429b8,8*30); hnd={}
for i in range(30):
    a,b=struct.unpack('<II',raw[i*8:i*8+8])
    if a==0: break
    hnd[a]=b
o=[]
o.append("Heaven 7 texture programs (extracted by emulation) + the operators they use")
o.append("programs live in .data at 0x41b6b1, cleartext, at the tail of the scene script")
o.append("interpreter .data:0x44189f   operator table .data:0x4429b8\n")
o.append("Texture object (allocated by .text:0x4088c9, width in EAX / height in EDX):")
o.append("   +0x18 width   +0x1c height   +0x20 stride=align8(w)*4   +0x24 buffer (4 B/px)\n")
for k,e in enumerate(entry):
    o.append(f"--- texture program {k} @ {e:#x} ---")
    f=[(p,x) for p,x in prog if p>=e and (k+1>=len(entry) or p<entry[k+1])]
    for j,(p,x) in enumerate(f):
        nxt=f[j+1][0] if j+1<len(f) else (entry[k+1] if k+1<len(entry) else p+1)
        args=read(p+1,max(nxt-p-1,0))
        o.append(f"  {p:#x}  op {x:#04x}  handler {hnd.get(x,0):#010x}  args {args.hex() or '-'}")
    o.append("")
used=sorted({x for _,x in prog})
o.append(f"OPERATORS USED: {[hex(u) for u in used]}  ({len(used)} of {len(hnd)})")
o.append("op 0x01 terminates every program (handler shared by ids 0x01-0x04).")
o.append("Args commonly begin 0x01/0x02 then 0x07 - reads as (target, channel-mask=RGB).")
o.append("")
o.append("="*72)
o.append("Focused disassembly: only the operators the intro uses")
o.append("="*72)
starts=sorted(set(hnd.values()))
for u in used:
    h=hnd[u]; nx=[s for s in starts if s>h]
    size=min((nx[0]-h) if nx else 0x100, 0x140)
    o.append(f"\n; ===== operator {u:#04x}  handler {h:#010x} =====")
    for i in disasm_range(h,size):
        if i.id==0: break
        b=' '.join('%02x'%y for y in i.bytes)
        o.append('%08x  %-20s %-9s %s'%(i.address,b,i.mnemonic,i.op_str))
open('texture-programs.txt','w').write('\n'.join(o)+'\n')
print('\n'.join(o[:6])); print('...\nlines:',len(o))
