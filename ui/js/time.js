// Formatação de tempo e cálculo do "quando o limite volta".

/** 5:30, 2h15, 12d4h */
export const short = ms => {
  const s = Math.ceil(ms / 1000);
  if (s >= 86400) return Math.floor(s / 86400) + 'd' + Math.floor((s % 86400) / 3600) + 'h';
  if (s >= 3600) return Math.floor(s / 3600) + 'h' + String(Math.floor((s % 3600) / 60)).padStart(2, '0');
  return Math.floor(s / 60) + ':' + String(s % 60).padStart(2, '0');
};

export const fmtD = ms =>
  new Date(ms).toLocaleString('pt-BR', { day: '2-digit', month: '2-digit', hour: '2-digit', minute: '2-digit' }).replace(',', '');

/**
 * Converte o que foi preenchido na janela do cronômetro no horário (ms) em que o limite volta.
 * v.h = hora do relógio, v.m = minutos, v.day = próximo dia do mês, v.d = data e hora exatas.
 * Vale a opção que foi mexida por último (v._last). Devolve 0 se não deu para calcular.
 */
export function untilFrom(v, nowMs = Date.now()) {
  const filled = k => v[k] !== undefined && String(v[k]).trim() !== '';
  let mode = v._last && filled(v._last) ? v._last : ['h', 'm', 'day', 'd'].find(filled);
  if (mode === 'h' && filled('day')) mode = 'day';
  const n = new Date(nowMs);
  const hm = filled('h') ? String(v.h).split(':').map(Number) : null;

  if (mode === 'd') {
    const x = new Date(v.d).getTime();
    return x > nowMs ? x : 0;
  }
  if (mode === 'day') {
    const day = parseInt(v.day, 10);
    if (!(day >= 1 && day <= 31)) return 0;
    const [hh, mm] = hm || [4, 0];
    for (let k = 0; k < 14; k++) {
      const c = new Date(n.getFullYear(), n.getMonth() + k, day, hh || 0, mm || 0);
      if (c.getDate() === day && c.getTime() > nowMs) return c.getTime();
    }
    return 0;
  }
  if (mode === 'h') {
    const c = new Date(n.getFullYear(), n.getMonth(), n.getDate(), hm[0] || 0, hm[1] || 0);
    if (c.getTime() <= nowMs) c.setDate(c.getDate() + 1);
    return c.getTime();
  }
  if (mode === 'm') {
    const m = parseFloat(String(v.m).replace(',', '.'));
    return m > 0 ? Math.round(nowMs + m * 60000) : 0;
  }
  return 0;
}
