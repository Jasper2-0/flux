// backgr — a full-frame background picture.
//
// The plugin's own help text: "Shows a background picture (.jpg) of
// 640x480", with `filename`, `alphaname` (a .pcx mask) and `zpos`. It
// asserts the picture is exactly 640x480 and blits it over the frame.
//
// Reconstructed: nothing beyond the choice to draw it as a screen-space
// quad with the mask in the alpha channel.

const ASPECT = 4 / 3;

export class Backgr {
  constructor(ctx, inst) {
    this.ctx = ctx;
    this.inst = inst;
    this.ready = false;
  }

  async load() {
    const file = this.inst.get('filename');
    if (!file) return;
    const alpha = this.inst.get('alphaname');
    const path = (p) => String(p).replace(/\\/g, '/').replace(/^\.\//, '').toLowerCase();
    this.tex = await this.ctx.assets.texture(
      path(file), { alpha: alpha ? path(alpha).replace(/\.pcx$/, '.png') : null, mipmap: false });
    this.hasAlpha = !!alpha;
    this.ready = true;
  }

  draw() {
    const mgl = this.ctx.mgl;
    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.ortho(-ASPECT, ASPECT, -1, 1, -1, 1);
    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    mgl.enableTexture(true);
    mgl.bindTexture(this.tex);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    mgl.enableCullFace(false);
    if (this.hasAlpha) {
      mgl.enableBlend(true);
      mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE_MINUS_SRC_ALPHA);
    } else {
      mgl.enableBlend(false);
    }

    mgl.begin(mgl.TRIANGLES);
    const quad = [[-ASPECT, 1, 0, 0], [ASPECT, 1, 1, 0], [ASPECT, -1, 1, 1], [-ASPECT, -1, 0, 1]];
    for (const i of [0, 1, 2, 0, 2, 3]) {
      const [x, y, u, v] = quad[i];
      mgl.color4(1, 1, 1, 1);
      mgl.texCoord2(u, v);
      mgl.vertex3(x, y, 0);
    }
    mgl.end();
  }
}
