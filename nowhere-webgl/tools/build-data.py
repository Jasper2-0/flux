#!/usr/bin/env python3
"""Turn a Nowhere release directory into the port's data/ tree.

    build-data.py <release-dir> <out-dir>

Unpacks nowhere.stv, converts PCX to PNG (browsers can't read PCX),
parses the .zeu scenes to JSON, and compiles script.txt into the
timeline the runtime plays.
"""
import json
import os
import re
import shutil
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from unstv import read_toc, unpack_entry          # noqa: E402
from unzeu import parse as parse_zeu              # noqa: E402

from PIL import Image                             # noqa: E402


# ---------------------------------------------------------------- script

def strip_comments(text):
    text = re.sub(r'/\*.*?\*/', ' ', text, flags=re.S)
    return re.sub(r'//[^\n]*', '', text)


def load_script(files, name, defines):
    """Expand #include / #define exactly as cAxScript does."""
    out = []
    for line in strip_comments(files[name.lower()]).splitlines():
        line = line.strip()
        if not line:
            continue
        if line.lower().startswith('#define'):
            parts = line.split()
            if len(parts) >= 3:
                # the engine lowercases every token, and the script does
                # lean on that: PREDEMO is spelled PREDEMo in six places
                defines[parts[1].lower()] = parts[2]
            continue
        if line.lower().startswith('#include'):
            inc = line.split(None, 1)[1].strip()
            out += load_script(files, norm_path(inc), defines)
            continue
        out.append(line)
    return out


def norm_path(p):
    return p.replace('\\', '/').lstrip('./').lower()


NUMERIC = re.compile(r'^-?\d+$')


def compile_script(files):
    defines = {}
    lines = load_script(files, 'script.txt', defines)

    timeline, registrations, sound = [], {}, {}
    for line in lines:
        tok = line.split()
        head = tok[0]
        low = head.lower()

        if low == 'register' and len(tok) >= 3:
            registrations[tok[1].lower()] = tok[2].lower()
            continue
        if low == 'sound' and len(tok) >= 3:
            sound[tok[1].lower()] = tok[2]
            continue

        # a cue line: <time|define> <instance> <param> [value]
        raw = defines.get(low, head)
        if not NUMERIC.match(raw) or len(tok) < 3:
            continue
        value = tok[3] if len(tok) > 3 else ''
        value = defines.get(value.lower(), value)
        timeline.append({
            'ms': int(raw),
            'instance': tok[1].lower(),
            'param': tok[2].lower(),
            'value': value,
        })

    timeline.sort(key=lambda e: e['ms'])
    return {'registrations': registrations, 'sound': sound, 'timeline': timeline}


# ------------------------------------------------------- materials/textures

MAT_FLAGS = {'additive', 'calcenv', 'envcalc', 'envmodulate', 'nomipmap', 'wrap', 'clamp'}


def compile_materials(files):
    """The per-part scripts declare textures and materials in a small
    block language; collect them all into one table."""
    textures, materials = {}, {}
    for name, text in files.items():
        if not name.endswith('.txt'):
            continue
        body = strip_comments(text)
        # material blocks contain their own `texture <name>` lines; blank
        # them out first so they don't shadow the real declarations
        outside = re.sub(r'material\s*\{.*?\}', ' ', body, flags=re.S | re.I)
        for m in re.finditer(r'^\s*texture\s+(\S+)\s+(\S+)\s+(\S+)(.*)$', outside, re.M | re.I):
            nm, slot, path, rest = m.groups()
            textures[nm.lower()] = {
                'slot': slot.lower(),
                'file': norm_path(path),
                'flags': [f.lower() for f in rest.split() if f.lower() in MAT_FLAGS],
            }
        for m in re.finditer(r'material\s*\{(.*?)\}', body, re.S | re.I):
            mat = {'textures': [], 'flags': [], 'cull': 'ccw'}
            nm = None
            toks = m.group(1).split()
            i = 0
            while i < len(toks):
                t = toks[i].lower()
                if t == 'name':
                    nm = toks[i + 1].lower(); i += 2
                elif t == 'texture':
                    mat['textures'].append(toks[i + 1].lower()); i += 2
                elif t == 'cull':
                    mat['cull'] = toks[i + 1].lower(); i += 2
                else:
                    if t in MAT_FLAGS:
                        mat['flags'].append(t)
                    i += 1
            if nm:
                materials[nm] = mat
    return textures, materials


# ------------------------------------------------------------------ main

def main(release, out):
    lib = open(os.path.join(release, 'nowhere.stv'), 'rb').read()
    entries = read_toc(lib)
    blobs = {norm_path(e['name']): unpack_entry(lib, e) for e in entries}

    os.makedirs(out, exist_ok=True)
    files = {k: v.decode('latin1') for k, v in blobs.items() if k.endswith('.txt')}

    manifest = {'images': {}, 'scenes': {}, 'animations': {}}

    for name, blob in sorted(blobs.items()):
        dst = os.path.join(out, name)
        os.makedirs(os.path.dirname(dst) or out, exist_ok=True)
        if name.endswith('.jpg'):
            open(dst, 'wb').write(blob)
            im = Image.open(dst)
            manifest['images'][name] = list(im.size)
        elif name.endswith('.pcx'):
            # PCX is an alpha mask or a sprite frame; browsers need PNG
            tmp = dst + '.pcx'
            open(tmp, 'wb').write(blob)
            im = Image.open(tmp).convert('L' if 'anim' not in name else 'RGB')
            png = dst[:-4] + '.png'
            im.save(png)
            os.remove(tmp)
            manifest['images'][name[:-4] + '.png'] = list(im.size)
        elif name.endswith('.zeu'):
            scene = parse_zeu(blob)
            for o in scene['objects']:
                o.pop('_consumed', None); o.pop('_size', None)
            js = dst[:-4] + '.json'
            with open(js, 'w') as f:
                json.dump(scene, f, separators=(',', ':'))
            manifest['scenes'][name[:-4] + '.json'] = sum(
                len(o.get('vertices', [])) for o in scene['objects'])
        elif name.endswith('.txt'):
            open(dst, 'w').write(blob.decode('latin1'))
            lines = [norm_path(l) for l in blob.decode('latin1').split() if l.strip()]
            if lines and all(l.endswith('.pcx') for l in lines):
                manifest['animations'][name] = [l[:-4] + '.png' for l in lines]

    script = compile_script(files)
    textures, materials = compile_materials(files)
    script['textures'] = textures
    script['materials'] = materials
    script['manifest'] = manifest
    with open(os.path.join(out, 'timeline.json'), 'w') as f:
        json.dump(script, f, indent=1)

    shutil.copyfile(os.path.join(release, 'sound.vic'), os.path.join(out, 'sound.mp3'))

    print('%d entries -> %s' % (len(entries), out))
    print('  %d images, %d scenes, %d sprite animations' %
          (len(manifest['images']), len(manifest['scenes']), len(manifest['animations'])))
    print('  %d timeline cues, %d instances, %d materials, %d textures' %
          (len(script['timeline']),
           len(set(e['instance'] for e in script['timeline'])),
           len(materials), len(textures)))


if __name__ == '__main__':
    main(sys.argv[1], sys.argv[2])
