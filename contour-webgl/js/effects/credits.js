// bally/credit3 — one credit per instance, cut from credits-tekst.jpg.
//
// The atlas holds every name and role label as pre-rendered type; the
// boxes below were measured from the artwork. The timeline creates eight
// credit3 instances on a 3.5-second grid (12 … 36.5 s) but does not name
// which credit each one shows — that lives in the effect's own state, so
// the order here follows the release capture.
//
// Reconstructed: placement, the drift, the rule under the name and the
// fade envelope. The type itself is the original artwork.

const ATLAS_W = 512, ATLAS_H = 256;
const box = (x0, x1, y0, y1) => ({
  u0: x0 / ATLAS_W, u1: x1 / ATLAS_W, v0: y0 / ATLAS_H, v1: y1 / ATLAS_H,
  aspect: (x1 - x0) / (y1 - y0),
});

const NAME = {
  NIX: box(15, 87, 29, 68),
  JACE: box(110, 204, 29, 68),
  BALANCE: box(232, 417, 29, 68),
  SCID: box(14, 106, 101, 141),
  SAFFRON: box(136, 320, 101, 141),
  TIM: box(352, 448, 108, 150),
  CRYSTAL: box(13, 163, 148, 183),
  SCORE: box(50, 161, 193, 231),
  SICK: box(191, 268, 148, 183),
  SJAAK: box(191, 302, 193, 231),
};
const ROLE = {
  code: box(19, 42, 2, 18),
  graphics: box(266, 321, 75, 97),
  modelling: box(303, 378, 203, 229),
  soundtrack: box(95, 152, 233, 250),
};

// creation order and placement, read off the release capture. `dx` shifts
// the second line of the stacked pairs, as the artwork sets them.
const ORDER = [
  { lines: ['NIX'], role: 'code', x: -0.30, y: 0.02 },
  { lines: ['JACE'], role: 'code', x: 0.16, y: -0.14 },
  { lines: ['BALANCE'], role: 'code', x: -0.16, y: -0.30 },
  { lines: ['SAFFRON'], role: 'graphics', x: 0.20, y: 0.16 },
  { lines: ['CRYSTAL', 'SCORE'], role: 'soundtrack', x: -0.34, y: 0.06, dx: [0, 0.14] },
  { lines: ['SCID'], role: 'code', x: -0.38, y: 0.32 },
  { lines: ['SICK', 'SJAAK'], role: 'modelling', x: 0.10, y: -0.12, dx: [0, 0.12] },
  { lines: ['TIM'], role: 'code', x: 0.46, y: 0.20 },
];

const ASPECT = 4 / 3;
const CAP = 0.22;        // cap height of a name line, in y-units
const RULE_Y = -0.02;    // the thin rule sits at a fixed height
const LIFE = 5.2;
const FADE = 0.9;

const clamp01 = (v) => (v < 0 ? 0 : v > 1 ? 1 : v);
const smooth = (x) => x * x * (3 - 2 * x);

let counter = 0;
export function resetCreditOrder() { counter = 0; }

export class Credit {
  constructor(mgl, tex) {
    this.mgl = mgl;
    this.tex = tex.loadTexture('data/credits/credits-tekst.jpg');
    this.spec = ORDER[counter % ORDER.length];
    this.duration = LIFE;
    counter++;
  }

  _quad(b, cx, cy, h, alpha) {
    const mgl = this.mgl;
    const w = h * b.aspect;
    mgl.color4(1, 1, 1, alpha);
    mgl.texCoord2(b.u0, b.v0); mgl.vertex3(cx - w / 2, cy + h / 2, 0);
    mgl.texCoord2(b.u1, b.v0); mgl.vertex3(cx + w / 2, cy + h / 2, 0);
    mgl.texCoord2(b.u1, b.v1); mgl.vertex3(cx + w / 2, cy - h / 2, 0);
    mgl.texCoord2(b.u0, b.v1); mgl.vertex3(cx - w / 2, cy - h / 2, 0);
  }

  do(time, timeStart) {
    const mgl = this.mgl;
    const t = time - timeStart;
    const alpha = smooth(clamp01(t / FADE)) * clamp01((LIFE - t) / FADE);
    if (alpha <= 0.004) return;

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
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);
    mgl.bindTexture(this.tex);

    const s = this.spec;
    const drift = t * 0.012;                 // slow leftward settle
    const cx = s.x * ASPECT - drift;
    const cy = s.y;
    const nLines = s.lines.length;
    const first = NAME[s.lines[0]];

    mgl.begin(mgl.QUADS);
    // role label, small italic, above and left-aligned with the first line
    const rb = ROLE[s.role];
    const rh = CAP * 0.36;
    const topY = cy + (nLines - 1) * CAP * 0.62;
    this._quad(rb, cx - first.aspect * CAP / 2 + rh * rb.aspect / 2,
               topY + CAP / 2 + rh * 0.75, rh, alpha * 0.8);
    // the name, one box per line
    s.lines.forEach((name, i) => {
      const b = NAME[name];
      const y = cy + (nLines - 1 - i * 2) * CAP * 0.62;
      this._quad(b, cx + ((s.dx && s.dx[i]) || 0) * ASPECT, y, CAP, alpha);
    });
    mgl.end();

    // the thin rule the release draws across the frame
    mgl.enableTexture(false);
    mgl.begin(mgl.QUADS);
    mgl.color4(0.78, 0.74, 0.68, alpha * 0.45);
    mgl.vertex3(-ASPECT, RULE_Y + 0.003, 0);
    mgl.vertex3(ASPECT, RULE_Y + 0.003, 0);
    mgl.vertex3(ASPECT, RULE_Y - 0.003, 0);
    mgl.vertex3(-ASPECT, RULE_Y - 0.003, 0);
    mgl.end();
    mgl.enableTexture(true);
  }
}
