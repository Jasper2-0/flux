// anim — the 2D sprite overlays.
//
// Six of these run on top of the 3D sections. The plugin's parameters,
// verbatim from its own registration table: `tlx`/`tly` and `brx`/`bry`
// are the picture's top-left and bottom-right on screen, `tlu`/`tlv` and
// `bru`/`brv` the same in texture space, `zpos` its depth, `colorr/g/b/a`
// its tint, and `speed` the playback rate. `filename` names a text file
// listing the frames — four .pcx images per animation, some listed twice
// so the loop runs through them again.
//
// The script only ever sets tlx, tly, layer, zpos and speed 12, so the
// frame rate is 12 fps and the size comes from the image itself.
//
// Reconstructed: that screen coordinates are 640x480 pixels (which is
// what backgr asserts its pictures are), and that the sprite is drawn
// with its black background keyed out — the frames are white line art on
// black, and additive blending is what makes them read as overlays.

const SCREEN_W = 640, SCREEN_H = 480;
const ASPECT = SCREEN_W / SCREEN_H;

export class Anim {
  constructor(ctx, inst) {
    this.ctx = ctx;
    this.inst = inst;
    this.ready = false;
    this.frames = [];
  }

  async load() {
    const file = this.inst.get('filename');
    if (!file) return;
    const path = String(file).replace(/\\/g, '/').replace(/^\.\//, '').toLowerCase();
    const list = await this.ctx.assets.textLines(path);
    this.frames = await Promise.all(
      list.map((p) => this.ctx.assets.texture(p.replace(/\.pcx$/, '.png'), { mipmap: false })));
    const img = await this.ctx.assets.image(list[0].replace(/\.pcx$/, '.png'));
    this.w = img.naturalWidth;
    this.h = img.naturalHeight;
    this.ready = this.frames.length > 0;
  }

  draw(local) {
    if (!this.frames.length) return;
    const mgl = this.ctx.mgl;
    const inst = this.inst;
    const fps = inst.num('speed', 12) || 12;
    const tex = this.frames[Math.floor(local * fps) % this.frames.length];

    const tlx = inst.num('tlx', 0), tly = inst.num('tly', 0);
    const brx = inst.num('brx', tlx + this.w), bry = inst.num('bry', tly + this.h);
    const tlu = inst.num('tlu', 0), tlv = inst.num('tlv', 0);
    const bru = inst.num('bru', 1), brv = inst.num('brv', 1);
    const r = inst.num('colorr', 1), g = inst.num('colorg', 1);
    const b = inst.num('colorb', 1), a = inst.num('colora', 1);

    // pixel space -> the ortho frame
    const sx = (x) => (x / SCREEN_W) * 2 * ASPECT - ASPECT;
    const sy = (y) => 1 - (y / SCREEN_H) * 2;

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.ortho(-ASPECT, ASPECT, -1, 1, -1, 1);
    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    mgl.enableTexture(true);
    mgl.bindTexture(tex);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    mgl.enableCullFace(false);
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);

    const quad = [
      [sx(tlx), sy(tly), tlu, tlv],
      [sx(brx), sy(tly), bru, tlv],
      [sx(brx), sy(bry), bru, brv],
      [sx(tlx), sy(bry), tlu, brv],
    ];
    mgl.begin(mgl.TRIANGLES);
    for (const i of [0, 1, 2, 0, 2, 3]) {
      const [x, y, u, v] = quad[i];
      mgl.color4(r, g, b, a);
      mgl.texCoord2(u, v);
      mgl.vertex3(x, y, 0);
    }
    mgl.end();
  }
}
