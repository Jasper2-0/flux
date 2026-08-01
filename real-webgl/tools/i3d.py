#!/usr/bin/env python3
"""Reader for Energy3D's .i3d scene files.

Transcribed from Energy3D.dll, which ships with its MSVC decorated names
intact, so every structure here comes from the loader rather than from
pattern-matching the bytes.

`energy3d_i3d_loader::load` (0x1000ca50) reads the whole file, checks for
a `0xDEAD` header whose size covers the rest, and then walks chunks of
`<u32 id> <u32 size> <payload>` to the end. Five ids are recognised; any
other is skipped by its size, and the same is true of every nested chunk,
which is what makes the format safe to parse partially:

    0x00  materials      0x30  lights
    0x10  trimeshes      0x60  scene settings
    0x20  cameras

Inside a trimesh or a camera:

    0x50  name and parent          0x14  texture coordinates
    0x51  transform (below)        0x15  material assignment
    0x12  vertices                 0x40  animation
    0x13  triangles                0x21/0x22  camera settings

The exporter's axis conversion is `remap` (0x1000e830), which turns Max's
Z-up right-handed world into `(-x, z, y)`. That determinant is +1, so the
result is still right-handed — Y-up right-handed, exactly what OpenGL
wants, and unlike the Direct3D-targeted exporters of the same era nothing
has to be mirrored on the way in.

Angles arrive in radians and are stored as degrees (0x1000bd83:
`fov * 180 * (1/pi)`).
"""
import json
import math
import os
import struct
import sys


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

    def f32(self):
        v = struct.unpack_from('<f', self.d, self.p)[0]; self.p += 4; return v

    def f32n(self, n):
        v = struct.unpack_from('<%df' % n, self.d, self.p); self.p += 4 * n
        return list(v)

    def s(self):
        e = self.d.index(b'\0', self.p)
        v = self.d[self.p:e].decode('latin1')
        self.p = e + 1
        return v

    def chunks(self, end):
        """Yield (id, size, payload-start) and leave the cursor past each."""
        while self.p + 8 <= end:
            cid = self.u32(); size = self.u32(); start = self.p
            yield cid, size, start
            self.p = start + size


def remap(v):
    """Max Z-up right-handed -> Energy3D Y-up right-handed."""
    return [-v[0], v[2], v[1]]


def axis_angle_to_quat(ax, ay, az, angle):
    """The engine's own conversion, as a quaternion in (x, y, z, w) order.

    A node's rotation is stored as an axis and an angle in radians, and the
    loader (0x10009d5d) hands the axis to `remap` and the angle *negated*
    to energy3d_quat::fromAngAxis. Written out, that is a plain component
    shuffle: the remapped axis (-ax, az, ay) with -angle gives
    (s*ax, -s*az, -s*ay, c).
    """
    s = math.sin(angle * 0.5)
    return [s * ax, -s * az, -s * ay, math.cos(angle * 0.5)]


def quat_remap(w, x, y, z):
    """The same conversion for a stored quaternion (0x1000ae00 reads it as
    (w, x, y, z), converts to axis-angle, remaps and negates), which
    reduces to (x, -z, -y, w)."""
    return [x, -z, -y, w]


# ---------------------------------------------------------------- materials

def read_texture(r):
    """One texture slot. 3 is specular, 8 is filter/opacity, 11 is
    reflection — the three slots Energy3D keeps (0x1000a682)."""
    tex = {'slot': r.u8(), 'name': r.s(), 'kind': r.s()}
    r.u16(); r.u32()
    if r.u8() == 1:
        tex['file'] = r.s()
    if r.u8() == 1:
        # the UV block: a flag and eleven floats (0x1000a550)
        r.u8(); tex['uv'] = r.f32n(11)
    r.u8(); r.u8()
    tex['sub'] = [read_texture(r) for _ in range(r.u16())]
    return tex


def read_material(r):
    r.u8()
    mat = {'index': r.u16(), 'name': r.s(), 'kind': r.s()}
    full = r.u8() == 1
    mat['diffuse'] = r.f32n(3)
    mat['ambient'] = r.f32n(3)
    mat['specular'] = r.f32n(3)
    mat['shininess'] = r.f32()
    r.f32()
    mat['opacity'] = 1.0 - r.f32()
    r.f32()
    if full:
        r.u8(); r.f32(); r.f32()
        flags = r.u8()
        mat['twoSided'] = bool(flags & 2)
        r.u8()
    mat['textures'] = [read_texture(r) for _ in range(r.u16())]
    mat['sub'] = [read_material(r) for _ in range(r.u16())]
    return mat


