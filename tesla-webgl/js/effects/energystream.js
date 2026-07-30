// EnergyStream (87 - 145s): the long middle part — a fan of huge scrolling
// textured planes rotating around the X axis with linear fog, plus 16 streams
// of billboarded flares racing along the X axis.

import { rand, RAND_MAX, clamp01, Vec3 } from '../mathlib.js';

const CHANGE_TIME = 15;
const CHANGE_TIME1 = 25;
const CHANGE_TIME2 = 38;
const CHANGE_TIME3 = 58;

function fmod(a, b) {
  // C fmod keeps the sign of the dividend
  return a - b * Math.trunc(a / b);
}

class FlareStream {
  constructor(seed, tex, basePos, speed) {
    this.tex = tex;
    this.speed = speed;
    this.flares = new Float32Array(seed * 3);
    this.nFlares = seed;
    for (let i = 0; i < seed; i++) {
      this.flares[i * 3] = -800.0 * rand() / RAND_MAX - 1150 + basePos.x;
      this.flares[i * 3 + 1] = 10.0 * rand() / RAND_MAX - 20 + basePos.y;
      this.flares[i * 3 + 2] = 10.0 * rand() / RAND_MAX - 20 + basePos.z;
    }
  }

  render(mgl, time, baseX, baseY, alpha) {
    if (time < CHANGE_TIME) return;
    const t = time - CHANGE_TIME;

    mgl.color4(1, 1, 1, alpha);
    mgl.bindTexture(this.tex);

    mgl.begin(mgl.QUADS);

    let multiplier = 1;
    if (time > CHANGE_TIME1) {
      multiplier = time > CHANGE_TIME2 ? 2.5 : 2;
    }

    const pos = new Vec3();
    for (let i = 0; i < this.nFlares; i++) {
      pos.x = fmod(this.flares[i * 3] + t * this.speed * multiplier, 800) - 400;
      pos.y = this.flares[i * 3 + 1] + 2 * Math.sin(t * 7 + this.flares[i * 3]);
      pos.z = this.flares[i * 3 + 2] + 2 * Math.cos(t * 7 + i * 3.14);

      mgl.texCoord2(0, 0);
      mgl.vertex3(pos.x - baseX.x + baseY.x, pos.y - baseX.y + baseY.y, pos.z - baseX.z + baseY.z);
      mgl.texCoord2(1, 0);
      mgl.vertex3(pos.x + baseX.x + baseY.x, pos.y + baseX.y + baseY.y, pos.z + baseX.z + baseY.z);
      mgl.texCoord2(1, 1);
      mgl.vertex3(pos.x + baseX.x - baseY.x, pos.y + baseX.y - baseY.y, pos.z + baseX.z - baseY.z);
      mgl.texCoord2(0, 1);
      mgl.vertex3(pos.x - baseX.x - baseY.x, pos.y - baseX.y - baseY.y, pos.z - baseX.z - baseY.z);
    }

    mgl.end();
  }
}

export class EnergyStream {
  constructor(mgl, tex) {
    this.mgl = mgl;
    this.tex1 = tex.loadTexture('data/textures/y7.jpg');
    this.tex2 = tex.loadTexture('data/textures/y6.jpg');
    const flareTex = tex.loadTexture('data/textures/flare02.jpg');

    const V = (x, y, z) => new Vec3(x, y, z);
    const config = [
      [V(0, 50, 0), 300], [V(0, 0, 0), 150], [V(0, 90, 60), 250], [V(0, -100, 30), 160],
      [V(0, 50, -100), 340], [V(0, -50, 50), 270], [V(0, 100, 50), 180], [V(0, -30, 90), 130],
      [V(0, 150, 10), 200], [V(0, 100, -100), 210], [V(0, 190, 160), 220], [V(0, -200, 130), 230],
      [V(0, 150, -200), 240], [V(0, -150, 250), 160], [V(0, 200, 150), 230], [V(0, -130, 190), 250],
    ];
    this.streams = config.map(([pos, speed]) => new FlareStream(150, flareTex, pos, speed));
  }

  _putQuad(u, v) {
    const mgl = this.mgl;
    mgl.begin(mgl.QUADS);
    mgl.texCoord2(u, v); mgl.vertex3(-300, 300, 0);
    mgl.texCoord2(1 + u, v); mgl.vertex3(300, 300, 0);
    mgl.texCoord2(1 + u, 1 + v); mgl.vertex3(300, -300, 0);
    mgl.texCoord2(u, 1 + v); mgl.vertex3(-300, -300, 0);
    mgl.end();
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

    mgl.translate(0, 0, -300);
    mgl.rotate(t * 30, 1, 0, 0);
    mgl.rotate(30 * Math.sin(t / 3) + 10, 0, 0, 1);

    if (t > CHANGE_TIME1) {
      mgl.rotate(t > CHANGE_TIME2 ? 90 : 180, 0, 1, 0);
    }

    mgl.enableFog(true);
    mgl.fog(200, 500);

    mgl.pushMatrix();

    let uvU = 0, uvV = 0;
    let alpha = 1;

    if (t > CHANGE_TIME) {
      uvU = 0.1 * Math.sin(t);
      uvV = t / 5;
      mgl.bindTexture(this.tex2);
      alpha = clamp01(t - CHANGE_TIME);

      if (t > CHANGE_TIME1) {
        alpha = clamp01(alpha * (t - CHANGE_TIME1));
        mgl.rotate(90, 0, 1, 0);

        if (t > CHANGE_TIME2) {
          alpha = clamp01(alpha * (t - CHANGE_TIME2));
          if (t > CHANGE_TIME3) {
            alpha = clamp01(alpha * (1 + CHANGE_TIME3 - t));
          }
        } else {
          alpha = clamp01(alpha * (CHANGE_TIME2 - t));
        }
      } else {
        alpha = clamp01(alpha * (CHANGE_TIME1 - t));
      }

      mgl.color4(1, 1, 1, alpha);
    } else {
      mgl.bindTexture(this.tex1);
      alpha = clamp01((CHANGE_TIME - t) * 0.5);
      mgl.color4(1, 1, 1, (Math.sin(t * 2) * 0.25 + 0.75) * alpha);
    }

    if (t > CHANGE_TIME) {
      for (let i = 0; i < 8; i++) {
        this._putQuad(uvU, uvV);
        mgl.rotate(180 / 7, 1, 0, 0);
      }
    } else {
      for (let i = 0; i < 16; i++) {
        this._putQuad(uvU, uvV);
        mgl.rotate(180 / 15, 1, 0, 0);
      }
    }

    mgl.popMatrix();

    const camera = mgl.getModelView();
    camera.inverse();
    const baseX = camera.baseX().mulSelf(10);
    const baseY = camera.baseY().mulSelf(10);

    for (const stream of this.streams) {
      stream.render(mgl, t, baseX, baseY, alpha);
    }

    mgl.enableFog(false);
  }
}
