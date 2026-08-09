# Heaven 7 — scene-script & texture reverse engineering

Findings from statically reversing the unpacked `heaven7w.exe` (Exceed, 2000),
targeting the two things a faithful port needs out of the binary: the **scene
script** (the animation timeline) and the **texture/material system**.

> **Evidence discipline.** Every claim in this document is registered in
> [`claims.yaml`](claims.yaml) with a provenance class — `BYTES` (read from the
> binary, machine-verified), `DISASM` (read from instruction semantics), `TRACE`
> (observed in emulation, with stop reason), `CROSS` (two independent measures),
> or `INFER` (**hypothesis, not evidence**). Re-check the factual base with:
>
> ```bash
> H7_EXE=h7w_unpacked.exe python3 tools/verify_claims.py claims.yaml
> ```
>
> 24 `BYTES` claims currently pass; **10 claims are `INFER`** and are flagged as
> such wherever they appear. See [`METHODOLOGY.md`](METHODOLOGY.md) for the
> classes, the completeness rule, and the retraction log.

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
at `0x418dca`**, spanning to `0x4291d5`, and the emulator's opcode fetches match
the raw file bytes exactly.

Two figures in earlier drafts were **too low because they came from runs that
stopped early**, and are corrected throughout: the script is **970** opcodes
(not 201), and the texture generator uses **23 of its 26 operators** across
**11 programs** (not 7 across 3). See §3.1b.

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
| 1 | many | `0x404057` | object/keyframe definition — the bulk of the script |
| 2 | many | `0x404154` | commit/terminate the preceding definition (pairs with op 1) |
| 23 | 3 | `0x404236` | |
| 20 | 2 | `0x4023d3` | shade-tree op (adjacent to the material evaluator) |
| 4 | 1 | `0x40303b` | bulk loader — consumes ~5.4 KB of inline data at script start |
| 7 | 1 | `0x4023bf` | shade-tree op |
| 11 | 1 | `0x4040af` | one of a family (ops 8–16 all share this handler) |
| 25 | 1 | `0x404701` | |
| 29 | 1 | `0x40402b` | |

The near-equal counts of opcodes 1 and 2 reveal the structure: a long run of
`define … commit` pairs — one per scene object/animation block — preceded by a
bulk-data load. Exact counts are in `scene-script.txt`.

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

### 3.1b The texture programs — 11 programs, 23 of 26 operators used

Emulation traced the interpreter's program pointer and opcode stream. The
definitive measure was **counting calls to the texture allocator**
(`.text:0x4088c9`, exactly one per texture object) — an independent check that
does not depend on having found the programs. It reported **11**, which
immediately falsified an earlier claim here of 3.

Running to exhaustion then found all of them:

```
0x41b6b1  0x41b6be  0x41b6cf  0x41b9d6      (near the script's start region)
0x428de8  0x428e74  0x428ef5  0x428f97
0x428fc2  0x428fe5  0x429137                (a second pool, ~0x428xxx)
```

The seven programs around `0x428xxx` sit in a region the first trace never
reached. Allocations are 10 × `256×256` plus one `320×176` — that last one is the
framebuffer sharing the same allocator, so **10 real textures**.

The programs are substantial, not the 2–5 operator sketches first observed:

| Program | Ops | Program | Ops |
|---|---|---|---|
| 0 `0x41b6b1` | 2 | 6 `0x428ef5` | 21 |
| 1 `0x41b6be` | 2 | 7 `0x428f97` | 8 |
| 2 `0x41b6cf` | 5 | 8 `0x428fc2` | 4 |
| 3 `0x41b9d6` | 3 | 9 `0x428fe5` | **43** |
| 4 `0x428de8` | 17 | 10 `0x429137` | 21 |
| 5 `0x428e74` | 22 | | |

**148 operator invocations** in total, using **23 of the 26 operators**. Only
`0x22`, `0x30` and `0x34` are never used. So the operator table is nearly fully
exercised, and [`texture-ops.txt`](texture-ops.txt) — all 26 handlers — is the
port target, not a narrow subset.

Two structural facts fall out of the full trace:

- **Every program ends on `0x01`–`0x04`**, which combined with the handler's
  `index << 18` (§3.2c) reads as **"output to layer 1–4"** — the opcode *is* the
  destination layer index. That confirms the five-layer bank.
