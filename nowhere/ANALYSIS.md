# Nowhere — Threestate, Takeover 2000

**Feasibility analysis for a JavaScript / WebGL reconstruction.**

Short answer: **yes, and this is by far the most tractable of the three.**
Tesla shipped its C++ source. Contour shipped nothing and had to be
disassembled record by record. Nowhere shipped something in between and
in some ways better than either — a **plain-text demo script** that names
every effect, every parameter and every cue time, and **effect plugins
that document their own parameters in prose**.

Everything below was established from the uploaded archive in this
session. The container format, the compression, the script and the scene
format were all unknown at the start and are all readable now;
`tools/unstv.py` unpacks the whole data library and `data/scripts/` holds
the recovered script sources.

---

## 1. What shipped

```
demo.exe        114 KB   thin launcher, imports Axiom.dll
Axiom.dll       327 KB   the engine: system, script, effect manager,
                         texture/material managers, file system, player
Zeus.dll         94 KB   the 3D scene library: cScene, c3dObject, cCamera,
                         cLight, cHelper, cSpline, cQuat, cMatrix, cVector
*.srx        14 files    one DLL per effect, uniform plugin ABI
nowhere.stv     2.4 MB   the data library: 97 files
sound.vic       3.3 MB   the soundtrack — a plain 128 kbps 44.1 kHz MP3
fmod.dll         74 KB   audio playback (UPX-packed)
Axiom.dll is dated 2014; this archive is a compatibility-patched release.
```

The DLLs were built with MSVC and **exported with full C++ decoration**,
so the engine's entire class surface reads out of the export table
without any disassembly:

```
cAxDemo · cAxSystem · cAxScript · cAxToken · cAxPlayer · cAxTexture
cAxMaterial · cAxEffectManager · cFile · cFileSys · cAxPicture · cAxHelp
cScene · c3dObject · cCamera · cLight · cHelper · cSpline · cQuat · cMatrix
```

Two things that matter for a port:

- The renderer is **Direct3D 7 with D3DX** (`IDirect3D7`,
  `IDirect3DDevice7`, `ID3DXContext`, `IDirectDraw7`, `_D3DVIEWPORT7`),
  not the DX5 immediate-mode execute buffers Contour used. DX7's
  fixed-function pipeline — matrix stacks, two texture stages, `SetRenderState`
  blending — maps almost one-to-one onto the `minigl.js` layer already
  written for the Tesla and Contour ports.
- `cAxHelp` exists: `WriteEffects`, `WriteExecute`, `WriteParam`,
  `WriteTOC`. The engine could **generate its own documentation**, which
  is why every effect carries a table of parameter names *with written
  descriptions*. Those strings are still in the binaries.

## 2. The data library, cracked

`nowhere.stv` is an Axiom "lib", opened by `cFileSys::OpenLib`
(`Axiom.dll` `0x1000a070`):

```
u16 count                                       -- 97
count x {
  u8  namelen
  u8  name[namelen]      each byte XOR (i + 0x73)
  u8  flags              bit 0 = compressed
  u32 offset
  u32 uncompressed_size
}
```

Each blob is `[u32 packed_size][stream]`. The stream is **aPLib**,
depacked by the routine inlined at `0x1000a870`. One detail matters and
cost a debugging round: this build's short-match case (`110`) does **not**
update the last-offset register, unlike the reference implementation.
With that corrected, all 97 entries decode to exactly their declared
sizes and all 65 images parse.

The library holds **39 JPEGs, 26 PCX (alpha masks and sprite frames),
21 text scripts and 11 `.zeu` scenes** — the complete visual content of
the demo, at 640×480 for backgrounds and 256×256 for effect textures.

## 3. The demo script — plain text

This is the find. `script.txt` is the whole timeline, in a C-preprocessor
flavoured language the engine tokenises at load:

```
sound       filename    sound.vic
sound       length      206488

PREDEMO     sphere      filename    .\sphere\sphere.zeu
PREDEMO     sphere      execute     load
PREDEMO     sphere      layer       1

000045400   sphere      run     true
000059200   sphere      run     false
```

Columns are `time | instance | parameter | value`; time is milliseconds,
`PREDEMO` is `-1000000` (a preload pass). `REGISTER <instance> <effect>`
binds a named instance to a plugin, `#include` pulls in the per-part
scripts, and `#define` gives the section constants. There is no hidden
state anywhere — the demo's structure is literally its text.

### The full running order

