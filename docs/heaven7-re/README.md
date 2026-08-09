# Heaven 7 — scene-script & texture reverse engineering

Findings from statically reversing the unpacked `heaven7w.exe` (Exceed, 2000),
targeting the two things a faithful port needs out of the binary: the **scene
script** (the animation timeline) and the **texture/material system**.

**Status: the scene script is extracted.** A headless CPU-emulation harness
(`tools/emulate_extract.py`, no Windows/Wine/GPU needed) runs the real binary
far enough to interpret the timeline and dumps it: **201 opcodes / 1735
decoded values**, checked in as
[`scene-script.txt`](scene-script.txt) (readable listing) and
[`scene-script-dump.json`](scene-script-dump.json) (raw). The stream format,
the interpreter, and the opcode dispatch table are all recovered and
documented below.

An earlier draft of this document guessed that the script was depacked into a
heap buffer; emulation disproved that — **the script is cleartext in `.data`
at `0x418dca`**, and the emulator's opcode fetches match the raw file bytes
exactly.

All addresses are RVAs in the UPX-unpacked image based at `0x00400000`. The
original binary is **not** redistributed here; point the tools at your own
`upx -d`'d copy via the `H7_EXE` environment variable.

---

## 1. The scene-script stream format (fully recovered)

The timeline is a byte stream read through a cursor held in `EDI`. Two
primitives consume it. Both are tiny and are ported faithfully, with
self-tests, in [`tools/decoders.py`](tools/decoders.py); the annotated
disassembly is in [`disasm-key-routines.txt`](disasm-key-routines.txt).

### `read_varint` — `.text:0x4086b8`

A signed variable-length integer. The low bit of the first byte is a width tag:

- bit 0 == 0 → 7-bit value: `byte >> 1`
- bit 0 == 1 → 15-bit value: `(int16)word >> 1` (arithmetic, so signed)

Used for counts, indices, and small integer parameters.

### `read_float` — `.text:0x4086c8`

A variable-length compressed IEEE-754 single. Three encodings, chosen so common
values cost the fewest bytes:

| First byte | Form | Bytes consumed | Meaning |
|---|---|---|---|
| `== 0x00` | zero | 1 | exactly `0.0` |
| bit 0 == 1 | compact | 2 | 6-bit exponent field + 9-bit mantissa |
| bit 0 == 0, `!= 0` | extended | 3 (reads a dword, uses 3 bytes) | 6-bit exponent + 17-bit mantissa |

