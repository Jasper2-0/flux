// TimScene — a replayer for the ARSE scene format: baked per-object
// transforms at 30 fps, a baked camera track, meshes with normals and UVs.
//
// Approximation notes: the file provides geometry, animation, camera and
// projection (fov/znear/zfar) — but not shading or blend state. This
// replayer renders textured with a simple headlamp diffuse baked into
// vertex colors (the XYZ|DIFFUSE|TEX1 layout the engine used). Handedness
// is a best guess until frames can be compared against the AVI capture.

import { Mat4 } from '../mathlib.js';

export class TimScene {
  constructor(mgl, tex, scene, textureName) {
    this.mgl = mgl;
    this.scene = scene;
    this.tex = tex.loadTexture(textureName);

    // scratch buffers per object
    this.world = [];
    this.colors = [];
    for (const obj of scene.objects) {
      this.world.push(new Float32Array(obj.nVerts * 3));
      this.colors.push(new Float32Array(obj.nVerts * 4));
    }
  }

  do(time, timeStart) {
    const mgl = this.mgl;
    const t = time - timeStart;
    const scene = this.scene;

    const frame = Math.min(scene.frames - 1, Math.floor(t * scene.fps));

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    // vertical fov from the file; 4:3 frame
    const fovRad = scene.fov * Math.PI / 180;
    const top = Math.tan(fovRad / 2) * scene.znear * 0.75;
    mgl.frustum(-top * 4 / 3, top * 4 / 3, -top, top, scene.znear, scene.zfar);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    // camera: track record is camera-to-world; view = inverse
    const cam = scene.camera[frame];
    const view = new Mat4();
    const m = cam.m, ct = cam.t;
    // rows of the 3x3 become columns (transpose = inverse rotation)
    view.m.set([
      m[0], m[3], m[6], 0,
      m[1], m[4], m[7], 0,
      m[2], m[5], m[8], 0,
      -(ct[0] * m[0] + ct[1] * m[1] + ct[2] * m[2]),
      -(ct[0] * m[3] + ct[1] * m[4] + ct[2] * m[5]),
      -(ct[0] * m[6] + ct[1] * m[7] + ct[2] * m[8]),
      1,
    ]);
    mgl.multMatrix(view);

    mgl.enableTexture(true);
    mgl.enableBlend(false);
    mgl.enableDepthTest(true);
    mgl.depthMask(true);
    mgl.enableCullFace(false);

    mgl.bindTexture(this.tex);

    for (let o = 0; o < scene.objects.length; o++) {
      const obj = scene.objects[o];
      const xf = scene.anim[frame][o];
      const w = this.world[o], col = this.colors[o];
      const om = xf.m, ot = xf.t;

      for (let v = 0; v < obj.nVerts; v++) {
        const px = obj.positions[v * 3], py = obj.positions[v * 3 + 1], pz = obj.positions[v * 3 + 2];
        w[v * 3] = px * om[0] + py * om[3] + pz * om[6] + ot[0];
        w[v * 3 + 1] = px * om[1] + py * om[4] + pz * om[7] + ot[1];
        w[v * 3 + 2] = px * om[2] + py * om[5] + pz * om[8] + ot[2];

        // headlamp diffuse from the baked normals (world-rotated)
        const nx = obj.normals[v * 3], ny = obj.normals[v * 3 + 1], nz = obj.normals[v * 3 + 2];
        const wnx = nx * om[0] + ny * om[3] + nz * om[6];
        const wny = nx * om[1] + ny * om[4] + nz * om[7];
        const wnz = nx * om[2] + ny * om[5] + nz * om[8];
        // light direction: from camera toward scene
        const lx = -cam.m[6], ly = -cam.m[7], lz = -cam.m[8];
        let d = wnx * lx + wny * ly + wnz * lz;
        d = 0.25 + 0.75 * Math.max(0, Math.abs(d));
        col[v * 4] = d; col[v * 4 + 1] = d; col[v * 4 + 2] = d; col[v * 4 + 3] = 1;
      }

      mgl.drawElements(w, obj.uv, obj.faces, col);
    }
  }
}
