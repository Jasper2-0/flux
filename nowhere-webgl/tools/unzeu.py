#!/usr/bin/env python3
"""Parse Zeus .zeu scenes (Threestate / Nowhere, 2000) into JSON.

The format is a chunk tree; `LoadZeu` (Zeus.dll 0x10006b70) reads a u16
version, then walks chunks with the dispatcher at 0x10006d00 against the
handler table at 0x10014030:

    u16 id · u32 size (includes the 6-byte header) · payload
    0x1000 SCENE  0x2000 MESH  0x3000 CAMERA  0x4000 LIGHT  0x5000 HELPER

Chunks nest: children live inside the parent's size. Every object starts
with the header that `readObjHeader` (0x10005f10) reads — two length-
prefixed strings, name and parent — and ends with animation splines.

Mesh payload (handler 0x10005750):
    u16 vertexCount · u16 polygonCount
    vertex[]   f32 x,y,z · u8 r,g,b · f32 u,v            (23 bytes)
    polygon[]  u16 a,b,c                                  (6 bytes)
    u32 bucketCount
    bucket[]   string material · u32 faceCount · u32 vertexCount
               u32 polygonIndex[faceCount]
    vector spline (position) · quat spline (rotation) · vector spline (scale)

Splines are `u8 loop · u32 keyCount · key[]`, and a key is written as the
raw sKey struct: an i32 frame number, five TCB floats, then the payload —
3 floats for a vector, 4 for a quaternion, 1 for a scalar. Cameras carry
a position spline plus two scalar splines (roll in radians, then field
of view in degrees); helpers carry a position spline only. Some exports
leave the TCB fields uninitialised, so non-finite values are zeroed.

Usage: unzeu.py <scene.zeu> [out.json]
"""
import json
import math
import struct
import sys

IDS = {0x1000: 'scene', 0x2000: 'mesh', 0x3000: 'camera', 0x4000: 'light', 0x5000: 'helper'}

VECTOR, QUAT, SCALAR = 3, 4, 1


class Reader:
    def __init__(self, data, pos=0):
        self.d = data
        self.p = pos

    def u8(self):
        v = self.d[self.p]; self.p += 1; return v

    def u16(self):
        v = struct.unpack_from('<H', self.d, self.p)[0]; self.p += 2; return v

    def u32(self):
        v = struct.unpack_from('<I', self.d, self.p)[0]; self.p += 4; return v

    def i32(self):
        v = struct.unpack_from('<i', self.d, self.p)[0]; self.p += 4; return v

    def f32(self, n=1):
        v = struct.unpack_from('<%df' % n, self.d, self.p); self.p += 4 * n
        return v[0] if n == 1 else list(v)

    def string(self):
        n = self.u8()
        s = self.d[self.p:self.p + n]; self.p += n
        return s.decode('latin1')


def clean(v):
    """TCB fields are sometimes uninitialised stack in the exports."""
    return v if math.isfinite(v) and abs(v) < 1e3 else 0.0


def read_spline(r, width):
    loop = r.u8()
    n = r.u32()
    keys = []
    for _ in range(n):
        frame = r.i32()
        tcb = [clean(x) for x in r.f32(5)]
        val = r.f32(width)
        keys.append({'frame': frame, 'tcb': tcb, 'v': val if width > 1 else [val]})
    return {'loop': loop, 'keys': keys}


def read_obj_header(r, obj):
    obj['name'] = r.string()
    obj['parent'] = r.string()


def read_mesh(r, obj):
    read_obj_header(r, obj)
    nv, npoly = r.u16(), r.u16()
    verts = []
    for _ in range(nv):
        x, y, z = r.f32(3)
        cr, cg, cb = r.u8(), r.u8(), r.u8()
        u, v = r.f32(2)
        verts.append([x, y, z, cr, cg, cb, u, v])
    polys = [list(struct.unpack_from('<3H', r.d, r.p + i * 6)) for i in range(npoly)]
    r.p += npoly * 6
    buckets = []
    for _ in range(r.u32()):
        mat = r.string()
        nfaces, nbverts = r.u32(), r.u32()
        faces = [r.u32() for _ in range(nfaces)]
        buckets.append({'material': mat, 'vertexCount': nbverts, 'faces': faces})
    obj.update({'vertices': verts, 'polygons': polys, 'buckets': buckets,
                'position': read_spline(r, VECTOR),
                'rotation': read_spline(r, QUAT),
                'scale': read_spline(r, VECTOR)})


