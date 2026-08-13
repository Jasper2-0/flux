# Porting the Farbrausch V2 Synthesizer to modern AU / VST3 — Feasibility Assessment

*Assessment date: 2026-08-13. Source analyzed: [`farbrausch/fr_public/v2`](https://github.com/farbrausch/fr_public/tree/master/v2) @ master.*

## Verdict

**Difficulty: moderate — and considerably easier than the codebase's reputation suggests.**

The V2's fearsome reputation comes from its original implementation: ~5,900 lines of
size-optimized 32-bit x86 assembly (`synth.asm`), written for NASM, littered with
`pushad`/`popad` tricks and a single global state block. **That problem has already been
solved.** The repository contains `synth_core.cpp` — a complete, faithful C++ port of the
entire sound core (3,352 lines, written by Fabian "ryg" Giesen) that is instance-based,
supports arbitrary sample rates, has no global mutable state, and even preserves
original V2 bugs behind compatibility switches (e.g. `BUG_V2_FM_RANGE`).

**Verified hands-on during this assessment** (Linux x86_64, g++ 13):

- `synth_core.cpp` compiles with exactly **one** code change (`vsprintf_s` →
  `vsnprintf`) plus a ~20-line portable replacement for `types.h`
  (the original typedefs `unsigned long` as `sU32`, which breaks on LP64 platforms
  like 64-bit macOS/Linux).
- `v2mplayer.cpp` (the V2M music player) contains four small `__asm` blocks — all
  trivial 64-bit fixed-point math or a memset — replaced with ~10 lines of standard C++.
- `ronan.cpp` (the speech synthesizer) contains three x87 helper functions
  (`sFtol`, `sFPow`, `sFExp`) — replaced with `lrintf` / `pow` / `exp` one-liners.
- With those shims in place, **both bundled V2M tunes ("Patient Zero" and
  "Zeitmaschine") render correct audio to WAV on 64-bit Linux**, with and without the
  speech synth compiled in. Total effort: under an hour.

The actual porting work is therefore **not DSP work**. It is (a) writing a modern
plugin wrapper (VST3 + AU) around a clean, already-portable C library, and (b)
rebuilding the editor GUI, which is ~6,700 lines of Windows-only WTL code and must be
rewritten from scratch. The GUI is the bulk of the effort.

**Estimated effort (experienced C++ audio developer):**

| Milestone | Scope | Estimate |
|---|---|---|
| 1. Headless plugin | VST3+AU shell, MIDI in, bank loading, program change, state save/restore, generic parameter view | 1–2 weeks |
| 2. Full editor GUI | ~90 patch params + ~30 global params, topic panels, mod-matrix editor, patch/bank management | 4–8 weeks |
| 3. Polish / parity | Ronan lyrics editing, VU meters, V2M export recorder, CI, code-signing/notarization for AU | 2–4 weeks |

A functional, music-making VST3/AU can exist in about two weeks; a polished,
feature-complete release is roughly a 2–3 month project.

---

## 1. What V2 is

V2 ("Viruz II", 2000–2008, Tammo "kb" Hinrichs) is the softsynth behind fr-08,
kkrieger, Debris and most Farbrausch productions:

- **16-part multitimbral**, MIDI-driven, up to 64 voices, 128 programs per bank.
- Per voice: 3 oscillators (saw/tri, pulse, sine, noise, FM, aux routing), ring mod,
  2 multi-mode filters (incl. Moog models) with serial/parallel/single routing,
  distortion, 2 ADSR-ish EGs, 2 LFOs, flexible modulation matrix.
- Per channel: distortion, chorus/flanger, compressor, bass boost.
- Global: stereo delay, Sean-Costello-style reverb, low-shelf EQ, DC filter, two aux buses.
- **Ronan**: a formant-based speech synthesizer that sings lyrics fed via text.
- **V2M**: a compact music file format + player (`v2mplayer.cpp`), used for demo
  soundtracks; `conv2m` converts old V2Ms to the current patch layout.

## 2. Component inventory

| Component | Files | Lines | Portability status |
|---|---|---:|---|
| Sound core (asm, historical) | `synth.asm` | 5,932 | **Ignore.** Superseded by `synth_core.cpp` |
| Sound core (C++) | `synth_core.cpp`, `synth.h` | 3,352 | **Clean.** No asm, no Windows headers, instance-based, variable sample rate. Compiles on g++/x64 with a 1-line fix |
| Type definitions | `types.h` | 38 | MSVC-isms (`__int64`, `unsigned long` as 32-bit, `__stdcall`). Replace with `<cstdint>` shim — trivial, **required** on LP64 targets |
| Speech synth | `ronan.cpp`, `phonemtab.h` | 741 | 3 tiny x87 asm helpers → `lrintf`/`pow`/`exp`. Verified portable |
| V2M player | `v2mplayer.cpp/h` | 770 | 4 tiny asm blocks → plain C++. Verified portable. Also duplicates the broken typedefs — same fix |
| V2M converter | `v2mconv.cpp/h`, `sounddef.cpp/h` | 1,100 | `#pragma intrinsic` (ignorable), one `MessageBox` + `windows.h` in `sounddef.cpp` — minutes of work |
| Patch/parameter metadata | `sounddef.h` | 371 | Pure data tables: every parameter's name, range, default, UI type, mod-target flag. **This is the data-driven backbone for a new GUI and VST3 parameter model** |
| Factory presets | `presets.v2b` | 126 KB | Binary bank, public domain, loadable via `sounddef.cpp` |
| VST2 wrapper | `vsti/vsti.cpp`, `vstiext.cpp` | 952 | Reference only. VST2 SDK is no longer licensable — must be replaced, not ported |
| Editor GUI | `vsti/v2view.h`, `wtlblaview.h`, misc | ~6,700 | **Full rewrite.** WTL/Win32, GDI drawing, skin bitmaps ("appearances"). Nothing reusable except as a spec |
| Audio/MIDI I/O | `dsio.asm`, `soundsys.cpp`, `libv2/` | ~1,300 | Irrelevant — the plugin host provides audio and MIDI |

### The synth API is already plugin-shaped

The entire core is driven through a C API that maps almost 1:1 onto a plugin
processor (`synth.h`):

```c
synthInit(pthis, patchmap, samplerate);   // patchmap = bank data
synthProcessMIDI(pthis, midibytes);       // raw MIDI stream, incl. program change
synthRender(pthis, buf, nsamples, ...);   // interleaved stereo float out
synthSetGlobals(pthis, ptr);              // reverb/delay/EQ settings
synthSetLyrics(pthis, ptr);               // Ronan speech text
synthGetChannelVU / synthGetMainVU        // metering
```

Internally the core processes in frames of 128 samples @ 44.1 kHz (scaled with
sample rate), and `synthRender` already handles arbitrary caller block sizes. MIDI
just needs to be interleaved with render calls at event timestamps to get
sample-accurate(ish) timing — exactly what the old VST2 wrapper (`vsti.cpp`, 478
lines) did. It is a straightforward template for the new wrapper.

## 3. Gap analysis: what a modern AU/VST3 needs

| Requirement | Status | Work |
|---|---|---|
| 64-bit build | Solved by `synth_core.cpp` + fixed `types.h` | Trivial |
| x86_64 and arm64 (Apple Silicon) | No SIMD, no endian issues (all targets little-endian), pure scalar float C++ | Recompile; verify with tests |
| Variable sample rate | `calcNewSampleRate()` supports it. Internal frame buffer caps at 280 samples → works up to ~96 kHz; 192 kHz needs a one-constant bump + validation. A couple of constants have acknowledged sample-rate-dependence bugs (marked `@@@BUG` in source) — keep for authenticity or fix behind a switch | Small |
| Multiple instances | Core is fully instance-based, zero global mutable state (verified) | None |
| VST3 wrapper | Does not exist; VST2 wrapper is a good spec | Moderate |
| AU wrapper | Does not exist | Free if using JUCE/iPlug2 |
| State save/restore | Old wrapper stored the whole bank as a chunk (`programsAreChunks`); same approach works in VST3/AU | Small |
| Parameter automation | See risk #1 below — design decision needed | Moderate |
| Editor GUI | Must be rebuilt | **Large — the main cost** |
| Denormal protection | Original used epsilon constants; on modern hosts also set FTZ/DAZ in the process callback | Trivial |

## 4. Risk register

1. **VST3/AU parameter-model mismatch (design risk, not code risk).** V2 is
   bank/program-oriented and 16-part multitimbral with byte-valued parameters edited
   via its own GUI; VST3 and AU want a flat set of normalized automatable parameters
   for a single logical instrument. Options: (a) expose only the *active program's*
   ~90 parameters as automatable parameters (as most V2-era romplers do), (b) chunk-only
   state with GUI editing and no host automation (what the original VST2 did), or
   (c) 16 × 90 parameters (noisy; not recommended). Recommendation: start with (b)
   for milestone 1, add (a) in milestone 2. Multitimbrality itself works fine in
   VST3/AU as a single stereo-out instrument receiving 16 MIDI channels.

2. **GUI effort dominates.** ~6,700 lines of WTL, custom-drawn controls and bitmap
   skins ("appearances"). The saving grace: `sounddef.h` defines every topic, control
   type, range and label as data, so a data-driven editor gets ~80% of the way
   cheaply. Pixel-faithful recreation of the classic skin is the long tail. The skin
   bitmaps in `appearances/` are covered by the public-domain grant only if taken
   from the listed dirs — check per-file before shipping (see §5).

3. **Bit-exactness vs. the original.** x87 → SSE float behavior means renders will
   not be bit-identical to 2004-era output (already true of `synth_core.cpp` on any
   modern compiler). Musically irrelevant; only matters for demoscene archaeology.
   The core's bug-compat defines show the author already cared about behavioral
   fidelity — keep them on.

4. **Old V2M/patch versions.** `v2mconv.cpp` + `sounddef.cpp` handle patch-format
   versioning (`v2vsizes` tables). Ship the converter path so old banks/V2Ms load.

5. **Ronan text input.** Lyrics arrive via `synthSetLyrics` (the old GUI had a text
   page; V2Ms embed speech). Needs a small custom UI page; no DSP risk (verified
   compiling & running).

## 5. Licensing

- **Core**: kb's release note places the main directory plus `bin`, `conv2m`,
  `in_v2m`, `libv2`, `tinyplayer` and `tool` **in the public domain** — this covers
  `synth_core.cpp`, `synth.asm`, `ronan.cpp`, `v2mplayer`, `v2mconv`, `sounddef.*`
  and `presets.v2b`. Unencumbered.
- **`vsti/` directory** (old wrapper + GUI): separate **BSD-2-clause** license
  (Ritter/Hinrichs) — fine to use as reference or port, with attribution.
- **`v2m/` example tunes**: CC-BY. "Patient Zero" in `tinyplayer/tune.asm` says
  "do not redistribute" — don't ship the demo tunes.
- **VST2 SDK headers are dead** (Steinberg no longer licenses VST2) — the old
  wrapper cannot legally be rebuilt as-is, which is fine since VST3 is the target.
  VST3 SDK: dual GPLv3 / proprietary. JUCE: AGPL or commercial. AU: Apple's
  headers, no issue.

## 6. Prior art (worth mining before writing anything)

- [jgilje/v2m-player](https://github.com/jgilje/v2m-player) — SDL port of the
  tinyplayer using `synth_core.cpp`; Linux/macOS; confirms the portability route
  taken in this assessment (and has already made the same `types.h`/asm fixes).
- [murkymark/v2synth](https://github.com/murkymark/v2synth) — V2 as a pure C++,
  g++-compatible library.
- [EZForever/Viruz2](https://github.com/EZForever/Viruz2) — "V2 revived": modern
  C++ V2 with libv2 + Ronan, x86/x64.
- [trancefish/V2-Juce-Fork](https://codeberg.org/trancefish/V2-Juce-Fork) — a JUCE
  fork of Viruz2 — the closest existing thing to the goal; evaluate before starting
  from scratch.
- [dejansubotin/Farbrausch-V2-VST3-x64](https://github.com/dejansubotin/Farbrausch-V2-VST3-x64) —
  VST3/x64 wrapper around the original synthesis path with chunk/state bridging.
- [webV2M](https://www.wothke.ch/webV2M/) — WASM/Web Audio port; proves the core
  runs on non-x86 targets.

None of these (as of this assessment) is a polished cross-platform AU+VST3 with a
complete recreated editor — that combination is the genuinely new work.

## 7. Recommended approach

1. **Do not touch the DSP.** Vendor `synth_core.cpp` verbatim; keep all fixes in a
   small portability header (`types.h` replacement + `__stdcall` no-op). Add FTZ/DAZ
   guards in the process callback.
2. **Use JUCE** (AGPL is fine for a public-domain-cored open-source project) to get
   VST3 + AU + AUv3 + standalone from one codebase. iPlug2 (MIT) is the alternative
   if AGPL is unacceptable.
3. **Milestone 1 — headless instrument**: wrap the C API, translate host MIDI events
   into the `synthProcessMIDI` byte stream split at event offsets, load
   `presets.v2b`, program change via MIDI, whole-bank state chunks. Generic
   slider UI generated from `sounddef.h` tables. This alone is a usable instrument.
4. **Milestone 2 — real editor**: data-driven panels per topic, mod-matrix list
   editor, patch naming/copy, bank import/export (`.v2b`), optional automatable
   parameters for the active program.
5. **Milestone 3 — character & extras**: classic skin, Ronan lyrics page, VU meters,
   V2M export (port `v2mrecorder.cpp`), old-format converters, CI with pluginval,
   macOS signing/notarization.
6. **Testing**: golden-master WAV renders of the bundled V2Ms per platform/sample
   rate (the render harness built for this assessment is exactly that test).

## 8. Note on Flux

If the eventual goal is using V2 inside Flux (Rust): don't rewrite the DSP in Rust.
The clean C API (`synthInit`/`synthProcessMIDI`/`synthRender`) is ideal for a small
`unsafe` FFI crate wrapping the vendored C++ core — a couple of days of work — while a
faithful Rust rewrite of the 3,352-line core would be a multi-month project with high
regression risk. A `v2-sys` crate + safe wrapper would slot into a Flux audio
operator naturally.

---

### Appendix: verification log (this assessment)

Environment: Ubuntu 24.04, g++ 13.3, x86_64.

1. Replaced `types.h` with `<cstdint>`-based typedefs, `__stdcall`/`__cdecl` no-ops.
2. `synth_core.cpp`: changed `vsprintf_s(buf, fmt, arg)` → `vsnprintf(...)`. Compiles clean.
3. `v2mplayer.cpp`: replaced 4 `__asm` blocks (64-bit mul/div tick math ×3, zero-fill ×1)
   with standard C++; removed duplicated typedefs from `v2mplayer.h`.
4. `ronan.cpp`: replaced `sFtol`/`sFPow`/`sFExp` x87 asm with `lrintf`/`pow`/`exp`.
   (File is Latin-1 encoded — mind the tooling.)
5. Rendered 20 s of `pzero_new.v2m` → RMS 0.083, peak 1.02; `v2_zeitmaschine_new.v2m` →
   RMS 0.069, peak 0.96. Audible, correct program material in both, with `RONAN`
   both off and on.
