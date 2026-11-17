# O que este teste ensina para o F1 Journal Studio

Notas curtas, escritas para quando for a hora de converter o Studio.

1. **Tauri, não "Rust puro".** O Studio tem editor visual, templates, vídeo. A interface continua em
   HTML/CSS/JS (o que já existe se reaproveita) e o Rust assume o trabalho pesado: arquivos, imagens,
   chamadas ao FFmpeg, rede. É o mesmo desenho deste projeto.
2. **Separe a lógica numa biblioteca (`menu_core`).** Ela não conhece janela nenhuma, então roda em testes
   automáticos em qualquer máquina. Para funcionários, isso significa que mexer numa tela não quebra a regra de negócio.
3. **Mantenha o formato dos dados.** Aqui os arquivos do Electron continuam valendo. No Studio isso evita perder
   templates e posts já feitos. Campos desconhecidos são preservados.
4. **Gravação segura + backup automático** (já feita em `store.rs`). Com várias pessoas usando, é a diferença
   entre um arquivo corrompido e um susto.
5. **Pontos de extensão** (`registry.js` e `plugins/`): cada recurso novo é um arquivo. O guia de apps do Studio
   (pastas soltas em `apps/`) combina com isso.
6. **Compilar na nuvem** (`.github/workflows/build-windows.yml`): quem for contratado não precisa instalar Rust.
   O GitHub entrega o `.exe`.

## O que ainda precisa ser resolvido para equipe (não existe neste teste)

- **Instalador e atualização automática** (plugin `tauri-plugin-updater` + `cargo tauri build` para gerar `.msi`/`.exe`).
- **Assinatura do programa** (certificado de código). Sem ela o Windows mostra aviso de "editor desconhecido".
- **Permissões por usuário**, se o Studio tiver níveis de acesso: decidir onde isso mora (servidor) antes de construir.
- **Registro de erros** que chegue até você (hoje o log é só um arquivo local).
- **Vídeo**: continua sendo FFmpeg chamado pelo Rust. A vantagem do Rust aqui é processar e exportar sem travar a tela.
