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


def main(release, out):
    os.makedirs(out, exist_ok=True)
    preload, cues = compile_script(os.path.join(release, 'real.scp'))
    instances = build_instances(cues)
    images = build_images(release, out, preload)

    os.makedirs(os.path.join(out, 'scenes'), exist_ok=True)
    for f in sorted(os.listdir(os.path.join(release, 'scenes'))):
        shutil.copyfile(os.path.join(release, 'scenes', f),
                        os.path.join(out, 'scenes', f))
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


if __name__ == '__main__':
    main(sys.argv[1], sys.argv[2])
