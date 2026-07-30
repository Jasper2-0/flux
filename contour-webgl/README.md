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
- **Two of Jace's effects from recovered closed-form math**:
  - `bloem` — the phase-modulated five-petal flower
  - `rogplay` — the Pickover attractor orbit, all three parameter sets
- **The engine layer** is Tesla's `minigl.js` with one addition the
  D3D-era vertex layout needs: per-vertex color in the array-draw path
  (`XYZ|DIFFUSE|TEX1`).

## What's honest to say about fidelity

The formulas, parameter sets, timeline times and scene data are read
from the binary and are solid. Everything else in the implemented
effects — camera moves, sprite sizes, blend intensities, texture tiling —
is reconstruction judgment awaiting comparison against the release AVI
capture. Handedness in the TimScene replayer is a best guess until then.

## Not yet ported

`flarefx`, `grid1fx` (formula recovered, not yet implemented), `linefx`,
`picflash`, `flash`, the `bally` credits, Scid's `Letters` (the poem
renderer), and the contour logo overlay treatment. The zoomer is
integrated (`js/effects/zoomer.js`), ported from the standalone
recreation with its measured nesting, feathered largest-first stack and
both endings — shipped (default) and the intended aligned landing.

Next steps, roughly in order of value: decode the parameter blocks the
timeline hands each record (dumped in `timeline.json` as `params_hex`),
enumerate per-effect blend state from the engine wrapper, integrate the
existing zoomer recreation, and frame-match against the AVI.

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
