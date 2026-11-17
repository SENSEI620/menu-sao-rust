// Sons (os arquivos que você enviou, em ui/sounds). Decodificados na memória para tocar sem atraso.
import { S } from './state.js';

const FILES = { enter: 'sounds/enter.wav', exit: 'sounds/exit.wav', confirm: 'sounds/confirm.wav', tick: 'sounds/tick.wav' };
let ctx = null;
const buffers = {};

function ac() {
  if (!ctx) {
    try { ctx = new (window.AudioContext || window.webkitAudioContext)(); } catch (e) { /* sem áudio */ }
  }
  if (ctx && ctx.state === 'suspended') ctx.resume().catch(() => {});
  return ctx;
}

export async function loadSounds() {
  const c = ac();
  if (!c) return;
  await Promise.all(Object.entries(FILES).map(async ([k, url]) => {
    try {
      const res = await fetch(url);
      buffers[k] = await c.decodeAudioData(await res.arrayBuffer());
    } catch (e) { /* som opcional */ }
  }));
}

export function play(k, v = 1) {
  const c = ac(), b = buffers[k];
  if (!c || !b) return;
  const src = c.createBufferSource(), g = c.createGain();
  src.buffer = b;
  g.gain.value = v * (S.settings.volume / 100);
  src.connect(g).connect(c.destination);
  src.start();
}

export const wake = ac;
/** Menu fechado: o áudio dorme (não gasta nada até a próxima abertura). */
export function sleep() { if (ctx && ctx.state === 'running') ctx.suspend().catch(() => {}); }
