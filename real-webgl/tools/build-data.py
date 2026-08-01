#!/usr/bin/env python3
"""Turn a "We Ain't Real" release directory into the port's data/ tree.

    build-data.py <release-dir> <out-dir>

The demo ships its whole timeline as text in `real.scp`, and that file
documents its own grammar at the top:

    <"xxx:xxx" | minute:sec.msec> <layer> [<id command> ...] <command> <params>
    animate{'id' params easeto easefrom}   -- animates parameters
    stop / reset

`xxx:xxx` marks a preload. `@origin M:SS.mmm` rebases the clock for the
lines that follow — sections are written from zero and then placed.
Layers hold state: a `drawImage` / `drawScene` / `show*` command starts
whatever the layer shows, later cues on the same layer carry only
`animate{}` blocks that retarget its properties, and `stopAll` clears
everything.

Each `animate{}` is a keyframe: reach this value at this cue's time,
interpolating from whatever the property held at the previous keyframe.
`alpha` takes one value, `color`, `translation`, `rotation` and `scale`
take three, and every one of them ends with an ease-to and an ease-from.

Images come in pairs — a colour JPEG and, where the artwork needs one, a
separate greyscale JPEG for opacity, which `loadImage <img> mask <msk>`
binds together. This merges those into single RGBA PNGs.
"""
import json
import math
import os
import re
import shutil
import sys

from PIL import Image


PRELOAD = re.compile(r'^xxx', re.I)
TIME = re.compile(r'^(\d+):(\d+(?:\.\d+)?)$')

# how many real values each animatable property carries, before the two
# trailing ease terms
PROP_ARITY = {'alpha': 1, 'color': 3, 'translation': 3, 'rotation': 3, 'scale': 3}

# what each property holds before anything animates it — the values Red's
# per-property key constructors write (see the baseline pass below)
PROP_DEFAULT = {'alpha': (255.0,), 'color': (255.0, 255.0, 255.0),
                'translation': (0.0, 0.0, 0.0), 'rotation': (0.0, 0.0, 0.0),
                'scale': (1.0, 1.0, 1.0)}

# Named in Real.exe's string table, one per effect init:
#   show2d 0x40100a/0x401039/0x401068, showDraai 0x4029f7,
#   showPartiekels 0x4031ed, showPlanes 0x403613..0x40369d,
#   showCylinder 0x402661, showTunnel 0x403e23.
# showBol and showPartiekels also load sphere.i3d, which is converted with
# the other scenes — the script never mentions it either.
EFFECT_ASSETS = {
    'solar_groot.jpg', 'eeeeeeeenv.tga', 'bruut.tga', 'envmap.jpg',
    'particle.jpg', 'eenv2.tga',
    'plane_01.jpg', 'plane_02.jpg', 'plane_03.jpg', 'plane_04.jpg',
}

DRAW = {'drawimage', 'drawscene', 'colorfade', 'show2d', 'showlines',
        'showcylinder', 'showbol', 'showdraai', 'showtunnel', 'showplanes',
        'showpartiekels'}


def to_seconds(tok):
    m = TIME.match(tok)
    if not m:
        return None
    return int(m.group(1)) * 60 + float(m.group(2))


def parse_animations(line):
    """Pull the [animate{...} animate{...}] block off a cue line."""
    anims = {}
    for m in re.finditer(r'animate\{(\w+)([^}]*)\}', line):
        prop = m.group(1).lower()
        vals = [float(x) for x in m.group(2).split()]
        n = PROP_ARITY.get(prop, max(0, len(vals) - 2))
        anims[prop] = {
            'v': vals[:n],
            'easeTo': vals[n] if len(vals) > n else 0.0,
            'easeFrom': vals[n + 1] if len(vals) > n + 1 else 0.0,
        }
    return anims


def strip_block(line):
    """The command is what's left once the animate block is removed."""
    return re.sub(r'\[[^\]]*\]', ' ', line).split()


