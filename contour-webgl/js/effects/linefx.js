// jace/linefx — the expanding ring emitter.
//
// The 128-byte block (constructor 0x41f4e0 stores it at this+0xb4) is
// eight triples plus a colour, an envelope, and four emitter fields.
// Each triple is (base, rate, drift): the spawn routine (0x41f920)
// writes `base + drift × T` into the particle when it is born — T being
// the effect's own clock, so successive rings differ — and the update
// (0x41f5c0) evaluates `spawned + rate × age` every frame.
//
//   +0x00 A centre x   +0x0c B centre y   +0x18 C half-width (× 0.001)
//   +0x24 colour 0xRRGGBB
//   +0x28 D brightness +0x34 E start angle (turns) +0x40 F end angle
//   +0x4c G inner radius   +0x58 H outer radius
//   +0x64 attack  +0x68 sustain  +0x6c release   (their sum is the life)
//   +0x70 spawn interval  +0x74 emit until  +0x78 max alive  +0x7c segments
//
// The init (0x41f530) clamps the segment count to [1, 70] — anything
// outside becomes 70 — and allocates 3 vertices and 4 indices per
// segment. The update walks u = i/(segments−1) and writes, per step,
//
//   ang = 2π·(E + (F − E)u)     r = G + (H − G)u
//   inner  = (cos ang·(r − C), sin ang·(r − C)·1.25) + (A, B)   black
//   middle = (cos ang· r     , sin ang· r     ·1.25) + (A, B)   colour
//   outer  = (cos ang·(r + C), sin ang·(r + C)·1.25) + (A, B)   black
//
// — a ribbon along an arc, lit down its centreline. The coordinates are
// normalised screen space with y downward and 1.25 correcting the 4:3
// frame; the release capture puts instance 501's five-segment ring at
// (0.38, 0.35) upper-left and 500's fifty-segment one at (0.82, 0.85)
// lower-right, which is what fixes the sign of y.
//
// The brightness triple is scaled by an attack/sustain/release envelope
// and clamped to [0, 1] before it multiplies the colour.
//
// Reconstructed: only the blend mode (additive, as the ctor's mode-1
// field and the white-on-dark capture both imply) and the strip's
// triangulation.

const clamp01 = (v) => (v < 0 ? 0 : v > 1 ? 1 : v);
const TAU = Math.PI * 2;
const YSCALE = 1.25;    // the constant at 0x435718
const WSCALE = 0.001;   // the constant at 0x435720

export class Linefx {
  constructor(mgl, tex, params) {
    this.mgl = mgl;
    this.p = params;
    this.life = params.attack + params.sustain + params.release;
    // emission stops at `emitUntil`; the last ring born then still runs
    this.duration = params.emitUntil + this.life;
  }

  // Rebuild the live particle set for effect-local time T. Emission is
  // deterministic (no randomness anywhere in the block), so this can be
  // evaluated from scratch at any point — which keeps seeking honest.
  _alive(T) {
    const p = this.p;
    const out = [];
    const step = p.interval > 1e-3 ? p.interval : 1e-3;
    for (let born = 0; born <= p.emitUntil && born <= T; born += step) {
      const age = T - born;
      if (age > this.life) continue;
      // the emitter refuses to spawn while it is already at capacity
      if (out.length >= p.maxAlive) continue;
      out.push({ born, age });
    }
    return out;
  }

  do(time, timeStart) {
    const mgl = this.mgl;
    const p = this.p;
    const T = time - timeStart;
    if (T < 0) return;
    const live = this._alive(T);
    if (!live.length) return;

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.ortho(0, 1, 1, 0, -1, 1);      // normalised screen, y downward
    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    mgl.enableTexture(false);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    mgl.enableCullFace(false);
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);

    const cr = ((p.color >> 16) & 255) / 255;
    const cg = ((p.color >> 8) & 255) / 255;
    const cb = (p.color & 255) / 255;
    const segs = p.segments;

    mgl.begin(mgl.TRIANGLES);
    for (const q of live) {
      // (base + drift × birthTime) + rate × age
      const ev = (tr) => tr[0] + tr[2] * q.born + tr[1] * q.age;

      let e = ev(p.bright);
      if (q.age < p.attack) e *= q.age / p.attack;
      if (p.attack + p.sustain < q.age) {
        e *= 1 - (q.age - p.attack - p.sustain) / (p.release || 1e-4);
      }
      e = clamp01(e);
      if (e <= 0.003) continue;

      const ax = ev(p.cx), ay = ev(p.cy);
      const hw = ev(p.width) * WSCALE;
      const a0 = ev(p.angle0) * TAU, a1 = ev(p.angle1) * TAU;
      const r0 = ev(p.radius0), r1 = ev(p.radius1);
      const [r, g, b] = [cr * e, cg * e, cb * e];

      let prev = null;
      for (let i = 0; i < segs; i++) {
        const u = segs > 1 ? i / (segs - 1) : 0;
        const ang = a0 + (a1 - a0) * u;
        const rr = r0 + (r1 - r0) * u;
        const c = Math.cos(ang), s = Math.sin(ang) * YSCALE;
        const pt = [
          [ax + c * (rr - hw), ay + s * (rr - hw)],
          [ax + c * rr, ay + s * rr],
          [ax + c * (rr + hw), ay + s * (rr + hw)],
        ];
        if (prev) {
          for (let k = 0; k < 2; k++) {
            // edge k..k+1 of the ribbon: dark → bright → dark
            const la = k === 0 ? 0 : 1, lb = k === 0 ? 1 : 0;
            const A = prev[k], B = prev[k + 1], C = pt[k], D = pt[k + 1];
            mgl.color4(r, g, b, la); mgl.vertex3(A[0], A[1], 0);
            mgl.color4(r, g, b, lb); mgl.vertex3(B[0], B[1], 0);
            mgl.color4(r, g, b, la); mgl.vertex3(C[0], C[1], 0);
            mgl.color4(r, g, b, la); mgl.vertex3(C[0], C[1], 0);
            mgl.color4(r, g, b, lb); mgl.vertex3(B[0], B[1], 0);
            mgl.color4(r, g, b, lb); mgl.vertex3(D[0], D[1], 0);
          }
        }
        prev = pt;
      }
    }
    mgl.end();
  }
}
