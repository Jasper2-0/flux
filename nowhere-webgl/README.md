# Nowhere by Threestate — WebGL port

A browser port of **Nowhere** (Threestate, Takeover 2000, PC demo), built
on the same engine layer as the Tesla and Contour ports in
`../tesla-webgl/` and `../contour-webgl/`. The feasibility analysis that
preceded it is in `../nowhere/ANALYSIS.md`.

Unlike the other two, this port does not reconstruct a timeline: **it
runs the demo's own script.** Nowhere shipped its script as text inside
the data library, so the port compiles that text and plays it cue for
cue.

## Running

```sh
cd nowhere-webgl
python3 -m http.server 8000
# open http://localhost:8000/
```

## What the runtime is

`js/demo.js` is Axiom's script player. The script is a flat list of

```
<time in ms>   <instance>   <parameter>   <value>
```

with `PREDEMO` (`-1000000`) as the preload pass, `REGISTER` binding an
instance name to an effect plugin, `execute load` loading whatever
`filename` was last set, and `run true|false` showing and hiding. That is
the whole language, and `tools/build-data.py` compiles it —
`#define`, `#include` and comments included — into `data/timeline.json`:
196 cues across 32 instances.

The clock is the shipped soundtrack. `sound.vic` is a plain 128 kbps MP3
under a different extension; the original drove its clock from the MP3
byte position and this port does the same from `audio.currentTime`.

One correction the data forced: the plugins' help text says "high numbers
are drawn before low numbers", but the script only makes sense the other
way round — backgrounds sit at layer 0, the 3D parts at layer 1, the
sprite overlays at layer 2. Ascending order it is.

## What renders

| part | time | state |
| --- | --- | --- |
| backgrounds (12 of them) | throughout | **done** — 640×480 JPEG, optional PCX alpha mask |
| sprite overlays (6) | throughout | **done** — 4-frame PCX loops at 12 fps, placed by `tlx`/`tly` |
| boarder | 1:40–1:54 | **done** — `zeusPlay` on `boarder.zeu` |
| sphere | 0:45–0:59 | **done** — per-triangle prisms pulsing between the two geospheres |
| credits ×3 | 0:08–0:45 | **done** — fisheye-mapped hemispheres, additive over `sbs` |
| flower ×2 | 1:12–1:40 | **ball done** — two moving bumps displace the sphere; poles and TV movie pending |
| greets | 1:54–2:08 | scene only |
| fire | 2:21–2:49 | scene only |
| smoke | 2:49–3:02 | scene only |
| slierten | 3:02–3:16 | scene only |
| sbs (plasma) | 0:00–0:45, 3:16–3:26 | not yet |
| kaleido | 0:59–1:12 | not yet |
| kubus | 2:08–2:21 | not yet |

"Scene only" means the part's `.zeu` is played straight — right geometry,
right camera, right timing — but without the procedural deformation the
plugin applies on top. The corner readout labels those honestly while
they are in that state.

## The `.zeu` scene format

`tools/unzeu.py` parses it; the layout was read off Zeus.dll's chunk
dispatcher (`0x10006d00`) and handler table (`0x10014030`):

```
u16 version
chunk { u16 id · u32 size (includes the header) · payload }, nested
  0x1000 SCENE   0x2000 MESH   0x3000 CAMERA   0x4000 LIGHT   0x5000 HELPER
```

- Every object starts with two length-prefixed strings, name and parent.
- A mesh carries a **23-byte vertex** — `f32 x,y,z` · `u8 r,g,b` ·
  `f32 u,v` — a `u16` triangle list, and *buckets*: one material group
  per draw, each listing its polygons, with vertices re-indexed locally
  exactly as the loader does it.
- Animation is `cSpline` key lists: `u8 loop · u32 count · key[]`, where
  a key is the raw `sKey` struct — an `i32` **frame number**, five TCB
  floats, then the payload. Three floats for a vector, four for a
  quaternion, one for a scalar.
- Meshes carry position, rotation and scale splines; cameras carry
  position plus two scalars, **roll in radians then field of view in
  degrees**; camera targets come through as lights named `Target`.

All eleven scenes parse with every chunk consuming exactly its declared
size, which is what makes the layout trustworthy rather than plausible.

`js/zeus.js` evaluates them. Zeus's spline is Kochanek–Bartels: 586 of
the 628 keys in these scenes have all-zero TCB and degenerate to
Catmull-Rom, but 42 do not, so the full form is implemented.

## The sphere, decoded

`sphere.srx` asks the scene for two objects by name and will not run
without them: `GeoSphere01` ("Please name the minimal object") and
`GeoSphere02` ("the maximal object"), which share topology — 418
vertices, 720 triangles, radius 50 and 200. At load it packs one
0x110-byte record per triangle holding *both* copies of it, and each
frame it computes

    phase = (i & 15) + 10                     // 10..25, per triangle
    blend = (sin(phase · k · π · time · 0.01) + 1) / 2

then rebuilds the mesh by lerping the maximal triangle towards the
minimal one — `(B − A) · blend + A`, coordinate by coordinate — and
allocates 6 vertices and 7 polygons per triangle: the base on the small
sphere, the moving triangle at the tip, and the walls between. A ball of
spikes, each breathing at its own rate.

