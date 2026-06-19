# Changelog

## [1.0.6] - 2026-06-17

### Adicionado
- Autostart real ao instalar `.deb` (`/etc/xdg/autostart`) — não precisa rodar `spotlight &`
- Configurações: capturar/testar atalhos antes de salvar; detecta conflitos no GNOME
- Extensões: habilitar/desabilitar integradas e de usuário; filtros (ativas, desativadas, etc.)

### Corrigido
- Desmarcar "Abrir com o sistema" grava override `Hidden=true` mesmo com autostart do sistema
- Índice de apps carrega em background (startup mais rápido no login)
- Busca de configurações do sistema só com 3+ caracteres

## [1.0.5] - 2026-06-17

### Adicionado
- Configurações repaginadas: sidebar, seções Geral / Atalhos / Busca / Avançado
- Página de extensões com cards, filtro e toggle para extensões de usuário

### Corrigido
- Busca mais fluida: extensões pesadas (docker, git) só rodam quando relevantes
- Listagem de janelas só quando a query menciona janelas
- Busca de arquivos usa índice primeiro; `fdfind`/`fd` como fallback
- Dependências (`xclip`, `wl-clipboard`, `wmctrl`, `wtype`, `xdotool`, `fd-find`) obrigatórias no `.deb`

## [1.0.4] - 2026-06-17

### Corrigido
- Janela invisível que bloqueava cliques no desktop: a UI monta antes de `show()` nativo
- Fechar esconde a janela nativa imediatamente (sem overlay transparente preso)
- Atalhos no GNOME: fallback via `gsettings` + segunda instância (`--toggle`, `--toggle-clipboard`)
- Janelas main e clipboard ocultas no startup

### Melhorado
- Detecção de ambiente (X11/Wayland, GNOME/KDE) com aviso se faltam `xclip`/`wl-clipboard`
- `.deb` recomenda: xclip, wl-clipboard, wmctrl, wtype, xdotool

## [1.0.3] - 2026-06-18

### Corrigido
- Spotlight voltava a fechar sozinho ao abrir (atalho e bandeja)
- Janelas de configurações, loja, extensões e guia registradas no app (funcionam no `.deb`)
- Loja usa catálogo embutido quando offline; exemplo builtin instala corretamente

### Adicionado
- Configuração **Abrir com o sistema** (checkbox nas Settings)

## [1.0.2] - 2026-06-18

### Corrigido
- Highlight de busca: só destaca substring exata da query (sem blocos azuis quebrados)
- Clipboard: janela não fecha sozinha ao abrir (`Ctrl+Alt+C`)
- Clipboard: Enter cola no app focado (não só copia para a área de transferência)
- Watcher do clipboard prioriza texto antes de imagem

## [1.0.1] - 2026-06-18

### Corrigido
- Autostart (`~/.config/autostart/spotlight.desktop`) passa a sincronizar o `Exec` com o binário instalado (corrige falha após migrar de build dev para `.deb`)
- Atalho padrão reduzido a `Ctrl+Alt+Space` — `Super+Space` removido por conflitar com troca de idioma no GNOME
- Aviso no log quando `Super+Space` estiver configurado manualmente

## [1.0.0] - 2026-06-17

### Adicionado
- Launcher universal: apps, arquivos, web, contatos, settings GNOME/KDE
- Quick answers: calculadora, conversões, moedas, hora/data, dicionário, fuso
- Clipboard Pro: pin, filtros (texto/imagem/fixados), paste stack
- Bandeja do sistema com menu (Spotlight, clipboard, config, extensões, loja, guia)
- Loja de extensões via GitHub (`catalog.json` + install)
- Extensões builtin: emoji, notas, git, docker, systemd, calculadora, tradução
- Extensões user via scripts shell + `manifest.json`
- Modo comando `>` (lock, suspend, logout, screenshot, etc.)
- Quicklinks, snippets, scripts configuráveis
- Settings UI completa
- CI (testes + build) e release automática (.deb / AppImage)

### Notas
- Extensão AI desabilitada por padrão (sem API key necessária)
- Testado em Ubuntu/Debian; Wayland e X11 com dependências opcionais

[1.0.3]: https://github.com/edsoncantuaria/spotlight/releases/tag/v1.0.3
[1.0.2]: https://github.com/edsoncantuaria/spotlight/releases/tag/v1.0.2
[1.0.1]: https://github.com/edsoncantuaria/spotlight/releases/tag/v1.0.1
[1.0.0]: https://github.com/edsoncantuaria/spotlight/releases/tag/v1.0.0
