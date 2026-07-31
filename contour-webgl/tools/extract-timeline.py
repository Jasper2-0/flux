#!/usr/bin/env python3
"""Reconstruct Contour's timeline from contour.exe — corrected layout.

The runtime dispatcher (tick at VA 0x403ce0, player ctor at 0x403840)
defines the truth: records are 0x28 bytes, based at VA 0x438930, and the
table ends at the first record whose kind is 2 (terminator).

    +0x00  uint32   kind: 0 create · 1 kill · 2 END · 3 message
                          4 music-object call · 5 clock jump
    +0x04  uint32   instance id
    +0x08  double   cue time in seconds (compared against the MP3 clock)
    +0x10  uint32   create: constructor/handler · message: code
                    clock jump: seconds (int) · music call: argument
    +0x14  uint32   parameter block pointer (create) / message param
    +0x18  uint32   extra message param (usually 0)
    +0x1c  char*    class (the coder's namespace)
    +0x20  char*    instance name

Earlier extractions used base 0x438948, which shifted class/instance one
record against everything else — the corrected base fixes the mislabeled
message rows.

Most of the table is written at runtime by the init routine at 0x401560;
this script replays its stores (tracking register-held string pointers),
overlays the static .data contents, and decodes the table.

Usage: extract-timeline.py contour.exe timeline.json
"""
import json
import struct
import sys

import capstone

EXE, OUT = sys.argv[1], sys.argv[2]
data = open(EXE, 'rb').read()

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

TABLE_BASE = 0x438930
TABLE_LIMIT = 0x439800
RECORD = 0x28

# ---- replay the init routine's stores ----
md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)

mem = {}
regs = {}
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
            if TABLE_BASE <= target < TABLE_LIMIT:
                if src.startswith('0x') or src.isdigit():
                    mem[target] = int(src, 0)
                    store_count += 1
                elif src in regs:
                    mem[target] = regs[src]
                    store_count += 1
                else:
                    mem[target] = None
                    unknown_count += 1
        elif dst in GPRS:
            if src.startswith('0x') or src.isdigit():
                regs[dst] = int(src, 0)
            else:
                regs.pop(dst, None)
    elif insn.mnemonic == 'xor':
        ops = [p.strip() for p in insn.op_str.split(',')]
        if len(ops) == 2 and ops[0] == ops[1] and ops[0] in GPRS:
            regs[ops[0]] = 0
    else:
        first = insn.op_str.split(',')[0].strip()
        if first in GPRS:
            regs.pop(first, None)
    va += insn.size
    off += insn.size

print(f'replayed init routine: {store_count} known stores, {unknown_count} runtime-valued', file=sys.stderr)

# overlay static .data for anything the init routine didn't write
for addr in range(TABLE_BASE, TABLE_LIMIT, 4):
    if addr not in mem:
        o = va2off(addr)
        mem[addr] = struct.unpack_from('<I', data, o)[0] if o else 0

def get(addr):
    v = mem.get(addr, 0)
    return 0 if v is None else v

KINDS = {0: 'create', 1: 'kill', 2: 'end', 3: 'message', 4: 'musicctl', 5: 'clockjump'}

