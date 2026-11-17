// Ponte entre a tela e o programa em Rust.
// Dentro do programa usa os comandos do Tauri; num navegador comum usa dados de exemplo,
// então dá para abrir ui/index.html direto no navegador para mexer no visual.

const T = window.__TAURI__;
export const isTauri = !!(T && T.core);

const mkId = () => Math.random().toString(36).slice(2, 9);

/** Área de trabalho de mentira para a prévia no navegador (janelas e texto, para ter o que refratar). */
function wallpaper() {
  const c = document.createElement('canvas');
  c.width = 960; c.height = 540;
  const g = c.getContext('2d');
  const grad = g.createLinearGradient(0, 0, 960, 540);
  grad.addColorStop(0, '#1f5f8f'); grad.addColorStop(.55, '#6a3c93'); grad.addColorStop(1, '#e0793a');
  g.fillStyle = grad; g.fillRect(0, 0, 960, 540);
  g.fillStyle = 'rgba(255,255,255,.9)'; g.fillRect(120, 70, 330, 220);
  g.fillStyle = '#2b3445'; g.fillRect(120, 70, 330, 26);
  g.fillStyle = 'rgba(20,24,34,.88)'; g.fillRect(520, 190, 330, 250);
  for (let i = 0; i < 9; i++) { g.fillStyle = i % 2 ? 'rgba(80,90,110,.8)' : 'rgba(150,160,180,.9)'; g.fillRect(140, 112 + i * 19, 130 + (i * 37) % 160, 8); }
  for (let i = 0; i < 8; i++) { g.fillStyle = ['#5eead4', '#f9a8d4', '#fde68a', '#93c5fd'][i % 4]; g.fillRect(545, 215 + i * 26, 60 + (i * 53) % 220, 10); }
  g.fillStyle = '#ffd166'; g.beginPath(); g.arc(780, 110, 46, 0, 7); g.fill();
  return c.toDataURL('image/jpeg', .85);
}

function sample() {
  const m = 60000, now = Date.now();
  return {
    config: [
      { id: mkId(), name: 'F1-JOURNAL-NEWS', icon: '💎', br: 'firefox', pf: 'F1-JOURNAL-NEWS', blocks: [
        { id: mkId(), t: 'Claude', url: 'https://claude.ai', until: 0, note: '' },
        { id: mkId(), t: 'ChatGPT', url: 'https://chatgpt.com', until: now + 95 * m, note: '' },
        { id: mkId(), t: 'Canva', url: 'https://canva.com', until: 0, note: 'Terminar o carrossel do GP' } ] },
      { id: mkId(), name: 'Brave — principal', icon: '🦁', br: 'brave', pf: 'Default', blocks: [
        { id: mkId(), t: 'Gemini', url: 'https://gemini.google.com', until: now + 30 * 60 * m, note: '' } ] },
      { id: mkId(), name: 'Estudos', icon: '📖', br: 'firefox', pf: 'Estudos', blocks: [] },
    ],
    bg: wallpaper(),
    settings: { shortcut: 'Control+Alt+M', mouseGesture: true, holdMs: 1000, volume: 100, firefoxNoRemote: false, autostart: false, packaged: true },
  };
}

let mockShow = null;
const mock = {
  saveConfig: () => {},
  saveSettings: () => {},
  importFirefox: async () => ({ n: 0, config: sample().config }),
  hide: () => { const h = document.getElementById('home'); if (h) h.hidden = false; },
  launch: (p, url) => console.log('[prévia] abriria', p.br, p.pf, url),
  onShow: cb => { mockShow = cb; setTimeout(() => cb(sample()), 60); },
  ready: () => {},
  onToast: () => {},
  reopen: () => mockShow && mockShow(sample()),
};

const tauri = !isTauri ? null : {
  saveConfig: c => T.core.invoke('save_config', { config: c }),
  saveSettings: s => T.core.invoke('save_settings', { settings: s }),
  importFirefox: () => T.core.invoke('import_firefox'),
  hide: () => T.core.invoke('hide_menu'),
  launch: (p, url) => T.core.invoke('launch', { br: p.br, pf: p.pf, url }),
  onShow: cb => T.event.listen('menu:show', e => cb(e.payload)),
  /** Avisa o Rust que a tela carregou; se ele estava esperando para abrir o menu, devolve os dados. */
  ready: async cb => { const d = await T.core.invoke('ui_ready'); if (d) cb(d); },
  onToast: cb => T.event.listen('toast', e => cb(e.payload)),
};

export const api = tauri || mock;
