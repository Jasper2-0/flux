// Rough-cut fillers: quick approximations of the parts that are not yet
// properly reverse-engineered, so the timeline plays as a continuous show.
// Everything here is reconstruction-by-eye from the release capture and is
// labeled "rough cut" in the readout; it will be replaced part by part.

function fullQuad(mgl, z, u0 = 0, v0 = 0, u1 = 1, v1 = 1, w = 4 / 3, h = 1) {
  mgl.begin(mgl.QUADS);
  mgl.texCoord2(u0, v0); mgl.vertex3(-w, h, z);
  mgl.texCoord2(u1, v0); mgl.vertex3(w, h, z);
  mgl.texCoord2(u1, v1); mgl.vertex3(w, -h, z);
  mgl.texCoord2(u0, v1); mgl.vertex3(-w, -h, z);
  mgl.end();
}

function ortho43(mgl) {
  mgl.matrixMode(mgl.PROJECTION);
  mgl.loadIdentity();
  mgl.ortho(-4 / 3, 4 / 3, -1, 1, -1, 1);
  mgl.matrixMode(mgl.MODELVIEW);
  mgl.loadIdentity();
}

const clamp01 = (v) => (v < 0 ? 0 : v > 1 ? 1 : v);

// 0–12 s: the white shatter burst of the intro (very loose approximation)
export class IntroBurst {
  constructor(mgl) { this.mgl = mgl; }
  do(time, timeStart) {
    const mgl = this.mgl, t = time - timeStart;
    ortho43(mgl);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    mgl.enableCullFace(false);
    mgl.enableTexture(false);
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);
    const boom = clamp01((t - 1.2) * 2) * clamp01((8 - t) * 0.5);
    if (boom <= 0) return;
    for (let i = 0; i < 22; i++) {
      const a = i * 2.85 + Math.sin(i * 12.9898) * 6.2 + t * (i % 3 === 0 ? 0.35 : -0.22);
      const len = (0.4 + ((i * 7919) % 97) / 97) * (0.6 + t * 0.35);
      const wdt = 0.02 + ((i * 104729) % 31) / 31 * 0.13;
      mgl.color4(1, 1, 1, (0.25 + ((i * 13) % 7) / 7 * 0.5) * boom);
      const ca = Math.cos(a), sa = Math.sin(a);
      mgl.begin(mgl.TRIANGLES);
      mgl.vertex3(0, 0, 0);
      mgl.vertex3(ca * len - sa * wdt, sa * len + ca * wdt, 0);
      mgl.vertex3(ca * len + sa * wdt, sa * len - ca * wdt, 0);
      mgl.end();
    }
  }
}

// 12–41 s: the credit backgrounds crossfading on their grid
export class CreditsRough {
  constructor(mgl, tex) {
    this.mgl = mgl;
    this.bks = [1, 2, 3, 4, 5].map((n) => tex.loadTexture('data/credits/credbk-' + n + '-image.jpg'));
  }
  do(time, timeStart) {
    const mgl = this.mgl, t = time - timeStart;
    const seg = t / 5.8; // five backgrounds across ~29 s
    const i = Math.min(this.bks.length - 1, Math.floor(seg));
    const frac = seg - i;
    ortho43(mgl);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    mgl.enableCullFace(false);
    mgl.enableTexture(true);
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE_MINUS_SRC_ALPHA);
    const zoom = 1 + 0.04 * Math.sin(t * 0.3);
    const fadeIn = clamp01(t * 0.7) * clamp01((29 - t) * 0.7);
    mgl.bindTexture(this.bks[i]);
    mgl.color4(fadeIn, fadeIn, fadeIn, 1);
    fullQuad(mgl, 0, 0, 0, 1, 1, (4 / 3) * zoom, zoom);
    if (frac > 0.72 && i + 1 < this.bks.length) {
      mgl.bindTexture(this.bks[i + 1]);
      mgl.color4(1, 1, 1, clamp01((frac - 0.72) / 0.28) * fadeIn);
      fullQuad(mgl, 0, 0, 0, 1, 1, (4 / 3) * zoom, zoom);
    }
  }
}