`k` is drawn per triangle from `rand() / 43690 + 0.2`. MSVC's `rand()`
tops out at 32767, so that division is always zero and every triangle
gets exactly 0.2 — a latent bug in the original. The port reproduces it,
because reproducing it is what matches the release.

## The credits, decoded

`credits.srx` never moves a vertex. Its scene holds two hemispheres —
`GeoSphere01` and `GeoSphere02`, 751 vertices each, radius 360 — plus
`Rectangle01`, an 800×100 plate carrying the credit name that the scene
animates itself. Every frame the plugin rewrites each hemisphere's
texture coordinates and vertex colour, normalising position by that same
360:

    ang = atan2(Z, X)                       r = hypot(X, Z)
    s   = 2·|X|·r^1.2 + 1
    ang = (−π/2 < ang ≤ π/2) ? ang/s : (ang − π)/s + π
    u   = asin(asin(cos(ang)·r·0.8))·0.25 + time/75 − 2 + 0.5
    v   = 0.5 − sin(ang)·r / (2.4 − 1.8·r)
    grey = clamp(Y − 1.2·r, 0.1, 1)

— a pinch centred on the poles, so the lettering in `credvicbol.jpg`
wraps and lenses across the dome, lit by height minus radius and walked
slowly sideways. The routine also carries two earlier passes that scroll
`u` directly from a saved copy of the original coordinates; both are
overwritten by the fisheye passes before anything is drawn, so they are
dead code in the shipped build and are not reproduced.

The credits render *additively over the `sbs` plasma*, which runs
underneath them for the whole first 45 seconds. Until `sbs` exists they
float on black, which is faithful but looks far darker than the release.

## The flower's ball, decoded

`33` generates most of its own geometry. What is implemented is the ball
(`0x10001d20`), which rewrites GeoSphere01's texture coordinates and then
pushes every vertex out along its own direction by two moving circular
bumps:

    scale = sin(time / bolturbdiv) · 0.15 + 0.75
    u = (x/70 + 0.5)·scale        v = (y/70 + 0.5)·scale

    a = time / bolanglediv
    bump(P, Q, gain):
        p = wrap(P + u·255) − 128     q = wrap(Q + v·255) − 128
        return max(0, 255 − hypot(p, q)·512/182) / 255 · gain
    r = bolscale + bump(50a, −31.8835a, 0.5) + bump(−23.796a, 34.4485a, 0.6)
    vertex ← source vertex · r

`wrap(t) = t − trunc(t) + (trunc(t) & 255)` is the binary's way of
folding a float into [0, 256) with an `and eax, 0xff`. The script sets
bolscale 1, bolturbdiv 32, bolanglediv 56; the constructor's defaults are
0.4, 50 and 60.

Still missing from this part: the **eighteen poles** the effect builds
itself — `0x10001fc0` allocates 18 tubes of 41×3 vertices and 240
triangles each, and `0x10001930` sways them using `center` and `uitslag`
("maximale uitslag van de paal") — and the movie playing on the four TV
screens, which `movietime` divides the frame rate for. Those parts of the
scene currently render straight.

## What is reconstruction, and what is read from the data

Read from the data and solid: the script and all its cue times, the
material and texture declarations, every scene's geometry, hierarchy,
camera track and keyframes, the effect parameter names and their
documented meanings, and the soundtrack.

Reconstructed, and worth doubting:

- **The scene frame rate.** Zeus stores frame numbers; nothing in the
  data says how fast they advance. This port uses 30 fps
  (`SCENE_FPS` in `js/zeus.js`). It is close for the boarder section —
  450 frames over 13.75 s wants 32.7 — but the credits want 20 and the
  smoke scene 53, so a single global rate cannot be what the original
  did. This needs a capture to settle.
- **The ease-in/ease-out reparameterisation** inside a spline span.
  Zeus's `CompAB` is not recovered; the classic Max ease is used.
- **Quaternion interpolation** is slerp where Zeus squads. With the TCB
  values these scenes carry the difference is under a degree.
- **Sprite blending.** The overlay frames are white line art on black and
  are drawn additively, which is what makes them read as overlays; the
  plugin's own blend state has not been recovered.
- **Lighting.** Zeus scenes carry lights and `CalcNormals` is called on
  rebuilt geometry, but nothing here is lit yet — everything is drawn
  flat-textured. The sphere in particular reads as a silhouette because
  of it.
- **The sphere's clock unit.** Its blend runs off the effect's own timer
  scaled by 0.01; taken as seconds that puts the slowest triangles on a
  100-second cycle and the fastest on 40, which is a slow staggered
  bloom across the section. Milliseconds would make it a blur, so
  seconds it is, but a capture would settle it.

## Tooling

- `tools/unstv.py` — unpacks the `.stv` data library (TOC with
  XOR-obfuscated names, aPLib-packed blobs). 97/97 entries, 65/65 images.
- `tools/unzeu.py` — parses `.zeu` scenes to JSON.
- `tools/build-data.py` — runs both, converts PCX to PNG, compiles the
  script, and writes `data/`.

Rebuild the data tree from a release directory with:

```sh
python3 tools/build-data.py /path/to/nowhere data
```

## Provenance

`data/` holds assets from the freely distributed Nowhere release —
artwork and code by sarix and stevie, music by vic, backing code by
sagacity, with an "extreme coding session" credited to kalms. © 2000
Threestate; included here for preservation and study of a freely
released demoscene production.