| time | part | plugin | assets |
| --- | --- | --- | --- |
| 0:00 – 0:45.4 | `sbs` (plasma) | sbs | sbs1/sbs2 + envmodulate |
| 0:08.5 – 0:21 | credits: **vic** | credits | ring + sphere, additive |
| 0:21 – 0:35 | credits: **srx** | credits | " |
| 0:35 – 0:45.4 | credits: **stv** | credits | " |
| 0:45.4 – 0:59.2 | sphere morph | Sphere | `sphere.zeu`, GeoSphere01→02 |
| 0:59.2 – 1:12.8 | kaleidoscope | kaleido | `kaleido.jpg` + overlay/alpha |
| 1:12.8 – 1:26 | flower / TV | 33 | `flower.zeu`, 4 TV screens |
| 1:26 – 1:40.3 | flower 2 | 33 | `flower2.zeu` |
| 1:40.3 – 1:54 | boarder | zeusPlay | `boarder.zeu` (10+ Line objects) |
| 1:54 – 2:08.1 | greetings | greets | `greets.zeu` Spring01 ribbon |
| 2:08.1 – 2:21.4 | kubus | kubus | procedural, no scene |
| 2:21.4 – 2:49.2 | fire | fire | `fire.zeu` Box01 |
| 2:49.2 – 3:02.4 | smoke | smoke | `smoke.zeu` Box01–07 blobs |
| 3:02.4 – 3:16.2 | slierten | slierten | `slierten.zeu` marching cubes |
| 3:16.2 – 3:26.4 | end card | backgr | `endpic.jpg` + `.pcx` alpha |

Six 2D sprite overlays (`animscale`, `animtext`, `animshooter`,
`animbanner`, `animmeter`, `animsignal`) run on top of those sections at
12 fps with explicit screen positions, each a 4-frame PCX loop.

Every part also has a background layer (`backgr`, a 640×480 JPEG with an
optional PCX alpha), and layering is explicit: `layer 1` for scenes,
`layer 2` for the overlays, higher numbers drawn first.

## 4. The effects document themselves

Each `.srx` registers its parameters with a description string. These are
still in the DLLs, in a mix of English and Dutch, and they give away the
algorithms:

| plugin | parameters (verbatim from the binary) |
| --- | --- |
| `33` | `uitslag` "maximale uitslag van de paal" · `center` · `bolscale` "groote van de bol" · `bolturbdiv` "turbulentie divider, hoe groter deste minder heftig" · `bolanglediv` · `movietime` "divider … als je op 30.0f fps draait wordt het 30/2 = 15" — expects `Cylinder01`, `GeoSphere01`, `tv01`–`tv04`, `camera01` |
| `Sphere` | expects `GeoSphere01` ("minimal object") and `GeoSphere02` ("maximal object"), one material only — a two-target vertex morph |
| `greets` | `straal` · `hoogte` "hoogte van segmentje, lintje voor lars" · `hoogte2` · `designmul` "hoe hoger hoe meer wit en zwart" — expects `Spring01` |
| `smoke` | `hoeveelBobbels` · `straal` "kleinste straal" · `bigstraal` "radius van grote cirkel" — expects `Box01`–`Box07` |
| `slierten` | `periodx/y/z` "periode van de sinus" · `uitslagx/y/z` · `multi` · `maxLength` — expects `Slierten` + `Blobs`, two cameras, and logs "MARCHING RENDER] TO MUCH POLYGONS" |
| `fire` | `objectsize` · `stevieparm` "iets met de kleur, hoe groter hoe donkerder ofzo" · `stevieadd` — expects `Box01` |
| `kubus` | `cubes` "het aantal cubes dat getekend wordt" · `rotation` "hoeveel de kubussen tov elkaar gedraaid zijn" · `colorR/G/B/A` |
| `kaleido` | `material` · `vtlx/vtly/vbrx/vbry` viewport corners (defaults 0,0 → 640,480) |
| `sbs` | `material` "material that is put on the plasma" |
| `anim` / `picture` | `tlx/tly/brx/bry` screen rect · `tlu/tlv/bru/brv` texture rect · `zpos` · `colorR/G/B/A` · `speed` |
| `backgr` | `filename` (.jpg) · `alphaname` (.pcx) · `zpos`, asserts 640×480 |
| `zeusPlay` | generic `.zeu` player: `filename` · `material` · viewport |
| `credits` | expects `GeoSphere01`, `GeoSphere02`, `camera01` |

Plus `LAYER`, `RUN`, `SPEED` on every effect from the shared base.

So `slierten` and `smoke` are **marching-cubes metaball systems** driven
by animated box/blob positions from the scene; `33` is a turbulent sphere
with four TV screens playing a texture strip; `Sphere` is a morph;
`greets` extrudes a ribbon along a Max spring. None of that has to be
guessed from pixels.

## 5. Materials and textures — also plain text

The per-part scripts declare textures and materials in a small block
language:

```
texture flower   primary   .\flower\flower.jpg
texture envmap   secondary .\flower\flowerenv.jpg

material { name flowermat  texture flower  texture envmap  calcenv }
material { name ringmat    texture credvic_jpg  additive }
material { name greets     texture greetings_jpg  cull none }
```

The flags observed across all scripts are `primary` / `secondary`,
`calcenv` / `envcalc` (spherical environment mapping, generated per
vertex — `c3dObject::CalcEnv` in Zeus), `envmodulate`, `additive`,
`nomipmap`, `cull none`. That is a complete, small render-state
vocabulary; there is nothing to reverse.