# ---------------------------------------------------------------- animation

def read_pos_keys(r):
    """0x41 — one key is a u16 frame, a remapped vector and five TCB terms
    (tension, continuity, bias, ease-to, ease-from)."""
    keys = []
    for _ in range(r.u16()):
        f = r.u16()
        v = remap(r.f32n(3))
        keys.append({'f': f, 'v': v, 'tcb': r.f32n(5)})
    return keys


def read_rot_keys(r):
    """0x42 — unlike position, rotation is baked: a first and last frame
    and then one quaternion per frame in between, with no TCB terms. The
    engine stores quaternions as (w, x, y, z)."""
    first, last = r.u16(), r.u16()
    keys = []
    for i in range(max(0, last - first)):
        w, x, y, z = r.f32n(4)
        keys.append({'f': first + i, 'q': quat_remap(w, x, y, z),
                     'tcb': [0, 0, 0, 0, 0]})
    return keys


def read_scale_keys(r):
    first, last = r.u16(), r.u16()
    keys = []
    for i in range(max(0, last - first)):
        v = r.f32n(3)
        keys.append({'f': first + i, 'v': [v[0], v[2], v[1]],
                     'tcb': [0, 0, 0, 0, 0]})
    return keys


def read_float_keys(r):
    keys = []
    for _ in range(r.u16()):
        f = r.u16()
        keys.append({'f': f, 'v': r.f32(), 'tcb': r.f32n(5)})
    return keys


def read_anim(r, end):
    """0x40 — a name, then per-property key chunks 0x41..0x46."""
    anim = {'node': r.s(), 'pos': [], 'rot': [], 'scale': [], 'extra': {}}
    for cid, size, start in r.chunks(end):
        sub = Reader(r.d, start)
        try:
            if cid == 0x41:
                anim['pos'] = read_pos_keys(sub)
            elif cid == 0x42:
                anim['rot'] = read_rot_keys(sub)
            elif cid == 0x43:
                anim['scale'] = read_scale_keys(sub)
            elif cid in (0x44, 0x45, 0x46):
                anim['extra'][cid] = read_float_keys(sub)
        except Exception:
            pass
    return anim


# ------------------------------------------------------------------- nodes

def read_transform(r):
    """0x51 — a name, a 4x3 matrix the engine ignores, then the position,
    rotation and scale it actually uses. 26 floats after the name."""
    t = {'node': r.s()}
    t['matrix'] = r.f32n(12)
    t['pos'] = remap(r.f32n(3))
    q = r.f32n(4)
    t['rot'] = axis_angle_to_quat(*q)
    t['rotRaw'] = q
    s = r.f32n(3)
    t['scale'] = [s[0], s[2], s[1]]
    r.f32n(4)                       # the scale-axis quaternion, unused
    return t


def read_trimesh(r, end):
    m = {'verts': [], 'faces': [], 'uvs': [], 'material': None,
         'transform': None, 'anim': [], 'name': '', 'parent': ''}
    for cid, size, start in r.chunks(end):
        sub = Reader(r.d, start)
        if cid == 0x50:
            m['name'] = sub.s(); m['parent'] = sub.s()
        elif cid == 0x51:
            m['transform'] = read_transform(sub)
        elif cid == 0x12:
            n = sub.u16()
            m['verts'] = [remap(sub.f32n(3)) for _ in range(n)]
        elif cid == 0x13:
            n = sub.u16()
            m['faces'] = [[sub.u16(), sub.u16(), sub.u16()] for _ in range(n)]
        elif cid == 0x14:
            if sub.u8() == 1:
                n = sub.u16()
                m['uvs'] = [[sub.f32(), sub.f32()] for _ in range(n)]
        elif cid == 0x15:
            if sub.u8() and sub.u8() == 0:
                m['material'] = sub.u16()
        elif cid == 0x40:
            m['anim'].append(read_anim(sub, start + size))
    return m


