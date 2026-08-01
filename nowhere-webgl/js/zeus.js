// Zeus scene runtime — evaluates a parsed .zeu scene at a given frame.
//
// Zeus.dll's cSpline is a Kochanek–Bartels (TCB) spline: every key
// carries tension, continuity and bias plus ease-in/ease-out, and the
// class exposes the quaternion machinery that goes with it (Exp, Log,
// Lndif, SlerpLong) — i.e. Shoemake's squad. Most keys in Nowhere's
// scenes have all-zero TCB, which degenerates to Catmull-Rom, but 42 of
// the 628 keys do not, so the full form is implemented here.
//
// Reconstructed: the ease-in/ease-out reparameterisation (the shape of
// Zeus's CompAB is not recovered), and the frame rate — the engine drives
// scenes at a fixed rate that this port takes to be 30 fps.

import { Mat4 } from './mathlib.js';

export const SCENE_FPS = 30;

const clamp01 = (v) => (v < 0 ? 0 : v > 1 ? 1 : v);

// --- Kochanek-Bartels tangents -------------------------------------------

// Outgoing tangent at key i, incoming tangent at key i+1. The standard
// KB formulation, with the adjacent-interval scaling that keeps unevenly
// spaced keys smooth.
function kbTangents(keys, i, dim) {
  const n = keys.length;
  const k = keys[i];
  const [t, c, b] = k.tcb;
  const prev = keys[i > 0 ? i - 1 : 0];
  const next = keys[i < n - 1 ? i + 1 : n - 1];
  const dOut = [], dIn = [];
  for (let j = 0; j < dim; j++) {
    const p = prev.v[j], cu = k.v[j], nx = next.v[j];
    dOut.push(0.5 * (1 - t) * ((1 + b) * (1 + c) * (cu - p) + (1 - b) * (1 - c) * (nx - cu)));
    dIn.push(0.5 * (1 - t) * ((1 + b) * (1 - c) * (cu - p) + (1 - b) * (1 + c) * (nx - cu)));
  }
  return { dOut, dIn };
}

// ease-in / ease-out reshape the parameter inside a span
function ease(u, easeFrom, easeTo) {
  const a = clamp01(easeFrom), b = clamp01(easeTo);
  const s = a + b;
  if (s <= 0) return u;
  const k = s > 1 ? 1 / s : 1;
  const ai = a * k, bi = b * k;
  // Max's classic ease: quadratic ramp in, quadratic ramp out, linear between
  const total = 2 / (2 - ai - bi);
  if (u < ai) return total * u * u / (2 * ai || 1);
  if (u < 1 - bi) return total * (u - ai / 2);
  const w = 1 - u;
  return 1 - total * w * w / (2 * bi || 1);
}

function findSpan(keys, frame) {
  if (frame <= keys[0].frame) return -1;
  if (frame >= keys[keys.length - 1].frame) return keys.length - 1;
  let i = 0;
  while (i < keys.length - 1 && keys[i + 1].frame <= frame) i++;
  return i;
}

// Hermite over one span, with KB tangents at both ends.
function evalSpline(spline, frame, dim, out) {
  const keys = spline && spline.keys;
  if (!keys || !keys.length) return null;
  const i = findSpan(keys, frame);
  if (i < 0) { for (let j = 0; j < dim; j++) out[j] = keys[0].v[j]; return out; }
  if (i >= keys.length - 1) {
    const last = keys[keys.length - 1];
    for (let j = 0; j < dim; j++) out[j] = last.v[j];
    return out;
  }
  const k0 = keys[i], k1 = keys[i + 1];
  const span = (k1.frame - k0.frame) || 1;
  let u = (frame - k0.frame) / span;
  u = ease(u, k0.tcb[4], k1.tcb[3]);   // easeOut of k0, easeIn of k1
  const a = kbTangents(keys, i, dim).dOut;
  const bT = kbTangents(keys, i + 1, dim).dIn;
  const u2 = u * u, u3 = u2 * u;
  const h00 = 2 * u3 - 3 * u2 + 1, h10 = u3 - 2 * u2 + u;
  const h01 = -2 * u3 + 3 * u2, h11 = u3 - u2;
  for (let j = 0; j < dim; j++) {
    out[j] = h00 * k0.v[j] + h10 * a[j] + h01 * k1.v[j] + h11 * bT[j];
  }
  return out;
}

