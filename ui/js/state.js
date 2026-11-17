import { api } from './bridge.js';

export const $ = s => document.querySelector(s);
export const clamp = (n, a, b) => Math.max(a, Math.min(b, n));
export const uid = () => Math.random().toString(36).slice(2, 9);
export const esc = s => String(s).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

/** Estado do menu: pi = esfera escolhida, bi = link, ai = ação, lv = coluna ativa (0, 1 ou 2). */
export const S = {
  P: [],
  pi: -1, bi: -1, ai: -1, lv: 0,
  settings: { shortcut: 'Control+Alt+M', mouseGesture: true, holdMs: 1000, volume: 100, firefoxNoRemote: false, autostart: false, packaged: true },
  busy: false,
  openedAt: 0,
};

export const isOut = b => b.until > Date.now();
export const pstate = p => {
  const r = p.blocks.filter(isOut).length;
  return !r ? '' : r === p.blocks.length ? 'full' : 'half';
};
export const mkBlock = (t, url) => ({ id: uid(), t, url, until: 0, note: '' });
export const mkProfile = (name, icon, br, pf) => ({ id: uid(), name, icon, br, pf, blocks: [] });

export const save = () => { try { api.saveConfig(S.P); } catch (e) { /* o log do programa mostra */ } };
export const menuIsOpen = () => $('#menu').classList.contains('on');
export const modalIsOpen = () => $('#modal').classList.contains('on');
