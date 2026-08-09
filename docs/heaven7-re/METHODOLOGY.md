# Evidence discipline for this reverse-engineering work

This document exists because claims in `README.md` were repeatedly stated with
more confidence than their evidence supported (see the retraction log at the
end). The fix is not "be more careful" — it is to attach a **provenance class**
to every claim, and to make the mechanically checkable ones actually checked by
a script.

## 1. Provenance classes

Every factual claim in this directory carries one of these. The class states
*how we know*, not how sure we feel.

| Class | Meaning | Re-verifiable by |
|---|---|---|
| **`BYTES`** | Read directly out of the binary: an address, a table entry, a float constant, a specific instruction encoding. | `tools/verify_claims.py` — automatically, every time |
| **`DISASM`** | Derived from the *semantics* of a named instruction range: what a routine computes. | A human re-reading the cited range in `texture-ops.txt` / `disasm-key-routines.txt` |
| **`TRACE`** | Observed while emulating, with the run's parameters and **stop reason** recorded. | Re-running the cited tool with the cited budget |
| **`CROSS`** | Confirmed by **two independent measures**, at least one of which does not depend on having found the thing being counted. | Re-running both measures |
| **`INFER`** | A hypothesis from pattern, naming, analogy or plausibility. **Not evidence.** May be useful; may be wrong. | Nothing — must be promoted to another class or dropped |

Rules:

- `INFER` claims must be written as hypotheses ("reads as", "likely"), never as
  fact, and must never be summarised as established in a report.
- A claim's class may only be raised by doing the corresponding work, not by
  repetition. Restating an `INFER` claim in a later section does not make it
  `DISASM`.
- Naming something is not knowing it. Calling a routine "the texture generator"
  is `INFER` until its instructions are read (`DISASM`).

## 2. The completeness rule

This is where the real failures happened. A claim of the form *"there are N of
X"*, *"all of X"*, or *"X spans A..B"* requires **all three**:

1. **A terminating run.** The emulation must stop for a *known* reason (program
   exit, script terminator reached, interpreter returned), not because it hit an
   instruction budget or a wall-clock timeout.
2. **An independent counter.** A measure that does not depend on having located
   the items. Counting texture *allocations* is independent of having found the
   texture *programs*; counting programs you happened to trace is not.
3. **The stop reason recorded** alongside the number, in the document.

**Never derive an extent from a truncated run.** The furthest address a bounded
run reached is a lower bound on the data's extent and nothing more. Both major
errors here came from treating such a value as the end of the data — and the
second error came from *reasoning further* from that false bound.

## 3. Running the checker

```bash
pip install unicorn pefile capstone
upx -d heaven7w.exe -o h7w_unpacked.exe
H7_EXE=h7w_unpacked.exe python3 tools/verify_claims.py claims.yaml
```

Every `BYTES` claim in [`claims.yaml`](claims.yaml) is re-read from the binary
and reported `PASS`/`FAIL`. This turns the factual base of this directory into
something that can be regression-tested rather than trusted. `DISASM`, `TRACE`,
`CROSS` and `INFER` claims are listed by the tool as **unchecked**, with their
locators, so the unverified surface is always visible rather than implied.

The binary itself is not redistributed here, so the checker is only meaningful
against your own `upx -d` copy of the original release.

## 4. Retraction log

Kept deliberately, so the failure modes stay visible:

| Claim | Status | Why it was wrong |
|---|---|---|
| "The scene script is depacked into a heap buffer" | **retracted** | Guessed from the presence of an allocator wrapper; emulation showed the script is cleartext in `.data`. |
| "`.text:0x40cb14` is the offline texture generator" | **retracted** | `INFER` from proximity to MMX code. Reading it (`DISASM`) showed a teardown routine that frees the context. |
| "`.data` code is JIT-emitted at runtime" | **retracted** | Inferred from 1.5 M writes into the code band. The writes target two fixed *variables* living among the code. |
| "Scene script is 201 opcodes, ending at `0x41b6f4`" | **retracted** | Both figures came from a run that stopped on its instruction budget. True: 970 opcodes to `0x4291d5`. |
| "`scene-script.txt` is only the opening of the timeline" | **retracted** | Correct conclusion, wrong reasoning — then reversed on the basis of the same truncated run's bound. |
| "3 texture programs, 7 of 26 operators used" | **retracted** | Generalised from one bounded trace. True: 11 programs, 23 operators, found only after an independent allocator count contradicted it. |
| "The pruning cuts the port from 26 operators to 7" | **retracted** | Followed from the above; the port needs the near-complete operator set. |
| "Operator `0x11` is a multi-octave/turbulence variant" | **retracted** | `INFER` from its argument count. Reading it showed plasma / diamond-square. |

The pattern in six of these eight: a plausible reading was stated as fact
without opening the relevant bytes, or a bounded observation was treated as a
bound.
