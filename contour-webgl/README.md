# Contour by The Black Lotus — WebGL port scaffold

A browser port-in-progress of **Contour** (The Black Lotus, Takeover 1999,
2nd place PC demo), built on the same architecture as the finished Tesla
port in `../tesla-webgl/`. Unlike Tesla, Contour's source was never
released — this port is built from binary reverse engineering of
`contour.exe` (a Direct3D Immediate Mode production on the group's
"Viewer" engine) and from the asset extraction and format analysis done
in the companion research repository.

## The dispatcher, decoded

The timeline's runtime semantics were recovered from the binary
(tick at VA `0x403ce0`, player ctor `0x403840`, WinMain `0x402ca1`):

- The record table lives at `0x438930`, stride 0x28, terminated by a
  kind-2 record. Earlier extractions used base `0x438948`, which shifted
  every class/instance label one record against its own time and id —
  the corrected base resolves the previously mislabeled message rows.
- Record layout: kind +0x00 · id +0x04 · time (double) +0x08 ·
  code/ctor +0x10 · params +0x14 · extra +0x18 · class +0x1c ·
  instance +0x20.
- Kinds: 0 create (preload; allocates a registry node with strdup'd
  names in a 64-slot table) · 1 kill (also notifies the engine with
  code 0x10) · 2 end-of-table · 3 message (code + two params delivered
  to the instance by id) · 4 call on the global music object ·
  5 clock jump (payload seconds × 4×44100 added to the MP3-byte clock —
  the demo's clock **is** the soundtrack position, like this port's).
- Message code `0x20` is the universal "show/start": every visible part
  begins with it. Codes `0x9004/0x9005/0x9010/0xa001` are per-effect
  mode switches, not yet interpreted.

This yields real activation windows: contlogo (running Tim's replayer on
`dildo.bin` — the copper ripple under the logo) shows at 40 s, rogplay
52–65.5 s, bloem 80–94 s, grid1fx 94.5–107 s, the zoomer at 108 s.

## What works

- **The full timeline, extracted from the executable.**
  `tools/extract-timeline.py` replays the 770-store init routine at
  `0x401560` with capstone and merges the five static records, yielding
  all ~90 timeline records (create/kill/message, float cue times,
  handler addresses, parameter-block dumps) as `data/timeline.json`.
- **A timeline player** (`js/demo.js`) that dispatches create/kill
  records against the extracted cue times, synced to the shipped
  soundtrack (`test.mp3`, 3:14). Unimplemented parts appear as live
  placeholders in the corner readout, so the demo's structure is visible
  even where rendering isn't done yet.
- **An ARSE scene parser** (`js/arse.js`) for Tim's baked-scene format
  (validated against both shipped scenes), and a first **TimScene
  replayer** rendering meshes with their baked 30 fps object transforms
  and camera track.
- **Jace's effects from recovered closed-form math and decoded
  parameter blocks**:
  - `bloem` — the phase-modulated five-petal flower
  - `rogplay` — the Pickover attractor orbit, all three parameter sets
  - `flash`, `picflash`, `linefx` — see the decode below
- **The engine layer** is Tesla's `minigl.js` with one addition the
  D3D-era vertex layout needs: per-vertex color in the array-draw path
  (`XYZ|DIFFUSE|TEX1`).

## What's honest to say about fidelity

The formulas, parameter sets, timeline times and scene data are read
from the binary and are solid. Everything else in the implemented
effects — camera moves, sprite sizes, blend intensities, texture tiling —
is reconstruction judgment awaiting comparison against the release AVI
capture. Handedness in the TimScene replayer is a best guess until then.

## The type is real

Two of the demo's most visible parts are now driven by the original
artwork and data rather than approximated:

- **Scid's poem** (`js/effects/letters.js`). Each `Letters` instance's
  48-byte parameter block decodes to a text pointer, a type, a start and
  end position, a scale and a duration — so the eleven lines, their
  corner placements, their inward drift and their timings all come out of
  the executable. Glyphs are cut from `letters/abc.jpg`; the proportional
  boxes were measured from the artwork into `data/font-abc.json`.
  Reconstructed: cap height per unit of scale, tracking, the fit-to-width
  rule for long lines, and the fade envelope.
- **Balance's credits** (`js/effects/credits.js`). `credits-tekst.jpg` is
  a type atlas holding every name and role label; the eighteen boxes were
  measured from it. The timeline creates eight `credit3` instances on a
  3.5-second grid but does not say which credit each shows, so the order
  and placement follow the release capture.

`grid1fx` and `flarefx` are implemented from the mathematics recovered
out of the binary — the standing-wave radius field and the Pickover
attractor parameter sets respectively.

## Jace's three parameter blocks, decoded

These were read straight off the constructors, the geometry builders and
the update methods, and they are now what drives the port. All three are
in `tools/extract-timeline.py`, so `timeline.json` carries named fields
rather than hex.

### `flash` — 32 bytes at `this+0x94` (ctor `0x41ff80`)

| offset | meaning |
| --- | --- |
| `+0x00,+0x04` | rim ramp, start and end time |
| `+0x08,+0x0c` | centre ramp, start and end time |
| `+0x14` | radius — *and* the fan's z |
| `+0x18,+0x1c` | colour from, colour to (`0xRRGGBB`) |

