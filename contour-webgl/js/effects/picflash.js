// jace/picflash — the rotating flash quad.
//
// Decoded from the constructor (0x41fce0), the buffer allocation
// (0x41fdc0: four vertices, two triangles, indices 0-2-1 / 0-3-2) and
// the update (0x41fe30), which rebuilds the quad every frame:
//
//   t     = T / p[0x14]                     (and sets the dead flag at t > 1)
//   size  = lerp(p[0x00], p[0x04], t) × 0.01
//   ang   = lerp(p[0x08], p[0x0c], t) + π/4
//   z     = p[0x10] + 0.002 t
//   alpha = clamp01((1 − t) × p[0x18]) × 255
//   colour = 0xffffff | alpha<<24
//
// The corner vector is (cos ang, sin ang) × size, and the other three
// corners are that vector turned by 90°, 180° and 270° — a square whose
// half-diagonal is `size`. The baked π/4 is what makes it axis-aligned
// when the angle parameter is zero, which is how the layout gives itself
// away. The four corners carry uv (0,0) (1,0) (1,1) (0,1).
//
// p[0x1c] is a texture name handed to the loader at init (0x41fda0), and
// it is null in all six instances — so the quad is white, and its only
// shape control is the alpha.
//
// Reconstructed: the falloff across the quad. A flat white square at
// alpha 0.8 additive would blow the frame out for the full 27 seconds of
// instances 552/469, and the release capture shows a bloom that decays
// from the centre instead. This port fans the quad from a bright centre
// to transparent corners, keeping the decoded size, spin, depth and
// alpha exactly.

const clamp01 = (v) => (v < 0 ? 0 : v > 1 ? 1 : v);
const ASPECT = 4 / 3;

export class Picflash {
  constructor(mgl, tex, params) {
    this.mgl = mgl;
    this.p = params;
    this.duration = params.duration;
  }

  do(time, timeStart) {
    const mgl = this.mgl;
    const p = this.p;
    const t = (time - timeStart) / (p.duration || 1e-4);
    if (t < 0 || t > 1) return;

    const alpha = clamp01((1 - t) * p.falloff);
    if (alpha <= 0.003) return;

    const size = (p.size[0] + (p.size[1] - p.size[0]) * t) * 0.01;
    const ang = p.angle[0] + (p.angle[1] - p.angle[0]) * t + Math.PI / 4;
    const z = p.depth + 0.002 * t;

    // the original projects the quad at depth z; here the half-diagonal
    // in frame-height units is what survives that projection
    const h = size / (z || 1e-4);

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.ortho(-ASPECT, ASPECT, -1, 1, -1, 1);
    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    mgl.enableTexture(false);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    mgl.enableCullFace(false);
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);

    // corner 0 is (cos, sin) × h; the rest are that turned by 90°
    const cx = Math.cos(ang) * h, cy = Math.sin(ang) * h;
    const corner = [[-cx, -cy], [cy, -cx], [cx, cy], [-cy, cx]];

    mgl.begin(mgl.TRIANGLES);
    for (let i = 0; i < 4; i++) {
      const a = corner[i], b = corner[(i + 1) & 3];
      mgl.color4(1, 1, 1, alpha); mgl.vertex3(0, 0, 0);
      mgl.color4(1, 1, 1, 0); mgl.vertex3(a[0], a[1], 0);
      mgl.color4(1, 1, 1, 0); mgl.vertex3(b[0], b[1], 0);
    }
    mgl.end();
  }
}
