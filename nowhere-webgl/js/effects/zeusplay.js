// zeusPlay — the generic scene player.
//
// "load the zeusfile" / "Filename of zeus file" / "Material to use" plus
// a viewport rect. It plays a .zeu straight: every mesh drawn with its
// own animated transform, seen through the scene's camera. The boarder
// section is this effect on boarder.zeu.
//
// The material vocabulary comes from the per-part scripts: `additive`,
// `calcenv` (spherical environment mapping generated per vertex, as
// Zeus's c3dObject::CalcEnv does), `envmodulate`, `cull none`.
//
// The exporter targets Direct3D, so it has already converted Max's Z-up
// right-handed world into D3D's **Y-up left-handed** one by swapping Y
// and Z. Two things follow, and both are read out of the data rather
// than assumed: every camera in all eleven scenes looks near-horizontally
// through the XZ plane, so +Y is up; and on every closed mesh — the
// geospheres, the boxes — the right-hand-rule normal of each triangle
// points *inward*, so the winding is left-handed.
//
// GL is right-handed, so the world is mirrored through Z on the way in.
// That fixes the orientation and flips the winding back to CCW-outward,
// which is what GL's default front face expects.
//
// Reconstructed: the scene frame rate.

import { Mat4, Vec3, DEG2RAD } from '../mathlib.js';

const ASPECT = 4 / 3;

const sub = (a, b) => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
const cross = (a, b) => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
const dot = (a, b) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const unit = (a) => { const l = Math.hypot(a[0], a[1], a[2]) || 1; return [a[0] / l, a[1] / l, a[2] / l]; };

export function lookAt(eye, at, roll) {
  const z = unit(sub(eye, at));
  const up = Math.abs(z[1]) > 0.999 ? [0, 0, 1] : [0, 1, 0];
  const x = unit(cross(up, z));
  const y = cross(z, x);
  const m = new Mat4();
  const v = m.m;
  v[0] = x[0]; v[4] = x[1]; v[8] = x[2]; v[12] = -dot(x, eye);
  v[1] = y[0]; v[5] = y[1]; v[9] = y[2]; v[13] = -dot(y, eye);
  v[2] = z[0]; v[6] = z[1]; v[10] = z[2]; v[14] = -dot(z, eye);
  v[3] = v[7] = v[11] = 0; v[15] = 1;
  if (roll) {
    const r = new Mat4();
    r.rotate(-roll / DEG2RAD, 0, 0, 1);
    r.mult(m);
    m.copy(r);
  }
  return m;
}

// Mirror through Z: left-handed scene space -> right-handed GL.
const MIRROR_Z = new Mat4();
MIRROR_Z.m[10] = -1;

// The view matrix an effect should start its model-view from: look-at
// built from the mirrored camera, with the world mirror folded in, so
// `view.mult(mesh.world)` is the complete model-view.
export function sceneView(cam) {
  const flip = (p) => [p[0], p[1], -p[2]];
  const v = lookAt(flip(cam.eye), flip(cam.at), cam.roll);
  v.mult(MIRROR_Z);
  return v;
}

// Apply one material's render state. Returns the primary texture.
export async function bindMaterial(ctx, name, cache) {
  const mat = ctx.materials[String(name || '').toLowerCase()];
  if (!mat) return null;
  if (cache && cache.has(name)) return cache.get(name);
  const texName = mat.textures[0];
  const tex = ctx.textures[texName];
  const out = { mat, tex: null, env: null };
  if (tex) out.tex = await ctx.assets.texture(tex.file, { mipmap: !tex.flags.includes('nomipmap') });
  if (mat.textures[1] && ctx.textures[mat.textures[1]]) {
    out.env = await ctx.assets.texture(ctx.textures[mat.textures[1]].file, {});
  }
  if (cache) cache.set(name, out);
  return out;
}

export function applyMaterial(mgl, m) {
  if (!m) { mgl.enableTexture(false); mgl.enableBlend(false); return; }
  const flags = m.mat.flags;
  mgl.enableTexture(!!m.tex);
  if (m.tex) mgl.bindTexture(m.tex);
  if (flags.includes('additive')) {
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);
    mgl.depthMask(false);
  } else {
    mgl.enableBlend(false);
    mgl.depthMask(true);
  }
  // Culling stays off. The release capture settles it: the greetings
  // ribbon shows its own back face with the lettering reversed (that
  // material does say `cull none`), and the boarder section is built
  // from flat one-sided cut-outs of the snowboarder that vanish under
  // either front-face convention. The closed meshes wind consistently —
  // right-hand normals point inward on every geosphere and box in all
  // eleven scenes — but nothing in the capture shows culling doing any
  // visible work, and turning it on loses content.
  mgl.enableCullFace(false);
}

export class ZeusPlay {
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
    this.ready = true;
  }

  frameAt(local) {
    const s = this.scene;
    const f = s.frameStart + local * this.ctx.fps * (this.inst.num('speed', 1) || 1);
    return Math.min(f, s.frameEnd);
  }

  draw(local) {
    const mgl = this.ctx.mgl;
    const s = this.scene;
    s.update(this.frameAt(local));
    const cam = s.cameraAt(0);
    if (!cam) return;

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    // Max stores the horizontal field of view
    const half = Math.tan(cam.fov * DEG2RAD / 2);
    const near = 1, far = 100000;
    mgl.frustum(-half * near, half * near, -half * near / ASPECT, half * near / ASPECT, near, far);

    const view = sceneView(cam);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.enableDepthTest(true);
    mgl.depthMask(true);

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
