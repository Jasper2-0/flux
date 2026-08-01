// xotrack's `show*` tasks, which live in Real.exe rather than in the
// engine DLLs. Real.exe is 64 KB and registers eight of them against
// Red's task table, each with its own vtable: slot 0 is the one-time
// init, slot 1 the per-frame run.
//
// They take their clock from the message's task-local elapsed
// milliseconds — the same `[msg+0x48]` that drives drawScene — and each
// declares a `kind` when it registers, which is what decides whether it
// draws in the 2D pixel frame or the standing 3D one. See the note on
// red_base::run_tasks in demo.js.

import { lookAt } from './scene.js';

const SCREEN_W = 640, SCREEN_H = 480;

// The effects seed themselves from srand(time(NULL)), so the exact line
// placement was never reproducible between runs of the original either.
// What has to match is the distribution, and the shape of the expression
// each value goes through: `(rand() * K) >> 15`, i.e. uniform over [0, K).
function rnd(k) {
  return Math.floor(Math.random() * k);
}

// showLines — 0x402c60 (init) and 0x402e80 (run).
//
// Two modes, chosen by the first parameter. Both draw additively with
// texturing off, which is why they read as glow rather than as geometry.
//
//   "0"  sixteen full-width bands, each riding a slow sine:
//          y = sin((t + jitter) * freq) * amp + centre
//        with its own grey level and a height of five to nine pixels.
//
//   "1"  six bright lines that jump to new random parameters at the top
//        of every 172 ms — the second parameter picks vertical (x from
//        a 320-pixel wrap) or horizontal (y through fmod 240).
export class ShowLines {
  static kind = 1;   // registered at 0x4046f1 — the 2D pixel frame

  constructor(inst) {
    this.mode = String(inst.args[0] || '0');
    this.sub = String(inst.args[1] || '-');
    this.bands = [];
    for (let i = 0; i < 16; i++) {
      this.bands.push({
        amp: rnd(4000) * 0.1,
        centre: rnd(1000) - 400,
        grey: rnd(64) / 255 + 0.25,
        freq: rnd(5000) * 5e-7,
        height: rnd(5) + 5,
        jitter: rnd(100),
      });
    }
    this.bars = [];
    this.phase = 0;
    this._reseed();
  }

  _reseed() {
    this.phase = rnd(100);
    this.bars = [];
    for (let i = 0; i < 6; i++) {
      this.bars.push({
        amp: rnd(4000) * 0.1,
        offset: Math.trunc(rnd(500) - 400),
        freq: rnd(5000) * 5e-7,
      });
    }
  }

  draw(mgl, ms) {
    const gl = mgl.gl;
    mgl.enableTexture(false);
    mgl.blendFunc(gl.ONE, gl.ONE);
    mgl.enableBlend(true);

    if (this.mode === '0') {
      mgl.begin(mgl.TRIANGLES);
      for (const b of this.bands) {
        const y = Math.sin((ms + b.jitter) * b.freq) * b.amp + b.centre;
        const c = b.grey;
        const quad = [[0, y], [SCREEN_W - 1, y],
                      [SCREEN_W - 1, y + b.height], [0, y + b.height]];
        for (const i of [0, 1, 2, 0, 2, 3]) {
          mgl.color4(c, c, c, 1);
          mgl.vertex3(quad[i][0], quad[i][1], 0);
        }
      }
      mgl.end();
      return;
    }

    // mode 1: new parameters at the top of every 172 ms window
    if (ms % 172 < 100) this._reseed();
    mgl.begin(mgl.LINES);
    for (const b of this.bars) {
      const d = Math.sin((ms + this.phase) * b.freq) * b.amp;
      mgl.color4(1, 1, 1, 1);
      if (this.sub === '0') {
        const x = d + (b.offset % 320) + 1;
        mgl.vertex3(x, 0, 0);
        mgl.vertex3(x, SCREEN_H - 1, 0);
      } else {
        let y = (d + b.offset) % 240;
        mgl.vertex3(0, y, 0);
        mgl.vertex3(SCREEN_W - 1, y, 0);
      }
    }
    mgl.end();
  }
}

// showDraai — 0x4029f0 (init) and 0x402a30 (run). "Draai" is Dutch for
// spin, and that is the whole effect: ten screen-filling additive quads
// of envmap.jpg, stacked at slightly different depths and rotated about
// the view axis by an angle beaten out of three sines.
//
// It is a kind-2 task, so it draws in whatever 3D frame is standing. In
// its section — part 8, alongside showTunnel — that is the projection
// gears.i3d's camera left behind two sections earlier, because nothing
// between them restores one. It never sets a projection of its own.
//
// The odd part is the feedback: the rotation angle is written to the same
// stack slot the elapsed time came in on, and the depth and half-size for
// the quad are then derived from *the angle*, not from the clock.
export class ShowDraai {
  static kind = 2;

