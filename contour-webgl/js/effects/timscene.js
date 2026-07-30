// TimScene — replayer for the ARSE scene format: baked per-object
// transforms at 30 fps, a baked camera track, meshes with normals and UVs.
//
// Camera convention CALIBRATED against the release AVI capture
// (wireframe-projection comparison over the neuron scene):
//   world = v · objRows + t          (row-basis object transform)
//   eye   = camRowsᵀ · (world − camPos)
//   +z is forward (D3D left-handed) — one z-flip converts to GL.
// Shading and blend state are still reconstruction guesses.

export class TimScene {
  // opts.additive / opts.gain present the scene as dim glowing geometry
  // rather than lit solids — used where the capture shows the mesh as a
  // dark presence rather than a surface.
  constructor(mgl, tex, scene, textureName, opts = {}) {
    this.mgl = mgl;
    this.scene = scene;
    this.additive = !!opts.additive;
    this.gain = opts.gain === undefined ? 1 : opts.gain;
    this.tex = tex.loadTexture(textureName);

    this.duration = scene.frames / scene.fps;

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
    // horizontal fov from the file; 4:3 frame
    const half = Math.tan((scene.fov * Math.PI / 180) / 2) * scene.znear;
    mgl.frustum(-half, half, -half * 0.75, half * 0.75, scene.znear, scene.zfar);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    const cam = scene.camera[frame];
    const cm = cam.m, ct = cam.t;
    // view = transpose(camRows) · translate(−camPos), with the D3D→GL z flip
    // baked into the third component. Column-major GL layout.
    const view = new Float32Array([
      cm[0], cm[1], -cm[2], 0,
      cm[3], cm[4], -cm[5], 0,
      cm[6], cm[7], -cm[8], 0,
      -(cm[0] * ct[0] + cm[3] * ct[1] + cm[6] * ct[2]),
      -(cm[1] * ct[0] + cm[4] * ct[1] + cm[7] * ct[2]),
      +(cm[2] * ct[0] + cm[5] * ct[1] + cm[8] * ct[2]),
      1,
    ]);
    mgl.multMatrix(view);

    mgl.enableTexture(true);
    mgl.enableCullFace(false);
    if (this.additive) {
      mgl.enableBlend(true);
      mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);
      mgl.enableDepthTest(false);
      mgl.depthMask(false);
    } else {
      mgl.enableBlend(false);
      mgl.enableDepthTest(true);
      mgl.depthMask(true);
    }

    mgl.bindTexture(this.tex);

    for (let o = 0; o < scene.objects.length; o++) {
      const obj = scene.objects[o];
      const xf = scene.anim[frame][o];
      const w = this.world[o], col = this.colors[o];
      const om = xf.m, ot = xf.t;

      for (let v = 0; v < obj.nVerts; v++) {
        const px = obj.positions[v * 3], py = obj.positions[v * 3 + 1], pz = obj.positions[v * 3 + 2];
        // row-basis: world = v.x·row0 + v.y·row1 + v.z·row2 + t
        w[v * 3] = px * om[0] + py * om[1] + pz * om[2] + ot[0];
        w[v * 3 + 1] = px * om[3] + py * om[4] + pz * om[5] + ot[1];
        w[v * 3 + 2] = px * om[6] + py * om[7] + pz * om[8] + ot[2];

        const nx = obj.normals[v * 3], ny = obj.normals[v * 3 + 1], nz = obj.normals[v * 3 + 2];
        const wnx = nx * om[0] + ny * om[1] + nz * om[2];
        const wny = nx * om[3] + ny * om[4] + nz * om[5];
        const wnz = nx * om[6] + ny * om[7] + nz * om[8];
        // headlamp: light along the camera's forward row
        const d = Math.abs(wnx * cm[2] + wny * cm[5] + wnz * cm[8]);
        const s = (0.3 + 0.7 * d) * this.gain;
        col[v * 4] = s; col[v * 4 + 1] = s; col[v * 4 + 2] = s;
        col[v * 4 + 3] = this.additive ? this.gain : 1;
      }

      mgl.drawElements(w, obj.uv, obj.faces, col);
    }
  }
}
