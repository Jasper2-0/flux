// Energy3D scene runtime — plays one .i3d at a frame.
//
// The exporter is a 3ds Max plug-in, so the animation is Kochanek–Bartels:
// every key carries tension, continuity and bias plus ease-to and
// ease-from. The engine's own ease (spline_tcb::Ease) is reused for the
// reparameterisation inside a span.
//
// Timing comes straight out of energy3d_scene::update_frame (0x10011aa0):
//
//     frame = elapsed_ms * 0.03 + offset,   wrapped into [0, frameEnd)
//
// — a hardcoded 30 fps, which is also what every scene's own header says.
//
// The camera is ogl_camera::calculate (0x10002dc0):
//
//     gluPerspective(fov, 4/3, 2, 5000)
//     glRotatef(roll, 0, 0, 1)
//     gluLookAt(position, target, (0,1,0))
//
// Note that Max stores a *horizontal* field of view and this hands it
// straight to gluPerspective, which wants a vertical one. That is the
// original's own arithmetic and is reproduced rather than corrected.

import { Mat4, DEG2RAD } from './mathlib.js';

export const SCENE_FPS = 30;
export const ASPECT = 4 / 3;
export const NEAR = 2, FAR = 5000;

// spline_tcb::Ease, the same curve the script's keyframes use.
function ease(u, a, b) {
  const s = a + b;
  if (s === 0) return u;
  if (s > 1) { a /= s; b /= s; }
  const k = 1 / (2 - a - b);
  if (u < a) return k * u * u / a;
  if (u < 1 - b) return k * (2 * u - a);
  const w = 1 - u;
  return 1 - k * w * w / b;
}

function kbTangents(keys, i, dim) {
  const n = keys.length, k = keys[i];
  const [t, c, b] = k.tcb;
  const prev = keys[i > 0 ? i - 1 : 0], next = keys[i < n - 1 ? i + 1 : n - 1];
  const dOut = [], dIn = [];
  for (let j = 0; j < dim; j++) {
    const p = prev.v[j], cu = k.v[j], nx = next.v[j];
    dOut.push(0.5 * (1 - t) * ((1 + b) * (1 + c) * (cu - p) + (1 - b) * (1 - c) * (nx - cu)));
    dIn.push(0.5 * (1 - t) * ((1 + b) * (1 - c) * (cu - p) + (1 - b) * (1 + c) * (nx - cu)));
  }
  return { dOut, dIn };
}

function span(keys, frame) {
  if (frame <= keys[0].f) return -1;
  if (frame >= keys[keys.length - 1].f) return keys.length - 1;
  let i = 0;
  while (i < keys.length - 1 && keys[i + 1].f <= frame) i++;
  return i;
}

function evalKeys(keys, frame, dim, out) {
  if (!keys || !keys.length) return null;
  const i = span(keys, frame);
  if (i < 0) { for (let j = 0; j < dim; j++) out[j] = keys[0].v[j]; return out; }
  if (i >= keys.length - 1) {
    const last = keys[keys.length - 1];
    for (let j = 0; j < dim; j++) out[j] = last.v[j];
    return out;
  }
  const k0 = keys[i], k1 = keys[i + 1];
  const w = (k1.f - k0.f) || 1;
  const u = ease((frame - k0.f) / w, k0.tcb[3], k0.tcb[4]);
  const a = kbTangents(keys, i, dim).dOut;
  const c = kbTangents(keys, i + 1, dim).dIn;
  const u2 = u * u, u3 = u2 * u;
  const h00 = 2 * u3 - 3 * u2 + 1, h10 = u3 - 2 * u2 + u;
  const h01 = -2 * u3 + 3 * u2, h11 = u3 - u2;
  for (let j = 0; j < dim; j++) {
    out[j] = h00 * k0.v[j] + h10 * a[j] + h01 * k1.v[j] + h11 * c[j];
  }
  return out;
}

// Rotation arrives baked: one quaternion per frame from `first`, so this
// only has to slerp between the two samples bracketing a fractional frame.
function evalQuat(track, frame, out) {
  if (!track || !track.q || track.q.length < 4) return null;
  const q = track.q, n = q.length / 4;
  let f = frame - track.first;
  if (f <= 0) f = 0;
  if (f >= n - 1) f = n - 1;
  const i = Math.floor(f), u = f - i;
  const j = Math.min(i + 1, n - 1);
  let x0 = q[i * 4], y0 = q[i * 4 + 1], z0 = q[i * 4 + 2], w0 = q[i * 4 + 3];
  const x1 = q[j * 4], y1 = q[j * 4 + 1], z1 = q[j * 4 + 2], w1 = q[j * 4 + 3];
  let d = x0 * x1 + y0 * y1 + z0 * z1 + w0 * w1;
  if (d < 0) { x0 = -x0; y0 = -y0; z0 = -z0; w0 = -w0; d = -d; }
  let s0 = 1 - u, s1 = u;
  if (d < 0.9995) {
    const th = Math.acos(Math.min(1, d)), st = Math.sin(th);
    s0 = Math.sin((1 - u) * th) / st;
    s1 = Math.sin(u * th) / st;
  }
  out[0] = s0 * x0 + s1 * x1; out[1] = s0 * y0 + s1 * y1;
  out[2] = s0 * z0 + s1 * z1; out[3] = s0 * w0 + s1 * w1;
  const l = Math.hypot(out[0], out[1], out[2], out[3]) || 1;
  for (let j = 0; j < 4; j++) out[j] /= l;
  return out;
}

