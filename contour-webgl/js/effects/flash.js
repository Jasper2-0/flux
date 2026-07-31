// jace/flash — the cross-fade wash.
//
// Fully decoded from the binary. The constructor (0x41ff80) stores its
// 32-byte parameter block at this+0x94. The geometry builder (0x4200e0)
// lays out N−1 vertices on a circle of radius p[0x14] and puts the last
// vertex at (0, 0) — a triangle fan with the centre last. Every vertex,
// rim and centre alike, is given z = p[0x14] as well, so the disc's
// radius and its depth are the same number: the fan always subtends the
// same angle at the camera whatever the parameter says. It is a
// full-frame wash, and the radius has no visual effect.
//
// The update (0x420160) writes nothing but vertex colours:
//
//   v = clamp01((T − p[0x00]) / (p[0x04] − p[0x00]))   → the rim
//   s = clamp01((T − p[0x08]) / (p[0x0c] − p[0x08]))   → the centre
//   rim    colour = lerp(colourA, colourB, v)
//   centre colour = lerp(colourA, colourB, s)
//
// with colourA at p[0x18] and colourB at p[0x1c], both 0xRRGGBB. When
// both ramps have passed 0.999 the object sets its own dead flag at
// this+0x90, so the two ramps are also the effect's lifetime.
//
// The blend is additive — the timeline proves it. At 51.5 s instance 22
// runs black→white and at 52.0 s instance 24 runs white→black: a cut
// through white. Under alpha blending the first of those would paint the
// screen *black* before it went white.
//
// Reconstructed: nothing about the maths. Only the mapping of "radius
// equals depth" onto this port's orthographic wash, and the choice of
// SRC_ALPHA/ONE with alpha carried in the vertex colour.

const clamp01 = (v) => (v < 0 ? 0 : v > 1 ? 1 : v);
const SEGS = 33;      // the fan the init builds: 33 rim vertices + centre
const ASPECT = 4 / 3;

export class Flash {
  constructor(mgl, tex, params) {
    this.mgl = mgl;
    this.p = params;
    const unpack = (c) => [((c >> 16) & 255) / 255, ((c >> 8) & 255) / 255, (c & 255) / 255];
    this.ca = unpack(params.colors[0]);
    this.cb = unpack(params.colors[1]);
    // the object kills itself once both ramps have finished
    this.duration = Math.max(params.rampRim[1], params.rampCentre[1]) + 0.05;
  }

  do(time, timeStart) {
    const mgl = this.mgl;
    const T = time - timeStart;
    const p = this.p;

    const ramp = (r) => clamp01((T - r[0]) / (r[1] - r[0] || 1e-4));
    const v = ramp(p.rampRim);
    const s = ramp(p.rampCentre);

    const mix = (t) => [
      this.ca[0] + (this.cb[0] - this.ca[0]) * t,
      this.ca[1] + (this.cb[1] - this.ca[1]) * t,
      this.ca[2] + (this.cb[2] - this.ca[2]) * t,
    ];
    const rim = mix(v);
    const mid = mix(s);
    if (rim[0] + rim[1] + rim[2] + mid[0] + mid[1] + mid[2] < 0.004) return;

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.ortho(-ASPECT, ASPECT, -1, 1, -1, 1);
    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    mgl.enableTexture(false);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    mgl.enableCullFace(false);
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);

    // radius == depth, so the disc always inscribes the frame height; go
    // out to the corner distance so a fully-lit rim really does fill it
    const R = Math.hypot(ASPECT, 1);
    mgl.begin(mgl.TRIANGLES);
    for (let i = 0; i < SEGS; i++) {
      const a0 = (i / SEGS) * Math.PI * 2;
      const a1 = ((i + 1) / SEGS) * Math.PI * 2;
      mgl.color4(mid[0], mid[1], mid[2], 1); mgl.vertex3(0, 0, 0);
      mgl.color4(rim[0], rim[1], rim[2], 1); mgl.vertex3(Math.cos(a0) * R, Math.sin(a0) * R, 0);
      mgl.color4(rim[0], rim[1], rim[2], 1); mgl.vertex3(Math.cos(a1) * R, Math.sin(a1) * R, 0);
    }
    mgl.end();
  }
}
