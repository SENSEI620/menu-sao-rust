# Como adicionar recursos ao menu

O menu foi feito para crescer sem mexer no miolo. Tudo que é "coisa nova" mora em `ui/js/plugins/`.

## 1. Uma ação nova (3ª coluna, ao lado de Abrir / Cronômetro / Nota...)

Crie `ui/js/plugins/copiar-link.js`:

```js
import { registerAction } from '../registry.js';
import { ico } from '../icons.js';

registerAction({
  id: 'copiar',                 // único
  label: 'Copiar link',
  icon: ico('go'),              // ícone de linha (ui/js/icons.js) ou um emoji
  order: 25,                    // posição na lista (menor = mais acima)
  tone: 'ok',                   // opcional: 'ok' (verde) ou 'danger' (vermelho)
  when: (b, p) => true,         // opcional: quando a ação aparece
  run: ({ b, toast }) => {      // o que fazer
    navigator.clipboard.writeText(b.url);
    toast('Link copiado!');
  },
});
```

Depois liste o arquivo em `ui/js/plugins/index.js`:

```js
import './core.js';
import './copiar-link.js';
```

O `run` recebe: `p` (perfil), `b` (link), `S` (estado), `api` (conversa com o Rust), `ask`, `toast`, `play` (sons),
`save`, `render`, `closeMenu`, `editBlock`, `backToLinks`.

## 2. Uma esfera nova (fim da 1ª coluna, junto de "+" e ajustes)

```js
import { registerDock } from '../registry.js';
registerDock({ id: 'stats', label: 'Estatísticas', icon: '📊', order: 30, run: ctx => { /* ... */ } });
```

## 3. Reagir a acontecimentos

```js
import { on } from '../registry.js';
on('open',  () => { /* menu abriu */ });
on('close', () => { /* menu fechou */ });
```

## 4. Guardar dados novos nos perfis ou links

Pode acrescentar campos livremente (`b.favorito = true`, `p.cor = '#f00'`). O Rust preserva os campos que ele
não conhece ao gravar o `config.json`.

## 5. Algo que só o Rust consegue (arquivos, rede, sistema)

1. Escreva a função em `crates/menu_core/` (ela é testável sozinha, com `cargo test -p menu_core`).
2. Exponha como comando em `src-tauri/src/commands.rs` (`#[tauri::command]`) e registre em `main.rs`
   (`generate_handler!`).
3. Chame da tela: acrescente uma linha em `ui/js/bridge.js` (e uma versão de mentira em `mock`, para a prévia
   no navegador continuar funcionando).

## Dicas de visual

- Tudo que é vidro usa a classe `glass`. Elementos novos com tamanho próprio que devam refratar o fundo:
  registre em `TARGETS` em `ui/js/glass.js`.
- As cores e o ritmo das animações estão em `ui/css/` (variáveis em `base.css`).
