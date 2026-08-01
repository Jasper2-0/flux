#!/usr/bin/env node
// Build the port as a single artifact page: body content only, since the
// artifact host supplies the document skeleton. Same bundling and same
// embedded data tree as build-single.mjs; different chrome.
//
// Usage: node build-artifact.mjs [out.html]

import { readFileSync, writeFileSync, readdirSync, statSync } from 'fs';
import { dirname, join, relative } from 'path';
import { fileURLToPath } from 'url';

const root = dirname(fileURLToPath(import.meta.url));
const out = process.argv[2] || join(root, 'real-artifact.html');

const MODULES = [
  'js/mathlib.js',
  'js/minigl.js',
  'js/scene.js',
  'js/effects.js',
  'js/demo.js',
];

function transform(src) {
  const exported = [];
  src = src.replace(/import\s*\{([^}]*)\}\s*from\s*['"][^'"]+['"];?/g,
                    (_, names) => `const {${names}} = G;`);
  src = src.replace(/^export\s+(?:async\s+)?(class|function|const|let)\s+([A-Za-z0-9_$]+)/gm,
                    (m, kind, name) => { exported.push(name); return m.replace(/^export\s+/, ''); });
  src = src.replace(/^export\s*\{([^}]*)\};?/gm, (_, names) => {
    for (const n of names.split(',')) { const t = n.trim(); if (t) exported.push(t); }
    return '';
  });
  return `(() => {\n${src}\nObject.assign(G, { ${[...new Set(exported)].join(', ')} });\n})();`;
}

const engine = MODULES
  .map((m) => `// ===== ${m} =====\n` + transform(readFileSync(join(root, m), 'utf8')))
  .join('\n\n');
if (engine.includes('</scr' + 'ipt>')) throw new Error('engine contains a script-closing tag');
if (/^\s*export\s/m.test(engine)) throw new Error('an export survived the transform');

const MIME = { '.json': 'application/json', '.jpg': 'image/jpeg', '.png': 'image/png', '.mp3': 'audio/mpeg' };

function walk(dir, acc = []) {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) walk(p, acc); else acc.push(p);
  }
  return acc;
}

