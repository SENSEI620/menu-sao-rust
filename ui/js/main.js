// Ponto de entrada: abrir/fechar o menu, navegação por teclado/roda/mouse e cronômetros.
import { api, isTauri } from './bridge.js';
import { S, $, clamp, save, isOut, menuIsOpen, modalIsOpen } from './state.js';
import { loadSounds, play, wake } from './audio.js';
import { render, resetView, orbs, curP, curB, actionsFor, tickCountdowns, makeMotes } from './view.js';
import { ask, closeModal, editBlock, initDialogs, toast } from './dialogs.js';
import { emit } from './registry.js';
import { initGlass } from './glass.js';
import './plugins/index.js';

// ───────── Contexto entregue às ações dos recursos ─────────
function makeCtx() {
  return {
    S, api, p: curP(), b: curB(),
    ask, toast, play, save, render, closeMenu, editBlock,
    backToLinks: () => { S.lv = 1; S.ai = -1; render(); },
  };
}

// ───────── Navegação ─────────
function move(d) {
  const before = [S.pi, S.bi, S.ai].join();
  if (S.lv === 0) {
    S.pi = clamp(S.pi < 0 ? 0 : S.pi + d, 0, orbs().length - 1);
    S.bi = S.pi < S.P.length ? 0 : -1;
  } else if (S.lv === 1) S.bi = clamp(S.bi + d, 0, curP().blocks.length);
  else S.ai = clamp(S.ai + d, 0, actionsFor(curB(), curP()).length - 1);
  if ([S.pi, S.bi, S.ai].join() !== before) { play('tick'); render(); }
}

function enter() {
  if (S.lv === 0) {
    if (S.pi < 0) return;
    const o = orbs()[S.pi];
    if (o.kind === 'dock') return o.d.run(makeCtx());
    S.lv = 1; S.bi = 0; play('confirm'); render();
  } else if (S.lv === 1) {
    const p = curP();
    if (S.bi === p.blocks.length) return editBlock(p, null);
    S.lv = 2; S.ai = 0; play('confirm'); render();
  } else act();
}

function back() {
  if (S.lv === 2) { S.lv = 1; S.ai = -1; play('tick'); render(); }
  else if (S.lv === 1) { S.lv = 0; play('tick'); render(); }
  else if (S.pi >= 0) { S.pi = S.bi = -1; play('tick'); render(); }
  else closeMenu();
}

function act() {
  const a = actionsFor(curB(), curP())[S.ai];
  if (a) a.run(makeCtx());
}

// ───────── Abrir e fechar ─────────
let autoSelect = 0;

function openMenu() {
  if (S.busy || menuIsOpen()) return;
  wake();
  S.pi = S.bi = S.ai = -1; S.lv = 0;
  resetView();
  makeMotes();
  $('#home').hidden = true;
  render({ pop: true });
  $('#menu').className = 'on' + ($('#menu').dataset.bg ? '' : ' nobg');
  S.openedAt = performance.now();
  play('enter');
  emit('open');
  // depois que o último ícone nasce, o primeiro perfil fica dourado sozinho
  clearTimeout(autoSelect);
  autoSelect = setTimeout(() => {
    if (menuIsOpen() && !$('#menu').classList.contains('closing') && S.pi === -1 && !modalIsOpen() && S.P.length) {
      S.pi = 0; S.bi = 0; S.lv = 0; render();
    }
  }, orbs().length * 110 + 380);
}

function closeMenu() {
  if (S.busy || !menuIsOpen()) return;
  S.busy = true;
  hideCtx();
  play('exit');
  emit('close');
  $('#menu').classList.add('closing');
  setTimeout(() => { $('#menu').className = ''; S.busy = false; api.hide(); }, 45 * orbs().length + 420);
}

// ───────── Teclado e roda ─────────
addEventListener('contextmenu', e => e.preventDefault());
addEventListener('keydown', e => {
  if (modalIsOpen()) { if (e.key === 'Escape') closeModal(); return; }
  if (!menuIsOpen()) return;
  const k = e.key;
  if (k === 'Escape') closeMenu();
  else if (k === 'ArrowDown') { e.preventDefault(); move(1); }
  else if (k === 'ArrowUp') { e.preventDefault(); move(-1); }
  else if (k === 'ArrowRight' || k === 'Enter') { e.preventDefault(); enter(); }
  else if (k === 'ArrowLeft' || k === 'Backspace') { e.preventDefault(); back(); }
});

let lastWheel = 0;
$('#menu').addEventListener('wheel', e => {
  e.preventDefault();
  const t = performance.now();
  if (t - lastWheel < 90) return;
  lastWheel = t;
  move(e.deltaY > 0 ? 1 : -1);
}, { passive: false });