def read_camera(r, end):
    c = {'name': '', 'parent': '', 'nodes': [], 'anim': [],
         'fov': 45.0, 'near': 1.0, 'far': 5000.0}
    for cid, size, start in r.chunks(end):
        sub = Reader(r.d, start)
        if cid == 0x50:
            c['name'] = sub.s(); c['parent'] = sub.s()
        elif cid == 0x51:
            c['nodes'].append(read_transform(sub))
        elif cid == 0x22:
            if sub.u8() == 1:
                sub.u32(); sub.u32()
            sub.f32(); sub.f32()
            c['fov'] = sub.f32() * 180.0 / math.pi
            c['targetDist'] = sub.f32()
        elif cid == 0x40:
            c['anim'].append(read_anim(sub, start + size))
    if c['nodes']:
        c['pos'] = c['nodes'][0]['pos']
        c['rot'] = c['nodes'][0]['rot']
    if len(c['nodes']) > 1:
        c['target'] = c['nodes'][1]['pos']
    return c


def read_light(r, end):
    l = {'name': '', 'parent': '', 'nodes': [], 'anim': []}
    for cid, size, start in r.chunks(end):
        sub = Reader(r.d, start)
        if cid == 0x50:
            l['name'] = sub.s(); l['parent'] = sub.s()
        elif cid == 0x51:
            l['nodes'].append(read_transform(sub))
        elif cid == 0x40:
            l['anim'].append(read_anim(sub, start + size))
    if l['nodes']:
        l['pos'] = l['nodes'][0]['pos']
    return l


def load(path):
    d = open(path, 'rb').read()
    r = Reader(d)
    magic, size = r.u32(), r.u32()
    if magic != 0xDEAD:
        raise ValueError('%s is not an i3d file' % path)
    scene = {'materials': [], 'meshes': [], 'cameras': [], 'lights': [],
             'frameStart': 0, 'frameEnd': 0}
    for cid, csize, start in r.chunks(min(size + 8, len(d))):
        sub = Reader(d, start)
        if cid == 0x00:
            scene['materials'] = [read_material(sub) for _ in range(sub.u16())]
        elif cid == 0x10:
            scene['meshes'].append(read_trimesh(sub, start + csize))
        elif cid == 0x20:
            scene['cameras'].append(read_camera(sub, start + csize))
        elif cid == 0x30:
            scene['lights'].append(read_light(sub, start + csize))
        elif cid == 0x60:
            sub.u16()
            scene['frameEnd'] = sub.u16()
            scene['fps'] = sub.u32()
            scene['flags'] = sub.u32()
    return scene


if __name__ == '__main__':
    for path in sys.argv[1:]:
        s = load(path)
        print('=== %s' % os.path.basename(path))
        print('  frames 0..%s  fps %s  flags %s' %
              (s['frameEnd'], s.get('fps'), s.get('flags')))
        for m in s['materials']:
            print('  material %-24s tex %s%s' %
                  (m['name'], [(t['slot'], t.get('file')) for t in m['textures']],
                   ' 2-sided' if m.get('twoSided') else ''))
            for sm in m['sub']:
                print('    sub %-22s tex %s' %
                      (sm['name'], [(t['slot'], t.get('file')) for t in sm['textures']]))
        for m in s['meshes']:
            t = m['transform'] or {}
            print('  mesh %-16s v %-5d f %-5d uv %-5d mat %-4s pos %s anim %s' %
                  (m['name'], len(m['verts']), len(m['faces']), len(m['uvs']),
                   m['material'],
                   ['%.1f' % x for x in t.get('pos', [])],
                   [(a['node'], len(a['pos']), len(a['rot']), len(a['scale']))
                    for a in m['anim']]))
        for c in s['cameras']:
            print('  camera %-14s fov %.1f pos %s target %s anim %s' %
                  (c['name'], c['fov'],
                   ['%.1f' % x for x in c.get('pos', [])],
                   ['%.1f' % x for x in c.get('target', [])],
                   [(a['node'], len(a['pos']), len(a['rot'])) for a in c['anim']]))
        for l in s['lights']:
            print('  light %-15s pos %s' %
                  (l['name'], ['%.1f' % x for x in l.get('pos', [])]))
