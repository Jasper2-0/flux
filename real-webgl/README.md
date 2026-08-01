# We Ain't Real — WebGL port

Solar's demo from Bizarre 2000, running in a browser. Design and graphics
by Coat, code by Druid and xotrack, music by Tim Brandwijk.

This is a port, not an emulator: the script, the scenes and the artwork are
the originals, and everything that consumed them — Red's layer runtime,
Energy3D's scene playback, xotrack's eight effects, and enough of
fixed-function OpenGL 1.x to host them — was read out of the binaries and
rewritten in JavaScript.

## Running it

`data/` is already built and committed, so the demo runs straight from a
static server:

```
python3 -m http.server 8620      # then open http://localhost:8620/
```

Sound is behind a click, as browsers require. The whole thing is 3:40.

To rebuild `data/` from a copy of the release:

```
python3 tools/build-data.py <release-dir> data     # needs Pillow
```

Two single-file bundles, for handing the demo to someone as one HTML file:

```
node build-single.mjs            # -> real-single.html   (data: URIs)
node build-artifact.mjs          # -> real-artifact.html (same, CSP-safe)
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
All eight are ported:

| effect | init | run | what it is |
| --- | --- | --- | --- |
| `show2d` | `0x401000` | `0x401160` | a textured grid deformed through its texture coordinates |
| `showBol` | `0x401d70` | `0x4021b0` | the geosphere, exported from Max because nobody wanted to write one |
| `showCylinder` | `0x402350` | `0x4026a0` | wireframe cylinder over counter-scrolling bands of text |
| `showDraai` | `0x4029f0` | `0x402a30` | *draai* — a spinning textured quad |
| `showLines` | `0x402c60` | `0x402e80` | per-frame jittered scanlines, in the 2D frame |
| `showPartiekels` | `0x403150` | `0x403220` | the particle field |
| `showPlanes` | `0x403590` | `0x4036d0` | billboards marching out of the distance |
| `showTunnel` | `0x4039e0` | `0x403e60` | the displaced tube |

## Some things the disassembly settled

**There are two drawing frames, and the task's *kind* picks one.** The
second argument to `red_base::add_task` is 1 for a 2D task and 2 for a 3D
one, and Red keeps a standing projection for each. That one integer is
why `showLines` lives in pixel coordinates while the other seven get a
perspective frame — and why the port sets up the frame per task rather
than per command.

**A task is born with a keyframe on every property.** After splicing a
cue's own `animate{}` blocks into the layer's five property lists, Red
walks them (`0x1000371a`) and, for each list still *empty*, allocates a key
at that cue's time from the property's default constructor — `(0, 0, 0)`
for translation and rotation, `(1, 1, 1)` for scale, `(255, 255, 255)`
for colour, `255` for alpha, ease terms zeroed. That baseline is what a
lone later key interpolates *from*, and a lot of the script's motion is
written assuming it: `zonnetje` carries one rotation key, `0 0 360` at
16.253 s, and each of the ten `rotator_*` layers one `0 0 720`. Without
the baseline they are single-key tracks that just snap to a whole number
of turns, i.e. sit still — the intro sun stops turning.

**Layers bind to tasks in file order, not clock order.** `red_base::run_tasks`
walks its slot array ascending, and a layer's identity is fixed when the
script is parsed. So the cue list must not be sorted by time on the way in;
`build-data.py` deliberately leaves it alone.

**The second `drawImage` parameter is a centre flag.** Nine of the 121
image cues pass `1` instead of `-`, and `drawImage::run` (`0x100025f0`)
replaces the bitmap's x with `(screenWidth - imageWidth) / 2` and drops
the animated translation x entirely. Without that, the intro sun sits in
the top-left corner.

**The masked artwork is premultiplied.** Colour and opacity ship as
separate JPEGs; `LImage.dll` multiplies them together at load and the
blend is `ONE, ONE_MINUS_SRC_ALPHA`, not `SRC_ALPHA`. A layer's own alpha
therefore has to scale the source *colour* too, or every fade goes bright
before it goes out.

**Reflection maps are added, not substituted.** The sphere-map pass is a
second draw with `depthFunc(EQUAL)` and `ONE, ONE` over the lit surface.
Substituting them, as a naive reading of the texture stage suggests, makes
every chrome object in the demo too dark.

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

## Where the port departs from the release

Three changes, all at Coat's request as the demo's designer, all marked in
the source where they happen:

- **Part 2's bars letterbox the scene.** The script puts `balk.jpg` on
  layers 6 and 7 and `cubes.i3d` on layer 8, and Red's ascending slot walk
  means the cubes spill over the bars as released. The bars are lifted
  above the scene here. Nothing in the binaries supports that reading —
  it is the intent, not the release.
- **Part 13's `onder.jpg` sits at y = 400, not 420.** It is 640×80, so as
  scripted twenty rows hang off the bottom of a 480-line screen.
- **`showPlanes` runs at 75% brightness.** The binary's falloff curve is
  `f³`; the planes were bright enough to swamp their own crossfade, so it
  reads as `f³ × 0.75`.

`BACKLOG.md` keeps the running list of what is settled, what is still
open, and where the evidence for each came from.

## Layout

```
index.html            the page; loads js/ and data/
tools/build-data.py   compile real.scp, merge the colour/opacity JPEG
                      pairs, convert the scenes, copy the textures
tools/i3d.py          the .i3d reader, usable on its own
js/demo.js            the layer runtime: drawImage, colorFade, drawScene
js/scene.js           Energy3D scene playback: KB splines, camera
js/effects.js         xotrack's show* tasks
js/minigl.js          fixed-function OpenGL 1.x over WebGL2
build-single.mjs      inline everything into one HTML file
build-artifact.mjs    same, for hosts with a strict CSP
notes/                the fine-tuning observation sheet
BACKLOG.md            findings, open questions, dead ends
```

## Provenance

The code here is new. `data/` is not: it is derived from the released
demo — Coat's artwork, Tim Brandwijk's soundtrack, Solar's `.i3d`
geometry and `real.scp` — reprocessed by `tools/build-data.py` into a form
a browser can load. The original binaries (`Real.exe`, `Red.dll`,
`Energy3D.dll`, `LImage.dll`) are **not** included; the addresses quoted
throughout refer to the release as distributed at Bizarre 2000.

The demo's assets belong to their authors. If you are not one of them,
ask before redistributing.