// ───────── Mouse ─────────
$('#menu').addEventListener('click', e => {
  if (performance.now() - S.openedAt < 500) return;
  hideCtx();
  const o = e.target.closest('[data-l]');
  if (!o) { if (!e.target.closest('.panel')) closeMenu(); return; }
  const l = +o.dataset.l, i = +o.dataset.i;

  if (l === 0) {
    const orb = orbs()[i];
    if (S.pi === i && S.lv >= 1 && orb.kind === 'profile') { S.pi = S.bi = S.ai = -1; S.lv = 0; play('tick'); render(); return; }
    S.pi = i; S.bi = orb.kind === 'profile' ? 0 : -1; S.ai = -1; S.lv = 0;
    play('confirm'); render();
    if (orb.kind === 'dock') orb.d.run(makeCtx());
    else { S.lv = 1; render(); }
  } else if (l === 1) {
    if (S.bi === i && S.lv >= 1) { S.lv = 1; enter(); }
    else { S.bi = i; S.ai = -1; S.lv = 1; play('tick'); render(); if (i === curP().blocks.length) enter(); }
  } else {
    S.ai = i; S.lv = 2; play('tick'); render(); act();
  }
});

$('#menu').addEventListener('mouseover', e => {
  const o = e.target.closest('[data-l="0"]');
  if (o && !o.contains(e.relatedTarget)) play('confirm', .7);
});

// clique direito numa esfera de perfil: editar ou excluir
const ctxMenu = $('#ctx');
function hideCtx() { ctxMenu.classList.remove('on'); }
$('#menu').addEventListener('contextmenu', e => {
  e.preventDefault();
  const o = e.target.closest('[data-l="0"]');
  if (!o) return;
  const orb = orbs()[+o.dataset.i];
  if (!orb || orb.kind !== 'profile') return;
  ctxMenu.innerHTML = '<button data-a="e">Editar perfil</button><button data-a="d">Excluir perfil</button>';
  ctxMenu.dataset.p = orb.p.id;
  ctxMenu.classList.add('on');
  ctxMenu.style.left = Math.min(e.clientX, innerWidth - 200) + 'px';
  ctxMenu.style.top = Math.min(e.clientY, innerHeight - 100) + 'px';
});
ctxMenu.addEventListener('click', async e => {
  const a = e.target.dataset.a;
  if (!a) return;
  hideCtx();
  const p = S.P.find(x => x.id === ctxMenu.dataset.p);
  if (!p) return;
  const { editProfile } = await import('./dialogs.js');
  if (a === 'e') editProfile(p);
  else ask('Excluir perfil', `Excluir “${p.name}” e os links dele?`, [], () => {
    S.P = S.P.filter(x => x !== p);
    S.pi = S.bi = S.ai = -1; S.lv = 0; save(); render();
  }, 'Excluir');
});

// ───────── Cronômetros (só enquanto o menu está aberto) ─────────
setInterval(() => {
  if (!menuIsOpen()) return;
  let freed = false;
  S.P.forEach(p => p.blocks.forEach(b => { if (b.until && b.until <= Date.now()) { b.until = 0; freed = true; } }));
  if (freed) { play('confirm'); save(); render(); return; }
  tickCountdowns();
}, 1000);

// ───────── Início ─────────
initDialogs();
initGlass();
loadSounds();
// o fundo desliza de leve com o mouse (parallax)
$('#menu').addEventListener('pointermove', e => {
  const m = $('#menu');
  m.style.setProperty('--px', ((e.clientX / innerWidth - .5) * -14).toFixed(1) + 'px');
  m.style.setProperty('--py', ((e.clientY / innerHeight - .5) * -14).toFixed(1) + 'px');
}, { passive: true });
if (!isTauri) {
  document.body.classList.add('mock');
  $('#home').hidden = false;
  $('#homeOpen').onclick = () => api.reopen();
}
api.onToast(msg => toast(msg));
api.onShow(d => {
  $('#menu').className = '';
  S.busy = false;
  hideCtx();
  S.P = Array.isArray(d.config) ? d.config : [];
  if (d.settings) S.settings = d.settings;
  const menu = $('#menu');
  if (d.bg) {
    menu.dataset.bg = '1';
    $('#bg').style.backgroundImage = 'url(' + d.bg + ')';
    const im = new Image();
    im.onload = im.onerror = openMenu;   // só abre depois da foto estar pronta
    im.src = d.bg;
  } else {
    delete menu.dataset.bg;
    openMenu();
  }
});