The builder (`0x4200e0`) lays N−1 vertices on a circle and puts the
centre last, giving every vertex `z = p[0x14]` as well. Radius and depth
being the same number means the fan subtends a fixed angle whatever the
parameter says: it is always a full-frame wash, and the radius has no
visual effect at all.

The update (`0x420160`) writes **nothing but vertex colours**. Two
clamped ramps, `v = (T−p0)/(p1−p0)` for the rim and `s = (T−p2)/(p3−p2)`
for the centre, each drive a `lerp(from, to)`; once both pass 0.999 the
object raises its own dead flag. The blend is additive, and the timeline
proves it: instance 22 runs black→white at 51.5 s and instance 24 runs
white→black at 52.0 s — a cut through white. Under alpha blending the
first of those would paint the screen *black* before it went white.

### `picflash` — 32 bytes at `this+0x94` (ctor `0x41fce0`)

| offset | meaning |
| --- | --- |
| `+0x00,+0x04` | half-diagonal, start and end (× 0.01) |
| `+0x08,+0x0c` | rotation, start and end (radians) |
| `+0x10` | base depth |
| `+0x14` | duration |
| `+0x18` | alpha falloff |
| `+0x1c` | texture name — null in all six instances |

Four vertices and two triangles (`0x41fdc0`), rebuilt every frame by
`0x41fe30` as a square whose corner vector is `(cos, sin) × size`, the
other three corners being that vector turned by 90°, 180° and 270°. The
π/4 baked into the angle is what makes rotation 0 axis-aligned — that
constant is what gives the layout away. Depth creeps by `0.002 t` over
the effect's life and the alpha is `clamp01((1−t) × falloff)`.

### `linefx` — 128 bytes at `this+0xb4` (ctor `0x41f4e0`)

A ring emitter. Eight `(base, rate, drift)` triples: the spawn routine
(`0x41f920`) writes `base + drift × T` into a new particle — `T` being
the effect's own clock, so successive rings differ — and the update
(`0x41f5c0`) evaluates `spawned + rate × age` each frame.

| offset | triple |
| --- | --- |
| `+0x00`, `+0x0c` | centre x, centre y |
| `+0x18` | ribbon half-width (× 0.001) |
| `+0x24` | colour (`0xRRGGBB`, not a triple) |
| `+0x28` | brightness |
| `+0x34`, `+0x40` | start angle, end angle (in turns) |
| `+0x4c`, `+0x58` | inner radius, outer radius |
| `+0x64`,`+0x68`,`+0x6c` | attack, sustain, release — their sum is the particle's life |
| `+0x70`,`+0x74`,`+0x78`,`+0x7c` | spawn interval, emit until, max alive, segments |

Per segment the update writes three vertices — inner *black*, middle
*colour*, outer *black* — so each particle is a ribbon lit down its own
centreline. The init (`0x41f530`) forces the segment count to 70 if it
falls outside `[1, 70]`.

The coordinates are normalised screen space with **y downward**, times a
constant 1.25 on y for the 4:3 frame. The release capture is what fixes
that sign, and it also confirms the whole reading: at 86 s instance 457
puts sixty-segment rings dead centre, and at 102 s instance 501's
five-segment ring sits upper-left at (0.38, 0.35) with 500's
fifty-segment one lower-right at (0.82, 0.85) — which is exactly what
the AVI shows.

### What the decode corrected

`scenes/neuron.bin` used to be dispatched as a scene belonging to
instance 455, because the string turned up in that record's parameter
dump. With the flash block pinned at 32 bytes it is now clear the
pointer sits one dword *past* the end of 455's block — it was the next
record's data, read through too wide a window. The extractor now clips
its string scan to each block's known size, no timeline record
references the neuron scene from inside its own block, and the port no
longer renders it. The scene stays in `data/` and the parser still
validates against it.

## Not yet ported

The credit backdrop layers (`credit1`, `credit4`, `credit5` — currently
a crossfade of the shipped paintings), the intro burst, the contour logo
overlay treatment, and the shard explosion visible in the capture around
86–90 s. Message codes `0x9004/0x9005/0x9010/0xa001` are recorded but
still uninterpreted. The zoomer is integrated
(`js/effects/zoomer.js`), ported from the standalone recreation with its
measured nesting, feathered largest-first stack and both endings —
shipped (default) and the intended aligned landing.

One judgment call worth flagging in the three effects above: `picflash`
has a null texture in every instance, so the data says "white square".
A flat white square at alpha 0.8 additive would blow the frame out for
the full 27 seconds of instances 552 and 469, and the capture instead
shows a bloom decaying from the centre — so this port fans the quad from
a bright centre to transparent corners. Its size, spin, depth and alpha
are the decoded ones untouched.

## Running

```sh
cd contour-webgl
python3 -m http.server 8000
# open http://localhost:8000/
```

## Provenance

`data/` holds assets extracted from the freely distributed
`contour.exe` (its `FILEDAT` resource container) — artwork by Saffron,
soundtrack by Crystal Score, scenes modelled by Sick Sjaak, code by
Nix, Jace, Balance, Scid and Tim. © 1999 The Black Lotus; included here
for preservation and study of a freely released demoscene production.
