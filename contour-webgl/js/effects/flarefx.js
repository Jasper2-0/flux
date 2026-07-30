// jace/flarefx — flare sprites driven by the same strange-attractor
// constant pool as rogplay (0x435524), drawn with textures/flare8.jpg.
// It is the smallest of Jace's geometry effects at 172 bytes of state,
// and it carries the section between the neuron scene and the flower.
//
// From the binary: the Pickover map and its parameter sets, and the
// texture. Reconstructed: sprite size and count, the colour, the camera
// move and the way the cloud accumulates.

const MODES = [
  { a: 2.24, b: 0.43, c: -0.65, d: -2.43, contraction: 0.7 },
  { a: 0.44, b: -0.66, c: 0.82, d: 2.22, contraction: 1.0 },
  { a: 0.66, b: -0.44, c: 0.74, d: 2.14, contraction: 0.7 },
];
const N = 5200;

export class Flarefx {
  constructor(mgl, tex, mode = 1) {
    this.mgl = mgl;
    this.tex = tex.loadTexture('data/textures/flare8.jpg');
    const p = MODES[mode];
    this.pts = new Float32Array(N * 3);
    let x = 0.15, y = -0.1, z = 0.3;
    for (let i = 0; i < N; i++) {
      const nx = Math.sin(p.a * y) - z * Math.cos(p.b * x);
      const ny = z * Math.sin(p.c * x) - Math.cos(p.d * y);
      const nz = Math.sin(x);
      x = nx * p.contraction; y = ny * p.contraction; z = nz * p.contraction;
      this.pts[i * 3] = nx; this.pts[i * 3 + 1] = ny; this.pts[i * 3 + 2] = nz;
    }
  }

  do(time, timeStart) {
    const mgl = this.mgl;
    const t = time - timeStart;

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.frustum(-0.6, 0.6, -0.45, 0.45, 0.5, 100);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    mgl.enableTexture(true);
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);
    mgl.enableCullFace(false);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);

    mgl.translate(0, 0, -3.4 - 0.5 * Math.sin(t * 0.25));
    mgl.rotate(t * 9, 0, 1, 0);
    mgl.rotate(22 * Math.sin(t * 0.33), 1, 0, 0);
    mgl.rotate(6 * Math.sin(t * 0.7), 0, 0, 1);

    const cam = mgl.getModelView();
    cam.inverse();
    const bx = cam.baseX().mulSelf(0.055);
    const by = cam.baseY().mulSelf(0.055);

    mgl.bindTexture(this.tex);
    // the cloud fills in over the first seconds and thins out at the end
    const grow = Math.min(1, 0.12 + t * 0.16);
    const fade = Math.min(1, t * 0.7);
    const count = Math.floor(N * grow);

    mgl.begin(mgl.QUADS);
    for (let i = 0; i < count; i++) {
      const px = this.pts[i * 3], py = this.pts[i * 3 + 1], pz = this.pts[i * 3 + 2];
      // gentle per-sprite twinkle, decorrelated by index
      const tw = 0.55 + 0.45 * Math.sin(t * 2.1 + i * 0.7);
      const a = 0.30 * tw * fade;
      mgl.color4(1, 0.985, 0.95, a);
      mgl.texCoord2(0, 0);
      mgl.vertex3(px - bx.x + by.x, py - bx.y + by.y, pz - bx.z + by.z);
      mgl.texCoord2(1, 0);
      mgl.vertex3(px + bx.x + by.x, py + bx.y + by.y, pz + bx.z + by.z);
      mgl.texCoord2(1, 1);
      mgl.vertex3(px + bx.x - by.x, py + bx.y - by.y, pz + bx.z - by.z);
      mgl.texCoord2(0, 1);
      mgl.vertex3(px - bx.x - by.x, py - bx.y - by.y, pz - bx.z - by.z);
    }
    mgl.end();
  }
}
