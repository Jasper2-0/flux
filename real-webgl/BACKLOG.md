# We Ain't Real — what's left

Screen times are out of the demo's 3:40.

Everything here can be done from the binaries and the data. This archive
is unusually friendly: the timeline is plain text, both engine DLLs keep
their MSVC decorated names, and `Real.exe` — where the remaining work is —
is 64 KB.

---

## Tier 1 — the seven unported `show*` tasks

All eight of xotrack's effects live in `Real.exe` and register against
Red's task table the same way, so the entry points are already known:
slot 0 of each vtable is the one-time init, slot 1 the per-frame run, and
slot 4 returns the required parameter count.

| effect | vtable | init | run | state | kind | screen time |
|---|---|---|---|---|---|---|
| `showLines` | `0x40c594` | `0x402c60` | `0x402e80` | `0x1c0` | 1 | 50 s — **done** |
| `show2d` | `0x40c580` | `0x401000` | `0x401160` | `0x1c0` | 1 | 38 s |
| `showPartiekels` | `0x40c56c` | `0x403150` | `0x403220` | `0x48` | 2 | 21 s — **done** |
| `showCylinder` | `0x40c558` | `0x402350` | `0x4026a0` | `0x98` | 2 | 11 s — init is a byte-for-byte copy of `showTunnel`'s, same 13x128 tube; only the run differs |
| `showPlanes` | `0x40c530` | `0x403590` | `0x4036d0` | `0x1e0` | 2 | 11 s — **done** |
| `showBol` | `0x40c5a8` | `0x401d70` | `0x4021b0` | `0x64` | 2 | 11 s — **done** |
| `showDraai` | `0x40c544` | `0x4029f0` | `0x402a30` | `0x38` | 2 | 11 s — **done** |
| `showTunnel` | `0x40c5bc` | `0x4039e0` | `0x403e60` | `0x98` | 2 | 11 s — **done** |

### The two drawing frames

The `kind` column is the second argument each task passes to
`red_base::add_task`, read off the registration block at `0x404600`,
where the vtable store and the `add_task` call sit a few instructions
apart. `red_base::run_tasks` (`0x10003d10`) watches it change between
consecutive layers and switches the GL frame:

    kind 2 -> kind 1   glDisable(CULL_FACE / DEPTH_TEST / LIGHTING)
                       PROJECTION: push, identity, glOrtho(0, w-1, h-1, 0, 0, 100)
                       MODELVIEW:  push, identity

    kind 1 -> kind 2   MODELVIEW: pop.  PROJECTION: pop.
                       glEnable(CULL_FACE / DEPTH_TEST / LIGHTING)

So kind 1 is the 2D pixel frame — `drawImage`, `colorFade`, and only two
of the eight effects — and kind 2 draws in whatever 3D frame is standing.

Which frame that is takes some tracing, because almost everything
restores what it borrows. `ogl_bitmap::render` pushes and pops both
matrices (`0x10001f4d` and `0x1000203b`); `showTunnel` and
`showPartiekels` push and pop the projection they set; `showBol`,
`showCylinder` and `showPlanes` never touch the projection at all. The
one thing that sets a projection and walks away is
`ogl_camera::calculate` (`0x10002dc0`), behind `drawScene`. So a kind-2
effect inherits the last scene camera's perspective — and before any
scene has drawn, the `gluPerspective(45, 4/3, 2, 5000)` that
`energy3d_ogl::create_display` set at `0x100011ba`.

That matters in part 8: `showDraai` and `showTunnel` run two sections
after the last `drawScene`, so they inherit `gears.i3d`'s camera.

Notes gathered so far:

- **`show2d` is the big one** and the only one that is not straightforward.
  It keeps a `0x65e78`-byte buffer — big enough for a 640×480 working
  image — and picks between three textures held at `+0x65e6c`, `+0x65e70`
  and `+0x65e74` on the third script parameter. It then compares the
  first parameter against the three joke strings in `.data`
  (`ranzigebotteparamomeeneffecttekiezen` at `0x40e05c`,
  `ditisparameternummertje2endieisookheelergbottoevallig` at `0x40e084`,
  `xotrackiseenbotteaapenhijverzintbotteparameternamen` at `0x40e0bc`) to
  pick a variant. So it is a software effect that uploads a texture each
  frame — the largest single reading job left.

