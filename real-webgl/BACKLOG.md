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

| effect | vtable | init | run | state | screen time |
|---|---|---|---|---|---|
| `showLines` | `0x40c594` | `0x402c60` | `0x402e80` | `0x1c0` | 50 s — **done** |
| `show2d` | `0x40c580` | `0x401000` | `0x401160` | `0x1c0` | 38 s |
| `showPartiekels` | `0x40c56c` | `0x403150` | `0x403220` | `0x48` | 21 s |
| `showCylinder` | `0x40c558` | `0x402350` | `0x4026a0` | `0x98` | 11 s |
| `showPlanes` | `0x40c530` | `0x403590` | `0x4036d0` | `0x1e0` | 11 s |
| `showBol` | `0x40c5a8` | `0x401d70` | `0x4021b0` | `0x64` | 11 s |
| `showDraai` | `0x40c544` | `0x4029f0` | `0x402a30` | `0x38` | 11 s |
| `showTunnel` | `0x40c5bc` | `0x4039e0` | `0x403e60` | `0x98` | 11 s |

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

- `Real.exe` also names `bruut.tga`, `eeeeeeeenv.tga`, `solar_groot.jpg`,
  `sphere.i3d`, `envmap.jpg`, `particle.jpg` and `plane_01`–`plane_04.jpg`.
  Those are the assets the effects use, and they map onto the effect names
  neatly: `particle.jpg` for `showPartiekels`, the four planes for
  `showPlanes`, `sphere.i3d` for `showBol`. All are present in the release.

- The effects draw in the same screen-pixel ortho frame as the 2D layers
  and take their clock from the message's task-local elapsed milliseconds,
  the same `[msg+0x48]` that drives `drawScene`.

---

## Tier 2 — fidelity gaps in what is already ported

### Camera roll
`ogl_camera::calculate` issues `glRotatef(roll, 0, 0, 1)` before
`gluLookAt`, and `energy3d_camera::set_roll` writes `+0x124`, but the
loader path that feeds it has not been traced. Every camera in these nine
scenes has a rotation on its node, so the roll is in the file — it is only
a question of which handler reads it. Nothing in the sections rendered so
far looks tilted, so this may be zero throughout, but it is unverified.

### Three missing textures
`zwart.jpg`, `Refmap.gif` and `lakerem2.jpg` are named by scene materials
and were never shipped. `zwart` ("black") and the two reflection maps are
cosmetic — the affected surfaces currently fall back to their reflection
map or to flat white.

### Lighting for `bolletje`
`ogl_scene::disable_all_lights` only enables `GL_LIGHTING` for a scene
that carries lights, and `bolletje.i3d` is the only one of the nine that
does (a single `Omni01`). It currently renders unlit like the rest.

### Texture-coordinate seams
The exporter drops 3ds Max's separate map-face table, so a handful of
meshes carry slightly more UVs than vertices (`Torus09`: 169 against 144).
Indexing by vertex, as the engine does, leaves the seam column wrong.
Only affects reflection-mapped meshes, where it is invisible.

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
