# Spotlight → Launcher Completo — Roadmap

> **Estimativa total:** ~8–10 semanas (1 dev full-time) · **Hoje:** v0.1.0 MVP avançado (~45% Raycast)

Legenda de esforço: **S** = 0.5–1 dia · **M** = 2–3 dias · **L** = 1 semana · **XL** = 2+ semanas

---

## Fase A — Bugs críticos · **~3 dias** ✅ em progresso

| # | Item | Esforço | Status |
|---|------|---------|--------|
| A1 | Foco de janela (`run_action` aceita ID wmctrl bruto) | S | ✅ |
| A2 | `copy_path` no backend (`run_preview_action`) | S | ✅ |
| A3 | `clipboard_limit` do config (remover `MAX_ITEMS=10`) | S | ✅ |
| A4 | Atalho clipboard → janela `ClipboardApp` | S | ✅ |
| A5 | Respeitar `enabled` no manifest de extensões | S | ✅ |
| A6 | Re-registrar atalhos após `save_config` | S | ✅ |

---

## Fase B — Produtividade diária · **~1 semana**

| # | Item | Esforço | Status |
|---|------|---------|--------|
| B1 | Snippet `paste` real (wtype/xdotool) | M | ✅ |
| B2 | Janelas na busca sem keyword `window` | S | ✅ |
| B3 | Apps populares quando query vazia | S | ✅ |
| B4 | Quick answer **hora/data** (`hora`, `time`, `data`) | S | ✅ |
| B5 | ClipboardApp usa `clipboard_limit` do config | S | ✅ |
| B6 | Pin de itens no clipboard | M | ✅ |
| B7 | Filtros clipboard por tipo (texto/imagem) | M | ✅ |
| B8 | Paste stack (colar histórico) | L | ✅ |

### Bandeja do sistema + Loja · **~2 dias** ✅

| # | Item | Status |
|---|------|--------|
| T1 | Ícone na bandeja (system tray) | ✅ |
| T2 | Menu: Spotlight, Clipboard, Settings, Extensões, Loja, Guia, Sair | ✅ |
| T3 | Loja via GitHub (`catalog.json` + `git clone`) | ✅ |
| T4 | Guia "Como criar extensões" | ✅ |
| T5 | Janelas Store / Extensions / Guide | ✅ |

---

## Fase C — Modo comando & extensões · **~2 semanas**

| # | Item | Esforço | Status |
|---|------|---------|--------|
| C1 | Modo `>` unificado (comandos de sistema) | M | ✅ |
| C2 | Comandos: lock, suspend, logout, screenshot, settings | M | ✅ |
| C3 | Extensões user via script (`search_command`/`run_command`) | L | ✅ |
| C4 | `extension_dirs` no config | S | ✅ |
| C5 | SDK extensões WASM/JS | XL | ⬜ |
| C6 | Extension Store / marketplace | XL | ⬜ |
| C7 | UI gerenciador de extensões | M | ⬜ |
| C8 | Hotkeys por comando | L | ⬜ |

---

## Fase D — Settings & config · **~1 semana**

| # | Item | Esforço | Status |
|---|------|---------|--------|
| D1 | Settings: todos os campos de `config.toml` | M | ✅ |
| D2 | Editor visual quicklinks/snippets | L | ⬜ |
| D3 | Backup tar completo (db + imagens + extensões) | M | ⬜ |
| D4 | Onboarding first-run | M | ⬜ |
| D5 | i18n PT-BR completo | M | ⬜ |

---

## Fase E — AI · **~1 semana**

| # | Item | Esforço | Status |
|---|------|---------|--------|
| E1 | Painel AI chat (streaming) | L | ⬜ |
| E2 | Histórico de conversas | M | ⬜ |
| E3 | Contexto clipboard/seleção | M | ⬜ |
| E4 | Model picker + API keys na UI | M | ⬜ |

---

## Fase F — Busca profunda · **~2 semanas**

| # | Item | Esforço | Status |
|---|------|---------|--------|
| F1 | Busca por conteúdo de arquivo (ripgrep) | L | ⬜ |
| F2 | Abas abertas do navegador | L | ⬜ |
| F3 | Filtros `@apps` `#files` `?web` | M | ⬜ |
| F4 | Rescan apps ao instalar (.desktop watcher) | M | ⬜ |
| F5 | E-mail / calendário nativo | XL | ⬜ |

---

## Fase G — Window management · **~1 semana**

| # | Item | Esforço | Status |
|---|------|---------|--------|
| G1 | Layouts (metade, terços) | M | ⬜ |
| G2 | Workspaces / monitores | L | ⬜ |
| G3 | Force quit processo | M | ⬜ |
| G4 | APIs nativas GNOME/KDE (sem wmctrl) | L | ⬜ |

---

## Fase H — Polish & UX · **~1 semana**

| # | Item | Esforço | Status |
|---|------|---------|--------|
| H1 | Tema claro completo | M | ⬜ |
| H2 | Animações Linux sem ghosting | M | ✅ parcial |
| H3 | Empty states por seção | S | ⬜ |
| H4 | Badges de tipo nos resultados | S | ⬜ |
| H5 | Quick answers: cripto, %, datas naturais | M | ⬜ |
| H6 | Dicionário PT-BR | M | ⬜ |

---

## Fase I — Integrações SaaS · **~3+ semanas**

| # | Item | Esforço | Status |
|---|------|---------|--------|
| I1 | OAuth framework | XL | ⬜ |
| I2 | GitHub / GitLab PRs & issues | L | ⬜ |
| I3 | Slack / Discord | L | ⬜ |
| I4 | Linear / Jira | L | ⬜ |
| I5 | Workflows (encadeamento) | XL | ⬜ |
| I6 | Cloud sync config | L | ⬜ |

---

## Cronograma sugerido

```
Semana 1–2   │ A + B (bugs, clipboard, snippets, hora)
Semana 3–4   │ C + D (comando >, ext user, settings)
Semana 5     │ E (AI chat)
Semana 6–7   │ F (busca profunda)
Semana 8     │ G + H (windows, polish)
Semana 9+    │ I (SaaS, store, workflows)
```

---

## Paridade Raycast (meta)

| Capacidade | Meta | Hoje (após Fase C) |
|------------|------|---------------------|
| Launcher core | 100% | ~85% |
| Clipboard Pro | 100% | ~40% |
| Extensões | 100% | ~35% |
| AI | 100% | ~20% |
| Window mgmt | 100% | ~50% |
| SaaS/OAuth | 100% | ~5% |
| **Overall** | **100%** | **~55%** |
