# Tesla by Sunflower — JavaScript / WebGL port

A browser port of **Tesla**, the demoscene production by **Sunflower**,
originally released in July 2000 as a Windows/OpenGL demo. This port
reimplements the demo's effects in JavaScript on WebGL2, driven by the
original's released data files (`data.pak`) and soundtrack ("Tournesol").

Original credits (from `data/tesla.txt`):

- Konrad Zagorowicz (yoghurt) — electronics
- Pauli Ojala (saffron) — projections
- Jakob Svanholm (radix) — high fidelity
- Anne Haessig (lluvia) — high fidelity
- with Dominik Behr (technomancer), Wojtek Podgorski (sunreal),
  Janusz Borkowski (division)

## Running

Serve this directory over HTTP and open it in a browser (WebGL2 required):

```sh
python3 -m http.server 8000
# then open http://localhost:8000/
```

Press **play** (a user gesture is required for audio), **Esc** stops,
**←/→** seek ±5 seconds. The demo runs 254 seconds.

Two bundlers, for handing the demo to someone as a single file:

```sh
node build-single.mjs                # -> tesla-single.html, the demo with
                                     #    data.pak and the mp3 inlined
node writeup/build-writeup.mjs       # -> the interactive "making of":
                                     #    prose, live effect viewers, no audio
```

## What's in the port

The port follows the structure of the publicly released original source:

| file | role |
| --- | --- |
| `js/minigl.js` | small fixed-function-style GL layer over WebGL2: matrix stacks (projection/modelview/texture), immediate mode, vertex-array draws, blending, culling, linear fog |
| `js/mathlib.js` | vectors, the demo's column-major `CMatrix` (incl. its affine inverse), MSVC-compatible `rand()`, sine helpers |
| `js/pak.js` | `data.pak` archive reader, TGA decoder, texture manager |
| `js/t3ds.js` | 3DS mesh parser with the original loader's semantics (Y/Z swap, UV v-flip, averaged vertex normals, pivot-space transform) |
| `js/effects/` | the eleven timeline effects |
| `js/demo.js` | effect timeline (identical timing to the original) and soundtrack sync |
| `writeup/` | the interactive "making of": prose, a maths primer, and live viewers that run each effect on its own |

Timeline (seconds): SpinZoom 0–24.5 · ShadeBall 24.5–48.5 · Splines
48.5–67.7 · FFDEnv 67.7–87 · Bands 69–85 · EnergyStream 87–145 · Tubes
145–165 · PolkaLike 165–203 · Tree 186–201 · FaceMorph 203–222.5 ·
credits (Thing) 222.5–254.

Deliberate differences from the original:

- Rendering is WebGL2 with a single shader pair instead of fixed-function
  OpenGL; fog and the texture matrix are implemented in the shader.
- Time comes from the soundtrack (`audio.currentTime`) instead of a
  hardware timer, so audio/video sync survives tab throttling and seeking.
- Frame-rate-dependent state (PolkaLike tracks) advances with a fixed
  60 Hz step so playback speed matches the original regardless of display
  refresh rate.

`data/data.pak` and `data/tournesol.mp3` are the unmodified files from the
original freely distributed release.
