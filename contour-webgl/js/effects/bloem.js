// bloem — Jace's closed-form flower, reimplemented from the formula
// recovered from the binary (see the repository notes): a sphere whose
// radius carries a phase-modulated five-petal wave, pinched by a
// two-lobe latitude envelope.
//
//   R(th, ph) = (3 + 1.5·sin(5·th + 0.8π·sin(2·ph))) · (0.7 − 0.3·cos(4·ph))
//   x = R·sin(ph)·cos(th),  y = 0.8·R·cos(ph),  z = R·sin(ph)·sin(th)
//
// Approximation notes: grid resolution, texture mapping and the animation
// (a slow tumble) are educated guesses — the surface formula is not.

const NU = 96, NV = 64;

export class Bloem {
  constructor(mgl, tex) {
    this.mgl = mgl;
    this.tex = tex.loadTexture('data/saftext/fx8.jpg');

    const nVerts = (NU + 1) * (NV + 1);
    this.positions = new Float32Array(nVerts * 3);
    this.uv = new Float32Array(nVerts * 2);
    const faces = [];
    for (let v = 0; v <= NV; v++) {
      for (let u = 0; u <= NU; u++) {
        const i = v * (NU + 1) + u;
        const th = 2 * Math.PI * u / NU;
        const ph = Math.PI * v / NV;
        const R = (3 + 1.5 * Math.sin(5 * th + 0.8 * Math.PI * Math.sin(2 * ph))) *
                  (0.7 - 0.3 * Math.cos(4 * ph));
        this.positions[i * 3] = R * Math.sin(ph) * Math.cos(th);
        this.positions[i * 3 + 1] = R * Math.cos(ph) * 0.8;
        this.positions[i * 3 + 2] = R * Math.sin(ph) * Math.sin(th);
        this.uv[i * 2] = u / NU * 4;
        this.uv[i * 2 + 1] = v / NV * 2;
        if (u < NU && v < NV) {
          faces.push(i, i + 1, i + NU + 1, i + 1, i + NU + 2, i + NU + 1);
        }
      }
    }
    this.faces = new Uint32Array(faces);
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
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);
    mgl.enableCullFace(false);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);

    mgl.translate(0, 0, -11);
    mgl.rotate(t * 18, 0, 1, 0);
    mgl.rotate(20 * Math.sin(t * 0.6), 1, 0, 0);

    mgl.bindTexture(this.tex);
    mgl.color4(1, 1, 1, 0.55);
    mgl.drawElements(this.positions, this.uv, this.faces);

    // second pass, slightly scaled: cheap glow shell
    mgl.pushMatrix();
    mgl.scale(1.04, 1.04, 1.04);
    mgl.color4(1, 1, 1, 0.18);
    mgl.drawElements(this.positions, this.uv, this.faces);
    mgl.popMatrix();
  }
}