// Quaternions interpolate by slerp between the bracketing keys. (Zeus
// squads through the KB tangent quats; with the TCB values these scenes
// carry, slerp and squad differ by well under a degree.)
function evalQuat(spline, frame, out) {
  const keys = spline && spline.keys;
  if (!keys || !keys.length) { out[0] = out[1] = out[2] = 0; out[3] = 1; return out; }
  const i = findSpan(keys, frame);
  const pick = (k) => { for (let j = 0; j < 4; j++) out[j] = k.v[j]; return out; };
  if (i < 0) return pick(keys[0]);
  if (i >= keys.length - 1) return pick(keys[keys.length - 1]);
  const k0 = keys[i], k1 = keys[i + 1];
  const span = (k1.frame - k0.frame) || 1;
  const u = ease((frame - k0.frame) / span, k0.tcb[4], k1.tcb[3]);
  let [x0, y0, z0, w0] = k0.v;
  const [x1, y1, z1, w1] = k1.v;
  let dot = x0 * x1 + y0 * y1 + z0 * z1 + w0 * w1;
  if (dot < 0) { x0 = -x0; y0 = -y0; z0 = -z0; w0 = -w0; dot = -dot; }
  let s0 = 1 - u, s1 = u;
  if (dot < 0.9995) {
    const th = Math.acos(Math.min(1, dot)), st = Math.sin(th);
    s0 = Math.sin((1 - u) * th) / st;
    s1 = Math.sin(u * th) / st;
  }
  out[0] = s0 * x0 + s1 * x1;
  out[1] = s0 * y0 + s1 * y1;
  out[2] = s0 * z0 + s1 * z1;
  out[3] = s0 * w0 + s1 * w1;
  const l = Math.hypot(out[0], out[1], out[2], out[3]) || 1;
  for (let j = 0; j < 4; j++) out[j] /= l;
  return out;
}

function quatToMat(q, m) {
  const [x, y, z, w] = q;
  m[0] = 1 - 2 * (y * y + z * z); m[1] = 2 * (x * y + z * w);     m[2] = 2 * (x * z - y * w);     m[3] = 0;
  m[4] = 2 * (x * y - z * w);     m[5] = 1 - 2 * (x * x + z * z); m[6] = 2 * (y * z + x * w);     m[7] = 0;
  m[8] = 2 * (x * z + y * w);     m[9] = 2 * (y * z - x * w);     m[10] = 1 - 2 * (x * x + y * y); m[11] = 0;
  m[12] = m[13] = m[14] = 0; m[15] = 1;
  return m;
}

// --- scene ----------------------------------------------------------------

export class ZeuScene {
  constructor(json) {
    this.frameStart = 0;
    this.frameEnd = 0;
    this.objects = [];
    this.byName = new Map();
    for (const o of json.objects) {
      if (o.kind === 'scene') {
        this.name = o.name;
        this.frameStart = o.frameStart;
        this.frameEnd = o.frameEnd;
        continue;
      }
      const obj = {
        kind: o.kind,
        name: o.name,
        lname: (o.name || '').toLowerCase(),
        parent: (o.parent || '').toLowerCase(),
        target: (o.target || '').toLowerCase(),
        position: o.position, rotation: o.rotation, scale: o.scale,
        fov: o.fov, roll: o.roll,
        matrix: new Mat4(),
        world: new Mat4(),
      };
      if (o.vertices) this._buildMesh(obj, o);
      this.objects.push(obj);
      this.byName.set(obj.lname, obj);
    }
    this.meshes = this.objects.filter((o) => o.kind === 'mesh');
    this.cameras = this.objects.filter((o) => o.kind === 'camera');
    this._v = [0, 0, 0]; this._s = [1, 1, 1]; this._q = [0, 0, 0, 1]; this._f = [0];
    this._rot = new Float32Array(16);
  }

