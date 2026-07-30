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
    const visible = watchVisible(cv);
    let phase = 0;

    function draw() {
      const s = parseFloat(sIn.value), th = parseFloat(thIn.value) * Math.PI / 180;
      const W = cv.width, H = cv.height;
      ctx.clearRect(0, 0, W, H);
      ctx.save();
      ctx.translate(W / 2, H / 2);
      ctx.globalCompositeOperation = 'lighter';
      let scale = Math.min(W, H) * 0.46, ang = phase;
      for (let i = 0; i < 18; i++) {
        ctx.save();
        ctx.rotate(ang);
        ctx.strokeStyle = 'rgba(111,195,255,0.5)';
        ctx.fillStyle = 'rgba(111,195,255,0.055)';
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.rect(-scale, -scale * 0.75, scale * 2, scale * 1.5);
        ctx.fill();
        ctx.stroke();
        ctx.restore();
        // the demo multiplies the current matrix by R(θ)·S(s) each copy, so
        // copy i carries rotation i·θ and scale s^i — but s > 1 shrinks here
        // because we walk outward-in for readability
        ang += th;
        scale /= s === 0 ? 1 : Math.pow(s, 1.4);
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
    sIn.addEventListener('input', draw);
    thIn.addEventListener('input', draw);
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
      const window0 = T - 6, window1 = T + 0.5;
      const xOf = (t) => pad + ((t - window0) / (window1 - window0)) * (W - pad * 2);

      ctx.strokeStyle = COL.grid;
      ctx.beginPath();
      ctx.moveTo(pad, H - 34);
      ctx.lineTo(W - pad, H - 34);
      ctx.stroke();
      ctx.fillStyle = COL.dim;
      mono(ctx, 10);
      for (let t = Math.ceil(window0); t <= window1; t++) {
        ctx.fillText(t.toFixed(0) + 's', xOf(t) - 6, H - 18);
        ctx.fillRect(xOf(t), H - 38, 1, 8);
      }

      // the 20 shells: each drawn at an earlier time, dimmer and smaller
      let t = T;
      const step = 0.014 * (8 * Math.sin(T) + 12);
      for (let i = 0; i < 20; i++) {
        const alpha = Math.max(0, 1 - i / 12);
        const x = xOf(t);
        const h = 18 + alpha * 52;
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
      ctx.fillStyle = COL.e;
      ctx.fillText('α = 1 − i/12 goes negative past i = 12 → shells 13…19 clamp invisible', pad, 32);
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
})();
