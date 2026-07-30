// nixfx/zoomer — the infinite zoom, integrated from the standalone
// recreation (jasperschelling.nl/contour-zoomer): ten plates nested at the
// measured quarter scale, dead centre, drawn largest-first with radially
// feathered edges so the sharper inner plate always wins; the finale
// painting (Shun) sits on plate 9 at full width, a quarter turn off.
//
// Two endings, as in the recreation:
//   'original' — constant zoom and spin, Shun fades up unaligned (shipped)
//   'aligned'  — the whole run solved backwards so zoom and spin decelerate
//                together and Shun lands upright, exactly framed (intended,
//                confirmed by TBL)

const RATIO = 4;            // measured: each plate holds the next at 1/4
const PULL = 0.55;          // start this many plate-steps further out
const LAYERS = 5;
const FEATHER = 0.85;
const COVER = 1.6;
const XFADE = 1.0;
const FINALE_ART = 0.75;    // artwork is the top 3/4 of the 512² texture
const FIN_TURN = -Math.PI / 2;
const ASPECT = 4 / 3;
const TAU = Math.PI * 2;
const GLIDE = 0.28;
const CRUISE = 1 / (1 - GLIDE + GLIDE / 3);

const smooth = (x) => x * x * (3 - 2 * x);
const clamp01 = (x) => (x < 0 ? 0 : x > 1 ? 1 : x);

function glide(s) {
  if (s <= 1 - GLIDE) return CRUISE * s;
  const t = (s - (1 - GLIDE)) / GLIDE;
  return CRUISE * ((1 - GLIDE) + GLIDE * (1 - Math.pow(1 - t, 3)) / 3);
}

const GRID = 8; // feather approximated per-vertex on an 8x8 quad grid

export class Zoomer {
  // opts: { ending: 'original' | 'aligned', speed, rotRate }
  constructor(mgl, tex, opts = {}) {
    this.mgl = mgl;
    this.plates = [];
    for (let i = 0; i <= 9; i++) {
      this.plates.push(tex.loadTexture('data/zoomer/image' + i + 'a.jpg'));
    }
    this.shun = tex.loadTexture('data/zoomer/shun-compo.jpg');
    this.ending = opts.ending || 'original';
    // shipped pacing: u reaches 9 after ~50 s (AVI: zoom 108→158 s)
    this.speed = opts.speed || 0.191;
    this.rotRate = opts.rotRate || 0.12;
  }

  // one feathered plate: unit quad [-1,1]², per-vertex radial alpha
  _plate(texture, scale, rot, alpha, feather) {
    if (alpha <= 0.003) return;
    const mgl = this.mgl;
    mgl.bindTexture(texture);
    mgl.pushMatrix();
    mgl.rotate(rot * 180 / Math.PI, 0, 0, 1);
    mgl.scale(scale, scale, 1);
    mgl.begin(mgl.QUADS);
    for (let gy = 0; gy < GRID; gy++) {
      for (let gx = 0; gx < GRID; gx++) {
        for (const [dx, dy] of [[0, 0], [1, 0], [1, 1], [0, 1]]) {
          const u = (gx + dx) / GRID, v = (gy + dy) / GRID;
          const r = Math.hypot((u - 0.5) * 2, (v - 0.5) * 2);
          const fade = feather > 1.5 ? 0 : smooth(clamp01((r - feather) / (1 - feather)));
          mgl.color4(1, 1, 1, alpha * (1 - fade));
          mgl.texCoord2(u, v);
          mgl.vertex3(u * 2 - 1, 1 - v * 2, 0);
        }
      }
    }
    mgl.end();
    mgl.popMatrix();
  }

  do(time, timeStart) {
    const mgl = this.mgl;
    const t = time - timeStart;

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.ortho(-ASPECT, ASPECT, -1, 1, -1, 1);
    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    mgl.enableTexture(true);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    mgl.enableCullFace(false);
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE_MINUS_SRC_ALPHA);

    const uMax = 9;
    const zoomBase = ASPECT * COVER;
    const fitW = ASPECT; // Shun is natively 4:3: fills the 4:3 frame exactly

    let u, rot, dissolve, chainOut;
    if (this.ending === 'aligned') {
      // solved backwards: land plate 9 (and the painting riding it) at fitW
      const uEnd = uMax + Math.log(fitW / zoomBase) / Math.log(RATIO);
      const span = uEnd - (-PULL);
      const runT = CRUISE * span / this.speed;
      const turns = Math.round((this.rotRate * runT + FIN_TURN) / TAU);
      const spinTotal = -FIN_TURN + TAU * turns;
      const s = clamp01(t / runT);
      const g = glide(s);
      u = -PULL + span * g;
      rot = spinTotal * g;
      dissolve = smooth(clamp01((s - 0.78) / 0.18));
      chainOut = smooth(clamp01((s - 0.94) / 0.06));
    } else {
      // constant rate throughout, the way it shipped
      u = Math.min(uMax, -PULL + t * this.speed);
      rot = t * this.rotRate;
      const over = t - uMax / this.speed;
      dissolve = smooth(clamp01(over / 1.6));
      chainOut = dissolve;
    }

    const K = Math.max(0, Math.min(Math.floor(u), uMax));
    const f = u - K;

    // largest plate first (unfeathered backdrop), sharper plates on top
    for (let j = 0; j < LAYERS; j++) {
      const idx = K + j;
      if (idx > uMax) break;
      const sc = Math.pow(RATIO, f - j) * zoomBase;
      const a = (j === LAYERS - 1) ? smooth(clamp01(f / XFADE)) : 1;
      this._plate(this.plates[idx], sc, rot, a * (1 - chainOut), j === 0 ? 2.0 : FEATHER);
    }

    // the finale painting
    if (dissolve > 0) {
      const mglc = this.mgl;
      let hx, hy, r;
      if (this.ending === 'aligned') {
        const sc = Math.pow(RATIO, u - uMax) * zoomBase; // == plate 9's size
        hx = sc; hy = sc * FINALE_ART; r = rot + FIN_TURN;
      } else {
        hx = fitW; hy = fitW * FINALE_ART; r = 0;
      }
      mglc.bindTexture(this.shun);
      mglc.pushMatrix();
      mglc.rotate(r * 180 / Math.PI, 0, 0, 1);
      mglc.begin(mglc.QUADS);
      mglc.color4(1, 1, 1, dissolve);
      // artwork occupies the top FINALE_ART of the texture
      mglc.texCoord2(0, 0); mglc.vertex3(-hx, hy, 0);
      mglc.texCoord2(1, 0); mglc.vertex3(hx, hy, 0);
      mglc.texCoord2(1, FINALE_ART); mglc.vertex3(hx, -hy, 0);
      mglc.texCoord2(0, FINALE_ART); mglc.vertex3(-hx, -hy, 0);
      mglc.end();
      mglc.popMatrix();
    }
  }
}