def read_camera(r, obj):
    read_obj_header(r, obj)
    obj['position'] = read_spline(r, VECTOR)
    # two scalar splines: roll (radians) then field of view (degrees)
    obj['roll'] = read_spline(r, SCALAR)
    obj['fov'] = read_spline(r, SCALAR)


def read_light(r, obj, end):
    read_obj_header(r, obj)
    # a third string: the object this light is bound to. Max camera
    # targets come through as lights named "Target" with parent
    # "Camera01.Target" and this field naming the camera itself.
    obj['target'] = r.string()
    obj['position'] = read_spline(r, VECTOR)
    obj['rotation'] = read_spline(r, QUAT)
    obj['scale'] = read_spline(r, VECTOR)


def read_helper(r, obj):
    read_obj_header(r, obj)
    obj['position'] = read_spline(r, VECTOR)


def parse_chunk(data, pos, out, depth=0):
    cid, size = struct.unpack_from('<HI', data, pos)
    kind = IDS.get(cid)
    if kind is None:
        raise ValueError('unknown chunk 0x%04x at 0x%x' % (cid, pos))
    end = pos + size
    r = Reader(data, pos + 6)
    obj = {'kind': kind}
    if kind == 'scene':
        # handler 0x10005690: one string, then two u16 frame bounds
        obj['name'] = r.string()
        obj['frameStart'] = r.u16()
        obj['frameEnd'] = r.u16()
    elif kind == 'mesh':
        read_mesh(r, obj)
    elif kind == 'camera':
        read_camera(r, obj)
    elif kind == 'light':
        read_light(r, obj, end)
    elif kind == 'helper':
        read_helper(r, obj)
    obj['_consumed'] = r.p - pos
    obj['_size'] = size
    out.append(obj)
    # anything left inside this chunk is a child chunk
    p = r.p
    while p + 6 <= end:
        p = parse_chunk(data, p, out, depth + 1)
    if p != end:
        raise ValueError('%s %r: ended at 0x%x, chunk ends 0x%x' %
                         (kind, obj.get('name'), p, end))
    return end


def parse(data):
    version = struct.unpack_from('<H', data, 0)[0]
    if version != 1:
        raise ValueError('unsupported .zeu version %d' % version)
    out = []
    p = parse_chunk(data, 2, out)
    if p != len(data):
        raise ValueError('trailing data: 0x%x != 0x%x' % (p, len(data)))
    return {'version': version, 'objects': out}


def summarise(scene):
    for o in scene['objects']:
        bits = ['%-6s %-14s' % (o['kind'], o.get('name', ''))]
        if o['kind'] == 'scene':
            bits.append('frames=%d..%d' % (o['frameStart'], o['frameEnd']))
        if 'vertices' in o:
            bits.append('v=%-5d p=%-5d buckets=%s' %
                        (len(o['vertices']), len(o['polygons']),
                         ','.join('%s:%d' % (b['material'], len(b['faces'])) for b in o['buckets'])))
        for k in ('position', 'rotation', 'scale', 'fov', 'roll'):
            if k in o and o[k]['keys']:
                bits.append('%s=%dk' % (k[:3], len(o[k]['keys'])))
        if o.get('parent'):
            bits.append('parent=%s' % o['parent'])
        print('   ' + '  '.join(bits))


if __name__ == '__main__':
    src = sys.argv[1]
    scene = parse(open(src, 'rb').read())
    summarise(scene)
    if len(sys.argv) > 2:
        with open(sys.argv[2], 'w') as f:
            json.dump(scene, f)
        print('wrote', sys.argv[2])
