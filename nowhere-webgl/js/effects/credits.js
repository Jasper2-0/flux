// credits — the three credit sections (vic, srx, stv).
//
// Decoded from credits.srx's per-frame routine (0x10001560). The scene
// holds two hemispheres named `GeoSphere01` and `GeoSphere02` — 751
// vertices, 1440 triangles, radius 360, y from 0 up — plus `Rectangle01`,
// an 800×100 plate carrying the credit name, which the scene animates on
// its own.
//
// The effect never moves a vertex. Every frame it rewrites each
// hemisphere's texture coordinates and vertex colour, normalising the
// position by that same 360:
//
//     X = x/360   Y = y/360   Z = z/360
//     ang = atan2(Z, X)                       r = hypot(X, Z)
//     s   = 2·|X|·r^1.2 + 1
//     ang = (−π/2 < ang ≤ π/2) ? ang/s : (ang − π)/s + π
//     u   = asin(asin(cos(ang)·r·0.8))·0.25 + offset + 0.5
//     v   = 0.5 − sin(ang)·r / (2.4 − 1.8·r)
//     grey = clamp(Y − 1.2·r, 0.1, 1)
//
// — a pinch centred on the two poles, so the credit lettering wraps and
// lenses across the dome, lit by height minus radius. `offset` is
// `time/75 − 2`, which walks the texture slowly sideways.
//
// The routine also contains two earlier passes that scroll `u` directly
// from a saved copy of the original coordinates. Both are immediately
// overwritten by the fisheye passes, so they are dead code in the
// shipped build and are not reproduced here.
//
// Reconstructed: the unit of the effect's clock (seconds, as elsewhere).

import { Mat4, DEG2RAD } from '../mathlib.js';
import { sceneView, bindMaterial, applyMaterial } from './zeusplay.js';

const ASPECT = 4 / 3;
const NORM = 1 / 360;            // 0.00277778 in the binary
const SCROLL = 1 / 75;           // 0.0133333
const HALF_PI = Math.PI / 2;

const clamp = (v, lo, hi) => (v < lo ? lo : v > hi ? hi : v);

export class Credits {
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
    this.domes = ['geosphere01', 'geosphere02']
      .map((n) => this.scene.byName.get(n))
      .filter(Boolean);
    this.ready = true;
  }

  // rewrite one hemisphere's uv and colour for this frame
  _shade(mesh, offset) {
    for (const b of mesh.batches) {
      const P = b.positions, U = b.uvs, C = b.colors;
      for (let i = 0, n = P.length / 3; i < n; i++) {
        const X = P[i * 3] * NORM, Y = P[i * 3 + 1] * NORM, Z = P[i * 3 + 2] * NORM;
        let ang = Math.atan2(Z, X);
        const r = Math.hypot(X, Z);
        const s = 2 * Math.abs(X) * Math.pow(r, 1.2) + 1;
        if (ang > -HALF_PI && ang <= HALF_PI) {
          ang /= s;
        } else {
          if (ang > Math.PI * 2) ang -= Math.PI * 2;
          if (ang < 0) ang += Math.PI * 2;
          ang = (ang - Math.PI) / s + Math.PI;
        }
        const a = Math.asin(clamp(Math.cos(ang) * r * 0.8, -1, 1));
        U[i * 2] = Math.asin(clamp(a, -1, 1)) * 0.25 + offset + 0.5;
        U[i * 2 + 1] = 0.5 - (Math.sin(ang) * r) / (2.4 - 1.8 * r);
        const g = clamp(Y - 1.2 * r, 0.1, 1);
        C[i * 4] = C[i * 4 + 1] = C[i * 4 + 2] = g;
        C[i * 4 + 3] = 1;
      }
    }
  }

  draw(local) {
    const mgl = this.ctx.mgl;
    const s = this.scene;
    s.update(s.frameStart + local * this.ctx.fps);
    const cam = s.cameraAt(0);
    if (!cam) return;

    const offset = local * SCROLL - 2;
    for (const dome of this.domes) this._shade(dome, offset);

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