// 41.5–52 s: the contour logo over rippling copper
export class ContLogoRough {
  constructor(mgl, tex) {
    this.mgl = mgl;
    this.bg = tex.loadTexture('data/saftext/fx8.jpg');
    this.logo = tex.loadTexture('data/contourlogo-overlay-image01.jpg');
  }
  do(time, timeStart) {
    const mgl = this.mgl, t = time - timeStart;
    ortho43(mgl);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    mgl.enableCullFace(false);
    mgl.enableTexture(true);
    mgl.enableBlend(false);
    const s = 0.04 * t;
    mgl.bindTexture(this.bg);
    const dim = 0.65 + 0.1 * Math.sin(t * 1.7);
    mgl.color4(dim, dim * 0.92, dim * 0.8, 1);
    fullQuad(mgl, 0, s, s * 0.6, 1.3 + s, 1 + s * 0.6);
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);
    mgl.bindTexture(this.logo);
    const a = clamp01(t * 0.8) * clamp01((10.5 - t) * 0.8);
    mgl.color4(a, a, a, 1);
    fullQuad(mgl, 0, 0, 0, 1, 1, 1, 0.75);
  }
}

// 108.5–174 s: minimal infinite zoom over the ten plates (quarter-scale,
// centred, per the measured nesting), ending in the as-shipped unaligned
// Shun fade. The polished standalone recreation supersedes this.
export class ZoomerLite {
  constructor(mgl, tex) {
    this.mgl = mgl;
    this.plates = [];
    for (let i = 0; i <= 9; i++) {
      this.plates.push(tex.loadTexture('data/zoomer/image' + i + 'a.jpg'));
    }
    this.shun = tex.loadTexture('data/zoomer/shun-compo.jpg');
  }
  do(time, timeStart) {
    const mgl = this.mgl, t = time - timeStart;
    ortho43(mgl);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    mgl.enableCullFace(false);
    mgl.enableTexture(true);
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE_MINUS_SRC_ALPHA);

    // u = how many plate-steps deep we are; decelerates to rest on plate 9
    const span = 46;
    const p = Math.min(1, t / span);
    const u = -0.55 + (9.0 + 0.55) * (1 - Math.pow(1 - p, 1.6));
    const rot = t * 1.1; // gentle drift, as shipped

    for (let k = 0; k <= 9; k++) {
      const scale = Math.pow(4, u - k);
      if (scale < 0.02 || scale > 24) continue;
      const alpha = clamp01((scale - 0.02) * 30);
      mgl.pushMatrix();
      mgl.rotate(rot, 0, 0, 1);
      mgl.scale(scale, scale, 1);
      mgl.bindTexture(this.plates[k]);
      mgl.color4(1, 1, 1, alpha);
      fullQuad(mgl, 0, 0, 0, 1, 1, 1, 1);
      mgl.popMatrix();
    }

    // as-shipped ending: Shun fades up in place, unaligned
    const shunA = clamp01((t - span - 2) * 0.35);
    if (shunA > 0) {
      mgl.bindTexture(this.shun);
      mgl.color4(1, 1, 1, shunA);
      fullQuad(mgl, 0, 0, 0, 1, 1, 4 / 3, 1);
    }
  }
}

// 174–192 s: the closing card
export class EndCard {
  constructor(mgl, tex) {
    this.mgl = mgl;
    this.logo = tex.loadTexture('data/logo.jpg');
  }
  do(time, timeStart) {
    const mgl = this.mgl, t = time - timeStart;
    ortho43(mgl);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    mgl.enableCullFace(false);
    mgl.enableTexture(true);
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE_MINUS_SRC_ALPHA);
    const a = clamp01((t - 0.5) * 0.6) * clamp01((17.5 - t) * 0.8);
    mgl.bindTexture(this.logo);
    mgl.color4(a, a, a, 1);
    fullQuad(mgl, 0, 0, 0, 1, 1, 1, 0.75);
  }
}
