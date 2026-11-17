# Menu SAO (versão Rust)

O mesmo menu flutuante estilo Sword Art Online, agora com a parte de dentro em **Rust** (Tauri 2) e a tela
redesenhada em **liquid glass**. Abre links no perfil certo do **Firefox** ou **Brave**, com cronômetro de
limite e notas por link.

Seus dados da versão Electron **continuam valendo**: a pasta é a mesma (`%APPDATA%\menu-sao`), com os mesmos
`config.json` e `settings.json`.

## Como usar

- **Ctrl + botão esquerdo + botão direito** segurados por 1 s, ou o atalho **Ctrl + Alt + M**, abrem o menu.
- Setas / roda do mouse / clique navegam. **Enter** ou **→** entra, **←** ou **Backspace** volta, **Esc** fecha.
- Clique direito numa esfera: editar ou excluir o perfil.
- O ícone da bandeja (perto do relógio) tem: abrir menu, ver log, importar perfis do Firefox, iniciar com o Windows e sair.
- Configurações (esfera de ajustes): atalho, gesto do mouse, tempo de espera, volume, início automático.

### Como ler o status

- **Anel em volta da esfera**: um segmento por link do perfil. Cinza discreto = livre, **vermelho aceso** = limite estourado.
- **Ponto vermelho** no canto da miniatura do link = limite estourado; com a **etiqueta de tempo** ao lado (ex.: `1h58`).
- **Ponto verde** = o link tem uma nota.

## Como obter o `Menu SAO.exe`

**Opção A: no seu PC.** Dê dois cliques em `COMPILAR.bat`. Na primeira vez ele instala o Rust e as Ferramentas de
Build do Visual Studio (alguns GB, uma vez só). Depois compila e cria `Menu SAO.exe` aqui na pasta.
Antes, feche o programa antigo (`FECHAR.bat`).

> **Espaço em disco:** compilar em Rust gera mais de 1 GB de arquivos temporários (o programa final tem uns 8 MB).
> Eles ficam em `%LOCALAPPDATA%\menu-sao-build`, fora desta pasta. `LIMPAR-CACHE.bat` apaga tudo e libera o espaço
> (a próxima compilação volta a demorar). O Rust em si também guarda pacotes em `%USERPROFILE%\.cargo` (algumas centenas de MB).

**Opção B: sem instalar nada.** Suba esta pasta para um repositório no GitHub e rode a aba
**Actions → Compilar para Windows → Run workflow**. No fim, baixe o `Menu-SAO-windows`
(é o `.exe`). Esse processo também roda os testes no Windows.

O programa precisa do **WebView2**, que já vem no Windows 10/11 atualizado.

## Ver só o visual (sem compilar nada)

Abra `ui/index.html` no **Chrome ou Edge**. Ele mostra o menu com dados e uma área de trabalho de exemplo,
para você mexer no visual rapidamente. (Em Firefox/Safari o vidro cai para desfoque simples: a refração
do liquid glass é recurso do Chromium, que é o motor do WebView2 usado no programa.)

## Como a pasta é organizada

```
crates/menu_core/   lógica em Rust pura (config, ajustes, abrir navegador, importar Firefox) + 25 testes
src-tauri/          a casca: janela, bandeja, atalho, gesto do mouse, captura de tela
ui/                 a tela (HTML/CSS/JS sem build): css/, js/, sounds/
  js/glass.js       o liquid glass (mapa de refração + dispersão)
  js/registry.js    pontos de extensão para recursos futuros
  js/plugins/       onde cada recurso novo mora
docs/               guias
```

Para acrescentar recursos novos (ações, esferas, avisos) sem mexer no resto, veja
[`docs/COMO-ADICIONAR-RECURSOS.md`](docs/COMO-ADICIONAR-RECURSOS.md).

## O que mudou em relação à versão Electron

- Rust no lugar do Node/Electron: janela, bandeja, atalho, gesto do mouse, captura de tela, leitura dos perfis do Firefox.
- Gravação segura do `config.json` + cópia de segurança (`config.bak.json`) a cada gravação.
- Se o `config.json` corromper, ele é guardado ao lado (`config.corrompido-*.json`) em vez de ser apagado.
- Campos novos que você acrescentar na interface no futuro são preservados no arquivo.
- Visual novo em liquid glass; status por anel segmentado e selos; ícones de linha; destaque que desliza entre os itens.

## Estado dos testes

Testado no Linux: lógica em Rust (testes automáticos), programa completo rodando de ponta a ponta numa tela virtual
(atalho, captura de tela, abrir menu, navegar, salvar, comando de abrir) e a interface no Chromium.
**Ainda não testado no Windows**: o gesto do mouse, a bandeja, o início automático e o som sem clique prévio
(o programa usa uma opção do WebView2 para isso). A Opção B acima roda os testes da lógica no Windows de verdade.
