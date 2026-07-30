// Math labs: interactive visualizations of the mathematics behind each
// effect. Loads after article.js; syncs to viewer playheads via
// window.__teslaLabs.onTime(key, fn).

(() => {
  const $ = (s) => document.querySelector(s);
  const labs = window.__teslaLabs || { onTime: () => {}, reducedMotion: false };
  const reducedMotion = labs.reducedMotion;
  const TAU = Math.PI * 2;

  const COL = {
    grid: '#1b2431', dim: '#67737f', ink: '#c9d4dd',
    a: '#6fc3ff', b: '#b48cff', c: '#f0b46a', d: '#8ae0a0', e: '#ff9bb0', f: '#f0e28a',
  };

  function ctx2d(sel) {
    const cv = $(sel);
    return cv ? { cv, ctx: cv.getContext('2d') } : null;
  }

  function watchVisible(el) {
    const s = { v: false };
    new IntersectionObserver((es) => { s.v = es[0].isIntersecting; }).observe(el);
    return s;
  }

  function mono(ctx, size) { ctx.font = size + 'px ui-monospace, Menlo, monospace'; }

  // the demo's Catmull-Rom kernel, exactly as ported
  function catmull(a, b, c, d, t) {
    const fa = 0.5 * (3 * b + d - a - 3 * c);
    const fb = a + 2 * c - 0.5 * (5 * b + d);
    const fc = 0.5 * (c - a);
    return t * t * t * fa + t * t * fb + t * fc + b;
  }

  // pointer dragging of an array of [x, y] handles on a canvas
  function draggable(cv, points, radius, onMove) {
    let drag = -1;
    const pos = (e) => {
      const r = cv.getBoundingClientRect();
      return [(e.clientX - r.left) * (cv.width / r.width), (e.clientY - r.top) * (cv.height / r.height)];
    };
    cv.addEventListener('pointerdown', (e) => {
      const [x, y] = pos(e);
      for (let i = 0; i < points.length; i++) {
        const dx = points[i][0] - x, dy = points[i][1] - y;
        if (dx * dx + dy * dy < radius * radius) { drag = i; cv.setPointerCapture(e.pointerId); break; }
      }
    });
    cv.addEventListener('pointermove', (e) => {
      if (drag < 0) return;
      const [x, y] = pos(e);
      points[drag][0] = Math.max(0, Math.min(cv.width, x));
      points[drag][1] = Math.max(0, Math.min(cv.height, y));
      onMove();
    });
    cv.addEventListener('pointerup', () => { drag = -1; });
    cv.style.touchAction = 'none';
  }

  /* ================================================================
   * SPINZOOM — nested transform tunnel: M_i = M_{i-1} · R(θ) · S(s)
   * ================================================================ */
  (function spinzoomLab() {
    const g = ctx2d('#lab-spinzoom');
    if (!g) return;
    const { cv, ctx } = g;
    const sIn = $('#sz-s'), thIn = $('#sz-th'), readout = $('#sz-read');
    const glowIn = $('#sz-glow'), exagIn = $('#sz-exag');
    const visible = watchVisible(cv);
    let phase = 0;

    function draw() {
      const s = parseFloat(sIn.value), th = parseFloat(thIn.value) * Math.PI / 180;
      const glow = glowIn && glowIn.checked;
      // honest mode uses the true s^i spacing; exaggerated mode raises the
      // exponent so the 18 copies separate enough to read as a diagram
      const exp = (exagIn && exagIn.checked) ? 1.4 : 1.0;
      const W = cv.width, H = cv.height;
      ctx.clearRect(0, 0, W, H);
      ctx.save();
      ctx.translate(W / 2, H / 2);
      ctx.globalCompositeOperation = 'lighter';
      let scale = Math.min(W, H) * 0.46, ang = phase;
      for (let i = 0; i < 18; i++) {
        ctx.save();
        ctx.rotate(ang);
        if (glow) {
          // filled soft quads: outlines melt into the smoke of the effect
          const grd = ctx.createRadialGradient(0, 0, scale * 0.1, 0, 0, scale * 1.1);
          grd.addColorStop(0, 'rgba(140,190,235,0.16)');
          grd.addColorStop(1, 'rgba(140,190,235,0)');
          ctx.fillStyle = grd;
          ctx.fillRect(-scale, -scale * 0.75, scale * 2, scale * 1.5);
        } else {
          ctx.strokeStyle = 'rgba(111,195,255,0.5)';
          ctx.fillStyle = 'rgba(111,195,255,0.055)';
          ctx.lineWidth = 1;
          ctx.beginPath();
          ctx.rect(-scale, -scale * 0.75, scale * 2, scale * 1.5);
          ctx.fill();
          ctx.stroke();
        }
        ctx.restore();
        ang += th;
        scale /= s === 0 ? 1 : Math.pow(s, exp);
      }
      ctx.restore();
      readout.textContent = 's = ' + s.toFixed(3) + ' · θ = ' + (th * 180 / Math.PI).toFixed(0) +
        '° · copy 18 scale = s¹⁸ = ' + Math.pow(parseFloat(sIn.value), 18).toFixed(2) + '×';
    }

    (function loop() {
      if (visible.v) {
        if (!reducedMotion) phase += 0.0035;
        draw();
      }
      requestAnimationFrame(loop);
    })();
    for (const el of [sIn, thIn]) el.addEventListener('input', draw);
    for (const el of [glowIn, exagIn]) if (el) el.addEventListener('change', draw);
    draw();
  })();

  /* ================================================================
   * SHADEBALL — the time comb: t_{i+1} = t_i − 0.014·(8 sin t + 12)
   * ================================================================ */
  (function shadeballComb() {
    const g = ctx2d('#lab-shadeball');
    if (!g) return;
    const { cv, ctx } = g;
    const visible = watchVisible(cv);
    let T = 4;

    function draw() {
      const W = cv.width, H = cv.height, pad = 30;
      ctx.clearRect(0, 0, W, H);
      // window sized to the trail itself, so the comb always fills the axis
      const step0 = 0.014 * (8 * Math.sin(T) + 12);
      const span = step0 * 19;
      const window0 = T - span - span * 0.15, window1 = T + span * 0.15;
      const xOf = (t) => pad + ((t - window0) / (window1 - window0)) * (W - pad * 2);

      ctx.strokeStyle = COL.grid;
      ctx.beginPath();
      ctx.moveTo(pad, H - 34);
      ctx.lineTo(W - pad, H - 34);
      ctx.stroke();
      ctx.fillStyle = COL.dim;
      mono(ctx, 10);
      const tickEvery = span > 3 ? 1 : 0.5;
      for (let t = Math.ceil(window0 / tickEvery) * tickEvery; t <= window1; t += tickEvery) {
        ctx.fillText(t.toFixed(1) + 's', xOf(t) - 10, H - 18);
        ctx.fillRect(xOf(t), H - 38, 1, 8);
      }

      // the 20 shells: each drawn at an earlier time, dimmer and smaller.
      // above each tick, a ghost circle — the sphere as it looked at that
      // sample time — so the comb visibly becomes a motion trail
      let t = T;
      const step = 0.014 * (8 * Math.sin(T) + 12);
      const ghostY = 74;
      for (let i = 19; i >= 0; i--) {
        let tt = T;
        for (let k = 0; k < i; k++) tt -= 0.014 * (8 * Math.sin(T) + 12);
        const alpha = Math.max(0, 1 - i / 12);
        const x = xOf(tt);
        if (alpha > 0) {
          ctx.strokeStyle = 'rgba(138,208,200,' + (0.12 + alpha * 0.55) + ')';
          ctx.lineWidth = 1.5;
          ctx.beginPath();
          ctx.arc(x, ghostY, 8 + alpha * 20, 0, TAU);
          ctx.stroke();
        }
      }
      ctx.lineWidth = 1;
      for (let i = 0; i < 20; i++) {
        const alpha = Math.max(0, 1 - i / 12);
        const x = xOf(t);
        const h = 14 + alpha * 34;
        ctx.strokeStyle = alpha > 0 ? 'rgba(138,208,200,' + (0.25 + alpha * 0.75) + ')' : 'rgba(255,155,176,0.5)';
        ctx.lineWidth = alpha > 0 ? 2 : 1;
        ctx.beginPath();
        ctx.moveTo(x, H - 38);
        ctx.lineTo(x, H - 38 - h);
        ctx.stroke();
        if (i === 0 || i === 12 || i === 19) {
          ctx.fillStyle = alpha > 0 ? COL.d : COL.e;
          ctx.fillText('i=' + i, x - 8, H - 44 - h);
        }
        t -= 0.014 * (8 * Math.sin(T) + 12);
      }
      ctx.fillStyle = COL.dim;
      ctx.fillText('spacing now: Δ = 0.014·(8·sin t + 12) = ' + step.toFixed(3) + ' s', pad, 16);
      ctx.fillText('the circles are the ball at each sample time — overlapped, they are the trail', pad, 32);
      ctx.fillStyle = COL.e;
      ctx.fillText('α = 1 − i/12 goes negative past i = 12 → shells 13…19 clamp invisible', pad, 48);
    }

    labs.onTime('shadeball', (t) => { T = t; if (visible.v) draw(); });
    (function loop() {
      if (visible.v) {
        if (!reducedMotion) T += 0.016;
        draw();
      }
      requestAnimationFrame(loop);
    })();
    draw();
  })();

  /* ================================================================
   * SPLINES — billboarding from the inverse modelview basis
   * ================================================================ */
  (function billboardLab() {
    const g = ctx2d('#lab-splines');
    if (!g) return;
    const { cv, ctx } = g;
    const camIn = $('#bb-cam');
    const visible = watchVisible(cv);
    let auto = 0;

    function draw() {
      const W = cv.width, H = cv.height;
      const camA = parseFloat(camIn.value) * Math.PI / 180 + auto;
      ctx.clearRect(0, 0, W, H);
      const cx = W / 2, cy = H / 2 + 8;

      // world path (top-down): one Lissajous trail
      const path = [];
      for (let i = 0; i <= 260; i++) {
        const t = (i / 260) * 9;
        path.push([
          cx + (Math.sin(t * 1.2) * 100 + Math.sin(t * 2.3 + 1) * 100) * 0.85,
          cy - (Math.sin(t * 1.32) * 100 + Math.sin(t * 2.16 + 1) * 100) * 0.55,
        ]);
      }
      ctx.strokeStyle = 'rgba(240,226,138,0.35)';
      ctx.lineWidth = 1;
      ctx.beginPath();
      path.forEach((p, i) => (i ? ctx.lineTo(p[0], p[1]) : ctx.moveTo(p[0], p[1])));
      ctx.stroke();

      // camera on a ring
      const camR = Math.min(W, H) * 0.46;
      const camX = cx + Math.cos(camA) * camR, camY = cy + Math.sin(camA) * camR;
      // view dir toward center; screen-right basis bx = perpendicular
      const vx = cx - camX, vy = cy - camY;
      const vl = Math.hypot(vx, vy);
      const dx = vx / vl, dy = vy / vl;
      const bx = -dy, by = dx; // right vector = first column of V⁻¹, top-down

      ctx.fillStyle = COL.c;
      ctx.beginPath();
      ctx.arc(camX, camY, 6, 0, TAU);
      ctx.fill();
      mono(ctx, 11);
      ctx.fillText('camera', camX - 20, camY - 12);
      ctx.strokeStyle = 'rgba(240,180,106,0.35)';
      ctx.beginPath();
      ctx.moveTo(camX, camY);
      ctx.lineTo(camX + dx * 70, camY + dy * 70);
      ctx.stroke();

      // billboards: segments through path samples along bx — always ⟂ view
      ctx.strokeStyle = COL.a;
      ctx.lineWidth = 3;
      ctx.beginPath();
      for (let i = 0; i < path.length; i += 13) {
        const [px, py] = path[i];
        ctx.moveTo(px - bx * 13, py - by * 13);
        ctx.lineTo(px + bx * 13, py + by * 13);
      }
      ctx.stroke();

      // annotate basis on one sprite
      const [sx, sy] = path[130];
      ctx.strokeStyle = COL.d;
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(sx, sy);
      ctx.lineTo(sx + bx * 34, sy + by * 34);
      ctx.stroke();
      ctx.fillStyle = COL.d;
      ctx.fillText('b_x = (V⁻¹)·x̂', sx + bx * 38 - 20, sy + by * 38);

      ctx.fillStyle = COL.dim;
      ctx.fillText('top-down view — every sprite lies along b_x, so all face the camera', 18, 20);
    }

    (function loop() {
      if (visible.v) {
        if (!reducedMotion) auto += 0.004;
        draw();
      }
      requestAnimationFrame(loop);
    })();
    camIn.addEventListener('input', draw);
    draw();
  })();

  /* ================================================================
   * FFDENV — morph ring + cosine easing
   * ================================================================ */
  (function morphRing() {
    const g = ctx2d('#lab-ffdenv');
    if (!g) return;
    const { cv, ctx } = g;
    const visible = watchVisible(cv);
    let T = 0;

    function draw() {
      const W = cv.width, H = cv.height;
      ctx.clearRect(0, 0, W, H);
      const cx = H / 2 + 10, cy = H / 2, R = H * 0.34;

      const cycle = (T * 0.5) % 6;
      const iO1 = Math.floor(cycle);
      const iO2 = (iO1 + 1) % 6;
      const frac = cycle - iO1;
      const eased = Math.cos((1 - frac) * Math.PI) * 0.5 + 0.5;

      // the ring of six morph targets
      for (let i = 0; i < 6; i++) {
        const a = -Math.PI / 2 + (i / 6) * TAU;
        const x = cx + Math.cos(a) * R, y = cy + Math.sin(a) * R;
        const isA = i === iO1, isB = i === iO2;
        ctx.beginPath();
        ctx.arc(x, y, isA || isB ? 13 : 9, 0, TAU);
        ctx.fillStyle = isA ? COL.b : isB ? COL.a : '#1b2431';
        ctx.fill();
        ctx.strokeStyle = COL.dim;
        ctx.stroke();
        ctx.fillStyle = COL.ink;
        mono(ctx, 11);
        ctx.fillText(String(i), x - 3, y + 4);
      }
      // arc showing eased progress between the active pair
      const a1 = -Math.PI / 2 + (iO1 / 6) * TAU;
      const a2 = -Math.PI / 2 + ((iO1 + 1) / 6) * TAU;
      ctx.strokeStyle = COL.c;
      ctx.lineWidth = 3;
      ctx.beginPath();
      ctx.arc(cx, cy, R, a1, a1 + (a2 - a1) * eased);
      ctx.stroke();
      ctx.lineWidth = 1;

      // easing inset: linear vs cos((1−t)π)/2 + 1/2
      const px = H + 40, pw = W - px - 30, py = 34, ph = H - 70;
      ctx.strokeStyle = COL.grid;
      ctx.strokeRect(px, py, pw, ph);
      ctx.strokeStyle = COL.dim;
      ctx.setLineDash([3, 4]);
      ctx.beginPath();
      ctx.moveTo(px, py + ph);
      ctx.lineTo(px + pw, py);
      ctx.stroke();
      ctx.setLineDash([]);
      ctx.strokeStyle = COL.c;
      ctx.lineWidth = 2;
      ctx.beginPath();
      for (let i = 0; i <= 100; i++) {
        const t = i / 100;
        const v = Math.cos((1 - t) * Math.PI) * 0.5 + 0.5;
        const x = px + t * pw, y = py + ph - v * ph;
        i ? ctx.lineTo(x, y) : ctx.moveTo(x, y);
      }
      ctx.stroke();
      ctx.lineWidth = 1;
      ctx.fillStyle = COL.c;
      ctx.beginPath();
      ctx.arc(px + frac * pw, py + ph - eased * ph, 4, 0, TAU);
      ctx.fill();
      mono(ctx, 11);
      ctx.fillStyle = COL.dim;
      ctx.fillText('linear', px + pw - 44, py + 14);
      ctx.fillStyle = COL.c;
      ctx.fillText('fT = cos((1−t)·π)/2 + 1/2', px + 8, py + ph + 18);
      ctx.fillStyle = COL.ink;
      ctx.fillText('object ' + iO1 + ' → ' + iO2 + '   t = ' + frac.toFixed(2) + '   fT = ' + eased.toFixed(2), px + 8, py - 12);
    }

    labs.onTime('ffdenv', (t) => { T = t; if (visible.v) draw(); });
    (function loop() {
      if (visible.v) {
        if (!reducedMotion) T += 0.016;
        draw();
      }
      requestAnimationFrame(loop);
    })();
    draw();
  })();

  /* ================================================================
   * FFDENV/FACEMORPH — sphere env-mapping: n̂ in camera space → UV
   * ================================================================ */
  (function envmapLab() {
    const g = ctx2d('#lab-envmap');
    if (!g) return;
    const { cv, ctx } = g;
    const visible = watchVisible(cv);
    let rot = 0.4;

    function draw() {
      const W = cv.width, H = cv.height;
      ctx.clearRect(0, 0, W, H);
      const cx = H / 2 + 14, cy = H / 2, R = H * 0.33;
      const sq = H * 0.62, sqx = W - sq - 40, sqy = (H - sq) / 2;

      // "object" circle with rotating normals
      ctx.strokeStyle = COL.dim;
      ctx.beginPath();
      ctx.arc(cx, cy, R, 0, TAU);
      ctx.stroke();

      // texture square (UV space)
      ctx.strokeStyle = COL.grid;
      for (let i = 0; i <= 4; i++) {
        ctx.beginPath();
        ctx.moveTo(sqx + (i / 4) * sq, sqy);
        ctx.lineTo(sqx + (i / 4) * sq, sqy + sq);
        ctx.moveTo(sqx, sqy + (i / 4) * sq);
        ctx.lineTo(sqx + sq, sqy + (i / 4) * sq);
        ctx.stroke();
      }
      ctx.strokeStyle = COL.dim;
      ctx.strokeRect(sqx, sqy, sq, sq);
      mono(ctx, 11);
      ctx.fillStyle = COL.dim;
      ctx.fillText('u = n̂.x/2 + ½', sqx, sqy + sq + 18);
      ctx.fillText('v = n̂.y/2 + ½', sqx, sqy + sq + 34);

      for (let i = 0; i < 12; i++) {
        const a = rot + (i / 12) * TAU;
        const nx = Math.cos(a), ny = Math.sin(a);
        const px = cx + nx * R, py = cy + ny * R;
        const hot = i === 0;
        ctx.strokeStyle = hot ? COL.c : 'rgba(111,195,255,0.5)';
        ctx.lineWidth = hot ? 2 : 1;
        ctx.beginPath();
        ctx.moveTo(px, py);
        ctx.lineTo(px + nx * 16, py + ny * 16);
        ctx.stroke();
        // where that normal samples the texture
        const u = nx * 0.5 + 0.5, v = ny * 0.5 + 0.5;
        const tx = sqx + u * sq, ty = sqy + v * sq;
        ctx.fillStyle = hot ? COL.c : 'rgba(111,195,255,0.55)';
        ctx.beginPath();
        ctx.arc(tx, ty, hot ? 5 : 3, 0, TAU);
        ctx.fill();
        if (hot) {
          ctx.strokeStyle = 'rgba(240,180,106,0.35)';
          ctx.lineWidth = 1;
          ctx.beginPath();
          ctx.moveTo(px + nx * 16, py + ny * 16);
          ctx.lineTo(tx, ty);
          ctx.stroke();
        }
      }
      ctx.lineWidth = 1;
      ctx.fillStyle = COL.dim;
      ctx.fillText('surface normals, camera space', cx - R, cy + R + 26);
    }

    (function loop() {
      if (visible.v) {
        if (!reducedMotion) rot += 0.006;
        draw();
      }
      requestAnimationFrame(loop);
    })();
    draw();
  })();

  /* ================================================================
   * BANDS — random angle walk on a circle
   * ================================================================ */
  (function bandsLab() {
    const g = ctx2d('#lab-bands');
    if (!g) return;
    const { cv, ctx } = g;
    const visible = watchVisible(cv);
    const chords = [];
    let angle = 0, acc = 0;
    const angleHist = [];

    function spawn() {
      angle = (angle + (Math.random() - 0.5) * 2) % TAU;
      const r = 1 + 0.3 * Math.random();
      const fx = (Math.random() - 0.5) * 0.5, fy = (Math.random() - 0.5) * 0.5;
      const width = 0.55;
      chords.push({
        a: [fx + Math.sin(angle) * r, fy + Math.cos(angle) * r],
        b: [fx + Math.sin(angle + width) * r, fy + Math.cos(angle + width) * r],
        age: 0,
      });
      angleHist.push(angle);
      if (chords.length > 32) chords.shift();
      if (angleHist.length > 64) angleHist.shift();
    }

    function draw() {
      const W = cv.width, H = cv.height;
      ctx.clearRect(0, 0, W, H);
      const cx = H / 2 + 10, cy = H / 2, S = H * 0.3;

      ctx.strokeStyle = COL.grid;
      ctx.beginPath();
      ctx.arc(cx, cy, S, 0, TAU);
      ctx.stroke();
      ctx.setLineDash([2, 4]);
      ctx.beginPath();
      ctx.arc(cx, cy, S * 1.3, 0, TAU);
      ctx.stroke();
      ctx.setLineDash([]);

      chords.forEach((c, i) => {
        const alpha = (i + 1) / chords.length;
        ctx.strokeStyle = 'rgba(125,156,255,' + (alpha * 0.9) + ')';
        ctx.lineWidth = 1 + alpha * 2.5;
        ctx.beginPath();
        ctx.moveTo(cx + c.a[0] * S, cy - c.a[1] * S);
        ctx.lineTo(cx + c.b[0] * S, cy - c.b[1] * S);
        ctx.stroke();
      });
      ctx.lineWidth = 1;
      mono(ctx, 11);
      ctx.fillStyle = COL.dim;
      ctx.fillText('far-plane cross-section: each new segment is a chord', 16, 18);
      ctx.fillText('r ∈ [1, 1.3]', cx + S * 0.75, cy - S * 1.28);

      // angle walk strip
      const px = H + 44, pw = W - px - 26, py = 40, ph = H - 84;
      ctx.strokeStyle = COL.grid;
      ctx.strokeRect(px, py, pw, ph);
      ctx.strokeStyle = COL.e;
      ctx.beginPath();
      angleHist.forEach((a, i) => {
        const x = px + (i / 63) * pw;
        const y = py + ph - ((a % TAU + TAU) % TAU) / TAU * ph;
        i ? ctx.lineTo(x, y) : ctx.moveTo(x, y);
      });
      ctx.stroke();
      ctx.fillStyle = COL.dim;
      ctx.fillText('aᵢ₊₁ = aᵢ + U(−1,1)  (mod 2π) — the twist is a random walk', px, py - 10);
      ctx.fillText('segment index →', px + pw - 120, py + ph + 18);
    }

    (function loop() {
      if (visible.v) {
        acc++;
        if (!reducedMotion && acc % 14 === 0) spawn();
        draw();
      }
      requestAnimationFrame(loop);
    })();
    for (let i = 0; i < 20; i++) spawn();
    draw();
  })();

  /* ================================================================
   * ENERGYSTREAM — the fmod conveyor and linear fog
   * ================================================================ */
  (function conveyorLab() {
    const g = ctx2d('#lab-energystream');
    if (!g) return;
    const { cv, ctx } = g;
    const visible = watchVisible(cv);
    let T = 0;
    const flares = [];
    for (let i = 0; i < 26; i++) {
      flares.push({ x0: Math.random() * -800 - 1150, speed: 130 + Math.random() * 210, y: 0.2 + 0.6 * Math.random() });
    }
    const fmod = (a, b) => a - b * Math.trunc(a / b);

    function draw() {
      const W = cv.width, H = cv.height, pad = 30;
      ctx.clearRect(0, 0, W, H);
      const laneY = H * 0.34, laneH = H * 0.3;
      const xOf = (x) => pad + ((x + 400) / 800) * (W - pad * 2);

      ctx.strokeStyle = COL.grid;
      ctx.strokeRect(pad, laneY - laneH / 2, W - pad * 2, laneH);
      mono(ctx, 11);
      ctx.fillStyle = COL.dim;
      ctx.fillText('x = −400', pad - 4, laneY + laneH / 2 + 16);
      ctx.fillText('x = +400 — wraps back', W - pad - 130, laneY + laneH / 2 + 16);

      for (const f of flares) {
        const x = fmod(f.x0 + T * f.speed, 800) - 400;
        const px = xOf(x), py = laneY + (f.y - 0.5) * laneH + 2 * Math.sin(T * 7 + f.x0) * 6;
        const grd = ctx.createRadialGradient(px, py, 0, px, py, 7);
        grd.addColorStop(0, 'rgba(240,180,106,0.9)');
        grd.addColorStop(1, 'rgba(240,180,106,0)');
        ctx.fillStyle = grd;
        ctx.beginPath();
        ctx.arc(px, py, 7, 0, TAU);
        ctx.fill();
      }
      ctx.fillStyle = COL.dim;
      ctx.fillText('x(t) = fmod(x₀ + v·m·t, 800) − 400   +  y wobble 2·sin(7t + x₀)', pad, 20);

      // fog factor curve
      const py0 = H * 0.66, ph = H * 0.26;
      ctx.strokeStyle = COL.grid;
      ctx.strokeRect(pad, py0, W - pad * 2, ph);
      ctx.strokeStyle = COL.b;
      ctx.lineWidth = 2;
      ctx.beginPath();
      for (let i = 0; i <= 200; i++) {
        const d = (i / 200) * 700;
        const fog = Math.max(0, Math.min(1, (500 - d) / (500 - 200)));
        const x = pad + (i / 200) * (W - pad * 2);
        const y = py0 + ph - fog * ph;
        i ? ctx.lineTo(x, y) : ctx.moveTo(x, y);
      }
      ctx.stroke();
      ctx.lineWidth = 1;
      ctx.fillStyle = COL.dim;
      ctx.fillText('linear fog: rgb × clamp((500 − d)/(500 − 200)) — full at 200, black at 500 units', pad + 6, py0 + 16);
      ctx.fillText('eye distance →', W - pad - 110, py0 + ph + 16);
    }

    (function loop() {
      if (visible.v) {
        if (!reducedMotion) T += 0.016;
        draw();
      }
      requestAnimationFrame(loop);
    })();
    draw();
  })();

  /* ================================================================
   * TUBES — draggable Catmull-Rom spline + basis functions
   * ================================================================ */
  (function catmullLab() {
    const g = ctx2d('#lab-tubes');
    if (!g) return;
    const { cv, ctx } = g;
    const W = cv.width, H = cv.height;
    const pts = [
      [W * 0.08, H * 0.62], [W * 0.24, H * 0.3], [W * 0.42, H * 0.66],
      [W * 0.58, H * 0.28], [W * 0.76, H * 0.6], [W * 0.92, H * 0.34],
    ];

    function basisWeights(t) {
      // rearrange the kernel as weights on (a, b, c, d)
      return [
        0.5 * (-t * t * t + 2 * t * t - t),
        0.5 * (3 * t * t * t - 5 * t * t + 2),
        0.5 * (-3 * t * t * t + 4 * t * t + t),
        0.5 * (t * t * t - t * t),
      ];
    }

    function draw() {
      ctx.clearRect(0, 0, W, H);
      mono(ctx, 11);

      // control polygon
      ctx.strokeStyle = 'rgba(103,115,127,0.4)';
      ctx.setLineDash([3, 5]);
      ctx.beginPath();
      pts.forEach((p, i) => (i ? ctx.lineTo(p[0], p[1]) : ctx.moveTo(p[0], p[1])));
      ctx.stroke();
      ctx.setLineDash([]);

      // spline through interior segments — endpoint clamping like the
      // tessellator's GetValid* neighbors (repeat the edge node)
      const P = [pts[0], ...pts, pts[pts.length - 1]];
      ctx.strokeStyle = COL.d;
      ctx.lineWidth = 2.5;
      ctx.beginPath();
      for (let s = 0; s < P.length - 3; s++) {
        for (let i = 0; i <= 24; i++) {
          const t = i / 24;
          const x = catmull(P[s][0], P[s + 1][0], P[s + 2][0], P[s + 3][0], t);
          const y = catmull(P[s][1], P[s + 1][1], P[s + 2][1], P[s + 3][1], t);
          (s === 0 && i === 0) ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
        }
      }
      ctx.stroke();
      ctx.lineWidth = 1;

      // tessellation dots at the demo's seed density (6 per segment)
      ctx.fillStyle = COL.f;
      for (let s = 0; s < P.length - 3; s++) {
        for (let i = 0; i < 6; i++) {
          const t = i / 5;
          const x = catmull(P[s][0], P[s + 1][0], P[s + 2][0], P[s + 3][0], t);
          const y = catmull(P[s][1], P[s + 1][1], P[s + 2][1], P[s + 3][1], t);
          ctx.fillRect(x - 1.5, y - 1.5, 3, 3);
        }
      }

      // handles
      pts.forEach((p, i) => {
        ctx.fillStyle = COL.a;
        ctx.beginPath();
        ctx.arc(p[0], p[1], 7, 0, TAU);
        ctx.fill();
        ctx.fillStyle = '#06080a';
        ctx.fillText(String(i), p[0] - 3, p[1] + 4);
      });

      ctx.fillStyle = COL.dim;
      ctx.fillText('drag the control points — the curve always passes through them (C¹ continuous)', 16, 20);

      // basis function inset
      const bx = W - 250, by = H - 120, bw = 220, bh = 92;
      ctx.strokeStyle = COL.grid;
      ctx.strokeRect(bx, by, bw, bh);
      const colors = [COL.e, COL.a, COL.d, COL.b];
      const names = ['w_a', 'w_b', 'w_c', 'w_d'];
      for (let k = 0; k < 4; k++) {
        ctx.strokeStyle = colors[k];
        ctx.beginPath();
        for (let i = 0; i <= 60; i++) {
          const t = i / 60;
          const w = basisWeights(t)[k];
          const x = bx + t * bw;
          const y = by + bh - (w + 0.2) / 1.4 * bh;
          i ? ctx.lineTo(x, y) : ctx.moveTo(x, y);
        }
        ctx.stroke();
        ctx.fillStyle = colors[k];
        ctx.fillText(names[k], bx + 6 + k * 34, by + 14);
      }
      ctx.fillStyle = COL.dim;
      ctx.fillText('the four basis weights: Σw = 1, and w can dip negative', bx - 92, by + bh + 16);
    }

    draggable(cv, pts, 16, draw);
    draw();
  })();

  /* ================================================================
   * TREE — hands-on 2D free-form deformation
   * ================================================================ */
  (function ffdLab() {
    const g = ctx2d('#lab-tree');
    if (!g) return;
    const { cv, ctx } = g;
    const W = cv.width, H = cv.height;
    const N = 5;
    const breatheBtn = $('#ffd-breathe'), resetBtn = $('#ffd-reset');
    const visible = watchVisible(cv);
    let breathing = !reducedMotion, T = 0;

    const cx = W / 2, cy = H / 2, cell = Math.min(W, H) * 0.17;
    const lattice = [];
    function resetLattice() {
      lattice.length = 0;
      for (let y = 0; y < N; y++) {
        for (let x = 0; x < N; x++) {
          lattice.push([cx + (x - 2) * cell, cy + (y - 2) * cell]);
        }
      }
    }
    resetLattice();

    // the embedded shape: a disc of sample points expressed in lattice space
    const shape = [];
    for (let ring = 1; ring <= 4; ring++) {
      for (let i = 0; i < ring * 10; i++) {
        const a = (i / (ring * 10)) * TAU;
        shape.push([2 + Math.cos(a) * ring * 0.42, 2 + Math.sin(a) * ring * 0.42]);
      }
    }
    shape.push([2, 2]);

    function latticeAt(x, y) {
      x = Math.max(0, Math.min(N - 1, x));
      y = Math.max(0, Math.min(N - 1, y));
      return lattice[y * N + x];
    }

    // 2D tensor Catmull-Rom through the 4x4 neighborhood, exactly the
    // structure of CFFD::calc_spline_deform minus one dimension
    function deform(u, v) {
      const nx = Math.floor(u), ny = Math.floor(v);
      const fx = u - nx, fy = v - ny;
      const rows = [];
      for (let j = -1; j <= 2; j++) {
        const p0 = latticeAt(nx - 1, ny + j), p1 = latticeAt(nx, ny + j);
        const p2 = latticeAt(nx + 1, ny + j), p3 = latticeAt(nx + 2, ny + j);
        rows.push([
          catmull(p0[0], p1[0], p2[0], p3[0], fx),
          catmull(p0[1], p1[1], p2[1], p3[1], fx),
        ]);
      }
      return [
        catmull(rows[0][0], rows[1][0], rows[2][0], rows[3][0], fy),
        catmull(rows[0][1], rows[1][1], rows[2][1], rows[3][1], fy),
      ];
    }

    function draw() {
      ctx.clearRect(0, 0, W, H);

      // lattice
      ctx.strokeStyle = 'rgba(192,240,122,0.35)';
      ctx.beginPath();
      for (let y = 0; y < N; y++) {
        for (let x = 0; x < N; x++) {
          const p = lattice[y * N + x];
          if (x < N - 1) { const q = lattice[y * N + x + 1]; ctx.moveTo(p[0], p[1]); ctx.lineTo(q[0], q[1]); }
          if (y < N - 1) { const q = lattice[(y + 1) * N + x]; ctx.moveTo(p[0], p[1]); ctx.lineTo(q[0], q[1]); }
        }
      }
      ctx.stroke();

      // deformed shape
      ctx.fillStyle = COL.a;
      for (const s of shape) {
        const [x, y] = deform(s[0], s[1]);
        ctx.beginPath();
        ctx.arc(x, y, 2.6, 0, TAU);
        ctx.fill();
      }

      // handles
      ctx.fillStyle = '#c0f07a';
      for (const p of lattice) {
        ctx.beginPath();
        ctx.arc(p[0], p[1], 6, 0, TAU);
        ctx.fill();
      }

      mono(ctx, 11);
      ctx.fillStyle = COL.dim;
      ctx.fillText('drag the green lattice — every blue point re-samples through 2 passes of Catmull-Rom', 16, 20);
    }

    (function loop() {
      if (visible.v) {
        if (breathing && !reducedMotion) {
          T += 0.016;
          for (let y = 0; y < N; y++) {
            for (let x = 0; x < N; x++) {
              lattice[y * N + x][0] = cx + (x - 2) * cell * (Math.sin(T * 1.33 + y * 4) * 0.2 + 0.8) / 0.8 * 0.8;
              lattice[y * N + x][1] = cy + (y - 2) * cell * (Math.cos(T * 2 + x * 10) * 0.2 + 0.8) / 0.8 * 0.8;
            }
          }
        }
        draw();
      }
      requestAnimationFrame(loop);
    })();

    draggable(cv, lattice, 14, () => { breathing = false; breatheBtn.textContent = 'breathe'; draw(); });
    if (breatheBtn) {
      breatheBtn.textContent = breathing ? 'freeze' : 'breathe';
      breatheBtn.addEventListener('click', () => {
        breathing = !breathing;
        breatheBtn.textContent = breathing ? 'freeze' : 'breathe';
      });
    }
    if (resetBtn) resetBtn.addEventListener('click', () => { breathing = false; breatheBtn.textContent = 'breathe'; resetLattice(); draw(); });
    draw();
  })();

  /* ================================================================
   * FACEMORPH — counter-rotating texture matrices
   * ================================================================ */
  (function driftLab() {
    const g = ctx2d('#lab-facemorph');
    if (!g) return;
    const { cv, ctx } = g;
    const visible = watchVisible(cv);
    let T = 0;

    function grid(color, tx, ty, rot) {
      const W = cv.width, H = cv.height;
      const S = Math.min(W, H) * 0.8;
      ctx.save();
      ctx.beginPath();
      ctx.rect((W - S) / 2, (H - S) / 2, S, S);
      ctx.clip();
      ctx.translate(W / 2, H / 2);
      ctx.rotate(rot);
      ctx.translate((tx % 1) * S * 0.25, (ty % 1) * S * 0.25);
      ctx.strokeStyle = color;
      ctx.beginPath();
      for (let i = -8; i <= 8; i++) {
        ctx.moveTo(i * S * 0.125, -S);
        ctx.lineTo(i * S * 0.125, S);
        ctx.moveTo(-S, i * S * 0.125);
        ctx.lineTo(S, i * S * 0.125);
      }
      ctx.stroke();
      ctx.restore();
    }

    function draw() {
      const W = cv.width, H = cv.height;
      ctx.clearRect(0, 0, W, H);
      const S = Math.min(W, H) * 0.8;
      ctx.strokeStyle = COL.dim;
      ctx.strokeRect((W - S) / 2, (H - S) / 2, S, S);
      grid('rgba(111,195,255,0.5)', T * 0.05, T * 0.1, T * 30 * Math.PI / 180);
      grid('rgba(240,180,106,0.5)', T * 0.05, -T * 0.05, -T * 20 * Math.PI / 180);
      mono(ctx, 11);
      ctx.fillStyle = COL.dim;
      ctx.fillText('the two UV lattices: +30°/s vs −20°/s — their beat makes the shimmer', 16, 20);
    }

    labs.onTime('facemorph', (t) => { T = t; if (visible.v) draw(); });
    (function loop() {
      if (visible.v) {
        if (!reducedMotion) T += 0.016;
        draw();
      }
      requestAnimationFrame(loop);
    })();
    draw();
  })();

  /* ================================================================
   * Viewer annotations — "show the numbers": the formulas' live values
   * drawn directly over the running effect.
   * ================================================================ */
  (function annotations() {
    if (!labs.annotators) return;
    const clamp01 = (v) => (v < 0 ? 0 : v > 1 ? 1 : v);

    function panelBg(ctx, x, y, w, h) {
      ctx.fillStyle = 'rgba(4,6,10,0.62)';
      ctx.fillRect(x, y, w, h);
    }
    function meter(ctx, x, y, w, label, value, color) {
      mono(ctx, 12);
      ctx.fillStyle = '#9fb0bd';
      ctx.fillText(label, x, y - 6);
      ctx.fillStyle = '#1b2431';
      ctx.fillRect(x, y, w, 8);
      ctx.fillStyle = color;
      ctx.fillRect(x, y, w * clamp01(value), 8);
      ctx.fillStyle = color;
      ctx.fillText(value.toFixed(2), x + w + 8, y + 8);
    }
    function line(ctx, x, y, text, color) {
      mono(ctx, 12);
      ctx.fillStyle = color || '#9fb0bd';
      ctx.fillText(text, x, y);
    }
    const piecewise = (pts) => (t) => {
      if (t <= pts[0][0]) return pts[0][1];
      for (let i = 0; i < pts.length - 1; i++) {
        if (t >= pts[i][0] && t < pts[i + 1][0]) {
          const f = (t - pts[i][0]) / (pts[i + 1][0] - pts[i][0]);
          return pts[i][1] + (pts[i + 1][1] - pts[i][1]) * f;
        }
      }
      return pts[pts.length - 1][1];
    };

    const szEnv = piecewise([[0, 0], [3.3, 0], [6.5, 1], [8, 1], [9, 0], [10, 0], [13, 1], [15, 0], [16, 1], [20, 0], [22, 1], [24, 0]]);
    labs.annotators.spinzoom = (ctx, W, H, t) => {
      panelBg(ctx, 12, 12, 296, 64);
      meter(ctx, 22, 36, 180, 'envelope brightness (the graph below, now)', szEnv(Math.max(0, t - 0.1)), '#6fc3ff');
      line(ctx, 22, 64, 'each quad drawn at α = 0.23 × this', '#67737f');
    };

    const sbKeys = piecewise([[0, 0], [1.5, 1], [5, 1], [7, 0], [9, 1], [14.5, 1], [16, 0], [18, 1], [20, 1], [24, 0]]);
    labs.annotators.shadeball = (ctx, W, H, t) => {
      panelBg(ctx, 12, 12, 316, 64);
      meter(ctx, 22, 36, 180, 'title/ball envelope', clamp01(sbKeys(t)), '#8ad0c8');
      line(ctx, 22, 64, 'shell spacing Δ = ' + (0.014 * (8 * Math.sin(t) + 12)).toFixed(3) + ' s (see comb below)', '#67737f');
    };

    labs.annotators.splines = (ctx, W, H, t) => {
      panelBg(ctx, 12, 12, 306, 64);
      meter(ctx, 22, 36, 180, 'cage sphere alpha 0.3 + 0.1·sin(2t)', 0.3 + 0.1 * Math.sin(t * 2), '#f0e28a');
      line(ctx, 22, 64, '16 trails × 64 sprites, each at p(t − 0.036·i)', '#67737f');
    };

    labs.annotators.ffdenv = (ctx, W, H, t) => {
      const cycle = (t * 0.5) % 6;
      const A = Math.floor(cycle), B = (A + 1) % 6;
      const frac = cycle - A;
      const fT = Math.cos((1 - frac) * Math.PI) * 0.5 + 0.5;
      panelBg(ctx, 12, 12, 306, 84);
      line(ctx, 22, 32, 'morphing shape ' + A + ' → ' + B + '   (the ring below)', '#b48cff');
      meter(ctx, 22, 56, 180, 'mix fT (eased)', fT, '#f0b46a');
      meter(ctx, 22, 82, 180, 'raw t before easing', frac, '#67737f');
    };

    labs.annotators.energystream = (ctx, W, H, t) => {
      const m = t > 38 ? 2.5 : t > 25 ? 2 : 1;
      panelBg(ctx, 12, 12, 322, 64);
      line(ctx, 22, 32, 'phase: speed multiplier m = ' + m + (t > 15 ? '  · texture 2' : '  · texture 1'), '#f0b46a');
      line(ctx, 22, 52, 'flares wrap: x = fmod(x₀ + v·' + m + '·t, 800) − 400', '#9fb0bd');
      line(ctx, 22, 68, 'fog eats everything past 500 units', '#67737f');
    };

    labs.annotators.tubes = (ctx, W, H, t) => {
      const ring = (v) => 50 + 10 * Math.sin(2.14 * t * v * 0.55);
      panelBg(ctx, 12, 12, 322, 64);
      line(ctx, 22, 32, 'ring radii breathe: r(v=1) = ' + ring(1).toFixed(0) + ' · r(v=3) = ' + ring(3).toFixed(0) + ' · r(v=6) = ' + ring(6).toFixed(0), '#8ae0a0');
      line(ctx, 22, 52, 'every frame the whole surface re-splines (drag the', '#9fb0bd');
      line(ctx, 22, 68, 'curve toy below to feel what that means)', '#9fb0bd');
    };

    labs.annotators.polkalike = (ctx, W, H, t) => {
      const phase = t < 10 ? 1 : t < 20 ? 2 : 3;
      panelBg(ctx, 12, 12, 322, 48);
      line(ctx, 22, 32, 'emitter path: phase ' + phase + ' of 3 (see paths below)', '#ff9bb0');
      line(ctx, 22, 52, 'sprite α = −z/300 + 0.2 — born dark, brighten as they near', '#67737f');
    };

    labs.annotators.facemorph = (ctx, W, H, t) => {
      const seq = [1, 0, 3, 0];
      const cycle = (t * 0.5) % 4;
      const A = seq[Math.floor(cycle)], B = seq[(Math.floor(cycle) + 1) % 4];
      const frac = cycle - Math.floor(cycle);
      const fT = Math.cos((1 - frac) * Math.PI) * 0.5 + 0.5;
      panelBg(ctx, 12, 12, 306, 84);
      line(ctx, 22, 32, 'head ' + A + ' → head ' + B + '   (sequence 1→0→3→0)', '#e0a0ff');
      meter(ctx, 22, 56, 180, 'mix fT (eased)', fT, '#f0b46a');
      line(ctx, 22, 82, 'skin UVs rotating +30°/s and −20°/s (grids below)', '#67737f');
    };

    labs.annotators.thing = (ctx, W, H, t) => {
      const global = t > 26.5 ? clamp01(1 + 26.5 - t) : 1;
      const l1 = (t < 8 ? clamp01((8 - t) * 0.3) : 0) * global;
      const l2 = (t >= 8 ? clamp01((t - 16) * 0.3) : 0) * global;
      const l3 = (t < 8 ? 1 - clamp01((8 - t) * 0.3) : 1 - clamp01((t - 16) * 0.3)) * global;
      panelBg(ctx, 12, 12, 306, 104);
      meter(ctx, 22, 36, 180, 'credit: saffron', l1, '#f0e28a');
      meter(ctx, 22, 62, 180, 'credit: radix / lluvia', l3, '#ff9bb0');
      meter(ctx, 22, 88, 180, 'credit: yoghurt', l2, '#b48cff');
    };
  })();
})();
