"""Turn the emulator's script_dump.json into a readable scene-script listing."""
import json, collections, struct
from analib import read
d=json.load(open('script_dump.json'))
ops=d['ops']; toks=d['toks']
import struct as st
TBL=st.unpack('<40H', read(0x440f67,80))
out=[]
out.append("Heaven 7 scene script  (stream base .data:0x418dca, VM at .text:0x40a60e)")
out.append("dispatch: handler = 0x400000 + word[0x440f67 + opcode*2]")
out.append(f"{len(ops)} opcodes executed, {len(toks)} decoder tokens\n")
hist=collections.Counter(o['op'] for o in ops)
out.append("opcode histogram (op: count -> handler):")
for op,c in hist.most_common():
    h=0x400000+TBL[op] if op < len(TBL) else 0
    out.append(f"   op {op:<3} x{c:<4} handler {h:#010x}")
out.append("")
out.append("listing (offset  opcode  consumed-bytes  decoded args):")
for i,o in enumerate(ops):
    lo=o['ntok']; hi=ops[i+1]['ntok'] if i+1<len(ops) else len(toks)
    nxt=ops[i+1]['p'] if i+1<len(ops) else o['p']
    span=nxt-o['p']
    args=toks[lo:hi]
    def f(t):
        v=t['v']
        if v is None: return '?'
        return (f"{v:.5g}" if t['k']=='f' else str(v))
    a=' '.join(('f' if t['k']=='f' else 'i')+':'+f(t) for t in args)
    h=0x400000+TBL[o['op']] if o['op']<len(TBL) else 0
    out.append(f"  {o['p']:#08x}  op{o['op']:<3} h={h:#08x} span={span:<6} {a}")
open('scene-script.txt','w').write('\n'.join(out)+'\n')
print('\n'.join(out[:8]))
print('...')
print('lines:',len(out))