- The assets, from the string table's cross-references — one per init,
  and none of them ever passing through `loadImage`, so the script never
  names them:

  | effect | assets |
  |---|---|
  | `show2d` | `solar_groot.jpg`, `eeeeeeeenv.tga`, `bruut.tga` |
  | `showBol` | `sphere.i3d` |
  | `showPartiekels` | `sphere.i3d`, `particle.jpg` |
  | `showCylinder` | `eeeeeeeenv.tga` |
  | `showTunnel` | `eenv2.tga` |
  | `showPlanes` | `plane_01`–`plane_04.jpg` |
  | `showDraai` | `envmap.jpg` |

  All eleven ship in the release. `build-data.py` copies them alongside
  the scene textures.

- **`sphere.i3d` is loaded by two effects, and they use it differently.**
  Both `load_scene` it, `init_scene`, `activate_camera`, and `memcpy` the
  vertex array into a pristine buffer of their own.

  `showBol` then writes a deformed copy back into the scene's *live*
  array, rebuilds the vertex normals (`0x401e80`), moves the camera and
  calls `energy3d_scene::draw_scene` at `0x402342` — so the engine draws
  it, on the same path any `drawScene` layer takes.

  `showPartiekels` never calls it; it draws the vertices itself.

  `draw_display` genuinely has no call site. `draw_scene` has exactly one,
  and it matters: `draw_scene` (`0x10011860`) reaches the camera's
  `calculate` through the vtable at `0x10011885`, and calculate never
  restores the projection. So `showBol` is the one effect that *sets* the
  standing 3D frame rather than borrowing it, and part 8's `showDraai`
  inherits `sphere.i3d`'s camera — fov 45 — not `gears.i3d`'s 39.6.

  `activate_camera` (`0x10011790`) is only a two-line setter and sets no
  projection by itself; the projection comes from `draw_scene`.

  The mesh is `GeoSphere01`: 1002 vertices, 2000 faces, radius exactly
  50, no UVs, and a valence histogram of twelve 5s and 990 6s — the
  signature of a geodesic sphere, with no poles and no seam. Coat
  exported it from 3ds Max rather than have xotrack write a geosphere
  generator, and inopia is who pointed the group at pole-free spheres for
  effects like these. It matters for the port as well as for the original:
  vertex density is near-uniform, so walking the array in index order is
  spatially even and nothing needs weighting.

- They all take their clock from the message's task-local elapsed
  milliseconds, the same `[msg+0x48]` that drives `drawScene`.

---

## Tier 2 — fidelity gaps in what is already ported

### Camera roll — traced, and dead in the `.i3d` path
`ogl_camera::calculate` issues `glRotatef(roll, 0, 0, 1)` before
`gluLookAt`, reading the roll from `+0x124`. That field has exactly one
setter, `0x10009470`, and exactly one caller: `0x10006f20`, which is
reached from the chunk dispatcher at `0x10008780` — and that dispatcher
switches on `0x3d3d / 0x4000 / 0x4100 / 0x4600 / 0x4700 / 0xafff /
0xb005`. Those are **3DS** chunk ids, not `.i3d` ones. Energy3D can load
raw `.3ds` files too, and roll only arrives on that path.

The `.i3d` camera loader (`0x1000bdc0`, jump table at `0x1000bf70`)
therefore never touches roll: it stays at the zero the constructor writes
at `0x1000cb85`. Cameras render unrolled, which is what the port does.

There *is* a roll track in the file. `gears.i3d`'s `Camera01` carries a
`0x44` chunk — a scalar TCB track, three keys, `0 → -2.823 → -1.290`
radians over 359 frames, so Max had the camera swinging through about
160°. Handler `0x1000b3d0` appends it to the list at container `+0x180`,
and that list has no read site anywhere in the DLL: the only three
references to offset `0x180` are the two loaders appending to it and one
unrelated byte store. The track is parsed and dropped on the floor.