def compile_script(path):
    preload, cues = [], []
    origin = 0.0
    for raw in open(path, encoding='latin1'):
        line = raw.strip()
        if not line or line.startswith('#'):
            continue
        if line.startswith('@origin'):
            t = to_seconds(line.split()[1])
            if t is not None:
                origin = t
            continue

        tok = strip_block(line)
        if len(tok) < 2:
            continue

        if PRELOAD.match(tok[0]):
            preload.append({'command': tok[2].lower(), 'args': tok[3:]} if len(tok) > 2 else None)
            continue

        t = to_seconds(tok[0])
        if t is None:
            continue
        layer = tok[1]
        rest = tok[2:]
        cmd = rest[0].lower() if rest else None
        cues.append({
            'time': round(origin + t, 4),
            'layer': layer,
            'command': cmd if cmd in DRAW or cmd in ('stopall',) else None,
            'args': rest[1:] if rest else [],
            'anim': parse_animations(line),
            'reset': '[resetTimer]' in line,
        })

    preload = [p for p in preload if p]
    cues.sort(key=lambda c: c['time'])
    return preload, cues


def build_instances(cues):
    """Group the flat cue list into per-layer instances.

    An instance begins at a draw/show command and runs until the layer is
    cleared by `stopAll` or replaced by another command. Every animate on
    that layer in between becomes a keyframe on the instance.
    """
    live, done = {}, []

    def close(layer, t):
        inst = live.pop(layer, None)
        if inst:
            inst['end'] = t
            done.append(inst)

    for c in cues:
        if c['command'] == 'stopall':
            for layer in list(live):
                close(layer, c['time'])
            continue

        layer = c['layer']
        if c['command']:
            close(layer, c['time'])
            live[layer] = {
                'layer': int(layer) if layer.lstrip('-').isdigit() else layer,
                'command': c['command'],
                'args': c['args'],
                'start': c['time'],
                'tracks': {},
            }
        inst = live.get(layer)
        if not inst:
            continue
        for prop, a in c['anim'].items():
            inst['tracks'].setdefault(prop, []).append({
                't': c['time'], 'v': a['v'],
                'easeTo': a['easeTo'], 'easeFrom': a['easeFrom'],
            })

    for layer in list(live):
        close(layer, 1e9)

    # Red seeds a baseline key when the task is created. After splicing that
    # cue's own animate{} blocks in, it walks the five property lists
    # (0x1000371a onwards) and, for each one still *empty*, allocates a key
    # at the cue's time using that property's default constructor:
    #
    #   translation 0x10004410 -> (0, 0, 0)      color 0x10004480 -> (255, 255, 255)
    #   rotation    0x10004430 -> (0, 0, 0)      alpha 0x100044b0 -> 255
    #   scale       0x10004450 -> (1, 1, 1)
    #
    # with the ease terms zeroed, so the span into the first authored key is
    # linear. Without it a lone later key would just snap, and half the
    # script's motion is written that way: `zonnetje` has one rotation key,
    # `0 0 360` at 16.253 s, and the ten `rotator_*` layers one each of
    # `0 0 720` — a full turn and two turns ramped up from the baseline.
    for inst in done:
        for prop, keys in inst['tracks'].items():
            if keys and keys[0]['t'] > inst['start']:
                keys.insert(0, {'t': inst['start'], 'v': list(PROP_DEFAULT[prop]),
                                'easeTo': 0.0, 'easeFrom': 0.0})

    done.sort(key=lambda i: (i['start'], i['layer'] if isinstance(i['layer'], int) else 0))
    return done