## 6. The `.zeu` scene format

Exported from 3ds Max (the scene chunks still carry `33scene2a.max`,
`slierten7.max`, `E:\test\boarder.ZEU`). Structure, confirmed against
several files:

```
u16 version = 1
chunk { u16 id; u32 size; payload }
  0x1000 SCENE   0x2000 MESH   0x3000 CAMERA   0x4000 LIGHT   0x5000 HELPER
```

Names are length-prefixed. Mesh payload holds a vertex count, a polygon
count, then a **23-byte vertex**: `float x,y,z` · `u8 r,g,b` · `float u,v`.
Verified on `fire.zeu`: 26 vertices, 12 polygons, first vertex
`(-150, -150, 150)` rgb `255,255,255` uv `(1, 0)` — a unit box with
UV-split corners, exactly as Max would export it.

Camera and light chunks carry animation as `cSpline` key lists.
Zeus's spline is a proper TCB/quaternion spline — `AddKey(t, cVector, …)`,
`AddKey(t, cQuat, cQuat, …)`, `GetVector`, `GetQuat`, `SlerpLong`,
`Exp`/`Log`/`Lndif` — so camera moves are keyframed, not baked per frame.
Interpolation will have to be matched from Zeus's `CompDeriv*` routines,
which is the one piece of real disassembly work left.

The scene graph reads out cleanly:

```
sphere3.max     GeoSphere01, GeoSphere02, Camera01, Target, Rectangle01..09
33scene2a.max   Cylinder01, GeoSphere01, tv01..tv04, Rectangle10..17, Camera01
slierten7.max   slierten, blobs, Camera01, Camera02, Target
smoke.max       Box01..Box07, Camera01, Target
boarder3.max    Line01..Line10+, Rectangle01, Camera01, Target
greetings.max   Spring01, Camera01, Target
```

## 7. Verdict

**Feasible, and cheaper than Contour was.** The three things that
normally dominate this kind of work are already solved:

1. *The timeline* — plain text, exact to the millisecond, no reverse
   engineering at all.
2. *The parameters* — named, described, and given real values in the
   script.
3. *The assets* — all 97 files extract losslessly, and the soundtrack is
   already an MP3 a browser can play directly as the clock.

What is left is honest implementation work:

| item | risk | note |
| --- | --- | --- |
| `.zeu` mesh/camera parser | low | format understood; a day's work |
| Zeus spline interpolation | medium | must match `CompDerivTwo` / `SlerpLong` from Zeus.dll |
| `backgr`, `anim`, `picture`, `zeusPlay` | low | generic 2D/scene players, ~4 small modules |
| `Sphere`, `kubus`, `kaleido`, `credits` | low | morph, instanced cubes, screen kaleidoscope, textured ring |
| `33`, `greets`, `fire` | medium | procedural deformation with named, documented knobs |
| `sbs` plasma | medium | algorithm not visible in strings; needs disassembly of sbs.srx |
| `smoke`, `slierten` | high | marching cubes; the iso field and threshold need reversing, and they are the two most expensive parts at runtime |
| render state / two-stage env mapping | low | already have `minigl.js` from the other two ports |

The one thing genuinely missing is a **reference capture**. Contour's
port only became accurate once frames from the release AVI were available
to calibrate against. For Nowhere I have no video — the archive holds no
capture. Before committing to the effect work I would either source a
recording of the Takeover 2000 entry, or run the patched executable under
Wine/dgVoodoo and record it. Everything else can be built from the data;
fidelity cannot be judged without it.

## 8. Suggested order of work

1. `tools/unzeu.py` — chunk walker, meshes and cameras out as JSON/binary.
2. Reverse Zeus's spline evaluation; validate by replaying `fire.zeu`'s
   single camera key and `boarder.zeu`'s long track.
3. A script interpreter: tokenise `script.txt`, resolve `#include` and
   `#define`, build the instance/parameter timeline. This *is* the demo's
   player, and it is maybe 200 lines.
4. The generic players (`backgr`, `anim`, `zeusPlay`) plus material
   handling — that alone puts backgrounds, the 2D overlays and the
   boarder section on screen with correct timing.
5. The procedural effects, cheapest first, calibrating against a capture.

## 9. Tools in this directory

- `tools/unstv.py` — unpacks `nowhere.stv` (TOC + aPLib). Verified:
  97/97 entries at exact size, 65/65 images decode.
- `data/scripts/` — the recovered demo script and every per-part script,
  as shipped.

## Provenance

`Nowhere` by Threestate (sarix, stevie, vic, sagacity, with kalms), PC
demo, Takeover 2000. Code and data © 1999–2000 Threestate. Nothing from
the archive is redistributed here beyond the demo's own text scripts,
quoted for study; the binaries and artwork stay out of the repository.
