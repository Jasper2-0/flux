// jace/grid1fx — a standing wave on a parametric surface, from the
// formula recovered out of the binary:
//
//   R(u,v,t) = 100 + 50·sin(1.2t + 8v)·sin(0.8t + 4u) + 40·f(0.1t)·sin(πv)
//
// A product of two travelling sines is the normal-mode solution for a
// vibrating membrane — Chladni mathematics — and the two temporal
// frequencies (1.2 and 0.8) are deliberately incommensurate, so the
// interference never visibly repeats. f() is the six-octave summed
// oscillator at 0x420820, used here as an fBm-ish animation source.
//
// Reconstructed: grid resolution, the mapping of R onto a sphere, the
// camera, and the blend. The radius field is the original.

const NU = 64, NV = 40;

// six-octave summed oscillator, standing in for the routine at 0x420820
function fbm(x) {
  let sum = 0, amp = 1, freq = 1, norm = 0;
  for (let i = 0; i < 6; i++) {
    sum += amp * Math.sin(x * freq + i * 1.7);
    norm += amp;
    amp *= 0.62;
    freq *= 1.9;
  }
  return sum / norm;
}

export class Grid1fx {
  constructor(mgl, tex) {
    this.mgl = mgl;
    this.tex = tex.loadTexture('data/saftext/fx8.jpg');
    this.line = tex.loadTexture('data/textures/line6.jpg');

    const n = (NU + 1) * (NV + 1);
    this.positions = new Float32Array(n * 3);
    this.colors = new Float32Array(n * 4);
    this.uv = new Float32Array(n * 2);
    const faces = [];
    for (let j = 0; j <= NV; j++) {
      for (let i = 0; i <= NU; i++) {
        const k = j * (NU + 1) + i;
        this.uv[k * 2] = i / NU * 3;
        this.uv[k * 2 + 1] = j / NV * 2;
        if (i < NU && j < NV) {
          faces.push(k, k + 1, k + NU + 1, k + 1, k + NU + 2, k + NU + 1);
        }
      }
    }
    this.faces = new Uint32Array(faces);
  }

  _evaluate(t) {
    const wob = fbm(0.1 * t);
    for (let j = 0; j <= NV; j++) {
      const v = j / NV;
      const ph = Math.PI * v;
      const sv = Math.sin(1.2 * t + 8 * v);
      const env = 40 * wob * Math.sin(ph);
      for (let i = 0; i <= NU; i++) {
        const u = i / NU;
        const th = 2 * Math.PI * u;
        const R = 100 + 50 * sv * Math.sin(0.8 * t + 4 * u) + env;
        const k = j * (NU + 1) + i;
        const s = R / 100;
        this.positions[k * 3] = s * Math.sin(ph) * Math.cos(th);
        this.positions[k * 3 + 1] = s * Math.cos(ph);
        this.positions[k * 3 + 2] = s * Math.sin(ph) * Math.sin(th);
        // brightness rides the wave itself, so the nodal lines read
        const w = 0.5 + 0.5 * sv * Math.sin(0.8 * t + 4 * u);
        this.colors[k * 4] = 0.30 + 0.55 * w;
        this.colors[k * 4 + 1] = 0.24 + 0.46 * w;
        this.colors[k * 4 + 2] = 0.26 + 0.34 * w;
        this.colors[k * 4 + 3] = 1;
      }
    }
  }

  do(time, timeStart) {
    const mgl = this.mgl;
    const t = time - timeStart;

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.frustum(-0.6, 0.6, -0.45, 0.45, 1, 1000);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    mgl.enableTexture(true);
    mgl.enableCullFace(false);
    mgl.enableBlend(false);
    mgl.enableDepthTest(true);
    mgl.depthMask(true);

    mgl.translate(0, 0, -7.2);
    mgl.rotate(t * 14, 0, 1, 0);
    mgl.rotate(18 * Math.sin(t * 0.5) - 8, 1, 0, 0);

    this._evaluate(t);

    mgl.bindTexture(this.tex);
    mgl.color4(1, 1, 1, 1);
    mgl.drawElements(this.positions, this.uv, this.faces, this.colors);

    // a larger, dim companion shell — the capture shows line structures
    // wrapping the membrane through this section
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    mgl.pushMatrix();
    mgl.scale(1.7, 1.7, 1.7);
    mgl.rotate(t * 9, 0, 1, 0);
    mgl.bindTexture(this.line);
    mgl.color4(0.35, 0.33, 0.30, 1);
    mgl.drawElements(this.positions, this.uv, this.faces);
    mgl.popMatrix();
  }
}