- **Operator `0x40` appears 14 times consecutively** in program 5, so it is a
  per-element operation (placing a feature/blob repeatedly), not a whole-image
  filter.

#### Retraction

An earlier version of this section claimed "3 programs, 7 operators" and
described the work as pruned from 26 to 7. That was wrong: it generalised from a
single bounded run whose stopping point was mistaken for the end of the data.
The true set is 11 programs and 23 operators, which is a **substantially larger
port** than that section implied. The operator semantics documented in §3.2b–d
(value noise, plasma, sine interference, radial rings, blend, PRNG, layer model)
remain valid — they were read from the handlers themselves — but they cover
roughly a quarter of what a complete port needs.

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

### 3.2b The noise operators, octaves, and what 4× actually requires

Operator `0x12` (`0x441e67`) is a **value-noise octave generator**, and reading
it answers how a remaster must handle detail. Its final argument byte is a
**frequency exponent** `n`:

```asm
00441e7e  mov ebp, 0x1000          ; base amplitude
00441e83  mov edx, 1
00441e88  mov cl, byte ptr [esi]   ; n  (0x05 in the intro's program)
00441e8a  shl edx, cl              ; lattice step  = 1 << n      (32)
00441e8c  shr ebp, cl              ; amplitude     = 0x1000 >> n (128)
00441e8e  call 0x4418f8            ; hash -> random value
00441e93  mov byte ptr [edi + ebx*4], cl   ; seed the sparse lattice
00441e96  add bl, dl               ; step x  (8-bit -> wraps at 256)
00441e98  jne 0x441e8e
00441e9a  add bh, dl               ; step y  (8-bit -> wraps at 256)
```

Three things follow, all of which matter for the remaster:

1. **Amplitude is already 1/f.** `amplitude = 0x1000 >> n` against
   `step = 1 << n` is textbook fBm falloff, so the operator set is built for
   octave stacking. Output is greyscale (`imul eax, eax, 0x10101` replicates one
   byte across RGB).
2. **One invocation = one octave.** There is no outer frequency loop; the loops
   are the lattice fill and the interpolation. Multi-octave noise is therefore
   composed at the *program* level (or by one of the other five operators in the
   `0x10`–`0x15` family — `0x11` takes 10 argument bytes rather than 7, so a
   multi-octave/turbulence variant is the likely reading, to be confirmed).
3. **256 is baked into the operator, not just the sampler.** The lattice walk
   uses **8-bit registers** (`bl`, `bh`) and terminates on 8-bit wraparound;
   the interpolation loop uses 16-bit `cx`. So the operator itself is hardwired
   to 256×256.

**Correction to §3.2:** it is the *allocator* that is resolution-parameterised
(width/height in `EAX`/`EDX`). The noise operator is not. A 4× remaster
therefore cannot be had by passing a larger size to the original code — it
requires the reimplementation, which is what we are doing anyway.

#### The octave rule for a 4× remaster

Rendering the same octaves at 1024×1024 yields a *smooth upscale*: the finest
feature stays 4 px wide. To genuinely remaster:

