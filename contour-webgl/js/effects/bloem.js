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
    this.bgTex = tex.loadTexture('data/saftext/fx4.jpg');

    const nVerts = (NU + 1) * (NV + 1);
    this.positions = new Float32Array(nVerts * 3);
    this.colors = new Float32Array(nVerts * 4);
    this.uv = new Float32Array(nVerts * 2);
    const faces = [];
    for (let v = 0; v <= NV; v++) {
      for (let u = 0; u <= NU; u++) {
        const i = v * (NU + 1) + u;
        this.uv[i * 2] = u / NU * 4;
        this.uv[i * 2 + 1] = v / NV * 2;
        if (u < NU && v < NV) {
          faces.push(i, i + 1, i + NU + 1, i + 1, i + NU + 2, i + NU + 1);
        }
      }
    }
    this.faces = new Uint32Array(faces);
  }

  // The AVI shows the petal shape itself animating; the recovered formula
  // is a single evaluation, so the port animates the modulation index and
  // amplitude gently around the recovered constants (1.5 and 0.8π).
  _evaluate(t) {
    const amp = 1.5 + 0.35 * Math.sin(t * 0.7);
    const modIndex = 0.8 * Math.PI * (1 + 0.15 * Math.sin(t * 0.45));
    for (let v = 0; v <= NV; v++) {
      for (let u = 0; u <= NU; u++) {
        const i = v * (NU + 1) + u;
        const th = 2 * Math.PI * u / NU;
        const ph = Math.PI * v / NV;
        const R = (3 + amp * Math.sin(5 * th + modIndex * Math.sin(2 * ph))) *
                  (0.7 - 0.3 * Math.cos(4 * ph));
        this.positions[i * 3] = R * Math.sin(ph) * Math.cos(th);
        this.positions[i * 3 + 1] = R * Math.cos(ph) * 0.8;
        this.positions[i * 3 + 2] = R * Math.sin(ph) * Math.sin(th);

        // iridescent per-vertex color (AVI: pink/green/copper gradients
        // sweeping the petals) — hue from surface angle, drifting with time
        const hue = th * 1.5 + ph * 2 + t * 0.5;
        this.colors[i * 4] = 0.62 + 0.3 * Math.sin(hue);
        this.colors[i * 4 + 1] = 0.5 + 0.28 * Math.sin(hue + 2.1);
        this.colors[i * 4 + 2] = 0.45 + 0.3 * Math.sin(hue + 4.2);
        this.colors[i * 4 + 3] = 1;
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

    // cloudy backdrop (AVI: golden cloud sky behind the flower)
    mgl.enableTexture(true);
    mgl.enableBlend(false);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    mgl.bindTexture(this.bgTex);
    mgl.color4(0.85, 0.8, 0.75, 1);
    mgl.begin(mgl.QUADS);
    const s = 0.05 * t;
    mgl.texCoord2(s, 0); mgl.vertex3(-2.7, 2, -3);
    mgl.texCoord2(1.5 + s, 0); mgl.vertex3(2.7, 2, -3);
    mgl.texCoord2(1.5 + s, 1.2); mgl.vertex3(2.7, -2, -3);
    mgl.texCoord2(s, 1.2); mgl.vertex3(-2.7, -2, -3);
    mgl.end();

    // opaque, depth-tested flower — the AVI shows solid petals, not glow
    this._evaluate(t);
    mgl.enableDepthTest(true);
    mgl.depthMask(true);

    mgl.translate(0, 0, -11);
    mgl.rotate(t * 18, 0, 1, 0);
    mgl.rotate(20 * Math.sin(t * 0.6), 1, 0, 0);

    mgl.bindTexture(this.tex);
    mgl.drawElements(this.positions, this.uv, this.faces, this.colors);
  }
}
