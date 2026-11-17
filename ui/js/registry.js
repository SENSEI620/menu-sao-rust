// ───────── PONTOS DE EXTENSÃO ─────────
// Para acrescentar algo ao menu no futuro você NÃO precisa mexer no resto do código:
// crie um arquivo em ui/js/plugins/, registre aqui o que quiser e liste o arquivo em
// ui/js/plugins/index.js. Veja docs/COMO-ADICIONAR-RECURSOS.md.

/** Ações da 3ª coluna (Abrir, Cronômetro, Nota...). */
export const actions = [];
/**
 * registerAction({
 *   id: 'copiar',  label: 'Copiar link',  icon: '📋',  order: 25,
 *   when: (block, profile) => true,        // opcional: quando aparece
 *   run: ctx => { ... },                   // ctx: ver app.js (makeCtx)
 * })
 */
export function registerAction(a) {
  actions.push({ order: 100, ...a });
  actions.sort((x, y) => x.order - y.order);
}

/** Esferas extras no fim da 1ª coluna (as de "+" e engrenagem são registradas assim). */
export const docks = [];
/** registerDock({ id: 'stats', label: 'Estatísticas', icon: '📊', order: 30, run: ctx => {...} }) */
export function registerDock(d) {
  docks.push({ order: 100, ...d });
  docks.sort((x, y) => x.order - y.order);
}

/** Avisos de acontecimentos: 'open', 'close', 'launch'. */
const listeners = {};
export const on = (evt, fn) => (listeners[evt] = listeners[evt] || []).push(fn);
export const emit = (evt, data) => (listeners[evt] || []).forEach(fn => { try { fn(data); } catch (e) { console.error(e); } });
