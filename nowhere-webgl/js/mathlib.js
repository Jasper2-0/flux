// Math helpers for the Tesla WebGL port.
// Mat4 mirrors the column-major float[16] layout the original demo used with
// OpenGL (glGetFloatv/GL_MODELVIEW_MATRIX), including its affine Inverse().

export const DEG2RAD = Math.PI / 180;

export class Vec3 {
  constructor(x = 0, y = 0, z = 0) { this.x = x; this.y = y; this.z = z; }
  set(x, y, z) { this.x = x; this.y = y; this.z = z; return this; }
  copy(v) { this.x = v.x; this.y = v.y; this.z = v.z; return this; }
  clone() { return new Vec3(this.x, this.y, this.z); }
  add(v) { return new Vec3(this.x + v.x, this.y + v.y, this.z + v.z); }
  sub(v) { return new Vec3(this.x - v.x, this.y - v.y, this.z - v.z); }
  mul(s) { return new Vec3(this.x * s, this.y * s, this.z * s); }
  addSelf(v) { this.x += v.x; this.y += v.y; this.z += v.z; return this; }
  mulSelf(s) { this.x *= s; this.y *= s; this.z *= s; return this; }
  dot(v) { return this.x * v.x + this.y * v.y + this.z * v.z; }
  normalize() {
    const len = 1 / Math.sqrt(this.x * this.x + this.y * this.y + this.z * this.z);
    this.x *= len; this.y *= len; this.z *= len;
    return this;
  }
}

// clamp() from the original inline3d.hpp: clamp float to [0, 1]
export function clamp01(v) {
  return v < 0 ? 0 : (v > 1 ? 1 : v);
}

export class Mat4 {
  constructor() {
    this.m = new Float32Array(16);
    this.identity();
  }

  identity() {
    const m = this.m;
    m.fill(0);
    m[0] = m[5] = m[10] = m[15] = 1;
    return this;
  }

  copy(other) { this.m.set(other.m); return this; }
  clone() { const r = new Mat4(); r.m.set(this.m); return r; }

  // this = this * b (right-multiply, like glMultMatrix)
  mult(b) {
    const a = this.m, bm = b.m ? b.m : b;
    const r = new Float32Array(16);
    for (let col = 0; col < 4; col++) {
      for (let row = 0; row < 4; row++) {
        r[col * 4 + row] =
          a[0 * 4 + row] * bm[col * 4 + 0] +
          a[1 * 4 + row] * bm[col * 4 + 1] +
          a[2 * 4 + row] * bm[col * 4 + 2] +
          a[3 * 4 + row] * bm[col * 4 + 3];
      }
    }
    this.m.set(r);
    return this;
  }

  translate(x, y, z) {
    const m = this.m;
    m[12] += m[0] * x + m[4] * y + m[8] * z;
    m[13] += m[1] * x + m[5] * y + m[9] * z;
    m[14] += m[2] * x + m[6] * y + m[10] * z;
    m[15] += m[3] * x + m[7] * y + m[11] * z;
    return this;
  }

  scale(x, y, z) {
    const m = this.m;
    for (let i = 0; i < 4; i++) {
      m[i] *= x; m[4 + i] *= y; m[8 + i] *= z;
    }
    return this;
  }

  // glRotatef(angle in degrees, axis x/y/z)
  rotate(angleDeg, x, y, z) {
    const a = angleDeg * DEG2RAD;
    const len = Math.sqrt(x * x + y * y + z * z);
    if (len === 0) return this;
    x /= len; y /= len; z /= len;
    const c = Math.cos(a), s = Math.sin(a), t = 1 - c;
    const rot = new Float32Array([
      t * x * x + c,     t * x * y + s * z, t * x * z - s * y, 0,
      t * x * y - s * z, t * y * y + c,     t * y * z + s * x, 0,
      t * x * z + s * y, t * y * z - s * x, t * z * z + c,     0,
      0, 0, 0, 1,
    ]);
    return this.mult(rot);
  }

