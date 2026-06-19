# Changelog

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

[1.0.1]: https://github.com/edsoncantuaria/spotlight/releases/tag/v1.0.1
[1.0.0]: https://github.com/edsoncantuaria/spotlight/releases/tag/v1.0.0
