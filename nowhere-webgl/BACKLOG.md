# Nowhere port — what's left

Ordered by value, not by difficulty. Everything here can be done from the
binaries and the data; the reference capture would confirm results but is
not needed to do the work. Items marked **capture-blocked** are the
exceptions.

Screen times are out of the demo's 3:26.

---

## Tier 1 — unported parts, tractable now

The four parts where I have both a decoded parameter set and a clear idea
of the algorithm. Together they cover **68 seconds** of currently-empty or
raw-scene screen time.

### `greets` — 14 s (1:54–2:08)
The one framegrab I have shows exactly the target: a wide double-sided
helix ribbon carrying the greeting names, filling the frame, seen close.
Raw, `Spring01` is a helix of radius 48 seen from 255 units away — it
occupies a fraction of the screen and is white-on-white against the
blowout, so the effect clearly rebuilds it far larger.

Parameters are already documented by the plugin itself: `straal`
("Straal enzo" — radius), `hoogte` ("hoogte van segmentje, lintje voor
lars"), `hoogte2` ("hoogte van het gehele"), `designmul` ("hoe hoger hoe
meer wit en zwart"). Needs the rebuild routine out of `greets.srx`.

**Confidence: high.** Clear target, named parameters, simple geometry.

### `kubus` — 13 s (2:08–2:21)
Currently nothing renders. No scene file at all — the effect generates
everything. The constructor is already decoded, giving field offsets and
defaults: `rotation` 0.01 at `+0xcb4`, `cubes` 100 at `+0xcb8`, and
`colora/r/g/b` 1.0/0.5/0.1/0.1 at `+0xcbc`–`+0xcc8`. The script overrides
the colour to 255/230/200, which clamps to white.

Just needs the draw routine read properly — I found the colour packing at
`0x100019a0` but mistook a helper for the vtable's draw slot.

**Confidence: high.** Smallest self-contained effect in the demo.

### `33` poles — 27 s (1:12–1:40, both instances)
The flower's petals and the white sweeping curves are the eighteen tubes
the effect builds itself, not part of the scene. `0x10001fc0` allocates
them: 18 records of 0x30 bytes, each a tube of 41×3 vertices and 240
triangles, with constants 600, 10, −100, 400, 60. `0x10001930` sways them
using `center` (+0xcf4, default 25) and `uitslag`.

The ball is already done; this is the silhouette around it.

**Confidence: high.** Generation routine already mapped, sway routine
located and using known fields.

### `kaleido` — 14 s (0:59–1:12)
Currently only its overlay background shows. A screen-space kaleidoscope
of `kaleido.jpg` across a viewport rect — the only parameters are
`material` and the four viewport corners (`vtlx/vtly/vbrx/vbry`,
defaulting to 0,0 → 640,480). No geometry, no scene.

**Confidence: medium-high.** Small effect, but the fold pattern has to
come out of the disassembly rather than the parameter names.

---

## Tier 2 — engine work that lifts every 3D part at once

### Lighting
The largest single fidelity gap. Zeus scenes carry lights, `c3dObject::
CalcNormals` is called on rebuilt geometry every frame, and **nothing in
this port is lit** — everything is flat-textured. It is why the sphere
reads as a silhouette. Needs `cLight::Render` and the shading path out of
Zeus.dll, plus a lighting term in `minigl`.

### Environment mapping
`calcenv` / `envcalc` appear on `flowermat`, `tvmat`, `sliertmat` and
`smokemat`; `envmodulate` on `sbsmat`. Zeus generates these per vertex in
`c3dObject::CalcEnv`. The port already *loads* each material's secondary
texture and then ignores it — so four of the demo's biggest surfaces are
missing their second texture stage entirely.

### The scene frame rate — worth attacking from the binary
Still the biggest open unknown, and I have been treating it as
capture-blocked. It may not be: the base effect class holds the local
clock at `+0x2c`, and whatever converts that into a Zeus frame number
lives in the shared code every plugin calls. Finding it would settle a
question no single value currently fits — boarder wants 32.7 fps, credits
20, smoke 53 — and would correct the timing of every 3D part at once.

**This is the highest-leverage item in the whole list.** Doing it before
Tier 1 might be right.

---

## Tier 3 — high screen time, genuinely hard

### `sbs` — 55 s (0:00–0:45, 3:16–3:26)
The largest single block of screen time and the only effect whose
algorithm is hinted at nowhere in the strings — its one parameter is
"material that is put on the plasma". The framegrab narrows it usefully:
dark, low-contrast, olive and yellow, slow, smoky. It also runs
underneath the whole credits sequence, so it is not just its own 55
seconds.

### `fire` — 28 s (2:21–2:49)
Renders as a raw box. Parameters decoded but opaque: `objectsize`,
`stevieparm` ("iets met de kleur, hoe groter hoe donkerder ofzo"),
`stevieadd` ("iets met adden"). The script sets 24 and 5.

### `smoke` and `slierten` — 27 s (2:49–3:16)
Marching-cubes metaball fields — `smoke` over `Box01`–`Box07`,
`slierten` over `Slierten` and `Blobs` with two cameras. `slierten.srx`
logs "MARCHING RENDER] TO MUCH POLYGONS", so the iso-surface extraction is
in there to be read. Parameters are named (`hoeveelBobbels`, `straal`,
`bigstraal`; `periodx/y/z`, `uitslagx/y/z`, `multi`, `maxLength`) but the
field function and threshold need recovering. These are also the two most
expensive parts at runtime.

---

## Tier 4 — smaller gaps and polish

- **The TV movie.** `tv01`–`tv04` carry material `screen` → `anim.jpg`,
  and `movietime` divides the frame rate ("standaard op 2 dus als je op
  30.0f fps draait wordt het 30/2 = 15"). The screens currently show a
  static frame.
- **Quaternion interpolation.** Currently slerp; Zeus squads, and has the
  `Exp`/`Log`/`Lndif`/`SlerpLong` machinery for it. Difference is under a
  degree with these scenes' TCB values, so this is low priority.
- **Spline ease curves.** `CompAB` is reconstructed as the classic Max
  ease, not recovered. 42 of 628 keys have non-zero TCB.
- **Credits: the two open discrepancies.** The vertical repeat and the
  dimness. Worth a second pass over `0x10001560` — the formulas read
  cleanly but something upstream of them is off, and re-reading is
  cheaper than waiting.
- **`picture`.** The engine's other 2D blitter, unused by this script.
  Only needed for completeness.

---

## Tier 5 — infrastructure

- **Calibration harness.** So that when the capture does arrive it is one
  command: extract frames at the port's timestamps, render matching
  frames headlessly, and emit side-by-side sheets per section. Would make
  the capture pay off in minutes instead of a session.
- **A making-of write-up**, as the Tesla port has — this one has better
  material than Tesla did, given the container, the aPLib depacker, the
  scene format and the plugin ABI all had to be cracked from scratch.

---

## Capture-blocked

Only these genuinely need the video:

- Confirming the scene frame rate *if* it cannot be recovered from the
  binary.
- Judging blend intensities and additive gains, which are eyeball
  decisions.
- Final frame-matching of anything already implemented.
