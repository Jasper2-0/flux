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
// showBol left behind in part 7: showBol calls draw_scene, which reaches
// the camera's calculate, which never restores it. showDraai itself never
// sets a projection.
//
// The odd part is the feedback: the rotation angle is written to the same
// stack slot the elapsed time came in on, and the depth and half-size for
// the quad are then derived from *the angle*, not from the clock.
export class ShowDraai {
  static kind = 2;

  draw(mgl, ms, p, demo) {
    const gl = mgl.gl;
    const tex = demo.fxTex && demo.fxTex.get('envmap.png');
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

  draw(mgl, ms, p, demo) {
    const gl = mgl.gl;
    const tex = demo.fxTex && demo.fxTex.get('eenv2.png');

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

// showBol — 0x401d70 (init) and 0x4021b0 (run).
//
// The one effect that hands its geometry back to the engine. Its init
// loads sphere.i3d, keeps a pristine `memcpy` of the vertex array, and
// then each frame it writes a deformed copy into the scene's live array,
// recomputes the normals (0x401e80), moves the camera and calls
// `energy3d_scene::draw_scene` (0x402342) — the same path any `drawScene`
// layer takes. So this renders as a scene, not as immediate-mode geometry.
//
// That also means it is the one effect that *sets* the standing 3D frame
// rather than borrowing it: draw_scene reaches the camera's `calculate`
// through the vtable at 0x10011885, and calculate never restores the
// projection. Part 8's showDraai inherits this camera, not gears'.
//
// The deformation reads x, y and z from the pristine copy and perturbs
// each axis by a product of two sines of the *other two* axes summed —
// so the sphere kneads itself rather than simply breathing.
export class ShowBol {
  static kind = 2;
  static scene = 'sphere.i3d';

  draw(mgl, ms, p, demo) {
    const scene = demo.scenes.get('sphere.i3d');
    if (!scene) return;
    const mesh = scene.meshes[0];
    if (!mesh) return;
    const base = mesh.base, out = mesh.positions;

    // The four phase terms the loop keeps on the FPU stack for its whole
    // run — 0x40c390, 0x40c398, 0x40c3a0, 0x40c3a8 in stack order.
    const T0 = ms * 0.003293609752;
    const T1 = ms * 0.0002854109754;
    const T2 = ms * 0.004097526959;
    const T3 = ms * 0.002854109754;

    for (let i = 0; i < base.length; i += 3) {
      const x = base[i], y = base[i + 1], z = base[i + 2];
      const s = z + y, u = z + x, v = y + x;
      out[i] = x + 25 * Math.sin(s * 0.044353453 + T3) * Math.sin(s * 0.019353453 + T1);
      out[i + 1] = y + 20 * Math.sin(u * 0.029052457 + T1) * Math.sin(u * 0.033245335 + T2);
      out[i + 2] = z + 21 * Math.sin(v * 0.035754676 + T0) * Math.sin(v * 0.025563549 + T1);
    }
    // 0x401e80 rebuilds the vertex normals from the deformed faces, by
    // accumulating each face's cross product onto its three corners. The
    // scene has no lights, so nothing lights them — but the material
    // carries a reflection map (eeeeeeeenv.tga), and GL_SPHERE_MAP is
    // computed from the normal, so a stale normal freezes the chrome onto
    // the rest pose instead of letting it slide over the deformation.
    const nrm = mesh.normals, idx = mesh.indices;
    nrm.fill(0);
    for (let f = 0; f < idx.length; f += 3) {
      const a = idx[f] * 3, b = idx[f + 1] * 3, c = idx[f + 2] * 3;
      const ux = out[b] - out[a], uy = out[b + 1] - out[a + 1], uz = out[b + 2] - out[a + 2];
      const vx = out[c] - out[a], vy = out[c + 1] - out[a + 1], vz = out[c + 2] - out[a + 2];
      const nx = uy * vz - uz * vy, ny = uz * vx - ux * vz, nz = ux * vy - uy * vx;
      nrm[a] += nx; nrm[a + 1] += ny; nrm[a + 2] += nz;
      nrm[b] += nx; nrm[b + 1] += ny; nrm[b + 2] += nz;
      nrm[c] += nx; nrm[c + 1] += ny; nrm[c + 2] += nz;
    }
    for (let i = 0; i < nrm.length; i += 3) {
      const l = Math.hypot(nrm[i], nrm[i + 1], nrm[i + 2]) || 1;
      nrm[i] /= l; nrm[i + 1] /= l; nrm[i + 2] /= l;
    }

    // energy3d_camera::set_position, the only virtual three-float setter
    // on the class, reached through vtable slot 2 at 0x402335. The target
    // is left at sphere.i3d's own, which is the origin.
    const cam = {
      eye: [
        50 * Math.sin(ms * 0.001357465396) + 100,
        50 * Math.sin(ms * 0.002357465396) + 100,
        100 * Math.sin(ms * 0.0007249999907799065 * 6.283185307 + 1.57079632675) + 160,
      ],
      at: scene.cameras[0].target,
      fov: scene.cameras[0].fov,
    };
    demo.renderScene(scene, cam, 0, 1);
  }
}

// showPartiekels — 0x403150 (init) and 0x403220 (run).
//
// The other sphere.i3d effect, and the opposite of showBol: it never hands
// the scene back to the engine, it walks the 1002 vertices itself and puts
// a 1x1 additive quad of particle.jpg at each one. Coat's pole-free
// geosphere is doing real work here — on a UV sphere the particles would
// pile up at the poles and stripe along the seam.
//
// Where showBol *offsets* each axis, this one *scales* it: the vertex is
// multiplied by a product of two sines rather than displaced by one, so
// the cloud collapses through the origin and blooms out again instead of
// wobbling around a fixed radius.
//
// The quads are axis-aligned, not billboarded — the same 1x1 square in
// object space at every vertex, which under the spin below shears them.
export class ShowPartiekels {
  static kind = 2;
  static scene = 'sphere.i3d';

  draw(mgl, ms, p, demo) {
    const gl = mgl.gl;
    const scene = demo.scenes.get('sphere.i3d');
    const tex = demo.fxTex && demo.fxTex.get('particle.png');
    if (!scene) return;
    const mesh = scene.meshes[0];
    if (!mesh) return;
    const base = mesh.base, out = mesh.positions;

    // Four phases and two envelopes, all hoisted out of the vertex loop —
    // the envelopes ride 0.4 * sin + 0.6, so they never quite reach zero.
    const P1 = ms * 0.001854109754;
    const P2 = ms * 0.002097526959;
    const P3 = ms * 0.0002854109754;
    const P4 = ms * 0.001293609752;
    const S1 = Math.sin(ms * 0.0010363) * 0.4 + 0.6;
    const S2 = Math.sin(ms * 0.0006363) * 0.4 + 0.6;

    for (let i = 0; i < base.length; i += 3) {
      const x = base[i], y = base[i + 1], z = base[i + 2];
      const w = z + y, u = z + x, v = y + x;
      out[i] = x * 2 * S1 * Math.sin(w * 0.084353453 + P1) * Math.sin(w * 0.019353453 + P3);
      out[i + 1] = y * 3 * S1 * Math.sin(u * 0.053245335 + P2) * Math.sin(u * 0.029052457 + P3);
      out[i + 2] = z * 4 * S2 * Math.sin(v * 0.025563549 + P3) * Math.sin(v * 0.065754676 + P4);
    }

    mgl.matrixMode(mgl.PROJECTION);
    mgl.pushMatrix();
    mgl.loadIdentity();
    // gluPerspective(45, 4/3, 0.1, 5000). The original only reloads the
    // projection and pops it at the end without ever pushing — it gets
    // away with it because the stack is deeper than one. Pushed properly
    // here.
    const half = Math.tan(45 * Math.PI / 360) * 0.1;
    mgl.frustum(-half * (4 / 3), half * (4 / 3), -half, half, 0.1, 5000);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.pushMatrix();
    mgl.loadIdentity();
    mgl.rotate(Math.sin(ms * 0.0002304234) * 360, 0, 0, 1);
    mgl.translate(Math.sin(ms * 0.00023534) * 30, Math.sin(ms * 0.00032453) * 30, -200);

    mgl.enableCullFace(false);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    mgl.blendFunc(gl.ONE, gl.ONE);
    mgl.enableBlend(true);
    mgl.enableLighting(false);
    mgl.enableTexture(!!tex);
    if (tex) mgl.bindTexture(tex);

    // glColor3ub straight off the message — this is the one effect that
    // reads the script's `color` property (0x40347d..0x403489).
    const [r, g, b] = p.color;
    mgl.color4(r / 255, g / 255, b / 255, 1);

    mgl.begin(mgl.TRIANGLES);
    for (let i = 0; i < out.length; i += 3) {
      const x = out[i], y = out[i + 1], z = out[i + 2];
      const q = [[x, y, 0, 0], [x + 1, y, 1, 0], [x + 1, y + 1, 1, 1], [x, y + 1, 0, 1]];
      for (const k of [0, 1, 2, 0, 2, 3]) {
        mgl.color4(r / 255, g / 255, b / 255, 1);
        mgl.texCoord2(q[k][2], q[k][3]);
        mgl.vertex3(q[k][0], q[k][1], z);
      }
    }
    mgl.end();

    mgl.popMatrix();
    mgl.matrixMode(mgl.PROJECTION);
    mgl.popMatrix();
    mgl.matrixMode(mgl.MODELVIEW);
    mgl.enableTexture(false);
    mgl.enableBlend(false);
  }
}

// showPlanes — 0x403590 (init) and 0x4036d0 (run).
//
// Fifty 16x16 additive quads cycling towards the viewer down a twenty-unit
// loop, cycling through plane_01..04.jpg by index & 3. Each has a fixed
// scatter from init — `rand() % 400 - 200` scaled by 0.01, so a couple of
// units either way — plus a slow sine wobble five times that size, which
// is what keeps them from reading as a fixed grid.
//
// The brightness is `(1 - |z| / 10)` cubed, so a plane fades up out of
// nothing at the far end of the loop and back down as it passes. Cubing a
// linear ramp is what makes the fade feel like it has a threshold.
export class ShowPlanes {
  static kind = 2;

  constructor() {
    this.ax = [];
    this.by = [];
    for (let i = 0; i < 50; i++) {
      this.ax.push((rnd(400) - 200) * 0.01);
      this.by.push((rnd(400) - 200) * 0.01);
    }
  }

  draw(mgl, ms, p, demo) {
    const gl = mgl.gl;
    const tex = [1, 2, 3, 4].map((n) =>
      demo.fxTex && demo.fxTex.get('plane_0' + n + '.png'));

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.pushMatrix();
    mgl.loadIdentity();
    // One spin about the view axis, its angle a sine of a sine.
    const spin = Math.sin(ms * 0.000324774764) *
      Math.sin(Math.sin(ms * 0.0002346574675) * 3.0 + ms * 0.00043675) * 360.0;
    mgl.rotate(spin, 0, 0, 1);
    mgl.multMatrix(lookAt([0, 0, 15], [0, 0, 0], [0, 1, 0]));

    mgl.enableTexture(true);
    mgl.enableCullFace(false);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    mgl.blendFunc(gl.ONE, gl.ONE);
    mgl.enableBlend(true);
    mgl.enableLighting(false);

    const dz = ms * 0.00243565;
    const wa = ms * 0.0002353, wb = ms * 0.0003236;
    for (let k = 0; k < 50; k++) {
      const t = tex[k & 3];
      if (t) mgl.bindTexture(t);
      // fmod against 20, then centred, so z runs -10 .. 10 and wraps
      let z = (4 * k * 0.4 + dz) % 20;
      if (z < 0) z += 20;
      z -= 10;
      const f = 1 - Math.abs(z) * 0.1;
      const c = f * f * f;
      const ax = Math.sin(k + wa) * 5 + this.ax[k];
      const by = Math.sin(k + wb) * 5 + this.by[k];
      const q = [[ax - 8, by - 8, 1, 0], [ax + 8, by - 8, 0, 0],
                 [ax + 8, by + 8, 0, 1], [ax - 8, by + 8, 1, 1]];
      mgl.begin(mgl.TRIANGLES);
      for (const i of [0, 1, 2, 0, 2, 3]) {
        mgl.color4(c, c, c, 1);
        mgl.texCoord2(q[i][2], q[i][3]);
        mgl.vertex3(q[i][0], q[i][1], z);
      }
      mgl.end();
    }

    mgl.popMatrix();
    mgl.enableTexture(false);
    mgl.enableBlend(false);
  }
}

// Named in Real.exe's string table and loaded by the effects themselves,
// so they never pass through the script's `loadImage`.
export const EFFECT_TEXTURES = [
  'envmap.png', 'eenv2.png', 'particle.png',
  'plane_01.png', 'plane_02.png', 'plane_03.png', 'plane_04.png',
];

export const EFFECTS = {
  showlines: ShowLines,
  showdraai: ShowDraai,
  showtunnel: ShowTunnel,
  showbol: ShowBol,
  showpartiekels: ShowPartiekels,
  showplanes: ShowPlanes,
};
