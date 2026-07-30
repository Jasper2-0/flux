#!/usr/bin/env python3
"""Reconstruct Contour's full timeline table from contour.exe.

Five 40-byte records are static in .data at VA 0x438948; the remaining 84 are
written at runtime by an init routine at VA 0x401560 as a long run of
`mov dword ptr [abs], imm32/reg` stores into 0x438a10..0x439758. This script
replays those stores symbolically (tracking register constants, since MSVC
hoisted the repeated class-name pointers into registers) and emits the whole
table as JSON, with class/instance pointers resolved to strings and each
record's parameter block hex-dumped for later decoding.

Usage: extract-timeline.py contour.exe timeline.json
"""
import json
import struct
import sys

import capstone

EXE, OUT = sys.argv[1], sys.argv[2]
data = open(EXE, 'rb').read()

IMAGE_BASE = 0x400000
# section: (va, raw, rawsize) — from the PE section table of this exe
SECTIONS = [
    (0x401000, 0x1000, 0x31000),
    (0x432000, 0x32000, 0x6000),
    (0x438000, 0x38000, 0xa000),
    (0x462000, 0x42000, 0x570000),
]

def va2off(va):
    for sva, raw, size in SECTIONS:
        if sva <= va < sva + size:
            return raw + (va - sva)
    return None

def cstr(va):
    off = va2off(va)
    if off is None:
        return None
    end = data.index(b'\0', off)
    s = data[off:end]
    if not s or any(c < 9 or c > 126 for c in s):
        return None
    return s.decode('latin1')

TABLE_START = 0x438948
TABLE_END = 0x439758 + 0x28
RECORD = 0x28

# ---- replay the init routine's stores ----
md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
md.detail = True

mem = {}      # target VA -> dword value (None = written from a runtime value)
regs = {}     # register name -> last known imm32
store_count = 0
unknown_count = 0
GPRS = ('eax', 'ebx', 'ecx', 'edx', 'esi', 'edi', 'ebp')

off = va2off(0x401560)
va = 0x401560
while True:
    insns = list(md.disasm(data[off:off + 15], va))
    if not insns:
        raise SystemExit(f'undecodable byte at 0x{va:x}')
    insn = insns[0]
    if insn.mnemonic == 'ret':
        break
    if insn.mnemonic == 'mov':
        dst, src = [p.strip() for p in insn.op_str.split(',', 1)]
        if dst.startswith('dword ptr [0x'):
            target = int(dst[len('dword ptr ['):-1], 16)
            if TABLE_START <= target < TABLE_END:
                if src.startswith('0x') or src.isdigit():
                    mem[target] = int(src, 0)
                    store_count += 1
                elif src in regs:
                    mem[target] = regs[src]
                    store_count += 1
                else:
                    mem[target] = None  # runtime-resolved (e.g. loaded from .data)
                    unknown_count += 1
        elif dst in GPRS:
            if src.startswith('0x') or src.isdigit():
                regs[dst] = int(src, 0)
            else:
                regs.pop(dst, None)  # value no longer known
    elif insn.mnemonic == 'xor':
        ops = [p.strip() for p in insn.op_str.split(',')]
        if len(ops) == 2 and ops[0] == ops[1] and ops[0] in GPRS:
            regs[ops[0]] = 0
    else:
        # any other instruction that names a register first may clobber it
        first = insn.op_str.split(',')[0].strip()
        if first in GPRS:
            regs.pop(first, None)
    va += insn.size
    off += insn.size

print(f'replayed init routine: {store_count} known stores, {unknown_count} runtime-valued', file=sys.stderr)

# overlay the five static records straight from .data
for rec_va in range(TABLE_START, 0x4389e8 + RECORD, RECORD):
    roff = va2off(rec_va)
    for i in range(0, RECORD, 4):
        mem.setdefault(rec_va + i, struct.unpack_from('<I', data, roff + i)[0])

# ---- decode records ----
def get(va, default=0):
    v = mem.get(va, default)
    return default if v is None else v

records = []
for rec_va in range(TABLE_START, TABLE_END, RECORD):
    cls_ptr = get(rec_va + 0x04)
    inst_ptr = get(rec_va + 0x08)
    kind = get(rec_va + 0x10)
    inst_id = get(rec_va + 0x14)
    lo, hi = get(rec_va + 0x18), get(rec_va + 0x1c)
    time = struct.unpack('<d', struct.pack('<II', lo, hi))[0]
    handler = get(rec_va + 0x20)
    params = get(rec_va + 0x24)

    cls = cstr(cls_ptr) if cls_ptr else None
    inst = cstr(inst_ptr) if inst_ptr else None
    if not cls and not inst and not handler:
        continue  # empty slot

    rec = {
        'va': f'0x{rec_va:x}',
        'class': cls,
        'instance': inst,
        'kind': kind,
        'id': inst_id,
        'time': round(time, 4),
        'handler': f'0x{handler:x}' if handler else None,
    }
    if params:
        rec['params'] = f'0x{params:x}'
        poff = va2off(params)
        if poff is not None:
            blob = data[poff:poff + 64]
            rec['params_hex'] = blob.hex()
            s = cstr(params)
            if s and len(s) > 3:
                rec['params_str'] = s
    records.append(rec)

records.sort(key=lambda r: (r['time'], r['va']))

KINDS = {0: 'create', 1: 'kill', 2: 'msg2', 3: 'msg3', 4: 'msg4'}
summary = {}
for r in records:
    summary.setdefault(r['class'] or '?', 0)
    summary[r['class'] or '?'] += 1

out = {
    'source': 'contour.exe (TBL, 1999) — static records at 0x438948 + init routine 0x401560',
    'record_count': len(records),
    'by_class': summary,
    'kinds': KINDS,
    'records': records,
}
with open(OUT, 'w') as f:
    json.dump(out, f, indent=1)
print(f'{len(records)} records -> {OUT}', file=sys.stderr)
print('by class:', summary, file=sys.stderr)
