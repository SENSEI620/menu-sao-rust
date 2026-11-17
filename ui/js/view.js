// Desenho da tela. Os elementos são reaproveitados (não recriados a cada movimento),
// por isso a barra dourada desliza, os painéis acompanham a esfera e só o que é novo "nasce".
import { S, $, isOut } from './state.js';
import { actions, docks } from './registry.js';
import { fmtD, short } from './time.js';
import { ico } from './icons.js';

const STEP = 64;          // altura de um item (58) + espaço (6)
const PAD = 8;            // respiro interno do painel
const ORB_STEP = 78;      // esfera (62) + espaço (16)
const MAX_SEG = 14;       // segmentos do anel de status

/** Esferas da 1ª coluna: perfis e depois as esferas extras (+, ajustes...). */
export const orbs = () => [
  ...S.P.map(p => ({ key: 'p' + p.id, kind: 'profile', p })),
  ...docks.map(d => ({ key: 'd' + d.id, kind: 'dock', d })),
];
export const curP = () => (S.pi >= 0 && S.pi < S.P.length ? S.P[S.pi] : null);
export const curB = () => { const p = curP(); return p ? p.blocks[S.bi] : null; };
export const actionsFor = (b, p) => actions.filter(a => !a.when || a.when(b, p));

/** Atualiza `container` para ficar igual a `rows`, reaproveitando elementos pela chave. */
function sync(container, rows, keyOf, create, update) {
  const have = new Map();
  for (const el of container.children) have.set(el.dataset.k, el);
  rows.forEach((row, i) => {
    const k = keyOf(row);
    let el = have.get(k);
    if (el) have.delete(k);
    else { el = create(row, i); el.dataset.k = k; }
    update(el, row, i);
    if (container.children[i] !== el) container.insertBefore(el, container.children[i] || null);
  });
  have.forEach(el => el.remove());
}

function replay(el, cls) {
  el.classList.remove(cls);
  void el.offsetWidth;
  el.classList.add(cls);
}

const once = (el, cls) => el.addEventListener('animationend', function f(e) {
  if (e.target !== el) return;
  el.classList.remove(cls);
  el.removeEventListener('animationend', f);
});

/** Ícone em texto (emoji) ou em SVG. */
function setIcon(el, icon) {
  if (el.dataset.ic === icon) return;
  el.dataset.ic = icon;
  if (icon.startsWith('<svg')) el.innerHTML = icon; else el.textContent = icon;
}

/** Uma linha pequena com ícone + texto (o texto entra como texto, nunca como HTML). */
function line(el, icon, text) {
  if ((el.dataset.t || '') === text) return;
  el.dataset.t = text;
  if (!text) { el.replaceChildren(); return; }
  el.innerHTML = ico(icon) + '<span></span>';
  el.lastChild.textContent = text;
}

const hue = s => { let h = 7; for (const c of s) h = (h * 31 + c.codePointAt(0)) >>> 0; return h % 360; };

/** Anel de status em volta da esfera: um segmento por link; os bloqueados acendem em vermelho. */
function ringHTML(blocks) {
  if (!blocks.length) return '';
  const total = blocks.length, n = Math.min(total, MAX_SEG);
  const out = total > MAX_SEG ? Math.round(blocks.filter(isOut).length / total * n) : null;
  const step = 100 / n, len = n === 1 ? 100 : Math.max(step - 5.8, .6);
  let s = '';
  for (let i = 0; i < n; i++) {
    const blocked = out === null ? isOut(blocks[i]) : i < out;
    s += `<circle class="${blocked ? 'out' : ''}" style="--s:${i}" cx="40" cy="40" r="37" pathLength="100" ` +
      `stroke-linecap="${n === 1 ? 'butt' : 'round'}" stroke-dasharray="${len.toFixed(2)} 100" ` +
      `stroke-dashoffset="${(-(i * step + (n === 1 ? 0 : 2.9))).toFixed(2)}"/>`;
  }
  return s;
}

const view = { k2: '', k3: '' };

/** Esquece o que estava na tela (ao abrir o menu, tudo nasce de novo). */
export function resetView() {
  view.k2 = view.k3 = '';
  $('#c1').replaceChildren();
  for (const id of ['#c2', '#c3']) {
    $(id).classList.remove('show', 'unroll');
    $(id + ' .list').replaceChildren();
  }
}

