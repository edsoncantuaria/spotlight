# Spotlight

Launcher estilo Raycast/Spotlight para **Linux** — busca universal, quick answers, clipboard, extensões e bandeja do sistema.

[![CI](https://github.com/edsoncantuaria/spotlight/actions/workflows/ci.yml/badge.svg)](https://github.com/edsoncantuaria/spotlight/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/edsoncantuaria/spotlight)](https://github.com/edsoncantuaria/spotlight/releases)

## Instalação

### Release (recomendado)

Baixe o `.deb` ou `.AppImage` na [página de Releases](https://github.com/edsoncantuaria/spotlight/releases).

```bash
# .deb (Ubuntu/Debian)
sudo dpkg -i spotlight_*_amd64.deb
sudo apt-get install -f

# AppImage
chmod +x spotlight_*_amd64.AppImage
./spotlight_*_amd64.AppImage
```

### Dependências recomendadas

```bash
sudo apt update
sudo apt install -y \
  fd-find wmctrl wtype xdotool git \
  libwebkit2gtk-4.1-0 libayatana-appindicator3-1
```

| Pacote | Uso |
|--------|-----|
| `fd-find` | Busca rápida de arquivos |
| `wmctrl` | Foco de janelas (GNOME/KDE) |
| `wtype` / `xdotool` | Colar snippets automaticamente |
| `git` | Instalar extensões da loja |

### Build from source

```bash
git clone https://github.com/edsoncantuaria/spotlight.git
cd spotlight
npm install
npm run tauri build
# binários em src-tauri/target/release/bundle/
```

## Funcionalidades v1.0

- **Busca:** apps, arquivos, web, favoritos, histórico, contatos, settings
- **Quick answers:** calc, conversão, moeda, hora, dicionário, fuso
- **Clipboard Pro:** pin, filtros, paste stack (`Ctrl+Alt+C`)
- **Bandeja:** menu com config, extensões, loja e guia
- **Comandos `>`:** lock, suspend, logout, screenshot…
- **Extensões:** builtin + scripts user + [loja GitHub](docs/extension-store/catalog.json)
- **Produtividade:** quicklinks, snippets, scripts (TOML/JSON)

## Atalhos

| Tecla | Ação |
|-------|------|
| `Ctrl+Alt+Space` | Abrir/fechar Spotlight |
| `Ctrl+Alt+C` | Área de transferência |
| `Ctrl+P` | Fixar item (clipboard) |
| `Shift+Enter` | Adicionar ao paste stack |
| `Ctrl+Shift+V` | Colar stack |
| `>` + comando | Modo comando |
| `Esc` | Fechar |

## Configuração

`~/.config/spotlight/config.toml` — ou abra **Configurações** pela bandeja.

Autostart: `~/.config/autostart/spotlight.desktop` (criado na 1ª execução).

## Extensões

Guia completo: [docs/extension-store/GUIA-EXTENSOES.md](docs/extension-store/GUIA-EXTENSOES.md)

Loja: bandeja → **Loja de extensões**, ou PR em `docs/extension-store/catalog.json`.

## Desenvolvimento

```bash
npm install
npm run tauri dev
```

## CI/CD

- **CI:** push/PR → testes Rust + build frontend
- **Release:** tag `v*` → build `.deb` + `.AppImage` + GitHub Release

```bash
git tag v1.0.0
git push origin v1.0.0
```

## Licença

MIT — see [LICENSE](LICENSE)
