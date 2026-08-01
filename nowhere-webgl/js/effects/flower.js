// 33 — the flower / TV section (two instances, `33` and `332`).
//
// The plugin wants `Cylinder01`, `GeoSphere01`, `tv01`–`tv04` and
// `camera01` from the scene, and takes six documented parameters:
// `uitslag` ("maximale uitslag van de paal" — how far the pole swings),
// `center`, `bolscale` ("groote van de bol"), `bolturbdiv` and
// `bolanglediv` (both "hoe groter, deste minder heftig") and `movietime`
// ("divider standaard op 2 dus als je op 30.0f fps draait wordt het
// 30/2 = 15"). Defaults out of the constructor: bolscale 0.4,
// bolturbdiv 50, bolanglediv 60, movietime 2; the script overrides them
// to 1, 32 and 56.
//
// What is implemented here is the ball, decoded from 0x10001d20. It
// rewrites GeoSphere01's texture coordinates and then pushes every
// vertex out along its own direction by two moving circular bumps:
//
//     scale = sin(time / bolturbdiv) · 0.15 + 0.75
//     u = (x/70 + 0.5)·scale        v = (y/70 + 0.5)·scale
//
//     a = time / bolanglediv
//     bump(P, Q, gain):
//         p = wrap(P + u·255) − 128     q = wrap(Q + v·255) − 128
//         return max(0, 255 − hypot(p, q)·512/182) / 255 · gain
//     r = bolscale + bump(50a, −31.8835a, 0.5) + bump(−23.796a, 34.4485a, 0.6)
//     vertex ← source vertex · r
//
// where `wrap(t) = t − trunc(t) + (trunc(t) & 255)` — the binary's way
// of folding a float into [0, 256) with an `and eax, 0xff`.
//
// Not yet implemented: the eighteen poles the effect generates itself
// (0x10001fc0 builds 18 tubes of 41×3 vertices and 0x10001930 sways them
// with `center` and `uitslag`), and the movie playing on the four TV
// screens. Those parts of the scene render straight for now.
//
// Reconstructed: the unit of the effect's clock (seconds, as elsewhere).

import { Mat4, DEG2RAD } from '../mathlib.js';
import { sceneView, bindMaterial, applyMaterial } from './zeusplay.js';

const ASPECT = 4 / 3;
const UV_SCALE = 1 / 70;         // 0.0142857 in the binary
const FALLOFF = 512 / 182;       // 0.00549451 · 512

// fold a float into [0, 256), the way the binary does with `and eax,0xff`
function wrap(x) {
  const i = Math.trunc(x);
  return x - i + (i & 255);
}

export class Flower {
  constructor(ctx, inst) {
    this.ctx = ctx;
    this.inst = inst;
    this.ready = false;
    this.matCache = new Map();
  }

  async load() {
    const file = this.inst.get('filename');
    if (!file) return;
    const path = String(file).replace(/\\/g, '/').replace(/^\.\//, '').toLowerCase();
    this.scene = await this.ctx.assets.scene(path);
    for (const mesh of this.scene.meshes) {
      for (const b of mesh.batches) await bindMaterial(this.ctx, b.material, this.matCache);
    }
    this.ball = this.scene.byName.get('geosphere01');
    if (this.ball) {
      // keep the undeformed positions; every frame scales from these
      this.rest = this.ball.batches.map((b) => Float32Array.from(b.positions));
    }
    this.ready = true;
  }

  _deform(t) {
    const inst = this.inst;
    const bolscale = inst.num('bolscale', 0.4);
    const turbdiv = inst.num('bolturbdiv', 50) || 50;
    const angdiv = inst.num('bolanglediv', 60) || 60;

    const scale = Math.sin(t / turbdiv) * 0.15 + 0.75;
    const a = t / angdiv;
    const A = a * 50, B = a * -31.8835, C = a * -23.796, D = a * 34.4485;

    this.ball.batches.forEach((b, bi) => {
      const rest = this.rest[bi], P = b.positions, U = b.uvs;
      for (let i = 0, n = rest.length / 3; i < n; i++) {
        const x = rest[i * 3], y = rest[i * 3 + 1], z = rest[i * 3 + 2];
        const u = (x * UV_SCALE + 0.5) * scale;
        const v = (y * UV_SCALE + 0.5) * scale;
        U[i * 2] = u;
        U[i * 2 + 1] = v;

        const us = u * 255, vs = v * 255;
        let p = wrap(A + us) - 128, q = wrap(B + vs) - 128;
        const f1 = Math.max(0, 255 - Math.hypot(p, q) * FALLOFF) / 255 * 0.5;
        p = wrap(C + us) - 128; q = wrap(D + vs) - 128;
        const f2 = Math.max(0, 255 - Math.hypot(p, q) * FALLOFF) / 255 * 0.6;

        const r = bolscale + f1 + f2;
        P[i * 3] = x * r; P[i * 3 + 1] = y * r; P[i * 3 + 2] = z * r;
      }
    });
  }

  draw(local) {
    const mgl = this.ctx.mgl;
    const s = this.scene;
    s.update(s.frameStart + local * this.ctx.fps);
    const cam = s.cameraAt(0);
    if (!cam) return;
    if (this.ball) this._deform(local);

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    const half = Math.tan(cam.fov * DEG2RAD / 2);
    mgl.frustum(-half, half, -half / ASPECT, half / ASPECT, 1, 100000);
    const view = sceneView(cam);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.enableDepthTest(true);
    for (const mesh of s.meshes) {
      const mv = new Mat4();
      mv.copy(view);
      mv.mult(mesh.world);
      mgl.loadMatrix(mv);
      for (const b of mesh.batches) {
        applyMaterial(mgl, this.matCache.get(b.material));
        mgl.drawElements(b.positions, b.uvs, b.indices, b.colors);
      }
    }
  }
}
