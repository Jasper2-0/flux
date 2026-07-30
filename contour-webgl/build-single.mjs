#!/usr/bin/env node
// Self-contained single-file build of the Contour port scaffold: all JS
// modules bundled into one namespace, and every asset the runtime uses
// (timeline, textures, scenes, soundtrack) embedded as base64.
//
// Usage: node build-single.mjs [out.html]

import { readFileSync, writeFileSync } from 'fs';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const root = dirname(fileURLToPath(import.meta.url));
const out = process.argv[2] || join(root, 'contour-single.html');

const MODULES = [
  'js/mathlib.js',
  'js/minigl.js',
  'js/arse.js',
  'js/effects/bloem.js',
  'js/effects/rogplay.js',
  'js/effects/timscene.js',
  'js/effects/zoomer.js',
  'js/effects/quickparts.js',
  'js/demo.js',
];

function transform(src) {
  const exported = [];
  src = src.replace(/import\s*\{([^}]*)\}\s*from\s*['"][^'"]+['"];?/g, (_, names) => `const {${names}} = G;`);
  src = src.replace(/^export\s+(class|function|const|let)\s+([A-Za-z0-9_$]+)/gm, (_, kind, name) => {
    exported.push(name);
    return `${kind} ${name}`;
  });
  return `(() => {\n${src}\nObject.assign(G, { ${exported.join(', ')} });\n})();`;
}

const engine = MODULES
  .map((m) => `// ===== ${m} =====\n` + transform(readFileSync(join(root, m), 'utf8')))
  .join('\n\n');

// everything demo.js touches at runtime
const ASSETS = [
  ['data/timeline.json', 'application/json'],
  ['data/saftext/fx4.jpg', 'image/jpeg'],
  ['data/saftext/fx8.jpg', 'image/jpeg'],
  ['data/saftext/fx9.jpg', 'image/jpeg'],
  ['data/textures/flare1.jpg', 'image/jpeg'],
  ['data/textures/flare8.jpg', 'image/jpeg'],
  ['data/textures/dildo.jpg', 'image/jpeg'],
  ['data/logo.jpg', 'image/jpeg'],
  ['data/contourlogo-overlay-image01.jpg', 'image/jpeg'],
  ['data/credits/credbk-1-image.jpg', 'image/jpeg'],
  ['data/credits/credbk-2-image.jpg', 'image/jpeg'],
  ['data/credits/credbk-3-image.jpg', 'image/jpeg'],
  ['data/credits/credbk-4-image.jpg', 'image/jpeg'],
  ['data/credits/credbk-5-image.jpg', 'image/jpeg'],
  ['data/zoomer/image0a.jpg', 'image/jpeg'],
  ['data/zoomer/image1a.jpg', 'image/jpeg'],
  ['data/zoomer/image2a.jpg', 'image/jpeg'],
  ['data/zoomer/image3a.jpg', 'image/jpeg'],
  ['data/zoomer/image4a.jpg', 'image/jpeg'],
  ['data/zoomer/image5a.jpg', 'image/jpeg'],
  ['data/zoomer/image6a.jpg', 'image/jpeg'],
  ['data/zoomer/image7a.jpg', 'image/jpeg'],
  ['data/zoomer/image8a.jpg', 'image/jpeg'],
  ['data/zoomer/image9a.jpg', 'image/jpeg'],
  ['data/zoomer/shun-compo.jpg', 'image/jpeg'],
  ['data/scenes/dildo.bin', 'application/octet-stream'],
  ['data/scenes/neuron.bin', 'application/octet-stream'],
  ['data/test.mp3', 'audio/mpeg'],
];
const embed = {};
for (const [path, mime] of ASSETS) {
  embed[path] = { mime, b64: readFileSync(join(root, path)).toString('base64') };
}
const embedJson = JSON.stringify(embed);

if (engine.includes('</scr' + 'ipt>')) throw new Error('engine contains a script-closing tag');