So `gears` was authored with a rolling camera and shipped without one.
The port reproduces the release, not the intent. Implementing `0x44`
would be a visible regression against the original, which is why the
parser keeps it in `anim['extra']` and the exporter ignores it.

### Camera chunk `0x21`
One byte, present once per camera in all nine scenes, always `01`. Its
handler is `0x1000bdb0`, which reads the byte into a stack local and
returns without storing it. Nothing to implement.

### Camera near and far
Chunk `0x22` carries Max's clip planes ahead of the field of view; they
are `(0, 1000)` in every camera in all nine scenes. `ogl_camera::calculate`
ignores them and passes literal `2` and `5000` to `gluPerspective`, so the
port hardcodes the same pair. `targetDist` is likewise a constant 160
everywhere and is unused — the target node supplies the look-at point.

### Chunk inventory
For the record, the ids that actually occur across the nine scenes:
`0x00` `0x10` `0x12` `0x13` `0x14` `0x15` `0x20` `0x21` `0x22` `0x30`
`0x31` `0x33` `0x40` `0x41` `0x42` `0x44` `0x47` `0x50` `0x51` `0x60`,
under the `0xdead` header. Every one is either implemented or, in the
case of `0x21` and `0x44`, proven inert in the original engine. `0x43`
(scale keys), `0x45` and `0x46` are handled by the parser but never
appear — no object in this demo is scaled over time.

### Three missing textures
`zwart.jpg`, `Refmap.gif` and `lakerem2.jpg` are named by scene materials
and were never shipped. In the original all three took the *textured*
branch of `ogl_material` and drew white plus their map; with the map gone
the port falls back to the material's own colour, which is the artist's
value and closer than flat white but is not what the engine did. Affects
the Solar logo (`Metal_Dark_Gold`), bolletje's inner sphere
(`Reflection_RefMap`) and the credits set (`Reflection_Lake`).

### Lighting — four scenes, not one — **done**
`ogl_scene::disable_all_lights` only enables `GL_LIGHTING` for a scene
that carries lights, and four of the nine do: `bolletje` (one `Omni01`),
`dings` (two omnis), `effect` (a `Spot01`) and `solar` (three omnis).
Chunks `0x31` (spot flag) and `0x33` (diffuse plus cutoff) now load, and
`minigl` grew an eight-light fixed-function path.

`ogl_light::calculate` writes only `GL_POSITION`, `GL_DIFFUSE`,
`GL_AMBIENT` and, for a spot, direction, cutoff and exponent. Ambient is
never written by any chunk and the constructor leaves it at zero, and
`ogl_material` never sets specular — so the whole model collapses to a
diffuse term, which is what the shader implements.

One thing to keep an eye on: lighting has to be turned back *off* on the
way out of a scene, or the 2D layers that follow get dimmed by it. That
bug showed up as `bg.png` rendering dark grey behind the `dings` chrome
instead of near-white.

### Vertex animation — `0x47` — **done**
`effect.i3d`'s `Cylinder01` carries a chunk `0x47` of 75,764 bytes,
handler `0x1000a340`. It is a snapshot cache, not a morph target set: a
count of 121, then vertex count, end frame, a stride, and per sample a
`u16` frame index followed by a full copy of the mesh's vertices. The
frames come out contiguous `0..120` and the engine picks the snapshot for
the current frame with no blending between them, so the port does the
same. Max vertex travel between samples 0 and 60 is 71.5 units — the
shape genuinely deforms across the section. It is the only `0x47` in the
nine scenes.

### The material flag byte is Wire, not 2-Sided — **done**
Bit 1 of the material flag byte is 3ds Max's **Wire** checkbox. It sets
`mat[0xd0]`, and the only code that reads it (`0x1000559d`) swaps the
primitive to `GL_LINES` over the same index list — same triangles, drawn
as edges. It is not a two-sided flag; back-face culling is decided per
pass by `ogl::draw_display`, not per material.

Only `effect.i3d` uses it, and it is the difference between that section
reading as a solid white blob and as the radiating wireframe mandala it
is supposed to be.

