// Janelas de edição, ajustes e avisos.
import { S, $, esc, clamp, save, mkBlock, mkProfile } from './state.js';
import { api } from './bridge.js';
import { play } from './audio.js';
import { render } from './view.js';

export function toast(t) {
  const n = $('#toast');
  n.textContent = t;
  n.classList.add('on');
  clearTimeout(n._t);
  n._t = setTimeout(() => n.classList.remove('on'), 3500);
}

const closeModal = () => $('#modal').classList.remove('on');

/**
 * Janela de perguntas. fields: [{k, l, v, type, hint, area, sel, sep}]
 * cb recebe { campo: valor, _last: último campo mexido }.
 */
export function ask(title, msg, fields, cb, ok = 'Salvar') {
  $('#mb').innerHTML = `<b>${esc(title)}</b>${msg ? `<span>${esc(msg)}</span>` : ''}` + fields.map(f => f.sep
    ? `<div class="sep"><span>${esc(f.sep)}</span></div>`
    : `<label>${esc(f.l)}${
      f.sel ? `<select data-k="${f.k}">${f.sel.map(o => `<option value="${o[0]}" ${o[0] === f.v ? 'selected' : ''}>${o[1]}</option>`).join('')}</select>`
        : f.area ? `<textarea data-k="${f.k}">${esc(f.v || '')}</textarea>`
          : `<input type="${f.type || 'text'}" data-k="${f.k}" value="${esc(f.v || '')}">`}${
      f.hint ? `<small class="hint">${esc(f.hint)}</small>` : ''}</label>`).join('') +
    `<div class="mrow"><button id="mc">Cancelar</button><button id="mo">${ok}</button></div>`;
  $('#modal').classList.add('on');
  const first = $('#mb input,#mb textarea');
  (first || $('#mo')).focus();
  let last = null;
  $('#mb').oninput = e => { if (e.target.dataset.k) last = e.target.dataset.k; };
  $('#mc').onclick = closeModal;
  $('#mo').onclick = () => {
    const v = {};
    document.querySelectorAll('#mb [data-k]').forEach(x => (v[x.dataset.k] = x.value));
    v._last = last;
    closeModal();
    cb(v);
  };
}

export function initDialogs() {
  $('#modal').addEventListener('click', e => { if (e.target.id === 'modal') closeModal(); });
  $('#mb').addEventListener('keydown', e => {
    if (e.key === 'Enter' && e.target.tagName === 'INPUT') { e.preventDefault(); $('#mo').click(); }
  });
}
export { closeModal };

export function editProfile(p) {
  ask(p ? 'Editar perfil' : 'Novo perfil', '', [
    { k: 'name', l: 'Nome', v: p ? p.name : '' },
    { k: 'icon', l: 'Ícone (um emoji ou símbolo)', v: p ? p.icon : '⭐' },
    { k: 'br', l: 'Navegador', v: p ? p.br : 'firefox', sel: [['firefox', 'Firefox'], ['brave', 'Brave']] },
    { k: 'pf', l: 'Perfil: nome ou caminho da pasta (Brave: Default)', v: p ? p.pf : '' },
  ], v => {
    if (!v.name.trim()) return;
    if (p) Object.assign(p, { name: v.name.trim(), icon: v.icon || '⭐', br: v.br, pf: v.pf.trim() });
    else {
      S.P.push(mkProfile(v.name.trim(), v.icon || '⭐', v.br, v.pf.trim() || v.name.trim()));
      S.pi = S.P.length - 1; S.bi = 0; S.lv = 1;
    }
    play('confirm'); save(); render();
  });
}

export function editBlock(p, b) {
  ask(b ? 'Editar link' : 'Novo link', '', [
    { k: 't', l: 'Título', v: b ? b.t : '' },
    { k: 'url', l: 'Link', v: b ? b.url : 'https://' },
  ], v => {
    if (!v.t.trim()) return;
    if (b) { b.t = v.t.trim(); b.url = v.url.trim(); }
    else { p.blocks.push(mkBlock(v.t.trim(), v.url.trim())); S.bi = p.blocks.length - 1; }
    S.lv = 1; S.ai = -1; play('confirm'); save(); render();
  });
}

/** Atalho (ex.: Control+Alt+M) a partir de uma tecla apertada; exige uma tecla de modificação. */
function accel(e) {
  const k = e.code;
  let key = null;
  if (/^Key[A-Z]$/.test(k)) key = k.slice(3);
  else if (/^Digit\d$/.test(k)) key = k.slice(5);
  else if (/^F\d{1,2}$/.test(k)) key = k;
  else if (k === 'Space') key = 'Space';
  if (!key) return null;
  const m = [];
  if (e.ctrlKey) m.push('Control');
  if (e.altKey) m.push('Alt');
  if (e.shiftKey) m.push('Shift');
  if (e.metaKey) m.push('Super');
  return m.length ? [...m, key].join('+') : null;
}

export function openSettings() {
  const s = S.settings;
  let val = s.shortcut || '';
  $('#mb').innerHTML = `<b>Configurações</b>
  <label>Atalho do teclado (clique no campo e aperte as teclas)<input id="sKey" readonly value="${esc(val)}" placeholder="Nenhum"></label>
  <div class="mrow"><button id="sClr">Limpar atalho</button></div>
  <label class="chk"><input type="checkbox" id="sMouse" ${s.mouseGesture ? 'checked' : ''}> Abrir segurando Ctrl + botão esquerdo + botão direito</label>
  <label>Segundos segurando<input id="sHold" type="number" min="0.3" max="5" step="0.1" value="${s.holdMs / 1000}"></label>
  <label>Volume dos sons<input id="sVol" type="range" min="0" max="100" value="${s.volume}"></label>
  <label class="chk"><input type="checkbox" id="sAuto" ${s.autostart ? 'checked' : ''}> Iniciar com o Windows</label>
  <label class="chk"><input type="checkbox" id="sNR" ${s.firefoxNoRemote ? 'checked' : ''}> Firefox: abrir em janela separada (-no-remote)</label>
  <div class="mrow"><button id="sImp">Importar perfis do Firefox</button></div>
  <div class="mrow"><button id="mc">Cancelar</button><button id="mo">Salvar</button></div>`;
  $('#modal').classList.add('on');
  const key = $('#sKey');
  key.onkeydown = e => { e.preventDefault(); e.stopPropagation(); const a = accel(e); if (a) { val = a; key.value = a; } };
  $('#sClr').onclick = () => { val = ''; key.value = ''; };
  $('#sImp').onclick = async () => {
    try {
      const r = await api.importFirefox();
      S.P = r.config; render();
      toast(r.n + ' perfil(is) importado(s) ou corrigido(s).');
    } catch (e) { toast('Não consegui importar.'); }
  };
  $('#mc').onclick = closeModal;
  $('#mo').onclick = () => {
    const h = parseFloat(String($('#sHold').value).replace(',', '.'));
    S.settings = {
      ...S.settings, shortcut: val, mouseGesture: $('#sMouse').checked,
      holdMs: Math.round(clamp(isNaN(h) ? 1 : h, .3, 5) * 1000), volume: +$('#sVol').value,
      firefoxNoRemote: $('#sNR').checked, autostart: $('#sAuto').checked,
    };
    api.saveSettings(S.settings);
    closeModal(); play('confirm');
  };
}
