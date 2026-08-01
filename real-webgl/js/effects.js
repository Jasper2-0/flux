// xotrack's `show*` tasks, which live in Real.exe rather than in the
// engine DLLs. Real.exe is 64 KB and registers eight of them against
// Red's task table, each with its own vtable: slot 0 is the one-time
// init, slot 1 the per-frame run.
//
// They all draw in the same screen-pixel ortho frame the 2D layers use,
// and they take their clock from the message's task-local elapsed
// milliseconds — the same `[msg+0x48]` that drives drawScene.

const SCREEN_W = 640, SCREEN_H = 480;

// The effects seed themselves from srand(time(NULL)), so the exact line
// placement was never reproducible between runs of the original either.
// What has to match is the distribution, and the shape of the expression
// each value goes through: `(rand() * K) >> 15`, i.e. uniform over [0, K).
function rnd(k) {
  return Math.floor(Math.random() * k);
}

// showLines — 0x402c60 (init) and 0x402e80 (run).
//
// Two modes, chosen by the first parameter. Both draw additively with
// texturing off, which is why they read as glow rather than as geometry.
//
//   "0"  sixteen full-width bands, each riding a slow sine:
//          y = sin((t + jitter) * freq) * amp + centre
//        with its own grey level and a height of five to nine pixels.
//
//   "1"  six bright lines that jump to new random parameters at the top
//        of every 172 ms — the second parameter picks vertical (x from
//        a 320-pixel wrap) or horizontal (y through fmod 240).
export class ShowLines {
  constructor(inst) {
    this.mode = String(inst.args[0] || '0');
    this.sub = String(inst.args[1] || '-');
    this.bands = [];
    for (let i = 0; i < 16; i++) {
      this.bands.push({
        amp: rnd(4000) * 0.1,
        centre: rnd(1000) - 400,
        grey: rnd(64) / 255 + 0.25,
        freq: rnd(5000) * 5e-7,
        height: rnd(5) + 5,
        jitter: rnd(100),
      });
    }
    this.bars = [];
    this.phase = 0;
    this._reseed();
  }

  _reseed() {
    this.phase = rnd(100);
    this.bars = [];
    for (let i = 0; i < 6; i++) {
      this.bars.push({
        amp: rnd(4000) * 0.1,
        offset: Math.trunc(rnd(500) - 400),
        freq: rnd(5000) * 5e-7,
      });
    }
  }

  draw(mgl, ms) {
    const gl = mgl.gl;
    mgl.enableTexture(false);
    mgl.blendFunc(gl.ONE, gl.ONE);
    mgl.enableBlend(true);

    if (this.mode === '0') {
      mgl.begin(mgl.TRIANGLES);
      for (const b of this.bands) {
        const y = Math.sin((ms + b.jitter) * b.freq) * b.amp + b.centre;
        const c = b.grey;
        const quad = [[0, y], [SCREEN_W - 1, y],
                      [SCREEN_W - 1, y + b.height], [0, y + b.height]];
        for (const i of [0, 1, 2, 0, 2, 3]) {
          mgl.color4(c, c, c, 1);
          mgl.vertex3(quad[i][0], quad[i][1], 0);
        }
      }
      mgl.end();
      return;
    }

    // mode 1: new parameters at the top of every 172 ms window
    if (ms % 172 < 100) this._reseed();
    mgl.begin(mgl.LINES);
    for (const b of this.bars) {
      const d = Math.sin((ms + this.phase) * b.freq) * b.amp;
      mgl.color4(1, 1, 1, 1);
      if (this.sub === '0') {
        const x = d + (b.offset % 320) + 1;
        mgl.vertex3(x, 0, 0);
        mgl.vertex3(x, SCREEN_H - 1, 0);
      } else {
        let y = (d + b.offset) % 240;
        mgl.vertex3(0, y, 0);
        mgl.vertex3(SCREEN_W - 1, y, 0);
      }
    }
    mgl.end();
  }
}

export const EFFECTS = { showlines: ShowLines };
