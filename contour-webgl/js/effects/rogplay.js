// rogplay — Jace's strange attractor, reimplemented from the parameter
// sets recovered from the binary: the Pickover map
//
//   x' = sin(a·y) − z·cos(b·x)
//   y' = z·sin(c·x) − cos(d·y)
//   z' = sin(x)
//
// iterated point-by-point, the orbit emitted directly as sprites.
// Approximation notes: sprite size, count, coloring and the camera move
// are guesses; the map, its three parameter sets and the contraction are
// straight from the constant pool at 0x435524.

const MODES = [
  { a: 2.24, b: 0.43, c: -0.65, d: -2.43, contraction: 0.7 },
  { a: 0.44, b: -0.66, c: 0.82, d: 2.22, contraction: 1.0 },
  { a: 0.66, b: -0.44, c: 0.74, d: 2.14, contraction: 0.7 },
];
const N_POINTS = 9000;

export class Rogplay {
  constructor(mgl, tex, mode = 0) {
    this.mgl = mgl;
    this.tex = tex.loadTexture('data/textures/flare1.jpg');
    this.bgTex = tex.loadTexture('data/saftext/fx9.jpg');
    this.mode = mode;
    this.orbit = this._computeOrbit(MODES[mode]);
  }

  _computeOrbit(p) {
    const pts = new Float32Array(N_POINTS * 3);
    let x = 0.1, y = 0.1, z = 0.1;
    for (let i = 0; i < N_POINTS; i++) {
      const nx = Math.sin(p.a * y) - z * Math.cos(p.b * x);
      const ny = z * Math.sin(p.c * x) - Math.cos(p.d * y);
      const nz = Math.sin(x);
      x = nx * p.contraction;
      y = ny * p.contraction;
      z = nz * p.contraction;
      pts[i * 3] = nx;
      pts[i * 3 + 1] = ny;
      pts[i * 3 + 2] = nz;
    }
    return pts;
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
    mgl.enableCullFace(false);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);

    // golden cloud backdrop (AVI: the sparkle clouds sit over warm texture
    // for most of the section, dropping to black near its end)
    const bgFade = Math.max(0, Math.min(1, 1 - (t - 11) * 0.4));
    if (bgFade > 0) {
      mgl.enableBlend(false);
      mgl.bindTexture(this.bgTex);
      mgl.color4(0.75 * bgFade, 0.65 * bgFade, 0.5 * bgFade, 1);
      mgl.begin(mgl.QUADS);
      const s = 0.03 * t;
      mgl.texCoord2(s, 0); mgl.vertex3(-1.35, 1, -1.5);
      mgl.texCoord2(1.4 + s, 0); mgl.vertex3(1.35, 1, -1.5);
      mgl.texCoord2(1.4 + s, 1); mgl.vertex3(1.35, -1, -1.5);
      mgl.texCoord2(s, 1); mgl.vertex3(-1.35, -1, -1.5);
      mgl.end();
    } else {
      mgl.enableTexture(false);
      mgl.enableBlend(false);
      mgl.color4(0, 0, 0, 1);
      mgl.begin(mgl.QUADS);
      mgl.vertex3(-1.35, 1, -1.5); mgl.vertex3(1.35, 1, -1.5);
      mgl.vertex3(1.35, -1, -1.5); mgl.vertex3(-1.35, -1, -1.5);
      mgl.end();
      mgl.enableTexture(true);
    }

    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);

    mgl.translate(0, 0, -4.2);
    mgl.rotate(t * 12, 0, 1, 0);
    mgl.rotate(15 * Math.sin(t * 0.4), 1, 0, 0);

    // billboard basis from the inverted modelview, as everywhere else
    const cam = mgl.getModelView();
    cam.inverse();
    const bx = cam.baseX().mulSelf(0.035);
    const by = cam.baseY().mulSelf(0.035);

    mgl.bindTexture(this.tex);

    // how much of the orbit is revealed grows with time
    const reveal = Math.min(N_POINTS, Math.floor(600 + t * 900));

    mgl.begin(mgl.QUADS);
    for (let i = 0; i < reveal; i++) {
      const px = this.orbit[i * 3], py = this.orbit[i * 3 + 1], pz = this.orbit[i * 3 + 2];
      // AVI: pearly white sparkles, warm-tinted, dense
      const a = 0.25 + 0.35 * (i / reveal);
      mgl.color4(1, 0.97, 0.9, a);
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
