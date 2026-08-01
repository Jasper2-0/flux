#!/usr/bin/env python3
"""Unpack Axiom .stv libraries (Threestate / Nowhere, 2000).

Container (from cFileSys::OpenLib, Axiom.dll 0x1000a070):
    u16 count
    count x { u8 namelen; u8[namelen] name ^ (i + 0x73); u8 flags;
              u32 offset; u32 uncompressed_size }
Each blob is [u32 packed_size][aPLib stream] when flags&1, else raw.
The depacker is the aPLib one inlined at 0x1000a870.
"""
import struct, sys, os


def aplib_depack(src):
    dst = bytearray()
    i = 0
    bitbuf = 0
    bitcnt = 0
    lastoff = 0

    def getbit():
        nonlocal bitbuf, bitcnt, i
        if bitcnt == 0:
            bitbuf = src[i]; i += 1; bitcnt = 8
        bitcnt -= 1
        bit = (bitbuf >> 7) & 1
        bitbuf = (bitbuf << 1) & 0xff
        return bit

    def gamma():
        v = 1
        while True:
            cont = getbit()
            v = v * 2 + getbit()
            if not cont:
                return v

    while True:
        if getbit() == 0:                    # literal
            dst.append(src[i]); i += 1
            continue
        if getbit() == 0:                    # 10 -> long match
            v = gamma() - 2
            if v == 0:                       # reuse last offset
                n = gamma()
                for _ in range(n):
                    dst.append(dst[-lastoff])
                continue
            off = ((v - 1) << 8) | src[i]; i += 1
            n = gamma()
            if off <= 0x7f:
                n += 2
            lastoff = off
            for _ in range(n):
                dst.append(dst[-off])
            continue
        if getbit() == 0:                    # 110 -> short match
            b = src[i]; i += 1
            n = 2 + (b & 1)
            off = b >> 1
            if off == 0:
                return bytes(dst)            # end of stream
            # note: the shipped depacker does NOT update lastoff here
            for _ in range(n):
                dst.append(dst[-off])
            continue
        # 111 -> 4-bit single byte
        off = 0
        for _ in range(4):
            off = off * 2 + getbit()
        dst.append(dst[-off] if off else 0)


def read_toc(data):
    p = 0
    count = struct.unpack_from('<H', data, p)[0]; p += 2
    out = []
    for _ in range(count):
        n = data[p]; p += 1
        name = bytes(b ^ ((j + 0x73) & 0xff) for j, b in enumerate(data[p:p+n])).decode('latin1')
        p += n
        flags = data[p]; p += 1
        off, size = struct.unpack_from('<II', data, p); p += 8
        out.append({'name': name, 'flags': flags, 'offset': off, 'size': size})
    return out


def unpack_entry(data, e):
    off = e['offset']
    if e['flags'] & 1:
        n = struct.unpack_from('<I', data, off)[0]
        return aplib_depack(data[off + 4:off + 4 + n])
    return data[off:off + e['size']]


if __name__ == '__main__':
    lib, outdir = sys.argv[1], sys.argv[2]
    data = open(lib, 'rb').read()
    os.makedirs(outdir, exist_ok=True)
    ok = bad = 0
    for e in read_toc(data):
        try:
            blob = unpack_entry(data, e)
        except Exception as ex:
            print('FAIL %-30s %s' % (e['name'], ex)); bad += 1; continue
        tag = 'ok ' if len(blob) == e['size'] else 'SIZE'
        if len(blob) != e['size']:
            bad += 1
        else:
            ok += 1
        path = os.path.join(outdir, e['name'].replace('\\', '/').lstrip('./'))
        os.makedirs(os.path.dirname(path) or '.', exist_ok=True)
        open(path, 'wb').write(blob)
        print('%s %-32s %7d -> %7d  %s' % (tag, e['name'], e['size'], len(blob), blob[:4].hex()))
    print('ok=%d bad=%d' % (ok, bad))