def build_images(release, out, preload):
    """Copy the artwork, merging each colour/opacity pair into one RGBA PNG."""
    src = os.path.join(release, 'images')
    dst = os.path.join(out, 'images')
    os.makedirs(dst, exist_ok=True)

    modes, masks = {}, {}
    for p in preload:
        if p['command'] != 'loadimage' or not p['args']:
            continue
        name = p['args'][0].lower()
        modes[name] = (p['args'][1].lower() if len(p['args']) > 1 else 'none')
        if len(p['args']) > 2 and p['args'][2] not in ('-', ''):
            masks[name] = p['args'][2].lower()

    def find(name):
        for cand in os.listdir(src):
            if cand.lower() == name.lower():
                return os.path.join(src, cand)
        return None

    manifest = {}
    for name, mode in sorted(modes.items()):
        path = find(name)
        if not path:
            print('  missing:', name)
            continue
        im = Image.open(path).convert('RGB')
        mask_name = masks.get(name)
        out_name = os.path.splitext(name)[0] + '.png'
        if mask_name:
            mp = find(mask_name)
            if mp:
                mk = Image.open(mp).convert('L').resize(im.size)
                im = im.convert('RGBA')
                im.putalpha(mk)
        im.save(os.path.join(dst, out_name))
        manifest[name] = {'file': 'images/' + out_name, 'mode': mode,
                          'size': list(im.size), 'mask': bool(mask_name)}
    return manifest


def build_scene(scene, wanted):
    """Flatten one .i3d into draw-ready arrays.

    Geometry stays indexed, which matters: expanded to independent corners
    the nine scenes come to 5.9 MB of JSON, indexed 1.8 MB. The exception
    is a mesh that both carries a base texture and stores one UV per
    triangle corner — those have to be expanded, and the index list is
    then implicit.

    Texture coordinates are dropped entirely for the meshes nothing
    samples: most of these materials work through a reflection map, which
    generates its own. That also sidesteps the seam the exporter leaves
    behind by omitting 3ds Max's separate map-face table, which is why a
    few meshes carry slightly more UVs than vertices.
    """
    mats = []
    for m in scene['materials']:
        slots = {t['slot']: t.get('file') for t in m['textures'] if t.get('file')}
        base = slots.get(3) or slots.get(0) or slots.get(1)
        env = slots.get(11) or slots.get(8)
        for f in (base, env):
            if f:
                wanted.add(f.lower())
        mats.append({
            'name': m['name'],
            'base': base and os.path.splitext(f_lower(base))[0] + '.png',
            'env': env and os.path.splitext(f_lower(env))[0] + '.png',
            'color': [round(x, 4) for x in m['color']],
            'opacity': round(m['opacity'], 4),
            'additive': bool(m.get('additive')),
            'wire': bool(m.get('wire')),
        })

    meshes = []
    for m in scene['meshes']:
        nv, nf, nuv = len(m['verts']), len(m['faces']), len(m['uvs'])
        mat = mats[m['material']] if m['material'] is not None and m['material'] < len(mats) else None
        # texture coordinates are only worth carrying when something
        # samples them: a reflection-mapped surface generates its own
        needs_uv = bool(mat and mat['base']) and nuv > 0
        per_corner = needs_uv and nuv == 3 * nf
        if per_corner and uvs_are_degenerate(m['uvs'], m['faces']):
            m['uvs'] = quad_unwrap(m['faces'])
        normals = smooth_normals(m['verts'], m['faces'])

        if per_corner:
            # one UV per triangle corner, so the geometry has to be
            # expanded and the index list is implicit
            pos, nrm, uv, idx = [], [], [], None
            for fi, f in enumerate(m['faces']):
                for ci, vi in enumerate(f):
                    v = m['verts'][vi] if vi < nv else [0, 0, 0]
                    pos.extend(round(x, 2) for x in v)
                    nrm.extend(normals[vi] if vi < len(normals) else [0, 0, 1])
                    t = m['uvs'][fi * 3 + ci]
                    uv.extend((round(t[0], 4), round(t[1], 4)))
        else:
            pos, nrm, idx = [], [], []
            for v in m['verts']:
                pos.extend(round(x, 2) for x in v)
            for n in normals:
                nrm.extend(n)
            for f in m['faces']:
                idx.extend(i if i < nv else 0 for i in f)
            uv = []
            if needs_uv:
                for i in range(nv):
                    t = m['uvs'][i] if i < nuv else [0, 0]
                    uv.extend((round(t[0], 4), round(t[1], 4)))

        tr = m['transform'] or {}
        meshes.append({
            'name': m['name'], 'material': m['material'],
            'pos': [round(x, 3) for x in tr.get('pos', [0, 0, 0])],
            'rot': [round(x, 5) for x in tr.get('rot', [0, 0, 0, 1])],
            'scale': [round(x, 4) for x in tr.get('scale', [1, 1, 1])],
            'positions': pos,
            'normals': nrm,
            'indices': idx,
            'uvs': uv or None,
            'morph': pack_morph(m.get('morph'), per_corner, m['faces']),
            'anim': pack_anim(m['anim']),
        })

    lights = []
    for l in scene['lights']:
        lights.append({
            'name': l['name'],
            'pos': [round(x, 3) for x in l.get('pos', [0, 0, 0])],
            'target': [round(x, 3) for x in l.get('target', [0, 0, 0])],
            'spot': bool(l['spot']),
            'cutoff': round(l['cutoff'], 3),
            'diffuse': [round(x, 4) for x in l['diffuse']],
            'anim': pack_anim(l['anim']),
        })

    cams = []
    for c in scene['cameras']:
        cams.append({
            'name': c['name'], 'fov': round(c['fov'], 4),
            'pos': [round(x, 4) for x in c.get('pos', [0, 0, 0])],
            'target': [round(x, 4) for x in c.get('target', [0, 0, 0])],
            'anim': pack_anim([a for a in c['anim'] if not is_target(a['node'])]),
            'targetAnim': pack_anim([a for a in c['anim'] if is_target(a['node'])]),
        })

    return {'frameEnd': scene['frameEnd'], 'fps': scene.get('fps', 30),
            'materials': mats, 'meshes': meshes, 'cameras': cams, 'lights': lights}