### The reflection map is added, not substituted — **done**
`ogl_trimesh` has two routes to a material's reflection map and both add
it on top of the base pass rather than replacing it:

  * with `GL_ARB_multitexture` and `GL_EXT_texture_env_add` (the flags at
    `+0x278` and `+0x27c`, probed at `0x100014c2` and `0x10001542`), one
    pass — the map goes on texture unit 1 under
    `glTexEnvf(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, 260.0)`, and 260 is
    `GL_ADD`.
  * without them, two passes — the base, then `glDepthFunc(GL_EQUAL)`,
    `glBlendFunc(GL_ONE, GL_ONE)` and the map again over the same
    triangles.

The port had been substituting the map for the base, so every reflective
surface rendered as its reflection alone on black. Five materials are
affected: `bolletje`, `credits`, `cross`, `dings` and `sphere`.

Two of them have a black diffuse colour (`cross`, and `credits` whose
base map is the missing `zwart.jpg` — "zwart" is Dutch for black), so
those look the same either way. The other three are mid grey, and they
get visibly brighter: `sphere.i3d`'s blob is chrome over 50% grey, not
chrome over nothing.

**Open:** the two routes do not agree on how bright. Multitexture adds the
map raw, so `0.5 + env`. The fallback leaves whatever colour the base
pass set current, and unit 0 defaults to `GL_MODULATE`, so it lands on
`0.5 + 0.5 * env`. The port follows the multitexture reading, on the
grounds that both extensions were near-universal on the hardware this was
shown on. If the release looked less blown out than that, the fallback
arithmetic is the other candidate and is a one-line change.

### Texture-coordinate seams
The exporter drops 3ds Max's separate map-face table, so a handful of
meshes carry slightly more UVs than vertices (`Torus09`: 169 against 144).
Indexing by vertex, as the engine does, leaves the seam column wrong.
Only affects reflection-mapped meshes, where it is invisible.

The same omission is what makes `cubes.i3d` unrecoverable — ten of every
twelve triangles enclose no UV area — so those 96 boxes get a synthesised
quad unwrap. If a capture ever turns up, that is the first thing to check
against it.

### The additive cubes letterbox — intent vs. release
Coat, who designed the demo, remembers the black bars top and bottom
sitting *above* the cube scene, letterboxing it. The script puts them
below: `balk.jpg` is at layers 6 and 7, the scene at layer 8, and
`red_base::run_tasks` (0x10003d10) walks its 64-slot layer array
ascending, with no inversion at either end — the parse at 0x100033fd
stores `atoi(token)` verbatim and the draw loop indexes by that same
number. There is no viewport or scissor anywhere in Energy3D either; the
only two `glViewport` calls are `(0, 0, w, h)` at display setup. So as
released, the cubes spilled over the bars, and the port reproduces that.

Recorded because it is the one place where the author's memory and the
shipped script disagree, and a capture would settle it. Moving the two
`balk.jpg` cues above layer 8 is the whole change if it ever turns out
the release did letterbox them.

Worth knowing regardless: the engine clears `GL_DEPTH_BUFFER_BIT` before
*every* layer, so a 3D scene can never occlude anything drawn after it.
Sitting 3D between 2D layers is a supported arrangement, not a trick.

### Map amount
Each texture record carries a Max map amount, which the parser skips. It
is 1.000 for every texture in all nine scenes, so nothing is being lost
today, but a scene added later could rely on it.

### `add` images and alpha
`ogl_bitmap::render` maps `add` to `glBlendFunc(GL_ONE, GL_ONE)`, which
ignores the fragment alpha entirely — so a script that fades an additive
image with `animate{alpha}` gets nothing. Worth a pass over the script to
see whether any layer relies on that, because if one does, the original
would have shown the same non-effect and the port should too.

---

## Tier 3 — infrastructure

- **A capture to calibrate against.** Nothing here has been checked
  against a recording of the original. Blend intensities, the exact look
  of the reflection maps, and the timing of the `show*` effects are all
  eyeball decisions until then.
- **A making-of write-up**, as the Tesla port has. The material is good:
  a self-documenting script language, a chunked scene format cracked from
  a symbol-rich DLL, an axis convention that turns out to be right-handed
  when every neighbouring demo of the era was not, and a rotation field
  that looks like a quaternion and isn't.
