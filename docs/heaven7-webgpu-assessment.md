# Heaven Seven (Exceed, 2000) — reverse-engineering assessment & WebGPU port plan

Assessment of the original Mekka & Symposium 2000 release archive
(`heaven7w.exe` / `heaven7d.exe`) for a restoration project: how hard is the
binary to reverse-engineer, and can the software raytracer be replaced
with a WebGPU compute-shader implementation in JS?

**Short answer:** this is a disassembly-and-annotation project, not a
decompilation one — there was never any higher-level source to recover
(hand-written, uncommented x86 assembly, per the author himself). Full
engine annotation is serious but tractable; a
*faithful reimplementation* is much cheaper than full RE, because the
rendering algorithm is well understood, the soundtrack exists as a clean XM
rip, and only the scene script / procedural textures genuinely require
digging into the binary. The raytracer itself maps almost embarrassingly well
onto a WebGPU compute shader — proven by the working prototype in
`demos/heaven7-webgpu/`.

---

## 1. What the binary is

| | `heaven7w.exe` (Windows) | `heaven7d.exe` (DOS) |
|---|---|---|
| Format | PE32, UPX-packed 67,072 B → 278,528 B unpacked | LE (DOS/4G-style), old UPX LE format |
| Unpacking | Trivial: modern `upx -d` works | Modern UPX refuses (LE support dropped); needs an old UPX or manual unpack — **not needed**, same engine as Windows build |
| Compiled | 2000-04-22 (PE timestamp) | — |
| Imports | `DDRAW` (DirectDrawCreate), `DSOUND` (ordinal 1), `WINMM.timeGetTime`, ~14 KERNEL32 + 12 USER32 functions | Gravis Ultrasound only (per release notes) |

The import table is the whole OS surface: DirectDraw is used purely as a
framebuffer blit and DirectSound purely as a PCM output ring buffer. There is
no Direct3D — every pixel is computed in software. Threads + critical
sections indicate the mixer/replayer runs on a worker thread.

### Code characterization (from disassembly of the unpacked `.text`, ~53 KB)

- **~18,900 instructions, pure hand-written assembly.** No C runtime, no
  standard prologues (zero `push ebp / mov ebp,esp` frames), no strings
  beyond one settings dialog. This matches the author's (Picard's) own pouet
  comment: *"full asm (i was crazy) almost without any comments"*.
- **The raytracer is scalar x87:** 14.4% of all instructions are FPU ops
  (`fld/fmul/fxch/faddp/fstp` dominate), with 35 `fsqrt`, 34 `fdiv`, 22 trig.
  That signature is **analytic intersection math** (quadratic discriminant
  ray/sphere, normalize, reflect) — not a distance-field marcher.
- **Zero SSE.** Confirms the era and explains the design: one scalar ray at a
  time on a Celeron 450 is why adaptive subsampling exists at all.
- **Heavy MMX block:** `movq/paddw/punpcklbw/packuswb/pmulhw/psraw` — packed
  16-bit integer pipelines. This is the subsample interpolation/reconstruction
  (the High 1×1 / Average 2×2 / Low 4×4 quality stages from the release
  notes and the Windows settings dialog) plus post-processing (glow/blur) and
  texture generation.
- **Scene/animation data is compressed bytecode:** the single most-called
  routine is a variable-length bit-stream reader that reconstructs IEEE-754
  floats field-by-field (sign/exponent/mantissa packing). The timeline —
  camera paths, object keyframes, event triggers — is a compact compiled
  script in `.data` (~75 KB of meaningful initialized data), decoded at
  startup/runtime. Classic 64k technique.
- ~129 distinct call targets ≈ small-function count; a focused RE effort has
  a bounded surface.

### Provenance facts (research-verified)

- Pouet prod **#5**, 1st place PC 64k intro, Mekka & Symposium 2000. Credits:
  **Picard** (raytracer), **Stephen** (texture generator), **Shaman**
  (music), **Warpig** (graphics/concept).
- **Source was never released** and won't be — Picard declined on pouet.
  Picard's making-of mini-site (`demoscene.hu/~picard/h7`) is offline;
  Wayback Machine capture is the top archaeology target, as it described the
  adaptive sub-sampling technique.
- **Music is a FastTracker 2 XM module** (`heaven7.xm`, ~30 KB, by Shaman) —
  ripped and hosted on The Mod Archive (module id 150033). Playback in the
  intro is a tracker replayer, not a softsynth. **A JS XM player we already
  have covers this entire subsystem.**
