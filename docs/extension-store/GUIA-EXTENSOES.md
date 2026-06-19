# Como criar extensões para o Spotlight

## Estrutura

Crie uma pasta em `~/.config/spotlight/extensions/<id>/`:

```
minha-ext/
├── manifest.json
├── search.sh      # opcional — busca
└── run.sh         # opcional — ações
```

## manifest.json

```json
{
  "id": "minha-ext",
  "title": "Minha Extensão",
  "icon": "applications-other",
  "keywords": ["minha", "ext"],
  "enabled": true,
  "search_command": "search.sh",
  "run_command": "run.sh"
}
```

## search.sh

Recebe a query como `$1` e variável `SPOTLIGHT_QUERY`. Deve imprimir JSON:

```json
[
  {
    "title": "Resultado",
    "subtitle": "Descrição",
    "action": "default",
    "icon": "face-smile"
  }
]
```

## run.sh

Recebe `$1` = action id, `$2` = args. Variáveis: `SPOTLIGHT_ACTION`, `SPOTLIGHT_ARGS`.

## Publicar na loja

1. Publique seu repositório no GitHub
2. Abra PR em `docs/extension-store/catalog.json` no repositório Spotlight
3. Formato do item:

```json
{
  "id": "minha-ext",
  "title": "Minha Extensão",
  "description": "O que faz",
  "repo": "seu-usuario/seu-repo",
  "version": "1.0.0"
}
```

## Permissões

Scripts rodam com permissões do usuário. Não inclua secrets no repositório.