export function render({ pop = false } = {}) {
  const list = orbs(), n = list.length;

  // ── coluna 1 ──
  sync($('#c1'), list, o => o.key,
    (o, i) => {
      const el = document.createElement('button');
      el.className = 'orb glass' + (pop ? ' pop' : '');
      el.dataset.l = '0';
      el.innerHTML = '<span class="ico"></span><svg class="ring" viewBox="0 0 80 80" aria-hidden="true"></svg><span class="tip"></span>';
      if (pop) { el.style.setProperty('--d', n - 1 - i); once(el, 'pop'); }
      return el;
    },
    (el, o, i) => {
      const [icon, ring, tip] = el.children;
      const label = o.kind === 'profile' ? o.p.name : o.d.label;
      el.dataset.i = i;
      el.setAttribute('aria-label', label);
      setIcon(icon, o.kind === 'profile' ? o.p.icon : o.d.icon);
      tip.textContent = label;
      el.classList.toggle('dock', o.kind === 'dock');
      const sel = S.pi === i;
      el.classList.toggle('sel', sel);
      const html = o.kind === 'profile' ? ringHTML(o.p.blocks) : '';
      if (ring.dataset.sig !== html) { ring.dataset.sig = html; ring.innerHTML = html; }
    });

  const p = curP(), b = curB();
  const cy = S.pi * ORB_STEP + 31;   // centro da esfera escolhida

  // ── coluna 2: links do perfil ──
  const c2 = $('#c2');
  if (p) {
    const k2 = 'p' + p.id;
    const fresh = view.k2 !== k2;
    const rows = [...p.blocks.map(x => ({ b: x })), { add: true }];
    sync(c2.querySelector('.list'), rows, r => (r.add ? 'add' : 'b' + r.b.id), itemEl('1'),
      (el, r, i) => {
        el.dataset.i = i;
        el.classList.toggle('sel', S.bi === i);
        const [av, tt, pill] = el.children, [t, s, nt] = tt.children;
        if (r.add) {
          av.className = 'av add';
          setIcon(av.firstChild, ico('plus'));
          t.textContent = 'Adicionar link';
          line(s, '', ''); line(nt, '', '');
          pill.textContent = ''; pill.className = 'pill'; delete pill.dataset.b;
          el.classList.remove('out', 'note');
          return;
        }
        const x = r.b, out = isOut(x);
        av.className = 'av tile';
        av.style.setProperty('--h', hue(x.t));
        av.firstChild.textContent = (x.t[0] || '?').toUpperCase();
        av.dataset.ic = '';
        t.textContent = x.t;
        line(s, 'clock', out ? 'volta ' + fmtD(x.until) : '');
        line(nt, 'note', x.note || '');
        pill.textContent = out ? short(x.until - Date.now()) : '';
        pill.className = 'pill' + (out ? ' on' : '');
        if (out) pill.dataset.b = x.id; else delete pill.dataset.b;
        el.classList.toggle('out', out);
        el.classList.toggle('note', !!x.note);
      });
    place(c2, S.bi, cy);
    c2.classList.toggle('dim', S.lv < 1);
    if (fresh) { view.k2 = k2; replay(c2, 'unroll'); }
    c2.classList.add('show');
  } else {
    view.k2 = '';
    c2.classList.remove('show', 'unroll');
  }

  // ── coluna 3: ações do link ──
  const c3 = $('#c3');
  if (S.lv === 2 && p && b) {
    const k3 = 'b' + b.id;
    const fresh = view.k3 !== k3;
    const acts = actionsFor(b, p);
    sync(c3.querySelector('.list'), acts, a => a.id, itemEl('2'),
      (el, a, i) => {
        el.dataset.i = i;
        el.classList.toggle('sel', S.ai === i);
        el.dataset.tone = a.tone || '';
        const [av, tt] = el.children;
        av.className = 'av act';
        setIcon(av.firstChild, a.icon);
        tt.firstChild.textContent = a.label;
      });
    place(c3, S.ai, cy);
    if (fresh) { view.k3 = k3; replay(c3, 'unroll'); }
    c3.classList.add('show');
  } else {
    view.k3 = '';
    c3.classList.remove('show', 'unroll');
  }
}

/** Molde de um item: miniatura, textos e etiqueta de tempo. */
function itemEl(level) {
  return (r, i) => {
    const el = document.createElement('button');
    el.className = 'item enter';
    el.dataset.l = level;
    el.innerHTML = '<span class="av"><b></b><i class="bd bd-out"></i><i class="bd bd-note"></i></span>' +
      '<span class="tt"><span class="t"></span><small class="s"></small><small class="n"></small></span><span class="pill"></span>';
    el.style.setProperty('--d', i);
    once(el, 'enter');
    return el;
  };
}

/** Posiciona o painel para o item escolhido ficar na altura da esfera, e a barra dourada nele. */
function place(panel, idx, cy) {
  panel.style.top = cy - (PAD + idx * STEP + 29) + 'px';
  const bar = panel.querySelector('.selbar');
  bar.style.top = PAD + Math.max(idx, 0) * STEP + 'px';
  bar.classList.toggle('off', idx < 0);
}

/** Atualiza só os números da contagem regressiva (chamado a cada segundo). */
export function tickCountdowns() {
  document.querySelectorAll('.pill[data-b]').forEach(el => {
    for (const p of S.P) {
      const b = p.blocks.find(x => x.id === el.dataset.b);
      if (b && isOut(b)) el.textContent = short(b.until - Date.now());
    }
  });
}

