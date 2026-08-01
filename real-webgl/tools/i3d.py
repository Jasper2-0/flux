#!/usr/bin/env python3
"""Reader for Energy3D's .i3d scene files.

The format is a flat chunk stream, as `energy3d_i3d_loader::load` reads it:
a `0xDEAD` header whose size covers the rest of the file, then chunks of
`<u32 id> <u32 size> <payload>` until that size is used up. Energy3D
recognises five ids and skips anything else by its size:

    0x00  materials
    0x10  cameras
    0x20  trimeshes
    0x30  lights
    0x60  ambient / scene settings

Strings are NUL-terminated. Counts are u16 unless noted.
"""
import struct
import sys


class Reader:
    def __init__(self, data, pos=0):
        self.d = data
        self.p = pos

    def u32(self):
        v = struct.unpack_from('<I', self.d, self.p)[0]; self.p += 4; return v

    def i32(self):
        v = struct.unpack_from('<i', self.d, self.p)[0]; self.p += 4; return v

    def u16(self):
        v = struct.unpack_from('<H', self.d, self.p)[0]; self.p += 2; return v

    def u8(self):
        v = self.d[self.p]; self.p += 1; return v

    def f32(self):
        v = struct.unpack_from('<f', self.d, self.p)[0]; self.p += 4; return v

    def vec(self):
        return [self.f32(), self.f32(), self.f32()]

    def s(self):
        e = self.d.index(b'\0', self.p)
        v = self.d[self.p:e].decode('latin1')
        self.p = e + 1
        return v


def chunks(data):
    r = Reader(data)
    cid, size = r.u32(), r.u32()
    assert cid == 0xDEAD, 'not an i3d file'
    end = size + 8
    while r.p < end:
        cid, size = r.u32(), r.u32()
        yield cid, size, r.p
        if cid in (0x00,):
            # the material chunk's size does not include its payload the
            # way the others do; walk it properly instead
            r.p = walk_materials(data, r.p)
        else:
            r.p += size


def walk_materials(data, pos):
    return pos


if __name__ == '__main__':
    d = open(sys.argv[1], 'rb').read()
    r = Reader(d)
    cid, size = r.u32(), r.u32()
    print('root %04x size %d (file %d)' % (cid, size, len(d)))
    p = 8
    while p < len(d) - 8:
        cid, size = struct.unpack_from('<II', d, p)
        print('  %08x  id %04x  size %-8d  %s' %
              (p, cid, size, d[p + 8:p + 40].hex(' ')))
        p += 8 + size