def uvs_are_degenerate(uvs, faces):
    """True when most triangles enclose no area in UV space.

    cubes.i3d is like this: the chunk parses to the byte, but ten of every
    twelve triangles collapse to a line, so no indexing of the stored data
    can map a texture onto a face. The exporter dropped 3ds Max's map-face
    table and what is left cannot be reassembled.
    """
    bad = 0
    for fi in range(len(faces)):
        a, b, c = uvs[fi * 3:fi * 3 + 3]
        area = abs((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]))
        if area < 1e-6:
            bad += 1
    return bad * 2 > len(faces)


def quad_unwrap(faces):
    """Map each pair of triangles across the whole 0..1 square.

    A reconstruction, not a transcription — but these meshes are boxes
    built from quad pairs and their texture is drawn as a face panel, so
    this is the mapping the artwork was made for.
    """
    out = []
    for i in range(0, len(faces), 2):
        out.extend([(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)])
        if i + 1 < len(faces):
            out.extend([(1.0, 1.0), (0.0, 1.0), (0.0, 0.0)])
    return out


def smooth_normals(verts, faces):
    """One normal per vertex, averaged over the faces meeting at it. The file only carries face normals — Energy3D computes vertex
    normals itself in c3dObject::CalcNormals — and the reflection maps that
    most of these materials use need smooth ones."""
    acc = [[0.0, 0.0, 0.0] for _ in verts]
    for f in faces:
        try:
            a, b, c = (verts[i] for i in f)
        except IndexError:
            continue
        u = [b[i] - a[i] for i in range(3)]
        v = [c[i] - a[i] for i in range(3)]
        n = [u[1] * v[2] - u[2] * v[1],
             u[2] * v[0] - u[0] * v[2],
             u[0] * v[1] - u[1] * v[0]]
        for i in f:
            for j in range(3):
                acc[i][j] += n[j]
    out = []
    for n in acc:
        l = math.sqrt(n[0] ** 2 + n[1] ** 2 + n[2] ** 2) or 1.0
        out.append([round(n[0] / l, 3), round(n[1] / l, 3), round(n[2] / l, 3)])
    return out


def f_lower(name):
    return name.lower()


def is_target(node):
    return node.lower().endswith('.target')


def pack_morph(morph, per_corner, faces):
    """The baked vertex animation, flattened to one run per frame.

    Meshes expanded to per-corner have to have each snapshot expanded the
    same way; indexed ones can keep the snapshot as it is stored.
    """
    if not morph:
        return None
    out = []
    for frame in morph['frames']:
        flat = []
        if per_corner:
            for f in faces:
                for vi in f:
                    v = frame[vi] if vi < len(frame) else [0, 0, 0]
                    flat.extend(round(x, 2) for x in v)
        else:
            for v in frame:
                flat.extend(round(x, 2) for x in v)
        out.append(flat)
    return {'step': morph['step'], 'at': morph['at'], 'frames': out}