const html = [
  `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Contour by The Black Lotus — WebGL port scaffold</title>
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<style>
  html, body {
    margin: 0;
    height: 100%;
    background: #000;
    color: #9aa;
    font-family: "Lucida Console", Monaco, monospace;
    overflow: hidden;
    -webkit-text-size-adjust: 100%;
  }
  #wrap {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 16px;
  }
  #stage { position: relative; display: none; }
  #screen {
    background: #000;
    width: min(100vw, calc(92vh * 4 / 3));
    aspect-ratio: 4 / 3;
    touch-action: none;
  }
  #parts {
    position: absolute;
    left: 10px;
    bottom: 10px;
    font-size: 10px;
    line-height: 1.65;
    color: #667;
    text-shadow: 0 1px 2px #000;
    pointer-events: none;
    max-height: 85%;
    overflow: hidden;
  }
  #parts .on { color: #8fd0a0; }
  #parts .stub { color: #776; }
  #panel { text-align: center; max-width: 38em; line-height: 1.6; padding: 0 16px; }
  #panel h1 { font-size: 18px; font-weight: normal; letter-spacing: 0.5em; color: #dde; margin-bottom: 4px; }
  #panel .sub { font-size: 12px; }
  #status { font-size: 12px; color: #667; min-height: 1.6em; }
  #play {
    margin-top: 12px;
    padding: 12px 46px;
    background: transparent;
    color: #dde;
    border: 1px solid #556;
    font: inherit;
    letter-spacing: 0.3em;
    cursor: pointer;
    display: none;
  }
  #play:hover { background: #112; }
  #hint { font-size: 11px; color: #445; margin-top: 10px; }
</style>
</head>
<body>
<div id="wrap">
  <div id="panel">
    <h1>C O N T O U R</h1>
    <div class="sub">the black lotus, 1999 &mdash; webgl port scaffold</div>
    <div class="sub" style="color:#665; margin-top:8px">
      work in progress: the timeline extracted from contour.exe plays against
      the shipped soundtrack; implemented parts render, the rest are listed
      live in the corner as the demo creates them
    </div>
    <div id="status">loading…</div>
    <button id="play">play</button>
    <div id="hint">runs 3:14 · sound on · tap the screen to stop</div>
  </div>
  <div id="stage">
    <canvas id="screen" width="960" height="720"></canvas>
    <div id="parts"></div>
  </div>
</div>
<script>
const G = {};
`,
  engine,
  `
const EMBED = `, embedJson, `;

(async () => {
  const canvas = document.getElementById('screen');
  const stage = document.getElementById('stage');
  const panel = document.getElementById('panel');
  const statusEl = document.getElementById('status');
  const playBtn = document.getElementById('play');
  const partsEl = document.getElementById('parts');

  const demo = new G.Demo(canvas, (msg) => { statusEl.textContent = msg; }, {
    embed: EMBED,
    onActiveChange(time, active) {
      let html = 't = ' + time.toFixed(1) + 's<br>';
      for (const a of active) {
        const cls = a.rendering ? 'on' : 'stub';
        const mark = a.rendering ? '▶ ' : '· ';
        const note = a.implemented ? (a.rendering ? '' : ' (idle)') : ' (not ported yet)';
        html += '<span class="' + cls + '">' + mark + a.name + ' #' + a.id + note + '</span><br>';
      }
      partsEl.innerHTML = html;
    },
  });
  window.demo = demo;

  try {
    await demo.load();
    statusEl.textContent = 'ready — scaffold build, 3:14 soundtrack';
    playBtn.style.display = 'inline-block';
  } catch (err) {
    statusEl.textContent = 'error: ' + err.message;
    throw err;
  }

  const stop = () => {
    demo.stop();
    stage.style.display = 'none';
    panel.style.display = 'block';
    statusEl.textContent = 'stopped — tap play to restart';
  };

  playBtn.addEventListener('click', () => {
    panel.style.display = 'none';
    stage.style.display = 'block';
    demo.start();
  });
  canvas.addEventListener('pointerdown', stop);
  window.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') stop();
    else if (e.key === 'ArrowRight') demo.seek(demo.getTime() + 5);
    else if (e.key === 'ArrowLeft') demo.seek(demo.getTime() - 5);
  });
})();
</script>
</body>
</html>
`,
].join('');

writeFileSync(out, html);
console.log(`wrote ${out} (${(html.length / 1024 / 1024).toFixed(1)} MB)`);