The exponent is reconstructed as `(((stored + 0xE0) & 0x1F) + 0x7A) << 23` and
OR'd with the mantissa shifted into place. The extended form deliberately reads
4 bytes but advances only 3 (the 4th byte's bits are unused) — worth noting for
a bounds-safe reimplementation near the end of a buffer.

`python3 tools/decoders.py` runs the self-tests and shows both float forms
decoding to scene-scale values (e.g. `8.015625`, `130.0`).

---

## 2. The scene-script VM (recovered and dumped)

The timeline is a **bytecode interpreted by a VM at `.text:0x40a60e`**. Its
main loop is eight instructions:

```asm
0040a624  movzx eax, byte ptr [edi]              ; fetch opcode
0040a627  inc   edi
0040a628  or    eax, eax
0040a62a  je    0x40a63b                         ; opcode 0 = end of script
0040a62c  movzx eax, word ptr [eax*2 + 0x440f67] ; dispatch table lookup
0040a634  add   eax, dword ptr [ebp]             ; + image base (0x400000)
0040a637  call  eax                              ; run handler (consumes args from EDI)
0040a639  jmp   0x40a617                         ; loop
```

- **Script data:** `.data:0x418dca`, cleartext, spanning to ~`0x41b6f4`.
- **Dispatch table:** `.data:0x440f67`, an array of 16-bit offsets;
  `handler = 0x400000 + table[opcode]`. Roughly 30 live opcodes; entries past
  ~30 decay into unrelated data.
- Handlers pull their arguments from the same `EDI` cursor using the two
  decoders from §1, so the script is self-describing: argument counts and
  types are implied by the handler, not stored.

### Opcodes actually used by the intro

| Opcode | Count | Handler | Notes |
|---|---|---|---|
| 1 | 96 | `0x404057` | object/keyframe definition — the bulk of the script |
| 2 | 95 | `0x404154` | commit/terminate the preceding definition (pairs with op 1) |
| 23 | 3 | `0x404236` | |
| 20 | 2 | `0x4023d3` | shade-tree op (adjacent to the material evaluator) |
| 4 | 1 | `0x40303b` | bulk loader — consumes ~5.4 KB of inline data at script start |
| 7 | 1 | `0x4023bf` | shade-tree op |
| 11 | 1 | `0x4040af` | one of a family (ops 8–16 all share this handler) |
| 25 | 1 | `0x404701` | |
| 29 | 1 | `0x40402b` | |

The near-equal counts of opcodes 1 and 2 reveal the structure: 96
`define … commit` pairs, i.e. **96 scene objects/animation blocks**, preceded
by a single bulk-data load.

### Sample of the decoded values

The extracted values are unmistakably scene data — `scene-script.txt` opens
with entries like:

```
0x41a340  op1   f:0 f:0 f:0 f:0.5771 f:0.5771 f:-0.5771 f:2.094 i:0
0x41a2ee  op1   f:1 f:0 f:0 f:140 i:0 f:10 i:1000 f:1000 i:255
```

`0.5771, 0.5771, -0.5771` is a **normalised direction vector** (1/√3 ≈ 0.5774)
— a light direction — and `2.094` is **2π/3**, the ring-symmetry angle that
also appears in the animation constant pool below. That cross-check is strong
evidence the decoders and the grammar segmentation are correct.

### The original static-analysis view of the handlers

- A per-object-type handler is looked up (`.text:0x40d70a`) and invoked
  indirectly (`call edi` at `.text:0x406a33`) once per object. The scene-build
  driver at `.text:0x406973` runs these in batches (`ecx = 5`, then `3`, …),
  seeding animation parameters from a small constant pool first.
- Each handler pulls typed fields from the `EDI` stream into an object struct
  addressed by `ESI`. The handler at **`.text:0x406a39`** is fully decoded and
  gives a concrete record schema:

  ```
  object[0x174] = read_varint      ; field A (id / type)
  object[0x178] = read_varint      ; field B
  object[0x19c] = read_varint = N  ; sub-element count
  repeat N times:                  ; array at object[0x1a0], 40-byte stride
      6 × read_float               ; e.g. position(3) + colour/param(3)
      (+ 16 bytes derived/padding)
  object[0x17c..0x188] = 4 × read_float
  ```

  Other handlers (`0x408345`, `0x408767`, `0x4088bc`, `0x409658`, `0x40a7ce`,
  `0x40b04f`) follow the same shape with different field sets — these are the
  per-primitive-type parsers (sphere, plane, light, camera path, text, …).

### Animation constant pool (`.data`, near `0x440ff0`)

The scene-build driver feeds these into curve/keyframe setup
(`.text:0x40d77c`). Decoded values:

| Address | Value | Reads as |
|---|---|---|
| `0x440ff1` | `0.39` | mix / weight |
| `0x440ff5` | `1.496` | — |
| `0x441005` | `1.2566` | `2π/5` (72°) |
| `0x441015` | `0.5712` | — |
| `0x441025` | `2.0944` | `2π/3` (120°) |
| `0x441035` | `1000.0` | ms → s time scale |

The `2π/3` and `2π/5` constants are the symmetry angles for the rings of
spheres; `1000.0` converts `timeGetTime` milliseconds to seconds.

### Why extraction is dynamic rather than a pure static decode

The bytecode is self-describing: each handler decides how many values to pull
and of which type, so you cannot segment the stream without running the
handlers. Rather than reimplement ~30 handlers to find out, the emulator runs
the *original* ones and records what they consume — which is why the listing's
argument grouping can be trusted. `.data` itself needs no depacking; only the
segmentation requires execution.

---

## 3. The texture / material system

Three layers, all located. **Two corrections to earlier drafts of this
document are folded in below** — see §3.4.

### 3.1 The texture generator is a second bytecode VM (in `.data`)

`.data` contains a **static library of x86/MMX code** — not just data — and the
texture generator lives there as its own little interpreter, independent of the
scene-script VM in §2:

| Piece | Address | Notes |
|---|---|---|
| Interpreter | `.data:0x44189f` | `lodsb` fetches an operator id, then linear-searches the table |
| Operator table | `.data:0x4429b8` | 8-byte entries `{u32 id, u32 handler}`, zero-terminated |
| Operator count | — | **26** |
| MMX channel masks | `.data:0x441840` | `0x1f`, `0x7e0`, `0xf800`, `0x3e0` → RGB 565/555 |
| Work-buffer pointer | `.data:0x44292c` | buffer zeroed `0x50000` dwords (1.25 MB) per run |

Entry is by `jmp 0x44189f` from `.text:0x4089e4`, with the program pointer in
`EDI` and the destination texture buffer in `EAX`.

The 26 operator ids fall into clean families, which is what a texture-generator
operator set looks like (sources, filters, combiners, colour ops):

| Ids | Count | Handlers |
|---|---|---|
| `0x01`–`0x04` | 4 | all share `0x4418ea` (one routine, variant selected by id) |
| `0x10`–`0x15` | 6 | `0x441d20`, `0x441d57`, `0x441e67`, `0x441f0c`, `0x441fb1`, `0x442050` |
| `0x20`–`0x22`, `0x24` | 4 | `0x4420db`, `0x44211a`, `0x442207`, `0x4422f6` |
| `0x30`–`0x37` | 8 | `0x44237d` … `0x44263b` |
| `0x40`–`0x41` | 2 | `0x442698`, `0x442789` |
| `0x50`–`0x51` | 2 | `0x44281a`, `0x442899` |

A full reference disassembly of the interpreter and all 26 handlers is checked
in as [`texture-ops.txt`](texture-ops.txt) (~1600 lines), produced by
`tools/texture_ops.py`.

Emulation confirms these really do generate the textures: the handler region
for operator `0x14` executed **14.6 million times** in one run — a per-pixel
inner loop over several 256×256 buffers.

### 3.1b The texture programs — extracted, and only 7 operators are used

Emulation traced the interpreter's program pointer and opcode stream
(`tools/emulate_texture_vm.py`). The texture programs turn out to sit in
`.data` at **`0x41b6b1`**, cleartext, immediately after the scene script — and
they are tiny:

```
program 0 @ 0x41b6b1 : op 0x15 (args 01 07 78)              ; then 0x01
program 1 @ 0x41b6be : op 0x11 (args 01 07 00 00 00 00 00 10 00 00) ; then 0x01
program 2 @ 0x41b6cf : op 0x12, op 0x13, op 0x41, op 0x21   ; then 0x01
```

**Only 7 of the 26 operators are used**, exactly as the scene script uses only
9 of ~30 opcodes:

| Op | Handler | Role |
|---|---|---|
| `0x01` | `0x4418ea` | terminator — ends every program (handler shared by ids `0x01`–`0x04`) |
| `0x11` | `0x441d57` | |
| `0x12` | `0x441e67` | |
| `0x13` | `0x441f0c` | |
| `0x15` | `0x442050` | |
| `0x21` | `0x44211a` | |
| `0x41` | `0x442789` | |

Operator arguments commonly start `0x01`/`0x02` followed by `0x07`, which reads
naturally as *(target layer, channel mask = RGB)* — a hypothesis to confirm when
reversing each handler, not an established fact.

[`texture-programs.txt`](texture-programs.txt) contains the programs plus a
**focused disassembly of just these 7 handlers** — that file, not the 26-handler
reference, is the actual port target. The raw trace is in
`texture-programs-dump.json`.

Caveat: this trace covers the textures generated in the observed window (3
programs). Later scenes may generate more; a longer run with the advancing
clock will confirm whether the operator set grows beyond these 7.

### 3.2 Texture objects are resolution-parameterised — the key to a remaster

The allocator at `.text:0x4088c9` is called with **width in `EAX`, height in
`EDX`**, and builds:

```
tex[0x18] = width
tex[0x1c] = height
tex[0x20] = stride = ((width + 7) & ~7) * 4     ; 4 bytes per pixel
tex[0x24] = buffer  = alloc(stride * (height+1) + 4)   ; zero-filled, 1 guard row
```

At the call site (`.text:0x4089d5`) the size is a plain immediate:

```asm
004089d5  mov eax, 0x100        ; width  = 256
004089da  mov edx, eax          ; height = 256
004089dc  call 0x4088c9         ; allocate
004089e1  mov eax, [esi+0x24]   ; buffer
004089e4  jmp 0x44189f          ; run the texture program
```

For `width = 256` the stride is exactly **1024 bytes**, which is precisely what
the shade-tree sampler assumes — so the pieces cross-check.

**Implication for the 4× remaster:** the generator side is parameterised, so
regenerating at 1024×1024 is a matter of the size argument, not a rewrite. The
*sampler* is the hardcoded part: `.text:0x4023b7` bakes 256×256/stride-1024
into its masks (`0x3fc00`, `0x3fc`) and shifts (`>>5`, `>>0xd`). In a WGSL port
we write our own sampler anyway, so that hardcoding is irrelevant — we need the
operator *math*, at which point any resolution is free. This is exactly why
dumped texture pixels are not sufficient for a remaster and the operators must
be reimplemented.

### 3.3 Runtime shade-tree — `.text:0x4023b7`

A **recursive evaluator over 32-byte nodes** (`ESI` = node). It is the material
system the raytracer calls at each hit. Per node:

- `node[0x00]` = flag byte (`0x40`, `0x0C`, `0x04`, `0x01` select operations:
  leaf sample / blend / modulate / recurse)
- `node[0x04]` = texture base pointer
- `node[0x10]`, `node[0x14]` = fixed-point U, V (masked `>>5 & 0x3FC00` and
  `>>0xD & 0x3FC` → a 256×256, 1024-byte-stride texture index)
- `node[0x18]` = colour (packed bytes)

It samples the texture (`movd mm1,[base+index]`), unpacks bytes→words
(`punpcklbw`), modulates by the node colour (`pmullw` then `psrlw 8`), and
recurses into child nodes at `ESI+0x20` for compositing. This is a compact
procedural-material tree — the thing to port to WGSL as a small node evaluator.

### Bump-mapped shading — `.text:0x40ae01`

Samples the texture as **signed 16-bit** values (two adjacent texels,
`fimul word[idx]` and `word[idx+2]`) and multiplies by floats to perturb the
surface normal — i.e. textures are stored both as 32-bit RGBA (colour) and
signed-16 height/normal maps, and the shading does bump mapping from them.

### 3.4 Corrections to earlier analysis

Two claims in earlier drafts of this document were wrong and are retracted:

- **`.text:0x40cb14` is *not* the texture generator.** It is a teardown
  routine: it fetches the engine context, destroys the graphics objects, and
  frees the context. The MMX routine near it actually begins at
  **`0x40cb28`** and has no direct callers (it is reached through the `.data`
  code library). The real generator is the VM in §3.1.
- **There is no runtime code generation.** An earlier reading of the profile
  suggested `.data` code was JIT-emitted, because instructions inside the
  `.data` band wrote into that band 1.5 M times. Those writes target two fixed
  *variables* (`0x442930`, `0x442b2c`) that simply live among the code. `.data`
  is a static code+data library, which is simpler and better news for porting.

One scope caveat on §2 as well: the 201-opcode dump was taken with a frozen
clock. With the clock advancing, **5495** decoder tokens are seen rather than
1735, because later scenes are parsed as the intro progresses. `scene-script.txt`
is therefore the *opening* of the timeline, not the whole of it; re-running with
`emulate_texture_vm.py`'s advancing clock and a raised token cap yields the rest.

---

## 4. Extraction: how to reproduce

### A. No Windows: headless CPU emulation (`tools/emulate_extract.py`) — works

```bash
pip install unicorn pefile capstone
upx -d heaven7w.exe -o h7w_unpacked.exe
python3 emulate_extract.py     # -> script_dump.json  (201 opcodes, 1735 values)
python3 script_listing.py      # -> scene-script.txt  (readable listing)
```

Runs anywhere Python does — macOS, Linux, CI. No Wine, no GPU, no display.
Three details were necessary to get the real code this far, and are worth
knowing if you extend the harness:

- **DirectDraw v1 vtable arg counts must be exact.** Counting pushes at the
  call site fails: before `SetDisplayMode` the code pushes an extra register
  and pops it afterwards, so a push-counting stub over-pops, corrupts `ESP`,
  and the function returns to address 0. `VT_ARGS` in the script has the real
  per-slot counts.
- **`GetSurfaceDesc` must report a real pixel format.** The blitter path is
  selected from `dwRGBBitCount` and the RGB masks; zeros there send it down a
  branch that dies. The harness reports 16-bit 565.
- **The render/mixer thread is never started** (`CreateThread` returns a fake
  handle) and `timeGetTime` returns 0, so once the script is parsed the main
  loop spins — the harness stops at that point, which is exactly after the
  timeline is fully decoded.

### B. Frida on the running program (`tools/extract_scene.js`)

Still the route to take if you want the **generated texture pixels**, which
the emulator does not yet dump (it stops before texture generation completes).

If you can run the intro (native Windows, or **HEAVEN7L on Linux/macOS**, or
Wine), attach Frida and let it observe the live decode:

1. `upx -d heaven7w.exe`
2. `frida -f heaven7w.exe -l tools/extract_scene.js --no-pause`, let the intro
   play through once.
3. `scene_dump.json` is the timeline as an ordered `varint/float` token stream;
   segment it using the handler schema in §2 to get per-object keyframes.
4. `tex_*.rgba` are the generated textures — usable as assets in the WebGPU
   port immediately; feed them as `texture_2d` samples into a WGSL port of the
   §3 shade-tree.
5. Index scene events by the XM row clock (already wired in the demo) to restore
   music sync.

## 5. Roadmap to a complete port + 4× remaster

What a *complete* port needs from the binary, and where each piece stands:

| Piece | Status | Where |
|---|---|---|
| Stream decoders (`varint`, `float`) | **done**, tested port | `tools/decoders.py` |
| Scene-script VM + dispatch table | **done** | §2 |
| Scene timeline (opening) | **extracted** | `scene-script.txt` |
| Scene timeline (full, clock advancing) | needs one longer run | §3.4 caveat |
| Texture generator VM + operator table | **located, 26 ops disassembled** | §3.1, `texture-ops.txt` |
| Texture programs (per-texture byte streams) | **extracted** | `texture-programs.txt` |
| Texture operator *semantics* | **not yet reversed** — the main remaining work | **7** handlers (not 26) |
| Shade-tree / material evaluator | characterized | §3.3 |
| Bump mapping | characterized | §3.3 |
| Music | **solved** — XM module + JS replayer | `demos/heaven7-webgpu` |
| Raytracer | **reimplemented** in WGSL | `demos/heaven7-webgpu` |

### The remaining work, honestly scoped

The generative machinery is now *mapped* but not yet *understood*: knowing that
operator `0x32` lives at `0x4423ee` is not the same as knowing it is (say) a
directional blur. Turning the map into a port means reading 26 short MMX
routines and writing each as a WGSL compute pass. They are small (typically
30–120 instructions, mostly packed-integer arithmetic over a scanline), and
they are all in `texture-ops.txt`, but 26 × careful reading is the bulk of the
work left. Budget that as the real task, not as a detail.

The recommended order:

1. ~~Trace the texture programs first~~ — **done**: see §3.1b. The pruning paid
   off, cutting the work from 26 operators to **7**, each a short MMX routine.
2. **Reimplement operator-by-operator**, validating each against the emulator:
   run the original operator on a known input buffer, dump the result, and
   diff it against the WGSL pass. This gives per-operator ground truth instead
   of a whole-image guess at the end.
3. **Then scale.** Because the generator is size-parameterised (§3.2) and the
   WGSL sampler is ours, 1024×1024 costs only a uniform change. Keep the
   original 256×256 path as the "authentic" mode and 1024×1024 as the
   remaster; both then come from the *same* operator code, which is the whole
   reason to port the generator rather than ship baked pixels.
4. **Re-derive the sampler in WGSL** with 12-bit UV instead of the original's
   hardcoded 8-bit masks, and keep bump mapping reading the signed-16 layer.

## Files

- **`scene-script.txt`** — the extracted timeline: 201 opcodes with decoded
  arguments. The primary artifact.
- **`scene-script-dump.json`** — the same data raw (opcode stream + token
  stream with stream offsets), for programmatic use by the port.
- **`tools/emulate_extract.py`** — the headless emulator that produces it.
- `tools/script_listing.py` — turns the JSON dump into the readable listing.
- `tools/decoders.py` — tested ports of `read_varint` / `read_float`.
- `tools/analib.py` — PE loader + capstone disassembler (needs `H7_EXE`).
- `tools/annotate.py` — prints annotated disassembly of the key routines.
- **`texture-ops.txt`** — reference disassembly of the texture VM: interpreter
  plus all 26 operator handlers.
- **`texture-programs.txt`** — the intro's actual texture programs plus a
  focused disassembly of only the 7 operators they use. The port target.
- `texture-programs-dump.json` — raw trace (program pointers + opcode stream).
- `tools/texture_ops.py` — regenerates `texture-ops.txt` from the binary.
- `tools/texture_programs.py` — regenerates `texture-programs.txt` from a trace.
- `tools/emulate_texture_vm.py` — emulator variant with an advancing clock that
  traces the texture VM (program pointers + operator stream) and reaches the
  later scenes.
- `tools/extract_scene.js` — Frida dynamic extractor (scene tokens + textures).
- `disasm-key-routines.txt` — checked-in annotated disassembly of the four
  routines above, so the analysis is readable without the binary.
