// scid/Letters — the poem renderer.
//
// Everything here except the presentation constants comes from the
// executable: each instance's parameter block carries the text, a start
// and end position, a scale, a duration and (for the title card) a
// fade-in window. Glyphs are cut from letters/abc.jpg, whose proportional
// boxes were measured from the artwork and shipped in data/font-abc.json.
//
// Coordinate space: the parameter positions are fractions of the half
// frame, x∈[-1,1] across the full 4:3 width, y∈[-1,1] over its height,
// and the text is centred on that point. That reading is what puts
// "YOUR VOICE" clipped against the left edge and "THE LIGHT THAT DEPARTS
// AND RETURNS" clipped on both, exactly as the release capture shows.
//
// Reconstructed (not from the binary): the cap height per unit of scale,
// letter tracking, and the hold/fade envelope around `duration`.

const ASPECT = 4 / 3;
const CAP = 0.9;         // cap height in y-units per unit of `scale`
const W_MAX = 2.6;       // long lines are scaled down to about this width
const TRACK = 1.06;      // advance multiplier between glyphs
const FADE_IN = 0.45;
const FADE_OUT = 0.9;
const HOLD = 2.2;        // beyond `duration`, matched to the capture

const clamp01 = (v) => (v < 0 ? 0 : v > 1 ? 1 : v);
const smooth = (x) => x * x * (3 - 2 * x);

export class Letters {
  // params: the decoded `letters` block from timeline.json
  constructor(mgl, tex, font, params) {
    this.mgl = mgl;
    this.tex = tex.loadTexture('data/letters/abc.jpg');
    this.font = font;
    this.p = params;

    // The capture shows short lines set large (and clipped at the frame
    // edge) while long ones are scaled down to roughly fit — so the block's
    // `scale` reads as a maximum, with a width budget below it.
    const chars = [...params.text.toUpperCase()];
    let sumW = 0;
    for (const ch of chars) sumW += (this.font[ch] || this.font[' ']).w;
    const cap = Math.min(params.scale * CAP, W_MAX / (sumW * TRACK));
    this.cap = cap;

    // lay the string out once: per-glyph x offset and width, in y-units
    this.glyphs = [];
    let x = 0;
    for (const ch of chars) {
      const g = this.font[ch] || this.font[' '];
      const w = g.w * cap;
      if (ch !== ' ') this.glyphs.push({ g, x, w });
      x += w * TRACK;
    }
    this.width = x;
    this.life = (params.type === 2 ? 4.6 : params.duration + HOLD) + FADE_IN + FADE_OUT;
    this.duration = params.type === 2 ? params.fade0 + this.life : this.life;
  }

  do(time, timeStart) {
    const mgl = this.mgl;
    const p = this.p;
    let t = time - timeStart;

    // the title card carries its own fade-in window in the parameter block
    let alpha;
    if (p.type === 2) {
      if (t < p.fade0) return;
      alpha = clamp01((t - p.fade0) / Math.max(0.05, p.fade1 - p.fade0)) *
              clamp01((p.fade0 + this.life - t) / FADE_OUT);
    } else {
      alpha = smooth(clamp01(t / FADE_IN)) * clamp01((this.life - t) / FADE_OUT);
    }
    if (alpha <= 0.004) return;

    // drift from the start position to the end position across its life
    const k = clamp01(t / this.life);
    const cx = (p.x0 + (p.x1 - p.x0) * k) * ASPECT;
    const cy = p.y0 + (p.y1 - p.y0) * k;

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
    // the type reads as light laid over the picture, not paint
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);
    mgl.bindTexture(this.tex);
    mgl.color4(1, 1, 1, alpha);

    const left = cx - this.width / 2;
    const top = cy + this.cap / 2;
    mgl.begin(mgl.QUADS);
    for (const { g, x, w } of this.glyphs) {
      const x0 = left + x, x1 = x0 + w;
      mgl.texCoord2(g.u0, g.v0); mgl.vertex3(x0, top, 0);
      mgl.texCoord2(g.u1, g.v0); mgl.vertex3(x1, top, 0);
      mgl.texCoord2(g.u1, g.v1); mgl.vertex3(x1, top - this.cap, 0);
      mgl.texCoord2(g.u0, g.v1); mgl.vertex3(x0, top - this.cap, 0);
    }
    mgl.end();
  }
}