  frustum(l, r, b, t, n, f) {
    const m = new Float32Array(16);
    m[0] = 2 * n / (r - l);
    m[5] = 2 * n / (t - b);
    m[8] = (r + l) / (r - l);
    m[9] = (t + b) / (t - b);
    m[10] = -(f + n) / (f - n);
    m[11] = -1;
    m[14] = -2 * f * n / (f - n);
    return this.mult(m);
  }

  ortho(l, r, b, t, n, f) {
    const m = new Float32Array(16);
    m[0] = 2 / (r - l);
    m[5] = 2 / (t - b);
    m[10] = -2 / (f - n);
    m[12] = -(r + l) / (r - l);
    m[13] = -(t + b) / (t - b);
    m[14] = -(f + n) / (f - n);
    m[15] = 1;
    return this.mult(m);
  }

  // Affine inverse, ported verbatim from the demo's CMatrix::Inverse().
  inverse() {
    const m = this.m;
    const a = m[0], b = m[1], c = m[2];
    const d = m[4], e = m[5], f = m[6];
    const g = m[8], h = m[9], i = m[10];
    const j = m[12], k = m[13], l = m[14];

    const w = 1.0 / (a * (e * i - f * h) - (b * (d * i - f * g) + c * (e * g - d * h)));

    m[0] = (e * i - f * h) * w;
    m[1] = (c * h - b * i) * w;
    m[2] = (b * f - c * e) * w;

    m[4] = (f * g - d * i) * w;
    m[5] = (a * i - c * g) * w;
    m[6] = (c * d - a * f) * w;

    m[8] = (d * h - e * g) * w;
    m[9] = (b * g - a * h) * w;
    m[10] = (a * e - b * d) * w;

    m[12] = (e * (g * l - i * j) + f * (h * j - g * k) - d * (h * l - i * k)) * w;
    m[13] = (a * (h * l - i * k) + b * (i * j - g * l) + c * (g * k - h * j)) * w;
    m[14] = (b * (d * l - f * j) + c * (e * j - d * k) - a * (e * l - f * k)) * w;
    return this;
  }

  // Column bases, as the demo's CMatrix::stBaseX/Y/Z (float[16] col-major).
  baseX() { const m = this.m; return new Vec3(m[0], m[1], m[2]); }
  baseY() { const m = this.m; return new Vec3(m[4], m[5], m[6]); }
  baseZ() { const m = this.m; return new Vec3(m[8], m[9], m[10]); }
  setBaseW(x, y, z, w) { const m = this.m; m[12] = x; m[13] = y; m[14] = z; m[15] = w; return this; }

  // CMatrix::operator*(CVector): rotate + translate
  mulPoint(v, out = new Vec3()) {
    const m = this.m;
    const x = v.x, y = v.y, z = v.z;
    out.x = x * m[0] + y * m[4] + z * m[8] + m[12];
    out.y = x * m[1] + y * m[5] + z * m[9] + m[13];
    out.z = x * m[2] + y * m[6] + z * m[10] + m[14];
    return out;
  }

  // CMatrix::operator<<(CVector): rotate only
  mulDir(v, out = new Vec3()) {
    const m = this.m;
    const x = v.x, y = v.y, z = v.z;
    out.x = x * m[0] + y * m[4] + z * m[8];
    out.y = x * m[1] + y * m[5] + z * m[9];
    out.z = x * m[2] + y * m[6] + z * m[10];
    return out;
  }
}

// MSVC-compatible rand(): matches the LCG the original demo linked against.
export const RAND_MAX = 32767;
let randSeed = 1;
export function srand(seed) { randSeed = seed | 0; }
export function rand() {
  randSeed = (Math.imul(randSeed, 214013) + 2531011) | 0;
  return (randSeed >> 16) & 0x7fff;
}

// CSinPulse / CSinWave from sinus.hpp
export class SinPulse {
  constructor(min, max, freq, f0) { this.min = min; this.max = max; this.freq = freq; this.f0 = f0; }
  calc(t) {
    return (this.max - this.min) * 0.5 * Math.sin(this.freq * t + this.f0) + (this.max + this.min) * 0.5;
  }
}
export class SinWave {
  constructor(amp, freq, f0) { this.amp = amp; this.freq = freq; this.f0 = f0; }
  calc(t) { return this.amp * Math.sin(this.freq * t + this.f0); }
}