records = []
rec_va = TABLE_BASE
index = 0
while rec_va < TABLE_LIMIT:
    kind = get(rec_va + 0x00)
    inst_id = get(rec_va + 0x04)
    lo, hi = get(rec_va + 0x08), get(rec_va + 0x0c)
    time = struct.unpack('<d', struct.pack('<II', lo, hi))[0]
    a = get(rec_va + 0x10)
    b = get(rec_va + 0x14)
    c = get(rec_va + 0x18)
    cls = cstr(get(rec_va + 0x1c)) if get(rec_va + 0x1c) else None
    inst = cstr(get(rec_va + 0x20)) if get(rec_va + 0x20) else None

    rec = {
        'index': index,
        'va': f'0x{rec_va:x}',
        'kind': KINDS.get(kind, kind),
        'id': inst_id,
        'time': round(time, 4),
        'class': cls,
        'instance': inst,
    }
    if kind == 0:
        rec['ctor'] = f'0x{a:x}'
        if b:
            rec['params'] = f'0x{b:x}'
    elif kind == 3:
        rec['code'] = f'0x{a:x}'
        if b:
            rec['param'] = f'0x{b:x}'
        if c:
            rec['extra'] = f'0x{c:x}'
    elif kind in (4, 5):
        rec['arg'] = a

    # Scid's Letters parameter block (48 bytes), layout confirmed against
    # the strings and the release capture:
    #   +0x00 char* text · +0x04 type (1 poem line, 2 title card)
    #   +0x0c,+0x10 start x,y · +0x14 scale · +0x18,+0x1c end x,y
    #   +0x20 duration · +0x24,+0x28 (title card only) fade-in window
    if kind == 0 and b:
        poff = va2off(b)
        if poff is not None:
            tptr = struct.unpack_from('<I', data, poff)[0]
            ttype = struct.unpack_from('<I', data, poff + 4)[0]
            txt = cstr(tptr) if 0x432000 <= tptr < 0x43a000 else None
            if txt and ttype in (1, 2) and len(txt) > 2:
                f = struct.unpack_from('<11f', data, poff + 4)
                rec['letters'] = {
                    'text': txt,
                    'type': ttype,
                    'x0': round(f[2], 4), 'y0': round(f[3], 4),
                    'scale': round(f[4], 4),
                    'x1': round(f[5], 4), 'y1': round(f[6], 4),
                    'duration': round(f[7], 4),
                    'fade0': round(f[8], 4), 'fade1': round(f[9], 4),
                }

    # Jace's parameter blocks, read off the constructors, the geometry
    # builders and the update methods.
    #
    # flash (ctor 0x41ff80, block 32 bytes at this+0x94). The builder at
    # 0x4200e0 lays a fan of N−1 rim vertices of radius p[0x14] with the
    # centre vertex last, and gives every vertex z = p[0x14] too — so the
    # radius cancels against the depth and the fan is always a full-frame
    # wash. The update 0x420160 writes only colours, from two clamped
    # 0→1 ramps; the object dies when both pass 0.999.
    #   +0x00,+0x04  rim ramp start/end        +0x08,+0x0c  centre ramp
    #   +0x14 radius (== depth)  +0x18,+0x1c   0xRRGGBB from/to
    #
    # picflash (ctor 0x41fce0, block 32 bytes at this+0x94; four vertices
    # allocated at 0x41fdc0, rebuilt each frame by 0x41fe30 as a square of
    # half-diagonal `size` rotated by `ang`, the baked π/4 making angle 0
    # axis-aligned):
    #   t = T / p[0x14];  size = lerp(p0,p1,t)×0.01;  ang = lerp(p2,p3,t)+π/4
    #   z = p[0x10] + 0.002t;  alpha = clamp01((1−t)×p[0x18])
    #   +0x1c is a texture name for the init (0x41fda0) — null everywhere.
    #
    # linefx (ctor 0x41f4e0, block 128 bytes at this+0xb4): a ring
    # emitter. Eight (base, rate, drift) triples — spawn writes
    # base + drift×T (0x41f920), update evaluates + rate×age (0x41f5c0) —
    # plus a colour, an attack/sustain/release envelope whose sum is the
    # particle life, and four emitter fields. The init (0x41f530) forces
    # the segment count to 70 if it falls outside [1, 70].
    if kind == 0 and b and inst in ('flash', 'picflash', 'linefx'):
        poff = va2off(b)
        if poff is not None:
            n = 32 if inst == 'linefx' else 8
            w = struct.unpack_from('<%dI' % n, data, poff)
            f = struct.unpack_from('<%df' % n, data, poff)
            r4 = lambda x: round(x, 4)
            if inst == 'flash':
                rec['flash'] = {
                    'rampRim': [r4(f[0]), r4(f[1])],
                    'rampCentre': [r4(f[2]), r4(f[3])],
                    'radius': r4(f[5]),
                    'colors': [w[6], w[7]],
                }
            elif inst == 'picflash':
                rec['picflash'] = {
                    'size': [r4(f[0]), r4(f[1])],
                    'angle': [r4(f[2]), r4(f[3])],
                    'depth': r4(f[4]),
                    'duration': r4(f[5]),
                    'falloff': r4(f[6]),
                    'texture': w[7],
                }
            else:
                # triples at 0x00/0x0c/0x18, then the colour, then
                # 0x28/0x34/0x40/0x4c/0x58 — indices 0,3,6,(9),10,13,16,19,22
                tri = lambda i: [r4(f[i]), r4(f[i + 1]), r4(f[i + 2])]
                segs = w[31]
                rec['linefx'] = {
                    'cx': tri(0), 'cy': tri(3), 'width': tri(6),
                    'color': w[9],
                    'bright': tri(10),
                    'angle0': tri(13), 'angle1': tri(16),
                    'radius0': tri(19), 'radius1': tri(22),
                    'attack': r4(f[25]), 'sustain': r4(f[26]), 'release': r4(f[27]),
                    'interval': r4(f[28]), 'emitUntil': r4(f[29]),
                    'maxAlive': w[30],
                    'segments': segs if 1 <= segs <= 70 else 70,
                }

    # Known block sizes, so the string scan below cannot walk past the end
    # of a block and attribute the *next* record's data to this one. (That
    # is exactly how `scenes\neuron.bin` — which lives one dword past
    # instance 455's 32-byte flash block — was once read as a scene name
    # belonging to 455.)
    BLOCK_SIZE = {'flash': 32, 'picflash': 32, 'linefx': 128, 'Letters': 48}
    span = BLOCK_SIZE.get(inst, 64)

    # annotate pointer-ish fields with resolved strings / hexdumps
    for key, ptr in (('params', b if kind == 0 else 0), ('param', b if kind == 3 else 0)):
        if not ptr:
            continue
        poff = va2off(ptr)
        if poff is None:
            continue
        blob = data[poff:poff + span]
        rec[key + '_hex'] = blob.hex()
        rec[key + '_span'] = span
        s = cstr(ptr)
        if s and len(s) > 3:
            rec[key + '_str'] = s
        refs = []
        for i in range(0, span, 4):
            p2 = struct.unpack_from('<I', blob, i)[0] if i + 4 <= len(blob) else 0
            if 0x432000 <= p2 < 0x43a000:
                rs = cstr(p2)
                if rs and len(rs) > 3:
                    refs.append({'offset': i, 'str': rs})
        if refs:
            rec[key + '_refs'] = refs

    records.append(rec)
    index += 1
    rec_va += RECORD
    if kind == 2:
        break

summary = {}
for r in records:
    summary.setdefault(r['kind'], 0)
    summary[r['kind']] += 1

out = {
    'source': 'contour.exe (TBL, 1999) — table at 0x438930, layout per dispatcher 0x403ce0',
    'record_count': len(records),
    'by_kind': summary,
    'records': records,
}
with open(OUT, 'w') as f:
    json.dump(out, f, indent=1)
print(f'{len(records)} records -> {OUT}', file=sys.stderr)
print('by kind:', summary, file=sys.stderr)
