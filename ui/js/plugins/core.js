// Recursos que já vêm no menu, registrados do mesmo jeito que os futuros: ações da 3ª coluna
// e as esferas "+" e engrenagem. Use este arquivo como modelo para os seus.
import { registerAction, registerDock } from '../registry.js';
import { isOut, S, save } from '../state.js';
import { api } from '../bridge.js';
import { play } from '../audio.js';
import { ask, editBlock, editProfile, openSettings, toast } from '../dialogs.js';
import { fmtD, short, untilFrom } from '../time.js';
import { ico } from '../icons.js';

// ── esferas do fim da coluna 1 ──
registerDock({ id: 'add', label: 'Adicionar perfil', icon: ico('plus'), order: 10, run: () => editProfile(null) });
registerDock({ id: 'settings', label: 'Configurações', icon: ico('sliders'), order: 20, run: () => openSettings() });

// ── ações da coluna 3 ──
registerAction({
  id: 'go', label: 'Abrir', icon: ico('go'), order: 10,
  run: ({ p, b, closeMenu }) => { play('confirm'); api.launch(p, b.url); closeMenu(); },
});

registerAction({
  id: 'free', label: 'Liberar', icon: ico('free'), tone: 'ok', order: 20, when: b => isOut(b),
  run: ({ b, backToLinks }) => { b.until = 0; play('confirm'); save(); backToLinks(); },
});

registerAction({
  id: 'timer', label: 'Cronômetro', icon: ico('timer'), order: 20, when: b => !isOut(b),
  run: ({ b, backToLinks }) => ask('Cronômetro', 'Quando o limite volta?', [
    { k: 'h', l: 'Defina um horário', type: 'time', v: '', hint: 'Volta na próxima vez que o relógio marcar essa hora (hoje, ou amanhã se já passou).' },
    { sep: 'ou escolha outra forma' },
    { k: 'm', l: 'Tempo restante', type: 'number', v: '', hint: 'Em minutos. Exemplo: 90 para 1h30.' },
    { k: 'day', l: 'Próximo dia do mês', type: 'number', v: '', hint: 'Exemplo: 20. Usa o horário definido acima, ou 04:00 se estiver vazio.' },
    { k: 'd', l: 'Data e hora específicas', type: 'datetime-local', v: '' },
  ], v => {
    const u = untilFrom(v);
    if (!u) { toast('Defina um horário ou escolha uma das outras opções.'); return; }
    b.until = u; play('confirm'); save(); backToLinks();
    toast('Volta em ' + short(u - Date.now()) + ' (' + fmtD(u) + ')');
  }),
});

registerAction({
  id: 'note', label: 'Nota', icon: ico('note'), order: 30,
  run: ({ b, backToLinks }) => ask('Nota', '', [
    { k: 'note', l: 'O que ficou inacabado aqui? (vazio apaga a nota)', v: b.note, area: 1 },
  ], v => { b.note = v.note.trim(); play('confirm'); save(); backToLinks(); }),
});

registerAction({ id: 'edit', label: 'Editar', icon: ico('edit'), order: 40, run: ({ p, b }) => editBlock(p, b) });

registerAction({
  id: 'del', label: 'Excluir', icon: ico('trash'), tone: 'danger', order: 50,
  run: ({ p, b, render }) => ask('Excluir link', `Excluir “${b.t}”?`, [], () => {
    p.blocks = p.blocks.filter(x => x !== b);
    save(); S.lv = 1; S.bi = 0; S.ai = -1; render();
  }, 'Excluir'),
});