function quatMat(q, m) {
  const [x, y, z, w] = q;
  m[0] = 1 - 2 * (y * y + z * z); m[1] = 2 * (x * y + z * w);     m[2] = 2 * (x * z - y * w);     m[3] = 0;
  m[4] = 2 * (x * y - z * w);     m[5] = 1 - 2 * (x * x + z * z); m[6] = 2 * (y * z + x * w);     m[7] = 0;
  m[8] = 2 * (x * z + y * w);     m[9] = 2 * (y * z - x * w);     m[10] = 1 - 2 * (x * x + y * y); m[11] = 0;
  m[12] = m[13] = m[14] = 0; m[15] = 1;
  return m;
}

const sub = (a, b) => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
const cross = (a, b) => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
const dot = (a, b) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const unit = (a) => { const l = Math.hypot(a[0], a[1], a[2]) || 1; return [a[0] / l, a[1] / l, a[2] / l]; };

// gluLookAt, right-handed, with GL's column-major storage.
export function lookAt(eye, at, up) {
  const f = unit(sub(at, eye));
  const s = unit(cross(f, up));
  const u = cross(s, f);
  const m = new Mat4(), v = m.m;
  v[0] = s[0]; v[4] = s[1]; v[8] = s[2]; v[12] = -dot(s, eye);
  v[1] = u[0]; v[5] = u[1]; v[9] = u[2]; v[13] = -dot(u, eye);
  v[2] = -f[0]; v[6] = -f[1]; v[10] = -f[2]; v[14] = dot(f, eye);
  v[3] = v[7] = v[11] = 0; v[15] = 1;
  return m;
}

export class Scene {
  constructor(json) {
    this.json = json;
    this.frameEnd = json.frameEnd || 1;
    this.materials = json.materials || [];
    this.cameras = json.cameras || [];
    this.meshes = json.meshes.map((m) => {
      const n = m.positions.length / 3;
      const idx = new Uint32Array(n);
      for (let i = 0; i < n; i++) idx[i] = i;
      return {
        src: m,
        positions: new Float32Array(m.positions),
        normals: new Float32Array(m.normals || n * 3),
        uvs: m.uvs ? new Float32Array(m.uvs) : null,
        zeroUV: new Float32Array(n * 2),
        indices: idx,
        envUV: null,
        world: new Mat4(),
      };
    });
    this._q = [0, 0, 0, 1];
    this._v = [0, 0, 0];
  }

  // energy3d_scene::update_frame
  frameAt(ms, offset = 0) {
    let f = ms * 0.03 + offset;
    if (this.frameEnd > 0) f -= Math.floor(f / this.frameEnd) * this.frameEnd;
    return f;
  }

  update(frame) {
    const rot = new Mat4();
    for (const mesh of this.meshes) {
      const m = mesh.src, a = m.anim || {};
      const p = evalKeys(a.pos, frame, 3, this._v.slice()) || m.pos;
      const q = evalQuat(a.rot, frame, this._q.slice()) || m.rot;
      const s = evalKeys(a.scale, frame, 3, this._v.slice()) || m.scale;
      quatMat(q, rot.m);
      const w = mesh.world;
      w.identity();
      w.translate(p[0], p[1], p[2]);
      w.mult(rot);
      w.scale(s[0], s[1], s[2]);
    }
  }

  cameraAt(nameOrIndex, frame) {
    let c = null;
    if (typeof nameOrIndex === 'string' && nameOrIndex !== '-') {
      const want = nameOrIndex.toLowerCase();
      c = this.cameras.find((k) => k.name.toLowerCase() === want);
    }
    if (!c) c = this.cameras[typeof nameOrIndex === 'number' ? nameOrIndex : 0];
    if (!c) return null;
    const eye = evalKeys(c.anim && c.anim.pos, frame, 3, [0, 0, 0]) || c.pos;
    const at = evalKeys(c.targetAnim && c.targetAnim.pos, frame, 3, [0, 0, 0]) || c.target;
    return { eye, at, fov: c.fov, name: c.name };
  }

  projection(fov) {
    const m = new Mat4();
    const half = Math.tan(fov * DEG2RAD / 2) * NEAR;
    m.frustum(-half * ASPECT, half * ASPECT, -half, half, NEAR, FAR);
    return m;
  }
}
