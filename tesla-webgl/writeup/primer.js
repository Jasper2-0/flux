// The five pictures: a gentle interactive primer for the math the effects
// reuse everywhere. Loads after article.js (uses window.__teslaLabs.pak).

(() => {
  const $ = (s) => document.querySelector(s);
  const labs = window.__teslaLabs || {};
  const reducedMotion = !!labs.reducedMotion;
  const TAU = Math.PI * 2;

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

  /* =========================================================
   * PICTURE 1 — sine is a spinning wheel, seen from the side
   * ========================================================= */
  (function sineWheel() {
    const g = ctx2d('#toy-sine');
    if (!g) return;
    const { cv, ctx } = g;
    const speedIn = $('#sine-speed');
    const visible = watchVisible(cv);
    let angle = 0;
    const trace = [];

    function draw() {
      const W = cv.width, H = cv.height;
      ctx.clearRect(0, 0, W, H);
      const cx = 130, cy = H / 2, R = 78;

      // the wheel
      ctx.strokeStyle = '#67737f';
      ctx.beginPath();
      ctx.arc(cx, cy, R, 0, TAU);
      ctx.stroke();
      // horizontal center line all the way right
      ctx.strokeStyle = '#1b2431';
      ctx.beginPath();
      ctx.moveTo(cx - R - 20, cy);
      ctx.lineTo(W - 20, cy);
      ctx.stroke();

      const px = cx + Math.cos(angle) * R;
      const py = cy - Math.sin(angle) * R;

      // spoke + the height, highlighted
      ctx.strokeStyle = '#67737f';
      ctx.beginPath();
      ctx.moveTo(cx, cy);
      ctx.lineTo(px, py);
      ctx.stroke();
      ctx.strokeStyle = '#f0b46a';
      ctx.lineWidth = 2.5;
      ctx.beginPath();
      ctx.moveTo(px, cy);
      ctx.lineTo(px, py);
      ctx.stroke();
      ctx.lineWidth = 1;

      // the dot
      ctx.fillStyle = '#6fc3ff';
      ctx.beginPath();
      ctx.arc(px, py, 7, 0, TAU);
      ctx.fill();

      // trace: the height, carried off to the right = the sine wave
      trace.unshift(py);
      if (trace.length > W - (cx + R + 60)) trace.pop();
      ctx.strokeStyle = '#6fc3ff';
      ctx.lineWidth = 2;
      ctx.beginPath();
      for (let i = 0; i < trace.length; i++) {
        const x = cx + R + 50 + i;
        i === 0 ? ctx.moveTo(x, trace[i]) : ctx.lineTo(x, trace[i]);
      }
      ctx.stroke();
      ctx.lineWidth = 1;
      // connector
      ctx.strokeStyle = 'rgba(240,180,106,0.35)';
      ctx.setLineDash([3, 4]);
      ctx.beginPath();
      ctx.moveTo(px, py);
      ctx.lineTo(cx + R + 50, py);
      ctx.stroke();
      ctx.setLineDash([]);

      mono(ctx, 12);
      ctx.fillStyle = '#67737f';
      ctx.fillText('a point rides a wheel…', cx - R, cy - R - 14);
      ctx.fillText('…its height over time IS  sin(t)', cx + R + 50, cy - R - 14);
      ctx.fillStyle = '#f0b46a';
      // keep the live readout clear of the wave trace: park it under the wheel
      ctx.fillText('height = sin(angle) = ' + Math.sin(angle).toFixed(2), cx - R, cy + R + 24);
      ctx.fillStyle = '#67737f';
      ctx.fillText('always between −1 and +1 — a smooth, endless wobble', cx + R + 50, H - 16);
    }

    (function loop() {
      if (visible.v) {
        if (!reducedMotion) angle += parseFloat(speedIn.value) * 0.016;
        draw();
      }
      requestAnimationFrame(loop);
    })();
    speedIn.addEventListener('input', draw);
    draw();
  })();

  /* =========================================================
   * PICTURE 2 — mixing (lerp): slide between two shapes
   * ========================================================= */
  (function lerpToy() {
    const g = ctx2d('#toy-lerp');
    if (!g) return;
    const { cv, ctx } = g;
    const tIn = $('#lerp-t');
    const easeIn = $('#lerp-ease');
    const read = $('#lerp-read');

    const N = 48;
    const shapeA = [], shapeB = [];
    for (let i = 0; i < N; i++) {
      const a = (i / N) * TAU;
      shapeA.push([Math.cos(a) * 80, Math.sin(a) * 80]); // circle
      const r = i % 6 < 3 ? 105 : 48; // spiky star
      shapeB.push([Math.cos(a) * r, Math.sin(a) * r]);
    }

    function draw() {
      const W = cv.width, H = cv.height;
      ctx.clearRect(0, 0, W, H);
      let t = parseFloat(tIn.value);
      const eased = easeIn.checked ? Math.cos((1 - t) * Math.PI) * 0.5 + 0.5 : t;

      const centers = [[W * 0.16, H / 2], [W * 0.5, H / 2], [W * 0.84, H / 2]];
      const draws = [
        { pts: shapeA, w: 1, label: 'shape A  (t = 0)', color: '#6fc3ff' },
        { pts: null, w: eased, label: 'the mix', color: '#f0b46a' },
        { pts: shapeB, w: 1, label: 'shape B  (t = 1)', color: '#b48cff' },
      ];
      mono(ctx, 12);
      draws.forEach((d, k) => {
        const [cx, cy] = centers[k];
        ctx.strokeStyle = d.color;
        ctx.lineWidth = k === 1 ? 2.5 : 1.5;
        ctx.beginPath();
        for (let i = 0; i <= N; i++) {
          const j = i % N;
          let x, y;
          if (d.pts) { x = d.pts[j][0]; y = d.pts[j][1]; }
          else {
            x = shapeA[j][0] + (shapeB[j][0] - shapeA[j][0]) * eased;
            y = shapeA[j][1] + (shapeB[j][1] - shapeA[j][1]) * eased;
          }
          i === 0 ? ctx.moveTo(cx + x, cy + y) : ctx.lineTo(cx + x, cy + y);
        }
        ctx.stroke();
        ctx.fillStyle = d.color;
        ctx.fillText(d.label, cx - 40, cy + 128);
      });
      ctx.lineWidth = 1;
      read.textContent = 't = ' + t.toFixed(2) + '  →  every point is ' +
        Math.round((1 - eased) * 100) + '% of A + ' + Math.round(eased * 100) + '% of B' +
        (easeIn.checked ? '  (eased: t reshaped by the cosine curve)' : '');
    }

    tIn.addEventListener('input', draw);
    easeIn.addEventListener('change', draw);
    draw();
  })();

  /* =========================================================
   * PICTURE 3 — a matrix is a recorded move; multiplying = "then do this"
   * ========================================================= */
  (function stackToy() {
    const g = ctx2d('#toy-stack');
    if (!g) return;
    const { cv, ctx } = g;
    const read = $('#stack-read');
    // current pose: angle + scale, plus a history of poses to draw
    let poses = [{ a: 0, s: 1 }];

    function apply(da, ds) {
      const last = poses[poses.length - 1];
      poses.push({ a: last.a + da, s: last.s * ds });
      if (poses.length > 40) poses.shift();
      draw();
    }

    function draw() {
      const W = cv.width, H = cv.height;
      ctx.clearRect(0, 0, W, H);
      ctx.save();
      ctx.translate(W / 2, H / 2);
      ctx.globalCompositeOperation = 'lighter';
      poses.forEach((p, i) => {
        const alpha = 0.12 + 0.88 * (i / (poses.length - 1 || 1));
        ctx.save();
        ctx.rotate(p.a);
        ctx.scale(p.s, p.s);
        ctx.strokeStyle = 'rgba(111,195,255,' + (alpha * 0.9) + ')';
        ctx.lineWidth = i === poses.length - 1 ? 2.5 : 1;
        ctx.strokeRect(-90, -60, 180, 120);
        ctx.restore();
      });
      ctx.restore();
      const last = poses[poses.length - 1];
      read.textContent = 'current pose: turned ' + (last.a * 180 / Math.PI).toFixed(0) +
        '°, size ×' + last.s.toFixed(2) + '  —  ' + (poses.length - 1) + ' moves recorded';
    }

    $('#stack-turn').addEventListener('click', () => apply(12 * Math.PI / 180, 1));
    $('#stack-shrink').addEventListener('click', () => apply(0, 0.9));
    $('#stack-grow').addEventListener('click', () => apply(0, 1.1));
    $('#stack-both').addEventListener('click', () => apply(12 * Math.PI / 180, 0.92));
    $('#stack-x18').addEventListener('click', () => {
      // replay the last move 18 times — this IS SpinZoom's loop
      if (poses.length < 2) return;
      const a = poses[poses.length - 1], b = poses[poses.length - 2];
      const da = a.a - b.a, ds = a.s / b.s;
      for (let i = 0; i < 18; i++) apply(da, ds);
    });
    $('#stack-reset').addEventListener('click', () => { poses = [{ a: 0, s: 1 }]; draw(); });
    draw();
  })();

  /* =========================================================
   * PICTURE 4 — UV coordinates: numbers as addresses into an image
   * ========================================================= */
  (function uvToy() {
    const g = ctx2d('#toy-uv');
    if (!g) return;
    const { cv, ctx } = g;
    let img = null;
    let u = 0.62, v = 0.35;

    // sphere.jpg straight out of the real data.pak — wait for article.js to
    // finish decoding the archive before asking for it
    (labs.ready || Promise.resolve()).then(() => {
      try {
        const bytes = labs.pak.get('data/textures/sphere.jpg');
        const reader = new FileReader();
        reader.onload = () => {
          img = new Image();
          img.onload = draw;
          img.src = reader.result;
        };
        reader.readAsDataURL(new Blob([bytes], { type: 'image/jpeg' }));
      } catch (e) { /* toy still draws the frame */ }
    });

    function draw() {
      const W = cv.width, H = cv.height;
      ctx.clearRect(0, 0, W, H);
      const S = Math.min(W * 0.5, H - 70);
      const x0 = 40, y0 = (H - S) / 2;

      if (img) ctx.drawImage(img, x0, y0, S, S);
      ctx.strokeStyle = '#67737f';
      ctx.strokeRect(x0, y0, S, S);

      mono(ctx, 12);
      ctx.fillStyle = '#67737f';
      ctx.fillText('u=0, v=0', x0 - 4, y0 - 8);
      ctx.fillText('u=1, v=0', x0 + S - 52, y0 - 8);
      ctx.fillText('u=0, v=1', x0 - 4, y0 + S + 16);
      ctx.fillText('u=1, v=1', x0 + S - 52, y0 + S + 16);

      // crosshair at the current address
      const px = x0 + u * S, py = y0 + v * S;
      ctx.strokeStyle = '#f0b46a';
      ctx.beginPath();
      ctx.moveTo(px, y0);
      ctx.lineTo(px, y0 + S);
      ctx.moveTo(x0, py);
      ctx.lineTo(x0 + S, py);
      ctx.stroke();
      ctx.fillStyle = '#f0b46a';
      ctx.beginPath();
      ctx.arc(px, py, 5, 0, TAU);
      ctx.fill();

      ctx.fillStyle = '#c9d4dd';
      const tx = x0 + S + 46;
      ctx.fillText('this spot’s address:', tx, H / 2 - 34);
      ctx.font = '20px ui-monospace, Menlo, monospace';
      ctx.fillStyle = '#f0b46a';
      ctx.fillText('u = ' + u.toFixed(2) + '   v = ' + v.toFixed(2), tx, H / 2 - 4);
      mono(ctx, 12);
      ctx.fillStyle = '#67737f';
      ctx.fillText('every corner of every triangle carries one', tx, H / 2 + 26);
      ctx.fillText('of these addresses — that’s how images', tx, H / 2 + 44);
      ctx.fillText('stick to shapes. (this is sphere.jpg from', tx, H / 2 + 62);
      ctx.fillText('the real data.pak)', tx, H / 2 + 80);
    }

    cv.addEventListener('pointermove', (e) => {
      const r = cv.getBoundingClientRect();
      const W = cv.width, H = cv.height;
      const S = Math.min(W * 0.5, H - 70);
      const x0 = 40, y0 = (H - S) / 2;
      const mx = (e.clientX - r.left) * (W / r.width);
      const my = (e.clientY - r.top) * (H / r.height);
      u = Math.max(0, Math.min(1, (mx - x0) / S));
      v = Math.max(0, Math.min(1, (my - y0) / S));
      draw();
    });
    cv.style.touchAction = 'none';
    draw();
  })();

  /* =========================================================
   * PICTURE 5 — paint vs light: normal alpha vs additive blending
   * ========================================================= */
  (function blendToy() {
    const g = ctx2d('#toy-blend');
    if (!g) return;
    const { cv, ctx } = g;
    const visible = watchVisible(cv);
    let T = 0;

    function blob(x, y, r, color) {
      const grd = ctx.createRadialGradient(x, y, 0, x, y, r);
      grd.addColorStop(0, color);
      grd.addColorStop(1, 'rgba(0,0,0,0)');
      ctx.fillStyle = grd;
      ctx.beginPath();
      ctx.arc(x, y, r, 0, TAU);
      ctx.fill();
    }

    function panel(cx, cy, mode, label, sub) {
      ctx.save();
      ctx.globalCompositeOperation = mode;
      const w = Math.sin(T) * 26;
      blob(cx - 40 + w, cy - 10, 85, 'rgba(111,195,255,0.85)');
      blob(cx + 40 - w, cy - 10, 85, 'rgba(240,180,106,0.85)');
      blob(cx, cy + 34 - w * 0.5, 85, 'rgba(180,140,255,0.85)');
      ctx.restore();
      ctx.globalCompositeOperation = 'source-over';
      mono(ctx, 12);
      ctx.fillStyle = '#c9d4dd';
      ctx.fillText(label, cx - 92, cy + 132);
      ctx.fillStyle = '#67737f';
      ctx.fillText(sub, cx - 92, cy + 150);
    }

    function draw() {
      const W = cv.width, H = cv.height;
      ctx.clearRect(0, 0, W, H);
      panel(W * 0.26, H / 2 - 22, 'source-over', 'PAINT: each layer covers the last',
        'normal alpha blending — overlaps go muddy');
      panel(W * 0.74, H / 2 - 22, 'lighter', 'LIGHT: each layer adds energy',
        'additive blending — overlaps glow toward white');
    }

    (function loop() {
      if (visible.v) {
        if (!reducedMotion) T += 0.012;
        draw();
      }
      requestAnimationFrame(loop);
    })();
    draw();
  })();
})();