  draw(mgl, ms, p, fxTex) {
    const gl = mgl.gl;
    const tex = fxTex && fxTex.get('envmap.png');
    mgl.enableTexture(!!tex);
    if (tex) mgl.bindTexture(tex);
    mgl.enableLighting(false);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    mgl.enableCullFace(false);
    mgl.blendFunc(gl.ONE, gl.ONE);
    mgl.enableBlend(true);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.pushMatrix();
    mgl.loadIdentity();
    mgl.translate(0, 0, -5);

    const a = ms * 0.00029754957395;
    const b = ms * 0.0004054359374;
    const c = ms * 0.0006432759545;
    for (let i = 0; i < 10; i++) {
      const q = i * 0.1;
      const angle = Math.sin(a + q) * Math.sin(b + q) * Math.sin(c + q) * 70;
      // no push/pop inside the loop, so the rotations accumulate
      mgl.rotate(angle, 0, 0, 1);
      const z = Math.sin(angle * 0.000633543 + i) * 0.5 - 1.0;
      const h = Math.sin(angle * 0.00123479375);
      const lo = h - 7, hi = h + 7;
      const quad = [[lo, lo, 0, 0], [hi, lo, 1, 0], [hi, hi, 1, 1], [lo, hi, 0, 1]];
      mgl.color4(0.16, 0.16, 0.16, 1);
      mgl.begin(mgl.TRIANGLES);
      for (const k of [0, 1, 2, 0, 2, 3]) {
        const [x, y, u, v] = quad[k];
        mgl.color4(0.16, 0.16, 0.16, 1);
        mgl.texCoord2(u, v);
        mgl.vertex3(x, y, z);
      }
      mgl.end();
    }
    mgl.popMatrix();
    mgl.enableTexture(false);
    mgl.enableBlend(false);
  }
}

// showTunnel — 0x4039e0 (init) and 0x403e60 (run).
//
// A 13 x 128 tube, warped by a product of three sines, flown through by a
// camera that rides the same warp. The one texture is eenv2.tga.
//
// The displacement is applied *twice*. `init` bakes a static version of it
// into the vertex array at 0x403d03, and then `run` computes the animated
// version at 0x404104 reading those same already-warped vertices — the
// buffer at +0x74 is both the source and the target of the init pass, and
// the run pass never writes back. Almost certainly the init copy is a
// leftover from before the effect was animated, but it is what shipped, so
// both passes are here.
const TAU = 6.283185307;

// The three frequencies the warp mixes, shared by init and run.
const WA = 0.17532754, WB = 0.3437543, WC = 0.2437543;

// [esi+0x58] and [esi+0x5c]: the tube's mean radius and how much the
// ring-to-ring cosine opens and closes it.
const R_MEAN = 1.100000023841858, R_SWING = 0.4000000059604645;

const RINGS = 128;      // [esi+0x48]
const SEG = 12;         // [esi+0x40] — segments around
const PER_RING = 13;    // [esi+0x44] — the closing vertex repeats the first
const PERIOD = 30.0;    // [esi+0x3c]

export class ShowTunnel {
  static kind = 2;

  constructor() {
    // The ring, straight from the init loop at 0x403b1f. The closing
    // vertex takes its angle from [0x40c3b0], which is 0.0 — so it lands
    // exactly on top of vertex 0 and seals the tube.
    const n = RINGS * PER_RING;
    const pos = new Float32Array(n * 3);
    const uv = new Float32Array(n * 2);
    for (let r = 0; r < RINGS; r++) {
      for (let i = 0; i < PER_RING; i++) {
        const ang = i === SEG ? 0 : (i * TAU) / SEG;
        const k = (r * PER_RING + i) * 3;
        pos[k] = Math.cos(ang);
        pos[k + 1] = Math.sin(ang);
        pos[k + 2] = -r;
        const u = (r * PER_RING + i) * 2;
        uv[u] = (i * 2) / SEG;          // two texture repeats around
        uv[u + 1] = (r * 20) / RINGS;   // twenty along
      }
    }
    // The static half of the warp (0x403d03), in place.
    for (let v = 0; v < n; v++) {
      const k = v * 3, z = pos[k + 2];
      const R = R_MEAN + R_SWING * Math.cos((z * TAU) / ((TAU * RINGS) / PERIOD));
      const a = z * WA, b = z * WB, c = z * WC;
      pos[k] = pos[k] * R + 3 * Math.cos(a) * Math.cos(b) * Math.cos(c);
      pos[k + 1] = pos[k + 1] * R + 3 * Math.sin(a) * Math.sin(b) * Math.sin(c);
    }
    this.pos = pos;
    this.uv = uv;

    const idx = new Uint32Array((RINGS - 1) * SEG * 6);
    let o = 0;
    for (let r = 0; r < RINGS - 1; r++) {
      for (let i = 0; i < SEG; i++) {
        const a = r * PER_RING + i, b = (r + 1) * PER_RING + i;
        idx[o++] = a;     idx[o++] = a + 1; idx[o++] = b;
        idx[o++] = a + 1; idx[o++] = b + 1; idx[o++] = b;
      }
    }
    this.idx = idx;
    this.out = new Float32Array(idx.length * 3);
    this.outUV = new Float32Array(idx.length * 2);
  }

