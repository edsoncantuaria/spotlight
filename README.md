# Spotlight

Launcher estilo macOS Spotlight para Linux — busca universal, quick answers, preview e ranking por uso.

## Funcionalidades

### Busca universal
- **Aplicativos** — arquivos `.desktop` com fuzzy match e highlight
- **Documentos** — arquivos em Documents, Downloads, Desktop (`fd` se instalado, senão walkdir)
- **Configurações** — painéis GNOME/Ubuntu (Wi-Fi, Bluetooth, Tela, etc.)
- **Sugestões** — itens usados recentemente (query vazia)

### Quick Answers
- Calculadora (`2+2`, `(10*3)/2`)
- Conversões (`100 km in miles`, `32c in f`)
- Moedas (`100 usd to brl`) — API Frankfurter
- Definições (`define hello`, `o que é serendipity`)
- Fuso horário (`time in tokyo`, `hora em são paulo`)

### UX estilo macOS
- Overlay centralizado com blur e animação
- Seções com cabeçalhos (Aplicativos, Documentos, Configurações)
- Painel de preview à direita ao selecionar item
- Ações: Abrir, Mostrar na pasta, Copiar caminho
- Ranking por frequência e recência (SQLite)
- Fecha ao clicar fora

### Atalhos
| Tecla | Ação |
|---|---|
| `Ctrl+Alt+Space` | Abrir/fechar (padrão) |
| `Super+Space` | Alternativo (pode conflitar no GNOME) |
| `↑/↓` | Navegar resultados |
| `Tab` | Alternar seção |
| `Enter` | Abrir / copiar quick answer |
| `Esc` | Fechar |

## Dependências de sistema

### Ubuntu / Debian

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl wget file \
  libxdo-dev libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev \
  fd-find xdg-utils
```

`fd-find` acelera a busca de arquivos (opcional mas recomendado).

## Desenvolvimento

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

## Configuração

Arquivo `~/.config/spotlight/config.toml`:

```toml
shortcuts = ["Ctrl+Alt+Space", "Super+Space"]
```

Histórico e ranking: `~/.config/spotlight/history.db`

Autostart: criado automaticamente em `~/.config/autostart/spotlight.desktop` na primeira execução.

## Paridade com Spotlight da Apple

| Recurso | Status |
|---|---|
| Busca de apps | Sim |
| Busca de arquivos | Sim |
| Configurações do sistema | Sim (GNOME) |
| Quick answers | Sim |
| Preview panel | Sim |
| Ranking por uso | Sim |
| Siri / Mail / Fotos nativos | Não (Apple-only) |

## Licença

MIT
