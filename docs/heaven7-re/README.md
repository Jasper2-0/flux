# Heaven 7 — scene-script & texture reverse engineering

Findings from statically reversing the unpacked `heaven7w.exe` (Exceed, 2000),
targeting the two things a faithful port needs out of the binary: the **scene
script** (the animation timeline) and the **texture/material system**.

**Status in one line:** the *formats* and the *architecture* are recovered and
documented here, with a tested Python port of the stream decoders; a complete
semantic dump of the scene script is best finished by a short dynamic capture
(Frida script included), because the script is depacked into a heap buffer and
walked by a pointer-driven interpreter that static analysis can characterize
but not cleanly evaluate offline.

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

## 2. The parser architecture (recovered; drives the dynamic capture)

The scene is built by a **data-driven interpreter**, not a flat table:

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

### Why the full dump is a dynamic step

The stream `EDI` points into a runtime buffer (allocated through the zero-fill
`GlobalAlloc` wrapper at `.text:0x4015c7`), and which handler runs next depends
on values already read — a classic self-describing bytecode. Reproducing it
offline means re-implementing the whole interpreter *and* locating the buffer's
contents at the right moment. Observing the running program is far cheaper and
exact. That is what [`tools/extract_scene.js`](tools/extract_scene.js) does:
it hooks the two decoders (logging `cursor → value` in stream order), captures
one full object struct, and dumps every texture buffer the shader touches.
Output is `scene_dump.json` + `object_record.bin` + `tex_*.rgba`.

---

## 3. The texture / material system (characterized)

Heaven 7 has two texture-related layers, both found:

### Runtime shade-tree — `.text:0x4023b7`

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

### Offline generator — `.text:0x40cb14` (called once from init `0x401238`)

The buffer-filling generator: an MMX integer routine using a constant table at
`.data:0x441840` with shift-heavy lattice arithmetic (`pslld 7` / `psrld 0xD`)
— a hash/value-noise synthesizer that writes the texture buffers the shade-tree
later samples. Reversing its exact noise math is the remaining piece if you
want to regenerate textures procedurally rather than dumping them; dumping them
at runtime (the Frida script) gives you the exact pixels immediately, and the
generator can be re-derived later against those as ground truth.

---

## 4. Recommended path to a complete extraction

1. `upx -d heaven7w.exe`
2. `frida -f heaven7w.exe -l tools/extract_scene.js --no-pause`, let the intro
   play through once (Wine works).
3. `scene_dump.json` is the timeline as an ordered `varint/float` token stream;
   segment it using the handler schema in §2 to get per-object keyframes.
4. `tex_*.rgba` are the generated textures — usable as assets in the WebGPU
   port immediately; feed them as `texture_2d` samples into a WGSL port of the
   §3 shade-tree.
5. Index scene events by the XM row clock (already wired in the demo) to restore
   music sync.

## Files

- `tools/decoders.py` — tested ports of `read_varint` / `read_float`.
- `tools/analib.py` — PE loader + capstone disassembler (needs `H7_EXE`).
- `tools/annotate.py` — prints annotated disassembly of the key routines.
- `tools/extract_scene.js` — Frida dynamic extractor (scene tokens + textures).
- `disasm-key-routines.txt` — checked-in annotated disassembly of the four
  routines above, so the analysis is readable without the binary.