def pack_anim(anims):
    """One node's animation.

    Position and scale are sparse spline keys and keep their frame numbers
    and TCB terms. Rotation is already baked to one quaternion per frame by
    the exporter, so it goes out as a first frame and a flat run.
    """
    out = {}
    for a in anims:
        for prop in ('pos', 'scale'):
            if a[prop]:
                out[prop] = [{
                    'f': k['f'],
                    'v': [round(x, 4) for x in k['v']],
                    'tcb': [round(x, 4) for x in k['tcb']],
                } for k in a[prop]]
        if a['rot']:
            flat = []
            for k in a['rot']:
                flat.extend(round(x, 5) for x in k['q'])
            out['rot'] = {'first': a['rot'][0]['f'], 'q': flat}
    return out or None


def build_scenes(release, out):
    """Convert the nine .i3d scenes and copy the textures they name."""
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    import i3d

    src = os.path.join(release, 'scenes')
    dst = os.path.join(out, 'scenes')
    os.makedirs(dst, exist_ok=True)
    wanted, built = set(), {}
    for f in sorted(os.listdir(src)):
        if not f.lower().endswith('.i3d'):
            continue
        s = build_scene(i3d.load(os.path.join(src, f)), wanted)
        built[f.lower()] = s
        with open(os.path.join(dst, os.path.splitext(f)[0] + '.json'), 'w') as fh:
            json.dump(s, fh, separators=(',', ':'))

    # The artwork xotrack's effects load for themselves. These never pass
    # through `loadImage`, so the script never names them — they are string
    # constants in Real.exe, each referenced from one effect's init.
    wanted |= EFFECT_ASSETS

    tdst = os.path.join(out, 'textures')
    os.makedirs(tdst, exist_ok=True)
    idir = os.path.join(release, 'images')
    have = {n.lower(): n for n in os.listdir(idir)}
    missing = []
    for name in sorted(wanted):
        real = have.get(name)
        if not real:
            missing.append(name)
            continue
        Image.open(os.path.join(idir, real)).convert('RGB').save(
            os.path.join(tdst, os.path.splitext(name)[0] + '.png'))
    return built, sorted(wanted), missing


def main(release, out):
    os.makedirs(out, exist_ok=True)
    preload, cues = compile_script(os.path.join(release, 'real.scp'))
    instances = build_instances(cues)
    images = build_images(release, out, preload)
    scenes, textures, missing_tex = build_scenes(release, out)

    shutil.copyfile(os.path.join(release, 'real.mp3'), os.path.join(out, 'real.mp3'))

    cfg = open(os.path.join(release, 'energy3d.cfg')).read().split()
    timeline = {
        'width': int(cfg[0]), 'height': int(cfg[1]),
        'length': max(c['time'] for c in cues),
        'preload': preload,
        'instances': instances,
        'images': images,
    }
    with open(os.path.join(out, 'timeline.json'), 'w') as f:
        json.dump(timeline, f, indent=1)

    kinds = {}
    for i in instances:
        kinds[i['command']] = kinds.get(i['command'], 0) + 1
    print('%d cues -> %d instances over %.1f s' % (len(cues), len(instances), timeline['length']))
    print('  %d images (%d with a separate opacity map)' %
          (len(images), sum(1 for v in images.values() if v['mask'])))
    print('  ' + ', '.join('%s:%d' % kv for kv in sorted(kinds.items())))
    print('  %d scenes, %d meshes, %d scene textures%s' % (
        len(scenes), sum(len(s['meshes']) for s in scenes.values()),
        len(textures) - len(missing_tex),
        (' (missing %s)' % ', '.join(missing_tex)) if missing_tex else ''))


if __name__ == '__main__':
    main(sys.argv[1], sys.argv[2])