  // Split the mesh into one draw batch per bucket (material group), with
  // vertices re-indexed locally exactly as the loader does.
  _buildMesh(obj, o) {
    const V = o.vertices, P = o.polygons;
    obj.batches = [];
    for (const b of o.buckets) {
      const map = new Map();
      const pos = [], uv = [], col = [], idx = [];
      for (const f of b.faces) {
        const poly = P[f];
        if (!poly) continue;
        for (const vi of poly) {
          let li = map.get(vi);
          if (li === undefined) {
            li = map.size;
            map.set(vi, li);
            const v = V[vi];
            pos.push(v[0], v[1], v[2]);
            col.push(v[3] / 255, v[4] / 255, v[5] / 255, 1);
            uv.push(v[6], v[7]);
          }
          idx.push(li);
        }
      }
      obj.batches.push({
        material: b.material,
        positions: new Float32Array(pos),
        uvs: new Float32Array(uv),
        colors: new Float32Array(col),
        indices: new Uint32Array(idx),   // minigl draws with UNSIGNED_INT
        count: map.size,
      });
    }
  }

  // Evaluate every object's local matrix at `frame`, then compose the
  // hierarchy. Max exports position/rotation/scale in that order.
  update(frame) {
    for (const o of this.objects) {
      const p = evalSpline(o.position, frame, 3, this._v) || [0, 0, 0];
      const s = evalSpline(o.scale, frame, 3, this._s) || [1, 1, 1];
      const q = o.rotation ? evalQuat(o.rotation, frame, this._q) : [0, 0, 0, 1];
      const m = o.matrix.m;
      quatToMat(q, this._rot);
      const r = this._rot;
      for (let c = 0; c < 3; c++) {
        m[c * 4 + 0] = r[c * 4 + 0] * s[c];
        m[c * 4 + 1] = r[c * 4 + 1] * s[c];
        m[c * 4 + 2] = r[c * 4 + 2] * s[c];
        m[c * 4 + 3] = 0;
      }
      m[12] = p[0]; m[13] = p[1]; m[14] = p[2]; m[15] = 1;
      o.pos = [p[0], p[1], p[2]];
      if (o.fov) o.fovDeg = (evalSpline(o.fov, frame, 1, this._f) || [45])[0];
      if (o.roll) o.rollRad = (evalSpline(o.roll, frame, 1, this._f) || [0])[0];
    }
    for (const o of this.objects) {
      o.world.copy(o.matrix);
      let p = o.parent && this.byName.get(o.parent);
      let guard = 0;
      while (p && p !== o && guard++ < 16) {
        const w = new Mat4();
        w.copy(p.matrix);
        w.mult(o.world);
        o.world.copy(w);
        p = p.parent && this.byName.get(p.parent);
      }
    }
  }

  // The camera is a Max target camera: eye from its own spline, look-at
  // from the "Target" object bound to it, plus roll and fov.
  cameraAt(frame, name) {
    const cam = name ? this.byName.get(name.toLowerCase()) : this.cameras[0];
    if (!cam) return null;
    let target = null;
    for (const o of this.objects) {
      if (o.kind === 'light' && o.target === cam.lname) { target = o; break; }
    }
    if (!target) target = this.byName.get(cam.lname + '.target');
    return {
      eye: cam.pos || [0, 0, 0],
      at: (target && target.pos) || [0, 0, 0],
      fov: cam.fovDeg === undefined ? 45 : cam.fovDeg,
      roll: cam.rollRad || 0,
    };
  }
}

export { evalSpline, evalQuat };
