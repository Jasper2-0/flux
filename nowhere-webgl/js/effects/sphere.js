// Sphere — "Triangled Sphere".
//
// Decoded from sphere.srx. The plugin asks the scene for two objects it
// insists on by name: `GeoSphere01` ("Please name the minimal object")
// and `GeoSphere02` ("Please name the maximal object"), and refuses to
// run if their polygon counts differ. They have identical topology —
// 418 vertices, 720 triangles.
//
// At load it packs one 0x110-byte record per triangle holding *both*
// versions of that triangle, three 44-byte vertices from each sphere,
// then per frame (0x100016e0) computes a blend factor into the record:
//
//     phase = (i & 15) + 10                    // 10..25, per triangle
//     blend = (sin(phase · k · π · time · 0.01) + 1) / 2
//
// and rebuilds the geometry (0x10001750) by lerping the maximal
// triangle towards the minimal one — literally `(B − A) · blend + A`,
// coordinate by coordinate — then allocating 6 vertices and 7 polygons
// per triangle: the base triangle on the small sphere, the moving
// triangle out at the tip, and the walls between them. A sphere of
// pulsing spikes, each at its own rate.
//
// `k` is per-triangle and set from `rand() / 43690 + 0.2` at load. MSVC's
// rand() tops out at 32767, so that division is always zero and every
// triangle gets exactly 0.2 — a latent bug in the original that this
// port reproduces, because reproducing it is what matches the release.
//
// Reconstructed: the unit of the effect's clock (taken as seconds, which
// puts the slowest triangles at a 100-second cycle and the fastest at
// 40 — a slow, staggered bloom across the section), and the wall
// triangulation, which the binary allocates but whose winding is not
// recovered.

import { Mat4, DEG2RAD } from '../mathlib.js';
import { sceneView, bindMaterial, applyMaterial } from './zeusplay.js';

const ASPECT = 4 / 3;
const K = 0.2;

export class Sphere {
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
    this.min = this.scene.byName.get('geosphere01');
    this.max = this.scene.byName.get('geosphere02');
    if (!this.min || !this.max) throw new Error('sphere: GeoSphere01/02 missing');

    // the batches are one per material and both spheres carry exactly
    // one ("Please don't put 2 materials on the GeoSpheres")
    this.material = this.min.batches[0].material;
    await bindMaterial(this.ctx, this.material, this.matCache);

    const A = this.min.batches[0], B = this.max.batches[0];
    const tris = Math.min(A.indices.length, B.indices.length) / 3 | 0;
    this.tris = tris;

    // base and tip vertices per triangle: 6 positions, 6 uvs, 8 triangles
    // of walls-and-cap, built once and re-lerped every frame
    this.src = { A, B };
    this.positions = new Float32Array(tris * 6 * 3);
    this.uvs = new Float32Array(tris * 6 * 2);
    this.colors = new Float32Array(tris * 6 * 4);
    const idx = [];
    for (let t = 0; t < tris; t++) {
      const b = t * 6;                       // 0,1,2 base · 3,4,5 tip
      for (let c = 0; c < 3; c++) {
        const ia = A.indices[t * 3 + c];
        for (let k = 0; k < 2; k++) {
          this.uvs[(b + c + k * 3) * 2] = A.uvs[ia * 2];
          this.uvs[(b + c + k * 3) * 2 + 1] = A.uvs[ia * 2 + 1];
          for (let k2 = 0; k2 < 4; k2++) this.colors[(b + c + k * 3) * 4 + k2] = A.colors[ia * 4 + k2];
        }
      }
      idx.push(b + 3, b + 4, b + 5);          // the cap, out at the tip
      for (let c = 0; c < 3; c++) {           // and the three walls
        const n = (c + 1) % 3;
        idx.push(b + c, b + n, b + 3 + n, b + c, b + 3 + n, b + 3 + c);
      }
    }
    this.indices = new Uint32Array(idx);
    this.ready = true;
  }

  draw(local) {
    const mgl = this.ctx.mgl;
    const s = this.scene;
    s.update(s.frameStart + local * this.ctx.fps);
    const cam = s.cameraAt(0);
    if (!cam) return;

    const A = this.src.A, B = this.src.B, P = this.positions;
    for (let t = 0; t < this.tris; t++) {
      const phase = (t & 15) + 10;
      const blend = (Math.sin(phase * K * Math.PI * local * 0.01) + 1) * 0.5;
      const b = t * 6;
      for (let c = 0; c < 3; c++) {
        const ia = A.indices[t * 3 + c] * 3, ib = B.indices[t * 3 + c] * 3;
        const o = (b + c) * 3, o2 = (b + 3 + c) * 3;
        for (let k = 0; k < 3; k++) {
          const a = A.positions[ia + k];
          P[o + k] = a;
          P[o2 + k] = (B.positions[ib + k] - a) * blend + a;
        }
      }
    }

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    const half = Math.tan(cam.fov * DEG2RAD / 2);
    mgl.frustum(-half, half, -half / ASPECT, half / ASPECT, 1, 100000);
    const mv = new Mat4();
    mv.copy(sceneView(cam));
    mv.mult(this.min.world);
    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadMatrix(mv);

    mgl.enableDepthTest(true);
    mgl.depthMask(true);
    applyMaterial(mgl, this.matCache.get(this.material));
    mgl.enableCullFace(false);      // the wall winding is not recovered
    mgl.drawElements(this.positions, this.uvs, this.indices, this.colors);
  }
}