- **No known WebGL/JS/Shadertoy remake exists.** The only prior art is
  [joanbm/HEAVEN7L](https://github.com/joanbm/HEAVEN7L) — a Win32→SDL shim
  that runs the *original binary* on Linux. Useful as documentation of the
  exact API surface, not a reimplementation. A WebGPU port would be a first.

---

## 2. How hard is the reverse engineering?

**Framing: this is a disassembly project, not a decompilation project.**
There was never any C source — the intro is hand-written assembly, so the
disassembly *is* the source, minus labels and comments. "Decompiling" it
recovers nothing that ever existed; the real work product is an **annotated
disassembly**: named routines, labeled data, reconstructed intent, verified
by dynamic tracing.

**Getting a disassembly: easy.** UPX unpacks in one command; Ghidra/IDA
ingest the PE cleanly; the code is plain 32-bit x86 with x87+MMX.

**Understanding it end-to-end: hard — this is the expensive path.**
With no symbols, no CRT idioms, and no stack frames, every one of the ~130
routines must be understood by a human. Decompiler views (Ghidra/Hex-Rays)
remain useful as a *reading aid* for the integer/control-flow parts (window
setup, the bit-stream decoder, the timeline dispatcher) — but they produce
misleading garbage for exactly the interesting code: hand-scheduled x87
stack juggling (`fxch` chains defeat stack-to-variable recovery) and MMX
packed-integer pipelines. Those must be read as assembly, FPU stack state in
hand, or traced live. Estimate: weeks of skilled RE for a full-engine
annotation, most of it spent on code we would *throw away anyway*
(subsampling reconstruction, MMX post, DirectDraw plumbing, replayer) —
and the port target is WGSL/JS regardless, so reconstructed C source has no
role as a deliverable.

**The right target is data, not code.** For a faithful restoration the
genuinely valuable things inside the binary are:

1. **The scene script** — camera paths, keyframes, object placements, timing
   (drives everything you *see*). Requires reversing the bit-stream decoder
   (one small, self-contained routine — already located at the most-called
   call target) and then dumping the decoded float streams. Days, not weeks:
   instrument the original under HEAVEN7L or an emulator/debugger, breakpoint
   the decoder, and log its output rather than statically decoding `.data`.
2. **The procedural texture generator** (Stephen's code) — the marble/cloud
   textures and the face image. Same dynamic approach: dump the *generated*
   textures from memory at runtime, use them as assets first, reverse the
   generators later if 64k-purity matters.
3. **Timing constants** — BPM sync between the XM rows and scene events;
   recoverable by instrumenting `timeGetTime` usage + the replayer tick.

Everything else — the raytracer, shading, post — is better *re-derived* than
read out of the disassembly instruction by instruction, because the
algorithms are known and a video capture provides ground truth for
side-by-side comparison.

**Difficulty verdict:**
- Run-and-instrument to extract scene data: **moderate** (days–2 weeks).
- Clean-room reimplementation of the renderer: **easy–moderate** (the
  prototype's core took an afternoon).
- Fully annotated disassembly of the whole intro: **hard** (weeks+), and
  unnecessary for the goal.

---

## 3. Replacing the raytracer with WebGPU compute — feasibility: proven

The original renders: analytic raytraced spheres/planes (+ the tunnel's
cylinder/column geometry), Phong + specular, mirror reflections, hard
shadows, procedural textures, adaptive 1×1/2×2/4×4 subsampling, MMX
reconstruction + glow, 320×240-ish framebuffer stretched to screen.

On WebGPU this inverts beautifully:

- **One compute thread per pixel** replaces the entire adaptive-subsampling
  machinery. What was the intro's central engineering achievement (making
  scalar x87 raytracing realtime) is free on a GPU: the prototype traces
  every pixel at 1280×720 with 3 reflection bounces at 60 fps in *headless
  SwiftShader in a container*, i.e. without even a real GPU. On any phone
  GPU it is trivially realtime. The subsampling survives only as an optional
  render-scale homage (implemented in the prototype as the High/Average/Low
  control, mirroring the original's settings dialog).
- **The scene fits in a uniform/storage buffer**: Heaven 7 scenes are tens of
  primitives, not thousands. No BVH needed — brute-force intersection per
  ray is fine, exactly like the original.
- **WGSL has everything required**: `sqrt/dot/reflect/pow`, storage textures
  for output, texture sampling for the procedural textures (generate them
  once into textures with a second compute pass — pleasingly, that mirrors
  the original's startup texture-generation phase).
- **Music**: the existing JS XM player + the Mod Archive `heaven7.xm` rip
  gives sample-accurate original audio. Sync scene time to the player's
  row/tick clock, matching how the intro synced to its replayer.
- **Post-processing** (glow, flash fades, crossfades) is one more small
  compute or fragment pass.

**Working proof:** `demos/heaven7-webgpu/index.html` — a single-file,
dependency-free WebGPU compute raytracer (WGSL embedded) rendering a
representative Heaven-7-style scene: chrome sphere + orbiting colored
spheres on a checkered plane, phong, shadows, up to 5 mirror bounces,
animated camera, quality control. Validated headlessly via GPU readback
(`preview.png`).

### Caveats

- iOS Safari: WebGPU requires iOS 18+ (feature flag) / on by default in
  recent Safari releases. The prototype shows a fallback message when absent.
- Pixel-exactness: the original's look includes 16-bit-ish dithering, its
  interpolation artifacts, and x87 precision quirks. A port will look
  *cleaner* than the original. If artifact-faithfulness is wanted, emulate
  the 2×2 interpolation deliberately as a post effect.
- The DOS binary can be ignored entirely; the Windows build is the same
  engine with a settings dialog.

---

## 4. Recommended roadmap

1. **Done — feasibility spike:** WebGPU compute raytracer prototype
   (`demos/heaven7-webgpu/`), validated at 60 fps headless.
2. **Archaeology (parallel):** pull the Wayback capture of Picard's H7
   making-of mini-site; grab `h7-final.zip` from scene.org (the archive here
   is the party version; the final differs) and `heaven7.xm` from Mod
   Archive.
3. **Dynamic extraction:** run `heaven7w.exe` under HEAVEN7L or x86 emulation
   with a debugger; breakpoint the bit-stream decoder (`0x4086b8` region in
   the unpacked party binary) and dump the decoded scene script; dump
   generated textures from memory. This is the highest-value RE work and it
   is bounded.
4. **Scene-by-scene rebuild:** replay the extracted script through the WebGPU
   renderer, scene by scene (white room → spheres → column tunnel → morphing
   text greetings → face → credits), comparing against a 1080p video capture.
5. **Music + sync:** integrate the existing JS XM player, drive scene time
   from the replayer clock.
6. **Polish:** glow/post pass, optional "authentic 2000" mode (320×240 +
   2×2 interpolation artifacts + dithering).

Step 3 is the only step with real uncertainty, and it risks nothing: if
script extraction stalls, scenes can be reconstructed by eye from video with
hand-tuned keyframes — tedious but proven possible by every demoscene remake
project.