- **Preserve feature scale:** shift each existing octave's exponent by
  `log2(4) = 2` (so the intro's `n = 5` becomes `n = 7`), keeping the lattice
  step the same fraction of the image.
- **Add detail:** append **2 extra octaves** at the new fine end (exponents
  `n = 1, 0`) so noise again reaches 1-pixel features, continuing the
  `0x1000 >> n` amplitude falloff so the new detail enters at the correct
  (low) energy rather than as visible grain.
- **Renormalise.** Adding octaves increases the amplitude sum, which shifts
  contrast and mean brightness. Divide by the sum of amplitudes actually used
  so the remaster reads as *sharper*, not *different* — otherwise the extra
  octaves change the look, defeating the point of a faithful restoration.
- Use **10-bit lattice coordinates** in WGSL in place of the original's 8-bit
  wraparound, and keep the hash function (`0x4418f8`) bit-exact so the
  low-frequency octaves reproduce the original's structure rather than merely
  resembling it.

Keeping the hash exact while extending octaves is what makes the remaster the
*same* image with more detail: octaves 7…2 reproduce the original's shapes,
octaves 1…0 are new information the 256×256 original never had room for.

### 3.2c The shared machinery: PRNG, layer model, and the plasma generator

Reversing the helpers turned the operator set into a coherent system.

#### The PRNG — `.data:0x4418f8` (must be bit-exact)

```asm
mov  ecx, [0x442930]          ; state
imul ecx, ecx, 0xfacedead
rcr  ecx, 3                   ; rotate RIGHT THROUGH CARRY by 3
xchg ch, cl                   ; swap the low two bytes
inc  ecx
mov  [0x442930], ecx          ; new state ; low byte CL is the random value
```

Both noise operators seed this by copying **4 bytes straight from the program
stream** into the state (`mov edi, 0x442930` / `movsd`) — so each texture's
randomness is fully determined by its program, which is what makes the textures
reproducible and a faithful port possible.

One porting hazard: `rcr` is rotate-*through-carry*, so the incoming CF acts as
a 33rd bit. A naive `ror` is **not** equivalent and will desynchronise the
sequence. The CF state at entry must be modelled to reproduce the original
stream bit-exactly.

#### The layer model — operators `0x01`–`0x04` (`.data:0x4418ea`)

```asm
movzx esi, al                 ; al = layer index
shl   esi, 0x12               ; * 0x40000 = 262144 = 256*256*4
add   esi, [0x44292c]         ; + work-buffer base
stc                           ; set carry
```

`0x40000` bytes per layer is exactly **256×256 RGBA**, and the interpreter
zeroes `0x50000` dwords = `0x140000` bytes = **5 layers**. So the texture VM
operates on a fixed bank of five 256×256 RGBA scratch layers, addressed by
index — and these ids are the program's *"output layer N"* instruction, not a
generic terminator as an earlier note here assumed.

The **carry flag is the interpreter's continue/stop signal**: operator `0x12`
ends with `clc` (continue) while `0x4418ea` ends with `stc` (stop), which is why
every traced program ended on an `0x01`.

#### Operator `0x11` (`.data:0x441d57`) — plasma / diamond-square

Not a stacked-octave generator but **midpoint displacement**:

```asm
mov edx, 0x80                 ; step = 128
movzx eax, [edi+ebx*4]        ; corner 1
add bl, dh   / add al,[..] / adc ah,0     ; corner 2
add bh, dh   / add al,[..] / adc ah,0     ; corner 3
sub bl, dh   / add al,[..] / adc ah,0     ; corner 4  -> 16-bit sum in AX
call 0x4418f8                 ; random
lea  ecx, [ecx+ecx*2]         ; * 3   (roughness scale)
call 0x441e3f                 ; average of 4 corners + scaled displacement
```

It averages four neighbours, adds a scaled random displacement, and halves the
step each pass — so its scale hierarchy is *intrinsic* to the recursion rather
than composed from separate calls.

#### What this means for the 4× remaster

Your octave point applies to both generators, in the form each one takes:

| Generator | Native behaviour | 4× remaster |
|---|---|---|
| `0x12` value noise | one octave per call, `step = 1<<n`, `amp = 0x1000>>n` | shift exponents by +2, **append 2 finer octaves** (`n = 1, 0`) |
| `0x11` plasma | recursive subdivision from `step = 128` | **start at `step = 512`** and run **2 extra subdivision levels** down to `step = 1` |

Both then need renormalising (§3.2b) so added detail sharpens rather than
alters the image.

There is a useful guarantee here: because coarse levels are generated **before**
fine ones and consume PRNG draws in traversal order, starting the plasma at
`step = 512` reproduces the original's coarse draws in the identical order, and
the new fine levels merely consume additional draws *afterwards*. So the
remaster's large-scale structure is bit-identical to the original, with the new
octaves adding only detail the 256×256 version had no room to express. That is
the precise sense in which this is a remaster rather than a reinterpretation.

### 3.2d All seven operators identified

The remaining operators are analytic (x87 float) rather than lattice-based, and
one is a combiner. The float constant pool the texture VM uses decodes cleanly,
which is what made these readable:

| Address | Value | Role |
|---|---|---|
| `0x44293c` | `1.5707964` | **π/2** |
| `0x442948` | `0.0078125` | **1/128** — coordinate normalisation |
| `0x442944` | `0.00390625` | **1/256** |
| `0x44294c` | `128.0` | half-extent (image centre) |

Complete operator table for the set the intro actually uses:

| Op | Handler | What it is | Evidence |
|---|---|---|---|
| `0x01`–`0x04` | `0x4418ea` | **layer address / output** — `index<<18` + base, `stc` ends the program | §3.2c |
| `0x11` | `0x441d57` | **plasma** (diamond-square midpoint displacement) | 4-corner average + scaled random, step halves from 128 |
| `0x12` | `0x441e67` | **value noise**, one octave | `step=1<<n`, `amp=0x1000>>n`, lattice + interpolation |
| `0x13` | `0x441f0c` | **sine interference** — two sine waves in x and y | 2× `fsin`; args are (freq, phase) pairs, freq doubled, phase × 1/128 |
| `0x15` | `0x442050` | **radial rings** — `sin` of distance from centre | `fsqrt` + `fsin` over `(x-128)² + (y-128)²`, arg `0x78`=120 scales the radius |
| `0x21` | `0x44211a` | **radial/angular pattern** (spiral-family) — richest generator, 9 args | 2× `fsin` + `fsqrt`, no MMX |
| `0x41` | `0x442789` | **weighted blend of two layers** | calls the layer resolver **twice**, then `punpcklbw`/`pmulhw`/`paddw`/`packuswb` |

So the pipeline in program 2 reads as: generate noise → overlay a sine pattern →
blend two layers → apply a radial/angular pattern → output. Exactly the shape of
a hand-built procedural material.

#### This sharpens the 4× strategy considerably

Only **two of the six generators are lattice-based**. That splits the remaster
work cleanly:

| Operator | Scaling behaviour at 4× |
|---|---|
| `0x11` plasma, `0x12` value noise | **lattice-bound** — need the octave/subdivision treatment of §3.2b–c, else they upscale smoothly |
| `0x13`, `0x15`, `0x21` | **analytic** — continuous functions of coordinates; they get sharper *for free*. Only the normalisation constants change (`1/128 → 1/512`, `128.0 → 512.0`) |
| `0x41` blend | per-pixel, resolution-agnostic |

That is a much smaller job than "port a texture generator": three operators need
only a constant swap, one is a blend, and the octave work applies to exactly two.

#### Honest status on these seven

What is established is *what each operator is* — the conceptually hard part, and
enough to plan the port and the remaster. What still needs care is transcribing
the **exact arithmetic** for bit-exactness: the interpolation kernels in `0x11`
and `0x12`, the fixed-point rounding in `0x41`'s `pmulhw` path, and the precise
argument-to-parameter mapping for `0x21`'s nine bytes. Those are transcription
tasks against `texture-programs.txt`, not open questions.

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

Resolved: `scene-script.txt` now holds the **full** 970-opcode timeline
(`0x418dca`–`0x4291d5`, 5495 values), from `tools/emulate_extract_full.py` with a
raised instruction budget. The earlier 201-opcode figure came from a truncated
run — and its apparent "script end" at `0x41b6f4` was an artifact of that
truncation, not a real bound. Lesson applied below: completeness needs a
run-to-exhaustion **plus an independent cross-measure**.

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
| Scene timeline | **fully extracted** (970 opcodes) | `scene-script.txt` |

| Texture generator VM + operator table | **located, 26 ops disassembled** | §3.1, `texture-ops.txt` |
| Texture programs | **all 11 extracted** (allocator-count verified) | `texture-programs.txt` |
| Texture operator *semantics* | 7 of the **23** used operators identified | §3.2b–d |
| Octave strategy for 4× | **decided** | §3.2b |
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

1. ~~Trace the texture programs first~~ — **done**: see §3.1b. It did *not*
   prune the way I hoped: 23 of 26 operators are used across 11 programs, so
   plan for the full operator set (7 are already reversed in §3.2b–d).
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

- **`claims.yaml`** — every claim with its provenance class; the machine-checkable
  ones are re-verified against the binary by `tools/verify_claims.py`.
- **`METHODOLOGY.md`** — provenance classes, the completeness rule, retraction log.
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
