# We Ain't Real — WebGL port

Solar's demo from Bizarre 2000, running in a browser. Design and graphics
by Coat, code by Druid and xotrack, music by Tim Brandwijk.

```
python3 tools/build-data.py <release-dir> data
python3 -m http.server 8620      # then open index.html
```

## What the original is made of

Four pieces, and each one gave up something different.

**`real.scp`** is the entire timeline in plain text, and it documents its
own grammar in a comment at the top. A cue is a time, a layer number, an
optional `[animate{...}]` block and a command. A `drawImage`, `drawScene`,
`colorFade` or `show*` command claims a layer; later cues on the same
layer carry only animation; `stopAll` clears everything. `@origin` rebases
the clock so each section can be written from zero and then placed.
`tools/build-data.py` compiles that into 205 layer instances over 220
seconds.

**`Red.dll`** is the engine that runs the script — "Red v2.6.0+bass by
Druid / Nostalgia", built 30 September 2000 — and it ships with its MSVC
decorated names intact. That is why the 2D path in `js/demo.js` is
transcribed rather than guessed: the ortho frame, the blend modes, the
centre flag on `drawImage`, the keyframe ease.

**`Energy3D.dll`** is the renderer, likewise symbol-rich, and behind it
the `.i3d` scene container: a flat chunk stream behind a `0xDEAD` header.
`tools/i3d.py` reads it out of `energy3d_i3d_loader::load`. Nine scenes,
157 meshes, 39,000 triangles.

**`Real.exe`** is only 64 KB and holds xotrack's eight `show*` effects.
One of the eight is ported so far; see `BACKLOG.md`.

## Four things the disassembly settled

**A task is born with a keyframe on every property.** After splicing a
cue's own `animate{}` blocks into the layer's five property lists, Red
walks them (0x1000371a) and, for each list still *empty*, allocates a key
at that cue's time from the property's default constructor — `(0, 0, 0)`
for translation and rotation, `(1, 1, 1)` for scale, `(255, 255, 255)`
for colour, `255` for alpha, ease terms zeroed. That baseline is what a
lone later key interpolates *from*, and a lot of the script's motion is
written assuming it: `zonnetje` carries one rotation key, `0 0 360` at
16.253 s, and each of the ten `rotator_*` layers one `0 0 720`. Without
the baseline they are single-key tracks that just snap to a whole number
of turns, i.e. sit still — the intro sun stops turning.

**The second `drawImage` parameter is a centre flag.** Nine of the 121
image cues pass `1` instead of `-`, and `drawImage::run` (0x100025f0)
replaces the bitmap's x with `(screenWidth - imageWidth) / 2` and drops
the animated translation x entirely. Without that, the intro sun sits in
the top-left corner.

**The ease terms are integers, not percentages.** `spline_tcb::Ease`
loads them with `fild`, so the script's `25` and `150` go in raw; the
routine renormalises anything summing over 1, which is why both produce
exactly the same curve.

**The world is right-handed.** The exporter's `remap` is
`(x, y, z) -> (-x, z, y)`, whose determinant is +1 — so unlike the
Direct3D-targeted exporters of the same era, Max's Z-up right-handed
world lands in a Y-up right-handed one and nothing has to be mirrored on
the way into GL. And a node's rotation is *not* a quaternion despite
occupying four floats: it is an axis and an angle, handed to `remap` and
`fromAngAxis` with the angle negated. Rotation keyframes, confusingly,
*are* quaternions — and baked one per frame with no TCB terms at all.

## Layout

```
tools/build-data.py   compile real.scp, merge the colour/opacity JPEG
                      pairs, convert the scenes, copy the textures
tools/i3d.py          the .i3d reader, usable on its own
js/demo.js            the layer runtime: drawImage, colorFade, drawScene
js/scene.js           Energy3D scene playback: KB splines, camera
js/effects.js         xotrack's show* tasks
js/minigl.js          fixed-function OpenGL 1.x over WebGL2
```
