// Dynamic scene-script + texture extractor for Heaven 7 (Exceed, 2000).
//
// Static analysis recovered the stream *format* and the parser *architecture*
// (see ../README.md), but the scene script is depacked into a heap buffer and
// walked by a pointer-driven interpreter, so the cleanest way to get the actual
// decoded values is to observe the running program. This Frida script does that.
//
// Usage (on Windows, or Linux+Wine with frida):
//   1. upx -d heaven7w.exe            (unpack; addresses below assume the
//                                      unpacked image based at 0x00400000)
//   2. frida -f heaven7w.exe -l extract_scene.js --no-pause
//   3. let it run through the intro; scene_dump.json / *.tex are written to CWD.
//
// If the loaded image base differs (ASLR), everything is rebased off Module.base.

'use strict';

const IMG = Process.enumerateModules()[0];
const base = IMG.base;
const OFF = (rva) => base.add(rva - 0x00400000);

// --- decoder call sites (see README: read_varint @ .text:0x4086b8,
//     read_float @ .text:0x4086c8). We hook the object-record parser
//     (0x406a39) to capture one fully-typed record layout, and the two
//     decoders to log the raw byte cursor + decoded value in stream order.
const A_VARINT = OFF(0x4086b8);
const A_FLOAT  = OFF(0x4086c8);
const A_OBJPARSE = OFF(0x406a39);

const log = [];
function edi() { return this.context.edi; }   // stream cursor lives in EDI

Interceptor.attach(A_VARINT, {
  onEnter(a) { this.p = this.context.edi; },
  onLeave(r) { log.push({ t: 'varint', p: this.p.toString(), v: r.toInt32() }); },
});
Interceptor.attach(A_FLOAT, {
  onEnter(a) { this.p = this.context.edi; },
  onLeave(r) {
    // return value is an IEEE-754 bit pattern in EAX
    const f = new Float32Array(new Uint32Array([r.toUInt32()]).buffer)[0];
    log.push({ t: 'float', p: this.p.toString(), v: f });
  },
});

// One-shot capture of a full object record (ESI = object base on entry).
let objCaptured = false;
Interceptor.attach(A_OBJPARSE, {
  onEnter() {
    if (objCaptured) return;
    objCaptured = true;
    const esi = this.context.esi;
    // Dump 0x200 bytes of the object struct once populated; re-read on return.
    this.esi = esi;
  },
  onLeave() {
    if (!this.esi) return;
    const bytes = Memory.readByteArray(this.esi, 0x200);
    const f = new File('object_record.bin', 'wb');
    f.write(bytes); f.close();
    this.esi = null;
  },
});

// Flush the token stream periodically and on exit.
function flush() {
  const f = new File('scene_dump.json', 'w');
  f.write(JSON.stringify(log, null, 0));
  f.close();
}
setInterval(flush, 2000);

// --- texture buffers -------------------------------------------------------
// The shade-tree evaluator (.text:0x4023b7) samples textures via a base
// pointer at node+0x04. Hooking it lets us collect every distinct texture
// base it touches and dump the buffers as raw RGBA / signed-16 height maps.
const A_SHADE = OFF(0x4023b7);
const seenTex = new Set();
Interceptor.attach(A_SHADE, {
  onEnter() {
    const node = this.context.esi;
    const texBase = Memory.readU32(node.add(0x04));
    if (texBase && !seenTex.has(texBase)) {
      seenTex.add(texBase);
      // 256x256 RGBA is the common size (uv masks 0x3fc00 / 0x3fc imply a
      // 1024-byte stride => 256 px * 4 bytes). Dump a generous window.
      try {
        const bytes = Memory.readByteArray(ptr(texBase), 256 * 1024);
        const f = new File('tex_' + texBase.toString(16) + '.rgba', 'wb');
        f.write(bytes); f.close();
      } catch (e) {}
    }
  },
});

console.log('[heaven7] hooks installed; run the intro, then check CWD.');