const embed = {};
const blocks = [];
let bytes = 0, id = 0;
for (const p of walk(join(root, 'data'))) {
  const rel = relative(root, p).replace(/\\/g, '/');
  const mime = MIME[rel.slice(rel.lastIndexOf('.'))];
  if (!mime) continue;
  const buf = readFileSync(p);
  bytes += buf.length;
  const tag = 'a' + id++;
  embed[rel.replace(/^data\//, '')] = { mime, id: tag };
  blocks.push(`<script type="text/plain" id="${tag}">${buf.toString('base64')}</` + `script>`);
}

const html = `<title>We Ain't Real — Solar, Bizarre 2000</title>
<style>
  /* The palette is the demo's own, sampled from Coat's artwork: the panel
     navy #214080, the amber of the SOLAR logotype #de9000, and black. */
  :root {
    --navy: #214080;
    --navy-lift: #2d539f;
    --amber: #de9000;
    --ground: #05070d;
    --panel: #0a0f1a;
    --ink: #e8edf6;
    --dim: #6d7f99;
    --faint: #34435c;
    --rule: rgba(33, 64, 128, 0.55);
    --screen-frame: #000;
  }
  @media (prefers-color-scheme: light) {
    :root {
      --ground: #eef1f6; --panel: #ffffff; --ink: #101828;
      --dim: #4d6180; --faint: #a9b6c9; --rule: rgba(33, 64, 128, 0.28);
    }
  }
  :root[data-theme="light"] {
    --ground: #eef1f6; --panel: #ffffff; --ink: #101828;
    --dim: #4d6180; --faint: #a9b6c9; --rule: rgba(33, 64, 128, 0.28);
  }
  :root[data-theme="dark"] {
    --ground: #05070d; --panel: #0a0f1a; --ink: #e8edf6;
    --dim: #6d7f99; --faint: #34435c; --rule: rgba(33, 64, 128, 0.55);
  }

  * { box-sizing: border-box; -webkit-tap-highlight-color: transparent; }
  body {
    margin: 0; min-height: 100vh; background: var(--ground); color: var(--ink);
    font: 400 15px/1.6 -apple-system, BlinkMacSystemFont, "Helvetica Neue", Arial, sans-serif;
    -webkit-text-size-adjust: 100%;
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    padding: 24px 16px; gap: 22px; overflow-x: hidden;
  }
  .mono { font-family: ui-monospace, "SF Mono", SFMono-Regular, Menlo, Consolas, monospace; }

  /* The bar with the notched corner is the demo's own furniture — the
     bovenbalk / onderbalk that frame the credits and greets sections. */
  .bar {
    width: min(680px, 100%); height: 26px; background: var(--navy); flex: none;
    clip-path: polygon(0 0, 100% 0, 100% 100%, 62px 100%, 40px 0, 0 0);
    display: flex; align-items: center; gap: 6px; padding-left: 74px;
  }
  .bar.under { clip-path: polygon(0 100%, 100% 100%, 100% 0, 62px 0, 40px 100%); }
  .pip { width: 6px; height: 6px; background: rgba(255,255,255,0.5); flex: none; }
  .pip.lit { background: var(--amber); }

  header { width: min(680px, 100%); text-align: center; display: flex; flex-direction: column; gap: 10px; }
  h1 {
    margin: 0; font-size: clamp(17px, 5vw, 30px); font-weight: 300;
    letter-spacing: 0.32em; text-indent: 0.32em; text-wrap: balance;
    text-transform: uppercase;
  }
  .credit { margin: 0; color: var(--dim); font-size: 12px; letter-spacing: 0.22em; text-transform: uppercase; }
  .credit b { color: var(--amber); font-weight: 500; }
  .note { margin: 0 auto; max-width: 46ch; color: var(--dim); font-size: 13.5px; }

  /* Load progress in the demo's idiom: the little indicator squares that
     run along its panels. Forty of them, so movement is visible. */
  #meter { display: flex; gap: 3px; justify-content: center; flex-wrap: wrap; min-height: 8px; }
  #meter i { width: 8px; height: 8px; background: var(--faint); display: block; }
  #meter i.on { background: var(--navy-lift); }
  #meter i.done { background: var(--amber); }
  #status { color: var(--dim); font-size: 12.5px; letter-spacing: 0.06em; min-height: 1.5em;
            max-width: 44ch; margin: 0 auto; overflow-wrap: anywhere; }

  #play {
    align-self: center; padding: 13px 46px; background: transparent; color: var(--ink);
    border: 1px solid var(--navy-lift); font: inherit; font-size: 14px;
    letter-spacing: 0.34em; text-indent: 0.34em; text-transform: uppercase;
    cursor: pointer; visibility: hidden; transition: background 120ms, color 120ms;
  }
  #play:hover, #play:focus-visible { background: var(--navy); color: #fff; }
  #play:focus-visible { outline: 2px solid var(--amber); outline-offset: 3px; }

  #stage { position: relative; width: min(680px, 100%); display: none; }
  #screen {
    width: 100%; aspect-ratio: 4 / 3; display: block; background: var(--screen-frame);
    touch-action: none; border: 1px solid var(--rule);
  }
  #parts {
    position: absolute; left: 8px; bottom: 8px; font-size: 10px; line-height: 1.65;
    color: #4d5c6b; text-shadow: 0 1px 2px #000; pointer-events: none;
    max-height: 84%; overflow: hidden; display: none;
  }
  #parts .on { color: #a8c8e8; }
  #bits { display: flex; gap: 14px; align-items: center; justify-content: center; flex-wrap: wrap; }
  button.ghost {
    background: none; border: 1px solid var(--rule); color: var(--dim);
    font: inherit; font-size: 11px; letter-spacing: 0.16em; text-transform: uppercase;
    padding: 7px 14px; cursor: pointer;
  }
  button.ghost:hover, button.ghost:focus-visible { color: var(--ink); border-color: var(--navy-lift); }
  button.ghost:focus-visible { outline: 2px solid var(--amber); outline-offset: 2px; }
  .gaps { color: var(--dim); font-size: 12px; max-width: 52ch; text-align: center; margin: 0 auto; }
  .gaps code { color: var(--amber); font-size: 11.5px; }
  @media (prefers-reduced-motion: reduce) { * { transition: none !important; } }
</style>

<div class="bar"><span class="pip lit"></span><span class="pip"></span><span class="pip"></span></div>

<header>
  <h1>We Ain't Real</h1>
  <p class="credit">Solar &middot; Bizarre 2000 &middot; <b>WebGL port</b></p>
  <p class="note">
    The demo's own script drives this, cue for cue, against the shipped
    soundtrack. Graphics by Coat, code by Druid and xotrack, music by
    Tim&nbsp;Brandwijk.
  </p>
</header>

<div id="meter"></div>
<p id="status" class="mono">loading&hellip;</p>
<button id="play">play</button>

<div id="stage">
  <canvas id="screen" width="640" height="480"></canvas>
  <div id="parts" class="mono"></div>
</div>

<div id="bits">
  <button class="ghost" id="layers" type="button">layer list</button>
  <button class="ghost" id="restart" type="button">restart</button>
</div>

<p class="gaps">
  Seven of xotrack's effects are still unported, so roughly 1:28&ndash;2:15
  and the last twenty seconds run mostly empty &mdash;
  <code>show2d</code>, <code>showCylinder</code>, <code>showBol</code>,
  <code>showDraai</code>, <code>showTunnel</code>, <code>showPlanes</code>,
  <code>showPartiekels</code>. Everything else is the demo.
</p>

<div class="bar under"><span class="pip"></span><span class="pip"></span><span class="pip lit"></span></div>

${blocks.join('\n')}
<script>
(() => {
  const say = (w) => { const el = document.getElementById('status'); if (el) el.textContent = w; };
  window.addEventListener('error', (e) =>
    say('error: ' + (e.message || e.error) + (e.lineno ? ' @' + e.lineno : '')));
  window.addEventListener('unhandledrejection', (e) =>
    say('error: ' + ((e.reason && e.reason.message) || e.reason)));
})();

const G = {};
${engine}

const EMBED = ${JSON.stringify(embed)};

(() => {
  const urls = {};
  const bytesOf = (e) => {
    const bin = atob(document.getElementById(e.id).textContent);
    const buf = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) buf[i] = bin.charCodeAt(i);
    return buf;
  };
  const urlOf = (e) => {
    if (!urls[e.id]) {
      urls[e.id] = URL.createObjectURL(new Blob([bytesOf(e)], { type: e.mime }));
      const el = document.getElementById(e.id);
      if (el) el.textContent = '';
    }
    return urls[e.id];
  };
  const lookup = (v) => EMBED[String(v || '').replace(/^data\\//, '')];
  const realFetch = window.fetch.bind(window);
  window.fetch = (url, opts) => {
    const e = lookup(url);
    if (!e) return realFetch(url, opts);
    return Promise.resolve(new Response(bytesOf(e), { headers: { 'Content-Type': e.mime } }));
  };
  const desc = Object.getOwnPropertyDescriptor(HTMLImageElement.prototype, 'src');
  Object.defineProperty(HTMLImageElement.prototype, 'src', {
    set(v) { const e = lookup(v); desc.set.call(this, e ? urlOf(e) : v); },
    get() { return desc.get.call(this); },
  });
  const RealAudio = window.Audio;
  window.Audio = function (src) { const e = lookup(src); return new RealAudio(e ? urlOf(e) : src); };
})();

(async () => {
  const canvas = document.getElementById('screen');
  const stage = document.getElementById('stage');
  const statusEl = document.getElementById('status');
  const playBtn = document.getElementById('play');
  const partsEl = document.getElementById('parts');
  const meter = document.getElementById('meter');

  const CELLS = 40;
  for (let i = 0; i < CELLS; i++) meter.appendChild(document.createElement('i'));
  const cells = meter.querySelectorAll('i');
  const progress = (frac, done) => {
    const n = Math.round(frac * CELLS);
    cells.forEach((c, i) => { c.className = i < n ? (done ? 'done' : 'on') : ''; });
  };

  let showParts = false;
  const demo = new G.Demo(canvas, (m) => {
    statusEl.textContent = m;
    const hit = /(\\d+)\\/(\\d+)/.exec(m);
    if (hit) {
      const base = m.indexOf('scenes') >= 0 ? 0.72 : 0;
      const span = m.indexOf('scenes') >= 0 ? 0.28 : 0.72;
      progress(base + span * (+hit[1] / +hit[2]));
    }
  }, {
    onActiveChange(time, active) {
      if (!showParts) return;
      let html = 't = ' + time.toFixed(1) + 's<br>';
      for (const a of active) {
        html += '<span class="' + (a.done ? 'on' : '') + '">' +
          (a.done ? '\\u25b6 ' : '\\u00b7 ') + String(a.layer).padStart(3, ' ') +
          '  ' + a.command + (a.what ? ' ' + a.what : '') +
          (a.done ? '' : ' (not ported yet)') + '</span><br>';
      }
      partsEl.innerHTML = html;
    },
  });
  window.demo = demo;

  try {
    await demo.load();
    progress(1, true);
    statusEl.textContent = 'ready \\u2014 3:40, sound on';
    playBtn.style.visibility = 'visible';
    playBtn.focus();
  } catch (err) {
    statusEl.textContent = 'error: ' + err.message;
    throw err;
  }

  const start = () => {
    stage.style.display = 'block';
    playBtn.style.visibility = 'hidden';
    statusEl.textContent = 'playing \\u2014 tap the screen to stop';
    demo.start();
  };
  const stop = () => {
    demo.stop();
    statusEl.textContent = 'stopped';
    playBtn.style.visibility = 'visible';
  };

  playBtn.addEventListener('click', start);
  canvas.addEventListener('pointerdown', stop);
  document.getElementById('restart').addEventListener('click', () => { demo.stop(); start(); });
  document.getElementById('layers').addEventListener('click', () => {
    showParts = !showParts;
    partsEl.style.display = showParts ? 'block' : 'none';
  });
  window.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') stop();
    else if (e.key === 'ArrowRight') demo.seek(demo.getTime() + 5);
    else if (e.key === 'ArrowLeft') demo.seek(demo.getTime() - 5);
  });
})();
</script>
`;

writeFileSync(out, html);
console.log(`wrote ${out} (${(html.length / 1048576).toFixed(1)} MB from ${(bytes / 1048576).toFixed(1)} MB of assets)`);
