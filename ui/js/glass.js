// LIQUID GLASS
// O vidro refrata o fundo de verdade: um "mapa de deslocamento" (gerado aqui, por tamanho)
// empurra os pixels perto da borda como uma lente convexa, com leve dispersão de cor (R, G e B
// deslocados um pouco diferente). Por cima vêm o brilho da borda e o reflexo que segue o mouse (CSS).
//
// Isso usa SVG dentro do backdrop-filter, que só o Chromium faz. No Windows o programa roda no
// WebView2 (Chromium), então funciona; em qualquer outro motor o vidro cai para o desfoque simples.
const SVG = 'http://www.w3.org/2000/svg';
let version = 0;

export const lensSupported = () => {
  try { return /Chrome\//.test(navigator.userAgent) && CSS.supports('backdrop-filter', 'url(#x)'); } catch (e) { return false; }
};

/** Mapa: cinza neutro no meio; perto da borda aponta para dentro, mais forte na borda. */
function lensMap(w, h, r, bevel) {
  const c = document.createElement('canvas');
  c.width = w; c.height = h;
  const ctx = c.getContext('2d');
  const img = ctx.createImageData(w, h), d = img.data;
  const cx = w / 2, cy = h / 2, hx = cx - r, hy = cy - r;
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const px = x + .5 - cx, py = y + .5 - cy;
      const qx = Math.abs(px) - hx, qy = Math.abs(py) - hy;
      const ox = Math.max(qx, 0), oy = Math.max(qy, 0);
      const depth = r - (Math.hypot(ox, oy) + Math.min(Math.max(qx, qy), 0)); // distância até a borda, por dentro
      let vx = 0, vy = 0;
      if (depth > 0 && depth < bevel) {
        const t = depth / bevel, m = (1 - t) * (1 - t);
        let nx, ny;                                   // normal para fora
        if (ox > 0 || oy > 0) { const L = Math.hypot(ox, oy) || 1; nx = Math.sign(px) * ox / L; ny = Math.sign(py) * oy / L; }
        else if (qx > qy) { nx = Math.sign(px); ny = 0; } else { nx = 0; ny = Math.sign(py); }
        vx = -nx * m; vy = -ny * m;
      }
      const i = (y * w + x) * 4;
      d[i] = 128 + vx * 127; d[i + 1] = 128 + vy * 127; d[i + 2] = 128; d[i + 3] = 255;
    }
  }
  ctx.putImageData(img, 0, 0);
  return c.toDataURL();
}

const chan = (m, scale, keep) =>
  `<feDisplacementMap in="SourceGraphic" in2="m" scale="${scale}" xChannelSelector="R" yChannelSelector="G" result="d${keep}"/>` +
  `<feColorMatrix in="d${keep}" type="matrix" values="${{ r: '1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 0', g: '0 0 0 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0 1 0', b: '0 0 0 0 0 0 0 0 0 0 0 0 1 0 0 0 0 0 1 0' }[keep]}" result="${keep}"/>`;

function filterNode(id, w, h, url, s) {
  const tmp = document.createElementNS(SVG, 'svg');
  tmp.innerHTML =
    `<filter id="${id}" filterUnits="userSpaceOnUse" x="0" y="0" width="${w}" height="${h}" color-interpolation-filters="sRGB">` +
    `<feImage href="${url}" x="0" y="0" width="${w}" height="${h}" preserveAspectRatio="none" result="m"/>` +
    chan('m', s, 'r') + chan('m', +(s * .93).toFixed(1), 'g') + chan('m', +(s * .86).toFixed(1), 'b') +
    '<feBlend in="r" in2="g" mode="screen" result="rg"/><feBlend in="rg" in2="b" mode="screen"/></filter>';
  return tmp.firstElementChild;
}

/** Cria/atualiza o filtro de um elemento. Usa um id novo a cada vez para o navegador redesenhar. */
function build(el, { key, r, bevel, s }) {
  const w = el.offsetWidth, h = el.offsetHeight;
  if (w < 8 || h < 8) return;
  const id = `lg-${key}-${++version}`;
  const defs = document.getElementById('lg-defs');
  const old = el.dataset.lens;
  defs.appendChild(filterNode(id, w, h, lensMap(w, h, Math.min(r, w / 2, h / 2), bevel), s));
  el.style.setProperty('--lens', `url(#${id})`);
  el.dataset.lens = id;
  if (old) document.getElementById(old)?.remove();
}

const TARGETS = [
  ['#c2', { key: 'c2', r: 26, bevel: 24, s: 34 }],
  ['#c3', { key: 'c3', r: 26, bevel: 24, s: 34 }],
  ['#mb', { key: 'mb', r: 18, bevel: 18, s: 28 }],
];

export function initGlass() {
  const root = document.documentElement;
  const on = lensSupported();
  root.classList.toggle('lg', on);

  if (on) {
    const pending = new Map();
    const ro = new ResizeObserver(entries => {
      for (const e of entries) pending.set(e.target, e.target._lg);
      requestAnimationFrame(() => { pending.forEach((cfg, el) => build(el, cfg)); pending.clear(); });
    });
    for (const [sel, cfg] of TARGETS) {
      const el = document.querySelector(sel);
      el._lg = cfg;
      ro.observe(el);
    }
  }
}