  draw(mgl, ms, p, fxTex) {
    const gl = mgl.gl;
    const tex = fxTex && fxTex.get('eenv2.png');

    // D is where the camera sits along the tube, W how far it has rolled.
    const D = 50 * Math.sin(ms * 5.265399886411615e-05) - 64;
    const W = Math.PI * Math.sin(ms * 3.234000178053975e-05) *
                        Math.sin(ms * 6.34500029264018e-05) *
                        Math.sin(ms * 8.556000102544203e-05) + 0.0006 * ms;

    // The warp evaluated at a depth, which is how both the eye and the
    // look-at point are placed on the tube's own centre curve.
    const cosMix = (z) => Math.cos(z * WA) * Math.cos(z * WB) * Math.cos(z * WC);
    const sinMix = (z) => Math.sin(z * WA) * Math.sin(z * WB) * Math.sin(z * WC);

    // [esi+0x34] is set to 1 in init and never written again, so the
    // effect always takes this branch: the eye sits ten units out from the
    // curve rather than exactly on it, which is the difference between
    // looking at the tunnel and flying down it. The on-the-curve variant
    // at 0x403f7b is dead code.
    const eye = [10 * (cosMix(D) + 1), 10 * (sinMix(D) + 1), D];
    const E = D + 2.0;
    const at = [cosMix(E), sinMix(E), E];

    mgl.matrixMode(mgl.PROJECTION);
    mgl.pushMatrix();
    mgl.loadIdentity();
    // gluPerspective(45, 4/3, 0.1, 100) — its own, unlike every other
    // kind-2 effect, and pushed so the frame it borrowed comes back.
    const half = Math.tan(45 * Math.PI / 360) * 0.1;
    mgl.frustum(-half * (4 / 3), half * (4 / 3), -half, half, 0.1, 100);
    mgl.matrixMode(mgl.MODELVIEW);
    mgl.pushMatrix();
    mgl.loadMatrix(lookAt(eye, at, [Math.cos(W), Math.sin(W), 0]));

    mgl.enableTexture(!!tex);
    if (tex) mgl.bindTexture(tex);
    mgl.enableLighting(false);
    mgl.enableCullFace(false);
    mgl.enableBlend(false);
    mgl.enableDepthTest(true);
    mgl.depthMask(true);
    mgl.color4(1, 1, 1, 1);

    // The animated half of the warp, per vertex per frame. The phase the
    // clock adds differs between the two terms — 0x40c518 for the cosine,
    // 0x40c510 for the sine — which is what stops the tube from simply
    // sliding and makes it churn.
    const pa = ms * 0.004754345, pb = ms * 0.002355343;
    const uw = ms * 0.00459373;
    const src = this.pos, suv = this.uv, idx = this.idx;
    const out = this.out, outUV = this.outUV;
    for (let e = 0; e < idx.length; e++) {
      const v = idx[e], k = v * 3;
      const x = src[k], y = src[k + 1], z = src[k + 2];
      const R = R_MEAN + R_SWING * Math.cos((z * TAU) / ((TAU * RINGS) / PERIOD));
      const a = z * WA, b = z * WB, c = z * WC;
      const o3 = e * 3;
      out[o3] = x * R + 3 * Math.cos(a + pa) * Math.cos(b) * Math.cos(c);
      out[o3 + 1] = y * R + 3 * Math.sin(a + pb) * Math.sin(b) * Math.sin(c);
      out[o3 + 2] = z;
      const o2 = e * 2;
      // u wobbles with the vertex's own x — 0x4043b8
      outUV[o2] = suv[v * 2] + 0.1 * Math.sin(uw + x);
      outUV[o2 + 1] = suv[v * 2 + 1];
    }
    mgl.drawArraysTri(out, outUV);

    mgl.popMatrix();
    mgl.matrixMode(mgl.PROJECTION);
    mgl.popMatrix();
    mgl.matrixMode(mgl.MODELVIEW);
    mgl.enableTexture(false);
  }
}

// Named in Real.exe's string table and loaded by the effects themselves,
// so they never pass through the script's `loadImage`.
export const EFFECT_TEXTURES = ['envmap.png', 'eenv2.png'];

export const EFFECTS = {
  showlines: ShowLines,
  showdraai: ShowDraai,
  showtunnel: ShowTunnel,
};
